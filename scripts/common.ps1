# Shared process, Cargo and archive validation helpers. Requires PowerShell 7.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$script:RepositoryRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
. (Join-Path $PSScriptRoot 'process-job.ps1')

function Get-FileSystemPath {
    param([string]$Path)
    $provider = $null
    $drive = $null
    $resolved = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path, [ref]$provider, [ref]$drive)
    if ($provider.Name -ne 'FileSystem') { throw "Expected a filesystem path: $Path" }
    [IO.Path]::GetFullPath($resolved)
}

function Invoke-ToolProcess {
    param([Parameter(Mandatory)][string]$Executable, [string[]]$Arguments = @(),
          [int]$TimeoutSeconds = 600, [string]$WorkingDirectory = $script:RepositoryRoot,
          [string]$InputText = '')
    if ($TimeoutSeconds -lt 1) { throw 'Timeout must be positive.' }
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $Executable
    $info.WorkingDirectory = $WorkingDirectory
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.RedirectStandardInput = $true
    foreach ($argument in $Arguments) { $info.ArgumentList.Add($argument) }
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    $started = $false
    $job = [RuxCmd.ProcessJob]::new()
    try {
        if (-not $process.Start()) { throw "Unable to start $Executable" }
        $started = $true
        try { $job.Attach($process.Handle) } catch { if (-not $process.HasExited) { throw } }
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $clock = [Diagnostics.Stopwatch]::StartNew()
        $inputTask = $process.StandardInput.WriteAsync($InputText)
        if (-not $inputTask.Wait($TimeoutSeconds * 1000)) {
            $process.Kill($true)
            $process.WaitForExit()
            throw "Timeout writing input: $Executable"
        }
        $process.StandardInput.Close()
        $remaining = [Math]::Max(0, $TimeoutSeconds * 1000 - [int]$clock.ElapsedMilliseconds)
        if (-not $process.WaitForExit($remaining)) {
            $process.Kill($true)
            $process.WaitForExit()
            throw "Timeout after ${TimeoutSeconds}s: $Executable $($Arguments -join ' ')"
        }
        $remaining = [Math]::Max(0, $TimeoutSeconds * 1000 - [int]$clock.ElapsedMilliseconds)
        $streams = [Threading.Tasks.Task]::WhenAll([Threading.Tasks.Task[]]@($stdout, $stderr))
        if (-not $streams.Wait($remaining)) { throw "Timeout reading output after ${TimeoutSeconds}s: $Executable" }
        [pscustomobject]@{ ExitCode = $process.ExitCode; Output = $stdout.GetAwaiter().GetResult(); Error = $stderr.GetAwaiter().GetResult() }
    } finally {
        $job.Dispose()
        if ($started -and -not $process.HasExited) { $process.Kill($true); $process.WaitForExit() }
        $process.Dispose()
    }
}

function Get-SmokeCases {
    param([string]$PolicyPath, $Workspace)
    $cases = @(Get-Content -LiteralPath $PolicyPath -Raw | ConvertFrom-Json)
    $expected = @($Workspace.Binaries.Name | Sort-Object)
    $names = @($cases.name | Sort-Object)
    if (@($names | Sort-Object -Unique).Count -ne $cases.Count -or @(Compare-Object $expected $names).Count -ne 0) { throw 'Smoke strategies must cover every workspace binary exactly once.' }
    $cases
}

function Assert-ToolSuccess {
    param($Result, [string]$Label)
    if ($Result.ExitCode -ne 0) { throw "$Label exited $($Result.ExitCode).`n$($Result.Error)`n$($Result.Output)" }
}

function Get-WorkspaceInfo {
    $result = Invoke-ToolProcess cargo @('metadata', '--no-deps', '--format-version', '1', '--locked')
    Assert-ToolSuccess $result 'cargo metadata'
    $metadata = $result.Output | ConvertFrom-Json
    $bins = @(
        foreach ($package in $metadata.packages) {
            if ($package.id -notin $metadata.workspace_members) { continue }
            foreach ($target in $package.targets) {
                if ('bin' -in $target.kind) {
                    [pscustomobject]@{ Name = $target.name; PackageId = $package.id; PackageVersion = $package.version }
                }
            }
        }
    )
    if ($bins.Count -eq 0 -or @($bins.Name | Sort-Object -Unique).Count -ne $bins.Count) { throw 'No binaries or duplicate binary names in workspace.' }
    $suite = $metadata.metadata.ruxcmd
    [pscustomobject]@{ Version = $suite.version; Target = $suite.target; Binaries = @($bins | Sort-Object Name) }
}

function Assert-ReleaseVersion {
    param([string]$Version, [string]$ExpectedTag)
    if ($Version -notmatch '^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$') { throw "Invalid suite version: $Version" }
    if ($ExpectedTag -and $ExpectedTag -cne "v$Version") { throw "Tag $ExpectedTag does not match suite version v$Version" }
}

function Get-BinaryArtifacts {
    param([string]$Messages, $Workspace)
    $artifacts = @{}
    foreach ($line in ($Messages -split '\r?\n')) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        $message = $line | ConvertFrom-Json
        if ($message.reason -ne 'compiler-artifact') { continue }
        if ('bin' -notin $message.target.kind -or -not $message.executable) { continue }
        $expected = @($Workspace.Binaries | Where-Object { $_.PackageId -eq $message.package_id -and $_.Name -ceq $message.target.name })
        if ($expected.Count -ne 1) { throw "Unexpected binary artifact: $($message.target.name)" }
        $name = $expected[0].Name
        if ($artifacts.ContainsKey($name)) { throw "Duplicate artifact: $name" }
        if (-not (Test-Path -LiteralPath $message.executable -PathType Leaf)) { throw "Missing artifact: $name" }
        if ([IO.Path]::GetFileName($message.executable) -cne "$name.exe") { throw "Incorrect executable name: $name" }
        $artifacts[$name] = $message.executable
    }
    foreach ($binary in $Workspace.Binaries) {
        if (-not $artifacts.ContainsKey($binary.Name)) { throw "Missing artifact: $($binary.Name)" }
    }
    $artifacts
}

function Get-FileDigest {
    param([string]$Path)
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Assert-ArchiveDirectory {
    param([string]$Directory, $Manifest)
    $expected = @('manifest.json', 'SHA256SUMS') + @($Manifest.files.path)
    if (@($expected | Sort-Object -Unique).Count -ne $expected.Count) { throw 'Duplicate archive file entries.' }
    foreach ($file in $Manifest.files) {
        if ($file.path -match '(^/|\\|(^|/)\.\.(/|$)|:)' ) { throw "Unsafe manifest path: $($file.path)" }
        $path = Join-Path $Directory $file.path
        if (-not (Test-Path -LiteralPath $path -PathType Leaf) -or (Get-FileDigest $path) -cne $file.sha256) { throw "Archive checksum mismatch: $($file.path)" }
    }
    $actual = @(Get-ChildItem -LiteralPath $Directory -File -Recurse | ForEach-Object { [IO.Path]::GetRelativePath($Directory, $_.FullName).Replace('\', '/') })
    if (@(Compare-Object ($expected | Sort-Object) ($actual | Sort-Object)).Count -ne 0) { throw 'Archive file set differs from manifest.' }
    $checksums = @($Manifest.files | ForEach-Object { "$($_.sha256)  $($_.path)" }) -join "`n"
    if ((Get-Content -LiteralPath (Join-Path $Directory 'SHA256SUMS') -Raw).TrimEnd() -cne $checksums) { throw 'SHA256SUMS differs from manifest.' }
    foreach ($binary in $Manifest.binaries) {
        $matching = @($Manifest.files | Where-Object { $_.path -ceq $binary.file -and $_.sha256 -ceq $binary.sha256 })
        if ($matching.Count -ne 1) { throw "Binary checksum entry missing: $($binary.name)" }
    }
}

function Remove-OwnedDirectory {
    param([string]$Path, [string]$Parent)
    $resolved = [IO.Path]::GetFullPath($Path)
    $parentResolved = [IO.Path]::GetFullPath($Parent).TrimEnd([IO.Path]::DirectorySeparatorChar)
    if (-not $resolved.StartsWith($parentResolved + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) { throw "Refusing cleanup outside parent: $resolved" }
    if (Test-Path -LiteralPath $resolved) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}

function Get-NativeDependencies {
    param([string]$Executable, [string]$Dumpbin)
    $result = Invoke-ToolProcess $Dumpbin @('/DEPENDENTS', $Executable)
    Assert-ToolSuccess $result 'dumpbin'
    $dlls = @([regex]::Matches($result.Output, '(?im)^\s+([a-z0-9_.-]+\.dll)\s*$') | ForEach-Object { $_.Groups[1].Value.ToLowerInvariant() } | Sort-Object -Unique)
    if ($dlls.Count -eq 0) { throw "No PE dependencies reported for $Executable" }
    foreach ($dll in $dlls) {
        if ($dll -match '^(api-ms-win-|ext-ms-win-)') { continue }
        if ($dll -match '^(vcruntime|msvcp|libssl|libcrypto)' -or -not (Test-Path -LiteralPath (Join-Path $env:SystemRoot "System32/$dll"))) { throw "Non-system DLL dependency $dll in $Executable; static CRT alone is insufficient." }
    }
    $dlls
}

function Find-Dumpbin {
    $command = Get-Command dumpbin.exe -ErrorAction SilentlyContinue
    if ($command) { return $command.Source }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) { throw 'Install MSVC Build Tools with the C++ workload; dumpbin/vswhere is missing.' }
    $result = Invoke-ToolProcess $vswhere @('-latest', '-products', '*', '-requires', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64', '-find', 'VC\Tools\MSVC\**\bin\Hostx64\x64\dumpbin.exe')
    Assert-ToolSuccess $result 'vswhere'
    $paths = @($result.Output -split '\r?\n' | Where-Object { $_ -and (Test-Path -LiteralPath $_ -PathType Leaf) })
    if ($paths.Count -eq 0) { throw 'MSVC dumpbin not found.' }
    $paths[0]
}

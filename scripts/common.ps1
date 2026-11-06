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
    # Cargo and the tools emit UTF-8; without this the .NET defaults decode
    # with the console codepage (e.g. GBK), corrupting non-ASCII descriptions.
    $info.StandardOutputEncoding = [Text.Encoding]::UTF8
    $info.StandardErrorEncoding = [Text.Encoding]::UTF8
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
    param([switch]$Offline)
    $metadataArguments = @('metadata', '--no-deps', '--format-version', '1', '--locked')
    if ($Offline) { $metadataArguments += '--offline' }
    $result = Invoke-ToolProcess cargo $metadataArguments
    Assert-ToolSuccess $result 'cargo metadata'
    $metadata = $result.Output | ConvertFrom-Json
    $bins = @(
        foreach ($package in $metadata.packages) {
            if ($package.id -notin $metadata.workspace_members) { continue }
            foreach ($target in $package.targets) {
                if ('bin' -in $target.kind) {
                    [pscustomobject]@{ Name = $target.name; PackageName = $package.name; PackageId = $package.id; PackageVersion = $package.version; Description = $package.description }
                }
            }
        }
    )
    if ($bins.Count -eq 0 -or @($bins.Name | Sort-Object -Unique).Count -ne $bins.Count) { throw 'No binaries or duplicate binary names in workspace.' }
    $suite = $metadata.metadata.ruxcmd
    $windows = if ($suite.PSObject.Properties['windows']) { $suite.windows } else { $null }
    [pscustomobject]@{ Version = $suite.version; Target = $suite.target; Windows = $windows; TargetDirectory = $metadata.target_directory; Packages = @($metadata.packages | Where-Object { $_.id -in $metadata.workspace_members } | ForEach-Object name); Binaries = @($bins | Sort-Object Name) }
}

function Assert-BinaryVersionInfo {
    param([string]$Path, $Binary, $Workspace, [switch]$DebugBuild)
    if (-not $Workspace.Windows) { throw 'Missing workspace Windows version metadata.' }
    $numeric = ($Workspace.Version -split '[-+]', 2)[0] + '.0'
    $expected = [ordered]@{
        OriginalFilename = "$($Binary.Name).exe"
        InternalName = $Binary.Name
        ProductName = $Workspace.Windows.productName
        ProductVersion = $Workspace.Version
        FileVersion = $numeric
        CompanyName = $Workspace.Windows.companyName
        LegalCopyright = $Workspace.Windows.copyright
        FileDescription = $Binary.Description
    }
    $info = [Diagnostics.FileVersionInfo]::GetVersionInfo((Get-FileSystemPath $Path))
    foreach ($field in $expected.Keys) {
        if ([string]::IsNullOrWhiteSpace($expected[$field]) -or $info.$field -cne $expected[$field]) {
            throw "Version resource mismatch in ${Path}: $field (expected '$($expected[$field])', got '$($info.$field)')."
        }
    }
    $fileWords = @($info.FileMajorPart, $info.FileMinorPart, $info.FileBuildPart, $info.FilePrivatePart) -join '.'
    $productWords = @($info.ProductMajorPart, $info.ProductMinorPart, $info.ProductBuildPart, $info.ProductPrivatePart) -join '.'
    if ($fileWords -cne $numeric -or $productWords -cne $numeric) { throw "Version resource fixed numeric version mismatch in $Path" }
    $isPrerelease = (($Workspace.Version -split '\+', 2)[0]).Contains('-')
    if ($info.IsDebug -ne [bool]$DebugBuild -or $info.IsPreRelease -ne $isPrerelease) { throw "Version resource build flags mismatch in $Path" }
    [pscustomobject]$expected
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

# Commands that clash with built-in shells/aliases get rux-* shims. The EXE
# installer and install.ps1 must generate exactly the same names and bytes.
function Get-RuxCmdPrefixShimCommands {
    @(
        'find', 'sort', 'where',
        'cat', 'ls', 'cp', 'mv', 'rm', 'echo', 'pwd', 'ps', 'kill', 'sleep', 'tee', 'clear', 'curl', 'wget'
    )
}

function Get-RuxCmdShimContent {
    param([string]$Command)
    "@echo off`r`n`"%~dp0$Command.exe`" %*`r`n"
}

# ISCC.exe carries no version resource (ProductVersion is 0.0.0.0), so the
# installed version is read from the Inno Setup uninstall registration.
function Get-InnoSetupVersion {
    param([string]$IsccPath)
    $directory = [IO.Path]::TrimEndingDirectorySeparator([IO.Path]::GetFullPath([IO.Path]::GetDirectoryName($IsccPath)))
    foreach ($key in @(
            'HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*',
            'HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*',
            'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*')) {
        foreach ($item in @(Get-ItemProperty -Path $key -ErrorAction SilentlyContinue)) {
            $nameProperty = $item.PSObject.Properties['DisplayName']
            $versionProperty = $item.PSObject.Properties['DisplayVersion']
            $locationProperty = $item.PSObject.Properties['InstallLocation']
            if (-not $nameProperty -or -not $versionProperty -or -not $locationProperty) { continue }
            if ($nameProperty.Value -notlike 'Inno Setup*' -or [string]::IsNullOrWhiteSpace([string]$versionProperty.Value)) { continue }
            $location = [string]$locationProperty.Value
            if ([string]::IsNullOrWhiteSpace($location)) { continue }
            $resolved = [IO.Path]::TrimEndingDirectorySeparator([IO.Path]::GetFullPath($location))
            if ([string]::Equals($resolved, $directory, [StringComparison]::OrdinalIgnoreCase)) { return [string]$versionProperty.Value }
        }
    }
    return ''
}

# Only one Inno Setup major is supported at a time. RuxCmd pins the major
# version in toolchain.json, so an unexpected major fails instead of silently
# changing the generated installer semantics.
function Get-InnoSetupMajorVersion {
    param([string]$Version)
    if ($Version -match '^(\d+)\.(\d+)\.(\d+)') { return [int]$Matches[1] }
    return -1
}

function Assert-InnoSetupVersion {
    param([string]$Version, [int]$SupportedMajor, [string]$Path, [string]$DownloadUrl)
    $major = Get-InnoSetupMajorVersion $Version
    if ($major -ne $SupportedMajor) {
        $found = if ($Version) { $Version } else { 'unknown' }
        throw "Inno Setup '$found' at $Path is not supported: RuxCmd requires Inno Setup $SupportedMajor.x; other majors are rejected. Install it from $DownloadUrl or pass -Iscc <path>."
    }
    Write-Host "Inno Setup compiler: $Version at $Path"
}

function Find-InnoSetupCompiler {
    param([int]$SupportedMajor = 0)
    $candidates = @()
    $command = Get-Command ISCC.exe -ErrorAction SilentlyContinue
    if ($command) { $candidates += $command.Source }
    foreach ($root in @(${env:ProgramFiles(x86)}, $env:ProgramFiles)) {
        if ([string]::IsNullOrWhiteSpace($root)) { continue }
        foreach ($edition in @('Inno Setup 6', 'Inno Setup 7')) {
            $candidates += (Join-Path $root "$edition\ISCC.exe")
        }
    }
    $first = $null
    foreach ($candidate in $candidates) {
        if (-not $candidate -or -not (Test-Path -LiteralPath $candidate -PathType Leaf)) { continue }
        $resolved = Get-FileSystemPath $candidate
        if (-not $first) { $first = $resolved }
        if ($SupportedMajor -le 0) { return $resolved }
        if ((Get-InnoSetupMajorVersion (Get-InnoSetupVersion $resolved)) -eq $SupportedMajor) { return $resolved }
    }
    # No supported compiler was found; return the first existing one so the
    # caller can report the exact version it rejected.
    return $first
}

function Expand-ReleaseArchive {
    param([string]$ArchivePath, [string]$Destination)
    $root = [IO.Path]::GetFullPath($Destination)
    [IO.Directory]::CreateDirectory($root) | Out-Null
    $prefix = [IO.Path]::TrimEndingDirectorySeparator($root) + [IO.Path]::DirectorySeparatorChar
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $zip = [IO.Compression.ZipFile]::OpenRead($ArchivePath)
    try {
        foreach ($entry in $zip.Entries) {
            $relative = $entry.FullName.Replace('\', '/')
            if ([string]::IsNullOrWhiteSpace($relative)) { continue }
            if ($relative -match '(^/|^[A-Za-z]:|(^|/)\.\.(/|$))') { throw "Unsafe archive entry: $($entry.FullName)" }
            if (-not $seen.Add($relative)) { throw "Duplicate archive entry: $($entry.FullName)" }
            $parts = $relative.Split('/') | Where-Object { $_ -ne '' }
            $target = [IO.Path]::GetFullPath((Join-Path $root ($parts -join [IO.Path]::DirectorySeparatorChar)))
            if (-not $target.StartsWith($prefix, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Archive entry escapes the destination: $($entry.FullName)"
            }
            if ($relative.EndsWith('/')) {
                [IO.Directory]::CreateDirectory($target) | Out-Null
                continue
            }
            [IO.Directory]::CreateDirectory([IO.Path]::GetDirectoryName($target)) | Out-Null
            [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $target, $true)
        }
    } finally {
        $zip.Dispose()
    }
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

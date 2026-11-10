#Requires -Version 7.0
<#
.SYNOPSIS
Build the standalone RuxCmd EXE installer from an existing release ZIP.

.DESCRIPTION
Verifies the release ZIP checksum and manifest, extracts it into a private
temporary directory, regenerates the rux-* shims from the manifest, generates
the Inno Setup [Files] list, compiles the installer with an Inno Setup 7.x
compiler, and writes the setup EXE plus its SHA256 sidecar into -OutputDir.

The installer packages exactly the executables already present in the ZIP.
No command list is maintained here; the manifest is the source of truth. The
142 binary version resources are checked again before compiling.

Inno Setup is not installed by this script. The pinned version and download
URL live in scripts/packaging/inno/toolchain.json. The compiler is located
through PATH, then the standard "Inno Setup 6" and "Inno Setup 7" directories;
its version is read from the Inno Setup uninstall registration because
ISCC.exe has no version resource of its own. The major version must match
toolchain.json; other majors are rejected.

.PARAMETER Archive
Release ZIP produced by scripts/release.ps1 or scripts/package.ps1.

.PARAMETER Checksum
Sidecar containing "<sha256>  <filename>". Defaults to <Archive>.sha256.

.PARAMETER OutputDir
Directory for the setup EXE and its .sha256 sidecar. Defaults to dist/.

.PARAMETER Iscc
Path to ISCC.exe. Defaults to PATH and the standard Inno Setup installation
directories.

.EXAMPLE
./scripts/package-installer.ps1 -Archive dist/ruxcmd-0.4.4-x86_64-pc-windows-msvc.zip
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Archive,
    [string]$Checksum,
    [string]$OutputDir,
    [string]$Iscc
)

. (Join-Path $PSScriptRoot 'common.ps1')

function Resolve-IsccExecutable {
    param([string]$Path, $Toolchain, [int]$SupportedMajor)
    $candidate = if ($Path) { Get-FileSystemPath $Path } else { Find-InnoSetupCompiler -SupportedMajor $SupportedMajor }
    if (-not $candidate) {
        throw "Inno Setup $SupportedMajor.x was not found. Install it from $($Toolchain.downloadUrl) or pass -Iscc <path>."
    }
    if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) { throw "ISCC.exe not found: $candidate" }
    Assert-InnoSetupVersion -Version (Get-InnoSetupVersion $candidate) -SupportedMajor $SupportedMajor -Path $candidate -DownloadUrl $Toolchain.downloadUrl
    $candidate
}

if (-not $IsWindows) { throw 'RuxCmd installer packaging requires Windows.' }
$workspace = Get-WorkspaceInfo
Assert-ReleaseVersion $workspace.Version
if (-not $workspace.Windows) { throw 'Missing workspace Windows version metadata.' }

$toolchainPath = Join-Path $PSScriptRoot 'packaging/inno/toolchain.json'
$toolchain = Get-Content -LiteralPath $toolchainPath -Raw | ConvertFrom-Json
$innoVersion = [string]$toolchain.innoSetupVersion
if ($innoVersion -notmatch '^\d+\.\d+\.\d+$') { throw "Invalid pinned Inno Setup version: $innoVersion" }
$supportedMajor = [int]$toolchain.supportedMajorVersion
if ($supportedMajor -ne (Get-InnoSetupMajorVersion $innoVersion)) {
    throw "toolchain.json supportedMajorVersion $supportedMajor does not match the pinned version $innoVersion."
}

$Archive = Get-FileSystemPath $Archive
if (-not (Test-Path -LiteralPath $Archive -PathType Leaf)) { throw "Release ZIP not found: $Archive" }
if (-not $Checksum) { $Checksum = "$Archive.sha256" }
$Checksum = Get-FileSystemPath $Checksum
if (-not (Test-Path -LiteralPath $Checksum -PathType Leaf)) { throw "Release checksum not found: $Checksum" }
$expectedHash = (Get-Content -LiteralPath $Checksum -Raw).Trim()
if ($expectedHash -notmatch '^([0-9A-Fa-f]{64})(?:\s+\*?.+)?$') { throw "Malformed checksum file: $Checksum" }
$expectedHash = $Matches[1].ToLowerInvariant()
if ((Get-FileDigest $Archive) -cne $expectedHash) { throw "Release ZIP checksum mismatch: $Archive" }

if (-not $OutputDir) { $OutputDir = Join-Path $script:RepositoryRoot 'dist' }
$OutputDir = Get-FileSystemPath $OutputDir
$outputBaseFilename = "ruxcmd-$($workspace.Version)-$($workspace.Target)-setup"
$setupPath = Join-Path $OutputDir "$outputBaseFilename.exe"
$setupChecksumPath = "$setupPath.sha256"
if ((Test-Path -LiteralPath $setupPath) -or (Test-Path -LiteralPath $setupChecksumPath)) {
    throw 'Installer output already exists; use a different OutputDir or remove the old installer first.'
}

# Fail fast before the expensive extraction and compilation.
$isccPath = Resolve-IsccExecutable -Path $Iscc -Toolchain $toolchain -SupportedMajor $supportedMajor

$zip = [IO.Compression.ZipFile]::OpenRead($Archive)
try {
    $entry = $zip.GetEntry('manifest.json')
    if (-not $entry) { throw 'Release ZIP is missing manifest.json.' }
    $reader = [IO.StreamReader]::new($entry.Open())
    try { $manifest = $reader.ReadToEnd() | ConvertFrom-Json } finally { $reader.Dispose() }
} finally { $zip.Dispose() }

if ([int]$manifest.schemaVersion -ne 1) { throw "Unsupported release manifest schema: $($manifest.schemaVersion)" }
if ([string]$manifest.suiteVersion -cne $workspace.Version) { throw "Release manifest version $($manifest.suiteVersion) does not match workspace version $($workspace.Version)." }
if ([string]$manifest.target -cne $workspace.Target) { throw "Release manifest target $($manifest.target) does not match $($workspace.Target)." }
$head = Invoke-ToolProcess git @('rev-parse', 'HEAD')
Assert-ToolSuccess $head 'git HEAD'
if ([string]$manifest.sourceCommit -cne $head.Output.Trim()) { throw 'Release ZIP source commit does not match HEAD; rebuild the release ZIP first.' }
if ([bool]$manifest.dirty) { Write-Warning 'Release ZIP was built from a dirty checkout.' }

$manifestBinaries = @($manifest.binaries)
$workspaceBinaries = @($workspace.Binaries)
if ($manifestBinaries.Count -ne $workspaceBinaries.Count) {
    throw "Release ZIP has $($manifestBinaries.Count) binaries but the workspace has $($workspaceBinaries.Count)."
}
$manifestNames = @($manifestBinaries.name | Sort-Object)
$workspaceNames = @($workspaceBinaries.Name | Sort-Object)
if (@(Compare-Object $manifestNames $workspaceNames).Count -ne 0) {
    throw 'Release ZIP binaries differ from the workspace binary list.'
}

$owned = Join-Path ([IO.Path]::GetTempPath()) "ruxcmd-installer-$([guid]::NewGuid().ToString('N'))"
$content = Join-Path $owned 'content'
$build = Join-Path $owned 'build'
$privateOutput = Join-Path $owned 'output'
$numericVersion = ($workspace.Version -split '[-+]', 2)[0] + '.0'
$createdSetup = $false

try {
    [IO.Directory]::CreateDirectory($content) | Out-Null
    [IO.Directory]::CreateDirectory($build) | Out-Null
    [IO.Directory]::CreateDirectory($privateOutput) | Out-Null
    Expand-ReleaseArchive $Archive $content
    Assert-ArchiveDirectory $content $manifest

    $workspaceByName = @{}
    foreach ($binary in $workspaceBinaries) { $workspaceByName[$binary.Name] = $binary }
    foreach ($binary in $manifestBinaries) {
        $name = [string]$binary.name
        if (-not $workspaceByName.ContainsKey($name)) { throw "Release ZIP contains an unexpected binary: $name" }
        $binaryPath = Join-Path $content ([string]$binary.file).Replace('/', '\')
        Assert-BinaryVersionInfo -Path $binaryPath -Binary $workspaceByName[$name] -Workspace $workspace | Out-Null
    }

    $desktop = $manifest.desktop
    if (-not $desktop) { throw 'Release ZIP manifest has no desktop entry.' }
    if ([string]$desktop.file -cne 'ruxcmd-desktop.exe') {
        throw "Release ZIP desktop file is '$($desktop.file)', expected 'ruxcmd-desktop.exe'."
    }
    if (@($manifest.files | Where-Object { [string]$_.path -ceq [string]$desktop.file }).Count -ne 1) {
        throw 'Release ZIP desktop file is not listed exactly once in manifest.files.'
    }
    Assert-DesktopVersionInfo (Join-Path $content 'ruxcmd-desktop.exe') $workspace | Out-Null

    # Regenerate the same shims install.ps1 writes, using the shared policy.
    $shims = @()
    foreach ($command in @(Get-RuxCmdPrefixShimCommands)) {
        if (-not ($manifestNames -contains $command)) { continue }
        $shimName = "rux-$command.cmd"
        [IO.File]::WriteAllText((Join-Path $content $shimName), (Get-RuxCmdShimContent $command), [Text.UTF8Encoding]::new($false))
        $shims += $shimName
    }

    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'packaging/inno/RuxCmd.iss') -Destination (Join-Path $build 'RuxCmd.iss')

    $buildInfo = @"
#define AppVersion "$($workspace.Version)"
#define VersionInfoVersion "$numericVersion"
#define AppProductName "$($workspace.Windows.productName)"
#define AppPublisher "$($workspace.Windows.companyName)"
#define AppCopyright "$($workspace.Windows.copyright)"
#define OutputDir "$privateOutput"
#define OutputBaseFilename "$outputBaseFilename"
"@
    [IO.File]::WriteAllText((Join-Path $build 'build-info.iss'), $buildInfo, [Text.UTF8Encoding]::new($false))

    $fileLines = [Collections.Generic.List[string]]::new()
    $fileLines.Add('[Files]')
    foreach ($binary in $manifestBinaries) {
        $source = Join-Path $content ([string]$binary.file).Replace('/', '\')
        $fileLines.Add("Source: `"$source`"; DestDir: `"{app}`"; Flags: ignoreversion")
    }
    $desktopSource = Join-Path $content 'ruxcmd-desktop.exe'
    # The UI is not a command: install it one level above {app} (the PATH
    # directory), next to the bin folder.
    $fileLines.Add("Source: `"$desktopSource`"; DestDir: `"{app}\..`"; Components: desktop; Flags: ignoreversion")
    foreach ($shim in $shims) {
        $source = Join-Path $content $shim
        $fileLines.Add("Source: `"$source`"; DestDir: `"{app}`"; Flags: ignoreversion")
    }
    # Everything outside bin/ is documentation, except the desktop UI which is
    # installed to {app} rather than {app}\share.
    $docs = @($manifest.files | Where-Object {
        $_.path -notlike 'bin/*' -and [string]$_.path -cne [string]$desktop.file
    })
    foreach ($doc in $docs) {
        $source = Join-Path $content ([string]$doc.path).Replace('/', '\')
        $fileLines.Add("Source: `"$source`"; DestDir: `"{app}\share`"; Flags: ignoreversion")
    }
    foreach ($extra in @('manifest.json', 'SHA256SUMS')) {
        $source = Join-Path $content $extra
        $fileLines.Add("Source: `"$source`"; DestDir: `"{app}\share`"; Flags: ignoreversion")
    }
    $expectedFileEntries = $manifestBinaries.Count + 1 + $shims.Count + $docs.Count + 2
    $actualFileEntries = @($fileLines | Where-Object { $_ -like 'Source:*' }).Count
    if ($actualFileEntries -ne $expectedFileEntries) {
        throw "Generated [Files] count $actualFileEntries does not match $expectedFileEntries."
    }
    [IO.File]::WriteAllText((Join-Path $build 'generated-files.iss'), ($fileLines -join "`r`n") + "`r`n", [Text.UTF8Encoding]::new($false))

    $clock = [Diagnostics.Stopwatch]::StartNew()
    $compile = Invoke-ToolProcess $isccPath @((Join-Path $build 'RuxCmd.iss')) -TimeoutSeconds 1800
    Assert-ToolSuccess $compile 'Inno Setup compiler'
    Write-Host $compile.Output.Trim()
    Write-Host "Inno Setup compile finished in $([Math]::Round($clock.Elapsed.TotalSeconds, 1))s."

    $privateSetup = Join-Path $privateOutput "$outputBaseFilename.exe"
    if (-not (Test-Path -LiteralPath $privateSetup -PathType Leaf)) { throw "Inno Setup did not produce $outputBaseFilename.exe." }
    $setupInfo = [Diagnostics.FileVersionInfo]::GetVersionInfo($privateSetup)
    $expectedInfo = [ordered]@{
        ProductName = $workspace.Windows.productName
        ProductVersion = $workspace.Version
        FileVersion = $numericVersion
        CompanyName = $workspace.Windows.companyName
        LegalCopyright = $workspace.Windows.copyright
        OriginalFilename = "$outputBaseFilename.exe"
    }
    foreach ($field in $expectedInfo.Keys) {
        # Inno Setup 7 pads version-info strings to fixed widths, so compare
        # trimmed values instead of rejecting harmless trailing spaces.
        $actual = ([string]$setupInfo.$field).Trim()
        $expected = ([string]$expectedInfo[$field]).Trim()
        if ($actual -cne $expected) {
            throw "Installer version resource mismatch: $field (expected '$expected', got '$actual')."
        }
    }

    New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null
    [IO.File]::Copy($privateSetup, $setupPath, $false)
    $createdSetup = $true
    $setupHash = Get-FileDigest $setupPath
    "$setupHash  $($outputBaseFilename).exe" | Set-Content -LiteralPath $setupChecksumPath -Encoding utf8NoBOM

    Write-Host "Installer : $setupPath"
    Write-Host "  version : $($workspace.Version) (file version $numericVersion)"
    Write-Host "  binaries: $($manifestBinaries.Count)"
    Write-Host "  shims   : $($shims.Count)"
    Write-Host "  docs    : $($docs.Count + 2)"
    Write-Host "SHA256    : $setupChecksumPath"
} catch {
    if ($createdSetup -and (Test-Path -LiteralPath $setupPath)) { Remove-Item -LiteralPath $setupPath -Force }
    if (Test-Path -LiteralPath $setupChecksumPath) { Remove-Item -LiteralPath $setupChecksumPath -Force }
    throw
} finally {
    Remove-OwnedDirectory $owned ([IO.Path]::GetTempPath())
}

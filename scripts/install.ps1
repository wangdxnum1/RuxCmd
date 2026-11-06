#Requires -Version 7.0
<#
.SYNOPSIS
Install, upgrade or remove a RuxCmd release package for the current user.

.DESCRIPTION
Verifies the release ZIP SHA256, extracts it, installs the executables from
bin/ directly into -InstallDir, and copies documentation and the build
manifest into <InstallDir>\share. A small state file records the version,
checksum, executables and generated shims.

Windows reserves several command names (find, sort, where) and PowerShell
defines aliases for others (cat, ls, cp, ...). Because the machine PATH
precedes the user PATH, bare names always resolve to the Windows tools or the
PowerShell alias. The installer therefore writes rux-<command>.cmd shims for
those names. Bare executables still exist in -InstallDir; use them by full
path or when no conflict applies.

Only the user-level PATH is modified, and only after an interactive
confirmation (skip with -Force). The machine PATH and other registry values
are never touched. Set RUXCMD_INSTALL_USER_PATH_FILE to a file path to read
and write the user PATH from that file instead; the test suite uses this seam
so it never touches a real profile.

This script and the standalone EXE installer are mutually exclusive: if the
packaged installation is present, this script refuses to run. Uninstall the
packaged version from Settings > Apps first.

.PARAMETER Source
Path to a release ZIP (with its .sha256 sidecar) or a version such as 0.4.3
or v0.4.3, which is downloaded from the GitHub Release.

.PARAMETER InstallDir
Directory that receives the executables. Default:
%LOCALAPPDATA%\RuxCmd\bin, the same default as the EXE installer's
"only for me" mode.

.PARAMETER Sha256
Expected SHA256 of the ZIP. Overrides the .sha256 sidecar.

.PARAMETER Force
Accept user PATH changes without prompting, and reinstall even when the same
version and checksum are already present.

.PARAMETER Uninstall
Remove the recorded shims, the user PATH entry and (with -RemoveInstallDir) the
recorded files. No source archive is needed.

.PARAMETER RemoveInstallDir
With -Uninstall, delete only the files recorded by this installer, then remove
the directory if nothing else remains.

.EXAMPLE
pwsh -File scripts/install.ps1 -Source .\ruxcmd-0.4.3-x86_64-pc-windows-msvc.zip -Force

.EXAMPLE
pwsh -File scripts/install.ps1 -Source 0.4.3

.EXAMPLE
pwsh -File scripts/install.ps1 -Uninstall -Force
#>
[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$Source,
    [string]$InstallDir = '',
    [string]$Sha256,
    [switch]$Force,
    [switch]$Uninstall,
    [switch]$RemoveInstallDir
)

. (Join-Path $PSScriptRoot 'common.ps1')

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$script:ProductName = 'RuxCmd'
$script:ReleaseTarget = 'x86_64-pc-windows-msvc'
$script:ReleaseRepository = 'wangdxnum1/RuxCmd'
$script:StateFileName = '.ruxcmd-install.json'
$script:UserPathOverrideVariable = 'RUXCMD_INSTALL_USER_PATH_FILE'
$script:PackagedAppId = '{06BD23F3-4290-4F15-AEC6-55FD9DC91B9F}'
$script:PrefixShimCommands = @(Get-RuxCmdPrefixShimCommands)

if ([string]::IsNullOrWhiteSpace($InstallDir)) {
    $localAppData = [Environment]::GetFolderPath([Environment+SpecialFolder]::LocalApplicationData)
    if ([string]::IsNullOrWhiteSpace($localAppData)) { throw 'Unable to resolve %LOCALAPPDATA% for the default install directory.' }
    $InstallDir = Join-Path $localAppData 'RuxCmd\bin'
}

function Resolve-InstallDirectory {
    param([string]$Path)
    if ([string]::IsNullOrWhiteSpace($Path)) { throw 'Install directory must not be empty.' }
    $provider = $null
    $drive = $null
    $resolved = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Path, [ref]$provider, [ref]$drive)
    if ($provider.Name -ne 'FileSystem') { throw "Install directory must be a filesystem path: $Path" }
    [IO.Path]::TrimEndingDirectorySeparator([IO.Path]::GetFullPath($resolved))
}

function Get-Sha256Hex {
    param([string]$Path)
    (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Write-TextIfChanged {
    param([string]$Path, [string]$Content)
    if ((Test-Path -LiteralPath $Path -PathType Leaf) -and [IO.File]::ReadAllText($Path) -ceq $Content) {
        return $false
    }
    $directory = [IO.Path]::GetDirectoryName($Path)
    if ($directory) { [IO.Directory]::CreateDirectory($directory) | Out-Null }
    [IO.File]::WriteAllText($Path, $Content, [Text.UTF8Encoding]::new($false))
    return $true
}

function Get-StateField {
    param($State, [string]$Name, $Default = $null)
    if ($null -eq $State) { return $Default }
    $property = $State.PSObject.Properties[$Name]
    if ($null -eq $property) { return $Default }
    return $property.Value
}

function Read-InstallState {
    param([string]$Directory)
    $path = Join-Path $Directory $script:StateFileName
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { return $null }
    try {
        Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
    } catch {
        throw "Unreadable install state: $path. Remove it and reinstall, or run -Uninstall."
    }
}

function Write-InstallState {
    param([string]$Directory, [System.Collections.Specialized.OrderedDictionary]$State)
    $path = Join-Path $Directory $script:StateFileName
    $json = ($State | ConvertTo-Json -Depth 6) + "`n"
    Write-TextIfChanged $path $json | Out-Null
}

function Read-PackageManifest {
    param([string]$Directory)
    $path = Join-Path $Directory 'manifest.json'
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { return $null }
    Get-Content -LiteralPath $path -Raw | ConvertFrom-Json
}

function Get-PackageVersion {
    param($Manifest)
    $value = [string](Get-StateField $Manifest 'suiteVersion' '')
    if ([string]::IsNullOrWhiteSpace($value)) {
        Write-Warning "The package has no manifest.json suiteVersion; recording the version as unknown."
        return 'unknown'
    }
    return $value
}

function Get-PackageBinaries {
    param([string]$Directory)
    $binDir = Join-Path $Directory 'bin'
    if (-not (Test-Path -LiteralPath $binDir -PathType Container)) { throw 'The package has no bin/ directory.' }
    $binaries = @{}
    foreach ($file in @(Get-ChildItem -LiteralPath $binDir -File -Filter '*.exe' | Sort-Object Name)) {
        $name = [IO.Path]::GetFileNameWithoutExtension($file.Name)
        if ($binaries.ContainsKey($name)) { throw "Duplicate executable name in the package: $name" }
        $binaries[$name] = $file.FullName
    }
    if ($binaries.Count -eq 0) { throw 'The package contains no executables under bin/.' }
    return $binaries
}

function Read-ExpectedHash {
    param([string]$ArchivePath, [string]$Explicit)
    if (-not [string]::IsNullOrWhiteSpace($Explicit)) {
        if ($Explicit -notmatch '^[0-9A-Fa-f]{64}$') { throw '-Sha256 must be exactly 64 hexadecimal characters.' }
        return $Explicit.ToLowerInvariant()
    }
    $sidecar = "$ArchivePath.sha256"
    if (-not (Test-Path -LiteralPath $sidecar -PathType Leaf)) {
        throw "No expected checksum: pass -Sha256 or place $([IO.Path]::GetFileName($sidecar)) next to the archive."
    }
    $text = [IO.File]::ReadAllText($sidecar).Trim()
    if ($text -notmatch '^([0-9A-Fa-f]{64})(?:\s+\*?.+)?$') { throw "Unrecognized checksum file: $sidecar" }
    return $Matches[1].ToLowerInvariant()
}

function Test-SupportedVersion {
    param([string]$Value)
    return $Value -match '^[vV]?\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$'
}

function Save-ReleaseArchive {
    param([string]$Version, [string]$Directory)
    $normalized = $Version -replace '^[vV]', ''
    $name = "ruxcmd-$normalized-$($script:ReleaseTarget)"
    $base = "https://github.com/$($script:ReleaseRepository)/releases/download/v$normalized/$name"
    $archive = Join-Path $Directory "$name.zip"
    Write-Host "Downloading $base.zip"
    try {
        Invoke-WebRequest -Uri "$base.zip" -OutFile $archive
        Invoke-WebRequest -Uri "$base.zip.sha256" -OutFile "$archive.sha256"
    } catch {
        throw "Failed to download RuxCmd $normalized from GitHub: $($_.Exception.Message)"
    }
    return $archive
}

function Assert-ArchiveContents {
    param([string]$Directory)
    $index = Join-Path $Directory 'SHA256SUMS'
    if (-not (Test-Path -LiteralPath $index -PathType Leaf)) {
        Write-Host 'The package has no SHA256SUMS; skipping per-file verification.'
        return
    }
    foreach ($line in @(Get-Content -LiteralPath $index)) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        if ($line -notmatch '^([0-9A-Fa-f]{64})\s+\*?(.+)$') { throw "Malformed SHA256SUMS line: $line" }
        $hash = $Matches[1].ToLowerInvariant()
        $relative = $Matches[2].Trim()
        if ($relative -match '(^/|^[A-Za-z]:|(^|/)\.\.(/|$))') { throw "Unsafe SHA256SUMS path: $relative" }
        $path = Join-Path $Directory ($relative.Replace('/', [IO.Path]::DirectorySeparatorChar))
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "The package is missing $relative" }
        if ((Get-Sha256Hex $path) -cne $hash) { throw "Package checksum mismatch: $relative" }
    }
}

function Install-Shims {
    param([string]$Directory, [hashtable]$Binaries)
    $names = @()
    $written = 0
    foreach ($command in $script:PrefixShimCommands) {
        if (-not $Binaries.ContainsKey($command)) { continue }
        $fileName = "rux-$command.cmd"
        if (Write-TextIfChanged (Join-Path $Directory $fileName) (Get-RuxCmdShimContent $command)) { $written++ }
        $names += $fileName
    }
    return [pscustomobject]@{ Names = $names; Written = $written }
}

function Get-PathStorage {
    $file = [Environment]::GetEnvironmentVariable($script:UserPathOverrideVariable)
    if (-not [string]::IsNullOrWhiteSpace($file)) { return [pscustomobject]@{ File = $file } }
    return $null
}

function Get-UserPathEntries {
    $storage = Get-PathStorage
    if ($storage) {
        if (-not (Test-Path -LiteralPath $storage.File -PathType Leaf)) { return @() }
        $raw = [IO.File]::ReadAllText($storage.File)
    } else {
        $raw = [Environment]::GetEnvironmentVariable('Path', 'User')
    }
    if ([string]::IsNullOrWhiteSpace($raw)) { return @() }
    return @($raw -split ';' | ForEach-Object { $_.Trim() } | Where-Object { $_ })
}

function Set-UserPathEntries {
    param([string[]]$Entries)
    $value = $Entries -join ';'
    $storage = Get-PathStorage
    if ($storage) {
        $directory = [IO.Path]::GetDirectoryName($storage.File)
        if ($directory) { [IO.Directory]::CreateDirectory($directory) | Out-Null }
        [IO.File]::WriteAllText($storage.File, $value, [Text.UTF8Encoding]::new($false))
        return
    }
    [Environment]::SetEnvironmentVariable('Path', $value, 'User')
}

function Test-SamePath {
    param([string]$Left, [string]$Right)
    $normalize = {
        param([string]$Value)
        [IO.Path]::TrimEndingDirectorySeparator([Environment]::ExpandEnvironmentVariables($Value).Trim())
    }
    return [string]::Equals((& $normalize $Left), (& $normalize $Right), [StringComparison]::OrdinalIgnoreCase)
}

function Confirm-PathChange {
    param([string]$Before, [string]$After, [switch]$Force)
    Write-Host 'Planned user PATH change:'
    Write-Host "  before: $Before"
    Write-Host "  after : $After"
    Write-Host 'Only the current user PATH changes; the machine PATH is never modified.'
    if ($Force) { return $true }
    $answer = Read-Host 'Apply this user PATH change? [y/N]'
    if ($answer -match '^(?i:y|yes)$') { return $true }
    Write-Host 'User PATH left unchanged. Re-run with -Force to accept, or edit the user PATH manually.'
    return $false
}

function Add-InstallDirToUserPath {
    param([string]$Directory, [switch]$Force)
    $entries = @(Get-UserPathEntries)
    if (@($entries | Where-Object { Test-SamePath $_ $Directory }).Count -gt 0) {
        Write-Host "User PATH already contains $Directory; no change."
        return $false
    }
    $before = $entries -join ';'
    $updated = @($entries) + @($Directory)
    if (-not (Confirm-PathChange $before ($updated -join ';') -Force:$Force)) { return $false }
    Set-UserPathEntries $updated
    Write-Host "Appended $Directory to the user PATH. Open a new terminal to use the shims."
    return $true
}

function Remove-InstallDirFromUserPath {
    param([string]$Directory, [switch]$Force)
    $entries = @(Get-UserPathEntries)
    $kept = @($entries | Where-Object { -not (Test-SamePath $_ $Directory) })
    if ($kept.Count -eq $entries.Count) {
        Write-Host "User PATH does not contain $Directory; no change."
        return $false
    }
    if (-not (Confirm-PathChange ($entries -join ';') ($kept -join ';') -Force:$Force)) { return $false }
    Set-UserPathEntries $kept
    Write-Host "Removed $Directory from the user PATH."
    return $true
}

function New-InstallBackup {
    param([string]$Directory, [string]$Version)
    $safeVersion = if ([string]::IsNullOrWhiteSpace($Version) -or $Version -eq 'unknown') { '' } else { $Version -replace '[^0-9A-Za-z._+-]', '_' }
    $base = if ($safeVersion) { "$Directory.old-$safeVersion" } else { "$Directory.old-backup-$([DateTime]::Now.ToString('yyyyMMdd-HHmmss'))" }
    $candidate = $base
    $index = 2
    while (Test-Path -LiteralPath $candidate) {
        $candidate = "$base-$index"
        $index++
    }
    [IO.Directory]::CreateDirectory($candidate) | Out-Null
    $files = @(Get-ChildItem -LiteralPath $Directory -File -Filter '*.exe' -ErrorAction SilentlyContinue)
    foreach ($file in $files) { Copy-Item -LiteralPath $file.FullName -Destination (Join-Path $candidate $file.Name) }
    $share = Join-Path $Directory 'share'
    if (Test-Path -LiteralPath $share -PathType Container) { Copy-Item -LiteralPath $share -Destination $candidate -Recurse }
    $statePath = Join-Path $Directory $script:StateFileName
    if (Test-Path -LiteralPath $statePath -PathType Leaf) { Copy-Item -LiteralPath $statePath -Destination $candidate }
    return $candidate
}

function Restore-InstallBackup {
    param([string]$Backup, [string]$Directory)
    Write-Warning "Install failed; restoring the previous files from $Backup"
    [IO.Directory]::CreateDirectory($Directory) | Out-Null
    foreach ($item in @(Get-ChildItem -LiteralPath $Backup -Force)) {
        Copy-Item -LiteralPath $item.FullName -Destination $Directory -Recurse -Force
    }
}

function Write-InstallSummary {
    param([string]$Version, [string]$Directory, [int]$Executables, [int]$Shims, [int]$ShimsWritten,
          [bool]$PathChanged, [string]$Backup, [string[]]$Removed)
    Write-Host ''
    Write-Host "Installed $($script:ProductName) $Version into $Directory"
    Write-Host "  executables : $Executables"
    Write-Host "  documentation: $(Join-Path $Directory 'share')"
    Write-Host "  shims       : $Shims (written or updated: $ShimsWritten)"
    if ($Removed.Count -gt 0) { Write-Host "  removed     : $($Removed -join ', ')" }
    Write-Host "  user PATH   : $(if ($PathChanged) { 'updated' } else { 'unchanged' })"
    if ($Backup) { Write-Host "  previous    : $Backup" }
    Write-Host ''
    Write-Host 'Try: rux-find --help   rux-sort --help   rux-cat --help'
    Write-Host 'The name "rux" is reserved for a future multi-call entry point; no rux shim is created.'
    Write-Host ''
    Write-Host 'Rollback:'
    if ($Backup) {
        Write-Host "  Copy everything from $Backup back into $Directory to restore the previous version."
    } else {
        Write-Host '  Reinstall the previous release archive with this script.'
    }
    Write-Host "  Run 'pwsh -File scripts/install.ps1 -Uninstall' to remove the shims and the user PATH entry."
    if ($PathChanged) { Write-Host "  The PATH change is one appended entry: $Directory" }
}

function Invoke-Install {
    param([string]$Source, [string]$InstallDir, [string]$Sha256, [bool]$Force)

    if ([string]::IsNullOrWhiteSpace($Source)) {
        throw 'Specify the release ZIP path or a version number (for example 0.4.3), or use -Uninstall.'
    }

    $temporaryRoot = $null
    $staging = $null
    try {
        if (Test-Path -LiteralPath $Source -PathType Leaf) {
            $archive = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($Source)
        } elseif (Test-SupportedVersion $Source) {
            $temporaryRoot = Join-Path ([IO.Path]::GetTempPath()) "ruxcmd-install-$([guid]::NewGuid().ToString('N'))"
            [IO.Directory]::CreateDirectory($temporaryRoot) | Out-Null
            $archive = Save-ReleaseArchive $Source $temporaryRoot
        } else {
            throw "Release archive not found and not a release version: $Source"
        }

        # Verify the archive before creating anything in or near the install directory.
        $expected = Read-ExpectedHash $archive $Sha256
        $actual = Get-Sha256Hex $archive
        if ($actual -cne $expected) {
            throw "Checksum mismatch for $([IO.Path]::GetFileName($archive)): expected $expected, got $actual. Nothing was installed."
        }
        Write-Host "Verified SHA256: $actual"

        $parent = [IO.Path]::GetDirectoryName($InstallDir)
        if ([string]::IsNullOrEmpty($parent)) { throw "Install directory must have a parent: $InstallDir" }
        [IO.Directory]::CreateDirectory($parent) | Out-Null
        $staging = Join-Path $parent ".ruxcmd-staging-$([guid]::NewGuid().ToString('N'))"
        Expand-ReleaseArchive $archive $staging
        Assert-ArchiveContents $staging
        $manifest = Read-PackageManifest $staging
        $version = Get-PackageVersion $manifest
        $packageBinaries = Get-PackageBinaries $staging
        $state = Read-InstallState $InstallDir
        $previousVersion = [string](Get-StateField $state 'version' '')
        $installedExes = @(Get-ChildItem -LiteralPath $InstallDir -File -Filter '*.exe' -ErrorAction SilentlyContinue)

        $missing = @($packageBinaries.Keys | Where-Object {
            -not (Test-Path -LiteralPath (Join-Path $InstallDir "$_.exe") -PathType Leaf)
        })
        $upToDate = $null -ne $state -and $previousVersion -ceq $version -and
            ([string](Get-StateField $state 'sha256' '')) -ceq $actual -and $missing.Count -eq 0
        if ($upToDate -and -not $Force) {
            Write-Host "$($script:ProductName) $version is already installed in $InstallDir; no executables were changed."
            $shims = Install-Shims $InstallDir $packageBinaries
            if ($shims.Written -gt 0) { Write-Host "Repaired $($shims.Written) missing or modified shim(s)." }
            $pathChanged = Add-InstallDirToUserPath $InstallDir
            Write-InstallSummary $version $InstallDir $packageBinaries.Count $shims.Names.Count $shims.Written $pathChanged (Get-StateField $state 'backup' '') @()
            return
        }

        $backup = $null
        if ($installedExes.Count -gt 0 -and ($null -eq $state -or $previousVersion -cne $version)) {
            $backup = New-InstallBackup $InstallDir $previousVersion
            Write-Host "Copied the existing installation to $backup"
        }

        $removed = @()
        try {
            [IO.Directory]::CreateDirectory($InstallDir) | Out-Null
            foreach ($name in @($packageBinaries.Keys | Sort-Object)) {
                Copy-Item -LiteralPath $packageBinaries[$name] -Destination (Join-Path $InstallDir "$name.exe") -Force
            }
            if ($state) {
                foreach ($old in @(Get-StateField $state 'binaries' @())) {
                    if ($packageBinaries.ContainsKey([string]$old)) { continue }
                    $stale = Join-Path $InstallDir "$([string]$old).exe"
                    if (Test-Path -LiteralPath $stale -PathType Leaf) {
                        Remove-Item -LiteralPath $stale -Force
                        $removed += "$old.exe"
                    }
                }
            }
            $shareDir = Join-Path $InstallDir 'share'
            if ($state -and (Test-Path -LiteralPath $shareDir -PathType Container)) { Remove-Item -LiteralPath $shareDir -Recurse -Force }
            [IO.Directory]::CreateDirectory($shareDir) | Out-Null
            foreach ($item in @(Get-ChildItem -LiteralPath $staging -Force | Where-Object { $_.Name -cne 'bin' })) {
                Copy-Item -LiteralPath $item.FullName -Destination $shareDir -Recurse -Force
            }
            $shims = Install-Shims $InstallDir $packageBinaries
            $newState = [ordered]@{
                schemaVersion = 1
                product       = $script:ProductName
                version       = $version
                target        = $script:ReleaseTarget
                source        = $Source
                sha256        = $actual
                installedAt   = [DateTime]::UtcNow.ToString('o')
                installDir    = $InstallDir
                binaries      = @($packageBinaries.Keys | Sort-Object)
                shims         = @($shims.Names)
                backup        = [string]$backup
            }
            Write-InstallState $InstallDir $newState
        } catch {
            if ($backup) { Restore-InstallBackup $backup $InstallDir }
            throw
        }

        $pathChanged = Add-InstallDirToUserPath $InstallDir -Force:$Force
        Write-InstallSummary $version $InstallDir $packageBinaries.Count $shims.Names.Count $shims.Written $pathChanged $backup $removed
    } finally {
        if ($staging -and (Test-Path -LiteralPath $staging)) { Remove-Item -LiteralPath $staging -Recurse -Force }
        if ($temporaryRoot -and (Test-Path -LiteralPath $temporaryRoot)) { Remove-Item -LiteralPath $temporaryRoot -Recurse -Force }
    }
}

function Assert-SafeRemovalTarget {
    param([string]$Directory)
    $root = [IO.Path]::TrimEndingDirectorySeparator([IO.Path]::GetPathRoot($Directory))
    if ([string]::Equals([IO.Path]::TrimEndingDirectorySeparator($Directory), $root, [StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove a drive root: $Directory"
    }
}

function Invoke-Uninstall {
    param([string]$InstallDir, [bool]$Force, [bool]$RemoveInstallDir)
    if ($RemoveInstallDir) { Assert-SafeRemovalTarget $InstallDir }
    $state = Read-InstallState $InstallDir

    $removedShims = @()
    if ($state) {
        foreach ($name in @(Get-StateField $state 'shims' @())) {
            $path = Join-Path $InstallDir ([string]$name)
            if (Test-Path -LiteralPath $path -PathType Leaf) {
                Remove-Item -LiteralPath $path -Force
                $removedShims += [string]$name
            }
        }
    } else {
        # No state: only remove shims that exactly match the installed template.
        foreach ($command in $script:PrefixShimCommands) {
            $path = Join-Path $InstallDir "rux-$command.cmd"
            if ((Test-Path -LiteralPath $path -PathType Leaf) -and [IO.File]::ReadAllText($path) -ceq (Get-RuxCmdShimContent $command)) {
                Remove-Item -LiteralPath $path -Force
                $removedShims += "rux-$command.cmd"
            }
        }
    }

    $pathChanged = Remove-InstallDirFromUserPath $InstallDir -Force:$Force

    $removedFiles = 0
    if ($RemoveInstallDir) {
        if (-not $state) {
            Write-Warning "No $($script:StateFileName) in $InstallDir; refusing to delete files this script did not record."
        } else {
            foreach ($name in @(Get-StateField $state 'binaries' @())) {
                $path = Join-Path $InstallDir "$([string]$name).exe"
                if (Test-Path -LiteralPath $path -PathType Leaf) {
                    Remove-Item -LiteralPath $path -Force
                    $removedFiles++
                }
            }
            $share = Join-Path $InstallDir 'share'
            if (Test-Path -LiteralPath $share -PathType Container) {
                Remove-Item -LiteralPath $share -Recurse -Force
                $removedFiles++
            }
            $statePath = Join-Path $InstallDir $script:StateFileName
            if (Test-Path -LiteralPath $statePath -PathType Leaf) {
                Remove-Item -LiteralPath $statePath -Force
                $removedFiles++
            }
            if (Test-Path -LiteralPath $InstallDir -PathType Container) {
                $remaining = @(Get-ChildItem -LiteralPath $InstallDir -Force)
                if ($remaining.Count -eq 0) {
                    Remove-Item -LiteralPath $InstallDir -Force
                    Write-Host "Removed directory $InstallDir"
                } else {
                    Write-Host "Kept $InstallDir; $($remaining.Count) unrecorded item(s) remain."
                }
            }
        }
    }

    Write-Host ''
    Write-Host "Removed $($script:ProductName) from $InstallDir"
    Write-Host "  shims       : $($removedShims.Count)$(if ($removedShims.Count) { ' (' + ($removedShims -join ', ') + ')' })"
    Write-Host "  user PATH   : $(if ($pathChanged) { 'updated' } else { 'unchanged' })"
    if ($RemoveInstallDir) { Write-Host "  files       : $removedFiles recorded item(s)" }
    else { Write-Host '  files       : kept (pass -RemoveInstallDir to delete the recorded files)' }
    Write-Host ''
    Write-Host "Rollback: reinstall with 'pwsh -File scripts/install.ps1 -Source <archive-or-version>'."
}

function Assert-NoPackagedInstall {
    # The test suite points the PATH seam at a file and never touches a real
    # profile, so skip the registry probe there to keep the suite hermetic.
    if (-not [string]::IsNullOrWhiteSpace([Environment]::GetEnvironmentVariable($script:UserPathOverrideVariable))) { return }
    $keys = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$($script:PackagedAppId)_is1",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\$($script:PackagedAppId)_is1",
        "HKLM:\Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\$($script:PackagedAppId)_is1"
    )
    foreach ($key in $keys) {
        if (Test-Path -LiteralPath $key) {
            throw "RuxCmd is installed from the EXE installer ($key). Uninstall it from Settings > Apps before using install.ps1."
        }
    }
}

Assert-NoPackagedInstall
$InstallDir = Resolve-InstallDirectory $InstallDir
if ($Uninstall) {
    Invoke-Uninstall -InstallDir $InstallDir -Force:$Force.IsPresent -RemoveInstallDir:$RemoveInstallDir.IsPresent
} else {
    Invoke-Install -Source $Source -InstallDir $InstallDir -Sha256 $Sha256 -Force:$Force.IsPresent
}

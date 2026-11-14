# Bumps the workspace version across every file that references it:
#   - Cargo.toml (workspace root)
#   - desktop/src-tauri/Cargo.toml + Cargo.lock
#   - README.md / README-zh.md download paths
#   - scripts/packaging/scoop/ruxcmd.json
#   - scripts/packaging/winget/*.yaml
# Also updates the desktop Cargo.lock via cargo update, and runs
# check-manifests.ps1 to confirm consistency before returning.
#
# Usage:
#   pwsh -File scripts/bump-version.ps1 -NewVersion 0.5.19
#   pwsh -File scripts/bump-version.ps1 -NewVersion 0.6.0 -SkipLockUpdate

[CmdletBinding()]
param(
    [Parameter(Mandatory)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$NewVersion,

    # Skip the desktop Cargo.lock cargo update (use when lock is already current).
    [switch]$SkipLockUpdate
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot ? (Split-Path $PSScriptRoot -Parent) : '.'
Push-Location $root
try {
    # 1. Read the current version from Cargo.toml.
    $cargo = Get-Content Cargo.toml -Raw
    if ($cargo -notmatch 'version\s*=\s*"(\d+\.\d+\.\d+)"') {
        throw 'Cannot read the workspace version from Cargo.toml.'
    }
    $oldVersion = $Matches[1]
    if ($oldVersion -eq $NewVersion) {
        Write-Host "Version is already $NewVersion; nothing to do."
        return
    }
    Write-Host "Bumping $oldVersion -> $NewVersion"

    # 2. Cargo.toml
    $cargo = $cargo -replace [regex]::Escape("version = `"$oldVersion`""), "version = `"$NewVersion`""
    Set-Content Cargo.toml $cargo -Encoding utf8NoBOM -NoNewline

    # 3. Desktop workspace
    $desktopToml = 'desktop/src-tauri/Cargo.toml'
    if (Test-Path $desktopToml) {
        $text = Get-Content $desktopToml -Raw
        $text = $text -replace [regex]::Escape("version = `"$oldVersion`""), "version = `"$NewVersion`""
        Set-Content $desktopToml $text -Encoding utf8NoBOM -NoNewline
    }

    # 4. Desktop Cargo.lock
    if (-not $SkipLockUpdate -and (Test-Path 'desktop/src-tauri/Cargo.toml')) {
        Write-Host 'Updating desktop Cargo.lock...'
        cargo update --manifest-path desktop/src-tauri/Cargo.toml 2>&1 | ForEach-Object { Write-Host "  $_" }
    }

    # 5. READMEs
    foreach ($readme in @('README.md', 'README-zh.md')) {
        $text = Get-Content $readme -Raw
        $text = $text.Replace("ruxcmd-$oldVersion-x86_64-pc-windows-msvc", "ruxcmd-$NewVersion-x86_64-pc-windows-msvc")
        $text = $text.Replace("v$oldVersion", "v$NewVersion")
        Set-Content $readme $text -Encoding utf8NoBOM -NoNewline
    }

    # 6. Packaging manifests
    foreach ($path in @(Get-ChildItem 'scripts/packaging/scoop/*.json', 'scripts/packaging/winget/*.yaml')) {
        $text = Get-Content $path.FullName -Raw
        $text = $text.Replace($oldVersion, $NewVersion)
        Set-Content $path.FullName $text -Encoding utf8NoBOM -NoNewline
    }

    # 7. Validate
    Write-Host "`nRunning check-manifests..."
    & (Join-Path $PSScriptRoot 'check-manifests.ps1')
    if ($LASTEXITCODE -ne 0) { throw 'check-manifests failed after the bump.' }

    Write-Host "`nVersion bumped to $NewVersion across all files."
    Write-Host 'Remember to add the CHANGELOG entry before releasing.'
} finally {
    Pop-Location
}

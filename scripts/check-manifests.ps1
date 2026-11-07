# Validates that the scoop and winget packaging manifests stay consistent
# with the workspace version and with each other.  Runs as part of Verify so
# a version bump that forgets the manifests (or a stale SHA256 hash) fails
# the gate before it can reach a release.
#
# Checks:
#   - scoop PackageVersion / winget PackageVersion / README download paths
#     all carry the workspace version from Cargo.toml
#   - the download URL names the matching ZIP file
#   - the recorded SHA256 is a 64-hex hash (its value can only be validated
#     against the published release after the Release workflow runs; a
#     mismatched HASH is what this guards against between versions)
#
# Usage: pwsh -File scripts/check-manifests.ps1 [-ExpectedVersion 0.5.7]

[CmdletBinding()]
param(
    # Workspace version; defaults to reading Cargo.toml.
    [string]$ExpectedVersion
)

$ErrorActionPreference = 'Stop'

if (-not $ExpectedVersion) {
    $cargo = Get-Content Cargo.toml -Raw
    if ($cargo -match 'version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"') {
        $ExpectedVersion = $Matches[1]
    } else {
        throw 'Cannot read the workspace version from Cargo.toml.'
    }
}
$failures = [System.Collections.Generic.List[string]]::new()

# --- scoop manifest ---------------------------------------------------------
$scoopPath = 'scripts/packaging/scoop/ruxcmd.json'
$scoop = Get-Content $scoopPath -Raw | ConvertFrom-Json
if ($scoop.version -ne $ExpectedVersion) {
    $failures.Add("scoop version '$($scoop.version)' != workspace '$ExpectedVersion'")
}
$expectedZip = "ruxcmd-$ExpectedVersion-x86_64-pc-windows-msvc.zip"
if ($scoop.architecture.'64bit'.url -notlike "*$expectedZip") {
    $failures.Add("scoop URL does not name ${expectedZip}: $($scoop.architecture.'64bit'.url)")
}
$hash = $scoop.architecture.'64bit'.hash
if ($hash -notmatch '^[0-9a-f]{64}$') {
    $failures.Add("scoop hash is not a 64-hex SHA256: '$hash'")
}

# --- winget manifests -------------------------------------------------------
$wingetFiles = @(
    'scripts/packaging/winget/wangdxnum1.RuxCmd.yaml',
    'scripts/packaging/winget/wangdxnum1.RuxCmd.installer.yaml',
    'scripts/packaging/winget/wangdxnum1.RuxCmd.locale.en-US.yaml'
)
foreach ($path in $wingetFiles) {
    $text = Get-Content $path -Raw
    if ($text -notmatch "PackageVersion:\s*([^\r\n]+)") {
        $failures.Add("$path has no PackageVersion")
        continue
    }
    $version = $Matches[1].Trim()
    if ($version -ne $ExpectedVersion) {
        $failures.Add("$path PackageVersion '$version' != workspace '$ExpectedVersion'")
    }
}
$installer = Get-Content $wingetFiles[1] -Raw
if ($installer -match "InstallerSha256:\s*([0-9A-Fa-f]{64})") {
    if ($Matches[1].ToLowerInvariant() -ne $hash) {
        $failures.Add("winget InstallerSha256 differs from the scoop hash")
    }
} else {
    $failures.Add('winget InstallerSha256 missing or malformed')
}
if ($installer -notlike "*$expectedZip*") {
    $failures.Add("winget installer does not reference $expectedZip")
}

# --- README download paths --------------------------------------------------
foreach ($readme in @('README.md', 'README-zh.md')) {
    $text = Get-Content $readme -Raw
    if ($text -notlike "*$expectedZip*") {
        $failures.Add("$readme download paths do not mention $expectedZip")
    }
}

if ($failures.Count -gt 0) {
    foreach ($failure in $failures) { Write-Host "FAIL  $failure" }
    throw "check-manifests failed with $($failures.Count) problem(s)."
}
Write-Host 'Manifests match the workspace version, URLs and hashes.'

# Replaces the SHA256 hash in the scoop and winget manifests with a new
# value, then runs check-manifests.ps1 to confirm the result.  This is the
# post-release step that keeps the packaging manifests in sync with the
# published artifacts; it replaces whatever hash is currently present, so
# callers do not need to know the previous value.
#
# Usage:
#   pwsh -File scripts/sync-manifest-hash.ps1 -ZipSha256 <64-hex-hash>
#   pwsh -File scripts/sync-manifest-hash.ps1 -ReleaseTag v0.5.15   (downloads
#       the release's .zip.sha256 asset first, then syncs from it)

[CmdletBinding()]
param(
    # The new SHA256 for the ZIP artifact (64 hex characters).
    [Parameter(ParameterSetName = 'Hash')]
    [string]$ZipSha256,

    # A released tag whose .zip.sha256 asset provides the hash.
    [Parameter(ParameterSetName = 'Tag')]
    [string]$ReleaseTag
)

$ErrorActionPreference = 'Stop'

if (-not $ZipSha256) {
    if (-not $ReleaseTag) { throw 'Provide -ZipSha256 <hex> or -ReleaseTag <tag>.' }
    $asset = "ruxcmd-$($ReleaseTag.TrimStart('v'))-x86_64-pc-windows-msvc.zip.sha256"
    Write-Host "Downloading $asset from release $ReleaseTag..."
    gh release download $ReleaseTag --pattern $asset --output (Join-Path $env:RUNNER_TEMP $asset) --clobber
    if ($LASTEXITCODE -ne 0) { throw "gh release download failed for $ReleaseTag." }
    $ZipSha256 = ((Get-Content (Join-Path $env:RUNNER_TEMP $asset) -TotalCount 1) -split '\s+')[0].Trim()
}

if ($ZipSha256 -notmatch '^[0-9a-fA-F]{64}$') {
    throw "The provided hash is not 64 hex characters: '$ZipSha256'"
}
$ZipSha256 = $ZipSha256.ToLowerInvariant()

$scoopPath = 'scripts/packaging/scoop/ruxcmd.json'
$wingetPath = 'scripts/packaging/winget/wangdxnum1.RuxCmd.installer.yaml'

# Replace whatever 64-hex hash currently sits in the hash fields.
$scoop = Get-Content $scoopPath -Raw
if ($scoop -notmatch '"hash":\s*"([0-9a-fA-F]{64})"') { throw "scoop manifest has no hash field." }
$scoop = [regex]::Replace($scoop, '"hash":\s*"[0-9a-fA-F]{64}"', "`"hash`": `"$ZipSha256`"")
Set-Content $scoopPath $scoop -Encoding utf8NoBOM -NoNewline

$winget = Get-Content $wingetPath -Raw
if ($winget -notmatch 'InstallerSha256:\s*([0-9a-fA-F]{64})') { throw "winget manifest has no InstallerSha256 field." }
$winget = [regex]::Replace($winget, 'InstallerSha256:\s*[0-9a-fA-F]{64}', "InstallerSha256: $($ZipSha256.ToUpperInvariant())")
Set-Content $wingetPath $winget -Encoding utf8NoBOM -NoNewline

Write-Host "Manifests synced to $ZipSha256."
& (Join-Path $PSScriptRoot 'check-manifests.ps1')
if ($LASTEXITCODE -ne 0) { throw 'check-manifests failed after the sync.' }

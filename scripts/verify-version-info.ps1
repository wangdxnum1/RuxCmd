#Requires -Version 7.0
param([string]$BinDir, [switch]$DebugBuild)
. (Join-Path $PSScriptRoot 'common.ps1')
$workspace = Get-WorkspaceInfo
if (-not $BinDir) { $BinDir = Join-Path $workspace.TargetDirectory "$($workspace.Target)/release" }
$BinDir = Get-FileSystemPath $BinDir
foreach ($binary in $workspace.Binaries) {
    Assert-BinaryVersionInfo (Join-Path $BinDir "$($binary.Name).exe") $binary $workspace -DebugBuild:$DebugBuild | Out-Null
}
Write-Host "Version resources verified: $($workspace.Binaries.Count) commands, suite $($workspace.Version)."

#Requires -Version 7.0
# Compatibility entry: suite releases are managed from the repository root.
param([string]$OutputDir)
$repository = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '../../..'))
$arguments = @{}
if ($OutputDir) { $arguments.OutputDir = $OutputDir }
& (Join-Path $repository 'scripts/release.ps1') @arguments

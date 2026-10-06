#Requires -Version 7.0
<#
.SYNOPSIS
Build, check, test, verify, or clean RuxCmd from any working directory.
.DESCRIPTION
Defaults to a locked Windows x64 release build. Verify runs all checks,
tests, builds, smoke/behavior checks and script tests. Use -WhatIf to preview.
.EXAMPLE
./scripts/build.ps1 -Package ps -Configuration Debug -Jobs 4
.EXAMPLE
./scripts/build.ps1 -Task Verify -Offline
#>
[CmdletBinding(SupportsShouldProcess)]
param(
    [ValidateSet('Build', 'Check', 'Test', 'Verify', 'Clean')][string]$Task = 'Build',
    [ValidateSet('Release', 'Debug')][string]$Configuration = 'Release',
    [string[]]$Package,
    [ValidateRange(1, 256)][int]$Jobs,
    [switch]$Offline
)
. (Join-Path $PSScriptRoot 'common.ps1')
if (-not $IsWindows) { throw 'RuxCmd build scripts require Windows and MSVC.' }
if ($Task -eq 'Verify' -and $Package) { throw 'Verify covers the entire workspace; omit -Package.' }
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'Install Rustup and add Cargo to PATH.' }
$workspace = Get-WorkspaceInfo -Offline:$Offline
$selection = @()
foreach ($name in $Package) {
    if ($name -in $workspace.Packages) { $resolved = $name }
    else {
        $binary = @($workspace.Binaries | Where-Object Name -CEQ $name)
        if ($binary.Count -ne 1) { throw "Unknown command/package '$name'. Use a name from crates/ or a binary name such as ps." }
        $resolved = $binary[0].PackageName
    }
    if ($resolved -notin $selection) { $selection += $resolved }
}
$scope = if ($selection.Count) { @($selection | ForEach-Object { '-p'; $_ }) } else { @('--workspace') }
$options = @('--locked')
if ($Offline) { $options += '--offline' }
if ($Jobs) { $options += @('--jobs', [string]$Jobs) }
if ($VerbosePreference -eq 'Continue') { $options += '--verbose' }
$targetOptions = @('--target', $workspace.Target)
$profile = if ($Configuration -eq 'Release') { 'release' } else { 'debug' }
$binaryDirectory = Join-Path $workspace.TargetDirectory "$($workspace.Target)/$profile"
$clock = [Diagnostics.Stopwatch]::StartNew()
function Invoke-CargoStep([string]$Label, [string[]]$CargoArguments) {
    Write-Host "`n== $Label ==" -ForegroundColor Cyan
    Write-Host "cargo $($CargoArguments -join ' ')"
    if ($PSCmdlet.ShouldProcess($script:RepositoryRoot, "cargo $($CargoArguments -join ' ')")) {
        & cargo @CargoArguments
        if ($LASTEXITCODE -ne 0) { throw "$Label failed (exit $LASTEXITCODE)." }
    }
}
Push-Location -LiteralPath $script:RepositoryRoot
$previousOffline = [Environment]::GetEnvironmentVariable('CARGO_NET_OFFLINE', 'Process')
try {
    # Applies to rustfmt metadata and verification subprocesses as well as Cargo.
    if ($Offline) { $env:CARGO_NET_OFFLINE = 'true' }
    Write-Host "RuxCmd $($workspace.Version) | $Task | $Configuration | $($workspace.Target)"
    if ($Task -in @('Check', 'Verify')) {
        $format = @('fmt') + $scope + @('--check')
        # cargo fmt selects individual packages with -p and uses --all for a workspace.
        if (-not $selection.Count) { $format = @('fmt', '--all', '--check') }
        Invoke-CargoStep 'Formatting' $format
        Invoke-CargoStep 'Check' (@('check') + $scope + @('--all-targets') + $targetOptions + $options)
        Invoke-CargoStep 'Clippy' (@('clippy') + $scope + @('--all-targets') + $targetOptions + $options)
    }
    if ($Task -in @('Test', 'Verify')) {
        Invoke-CargoStep 'Tests' (@('test') + $scope + @('--all-targets') + $targetOptions + $options)
    }
    if ($Task -in @('Build', 'Verify')) {
        $buildArguments = @('build') + $scope + @('--bins') + $targetOptions + $options
        if ($Configuration -eq 'Release') { $buildArguments += '--release' }
        Invoke-CargoStep 'Build' $buildArguments
    }
    if ($Task -eq 'Verify' -and $PSCmdlet.ShouldProcess($binaryDirectory, 'Run smoke, behavior and script tests')) {
        & (Join-Path $PSScriptRoot 'smoke-test.ps1') -BinDir $binaryDirectory
        & (Join-Path $PSScriptRoot 'verify-behavior.ps1') -BinDir $binaryDirectory
        & (Join-Path $PSScriptRoot 'tests/package-tests.ps1') -BinDir $binaryDirectory
        & (Join-Path $PSScriptRoot 'tests/entrypoint-tests.ps1')
    }
    if ($Task -eq 'Clean') {
        $cleanProfile = if ($Configuration -eq 'Release') { 'release' } else { 'dev' }
        $cleanArguments = @('clean') + $targetOptions + @('--profile', $cleanProfile)
        if ($selection.Count) { $cleanArguments += @($selection | ForEach-Object { '-p'; $_ }) }
        else { $cleanArguments += '--workspace' }
        Invoke-CargoStep 'Clean build outputs' $cleanArguments
    }
    $completedTask = if ($WhatIfPreference) { "$Task preview" } else { $Task }
    Write-Host "`n$completedTask completed in $([Math]::Round($clock.Elapsed.TotalSeconds, 1))s." -ForegroundColor Green
    if ($Task -in @('Build', 'Verify')) { Write-Host "Executables: $binaryDirectory" }
} finally {
    if ($Offline) { [Environment]::SetEnvironmentVariable('CARGO_NET_OFFLINE', $previousOffline, 'Process') }
    Pop-Location
}

#!/usr/bin/env pwsh
# scripts/release.ps1
# Build optimized ps binary in release mode and stage it under dist/.

$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")

$TargetTriple = "x86_64-pc-windows-msvc"
$BuildProfile = "release"
$BinName = "ps.exe"

Write-Host "[release] cargo --version" -ForegroundColor Cyan
cargo --version

Write-Host "[release] cargo build --release --target $TargetTriple" -ForegroundColor Cyan
cargo build --release --target $TargetTriple --locked

$Src = "target\$TargetTriple\$BuildProfile\$BinName"
if (-not (Test-Path -LiteralPath $Src)) {
    Write-Error ("Built binary not found at: {0}" -f $Src)
    exit 1
}

$DistDir = "dist"
if (-not (Test-Path -LiteralPath $DistDir)) {
    New-Item -ItemType Directory -Path $DistDir | Out-Null
}
$Dst = Join-Path $DistDir $BinName
if (Test-Path -LiteralPath $Dst) {
    Remove-Item -LiteralPath $Dst -Force
}
Copy-Item -LiteralPath $Src -Destination $Dst -Force

$Size = (Get-Item -LiteralPath $Dst).Length
Write-Host ("[release] done -> {0} ({1:N2} MB)" -f $Dst, ($Size / 1MB)) -ForegroundColor Green

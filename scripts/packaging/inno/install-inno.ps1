#Requires -Version 7.0
<#
.SYNOPSIS
Install the pinned Inno Setup compiler when it is missing or outdated.

.DESCRIPTION
Reads scripts/packaging/inno/toolchain.json, so CI and local packaging agree on
the exact version. If the pinned version is already installed it does nothing;
otherwise it downloads the official installer and runs it silently. ISCC.exe
has no version resource, so the version is read from the Inno Setup uninstall
registration keys.

.PARAMETER Iscc
Existing ISCC.exe to check first. Defaults to PATH and the standard
"Inno Setup 6" / "Inno Setup 7" installation directories.

.EXAMPLE
./scripts/packaging/inno/install-inno.ps1
#>
[CmdletBinding()]
param(
    [string]$Iscc
)

. (Join-Path $PSScriptRoot '..\..\common.ps1')

if (-not $IsWindows) { throw 'Inno Setup is only required on Windows.' }

$toolchain = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'toolchain.json') -Raw | ConvertFrom-Json
$expected = [string]$toolchain.innoSetupVersion
if ($expected -notmatch '^\d+\.\d+\.\d+$') { throw "Invalid pinned Inno Setup version: $expected" }

$existing = if ($Iscc) { $Iscc } else { Find-InnoSetupCompiler }
if ($existing) {
    $version = Get-InnoSetupVersion $existing
    if ($version -match "^$([regex]::Escape($expected))") {
        Write-Host "Inno Setup $version is already installed at $existing"
        return
    }
    Write-Host "Found Inno Setup '$version' at $existing; installing $expected."
}

$installer = Join-Path ([IO.Path]::GetTempPath()) "innosetup-$expected.exe"
Write-Host "Downloading $($toolchain.downloadUrl)"
Invoke-WebRequest -Uri $toolchain.downloadUrl -OutFile $installer
$process = Start-Process -FilePath $installer -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-' -Wait -PassThru
if ($process.ExitCode -ne 0) { throw "Inno Setup installer exited $($process.ExitCode)." }
Remove-Item -LiteralPath $installer -Force

$iscc = Find-InnoSetupCompiler
if (-not $iscc) { throw 'ISCC.exe was not found after installing Inno Setup.' }
$version = Get-InnoSetupVersion $iscc
if ($version -notmatch "^$([regex]::Escape($expected))") {
    throw "Inno Setup version mismatch after install: expected $expected, found '$version'."
}
Write-Host "Inno Setup $version at $iscc"

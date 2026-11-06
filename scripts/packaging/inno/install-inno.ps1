#Requires -Version 7.0
<#
.SYNOPSIS
Install the pinned Inno Setup compiler.

.DESCRIPTION
Reads scripts/packaging/inno/toolchain.json, so CI and local packaging agree on
the pinned version and the supported major version. If a compiler with the
supported major is already installed it does nothing; otherwise it downloads
the official installer and runs it silently.

Inno Setup 7's own installer accepts the same unattended switches as Inno 6
(/VERYSILENT /SUPPRESSMSGBOXES /NORESTART /SP-); there is no separate EULA
acceptance switch, and silent mode does not show the license page.

ISCC.exe has no version resource, so the installed version is read from the
Inno Setup uninstall registration keys. A compiler outside the supported major
is rejected instead of silently used.

.PARAMETER Iscc
Existing ISCC.exe to verify and use. Defaults to PATH and the standard
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
$supportedMajor = [int]$toolchain.supportedMajorVersion
if ($supportedMajor -ne (Get-InnoSetupMajorVersion $expected)) {
    throw "toolchain.json supportedMajorVersion $supportedMajor does not match the pinned version $expected."
}

if ($Iscc) {
    $explicit = Get-FileSystemPath $Iscc
    if (-not (Test-Path -LiteralPath $explicit -PathType Leaf)) { throw "ISCC.exe not found: $explicit" }
    Assert-InnoSetupVersion -Version (Get-InnoSetupVersion $explicit) -SupportedMajor $supportedMajor -Path $explicit -DownloadUrl $toolchain.downloadUrl
    return
}

$existing = Find-InnoSetupCompiler -SupportedMajor $supportedMajor
if ($existing) {
    $version = Get-InnoSetupVersion $existing
    if ((Get-InnoSetupMajorVersion $version) -eq $supportedMajor) {
        Write-Host "Inno Setup $version is already installed at $existing"
        return
    }
    Write-Host "Found Inno Setup '$version' at $existing; installing $expected."
} else {
    Write-Host "Inno Setup $expected is not installed."
}

$installer = Join-Path ([IO.Path]::GetTempPath()) "innosetup-$expected.exe"
Write-Host "Downloading $($toolchain.downloadUrl)"
Invoke-WebRequest -Uri $toolchain.downloadUrl -OutFile $installer
$process = Start-Process -FilePath $installer -ArgumentList '/VERYSILENT', '/SUPPRESSMSGBOXES', '/NORESTART', '/SP-' -Wait -PassThru
if ($process.ExitCode -ne 0) { throw "Inno Setup installer exited $($process.ExitCode)." }
Remove-Item -LiteralPath $installer -Force

$iscc = Find-InnoSetupCompiler -SupportedMajor $supportedMajor
if (-not $iscc) { throw "ISCC.exe was not found after installing Inno Setup $expected." }
$version = Get-InnoSetupVersion $iscc
Assert-InnoSetupVersion -Version $version -SupportedMajor $supportedMajor -Path $iscc -DownloadUrl $toolchain.downloadUrl

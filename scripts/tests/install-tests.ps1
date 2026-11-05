#Requires -Version 7.0
# Exercises scripts/install.ps1 in isolated temporary directories. The user
# PATH is redirected through RUXCMD_INSTALL_USER_PATH_FILE, so no real profile,
# registry value, or installation directory is touched.
param([string]$Case)
. (Join-Path $PSScriptRoot '../common.ps1')
$temporaryParent = [IO.Path]::GetTempPath()
$temporary = Join-Path $temporaryParent "ruxcmd install tests $([guid]::NewGuid().ToString('N'))"
[IO.Directory]::CreateDirectory($temporary) | Out-Null
$script:InstallScript = Join-Path $script:RepositoryRoot 'scripts/install.ps1'
$script:Pwsh = Join-Path $PSHOME 'pwsh.exe'
$script:PathFile = Join-Path $temporary 'user-path.txt'
$script:Encoding = [Text.UTF8Encoding]::new($false)
$script:Passed = 0
$env:RUXCMD_INSTALL_USER_PATH_FILE = $script:PathFile

function Test-Case([string]$Name, [scriptblock]$Body) {
    if ($Case -and $Name -notlike $Case) { return }
    & $Body
    $script:Passed++
    Write-Host "PASS: $Name"
}
function Assert-Equal($Actual, $Expected) { if ($Actual -cne $Expected) { throw "Expected <$Expected>, got <$Actual>" } }
function Assert-True($Value, [string]$Message) { if (-not $Value) { throw $Message } }
function Assert-Contains([string]$Text, [string]$Pattern) {
    if ($Text -notmatch [regex]::Escape($Pattern)) { throw "Expected output to contain '$Pattern'.`n$Text" }
}
function Assert-Success($Result, [string]$Label) {
    if ($Result.ExitCode -ne 0) { throw "$Label exited $($Result.ExitCode).`n$($Result.Output)`n$($Result.Error)" }
}
function Set-TestUserPath([string]$Value) {
    [IO.File]::WriteAllText($script:PathFile, $Value, $script:Encoding)
}
function Get-TestUserPath {
    [IO.File]::ReadAllText($script:PathFile)
}
function Invoke-Installer([string[]]$Arguments) {
    Invoke-ToolProcess $script:Pwsh (@('-NoProfile', '-File', $script:InstallScript) + $Arguments) -TimeoutSeconds 180
}
function New-FakePackage {
    param([string]$Root, [string]$Version, [string[]]$Commands)
    $stage = Join-Path $Root "stage-$Version"
    [IO.Directory]::CreateDirectory((Join-Path $stage 'bin')) | Out-Null
    foreach ($command in $Commands) {
        [IO.File]::WriteAllText((Join-Path $stage "bin/$command.exe"), "$command $Version", $script:Encoding)
    }
    [IO.File]::WriteAllText((Join-Path $stage 'README.md'), "# RuxCmd $Version", $script:Encoding)
    $files = @(Get-ChildItem -LiteralPath $stage -File -Recurse | Sort-Object FullName | ForEach-Object {
        [ordered]@{
            path   = [IO.Path]::GetRelativePath($stage, $_.FullName).Replace('\', '/')
            sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    })
    $manifest = [ordered]@{
        schemaVersion = 1
        suiteVersion  = $Version
        target        = 'x86_64-pc-windows-msvc'
        binaries      = @($Commands | ForEach-Object { [ordered]@{ name = $_; file = "bin/$_.exe" } })
        files         = $files
    }
    $manifest | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $stage 'manifest.json') -Encoding utf8NoBOM
    ($files | ForEach-Object { "$($_.sha256)  $($_.path)" }) -join "`n" | Set-Content -LiteralPath (Join-Path $stage 'SHA256SUMS') -Encoding utf8NoBOM
    $archive = Join-Path $Root "ruxcmd-$Version-x86_64-pc-windows-msvc.zip"
    [IO.Compression.ZipFile]::CreateFromDirectory($stage, $archive)
    $hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    [IO.File]::WriteAllText("$archive.sha256", "$hash  $([IO.Path]::GetFileName($archive))", $script:Encoding)
    return $archive
}

try {
    Test-Case 'install: verified archive installs executables, shims and user PATH' {
        $root = Join-Path $temporary 'fresh'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        $archive = New-FakePackage $root '0.4.3' @('find', 'cat', 'sleep', 'true')
        $installDir = Join-Path $root 'install dir\bin'
        Set-TestUserPath 'C:\Windows;C:\Tools'
        $result = Invoke-Installer @('-Source', $archive, '-InstallDir', $installDir, '-Force')
        Assert-Success $result 'install'
        Assert-True (Test-Path -LiteralPath (Join-Path $installDir 'find.exe') -PathType Leaf) 'find.exe was not installed.'
        Assert-True (Test-Path -LiteralPath (Join-Path $installDir 'true.exe') -PathType Leaf) 'true.exe was not installed.'
        Assert-True (Test-Path -LiteralPath (Join-Path $installDir 'share\README.md') -PathType Leaf) 'README.md was not placed under share.'
        Assert-True (Test-Path -LiteralPath (Join-Path $installDir '.ruxcmd-install.json') -PathType Leaf) 'Install state is missing.'
        $expected = "@echo off`r`n`"%~dp0find.exe`" %*`r`n"
        Assert-Equal ([IO.File]::ReadAllText((Join-Path $installDir 'rux-find.cmd'))) $expected
        Assert-Equal ([IO.File]::ReadAllText((Join-Path $installDir 'rux-sleep.cmd'))) "@echo off`r`n`"%~dp0sleep.exe`" %*`r`n"
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $installDir 'rux-true.cmd'))) 'A shim was created for a non-conflicting command.'
        Assert-Equal (Get-TestUserPath) "C:\Windows;C:\Tools;$installDir"
        $state = Get-Content -LiteralPath (Join-Path $installDir '.ruxcmd-install.json') -Raw | ConvertFrom-Json
        Assert-Equal $state.version '0.4.3'
        Assert-Equal $state.sha256 (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    }

    Test-Case 'install: same version and checksum is idempotent' {
        $root = Join-Path $temporary 'idempotent'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        $archive = New-FakePackage $root '0.4.3' @('find', 'cat', 'sleep')
        $installDir = Join-Path $root 'bin'
        Set-TestUserPath 'C:\Windows'
        Assert-Success (Invoke-Installer @('-Source', $archive, '-InstallDir', $installDir, '-Force')) 'first install'
        $pathBefore = Get-TestUserPath
        $marks = @{}
        foreach ($name in @('find.exe', 'rux-find.cmd', '.ruxcmd-install.json')) {
            $marks[$name] = (Get-Item -LiteralPath (Join-Path $installDir $name)).LastWriteTimeUtc
        }
        $result = Invoke-Installer @('-Source', $archive, '-InstallDir', $installDir)
        Assert-Success $result 'repeated install'
        Assert-Contains ($result.Output + $result.Error) 'already installed'
        foreach ($name in $marks.Keys) {
            Assert-Equal (Get-Item -LiteralPath (Join-Path $installDir $name)).LastWriteTimeUtc $marks[$name]
        }
        Assert-Equal (Get-TestUserPath) $pathBefore
        Assert-Equal @((Get-TestUserPath).Split(';') | Where-Object { $_ -ceq $installDir }).Count 1
    }

    Test-Case 'install: checksum failures abort before writing anything' {
        $root = Join-Path $temporary 'checksum'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        $archive = New-FakePackage $root '0.4.3' @('find')
        $installDir = Join-Path $root 'not-created\bin'
        $result = Invoke-Installer @('-Source', $archive, '-InstallDir', $installDir, '-Sha256', ('0' * 64), '-Force')
        Assert-Equal $result.ExitCode 1
        Assert-Contains ($result.Output + $result.Error) 'Checksum mismatch'
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $root 'not-created'))) 'A failed checksum still created the install directory.'

        $withoutSidecar = Join-Path $root 'no-sidecar.zip'
        Copy-Item -LiteralPath $archive -Destination $withoutSidecar
        $result = Invoke-Installer @('-Source', $withoutSidecar, '-InstallDir', $installDir, '-Force')
        Assert-Equal $result.ExitCode 1
        Assert-Contains ($result.Output + $result.Error) 'No expected checksum'
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $root 'not-created'))) 'A missing checksum still created the install directory.'
    }

    Test-Case 'install: upgrade keeps the previous version directory' {
        $root = Join-Path $temporary 'upgrade'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        $installDir = Join-Path $root 'bin'
        Set-TestUserPath 'C:\Windows;C:\Tools'
        Assert-Success (Invoke-Installer @('-Source', (New-FakePackage $root '0.4.3' @('find', 'cat')), '-InstallDir', $installDir, '-Force')) '0.4.3 install'
        Assert-Success (Invoke-Installer @('-Source', (New-FakePackage $root '0.4.4' @('find', 'cat', 'sed')), '-InstallDir', $installDir, '-Force')) '0.4.4 upgrade'
        $backup = "$installDir.old-0.4.3"
        Assert-True (Test-Path -LiteralPath (Join-Path $backup 'find.exe') -PathType Leaf) 'The previous version was not kept.'
        Assert-True (Test-Path -LiteralPath (Join-Path $installDir 'sed.exe') -PathType Leaf) 'The new executable was not installed.'
        $state = Get-Content -LiteralPath (Join-Path $installDir '.ruxcmd-install.json') -Raw | ConvertFrom-Json
        Assert-Equal $state.version '0.4.4'
        Assert-Equal (Get-TestUserPath) "C:\Windows;C:\Tools;$installDir"
    }

    Test-Case 'uninstall: removes shims, recorded files and the PATH entry' {
        $root = Join-Path $temporary 'uninstall'
        [IO.Directory]::CreateDirectory($root) | Out-Null
        $installDir = Join-Path $root 'bin'
        Set-TestUserPath 'C:\Windows;C:\Tools'
        $archive = New-FakePackage $root '0.4.3' @('find', 'cat')
        Assert-Success (Invoke-Installer @('-Source', $archive, '-InstallDir', $installDir, '-Force')) 'install'
        Assert-Success (Invoke-Installer @('-Uninstall', '-InstallDir', $installDir, '-Force')) 'uninstall without deleting files'
        Assert-True (Test-Path -LiteralPath (Join-Path $installDir 'find.exe') -PathType Leaf) 'Uninstall deleted files without -RemoveInstallDir.'
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $installDir 'rux-find.cmd'))) 'The shim was not removed.'
        Assert-Equal (Get-TestUserPath) 'C:\Windows;C:\Tools'

        Assert-Success (Invoke-Installer @('-Source', $archive, '-InstallDir', $installDir, '-Force')) 'reinstall after uninstall'
        Assert-Success (Invoke-Installer @('-Uninstall', '-InstallDir', $installDir, '-Force', '-RemoveInstallDir')) 'uninstall with files'
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $installDir 'find.exe'))) 'A recorded executable was not removed.'
        Assert-True (-not (Test-Path -LiteralPath (Join-Path $installDir 'share'))) 'The share directory was not removed.'
        Assert-True (-not (Test-Path -LiteralPath $installDir)) 'The empty install directory was not removed.'
        Assert-Equal (Get-TestUserPath) 'C:\Windows;C:\Tools'
    }

    Write-Host "Install script tests passed: $($script:Passed) cases."
} finally {
    Remove-Item Env:RUXCMD_INSTALL_USER_PATH_FILE -ErrorAction SilentlyContinue
    Remove-OwnedDirectory $temporary $temporaryParent
}

#Requires -Version 7.0
# Compile a tiny, independent workspace; never launch the generated executable.
. (Join-Path $PSScriptRoot '../common.ps1')
$fixtureParent = [IO.Path]::GetTempPath()
$fixture = Join-Path $fixtureParent "ruxcmd resource fixture $([guid]::NewGuid().ToString('N'))"
$encoding = [Text.UTF8Encoding]::new($false)
try {
    $crate = Join-Path $fixture 'crates/resource-probe'
    [IO.Directory]::CreateDirectory((Join-Path $crate 'src')) | Out-Null
    [IO.Directory]::CreateDirectory((Join-Path $fixture 'build')) | Out-Null
    $helperPath = (Join-Path $script:RepositoryRoot 'crates/ruxcmd-build').Replace('\', '/')
    $helperLiteral = ConvertTo-Json -InputObject $helperPath -Compress
    $workspace = @'
[workspace]
members = ["crates/resource-probe"]
resolver = "3"
[workspace.metadata.ruxcmd]
version = "1.2.3-rc.1"
[workspace.metadata.ruxcmd.windows]
productName = "RuxCmd"
companyName = "Tim and RuxCmd contributors"
copyright = "Copyright (c) 2026 Tim and RuxCmd contributors"
'@
    $package = @'
[package]
name = "resource-probe"
version = "0.1.0"
edition = "2024"
build = "../../build/windows-version.rs"
description = '列出 "files" at C:\temp'
[[bin]]
name = "probe-alias"
path = "src/main.rs"
[build-dependencies]
'@
    $package += "`nruxcmd-build = { path = $helperLiteral }`n"
    $workspacePath = Join-Path $fixture 'Cargo.toml'
    [IO.File]::WriteAllText($workspacePath, $workspace, $encoding)
    [IO.File]::WriteAllText((Join-Path $crate 'Cargo.toml'), $package, $encoding)
    [IO.File]::WriteAllText((Join-Path $crate 'src/main.rs'), 'fn main() {}', $encoding)
    Copy-Item -LiteralPath (Join-Path $script:RepositoryRoot 'build/windows-version.rs') -Destination (Join-Path $fixture 'build/windows-version.rs')
    $target = 'x86_64-pc-windows-msvc'
    $targetDir = Join-Path $fixture 'target with spaces'
    $arguments = @('build', '--offline', '--target', $target, '--target-dir', $targetDir)
    Assert-ToolSuccess (Invoke-ToolProcess cargo ($arguments + @('--release')) -WorkingDirectory $fixture -TimeoutSeconds 180) 'resource fixture release build'
    $releaseExe = Join-Path $targetDir "$target/release/probe-alias.exe"
    $info = [Diagnostics.FileVersionInfo]::GetVersionInfo($releaseExe)
    if ($info.ProductVersion -cne '1.2.3-rc.1' -or $info.FileVersion -cne '1.2.3.0' -or
        $info.FileDescription -cne '列出 "files" at C:\temp' -or $info.OriginalFilename -cne 'probe-alias.exe' -or
        -not $info.IsPreRelease -or $info.IsDebug) { throw 'Unicode/quote/backslash/alias/prerelease resource roundtrip failed.' }
    Write-Host 'PASS: real RC Unicode, quotes, backslashes, spaced paths and binary alias roundtrip'
    $workspace = $workspace.Replace('1.2.3-rc.1', '1.2.4+build-7')
    [IO.File]::WriteAllText($workspacePath, $workspace, $encoding)
    Assert-ToolSuccess (Invoke-ToolProcess cargo ($arguments + @('--release', '--locked')) -WorkingDirectory $fixture -TimeoutSeconds 180) 'resource root-version rebuild'
    $info = [Diagnostics.FileVersionInfo]::GetVersionInfo($releaseExe)
    if ($info.ProductVersion -cne '1.2.4+build-7' -or $info.FileVersion -cne '1.2.4.0' -or $info.IsPreRelease) { throw 'Root suite-version-only change left stale resources.' }
    $expectedWorkspace = [pscustomobject]@{
        Version = '1.2.4+build-7'
        Windows = [pscustomobject]@{ productName = 'RuxCmd'; companyName = 'Tim and RuxCmd contributors'; copyright = 'Copyright (c) 2026 Tim and RuxCmd contributors' }
    }
    $expectedBinary = [pscustomobject]@{ Name = 'probe-alias'; Description = '列出 "files" at C:\temp' }
    Assert-BinaryVersionInfo $releaseExe $expectedBinary $expectedWorkspace | Out-Null
    Write-Host 'PASS: root suite-version-only changes invalidate cached resources'
    Assert-ToolSuccess (Invoke-ToolProcess cargo ($arguments + @('--locked')) -WorkingDirectory $fixture -TimeoutSeconds 180) 'resource debug build'
    $debugExe = Join-Path $targetDir "$target/debug/probe-alias.exe"
    $info = [Diagnostics.FileVersionInfo]::GetVersionInfo($debugExe)
    if (-not $info.IsDebug -or $info.IsPreRelease -or $info.ProductVersion -cne '1.2.4+build-7') { throw 'Debug resource flags are incorrect.' }
    Assert-BinaryVersionInfo $debugExe $expectedBinary $expectedWorkspace -DebugBuild | Out-Null
    Write-Host 'PASS: debug and release resources use the correct build flags'
    Write-Host 'Resource build tests passed: 3 cases.'
} finally {
    Remove-OwnedDirectory $fixture $fixtureParent
}

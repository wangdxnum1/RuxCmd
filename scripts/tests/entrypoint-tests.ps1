#Requires -Version 7.0
# Real subprocesses and a disposable Git/Cargo workspace; never contacts GitHub.
param()
. (Join-Path $PSScriptRoot '../common.ps1')
. (Join-Path $PSScriptRoot '../release-helpers.ps1')
$script:Passed = 0
function Test-Entry([string]$Name, [scriptblock]$Body) {
    & $Body
    $script:Passed++
    Write-Host "PASS: $Name"
}
function Assert-EntryThrows([scriptblock]$Body, [string]$Pattern) {
    try { & $Body | Out-Null } catch {
        if ($_.Exception.Message -notmatch $Pattern) { throw "Wrong failure: $($_.Exception.Message); expected $Pattern" }
        return
    }
    throw "Expected failure matching $Pattern"
}
function Assert-EntryProcess($Result, [int]$Code, [string]$Pattern) {
    if (($Code -eq 0 -and $Result.ExitCode -ne 0) -or ($Code -ne 0 -and $Result.ExitCode -eq 0) -or ($Result.Output + $Result.Error) -notmatch $Pattern) {
        throw "Unexpected subprocess result ($($Result.ExitCode)): $($Result.Output) $($Result.Error)"
    }
}
$pwsh = Join-Path $PSHOME 'pwsh.exe'
$fixtureParent = Join-Path $script:RepositoryRoot '.infrastructure-work'
$fixture = Join-Path $fixtureParent "entrypoint fixture $([guid]::NewGuid().ToString('N'))"
[IO.Directory]::CreateDirectory((Join-Path $fixture 'scripts')) | Out-Null
[IO.Directory]::CreateDirectory((Join-Path $fixture 'src')) | Out-Null
try {
    foreach ($scriptName in @('build.ps1', 'release.ps1', 'common.ps1', 'release-helpers.ps1', 'process-job.ps1')) {
        Copy-Item -LiteralPath (Join-Path $script:RepositoryRoot "scripts/$scriptName") -Destination (Join-Path $fixture 'scripts')
    }
    @'
[package]
name = "fixture"
version = "0.1.0"
edition = "2021"
[workspace]
[workspace.metadata.ruxcmd]
version = "0.1.0"
target = "x86_64-pc-windows-msvc"
[[bin]]
name = "fixture-command"
path = "src/main.rs"
'@ | Set-Content -LiteralPath (Join-Path $fixture 'Cargo.toml') -Encoding utf8NoBOM
    'fn main(){println!("fixture");}' | Set-Content -LiteralPath (Join-Path $fixture 'src/main.rs') -Encoding utf8NoBOM
    'Fixture release notes.' | Set-Content -LiteralPath (Join-Path $fixture 'CHANGELOG.md') -Encoding utf8NoBOM
    'target/' | Set-Content -LiteralPath (Join-Path $fixture '.gitignore') -Encoding utf8NoBOM
    Assert-ToolSuccess (Invoke-ToolProcess cargo @('generate-lockfile', '--offline') -WorkingDirectory $fixture) 'fixture lockfile'
    foreach ($gitArguments in @(@('init'), @('config', 'user.name', 'RuxCmd tests'), @('config', 'user.email', 'tests@example.invalid'), @('add', '.'), @('commit', '-m', 'fixture baseline'))) {
        Assert-ToolSuccess (Invoke-ToolProcess git $gitArguments -WorkingDirectory $fixture) 'fixture Git setup'
    }
    $build = Join-Path $fixture 'scripts/build.ps1'
    $release = Join-Path $fixture 'scripts/release.ps1'
    Test-Entry 'preview resolves binary aliases, jobs and offline without creating outputs' {
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $build, '-Package', 'fixture-command', '-Configuration', 'Debug', '-Jobs', '2', '-Offline', '-WhatIf') -WorkingDirectory $script:RepositoryRoot
        Assert-EntryProcess $result 0 'cargo build -p fixture --bins --target x86_64-pc-windows-msvc --locked --offline --jobs 2'
        if (Test-Path -LiteralPath (Join-Path $fixture 'target')) { throw 'Preview created build outputs.' }
    }
    Test-Entry 'unknown package fails before Cargo build' {
        Assert-EntryProcess (Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $build, '-Package', 'missing')) 1 'Unknown command/package'
    }
    Test-Entry 'full Verify refuses partial package selection' {
        Assert-EntryProcess (Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $build, '-Task', 'Verify', '-Package', 'fixture')) 1 'entire workspace'
    }
    Test-Entry 'Cargo formatting failure propagates and stops later steps' {
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $build, '-Task', 'Check') -TimeoutSeconds 120
        Assert-EntryProcess $result 1 'Formatting failed'
        if ($result.Output -match '== Clippy ==|== Check ==') { throw 'Check continued after formatting failure.' }
    }
    Test-Entry 'actual Debug build succeeds from another working directory' {
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $build, '-Package', 'fixture-command', '-Configuration', 'Debug', '-Offline') -TimeoutSeconds 120
        Assert-EntryProcess $result 0 'Build completed'
        if (-not (Test-Path -LiteralPath (Join-Path $fixture 'target/x86_64-pc-windows-msvc/debug/fixture-command.exe'))) { throw 'Debug executable is missing.' }
    }
    Test-Entry 'clean preview does not delete outputs' {
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $build, '-Task', 'Clean', '-Configuration', 'Debug', '-WhatIf')
        Assert-EntryProcess $result 0 'cargo clean --target x86_64-pc-windows-msvc --profile dev'
        if (-not (Test-Path -LiteralPath (Join-Path $fixture 'target/x86_64-pc-windows-msvc/debug/fixture-command.exe'))) { throw 'Clean preview deleted outputs.' }
    }
    Test-Entry 'actual clean removes only selected configuration' {
        $releaseDirectory = Join-Path $fixture 'target/x86_64-pc-windows-msvc/release'
        [IO.Directory]::CreateDirectory($releaseDirectory) | Out-Null
        'keep release output' | Set-Content -LiteralPath (Join-Path $releaseDirectory 'sentinel.txt')
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $build, '-Task', 'Clean', '-Configuration', 'Debug')
        Assert-EntryProcess $result 0 'Clean completed'
        if (Test-Path -LiteralPath (Join-Path $fixture 'target/x86_64-pc-windows-msvc/debug/fixture-command.exe')) { throw 'Clean left selected Debug output behind.' }
        if (-not (Test-Path -LiteralPath (Join-Path $releaseDirectory 'sentinel.txt'))) { throw 'Clean removed another configuration.' }
    }
    Test-Entry 'default build produces optimized Release output' {
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $build, '-Offline') -TimeoutSeconds 120
        Assert-EntryProcess $result 0 'Build completed'
        if (-not (Test-Path -LiteralPath (Join-Path $fixture 'target/x86_64-pc-windows-msvc/release/fixture-command.exe'))) { throw 'Release executable is missing.' }
    }
    Test-Entry 'local release preview does not build or tag' {
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $release, '-WhatIf')
        Assert-EntryProcess $result 0 'Build and verify a local release ZIP'
        if (Test-Path -LiteralPath (Join-Path $fixture 'dist')) { throw 'Release preview created output.' }
    }
    Test-Entry 'publish preview on a clean fixture has no remote effects or local tag' {
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $release, '-Publish', '-WhatIf')
        Assert-EntryProcess $result 0 'publish v0.1.0 on GitHub'
        $tags = Invoke-ToolProcess git @('tag', '--list') -WorkingDirectory $fixture
        Assert-ToolSuccess $tags 'fixture tags'
        if ($tags.Output.Trim()) { throw 'Publish preview created a tag.' }
    }
    Test-Entry 'release version mismatch and missing tags fail before packaging' {
        Assert-EntryProcess (Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $release, '-Tag', 'v0.2.0')) 1 'does not match'
        Assert-EntryProcess (Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $release, '-Tag', 'v0.1.0')) 1 'does not exist locally'
    }
    Test-Entry 'publish-only options require explicit Publish' {
        Assert-EntryProcess (Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $release, '-Draft')) 1 'require -Publish'
    }
    Test-Entry 'existing release output is preserved' {
        $output = Join-Path $fixture 'dist'
        [IO.Directory]::CreateDirectory($output) | Out-Null
        $existing = Join-Path $output 'ruxcmd-0.1.0-x86_64-pc-windows-msvc.zip'
        'preserve me' | Set-Content -LiteralPath $existing
        Assert-EntryProcess (Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $release)) 1 'already exists'
        if ((Get-Content -LiteralPath $existing -Raw).Trim() -cne 'preserve me') { throw 'Existing output was overwritten.' }
    }
    Test-Entry 'Publish refuses untracked files before invoking GitHub' {
        'untracked' | Set-Content -LiteralPath (Join-Path $fixture 'untracked.txt')
        Assert-EntryProcess (Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $release, '-Publish', '-OutputDir', (Join-Path $fixture 'another output'))) 1 'clean checkout'
    }
    Test-Entry 'only GitHub HTTPS and SSH remotes are accepted' {
        foreach ($remote in @('https://github.com/owner/repo.git', 'git@github.com:owner/repo.git', 'ssh://git@github.com/owner/repo.git')) {
            if ((Get-GitHubRepository $remote) -cne 'owner/repo') { throw "Wrong repository for $remote" }
        }
        Assert-EntryThrows { Get-GitHubRepository 'https://github.com.example.invalid/owner/repo' } 'origin must point'
    }
    Test-Entry 'local and remote tag conflicts are rejected, annotated tags use peeled commit' {
        Assert-EntryThrows { Assert-ReleaseSource '' 'abc' 'def' } 'another commit'
        Assert-EntryThrows { Assert-RemoteTag "def`trefs/tags/v0.1.0" 'v0.1.0' 'abc' } 'another commit'
        Assert-RemoteTag "object`trefs/tags/v0.1.0`nabc`trefs/tags/v0.1.0^{}" 'v0.1.0' 'abc'
        Assert-RemoteTag "abc`trefs/tags/v0.1.0" 'v0.1.0' 'abc'
        Assert-RemoteTag '' 'v0.1.0' 'abc'
    }
    Test-Entry 'release assets reject wrong commit, dirty source and checksum tampering' {
        $content = Join-Path $fixture 'assets'
        [IO.Directory]::CreateDirectory($content) | Out-Null
        $manifest = @{ suiteVersion = '0.1.0'; target = 'x86_64-pc-windows-msvc'; sourceCommit = 'abc'; dirty = $true }
        $manifest | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $content 'manifest.json')
        $archive = Join-Path $fixture 'assets.zip'
        [IO.Compression.ZipFile]::CreateFromDirectory($content, $archive)
        $checksum = "$archive.sha256"
        "$(Get-FileDigest $archive)  assets.zip" | Set-Content -LiteralPath $checksum
        $workspace = [pscustomobject]@{ Version = '0.1.0'; Target = 'x86_64-pc-windows-msvc' }
        Assert-ReleaseAssets $archive $checksum $workspace 'abc'
        Assert-EntryThrows { Assert-ReleaseAssets $archive $checksum $workspace 'def' } 'manifest does not match'
        Assert-EntryThrows { Assert-ReleaseAssets $archive $checksum $workspace 'abc' -RequireClean } 'dirty release manifest'
        'bad' | Set-Content -LiteralPath $checksum
        Assert-EntryThrows { Assert-ReleaseAssets $archive $checksum $workspace 'abc' } 'checksum mismatch'
    }
    Test-Entry 'Verify forwards Offline to child checks and propagates their failure' {
        @'
fn main() {
    println!("fixture");
}
'@ | Set-Content -LiteralPath (Join-Path $fixture 'src/main.rs') -Encoding utf8NoBOM
        @'
param([string]$BinDir)
if ($env:CARGO_NET_OFFLINE -ne 'true') { throw 'Offline was not passed to verification.' }
throw 'Offline child confirmed; controlled failure.'
'@ | Set-Content -LiteralPath (Join-Path $fixture 'scripts/smoke-test.ps1') -Encoding utf8NoBOM
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', $build, '-Task', 'Verify', '-Offline') -TimeoutSeconds 120
        Assert-EntryProcess $result 1 'Offline child confirmed; controlled failure'
        if ($result.Output -match 'Verify completed') { throw 'Verify claimed success after child failure.' }
    }
    Write-Host "Entrypoint tests passed: $script:Passed cases."
} finally { Remove-OwnedDirectory $fixture $fixtureParent }

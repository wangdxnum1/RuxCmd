#Requires -Version 7.0
param([string]$BinDir)
. (Join-Path $PSScriptRoot '../common.ps1')
$workspace = Get-WorkspaceInfo
if (-not $BinDir) { $BinDir = Join-Path $script:RepositoryRoot "target/$($workspace.Target)/release" }
$BinDir = [IO.Path]::GetFullPath($BinDir)
$temporaryParent = [IO.Path]::GetTempPath()
$temporary = Join-Path $temporaryParent "ruxcmd-tests-$([guid]::NewGuid().ToString('N'))"
[IO.Directory]::CreateDirectory($temporary) | Out-Null
$script:Passed = 0
function Test-Case([string]$Name, [scriptblock]$Body) {
    & $Body
    $script:Passed++
    Write-Host "PASS: $Name"
}
function Assert-Throws([scriptblock]$Body, [string]$Pattern) {
    try { & $Body | Out-Null } catch {
        if ($_.Exception.Message -notmatch $Pattern) { throw "Wrong failure: $($_.Exception.Message); expected $Pattern" }
        return
    }
    throw "Expected failure matching $Pattern"
}
function Assert-Equal($Actual, $Expected) { if ($Actual -cne $Expected) { throw "Expected <$Expected>, got <$Actual>" } }
try {
    $pwsh = Join-Path $PSHOME 'pwsh.exe'
    Test-Case 'positive and negative release tag validation' {
        Assert-ReleaseVersion '0.1.0' 'v0.1.0'
        Assert-Throws { Assert-ReleaseVersion '0.1.0' 'v0.2.0' } 'does not match'
        Assert-Throws { Assert-ReleaseVersion '../bad' '' } 'Invalid suite version'
    }
    Test-Case 'expected false exit code with zero arguments' {
        $result = Invoke-ToolProcess (Join-Path $BinDir 'false.exe') -Arguments @()
        Assert-Equal $result.ExitCode 1
        Assert-Equal $result.Error ''
    }
    Test-Case 'spaces in executable directory and arguments' {
        $withSpaces = Join-Path $temporary 'directory with spaces'
        [IO.Directory]::CreateDirectory($withSpaces) | Out-Null
        Copy-Item -LiteralPath (Join-Path $BinDir 'echo.exe') -Destination $withSpaces
        $result = Invoke-ToolProcess (Join-Path $withSpaces 'echo.exe') @('an argument with spaces') -WorkingDirectory $withSpaces
        Assert-Equal $result.ExitCode 0
        Assert-Equal $result.Output.TrimEnd() 'an argument with spaces'
    }
    Test-Case 'nonzero process exit propagation' {
        Assert-Throws { Assert-ToolSuccess (Invoke-ToolProcess $pwsh @('-NoProfile', '-Command', 'exit 7')) 'fixture' } 'exited 7'
    }
    Test-Case 'timeout terminates own child process' {
        $pidPath = Join-Path $temporary 'child.pid'
        $escaped = $pidPath.Replace("'", "''")
        Assert-Throws { Invoke-ToolProcess $pwsh @('-NoProfile', '-Command', "[IO.File]::WriteAllText('$escaped', [string]`$PID); Start-Sleep -Seconds 30") -TimeoutSeconds 2 } 'Timeout'
        $childId = [int](Get-Content -LiteralPath $pidPath -Raw)
        if (Get-Process -Id $childId -ErrorAction SilentlyContinue) { throw 'Timed out child still running.' }
    }
    Test-Case 'blocked stdin also respects timeout' {
        Assert-Throws { Invoke-ToolProcess $pwsh @('-NoProfile', '-Command', 'Start-Sleep -Seconds 30') -InputText ('a' * 1000000) -TimeoutSeconds 1 } 'Timeout'
    }
    Test-Case 'large output is drained without deadlock' {
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-Command', "[Console]::Out.Write(('x' * 200000)); [Console]::Error.Write(('y' * 200000))") -TimeoutSeconds 10
        Assert-Equal $result.ExitCode 0
        if (-not $result.Output.EndsWith(('x' * 200000)) -or -not $result.Error.EndsWith(('y' * 200000))) { throw 'Large output was truncated.' }
    }
    $fake = [pscustomobject]@{ Binaries = @([pscustomobject]@{ Name = 'ps'; PackageId = 'fixture#ps-bin'; PackageVersion = '0.1.0' }) }
    $exe = Join-Path $temporary 'ps.exe'
    [IO.File]::WriteAllText($exe, 'fixture')
    $message = [ordered]@{ reason = 'compiler-artifact'; package_id = 'fixture#ps-bin'; target = @{ kind = @('bin'); name = 'ps' }; executable = $exe } | ConvertTo-Json -Depth 4 -Compress
    Test-Case 'ps-bin maps to ps executable' {
        $artifacts = Get-BinaryArtifacts $message $fake
        Assert-Equal $artifacts['ps'] $exe
    }
    Test-Case 'stale or missing artifacts cannot enter a package' {
        [IO.File]::WriteAllText((Join-Path $temporary 'stale.exe'), 'stale')
        $artifacts = Get-BinaryArtifacts $message $fake
        Assert-Equal $artifacts.Count 1
        Assert-Throws { Get-BinaryArtifacts '' $fake } 'Missing artifact: ps'
        Assert-Throws { Get-BinaryArtifacts "$message`n$message" $fake } 'Duplicate artifact'
        $unknown = $message.Replace('"name":"ps"', '"name":"unknown"')
        Assert-Throws { Get-BinaryArtifacts $unknown $fake } 'Unexpected binary artifact'
    }
    Test-Case 'smoke strategy drift is rejected' {
        $policy = Join-Path $temporary 'policy.json'
        '[{"name":"other","arguments":["--help"],"exitCode":0}]' | Set-Content -LiteralPath $policy
        Assert-Throws { Get-SmokeCases $policy $fake } 'cover every'
        '[{"name":"ps","arguments":["--help"],"exitCode":0},{"name":"ps","arguments":[],"exitCode":0}]' | Set-Content -LiteralPath $policy
        Assert-Throws { Get-SmokeCases $policy $fake } 'cover every'
    }
    $content = Join-Path $temporary 'archive'
    [IO.Directory]::CreateDirectory((Join-Path $content 'bin')) | Out-Null
    Copy-Item -LiteralPath $exe -Destination (Join-Path $content 'bin/ps.exe')
    $digest = Get-FileDigest (Join-Path $content 'bin/ps.exe')
    $manifest = [ordered]@{ files = @(@{ path = 'bin/ps.exe'; sha256 = $digest }); binaries = @(@{ name = 'ps'; file = 'bin/ps.exe'; sha256 = $digest }) }
    $manifest | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $content 'manifest.json')
    "$digest  bin/ps.exe" | Set-Content -LiteralPath (Join-Path $content 'SHA256SUMS')
    Test-Case 'extracted archive validates and detects tampering' {
        Assert-ArchiveDirectory $content $manifest
        [IO.File]::WriteAllText((Join-Path $content 'bin/ps.exe'), 'tampered')
        Assert-Throws { Assert-ArchiveDirectory $content $manifest } 'checksum mismatch'
        Copy-Item -LiteralPath $exe -Destination (Join-Path $content 'bin/ps.exe') -Force
    }
    Test-Case 'extra archive files and corrupted checksum index are rejected' {
        $extra = Join-Path $content 'old.exe'
        [IO.File]::WriteAllText($extra, 'old')
        Assert-Throws { Assert-ArchiveDirectory $content $manifest } 'file set'
        Remove-Item -LiteralPath $extra
        'bad checksum index' | Set-Content -LiteralPath (Join-Path $content 'SHA256SUMS')
        Assert-Throws { Assert-ArchiveDirectory $content $manifest } 'SHA256SUMS differs'
    }
    Test-Case 'cleanup refuses parent itself' {
        Assert-Throws { Remove-OwnedDirectory $temporary $temporary } 'Refusing cleanup'
    }
    Test-Case 'existing releases are never overwritten even from another working directory' {
        $output = Join-Path $temporary 'output with spaces'
        [IO.Directory]::CreateDirectory($output) | Out-Null
        $zip = Join-Path $output "ruxcmd-$($workspace.Version)-$($workspace.Target).zip"
        [IO.File]::WriteAllText($zip, 'keep existing archive')
        $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', (Join-Path $script:RepositoryRoot 'scripts/package.ps1'), '-OutputDir', $output) -WorkingDirectory $temporary
        if ($result.ExitCode -eq 0 -or $result.Error -notmatch 'already exists') { throw 'Existing archive was not rejected.' }
        Assert-Equal ([IO.File]::ReadAllText($zip)) 'keep existing archive'
    }
    Test-Case 'formal releases refuse dirty checkout' {
        $status = Invoke-ToolProcess git @('status', '--porcelain')
        if (-not [string]::IsNullOrWhiteSpace($status.Output)) {
            $result = Invoke-ToolProcess $pwsh @('-NoProfile', '-File', (Join-Path $script:RepositoryRoot 'scripts/package.ps1'), '-RequireClean', '-OutputDir', (Join-Path $temporary 'formal'))
            if ($result.ExitCode -eq 0 -or $result.Error -notmatch 'clean checkout') { throw 'Dirty formal release was not rejected.' }
        }
    }
    Write-Host "Package tests passed: $script:Passed cases."
} finally { Remove-OwnedDirectory $temporary $temporaryParent }

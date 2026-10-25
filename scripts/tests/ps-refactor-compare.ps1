#Requires -Version 7.0
# Optional acceptance check against a locally preserved pre-refactor ps.exe.
# Only queries this script's process; does not signal processes or open windows.
param([Parameter(Mandatory)][string]$BaselineBinary, [string]$NewBinary)
. (Join-Path $PSScriptRoot '../common.ps1')
$BaselineBinary = Get-FileSystemPath $BaselineBinary
if (-not $NewBinary) { $NewBinary = Join-Path $script:RepositoryRoot 'target/x86_64-pc-windows-msvc/release/ps.exe' }
$NewBinary = Get-FileSystemPath $NewBinary
if ([IO.Path]::GetFileName($BaselineBinary) -cne [IO.Path]::GetFileName($NewBinary)) { throw 'Keep the same executable filename in separate directories when comparing CLI usage.' }
$stable = 'pid,ppid,pgid,pgrp,tid,sid,sess,user,uid,gid,group,ruser,rgroup,suser,nice,ni,pri,class,stat,flags,f,tty,tt,comm,args,cmd,command,c,cp,lstart,start,stime'
$selection = @('--no-cmdline', '-p', [string]$PID, '-o', $stable)
$sid = [Security.Principal.WindowsIdentity]::GetCurrent().User.Value
$cases = @(
    @{ Name = 'help'; Arguments = @('--help'); Code = 0 }
    @{ Name = 'version'; Arguments = @('--version'); Code = 0 }
    @{ Name = 'registry names and order'; Arguments = @('--list-columns'); Code = 0 }
    @{ Name = 'stable fields and aliases'; Arguments = $selection; Code = 0 }
    @{ Name = 'headers omitted'; Arguments = $selection + @('--no-headers'); Code = 0 }
    @{ Name = 'ASCII tree'; Arguments = $selection + @('--forest', '--ascii'); Code = 0 }
    @{ Name = 'Unicode tree'; Arguments = $selection + @('--forest'); Code = 0 }
    @{ Name = 'empty selection'; Arguments = @('--no-cmdline', '-p', '4294967295'); Code = 0 }
    @{ Name = 'invalid column error'; Arguments = @('-o', 'does-not-exist'); Code = 2 }
    @{ Name = 'mixed unknown column warning'; Arguments = @('--no-cmdline', '-p', [string]$PID, '-o', 'pid,does-not-exist,comm'); Code = 0 }
    @{ Name = 'canonical alias sort key'; Arguments = $selection + @('--sort=-nice,pid'); Code = 0 }
    @{ Name = 'user SID selection'; Arguments = $selection + @('-u', $sid); Code = 0 }
    @{ Name = 'command stem selection'; Arguments = $selection + @('-C', (Get-Process -Id $PID).ProcessName); Code = 0 }
)
$passed = 0
foreach ($case in $cases) {
    $before = Invoke-ToolProcess $BaselineBinary $case.Arguments -TimeoutSeconds 30
    $after = Invoke-ToolProcess $NewBinary $case.Arguments -TimeoutSeconds 30
    if ($before.ExitCode -ne $case.Code -or $after.ExitCode -ne $before.ExitCode -or
        $before.Output -cne $after.Output -or $before.Error -cne $after.Error) {
        throw "ps refactor parity mismatch: $($case.Name)"
    }
    $passed++
    Write-Host "PASS: old/new ps $($case.Name)"
}
$parent = [IO.Path]::GetTempPath()
$temporary = Join-Path $parent "ruxcmd-ps-output-$([guid]::NewGuid().ToString('N'))"
[IO.Directory]::CreateDirectory($temporary) | Out-Null
try {
    $beforeFile = Join-Path $temporary 'before.txt'
    $afterFile = Join-Path $temporary 'after.txt'
    $before = Invoke-ToolProcess $BaselineBinary ($selection + @('--write', $beforeFile)) -TimeoutSeconds 30
    $after = Invoke-ToolProcess $NewBinary ($selection + @('--write', $afterFile)) -TimeoutSeconds 30
    if ($before.ExitCode -ne 0 -or $after.ExitCode -ne 0 -or $before.Output -cne $after.Output -or
        $before.Error -cne $after.Error -or (Get-FileDigest $beforeFile) -cne (Get-FileDigest $afterFile)) {
        throw 'ps refactor parity mismatch: file output'
    }
    $passed++
    Write-Host 'PASS: old/new ps file output'
} finally { Remove-OwnedDirectory $temporary $parent }
Write-Host "ps refactor parity passed: $passed cases."

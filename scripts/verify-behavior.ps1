#Requires -Version 7.0
param([Parameter(Mandatory)][string]$BinDir)
. (Join-Path $PSScriptRoot 'common.ps1')
$directory = [IO.Path]::GetFullPath($BinDir)
$tests = @(
    @{ Name = 'cat'; Arguments = @(); Input = "hello`n"; Output = "hello`n"; Exit = 0 },
    @{ Name = 'echo'; Arguments = @('hello', 'world'); Input = ''; Output = "hello world`n"; Exit = 0 },
    @{ Name = 'wc'; Arguments = @('-l'); Input = "one`ntwo`n"; Output = "2 <stdin>`n"; Exit = 0 },
    @{ Name = 'grep'; Arguments = @('needle'); Input = "hay`nneedle`n"; Output = "needle`n"; Exit = 0 },
    @{ Name = 'grep'; Arguments = @('missing'); Input = "hay`n"; Output = ''; Exit = 1 },
    @{ Name = 'true'; Arguments = @(); Input = ''; Output = ''; Exit = 0 },
    @{ Name = 'false'; Arguments = @(); Input = ''; Output = ''; Exit = 1 }
)
foreach ($test in $tests) {
    $result = Invoke-ToolProcess (Join-Path $directory "$($test.Name).exe") -Arguments $test.Arguments -InputText $test.Input -TimeoutSeconds 10
    if ($result.ExitCode -ne $test.Exit -or $result.Output.Replace("`r`n", "`n") -cne $test.Output) { throw "Behavior check failed: $($test.Name), exit=$($result.ExitCode), output=$($result.Output), error=$($result.Error)" }
}
$ps = Invoke-ToolProcess (Join-Path $directory 'ps.exe') @('--list-columns') -TimeoutSeconds 10
Assert-ToolSuccess $ps 'ps list columns'
if ($ps.Output -notmatch '(?m)^pid\s*$') { throw 'ps column list missing pid.' }
Write-Host 'Verified 8 representative behaviors from extracted binaries.'

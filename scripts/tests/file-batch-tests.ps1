#Requires -Version 7.0
param([Parameter(Mandatory)][string]$BinDir)
. (Join-Path $PSScriptRoot '../common.ps1')
$batchBins = Get-FileSystemPath $BinDir
foreach ($name in @('find', 'xargs', 'cp', 'sha256sum')) {
    if (-not (Test-Path -LiteralPath (Join-Path $batchBins "$name.exe") -PathType Leaf)) { throw "Missing file batch executable: $name" }
}

function New-BatchProcess([string]$Name, [string[]]$Arguments) {
    $process = [Diagnostics.Process]::new()
    $info = $process.StartInfo
    $info.FileName = Join-Path $batchBins "$Name.exe"
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.StandardErrorEncoding = [Text.Encoding]::UTF8
    foreach ($argument in $Arguments) { $info.ArgumentList.Add($argument) }
    return $process
}

# Connect BaseStreams, never PowerShell's text pipeline. One deadline covers
# the producer, consumer, pipe and both output streams; jobs own descendants.
function Invoke-BatchPipe([string[]]$FindArguments, [string[]]$XargsArguments) {
    $producer = New-BatchProcess 'find' $FindArguments
    $consumer = New-BatchProcess 'xargs' $XargsArguments
    $owner = [RuxCmd.ProcessJob]::new()
    $clock = [Diagnostics.Stopwatch]::StartNew()
    function Wait-BatchTask($Task) {
        $remaining = [Math]::Max(1, 120000 - [int]$clock.ElapsedMilliseconds)
        if (-not $Task.Wait($remaining)) { throw 'File batch pipeline timed out.' }
    }
    try {
        if (-not $consumer.Start()) { throw 'Cannot start xargs.' }
        try { $owner.Attach($consumer.Handle) } catch { if (-not $consumer.HasExited) { throw } }
        $consumerError = $consumer.StandardError.ReadToEndAsync()
        $consumerOutput = $consumer.StandardOutput.ReadToEndAsync()
        if (-not $producer.Start()) { throw 'Cannot start find.' }
        try { $owner.Attach($producer.Handle) } catch { if (-not $producer.HasExited) { throw } }
        $producer.StandardInput.Close()
        $producerError = $producer.StandardError.ReadToEndAsync()
        $pipe = $producer.StandardOutput.BaseStream.CopyToAsync($consumer.StandardInput.BaseStream)
        $pipeError = $null
        try { Wait-BatchTask $pipe } catch { $pipeError = $_ } finally { $consumer.StandardInput.Close() }
        foreach ($process in @($producer, $consumer)) {
            $remaining = [Math]::Max(1, 120000 - [int]$clock.ElapsedMilliseconds)
            if (-not $process.WaitForExit($remaining)) { throw 'File batch process timed out.' }
        }
        foreach ($task in @($producerError, $consumerError, $consumerOutput)) { Wait-BatchTask $task }
        if ($producer.ExitCode -ne 0 -or $consumer.ExitCode -ne 0) {
            throw "File batch pipeline failed: find=$($producer.ExitCode), xargs=$($consumer.ExitCode); $($producerError.Result) $($consumerError.Result)"
        }
        if ($pipeError) { throw $pipeError }
    } finally {
        $owner.Dispose()
        $producer.Dispose()
        $consumer.Dispose()
    }
}

$batchParent = [IO.Path]::GetTempPath()
$batchRoot = Join-Path $batchParent "rux-batch-$([guid]::NewGuid().ToString('N'))"
try {
    $source = Join-Path $batchRoot 'source 中文'
    $pipeDestination = Join-Path $batchRoot 'pipe result'
    $findDestination = Join-Path $batchRoot 'find result'
    $emptyDestination = Join-Path $batchRoot 'empty result'
    foreach ($path in @($source, $pipeDestination, $findDestination, $emptyDestination)) { [IO.Directory]::CreateDirectory($path) | Out-Null }
    $names = @(for ($i = 0; $i -lt 80; $i++) { '{0:D3}-中文😀 &^%''()-{1}.txt' -f $i, ('a' * 65) })
    $names += '-leading name.txt'
    foreach ($name in $names) { [IO.File]::WriteAllBytes((Join-Path $source $name), [Text.Encoding]::UTF8.GetBytes("payload:$name`n")) }
    [IO.File]::WriteAllBytes((Join-Path $source 'ignored.bin'), [byte[]]@(0,255,13,10))
    $fixtureUnits = ($names | ForEach-Object { (Join-Path $source $_).Length } | Measure-Object -Sum).Sum
    if ($fixtureUnits -le 8000) { throw 'Fixture must require multiple batches.' }
    $cp = Join-Path $batchBins 'cp.exe'
    Invoke-BatchPipe @($source, '-type', 'f', '-name', '*.txt', '-print0') @('-0', '-r', $cp, '-t', $pipeDestination, '--')
    Assert-ToolSuccess (Invoke-ToolProcess (Join-Path $batchBins 'find.exe') @($source, '-type', 'f', '-name', '*.txt', '-exec', $cp, '-t', $findDestination, '--', '{}', '+')) 'find batch copy'
    $manifest = [Collections.Generic.List[string]]::new()
    foreach ($destination in @($pipeDestination, $findDestination)) {
        $actualNames = @(Get-ChildItem -LiteralPath $destination -File | ForEach-Object Name | Sort-Object)
        if (@(Compare-Object ($names | Sort-Object) $actualNames).Count) { throw 'File batch copy lost or added files.' }
        foreach ($name in $names) {
            $expected = (Get-FileHash -LiteralPath (Join-Path $source $name) -Algorithm SHA256).Hash.ToLowerInvariant()
            if ((Get-FileHash -LiteralPath (Join-Path $destination $name) -Algorithm SHA256).Hash.ToLowerInvariant() -cne $expected) { throw "File batch content mismatch: $name" }
            if ($destination -eq $pipeDestination) { $manifest.Add("$expected  $name") }
        }
    }
    Assert-ToolSuccess (Invoke-ToolProcess (Join-Path $batchBins 'sha256sum.exe') @('--check', '-') -WorkingDirectory $pipeDestination -InputText (($manifest -join "`n") + "`n")) 'file batch SHA256 manifest'
    Invoke-BatchPipe @($source, '-name', '*.absent', '-print0') @('-0', '-r', $cp, '-t', $emptyDestination, '--')
    Assert-ToolSuccess (Invoke-ToolProcess (Join-Path $batchBins 'find.exe') @($source, '-name', '*.absent', '-exec', $cp, '-t', $emptyDestination, '--', '{}', '+')) 'empty find batch'
    if (@(Get-ChildItem -LiteralPath $emptyDestination).Count) { throw 'Empty selection created files.' }
    $caught = $false
    try { Invoke-BatchPipe @($source, '-name', '-leading name.txt', '-print0') @('-0', '-r', '__rux_missing_batch_child_169__') }
    catch { if ($_.Exception.Message -notmatch 'xargs=127') { throw }; $caught = $true }
    if (-not $caught) { throw 'Pipeline child failure was not propagated.' }
    Write-Host "File batch scenarios passed: byte pipe, find -exec +, SHA256 manifest, empty selections, child failure ($($names.Count) files)."
} finally { Remove-OwnedDirectory $batchRoot $batchParent }

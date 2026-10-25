#Requires -Version 7.0
<#
.SYNOPSIS
Differential-test RuxCmd commands against the GNU tools shipped with Git.
.DESCRIPTION
Generates deterministic random inputs, runs a matrix of argument cases
through both the RuxCmd executable and the matching GNU tool from the
Git for Windows installation, and compares stdout bytes plus exit codes.
Intentional differences are recorded in the $KnownDifferences table and
reported as SKIP; anything else that diverges is a FAIL worth triaging.
.EXAMPLE
./scripts/difftest.ps1                # full matrix, default seed
.EXAMPLE
./scripts/difftest.ps1 -Cases cat,sort
.EXAMPLE
./scripts/difftest.ps1 -Seed 42 -Verbose
#>
[CmdletBinding()]
param(
    # Seed for the deterministic input generator; the same seed reproduces
    # the same inputs and therefore the same comparison run.
    [int]$Seed = 20261025,

    # Number of input variants generated per input shape.
    [ValidateRange(1, 10)][int]$Variants = 2,

    # Restrict the run to these command names (default: the whole matrix).
    [string[]]$Cases,

    # RuxCmd binary directory; defaults to release, falls back to debug.
    [string]$BinDir,

    # GNU tool directory; defaults to <git>\usr\bin of the git on PATH.
    [string]$GnuBin,

    # Print the compared outputs even when they match.
    [switch]$ShowAll
)

$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# Locate the two tool sets.
# ---------------------------------------------------------------------------
if (-not $BinDir) {
    $release = Join-Path $PSScriptRoot '..' 'target' 'x86_64-pc-windows-msvc' 'release'
    $debug = Join-Path $PSScriptRoot '..' 'target' 'x86_64-pc-windows-msvc' 'debug'
    $BinDir = if (Test-Path (Join-Path $release 'cat.exe')) { $release } else { $debug }
}
if (-not $GnuBin) {
    # git.exe may live in <root>\cmd, <root>in or <root>\mingw64in;
    # walk up until a sibling usrin with cat.exe is found.
    $dir = Split-Path (Get-Command git -ErrorAction Stop).Source -Parent
    $GnuBin = $null
    for ($i = 0; $i -lt 4 -and $dir -and -not $GnuBin; $i++) {
        $probe = Join-Path $dir 'usr' 'bin'
        if (Test-Path (Join-Path $probe 'cat.exe')) { $GnuBin = $probe }
        else { $dir = Split-Path $dir -Parent }
    }
    if (-not $GnuBin) { throw 'Cannot locate GNU tools (usrin) next to the git installation.' }
}
foreach ($dir in @($BinDir, $GnuBin)) {
    if (-not (Test-Path (Join-Path $dir 'cat.exe'))) { throw "cat.exe not found in $dir" }
}
Write-Host "gen-inputs..."
Write-Host "GNU:    $GnuBin"

# ---------------------------------------------------------------------------
# Deterministic input generator (LCG, matches the shape names used below).
# ---------------------------------------------------------------------------
$script:randState = $Seed
function Get-Rand32 {
    $script:randState = [uint64]($script:randState * 1103515245 + 12345) % 2147483648
    return [int]($script:randState % 2147483648)
}
function New-RandText([int]$lines) {
    # ASCII letters, digits, spaces; words of 1-8 chars, 1-6 per line.
    $letters = 'abcdefghijklmnopqrstuvwxyz'
    $sb = [System.Text.StringBuilder]::new()
    for ($i = 0; $i -lt $lines; $i++) {
        $words = 1 + ((Get-Rand32) % 6)
        for ($w = 0; $w -lt $words; $w++) {
            $len = 1 + ((Get-Rand32) % 8)
            for ($c = 0; $c -lt $len; $c++) {
                [void]$sb.Append($letters[((Get-Rand32) % 26)])
            }
            if ($w -lt $words - 1) { [void]$sb.Append(' ') }
        }
        [void]$sb.Append("`n")
    }
    return $sb.ToString()
}

# Input shapes: each variant produces one file per shape.
$shapes = @('ascii', 'crlf', 'nonutf8', 'empty', 'notrail', 'long')
$work = Join-Path ([System.IO.Path]::GetTempPath()) ("difftest-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null

$inputs = @{}   # "$shape$variant" -> file path
for ($v = 0; $v -lt $Variants; $v++) {
    foreach ($shape in $shapes) {
        $path = Join-Path $work ("$shape$v.in")
        switch ($shape) {
            'ascii'    { [System.IO.File]::WriteAllText($path, (New-RandText 40)) }
            'crlf'     { [System.IO.File]::WriteAllText($path, ((New-RandText 40) -replace "`n", "`r`n")) }
            'notrail'  { $t = New-RandText 40; [System.IO.File]::WriteAllText($path, $t.TrimEnd("`n")) }
            'empty'    { [System.IO.File]::WriteAllBytes($path, [byte[]]@()) }
            'long'     { $t = New-RandText 8; $long = ($t -split "`n" | ForEach-Object { $_.PadRight(300, 'x') }) -join "`n"
                         [System.IO.File]::WriteAllText($path, $long) }
            'nonutf8'  {
                # Random bytes with >0x80 sprinkled in, \n-terminated lines.
                $bytes = [System.Collections.Generic.List[byte]]::new()
                for ($i = 0; $i -lt 300; $i++) {
                    $b = (Get-Rand32) % 256
                    if ($b -eq 13) { $b = 97 }        # keep \r out of line middles
                    $bytes.Add([byte]$b)
                    if (($i + 1) % 17 -eq 0) { $bytes.Add(10) }
                }
                [System.IO.File]::WriteAllBytes($path, $bytes.ToArray())
            }
        }
        $inputs["$shape$v"] = $path
    }
}

# ---------------------------------------------------------------------------
# Case matrix. Each case:
#   cmd    - executable name (same on both sides)
#   argv   - argument list; '{in}' is replaced by the input file path,
#            '{inbare}' by the path without the trailing ".in"
#   shapes - which input shapes to run it against (all variants each)
# Known differences are keyed 'cmd|argv-pattern' in $KnownDifferences.
# ---------------------------------------------------------------------------
$allShapes = $shapes
$matrix = @(
    # --- cat -------------------------------------------------------------
    @{ cmd='cat';    argv=@();                shapes=$allShapes }
    @{ cmd='cat';    argv=@('-n');            shapes=@('ascii','notrail','empty') }
    @{ cmd='cat';    argv=@('-b');            shapes=@('ascii') }
    @{ cmd='cat';    argv=@('-s');            shapes=@('ascii') }
    @{ cmd='cat';    argv=@('-E');            shapes=@('ascii','notrail') }
    @{ cmd='cat';    argv=@('-A');            shapes=@('ascii') }
    @{ cmd='cat';    argv=@('-T');            shapes=@('ascii') }
    # --- head / tail -----------------------------------------------------
    @{ cmd='head';   argv=@();                shapes=$allShapes }
    @{ cmd='head';   argv=@('-n','5');        shapes=$allShapes }
    @{ cmd='head';   argv=@('-c','40');       shapes=$allShapes }
    @{ cmd='head';   argv=@('-n','1');        shapes=@('empty','nonutf8') }
    @{ cmd='tail';   argv=@('-n','5');        shapes=$allShapes }
    @{ cmd='tail';   argv=@('-n','+3');       shapes=@('ascii','crlf','notrail') }
    @{ cmd='tail';   argv=@('-c','40');       shapes=$allShapes }
    @{ cmd='tail';   argv=@('-c','+10');      shapes=@('ascii','nonutf8') }
    # --- sort ------------------------------------------------------------
    @{ cmd='sort';   argv=@();                shapes=@('ascii','empty','notrail') }
    @{ cmd='sort';   argv=@('-r');            shapes=@('ascii') }
    @{ cmd='sort';   argv=@('-n');            shapes=@('ascii') }
    @{ cmd='sort';   argv=@('-u');            shapes=@('ascii') }
    @{ cmd='sort';   argv=@('-f');            shapes=@('ascii') }
    @{ cmd='sort';   argv=@('-k','2');        shapes=@('ascii') }
    @{ cmd='sort';   argv=@('-t',' ','-k','2','-n'); shapes=@('ascii') }
    # --- uniq / tac / rev --------------------------------------------------
    @{ cmd='uniq';   argv=@();                shapes=@('ascii') }
    @{ cmd='uniq';   argv=@('-c');            shapes=@('ascii') }
    @{ cmd='uniq';   argv=@('-d');            shapes=@('ascii') }
    @{ cmd='uniq';   argv=@('-i');            shapes=@('ascii') }
    @{ cmd='tac';    argv=@();                shapes=@('ascii','crlf','notrail') }
    @{ cmd='rev';    argv=@();                shapes=@('ascii','nonutf8') }
    # --- tr --------------------------------------------------------------
    @{ cmd='tr';     argv=@('a-z','A-Z');     shapes=@('ascii','notrail') }
    @{ cmd='tr';     argv=@('-d','aeiou');    shapes=@('ascii') }
    @{ cmd='tr';     argv=@('-s','a');        shapes=@('ascii') }
    @{ cmd='tr';     argv=@('-c','a','x');    shapes=@('ascii') }
    @{ cmd='tr';     argv=@('[:lower:]','[:upper:]'); shapes=@('ascii') }
    @{ cmd='tr';     argv=@('\n',' ');        shapes=@('ascii') }
    # --- cut / paste / fold ------------------------------------------------
    @{ cmd='cut';    argv=@('-c','1-3');      shapes=@('ascii') }
    @{ cmd='cut';    argv=@('-b','1-5');      shapes=@('ascii','nonutf8') }
    @{ cmd='cut';    argv=@('-d',' ','-f','2'); shapes=@('ascii') }
    @{ cmd='paste';  argv=@('-d',',');        shapes=@('ascii') }
    @{ cmd='paste';  argv=@('-s');            shapes=@('ascii','notrail') }
    @{ cmd='fold';   argv=@('-w','17');       shapes=@('ascii') }
    @{ cmd='fold';   argv=@('-w','9','-s');   shapes=@('ascii') }
    @{ cmd='fold';   argv=@('-b','-w','11');  shapes=@('ascii') }
    # --- wc / nl / expand / unexpand ----------------------------------------
    @{ cmd='wc';     argv=@('-l');            shapes=@('ascii','crlf','notrail','empty') }
    @{ cmd='wc';     argv=@('-c');            shapes=@('ascii','nonutf8') }
    @{ cmd='wc';     argv=@('-w');            shapes=@('ascii') }
    @{ cmd='wc';     argv=@('-L');            shapes=@('ascii','notrail') }
    @{ cmd='nl';     argv=@();                shapes=@('ascii','notrail') }
    @{ cmd='nl';     argv=@('-b','a');        shapes=@('ascii') }
    @{ cmd='nl';     argv=@('-w','2');        shapes=@('ascii') }
    @{ cmd='expand'; argv=@('-t','4');        shapes=@('ascii') }
    @{ cmd='unexpand'; argv=@('-t','4');      shapes=@('ascii') }
    # --- grep (fixed-string subset) -----------------------------------------
    @{ cmd='grep';   argv=@('-F','abc');      shapes=@('ascii') }
    @{ cmd='grep';   argv=@('-F','-c','x');   shapes=@('ascii') }
    @{ cmd='grep';   argv=@('-F','-v','ab');  shapes=@('ascii') }
    @{ cmd='grep';   argv=@('-F','-n','e');   shapes=@('ascii') }
    # --- sed (literal-substring subset) --------------------------------------
    @{ cmd='sed';    argv=@('s/a/A/g');       shapes=@('ascii','notrail') }
    @{ cmd='sed';    argv=@('s/ab//');        shapes=@('ascii') }
    @{ cmd='sed';    argv=@('-n','3p');       shapes=@('ascii') }
    @{ cmd='sed';    argv=@('2d');            shapes=@('ascii') }
    @{ cmd='sed';    argv=@('$d');            shapes=@('ascii') }
    # --- checksums / base64 ---------------------------------------------------
    @{ cmd='md5sum'; argv=@('{in}');          shapes=@('ascii','nonutf8','empty'); fileArgs=$true }
    @{ cmd='sha256sum'; argv=@('{in}');       shapes=@('ascii','nonutf8','empty'); fileArgs=$true }
    @{ cmd='base64'; argv=@('-w','0');        shapes=@('nonutf8','empty') }
    @{ cmd='base64'; argv=@();                shapes=@('nonutf8') }
    # --- od -------------------------------------------------------------------
    @{ cmd='od';     argv=@('-t','x1');       shapes=@('nonutf8','empty') }
    @{ cmd='od';     argv=@('-t','d1');       shapes=@('nonutf8') }
    @{ cmd='od';     argv=@('-t','o1');       shapes=@('nonutf8') }
    @{ cmd='od';     argv=@('-t','u1');       shapes=@('nonutf8') }
    @{ cmd='od';     argv=@('-t','c');        shapes=@('ascii') }
    @{ cmd='od';     argv=@('-t','a');        shapes=@('ascii') }
    @{ cmd='od';     argv=@('-A','x','-t','x1'); shapes=@('nonutf8') }
    @{ cmd='od';     argv=@('-A','n','-t','o1'); shapes=@('nonutf8') }
    @{ cmd='od';     argv=@('-N','16','-t','x1'); shapes=@('nonutf8') }
    # --- misc determinists ------------------------------------------------------
    @{ cmd='seq';    argv=@('1','5');         shapes=@('empty') }
    @{ cmd='seq';    argv=@('2','3','11');    shapes=@('empty') }
    @{ cmd='seq';    argv=@('-s',',','1','4');shapes=@('empty') }
    @{ cmd='comm';   argv=@();                shapes=@('ascii'); twoInputs=$true }
    @{ cmd='true';   argv=@();                shapes=@('empty') }
    @{ cmd='false';  argv=@();                shapes=@('empty') }
    @{ cmd='echo';   argv=@('-n','abc');      shapes=@('empty') }
    @{ cmd='echo';   argv=@('-e','a\tb');     shapes=@('empty') }
    @{ cmd='printf'; argv=@('%s-%d','x','7'); shapes=@('empty') }
    @{ cmd='printf'; argv=@('%s','[:lower:]'); shapes=@('empty') }
    @{ cmd='printf'; argv=@('%s','quote" & path\'); shapes=@('empty') }
    @{ cmd='basename'; argv=@('/a/b/c.txt'); shapes=@('empty') }
    @{ cmd='basename'; argv=@('a/b/');        shapes=@('empty') }
    @{ cmd='dirname';  argv=@('/a/b/c.txt'); shapes=@('empty') }
    @{ cmd='numfmt'; argv=@('--to=si','2048'); shapes=@('empty') }
    @{ cmd='numfmt'; argv=@('--from=auto','1K'); shapes=@('empty') }
    # --- Round 15 additions --------------------------------------------------
    @{ cmd='rmdir';  argv=@('empty');         shapes=@('empty'); fixture=$true }
    @{ cmd='rmdir';  argv=@('-p','chain/a/b'); shapes=@('empty'); fixture=$true }
    @{ cmd='rmdir';  argv=@('nonempty');      shapes=@('empty'); fixture=$true }
    @{ cmd='rmdir';  argv=@('--ignore-fail-on-non-empty','nonempty'); shapes=@('empty'); fixture=$true }
    @{ cmd='link';   argv=@('source','linked'); shapes=@('ascii','nonutf8'); fixture=$true }
    @{ cmd='link';   argv=@('missing','linked'); shapes=@('empty'); fixture=$true }
    @{ cmd='unlink'; argv=@('source');        shapes=@('ascii','nonutf8'); fixture=$true }
    @{ cmd='unlink'; argv=@('nonempty');      shapes=@('empty'); fixture=$true }
    @{ cmd='install'; argv=@('source','installed'); shapes=@('ascii','nonutf8','empty'); fixture=$true }
    @{ cmd='install'; argv=@('source','installed'); shapes=@('ascii'); fixture=$true; hardlinkTarget=$true }
    @{ cmd='install'; argv=@('-m','444','source','installed'); shapes=@('ascii'); fixture=$true }
    @{ cmd='install'; argv=@('-D','source','parents/a/installed'); shapes=@('ascii'); fixture=$true }
    @{ cmd='install'; argv=@('-d','created/a'); shapes=@('empty'); fixture=$true }
    @{ cmd='install'; argv=@('source');       shapes=@('empty'); fixture=$true }
    @{ cmd='mktemp'; argv=@('tmp.XXXXXX');     shapes=@('empty'); fixture=$true }
    @{ cmd='mktemp'; argv=@('-d','tmp.XXXXXX'); shapes=@('empty'); fixture=$true }
    @{ cmd='mktemp'; argv=@('--suffix=.txt','tmp.XXXXXX'); shapes=@('empty'); fixture=$true }
    @{ cmd='mktemp'; argv=@('badXX');          shapes=@('empty'); fixture=$true }
    @{ cmd='vdir';   argv=@();                shapes=@('ascii') }
    @{ cmd='vdir';   argv=@('-h');            shapes=@('ascii') }
    @{ cmd='cksum';  argv=@('{in}');          shapes=@('ascii','nonutf8','empty'); fileArgs=$true }
    @{ cmd='cksum';  argv=@();               shapes=@('ascii','nonutf8','empty') }
    @{ cmd='cksum';  argv=@('-');            shapes=@('ascii','nonutf8','empty') }
    @{ cmd='sha1sum';  argv=@('{in}');        shapes=@('ascii','nonutf8','empty'); fileArgs=$true }
    @{ cmd='sha512sum'; argv=@('{in}');       shapes=@('ascii','nonutf8','empty'); fileArgs=$true }
    @{ cmd='sha1sum'; argv=@('--check','good.sum'); shapes=@('ascii','nonutf8'); fixture=$true }
    @{ cmd='sha1sum'; argv=@('--check'); shapes=@('ascii'); fixture=$true; stdinFile='good.sum' }
    @{ cmd='sha1sum'; argv=@('--check','-'); shapes=@('empty'); fixture=$true }
    @{ cmd='sha1sum'; argv=@('--check','bad.sum'); shapes=@('ascii'); fixture=$true }
    @{ cmd='sha512sum'; argv=@('--check','good.sum'); shapes=@('ascii','nonutf8'); fixture=$true }
    @{ cmd='sha512sum'; argv=@('--check'); shapes=@('ascii'); fixture=$true; stdinFile='good.sum' }
    @{ cmd='sha512sum'; argv=@('--check','-'); shapes=@('empty'); fixture=$true }
    @{ cmd='sha512sum'; argv=@('--check','bad.sum'); shapes=@('ascii'); fixture=$true }
    @{ cmd='users';  argv=@();                shapes=@('empty') }
    @{ cmd='logname'; argv=@();               shapes=@('empty') }
)

# ---------------------------------------------------------------------------
# Known, intentional differences: key 'cmd|argv-joined', value = reason.
# ---------------------------------------------------------------------------
$KnownDifferences = @{
    # MSYS2 coreutils default to binary mode: checksum lines gain a leading
    # backslash and use the `*` separator. RuxCmd matches Linux GNU text
    # mode (`hash  name`, two spaces, no escape).
    'md5sum|*'    = 'MSYS binary-mode checksum formatting; RuxCmd matches Linux text mode'
    'sha256sum|*' = 'MSYS binary-mode checksum formatting; RuxCmd matches Linux text mode'
    'sha1sum|{in}'   = 'MSYS binary-mode checksum formatting; RuxCmd matches Linux text mode'
    'sha512sum|{in}' = 'MSYS binary-mode checksum formatting; RuxCmd matches Linux text mode'
    # vdir inherits the ls long-format layout (column set, nlink, date style),
    # which differs from GNU ls -l at the P2 formatting level.
    'vdir|*'      = 'ls long-format layout difference (columns/date), not vdir-specific'
    # GNU users reads utmp, which does not exist in an MSYS session, so the
    # reference tool prints nothing here. RuxCmd enumerates TS sessions.
    'users|*'     = 'MSYS utmp unavailable; reference prints empty'
}

# ---------------------------------------------------------------------------
# Runner.
# ---------------------------------------------------------------------------
function Invoke-Tool([string]$exe, [string[]]$argv, [string]$stdinPath, [string]$workingDirectory) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = [IO.Path]::GetFullPath($exe)
    $info.UseShellExecute = $false
    $info.CreateNoWindow = $true
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    # Keep byte-oriented fixtures independent of the host locale.
    $info.Environment['LC_ALL'] = 'C'
    if ($workingDirectory) { $info.WorkingDirectory = $workingDirectory }
    # MSYS expands unquoted wildcard arguments even without a shell. Quote
    # every argument, escaping backslashes before quotes and at the end by
    # Windows CRT rules; ArgumentList only quotes arguments when necessary.
    $info.Arguments = (@($argv | ForEach-Object {
        $argument = [regex]::Replace($_, '(\\*)"', '$1$1\"')
        $argument = [regex]::Replace($argument, '(\\+)$', '$1$1')
        '"' + $argument + '"'
    }) -join ' ')
    $process = [Diagnostics.Process]::new()
    $process.StartInfo = $info
    $output = [IO.MemoryStream]::new()
    $inputFile = [IO.File]::OpenRead($stdinPath)
    $started = $false
    try {
        $started = $process.Start()
        $outputTask = $process.StandardOutput.BaseStream.CopyToAsync($output)
        $errorTask = $process.StandardError.ReadToEndAsync()
        $inputTask = $inputFile.CopyToAsync($process.StandardInput.BaseStream)
        try {
            if (-not $inputTask.Wait(10000)) { throw "stdin timed out: $exe" }
        } catch {
            # A command may exit without consuming stdin (e.g. true/false).
            if (-not $process.HasExited) { throw }
        } finally { $process.StandardInput.Close() }
        if (-not $process.WaitForExit(10000)) { throw "command timed out: $exe" }
        if (-not $outputTask.Wait(10000) -or -not $errorTask.Wait(10000)) { throw "output timed out: $exe" }
        return @{ Code = $process.ExitCode; Bytes = $output.ToArray(); Error = $errorTask.Result }
    } finally {
        if ($started -and -not $process.HasExited) { $process.Kill($true); $process.WaitForExit() }
        $inputFile.Dispose()
        $output.Dispose()
        $process.Dispose()
    }
}

function New-CaseFixture([string]$directory, [string]$inputPath, [string]$command, [bool]$hardlinkTarget) {
    foreach ($path in @('', 'empty', 'chain/a/b', 'nonempty')) {
        [IO.Directory]::CreateDirectory((Join-Path $directory $path)) | Out-Null
    }
    [IO.File]::Copy($inputPath, (Join-Path $directory 'source'))
    [IO.File]::WriteAllText((Join-Path $directory 'nonempty/keep'), 'keep')
    if ($hardlinkTarget) {
        [IO.File]::WriteAllText((Join-Path $directory 'other'), 'keep other')
        New-Item -ItemType HardLink -Path (Join-Path $directory 'installed') -Target (Join-Path $directory 'other') | Out-Null
    }
    if ($command -in @('sha1sum','sha512sum')) {
        $algorithm = if ($command -eq 'sha1sum') { [Security.Cryptography.SHA1]::Create() } else { [Security.Cryptography.SHA512]::Create() }
        try { $digest = [Convert]::ToHexString($algorithm.ComputeHash([IO.File]::ReadAllBytes($inputPath))).ToLowerInvariant() }
        finally { $algorithm.Dispose() }
        [IO.File]::WriteAllText((Join-Path $directory 'good.sum'), "$digest  source`n")
        $bad = ('0' * $digest.Length)
        [IO.File]::WriteAllText((Join-Path $directory 'bad.sum'), "$bad  source`n")
    }
}

function Get-FixtureState([string]$directory) {
    $records = foreach ($entry in Get-ChildItem -LiteralPath $directory -Recurse -Force) {
        $name = [IO.Path]::GetRelativePath($directory, $entry.FullName).Replace('\','/')
        $name = $name -creplace '^tmp\.[a-zA-Z0-9]{6}(\.txt)?$', 'tmp.RANDOM$1'
        if ($entry.PSIsContainer) { "dir:$name" }
        else { "file:${name}:$((Get-FileHash -LiteralPath $entry.FullName -Algorithm SHA256).Hash)" }
    }
    return (@($records | Sort-Object) -join "`n")
}

function Show-Bytes([byte[]]$bytes, [int]$max = 120) {
    if ($bytes.Length -le $max) { $slice = $bytes } else { $slice = $bytes[0..($max-1)] }
    $text = ($slice | ForEach-Object { if ($_ -ge 32 -and $_ -le 126) { [char]$_ } else { '.' } }) -join ''
    return "{0}B [{1}]" -f $bytes.Length, $text
}

$stats = [ordered]@{ Pass = 0; Fail = 0; Skip = 0 }
$failures = [System.Collections.Generic.List[string]]::new()
$skipReported = @{}

$selected = if ($Cases) { $matrix | Where-Object { $_.cmd -cin $Cases } } else { $matrix }
Write-Host ("inputs=" + $inputs.Count + " cases=" + $selected.Count)
foreach ($case in $selected) {
    $key = $case.cmd + '|' + ($case.argv -join ' ')
    if (-not (Test-Path (Join-Path $BinDir ($case.cmd + '.exe')))) { throw "Missing RuxCmd executable: $($case.cmd)" }
    # Skip the whole case when the GNU reference tool is not installed
    # (Git for Windows ships only a subset of coreutils, e.g. no rev).
    if (-not (Test-Path (Join-Path $GnuBin ($case.cmd + '.exe')))) {
        $stats.Skip++
        if (-not $skipReported[$key]) {
            Write-Host ("SKIP  {0}: GNU tool not present in {1}" -f $key, $GnuBin)
            $skipReported[$key] = $true
        }
        continue
    }
    Write-Host ("case: " + $case.cmd)
    $skip = $null
    foreach ($k in $KnownDifferences.Keys) {
        if ($key -like $k) { $skip = $KnownDifferences[$k]; break }
    }
    foreach ($shape in $case.shapes) {
        for ($v = 0; $v -lt $Variants; $v++) {
            $label = "{0} [{1}#{2}]" -f $key, $shape, $v
            if ($skip) {
                $stats.Skip++
                if (-not $skipReported[$k]) {
                    Write-Host ("SKIP  {0}: {1}" -f $key, $skip)
                    $skipReported[$k] = $true
                }
                continue
            }
            $inPath = $inputs["$shape$v"]
            $argvRux = @($case.argv | ForEach-Object { $_ -replace '\{inbare\}', ($inPath -replace '\.in$','') -replace '\{in\}', $inPath })
            $argvGnu = $argvRux

            # Commands that read operands instead of stdin get the input as
            # a file argument; stdin stays an empty pipe for both sides.
            $stdinRux = $inPath
            $stdinGnu = $inPath
            if ($case.fileArgs) {
                $stdinRux = $inputs['empty0']; $stdinGnu = $inputs['empty0']
                $argvRux = @($case.argv | ForEach-Object { $_ -replace '\{in\}', $inPath })
                $argvGnu = $argvRux
            }
            if ($case.twoInputs) {
                # comm reads two sorted files: use the same input twice.
                $stdinRux = $inputs['empty0']; $stdinGnu = $inputs['empty0']
                $argvRux = @($inPath, $inPath)
                $argvGnu = $argvRux
            }

            $ruxDirectory = $null
            $gnuDirectory = $null
            if ($case.fixture) {
                $fixtureRoot = Join-Path $work ([guid]::NewGuid().ToString('N'))
                $ruxDirectory = Join-Path $fixtureRoot 'rux'
                $gnuDirectory = Join-Path $fixtureRoot 'gnu'
                New-CaseFixture $ruxDirectory $inPath $case.cmd ([bool]$case.hardlinkTarget)
                New-CaseFixture $gnuDirectory $inPath $case.cmd ([bool]$case.hardlinkTarget)
                $stdinRux = $inputs['empty0']; $stdinGnu = $inputs['empty0']
                if ($case.stdinFile) {
                    $stdinRux = Join-Path $ruxDirectory $case.stdinFile
                    $stdinGnu = Join-Path $gnuDirectory $case.stdinFile
                }
            }
            $rux = Invoke-Tool (Join-Path $BinDir ($case.cmd + '.exe')) $argvRux $stdinRux $ruxDirectory
            $gnu = Invoke-Tool (Join-Path $GnuBin ($case.cmd + '.exe')) $argvGnu $stdinGnu $gnuDirectory
            if ($case.cmd -eq 'mktemp' -and $rux.Code -eq 0 -and $gnu.Code -eq 0) {
                foreach ($result in @($rux,$gnu)) {
                    $name = [Text.Encoding]::UTF8.GetString($result.Bytes)
                    $result.Bytes = [Text.Encoding]::UTF8.GetBytes(($name -creplace 'tmp\.[a-zA-Z0-9]{6}', 'tmp.RANDOM'))
                }
            }
            $stateEqual = -not $case.fixture -or (Get-FixtureState $ruxDirectory) -ceq (Get-FixtureState $gnuDirectory)

            $bytesEqual = ($rux.Bytes.Length -eq $gnu.Bytes.Length) -and
                (-not (Compare-Object $rux.Bytes $gnu.Bytes -SyncWindow 0 -ErrorAction SilentlyContinue))
            # Compare-Object is slow for large arrays; fall back to a hash.
            if ($bytesEqual -and $rux.Bytes.Length -gt 0) {
                $h1 = [System.Security.Cryptography.SHA256]::Create().ComputeHash($rux.Bytes)
                $h2 = [System.Security.Cryptography.SHA256]::Create().ComputeHash($gnu.Bytes)
                $bytesEqual = -not (Compare-Object $h1 $h2 -SyncWindow 0)
            }
            $codeEqual = $rux.Code -eq $gnu.Code

            if ($bytesEqual -and $codeEqual -and $stateEqual) {
                $stats.Pass++
                if ($ShowAll) { Write-Host "PASS  $label" }
            } else {
                $stats.Fail++
                $why = @()
                if (-not $bytesEqual) { $why += 'stdout' }
                if (-not $codeEqual) { $why += "exit($($rux.Code) vs $($gnu.Code))" }
                if (-not $stateEqual) { $why += 'filesystem state' }
                $failures.Add(("FAIL  {0}  diff: {1}" -f $label, ($why -join ' ')))
                $failures.Add(("      rux : " + (Show-Bytes $rux.Bytes)))
                $failures.Add(("      gnu : " + (Show-Bytes $gnu.Bytes)))
                if ($rux.Error) { $failures.Add("      rux stderr: $($rux.Error.Trim())") }
                if ($gnu.Error) { $failures.Add("      gnu stderr: $($gnu.Error.Trim())") }
            }
        }
    }
}

# ---------------------------------------------------------------------------
# Report.
# ---------------------------------------------------------------------------
Write-Host ''
Write-Host ("Differential test: {0} pass, {1} fail, {2} skip (seed {3}, variants {4})" -f
    $stats.Pass, $stats.Fail, $stats.Skip, $Seed, $Variants)
if ($failures.Count -gt 0) {
    Write-Host ''
    $failures | ForEach-Object { Write-Host $_ }
}

# Cleanup, then exit with the verdict.
$workRoot = [IO.Path]::GetFullPath($work)
$tempRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\') + '\'
if (-not $workRoot.StartsWith($tempRoot, [StringComparison]::OrdinalIgnoreCase)) { throw "Unexpected cleanup path: $workRoot" }
Remove-Item -LiteralPath $workRoot -Recurse -Force -ErrorAction SilentlyContinue
if ($stats.Fail -gt 0) { exit 1 } else { exit 0 }

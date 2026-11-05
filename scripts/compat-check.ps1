#Requires -Version 7.0
<#
.SYNOPSIS
Validate the compatibility tables against Cargo metadata, the built binaries' --help output, and the documented command counts.
.DESCRIPTION
Checks every Markdown compatibility table (column count, duplicate rows, blank-line splits, bilingual row parity) and
compares each command's declared long options with the options reported by `<command> --help`. Also verifies that the
command/package counts quoted in README, development and compatibility pages match `cargo metadata`.

Commands whose help text is not a reliable option inventory are skipped explicitly; the skip list and reasons are
printed so a partial result is never mistaken for a full pass.

The binary comparison needs a directory containing one `<command>.exe` per workspace binary target. By default the
release directory reported by Cargo metadata is used.
.EXAMPLE
./scripts/compat-check.ps1
.EXAMPLE
./scripts/compat-check.ps1 -BinDir target/x86_64-pc-windows-msvc/debug
#>
[CmdletBinding()]
param(
    [string]$BinDir,
    [ValidateRange(1, 120)][int]$TimeoutSeconds = 20
)
. (Join-Path $PSScriptRoot 'common.ps1')
if (-not $IsWindows) { throw 'compat-check.ps1 requires Windows.' }

$workspace = Get-WorkspaceInfo
$binaryNames = @($workspace.Binaries.Name | Sort-Object)
$packageCount = $workspace.Packages.Count
$compatibilityFiles = @(
    (Join-Path $script:RepositoryRoot 'docs/compatibility.md'),
    (Join-Path $script:RepositoryRoot 'docs/compatibility-zh.md')
)
# Help text is the option inventory for clap-based commands. These commands parse options themselves, so their
# clap help intentionally omits options that the compatibility table may legitimately list. Keep this list explicit
# rather than relaxing the rule for every command.
$optionCheckSkips = [ordered]@{
    'find' = 'hand-written GNU-style option parser; clap help lists only the initial subset and omits -print0/-delete/-size/-mtime/-exec'
    'dd'   = 'operands and conv=/status= keywords are parsed by hand; help is not a complete option inventory'
}
$failures = [Collections.Generic.List[string]]::new()
function Add-Failure([string]$Message) {
    $script:failures.Add($Message)
    Write-Host "FAIL  $Message" -ForegroundColor Red
}
function Split-TableRow([string]$Line) {
    $trimmed = $Line.Trim()
    if (-not $trimmed.StartsWith('|')) { throw "Not a table row: $Line" }
    $inner = $trimmed.Substring(1)
    if ($inner.EndsWith('|')) { $inner = $inner.Substring(0, $inner.Length - 1) }
    return @([regex]::Split($inner, '(?<!\\)\|') | ForEach-Object { $_.Trim() })
}
function Get-LongOptions([string]$Text) {
    # Only option-definition lines are considered: the short option, if any, starts the line and the description
    # column is indented much further, so prose mentions of other options are ignored.
    $found = [Collections.Generic.List[string]]::new()
    foreach ($line in ($Text -split '\r?\n')) {
        $match = [regex]::Match($line, '^\s{1,12}(?:-\S+,\s+)?(--[A-Za-z][A-Za-z0-9-]*)')
        if ($match.Success) { $found.Add($match.Groups[1].Value) }
    }
    return @($found | Sort-Object -Unique)
}
function Read-CompatibilityTable([string]$Path) {
    $lines = Get-Content -LiteralPath $Path
    $headerIndex = -1
    for ($i = 0; $i -lt $lines.Count; $i++) {
        if ($lines[$i] -match '^\|\s*(命令|Command)\s*\|') { $headerIndex = $i; break }
    }
    if ($headerIndex -lt 0) { throw "No compatibility table header in $Path" }
    $header = @(Split-TableRow $lines[$headerIndex])
    if ($headerIndex + 1 -ge $lines.Count -or $lines[$headerIndex + 1] -notmatch '^\|\s*:?-{3,}') {
        throw "Missing table separator after $Path line $($headerIndex + 2)"
    }
    $rows = [Collections.Generic.List[object]]::new()
    $i = $headerIndex + 2
    while ($i -lt $lines.Count -and $lines[$i].StartsWith('|')) {
        $cells = @(Split-TableRow $lines[$i])
        if ($cells.Count -ne $header.Count) {
            Add-Failure "$([IO.Path]::GetFileName($Path)) line $($i + 1): $($cells.Count) columns, header has $($header.Count)"
        } elseif ([string]::IsNullOrWhiteSpace($cells[0])) {
            Add-Failure "$([IO.Path]::GetFileName($Path)) line $($i + 1): empty command column"
        } else {
            $rows.Add([pscustomobject]@{ Line = $i + 1; Command = $cells[0]; Options = $cells[1]; Cells = $cells })
        }
        $i++
    }
    for ($j = $i; $j -lt $lines.Count; $j++) {
        if ($lines[$j].StartsWith('|')) {
            Add-Failure "$([IO.Path]::GetFileName($Path)) line $($j + 1): table resumes after a blank line or non-table content (split table)"
        }
    }
    [pscustomobject]@{ Path = $Path; Rows = $rows; Header = $header }
}

$tables = @{}
foreach ($path in $compatibilityFiles) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { Add-Failure "Missing compatibility table: $path"; continue }
    try { $tables[$path] = Read-CompatibilityTable $path } catch { Add-Failure $_.Exception.Message }
}

# --- Table structure and command coverage -------------------------------------------------
foreach ($path in $tables.Keys) {
    $table = $tables[$path]
    $name = [IO.Path]::GetFileName($path)
    $seen = @{}
    foreach ($row in $table.Rows) {
        if ($seen.ContainsKey($row.Command)) {
            Add-Failure "$name line $($row.Line): duplicate row for '$($row.Command)' (first at line $($seen[$row.Command]))"
        } else { $seen[$row.Command] = $row.Line }
    }
    $documented = @($table.Rows.Command | Sort-Object)
    foreach ($diff in @(Compare-Object $binaryNames $documented)) {
        if ($diff.SideIndicator -eq '<=') { Add-Failure "$name is missing a row for workspace binary '$($diff.InputObject)'" }
        else { Add-Failure "$name documents '$($diff.InputObject)', which is not a workspace binary" }
    }
}

$primary = $tables[$compatibilityFiles[0]]
$secondary = $tables[$compatibilityFiles[1]]
if ($primary -and $secondary) {
    if ($primary.Rows.Count -ne $secondary.Rows.Count) {
        Add-Failure "Compatibility tables have different row counts: $(@($primary.Rows).Count) vs $(@($secondary.Rows).Count)"
    }
    $secondaryByName = @{}
    foreach ($row in $secondary.Rows) { if (-not $secondaryByName.ContainsKey($row.Command)) { $secondaryByName[$row.Command] = $row } }
    foreach ($row in $primary.Rows) {
        if (-not $secondaryByName.ContainsKey($row.Command)) { Add-Failure "Chinese table is missing '$($row.Command)'" ; continue }
        if ($secondaryByName[$row.Command].Options -cne $row.Options) {
            Add-Failure "'$($row.Command)' option lists differ between locales: '$($row.Options)' vs '$($secondaryByName[$row.Command].Options)'"
        }
    }
}
if ($primary) {
    Write-Host "Table rows: $($primary.Rows.Count) (workspace binaries: $($binaryNames.Count))"
}

# --- Help option comparison --------------------------------------------------------------
$directory = if ($BinDir) { Get-FileSystemPath $BinDir } else { Join-Path $workspace.TargetDirectory "$($workspace.Target)/release" }
if (-not (Test-Path -LiteralPath $directory -PathType Container)) {
    Add-Failure "Binary directory not found: $directory. Build with scripts/build.ps1 before running the option check."
} elseif ($primary) {
    $checked = 0
    $skipped = [Collections.Generic.List[string]]::new()
    foreach ($row in $primary.Rows) {
        if ($row.Options -match '(?i)\bsee --help\b|见 --help') {
            $skipped.Add("$($row.Command): table defers to --help")
            continue
        }
        if ($optionCheckSkips.Contains($row.Command)) {
            $skipped.Add("$($row.Command): $($optionCheckSkips[$row.Command])")
            continue
        }
        $declared = @([regex]::Matches($row.Options, '--[A-Za-z][A-Za-z0-9-]*') | ForEach-Object Value | Sort-Object -Unique)
        if ($declared.Count -eq 0) { $skipped.Add("$($row.Command): table lists no long options"); continue }
        $exe = Join-Path $directory "$($row.Command).exe"
        if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { Add-Failure "Missing executable for '$($row.Command)': $exe"; continue }
        $result = Invoke-ToolProcess $exe -Arguments @('--help') -TimeoutSeconds $TimeoutSeconds -WorkingDirectory $directory
        if ($result.ExitCode -ne 0) { Add-Failure "'$($row.Command) --help' exited $($result.ExitCode)" ; continue }
        $actual = @(Get-LongOptions $result.Output | Where-Object { $_ -cne '--help' })
        foreach ($option in @($declared | Where-Object { $_ -notin $actual })) {
            Add-Failure "'$($row.Command)': table lists $option, but --help does not"
        }
        # --version is framework-provided, so the table does not have to list it; every other
        # command-specific long option in help must appear in the table.
        foreach ($option in @($actual | Where-Object { $_ -cne '--version' -and $_ -notin $declared })) {
            Add-Failure "'$($row.Command)': --help exposes $option, but the table does not list it"
        }
        $checked++
    }
    Write-Host "Option checks run: $checked; skipped: $($skipped.Count)"
    foreach ($entry in $skipped) { Write-Host "SKIP  $entry" -ForegroundColor Yellow }
}

# --- Documented counts -------------------------------------------------------------------
$countRules = @(
    [pscustomobject]@{ File = 'README.md'; Pattern = 'builds \*\*(\d+) standalone executables\*\*'; CommandGroup = 1; PackageGroup = 0 }
    [pscustomobject]@{ File = 'README.md'; Pattern = '\*\*(\d+) packages produce (\d+) commands\*\*'; CommandGroup = 2; PackageGroup = 1 }
    [pscustomobject]@{ File = 'README-zh.md'; Pattern = '\*\*(\d+) 个独立可执行文件\*\*'; CommandGroup = 1; PackageGroup = 0 }
    [pscustomobject]@{ File = 'README-zh.md'; Pattern = '\*\*(\d+) 个包生成 (\d+) 个命令\*\*'; CommandGroup = 2; PackageGroup = 1 }
    [pscustomobject]@{ File = 'docs/development.md'; Pattern = 'Currently (\d+) packages produce (\d+) commands'; CommandGroup = 2; PackageGroup = 1 }
    [pscustomobject]@{ File = 'docs/development-zh.md'; Pattern = '目前 (\d+) 个 package 产出 (\d+) 个命令'; CommandGroup = 2; PackageGroup = 1 }
    [pscustomobject]@{ File = 'docs/compatibility.md'; Pattern = '当前 workspace 范围为 (\d+) 个命令'; CommandGroup = 1; PackageGroup = 0 }
    [pscustomobject]@{ File = 'docs/compatibility-zh.md'; Pattern = '当前 workspace 范围为 (\d+) 个命令'; CommandGroup = 1; PackageGroup = 0 }
    [pscustomobject]@{ File = 'scripts/README.md'; Pattern = '\((\d+) in the current workspace'; CommandGroup = 1; PackageGroup = 0 }
    [pscustomobject]@{ File = 'scripts/README-zh.md'; Pattern = '（当前 workspace 为 (\d+) 个'; CommandGroup = 1; PackageGroup = 0 }
)
foreach ($rule in $countRules) {
    $path = Join-Path $script:RepositoryRoot $rule.File
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { Add-Failure "Missing count source: $($rule.File)"; continue }
    $text = Get-Content -LiteralPath $path -Raw
    $hits = [regex]::Matches($text, $rule.Pattern)
    if ($hits.Count -eq 0) { Add-Failure "$($rule.File): count pattern not found ('$($rule.Pattern)')"; continue }
    foreach ($match in $hits) {
        $commands = [int]$match.Groups[$rule.CommandGroup].Value
        if ($commands -ne $binaryNames.Count) {
            Add-Failure "$($rule.File): documents $commands commands, workspace has $($binaryNames.Count)"
        }
        if ($rule.PackageGroup -gt 0) {
            $packages = [int]$match.Groups[$rule.PackageGroup].Value
            if ($packages -ne $packageCount) {
                Add-Failure "$($rule.File): documents $packages packages, workspace has $packageCount"
            }
        }
    }
}

if ($failures.Count) { throw "compat-check failed with $($failures.Count) problem(s)." }
Write-Host "Compatibility tables, option lists and documented counts match the workspace." -ForegroundColor Green

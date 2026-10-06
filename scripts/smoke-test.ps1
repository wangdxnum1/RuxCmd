#Requires -Version 7.0
[CmdletBinding()]
param([Parameter(Mandatory)][string]$BinDir, [ValidateRange(1, 300)][int]$TimeoutSeconds = 10)
. (Join-Path $PSScriptRoot 'common.ps1')
$workspace = Get-WorkspaceInfo
$cases = @(Get-SmokeCases (Join-Path $PSScriptRoot 'smoke-cases.json') $workspace)
$directory = Get-FileSystemPath $BinDir
$failures = [Collections.Generic.List[string]]::new()
foreach ($case in $cases) {
    try {
        $path = Join-Path $directory "$($case.name).exe"
        if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Missing executable: $path" }
        $arguments = @($case.arguments | Where-Object { $null -ne $_ })
        $result = Invoke-ToolProcess $path -Arguments $arguments -TimeoutSeconds $TimeoutSeconds -WorkingDirectory $directory
        if ($result.ExitCode -ne $case.exitCode) { throw "Exit $($result.ExitCode), expected $($case.exitCode). $($result.Error)" }
        if ($arguments.Count -gt 0 -and [string]::IsNullOrWhiteSpace($result.Output)) { throw 'Help output is empty.' }
    } catch { $failures.Add("$($case.name): $($_.Exception.Message)") }
}
if ($failures.Count) { throw ($failures -join "`n") }
Write-Host "Smoke passed: $($cases.Count) commands."

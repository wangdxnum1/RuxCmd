# Release validation is separate so failure paths can be tested without publishing.
function Get-GitHubRepository([string]$RemoteUrl) {
    if ($RemoteUrl -match '^(?:https://github\.com/|git@github\.com:|ssh://git@github\.com/)([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+?)(?:\.git)?/?$') { return $Matches[1] }
    throw 'origin must point to a github.com repository using HTTPS or SSH.'
}

function Assert-ReleaseSource([string]$Status, [string]$Commit, [string]$TagCommit) {
    if (-not [string]::IsNullOrWhiteSpace($Status)) { throw 'Formal releases require a clean checkout, including untracked files. Commit or set aside your changes first.' }
    if ($TagCommit -and $TagCommit -cne $Commit) { throw 'Release tag points to another commit; tags are never moved or overwritten.' }
}

function Assert-RemoteTag([string]$References, [string]$Tag, [string]$Commit) {
    $tagRows = @($References -split '\r?\n' | Where-Object { $_.Trim() })
    if (-not $tagRows.Count) { return }
    $peeled = @($tagRows | Where-Object { ($_ -split '\s+')[1] -ceq "refs/tags/$Tag^{}" })
    $direct = @($tagRows | Where-Object { ($_ -split '\s+')[1] -ceq "refs/tags/$Tag" })
    $chosen = @(if ($peeled.Count) { $peeled } else { $direct })
    if ($chosen.Count -ne 1 -or ($chosen[0] -split '\s+')[0] -cne $Commit) { throw 'Remote release tag points to another commit; refusing to overwrite.' }
}

function Assert-ReleaseAssets([string]$Archive, [string]$Checksum, $Workspace, [string]$Commit, [switch]$RequireClean) {
    $expected = "$(Get-FileDigest $Archive)  $([IO.Path]::GetFileName($Archive))"
    if ((Get-Content -LiteralPath $Checksum -Raw).TrimEnd() -cne $expected) { throw 'Release ZIP checksum mismatch.' }
    $zip = [IO.Compression.ZipFile]::OpenRead($Archive)
    try {
        $entry = $zip.GetEntry('manifest.json')
        if (-not $entry) { throw 'Release ZIP is missing manifest.json.' }
        $reader = [IO.StreamReader]::new($entry.Open())
        try { $manifest = $reader.ReadToEnd() | ConvertFrom-Json } finally { $reader.Dispose() }
        if ($manifest.suiteVersion -cne $Workspace.Version -or $manifest.target -cne $Workspace.Target -or $manifest.sourceCommit -cne $Commit) { throw 'Release manifest does not match the requested version, target and source commit.' }
        if ($RequireClean -and $manifest.dirty) { throw 'Cannot publish a dirty release manifest.' }
    } finally { $zip.Dispose() }
}

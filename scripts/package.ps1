#Requires -Version 7.0
[CmdletBinding()]
param([string]$OutputDir, [switch]$RequireClean, [string]$ExpectedTag)
. (Join-Path $PSScriptRoot 'common.ps1')
$workspace = Get-WorkspaceInfo
Assert-ReleaseVersion $workspace.Version $ExpectedTag
$status = Invoke-ToolProcess git @('status', '--porcelain', '--untracked-files=normal')
Assert-ToolSuccess $status 'git status'
$dirty = -not [string]::IsNullOrWhiteSpace($status.Output)
if ($RequireClean -and $dirty) { throw 'Formal releases require a clean checkout.' }
$commit = Invoke-ToolProcess git @('rev-parse', 'HEAD')
Assert-ToolSuccess $commit 'git rev-parse'
if ($ExpectedTag) {
    $tagCommit = Invoke-ToolProcess git @('rev-parse', '--verify', "refs/tags/$ExpectedTag^{commit}")
    Assert-ToolSuccess $tagCommit 'git tag verification'
    if ($tagCommit.Output.Trim() -cne $commit.Output.Trim()) { throw 'Requested tag does not refer to the current checkout.' }
}
if (-not $OutputDir) { $OutputDir = Join-Path $script:RepositoryRoot 'dist' }
$OutputDir = Get-FileSystemPath $OutputDir
[IO.Directory]::CreateDirectory($OutputDir) | Out-Null
$name = "ruxcmd-$($workspace.Version)-$($workspace.Target)"
$archive = Join-Path $OutputDir "$name.zip"
$checksumFile = "$archive.sha256"
if ((Test-Path -LiteralPath $archive) -or (Test-Path -LiteralPath $checksumFile)) { throw 'Release output already exists; use a different OutputDir or version.' }
$owned = Join-Path $OutputDir ".staging-$([guid]::NewGuid().ToString('N'))"
$stage = Join-Path $owned 'content'
$verify = Join-Path $owned 'verify'
$createdArchive = $false
$createdChecksum = $false
try {
    [IO.Directory]::CreateDirectory((Join-Path $stage 'bin')) | Out-Null
    $build = Invoke-ToolProcess cargo @('build', '--workspace', '--bins', '--release', '--target', $workspace.Target, '--locked', '--message-format=json-render-diagnostics') -TimeoutSeconds 1800
    if ($build.Error) { Write-Host $build.Error }
    Assert-ToolSuccess $build 'cargo release build'
    $artifacts = Get-BinaryArtifacts $build.Output $workspace
    $dumpbin = Find-Dumpbin
    $binaryRecords = @(
        foreach ($binary in $workspace.Binaries) {
            $source = $artifacts[$binary.Name]
            $dlls = @(Get-NativeDependencies $source $dumpbin)
            $relative = "bin/$($binary.Name).exe"
            $destination = Join-Path $stage $relative
            Copy-Item -LiteralPath $source -Destination $destination
            [ordered]@{ name = $binary.Name; packageVersion = $binary.PackageVersion; file = $relative; sha256 = Get-FileDigest $destination; dependencies = $dlls }
        }
    )
    & (Join-Path $PSScriptRoot 'smoke-test.ps1') -BinDir (Join-Path $stage 'bin')
    foreach ($doc in @('README.md', 'CHANGELOG.md')) { Copy-Item -LiteralPath (Join-Path $script:RepositoryRoot $doc) -Destination $stage }
    Copy-Item -LiteralPath (Join-Path $script:RepositoryRoot 'docs/compatibility.md') -Destination $stage
    $files = @(Get-ChildItem -LiteralPath $stage -File -Recurse | Sort-Object FullName | ForEach-Object {
        [ordered]@{ path = [IO.Path]::GetRelativePath($stage, $_.FullName).Replace('\', '/'); sha256 = Get-FileDigest $_.FullName }
    })
    $rustc = Invoke-ToolProcess rustc @('--version')
    Assert-ToolSuccess $rustc 'rustc'
    $manifest = [ordered]@{ schemaVersion = 1; suiteVersion = $workspace.Version; target = $workspace.Target; rustcVersion = $rustc.Output.Trim(); sourceCommit = $commit.Output.Trim(); dirty = $dirty; binaries = $binaryRecords; files = $files }
    $manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stage 'manifest.json') -Encoding utf8NoBOM
    ($files | ForEach-Object { "$($_.sha256)  $($_.path)" }) -join "`n" | Set-Content -LiteralPath (Join-Path $stage 'SHA256SUMS') -Encoding utf8NoBOM
    Assert-ArchiveDirectory $stage $manifest
    $privateArchive = Join-Path $owned 'release.zip'
    [IO.Compression.ZipFile]::CreateFromDirectory($stage, $privateArchive)
    [IO.Compression.ZipFile]::ExtractToDirectory($privateArchive, $verify)
    $readManifest = Get-Content -LiteralPath (Join-Path $verify 'manifest.json') -Raw | ConvertFrom-Json
    Assert-ArchiveDirectory $verify $readManifest
    & (Join-Path $PSScriptRoot 'smoke-test.ps1') -BinDir (Join-Path $verify 'bin')
    & (Join-Path $PSScriptRoot 'verify-behavior.ps1') -BinDir (Join-Path $verify 'bin')
    # Reserve the final paths exclusively after verification. Never overwrite a concurrent run.
    $destinationStream = [IO.File]::Open($archive, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    $createdArchive = $true
    try {
        $sourceStream = [IO.File]::OpenRead($privateArchive)
        try { $sourceStream.CopyTo($destinationStream) } finally { $sourceStream.Dispose() }
    } finally { $destinationStream.Dispose() }
    $checksumStream = [IO.File]::Open($checksumFile, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    $createdChecksum = $true
    try {
        $writer = [IO.StreamWriter]::new($checksumStream, [Text.UTF8Encoding]::new($false))
        try { $writer.WriteLine("$(Get-FileDigest $archive)  $([IO.Path]::GetFileName($archive))") } finally { $writer.Dispose() }
    } finally { $checksumStream.Dispose() }
    Write-Host "Verified release: $archive ($($binaryRecords.Count) executables, dirty=$dirty)"
} catch {
    if ($createdArchive -and (Test-Path -LiteralPath $archive)) { Remove-Item -LiteralPath $archive -Force }
    if ($createdChecksum -and (Test-Path -LiteralPath $checksumFile)) { Remove-Item -LiteralPath $checksumFile -Force }
    throw
} finally { Remove-OwnedDirectory $owned $OutputDir }

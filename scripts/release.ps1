#Requires -Version 7.0
<#
.SYNOPSIS
Build a verified local ZIP, or explicitly publish a version to GitHub.
.DESCRIPTION
Local packages may record dirty work. -Publish requires a clean checkout,
runs full verification, creates a missing version tag at HEAD, packages,
pushes the tag and uploads a new GitHub Release. Existing releases are refused.
An explicit -Tag must match the suite version. For local tagged builds the
tag must already exist. -WhatIf previews without building, tagging or uploading.
.EXAMPLE
./scripts/release.ps1 -OutputDir dist/local-check
.EXAMPLE
./scripts/release.ps1 -Publish -Draft -NotesFile release-notes.md
#>
[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$OutputDir,
    [string]$Tag,
    [switch]$RequireClean,
    [switch]$Publish,
    [switch]$Draft,
    [string]$NotesFile
)
. (Join-Path $PSScriptRoot 'common.ps1')
. (Join-Path $PSScriptRoot 'release-helpers.ps1')
if (-not $IsWindows) { throw 'RuxCmd releases require Windows and MSVC.' }
if (($Draft -or $NotesFile) -and -not $Publish) { throw '-Draft and -NotesFile require -Publish.' }
$workspace = Get-WorkspaceInfo
if ($Publish -and -not $Tag) { $Tag = "v$($workspace.Version)" }
Assert-ReleaseVersion $workspace.Version $Tag
if (-not $OutputDir) { $OutputDir = Join-Path $script:RepositoryRoot 'dist' }
$OutputDir = Get-FileSystemPath $OutputDir
$archive = Join-Path $OutputDir "ruxcmd-$($workspace.Version)-$($workspace.Target).zip"
$checksum = "$archive.sha256"
if ((Test-Path -LiteralPath $archive) -or (Test-Path -LiteralPath $checksum)) { throw 'Release output already exists; use a different OutputDir or version.' }
if ($NotesFile) { $NotesFile = Get-FileSystemPath $NotesFile }
else { $NotesFile = Join-Path $script:RepositoryRoot 'CHANGELOG.md' }
if ($Publish -and (-not (Test-Path -LiteralPath $NotesFile -PathType Leaf) -or [string]::IsNullOrWhiteSpace((Get-Content -LiteralPath $NotesFile -Raw)))) { throw 'Release notes must be a nonempty file.' }
$commitResult = Invoke-ToolProcess git @('rev-parse', 'HEAD')
Assert-ToolSuccess $commitResult 'git HEAD'
$commit = $commitResult.Output.Trim()
$tagExists = $false
$tagCommit = ''
if ($Tag) {
    $tagResult = Invoke-ToolProcess git @('rev-parse', '--verify', '--quiet', "refs/tags/$Tag^{commit}")
    if ($tagResult.ExitCode -eq 0) { $tagExists = $true; $tagCommit = $tagResult.Output.Trim() }
    elseif ($tagResult.ExitCode -ne 1) { Assert-ToolSuccess $tagResult 'git tag lookup' }
    elseif (-not $Publish) { throw "Tag '$Tag' does not exist locally. Create/check out the tag, or omit -Tag for a local candidate." }
    if ($tagCommit -and $tagCommit -cne $commit) { throw 'Release tag points to another commit; tags are never moved or overwritten.' }
}
if ($Publish -or $RequireClean) {
    $status = Invoke-ToolProcess git @('-c', 'core.fsmonitor=false', '-c', 'core.untrackedCache=false', 'status', '--porcelain', '--untracked-files=normal')
    Assert-ToolSuccess $status 'git status'
    Assert-ReleaseSource $status.Output $commit $tagCommit
}
$action = if ($Publish) { "Verify, package and publish $Tag on GitHub (draft=$Draft)" } else { 'Build and verify a local release ZIP' }
Write-Host "RuxCmd $($workspace.Version) | $($workspace.Target) | commit $commit"
Write-Host "Output: $archive"
if (-not $PSCmdlet.ShouldProcess($script:RepositoryRoot, $action)) { return }
$clock = [Diagnostics.Stopwatch]::StartNew()
if ($Publish) {
    if (-not (Get-Command gh -ErrorAction SilentlyContinue)) { throw 'Install GitHub CLI and run gh auth login before publishing.' }
    $remote = Invoke-ToolProcess git @('remote', 'get-url', 'origin')
    Assert-ToolSuccess $remote 'git origin'
    $repository = Get-GitHubRepository $remote.Output.Trim()
    Assert-ToolSuccess (Invoke-ToolProcess gh @('auth', 'status', '--hostname', 'github.com')) 'GitHub authentication'
    $existing = Invoke-ToolProcess gh @('api', '--hostname', 'github.com', "repos/$repository/releases/tags/$Tag")
    if ($existing.ExitCode -eq 0) { throw "GitHub Release '$Tag' already exists; releases are never overwritten." }
    if ($existing.Error -notmatch 'HTTP 404') { Assert-ToolSuccess $existing 'GitHub release lookup' }
    $remoteTag = Invoke-ToolProcess git @('ls-remote', '--exit-code', 'origin', "refs/tags/$Tag", "refs/tags/$Tag^{}")
    if ($remoteTag.ExitCode -notin @(0, 2)) { Assert-ToolSuccess $remoteTag 'remote tag lookup' }
    Assert-RemoteTag $remoteTag.Output $Tag $commit
    & (Join-Path $PSScriptRoot 'build.ps1') -Task Verify
    if (-not $tagExists) {
        if ($remoteTag.Output.Trim()) {
            Assert-ToolSuccess (Invoke-ToolProcess git @('fetch', '--no-tags', 'origin', "refs/tags/${Tag}:refs/tags/$Tag")) 'fetch existing version tag'
            $fetched = Invoke-ToolProcess git @('rev-parse', '--verify', "refs/tags/$Tag^{commit}")
            Assert-ToolSuccess $fetched 'fetched tag lookup'
            Assert-ReleaseSource '' $commit $fetched.Output.Trim()
        } else {
            Assert-ToolSuccess (Invoke-ToolProcess git @('tag', '-a', $Tag, '-m', "RuxCmd $($workspace.Version)", $commit)) 'create release tag'
        }
    }
}
$packageArguments = @{ OutputDir = $OutputDir; RequireClean = [bool]($RequireClean -or $Publish) }
if ($Tag) { $packageArguments.ExpectedTag = $Tag }
& (Join-Path $PSScriptRoot 'package.ps1') @packageArguments
Assert-ReleaseAssets $archive $checksum $workspace $commit -RequireClean:($RequireClean -or $Publish)
if ($Publish) {
    # Recheck the source immediately before the first remote mutation.
    $current = Invoke-ToolProcess git @('rev-parse', 'HEAD')
    Assert-ToolSuccess $current 'git HEAD'
    if ($current.Output.Trim() -cne $commit) { throw 'HEAD changed during release preparation.' }
    $status = Invoke-ToolProcess git @('-c', 'core.fsmonitor=false', '-c', 'core.untrackedCache=false', 'status', '--porcelain', '--untracked-files=normal')
    Assert-ToolSuccess $status 'git status'
    $currentTag = Invoke-ToolProcess git @('rev-parse', '--verify', "refs/tags/$Tag^{commit}")
    Assert-ToolSuccess $currentTag 'final tag lookup'
    Assert-ReleaseSource $status.Output $commit $currentTag.Output.Trim()
    Assert-ToolSuccess (Invoke-ToolProcess git @('push', 'origin', "refs/tags/${Tag}:refs/tags/$Tag")) 'push version tag'
    $releaseArguments = @('release', 'create', $Tag, $archive, $checksum, '--repo', "github.com/$repository", '--verify-tag', '--title', "RuxCmd $($workspace.Version)", '--notes-file', $NotesFile)
    if ($Draft) { $releaseArguments += '--draft' }
    if ($workspace.Version.Contains('-')) { $releaseArguments += '--prerelease' }
    $published = Invoke-ToolProcess gh $releaseArguments
    Assert-ToolSuccess $published 'create GitHub Release (local package and pushed tag are retained if upload fails)'
    Write-Host $published.Output.Trim()
}
Write-Host "Release completed in $([Math]::Round($clock.Elapsed.TotalSeconds, 1))s." -ForegroundColor Green
Write-Host "ZIP: $archive`nSHA256: $checksum"

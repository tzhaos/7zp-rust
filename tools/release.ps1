[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^v\d+\.\d+\.\d+$')][string]$Tag,
    [switch]$Publish
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$names = @('7zplus-amd64-installer.exe', '7zplus-amd64-portable.zip')

function Confirm-ReleaseAssets([string]$Directory) {
    $checksums = @(Get-Content -LiteralPath (Join-Path $Directory 'SHA256SUMS.txt'))
    if ($checksums.Count -ne $names.Count) { throw 'Unexpected release checksum manifest' }
    foreach ($name in $names) {
        $hash = (Get-FileHash -LiteralPath (Join-Path $Directory $name) -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($checksums -cnotcontains "$hash  $name") { throw "Release checksum mismatch: $name" }
    }
}

function Confirm-RemoteAssets([switch]$MatchLocal) {
    $cacheRoot = [IO.Path]::GetFullPath((Join-Path $projectRoot '.tools/releases'))
    $download = Join-Path $cacheRoot ([guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Force -Path $download | Out-Null
    try {
        & gh release download $Tag --dir $download --pattern '7zplus-amd64-installer.exe' --pattern '7zplus-amd64-portable.zip' --pattern 'SHA256SUMS.txt'
        if ($LASTEXITCODE -ne 0) { throw 'Cannot download release assets for verification' }
        Confirm-ReleaseAssets $download
        if ($MatchLocal -and (Get-FileHash -LiteralPath (Join-Path $download 'SHA256SUMS.txt')).Hash -ne (Get-FileHash -LiteralPath 'dist/SHA256SUMS.txt').Hash) {
            throw 'Uploaded packages do not match the local build'
        }
    }
    finally {
        $resolvedDownload = (Resolve-Path -LiteralPath $download).Path
        if (-not $resolvedDownload.StartsWith($cacheRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Release verification directory is outside the project cache'
        }
        Remove-Item -LiteralPath $resolvedDownload -Recurse -Force
    }
}

Push-Location $projectRoot
try {
    $headCommit = & git rev-parse HEAD
    if ($LASTEXITCODE -ne 0) { throw 'Cannot identify release checkout' }
    $tagCommit = & git rev-parse "refs/tags/$Tag^{}"
    if ($LASTEXITCODE -ne 0 -or $tagCommit -ne $headCommit) { throw 'Release tag does not identify this checkout' }
    $build = Get-Content -LiteralPath 'dist/build-info.json' -Raw | ConvertFrom-Json
    $repository = & gh repo view --json nameWithOwner --jq .nameWithOwner
    if ($LASTEXITCODE -ne 0) { throw 'Cannot identify release repository' }
    if ($build.dirty -or $build.commit -ne $headCommit -or "v$($build.version)" -cne $Tag -or $build.repository -cne $repository -or $build.target -ne 'x86_64-pc-windows-msvc') {
        throw 'Build provenance does not match the clean tagged release checkout'
    }
    if ($build.checksums -ne (Get-FileHash -LiteralPath 'dist/SHA256SUMS.txt' -Algorithm SHA256).Hash.ToLowerInvariant()) {
        throw 'Build checksum manifest changed after packaging'
    }
    Confirm-ReleaseAssets 'dist'
    $assets = @($names | ForEach-Object { "dist/$_" }) + @('dist/SHA256SUMS.txt')
    $existing = & gh release view $Tag --json 'isDraft,assets' 2>$null
    if ($LASTEXITCODE -eq 0) {
        $release = $existing | ConvertFrom-Json
        if (-not $release.isDraft) {
            $missing = @($assets | Where-Object { [IO.Path]::GetFileName($_) -notin $release.assets.name })
            if ($missing.Count) { throw 'Published release is incomplete; inspect it before issuing a new version' }
            Confirm-RemoteAssets
            Write-Host "Release $Tag is already published and its packages passed checksum verification."
            return
        }
    }
    else {
        $notes = Join-Path $projectRoot ".github/release-notes/$Tag.md"
        $noteArgs = if (Test-Path -LiteralPath $notes) { @('--notes-file', $notes) } else { @('--generate-notes') }
        & gh release create $Tag --verify-tag --draft --title "7zplus $Tag" @noteArgs
        if ($LASTEXITCODE -ne 0) { throw 'Cannot create draft release' }
    }
    & gh release upload $Tag @assets --clobber
    if ($LASTEXITCODE -ne 0) { throw 'Upload failed; the release remains a draft and can be retried' }
    Confirm-RemoteAssets -MatchLocal
    if ($Publish) {
        & gh release edit $Tag --draft=false --latest
        if ($LASTEXITCODE -ne 0) { throw 'Cannot publish the completed draft release' }
    }
}
finally {
    Pop-Location
}

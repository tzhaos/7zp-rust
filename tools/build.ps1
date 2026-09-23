[CmdletBinding()]
param(
    [string]$ReleaseRepository = $env:GITHUB_REPOSITORY,
    [ValidateSet('Run', 'Portable', 'Installer', 'All')]
    [string]$Package = 'All'
)

$ErrorActionPreference = 'Stop'
$previousRepository = $env:SEVENZIP_RELEASE_REPOSITORY
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    if ($ReleaseRepository -and $ReleaseRepository -notmatch '^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$') {
        throw 'ReleaseRepository must be owner/repo'
    }
    $env:SEVENZIP_RELEASE_REPOSITORY = $ReleaseRepository
    $metadata = & cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read package version' }
    $version = ($metadata.packages | Where-Object name -eq 'cardo-7zp-app').version
    if ($version -notmatch '^\d+\.\d+\.\d+$') { throw 'Installer requires a stable major.minor.patch version' }
    & (Join-Path $PSScriptRoot 'prepare-engine.ps1')
    & (Join-Path $PSScriptRoot 'prepare-msvc.ps1')
    if ($Package -in @('Installer', 'All')) {
        $compiler = & (Join-Path $PSScriptRoot 'prepare-nsis.ps1')
    }
    & cargo build --workspace --locked --release --target x86_64-pc-windows-msvc --target-dir target
    if ($LASTEXITCODE -ne 0) { throw "Cargo build failed: $LASTEXITCODE" }
    Copy-Item -LiteralPath 'target/x86_64-pc-windows-msvc/release/7zplus.exe' -Destination 'bin/7zplus.exe' -Force
    Copy-Item -LiteralPath 'target/x86_64-pc-windows-msvc/release/cardo_7zp_explorer.dll' -Destination 'bin/7-zip-plus.dll' -Force
    Copy-Item -LiteralPath 'assets/fluent/LICENSE' -Destination 'bin/Fluent-LICENSE.txt' -Force
    foreach ($leftover in @('bin/cardo_7zp_explorer.dll', 'bin/cardo_7zp_shell.dll')) {
        if (Test-Path -LiteralPath $leftover) {
            Remove-Item -LiteralPath $leftover
        }
    }
    $artifacts = @()
    if ($Package -ne 'Run') {
        New-Item -ItemType Directory -Force -Path 'dist' | Out-Null
    }
    if ($Package -in @('Portable', 'All')) {
        $stagingBase = [IO.Path]::GetFullPath((Join-Path $projectRoot '.tools/packages'))
        $staging = Join-Path $stagingBase ([guid]::NewGuid().ToString('N'))
        $portableRoot = Join-Path $staging '7zplus'
        New-Item -ItemType Directory -Force -Path (Join-Path $portableRoot 'runtime/7zip') | Out-Null
        try {
            foreach ($name in @('7zplus.exe', '7-zip-plus.dll', 'vcruntime140.dll', 'Fluent-LICENSE.txt')) {
                Copy-Item -LiteralPath (Join-Path 'bin' $name) -Destination (Join-Path $portableRoot $name)
            }
            $engine = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'engine.lock.json') -Raw | ConvertFrom-Json
            foreach ($name in $engine.files) {
                Copy-Item -LiteralPath (Join-Path 'bin/runtime/7zip' $name) -Destination (Join-Path $portableRoot "runtime/7zip/$name")
            }
            $archive = Join-Path $projectRoot 'dist/7zplus-amd64-portable.zip'
            if (Test-Path -LiteralPath $archive) { Remove-Item -LiteralPath $archive }
            [IO.Compression.ZipFile]::CreateFromDirectory($portableRoot, $archive, [IO.Compression.CompressionLevel]::Optimal, $true)
            $artifacts += $archive
        }
        finally {
            $resolvedStaging = (Resolve-Path -LiteralPath $staging).Path
            if (-not $resolvedStaging.StartsWith($stagingBase + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
                throw 'Package staging directory is outside the project packaging cache'
            }
            Remove-Item -LiteralPath $resolvedStaging -Recurse -Force
        }
    }
    if ($Package -in @('Installer', 'All')) {
        $shellHash = (Get-FileHash -LiteralPath 'bin/7-zip-plus.dll' -Algorithm SHA256).Hash.ToLowerInvariant()
        & $compiler /V2 /INPUTCHARSET UTF8 "/DPROJECT_ROOT=$projectRoot" "/DAPP_VERSION=$version" "/DSHELL_HASH=$shellHash" (Join-Path $projectRoot 'packaging/installer.nsi')
        if ($LASTEXITCODE -ne 0) { throw "Installer build failed: $LASTEXITCODE" }
        $artifacts += Join-Path $projectRoot 'dist/7zplus-amd64-installer.exe'
    }
    if ($artifacts.Count) {
        $artifacts | ForEach-Object {
            $hash = (Get-FileHash -LiteralPath $_ -Algorithm SHA256).Hash.ToLowerInvariant()
            "$hash  $([IO.Path]::GetFileName($_))"
        } | Set-Content -LiteralPath 'dist/SHA256SUMS.txt' -Encoding ascii
        $commit = & git rev-parse HEAD
        if ($LASTEXITCODE -ne 0) { throw 'Cannot identify build commit' }
        $changes = & git status --porcelain
        if ($LASTEXITCODE -ne 0) { throw 'Cannot identify build working tree state' }
        [ordered]@{
            version = $version
            commit = $commit
            dirty = [bool]$changes
            repository = $ReleaseRepository
            target = 'x86_64-pc-windows-msvc'
            checksums = (Get-FileHash -LiteralPath 'dist/SHA256SUMS.txt' -Algorithm SHA256).Hash.ToLowerInvariant()
        } | ConvertTo-Json | Set-Content -LiteralPath 'dist/build-info.json' -Encoding utf8
    }
    foreach ($legacy in @('bin/7zplus-amd64-installer.exe', 'bin/SHA256SUMS.txt', 'bin/THIRD_PARTY.md')) {
        if (Test-Path -LiteralPath $legacy) { Remove-Item -LiteralPath $legacy }
    }
    Write-Host "Application: $projectRoot/bin/7zplus.exe"
    $artifacts | ForEach-Object { Write-Host "Package: $_" }
}
finally {
    $env:SEVENZIP_RELEASE_REPOSITORY = $previousRepository
    Pop-Location
}

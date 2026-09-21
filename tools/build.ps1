[CmdletBinding()]
param([string]$ReleaseRepository = $env:GITHUB_REPOSITORY)

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
    $version = ($metadata.packages | Where-Object name -eq 'sevenzip-plus').version
    if ($version -notmatch '^\d+\.\d+\.\d+$') { throw 'Installer requires a stable major.minor.patch version' }
    & cargo build --workspace --locked --release --target x86_64-pc-windows-msvc --target-dir target
    if ($LASTEXITCODE -ne 0) { throw "Cargo build failed: $LASTEXITCODE" }
    & (Join-Path $PSScriptRoot 'prepare-engine.ps1')
    Copy-Item -LiteralPath 'target/x86_64-pc-windows-msvc/release/7zplus.exe' -Destination 'bin/7zplus.exe' -Force
    Copy-Item -LiteralPath 'target/x86_64-pc-windows-msvc/release/sevenzip_shell.dll' -Destination 'bin/7-zip-plus.dll' -Force
    if (Test-Path -LiteralPath 'bin/sevenzip_shell.dll') {
        Remove-Item -LiteralPath 'bin/sevenzip_shell.dll'
    }
    $shellHash = (Get-FileHash -LiteralPath 'bin/7-zip-plus.dll' -Algorithm SHA256).Hash.ToLowerInvariant()
    $compiler = & (Join-Path $PSScriptRoot 'prepare-nsis.ps1')
    & $compiler /V2 /INPUTCHARSET UTF8 "/DPROJECT_ROOT=$projectRoot" "/DAPP_VERSION=$version" "/DSHELL_HASH=$shellHash" (Join-Path $projectRoot 'packaging/installer.nsi')
    if ($LASTEXITCODE -ne 0) { throw "Installer build failed: $LASTEXITCODE" }
    $hash = (Get-FileHash -LiteralPath 'bin/7zplus-amd64-installer.exe' -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  7zplus-amd64-installer.exe" | Set-Content -LiteralPath 'bin/SHA256SUMS.txt' -Encoding ascii
    Write-Host "Application: $projectRoot/bin/7zplus.exe"
    Write-Host "Installer: $projectRoot/bin/7zplus-amd64-installer.exe"
}
finally {
    $env:SEVENZIP_RELEASE_REPOSITORY = $previousRepository
    Pop-Location
}

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$engine = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'engine.lock.json') -Raw | ConvertFrom-Json
$cache = Join-Path $projectRoot ".tools/engine-$($engine.version)"
$unpacked = Join-Path $cache 'unpacked'
$runtime = Join-Path $projectRoot 'bin/runtime/7zip'
New-Item -ItemType Directory -Force -Path $cache, $unpacked, $runtime | Out-Null

foreach ($artifact in @($engine.installer, $engine.bootstrap)) {
    $download = Join-Path $cache $artifact.name
    if (-not (Test-Path -LiteralPath $download)) {
        Invoke-WebRequest -Uri $artifact.url -OutFile $download
    }
    if ((Get-FileHash -LiteralPath $download -Algorithm SHA256).Hash -ne $artifact.sha256) {
        throw "SHA-256 mismatch: $download"
    }
}

$bootstrap = Join-Path $cache $engine.bootstrap.name
$installer = Join-Path $cache $engine.installer.name
& $bootstrap x $installer "-o$unpacked" -y
if ($LASTEXITCODE -ne 0) { throw "7-Zip extraction failed: $LASTEXITCODE" }
foreach ($name in $engine.files) {
    Copy-Item -LiteralPath (Join-Path $unpacked $name) -Destination (Join-Path $runtime $name) -Force
}
Write-Host "7-Zip $($engine.version) prepared at $runtime"

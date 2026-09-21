[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$tools = Join-Path $root '.tools'
$compiler = Join-Path $tools 'nsis-3.12/makensis.exe'
if (Test-Path -LiteralPath $compiler) { return $compiler }
New-Item -ItemType Directory -Force -Path $tools | Out-Null
$archive = Join-Path $tools 'nsis-3.12.zip'
Invoke-WebRequest -Uri 'https://downloads.sourceforge.net/project/nsis/NSIS%203/3.12/nsis-3.12.zip' -OutFile $archive
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash -ne '56581F90DB321581C5381193D796FFFCF2D24B2F8FED2160A6C6A3BAA67F2C4F') {
    throw 'NSIS archive SHA-256 mismatch'
}
Expand-Archive -LiteralPath $archive -DestinationPath $tools -Force
return $compiler

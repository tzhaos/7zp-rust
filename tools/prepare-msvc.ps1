[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
$vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe'
$installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
if ($LASTEXITCODE -ne 0 -or -not $installation) { throw 'Visual C++ build tools with redistributable files are required' }
$redistRoot = Join-Path $installation 'VC/Redist/MSVC'
$versions = Get-ChildItem -LiteralPath $redistRoot -Directory | Where-Object Name -match '^\d+\.\d+\.\d+$' | Sort-Object { [version]$_.Name } -Descending
$runtime = $versions | ForEach-Object {
    Get-ChildItem -Path (Join-Path $_.FullName 'x64/Microsoft.VC*.CRT/vcruntime140.dll') -File -ErrorAction SilentlyContinue
} | Select-Object -First 1
if (-not $runtime) { throw 'The x64 Visual C++ redistributable vcruntime140.dll was not found' }
Copy-Item -LiteralPath $runtime.FullName -Destination (Join-Path $projectRoot 'bin/vcruntime140.dll') -Force
Write-Host "Visual C++ runtime: $($runtime.VersionInfo.FileVersion)"

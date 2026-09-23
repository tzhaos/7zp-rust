[CmdletBinding()]
param(
    [ValidateSet('Show', 'Patch', 'Minor', 'Major')]
    [string]$Part = 'Show'
)

$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
Push-Location $projectRoot
try {
    $metadata = & cargo metadata --no-deps --format-version 1 | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read package version' }
    $current = ($metadata.packages | Where-Object name -eq 'zip-app').version
    if ($Part -eq 'Show') { return $current }
    $version = [version]$current
    $next = switch ($Part) {
        'Patch' { '{0}.{1}.{2}' -f $version.Major, $version.Minor, ($version.Build + 1) }
        'Minor' { '{0}.{1}.0' -f $version.Major, ($version.Minor + 1) }
        'Major' { '{0}.0.0' -f ($version.Major + 1) }
    }
    $manifestPath = Join-Path $projectRoot 'Cargo.toml'
    $manifest = [IO.File]::ReadAllText($manifestPath)
    $pattern = '(?m)(^\[workspace\.package\]\r?\nversion = ")[^"]+("\r?$)'
    if (-not [regex]::IsMatch($manifest, $pattern)) { throw 'Workspace version field not found' }
    [IO.File]::WriteAllText($manifestPath, [regex]::Replace($manifest, $pattern, ('${1}' + $next + '${2}')))
    & cargo update --workspace --offline
    if ($LASTEXITCODE -ne 0) { throw 'Cannot synchronize Cargo.lock; resolve the error before committing' }
    Write-Host "Version: $current -> $next"
    Write-Host "Commit Cargo.toml and Cargo.lock together, then tag that commit v$next for release."
}
finally {
    Pop-Location
}

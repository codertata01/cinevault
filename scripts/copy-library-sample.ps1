param(
    [Parameter(Mandatory = $true)]
    [string] $SourceRoot,

    [string] $DestRoot = (Join-Path $PSScriptRoot "..\dev\library-copy"),

    [int] $MaxFolders = 3
)

$ErrorActionPreference = "Stop"

if (-not (Test-Path -LiteralPath $SourceRoot)) {
    throw "SourceRoot not found: $SourceRoot"
}

New-Item -ItemType Directory -Force -Path $DestRoot | Out-Null

$children = Get-ChildItem -LiteralPath $SourceRoot -Directory | Select-Object -First $MaxFolders

if ($children.Count -eq 0) {
    Write-Host "No child folders under $SourceRoot (nothing copied)."
    exit 0
}

foreach ($dir in $children) {
    $target = Join-Path $DestRoot $dir.Name
    Write-Host "Copying $($dir.FullName) -> $target"
    Copy-Item -LiteralPath $dir.FullName -Destination $target -Recurse -Force
}

Write-Host "Done. Writable test library: $DestRoot"
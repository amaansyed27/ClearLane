$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "target\bundle\clearlane.exe"
if (-not (Test-Path $exe)) {
    throw "ClearLane is not built. Run .\scripts\setup.ps1 first."
}
Start-Process -FilePath $exe -ArgumentList $args

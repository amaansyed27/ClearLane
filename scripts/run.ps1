$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root "target\bundle\clearlane.exe"
if (-not (Test-Path $exe)) {
    throw "ClearLane is not built. Run .\scripts\setup.ps1 first."
}

$bundleDir = Split-Path -Parent $exe
Write-Host "Launching ClearLane from $bundleDir"

$start = @{
    FilePath = $exe
    WorkingDirectory = $bundleDir
    PassThru = $true
}
if ($args.Count -gt 0) {
    $start.ArgumentList = $args
}

$process = Start-Process @start
Start-Sleep -Milliseconds 1500
$process.Refresh()

if ($process.HasExited) {
    throw "ClearLane exited during startup with code $($process.ExitCode)."
}

Write-Host "ClearLane is running (PID $($process.Id))."

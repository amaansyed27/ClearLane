$ErrorActionPreference = "Stop"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    throw "Rust/Cargo was not found. Install Rust stable from https://rustup.rs and reopen PowerShell."
}
if (-not (Get-Command cmake -ErrorAction SilentlyContinue)) {
    throw "CMake was not found. Install CMake and reopen PowerShell."
}
if (-not (Get-Command ninja -ErrorAction SilentlyContinue)) {
    throw "Ninja was not found. Install it with: winget install Ninja-build.Ninja, then reopen PowerShell."
}

$runningClearLane = Get-Process clearlane -ErrorAction SilentlyContinue
if ($runningClearLane) {
    $ids = ($runningClearLane | Select-Object -ExpandProperty Id) -join ", "
    throw "ClearLane is still running (PID(s): $ids) and will lock the CEF bundle during rebuild. Close ClearLane or run: Get-Process clearlane -ErrorAction SilentlyContinue | Stop-Process -Force"
}

$root = Split-Path -Parent $PSScriptRoot
$bundleDir = Join-Path $root "target\bundle"
$filterDir = Join-Path $env:LOCALAPPDATA "ClearLane\filters"
New-Item -ItemType Directory -Force -Path $filterDir | Out-Null

Write-Host "Updating Shields filter lists and scriptlet resources..."
$shieldAssets = @(
    @{ Name = "easylist.txt"; Url = "https://easylist.to/easylist/easylist.txt" },
    @{ Name = "easyprivacy.txt"; Url = "https://easylist.to/easylist/easyprivacy.txt" },
    @{ Name = "ublock-filters.txt"; Url = "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/filters.txt" },
    @{ Name = "ublock-privacy.txt"; Url = "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/privacy.txt" },
    @{ Name = "ublock-quick-fixes.txt"; Url = "https://raw.githubusercontent.com/uBlockOrigin/uAssets/master/filters/quick-fixes.txt" },
    @{ Name = "resources.json"; Url = "https://raw.githubusercontent.com/brave/adblock-resources/master/dist/resources.json" }
)

foreach ($asset in $shieldAssets) {
    try {
        Invoke-WebRequest -UseBasicParsing $asset.Url -OutFile (Join-Path $filterDir $asset.Name)
    } catch {
        Write-Warning "Could not update Shields asset $($asset.Name). Any existing cached copy will be kept."
    }
}

if (Test-Path $bundleDir) {
    Remove-Item -Recurse -Force $bundleDir
}

Push-Location $root
try {
    Write-Host "Building sandboxed ClearLane + CEF bundle (first build downloads CEF)..."
    cargo run --release --bin clearlane-bundle
    if ($LASTEXITCODE -ne 0) { throw "ClearLane bundle build failed." }
} finally {
    Pop-Location
}

Write-Host "Ready. Run: .\scripts\run.ps1"

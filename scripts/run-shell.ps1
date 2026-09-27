$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    cargo run --release
}
finally {
    Pop-Location
}

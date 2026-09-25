#[cfg(target_os = "windows")]
fn main() -> Result<(), &'static str> {
    Err(
        "ClearLane must be launched from the sandboxed Windows bundle. Run scripts/setup.ps1, then scripts/run.ps1.",
    )
}

#[cfg(not(target_os = "windows"))]
fn main() -> Result<(), &'static str> {
    Err("Slice 1 is Windows-first. Build ClearLane on Windows.")
}

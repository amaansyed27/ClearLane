#[cfg(all(target_os = "windows", feature = "sandbox"))]
fn main() -> Result<(), &'static str> {
    Err("ClearLane's Windows sandbox build must be launched from the bundled clearlane.exe. Run scripts/setup.ps1 first.")
}

#[cfg(all(target_os = "windows", not(feature = "sandbox")))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    clearlane::chromium::run_unsandboxed_for_development()
}

#[cfg(not(target_os = "windows"))]
fn main() -> Result<(), &'static str> {
    Err("Slice 1 is Windows-first. Build ClearLane on Windows.")
}

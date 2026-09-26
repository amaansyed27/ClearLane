#[cfg(target_os = "windows")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::current_dir()?.join("target").join("bundle");
    std::fs::create_dir_all(&output)?;
    let executable = cef::build_util::win::build_bundle(&output, "clearlane", true)?;
    println!("ClearLane bundle ready: {}", executable.display());
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn main() -> Result<(), &'static str> {
    Err("The Slice 1 bundle is Windows-only.")
}

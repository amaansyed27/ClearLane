use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use cef::*;

pub(crate) fn startup_log(message: &str) {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("ClearLane");
    let _ = fs::create_dir_all(&base);
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(base.join("startup.log"))
    {
        let _ = writeln!(file, "pid={} {message}", std::process::id());
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn RunWinMain(
    instance: sys::HINSTANCE,
    _command_line: *const u8,
    _command_show: i32,
    sandbox_info: *mut u8,
) -> i32 {
    startup_log("RunWinMain entered");
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    startup_log("CEF API initialized");

    let main_args = MainArgs { instance };
    let args = args::Args::from(main_args);
    let Some(command_line) = args.as_cmd_line() else {
        startup_log("failed to parse command line");
        return 1;
    };

    startup_log("starting Chromium runtime");
    match crate::chromium::run(args.as_main_args(), &command_line, sandbox_info) {
        Ok(()) => {
            startup_log("Chromium runtime exited normally");
            0
        }
        Err(error) => {
            startup_log(&format!("startup failed: {error}"));
            eprintln!("ClearLane startup failed: {error}");
            1
        }
    }
}

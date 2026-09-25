use cef::*;

#[unsafe(no_mangle)]
unsafe extern "C" fn RunWinMain(
    instance: sys::HINSTANCE,
    _command_line: *const u8,
    _command_show: i32,
    sandbox_info: *mut u8,
) -> i32 {
    let _ = api_hash(sys::CEF_API_VERSION_LAST, 0);
    let main_args = MainArgs { instance };
    let args = args::Args::from(main_args);
    let Some(command_line) = args.as_cmd_line() else { return 1; };
    match crate::chromium::run(args.as_main_args(), &command_line, sandbox_info) {
        Ok(()) => 0,
        Err(error) => { eprintln!("ClearLane startup failed: {error}"); 1 }
    }
}

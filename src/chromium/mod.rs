mod client;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak},
};

use cef::*;
use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos, ShowWindow,
    },
};

use crate::{app::Runtime, core::TabId, shields::Shields};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct EngineBounds {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Default)]
pub(crate) struct EngineSet {
    browsers: HashMap<TabId, Browser>,
}

impl EngineSet {
    pub(crate) fn attach(&mut self, tab_id: TabId, browser: Browser) {
        self.browsers.insert(tab_id, browser);
    }

    pub(crate) fn detach(&mut self, tab_id: TabId) {
        self.browsers.remove(&tab_id);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.browsers.is_empty()
    }

    fn clone_browser(&self, tab_id: TabId) -> Option<Browser> {
        self.browsers.get(&tab_id).cloned()
    }

    fn clone_all(&self) -> Vec<Browser> {
        self.browsers.values().cloned().collect()
    }

    pub(crate) fn focus(&self, tab_id: TabId) {
        if let Some(browser) = self.browsers.get(&tab_id) {
            if let Some(host) = browser.host() {
                host.set_focus(1);
            }
        }
    }

    pub(crate) fn layout(&self, bounds: EngineBounds, active: Option<TabId>) {
        for (tab_id, browser) in &self.browsers {
            let Some(host) = browser.host() else {
                continue;
            };
            let handle: HWND = host.window_handle().0.cast();
            if handle.is_null() {
                continue;
            }
            // SAFETY: CEF owns this child HWND and it remains valid while BrowserHost is alive.
            unsafe {
                SetWindowPos(
                    handle,
                    std::ptr::null_mut(),
                    bounds.x,
                    bounds.y,
                    bounds.width,
                    bounds.height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                ShowWindow(
                    handle,
                    if Some(*tab_id) == active {
                        SW_SHOW
                    } else {
                        SW_HIDE
                    },
                );
            }
        }
    }
}

pub(crate) fn create_browser(
    runtime: Weak<Mutex<Runtime>>,
    shields: Arc<Mutex<Shields>>,
    parent: HWND,
    bounds: EngineBounds,
    tab_id: TabId,
    url: &str,
) {
    let window_info = WindowInfo::default().set_as_child(
        sys::HWND(parent.cast()),
        &Rect {
            x: bounds.x,
            y: bounds.y,
            width: bounds.width,
            height: bounds.height,
        },
    );
    let current_url = Arc::new(Mutex::new(url.to_string()));
    let mut browser_client = client::BrowserClient::new(client::ClientContext {
        tab_id,
        runtime,
        shields,
        current_url,
    });
    let url = CefString::from(url);
    let settings = BrowserSettings::default();
    let created = browser_host_create_browser(
        Some(&window_info),
        Some(&mut browser_client),
        Some(&url),
        Some(&settings),
        None,
        None,
    );
    if created == 0 {
        eprintln!(
            "ClearLane: CEF rejected browser creation for tab {}",
            tab_id.0
        );
    }
}

fn active_browser(runtime: &Arc<Mutex<Runtime>>) -> Option<Browser> {
    let locked = runtime.lock().ok()?;
    locked
        .state
        .active_id()
        .and_then(|id| locked.engine.clone_browser(id))
}

pub(crate) fn navigate_active(runtime: &Arc<Mutex<Runtime>>, url: &str) {
    let Some(browser) = active_browser(runtime) else {
        return;
    };
    if let Some(frame) = browser.main_frame() {
        frame.load_url(Some(&CefString::from(url)));
    }
}

pub(crate) fn go_back_active(runtime: &Arc<Mutex<Runtime>>) {
    if let Some(browser) = active_browser(runtime) {
        if browser.can_go_back() != 0 {
            browser.go_back();
        }
    }
}

pub(crate) fn go_forward_active(runtime: &Arc<Mutex<Runtime>>) {
    if let Some(browser) = active_browser(runtime) {
        if browser.can_go_forward() != 0 {
            browser.go_forward();
        }
    }
}

pub(crate) fn reload_active(runtime: &Arc<Mutex<Runtime>>) {
    if let Some(browser) = active_browser(runtime) {
        browser.reload();
    }
}

pub(crate) fn stop_active(runtime: &Arc<Mutex<Runtime>>) {
    if let Some(browser) = active_browser(runtime) {
        browser.stop_load();
    }
}

pub(crate) fn close_active(runtime: &Arc<Mutex<Runtime>>, force: bool) {
    if let Some(browser) = active_browser(runtime) {
        close_browser(browser, force);
    }
}

pub(crate) fn close_all(runtime: &Arc<Mutex<Runtime>>, force: bool) {
    let (browsers, hwnd) = {
        let Ok(locked) = runtime.lock() else {
            return;
        };
        (locked.engine.clone_all(), locked.hwnd)
    };
    if browsers.is_empty() {
        // SAFETY: hwnd is the live top-level ClearLane window.
        unsafe {
            windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow(hwnd);
        }
        return;
    }
    for browser in browsers {
        close_browser(browser, force);
    }
}

fn close_browser(browser: Browser, force: bool) {
    if let Some(host) = browser.host() {
        host.close_browser(force as i32);
    }
}

pub(crate) fn run(
    main_args: &MainArgs,
    command_line: &CommandLine,
    sandbox_info: *mut u8,
) -> Result<(), String> {
    let switch = CefString::from("type");
    let process_type = CefString::from(&command_line.switch_value(Some(&switch))).to_string();
    let browser_process = command_line.has_switch(Some(&switch)) != 1;
    crate::win::startup_log(&format!(
        "process classified browser_process={browser_process} type='{process_type}'"
    ));
    crate::win::startup_log("calling execute_process");
    let ret = execute_process(Some(main_args), None, sandbox_info);
    crate::win::startup_log(&format!("execute_process returned {ret}"));
    if !browser_process {
        return if ret >= 0 {
            Ok(())
        } else {
            Err("CEF subprocess failed".into())
        };
    }
    if ret != -1 {
        return Err(format!(
            "CEF browser process returned unexpected code {ret}"
        ));
    }

    let state_dir = crate::app::state_dir();
    std::fs::create_dir_all(&state_dir).map_err(|error| error.to_string())?;
    let cache_dir = crate::app::chromium_cache_dir(&state_dir);
    std::fs::create_dir_all(&cache_dir).map_err(|error| error.to_string())?;
    crate::win::startup_log(&format!(
        "browser process state_dir='{}' cache_dir='{}'",
        state_dir.display(),
        cache_dir.display()
    ));
    let cache = CefString::from(cache_dir.to_string_lossy().as_ref());
    let mut app = ClearLaneApp::new(state_dir);
    let settings = Settings {
        no_sandbox: (!cfg!(feature = "sandbox")) as _,
        root_cache_path: cache.clone(),
        cache_path: cache,
        persist_session_cookies: 1,
        ..Default::default()
    };
    crate::win::startup_log("calling cef_initialize");
    let initialized = initialize(
        Some(main_args),
        Some(&settings),
        Some(&mut app),
        sandbox_info,
    );
    crate::win::startup_log(&format!("cef_initialize returned {initialized}"));
    if initialized != 1 {
        return Err("CEF initialization failed".into());
    }
    crate::win::startup_log("entering CEF message loop");
    run_message_loop();
    crate::win::startup_log("CEF message loop exited");
    shutdown();
    crate::win::startup_log("CEF shutdown complete");
    Ok(())
}

wrap_app! {
    struct ClearLaneApp { state_dir: std::path::PathBuf }
    impl App {
        fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
            crate::win::startup_log("browser_process_handler requested");
            Some(ClearLaneBrowserProcessHandler::new(self.state_dir.clone()))
        }
    }
}

wrap_browser_process_handler! {
    struct ClearLaneBrowserProcessHandler { state_dir: std::path::PathBuf }
    impl BrowserProcessHandler {
        fn on_context_initialized(&self) {
            crate::win::startup_log("on_context_initialized entered");
            debug_assert_ne!(currently_on(ThreadId::UI), 0);
            match crate::app::launch(self.state_dir.clone()) {
                Ok(_) => crate::win::startup_log("native ClearLane window launch completed"),
                Err(error) => {
                    crate::win::startup_log(&format!(
                        "native ClearLane window launch failed: {error}"
                    ));
                    eprintln!("ClearLane failed to create its browser window: {error}");
                    quit_message_loop();
                }
            }
        }
    }
}

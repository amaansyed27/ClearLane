mod persistence;
mod window;

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, Weak},
    time::{Duration, Instant},
};

use windows_sys::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{DestroyWindow, PostMessageW, SW_HIDE, SW_SHOW, ShowWindow},
};

use crate::{
    chromium::{self, EngineBounds, EngineSet},
    core::{BrowserState, TabId, normalize_omnibox},
    shields::Shields,
};
use persistence::Persistence;

pub(crate) const WM_APP_SHIELDS: u32 = 0x8000 + 1;
pub(crate) const WM_APP_NEW_TAB: u32 = 0x8000 + 2;

pub(crate) struct Runtime {
    pub(crate) hwnd: HWND,
    pub(crate) controls: window::Controls,
    pub(crate) state: BrowserState,
    pub(crate) engine: EngineSet,
    shields: Arc<Mutex<Shields>>,
    persistence: Persistence,
    state_dir: PathBuf,
    sidebar_open: bool,
    closing: bool,
    started_at: Instant,
    load_started: std::collections::HashMap<TabId, Instant>,
}

impl Runtime {
    fn empty(
        state_dir: PathBuf,
        persistence: Persistence,
        shields: Arc<Mutex<Shields>>,
        sidebar_open: bool,
    ) -> Self {
        Self {
            hwnd: std::ptr::null_mut(),
            controls: window::Controls::default(),
            state: BrowserState::default(),
            engine: EngineSet::default(),
            shields,
            persistence,
            state_dir,
            sidebar_open,
            closing: false,
            started_at: Instant::now(),
            load_started: std::collections::HashMap::new(),
        }
    }

    pub(crate) fn shields(&self) -> Arc<Mutex<Shields>> {
        self.shields.clone()
    }

    pub(crate) fn browser_bounds(&self) -> EngineBounds {
        window::browser_bounds(self.hwnd, self.sidebar_open)
    }

    pub(crate) fn on_browser_created(&mut self, tab_id: TabId) {
        self.engine
            .layout(self.browser_bounds(), self.state.active_id());
        self.refresh_ui();
        if self.started_at.elapsed() < Duration::from_secs(30) {
            let marker = self.state_dir.join("startup-ready.txt");
            let _ = fs::write(
                marker,
                format!("{}\n", self.started_at.elapsed().as_millis()),
            );
        }
        if self.state.active_id() == Some(tab_id) {
            self.engine.focus(tab_id);
        }
    }

    pub(crate) fn on_browser_closed(&mut self, tab_id: TabId) {
        self.state.close_tab(tab_id);
        if let Ok(mut shields) = self.shields.lock() {
            shields.clear_tab(tab_id.0);
        }
        if self.closing {
            if self.engine.is_empty() {
                unsafe {
                    DestroyWindow(self.hwnd);
                }
            }
            return;
        }
        if self.state.tabs().is_empty() {
            unsafe {
                PostMessageW(self.hwnd, WM_APP_NEW_TAB, 0, 0);
            }
        } else {
            self.engine
                .layout(self.browser_bounds(), self.state.active_id());
            self.refresh_ui();
        }
    }

    pub(crate) fn on_address(&mut self, tab_id: TabId, url: String) {
        self.state.update_address(tab_id, url);
        if self.state.active_id() == Some(tab_id) {
            self.refresh_ui();
        }
    }

    pub(crate) fn on_title(&mut self, tab_id: TabId, title: String) {
        self.state.update_title(tab_id, title);
        self.refresh_ui();
    }

    pub(crate) fn on_loading(&mut self, tab_id: TabId, loading: bool, back: bool, forward: bool) {
        self.state.update_loading(tab_id, loading, back, forward);
        if loading {
            self.load_started.entry(tab_id).or_insert_with(Instant::now);
        } else if let Some(started) = self.load_started.remove(&tab_id) {
            self.record_load(tab_id, started.elapsed());
        }
        if self.state.active_id() == Some(tab_id) {
            self.refresh_ui();
        }
    }

    pub(crate) fn notify_shields(&self, tab_id: TabId) {
        unsafe {
            PostMessageW(self.hwnd, WM_APP_SHIELDS, tab_id.0 as usize, 0);
        }
    }

    pub(crate) fn refresh_ui(&mut self) {
        window::refresh(self);
    }

    pub(crate) fn persist(&self) {
        let urls: Vec<String> = self
            .state
            .tabs()
            .iter()
            .map(|tab| tab.url.clone())
            .collect();
        let active_index = self
            .state
            .active_id()
            .and_then(|id| self.state.tabs().iter().position(|tab| tab.id == id))
            .unwrap_or(0);
        let disabled = self
            .shields
            .lock()
            .map(|s| s.disabled_sites())
            .unwrap_or_default();
        self.persistence
            .save(&urls, active_index, self.sidebar_open, &disabled);
    }

    fn record_load(&self, tab_id: TabId, elapsed: Duration) {
        let Some(tab) = self.state.tab(tab_id) else {
            return;
        };
        let path = self.state_dir.join("perf-loads.tsv");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
            let _ = writeln!(
                file,
                "{}\t{}\t{}",
                tab_id.0,
                elapsed.as_millis(),
                tab.url.replace('\t', " ")
            );
        }
    }
}

pub(crate) fn launch(state_dir: PathBuf) -> Result<Arc<Mutex<Runtime>>, String> {
    crate::win::startup_log("app launch: creating state directory");
    fs::create_dir_all(&state_dir).map_err(|error| error.to_string())?;

    crate::win::startup_log("app launch: loading persisted state");
    let persistence = Persistence::new(&state_dir);
    let saved = persistence.load();
    crate::win::startup_log("app launch: persisted state loaded");

    // Keep startup responsive: use the small built-in blocker immediately. EasyList and
    // EasyPrivacy are substantially larger and are compiled in the background after the
    // native browser shell is visible, then atomically swapped into the shared Shields state.
    let filter_dir = state_dir.join("filters");
    crate::win::startup_log("app launch: building fallback Shields engine");
    let shields = Arc::new(Mutex::new(Shields::fallback(
        saved.disabled_shields.clone(),
    )));
    crate::win::startup_log("app launch: fallback Shields engine ready");
    let shields_for_full_load = shields.clone();

    let runtime = Arc::new(Mutex::new(Runtime::empty(
        state_dir,
        persistence,
        shields,
        saved.sidebar_open,
    )));

    crate::win::startup_log("app launch: creating native window");
    window::create(runtime.clone())?;
    crate::win::startup_log("app launch: native window created");

    let _ = std::thread::Builder::new()
        .name("clearlane-shields-loader".into())
        .spawn(move || {
            crate::win::startup_log("full Shields filter compilation started");
            let engine = Shields::build_full_engine(&filter_dir);
            if let Ok(mut shields) = shields_for_full_load.lock() {
                shields.replace_engine(engine);
                crate::win::startup_log("full Shields filter compilation completed");
            } else {
                crate::win::startup_log("full Shields filter compilation could not acquire lock");
            }
        });

    crate::win::startup_log("app launch: opening restored tabs");
    let perf_tabs = perf_tab_count();
    let mut urls = if perf_tabs > 0 {
        perf_urls(perf_tabs)
    } else {
        saved.urls
    };
    if urls.is_empty() {
        urls.push("https://www.google.com/".into());
    }
    let wanted_active = if perf_tabs > 0 {
        0
    } else {
        saved.active_index.min(urls.len() - 1)
    };
    for url in urls {
        open_tab(&runtime, url);
    }
    let active_id = {
        let mut locked = runtime
            .lock()
            .map_err(|_| "Runtime lock poisoned".to_string())?;
        let id = locked
            .state
            .tabs()
            .get(wanted_active)
            .map(|tab| tab.id)
            .or_else(|| locked.state.active_id());
        if let Some(id) = id {
            locked.state.activate(id);
        }
        id
    };
    if let Some(id) = active_id {
        activate_tab(&runtime, id);
    }
    crate::win::startup_log("app launch: restored tabs opened");
    Ok(runtime)
}

pub(crate) fn open_tab(runtime: &Arc<Mutex<Runtime>>, url: String) {
    let (tab_id, hwnd, bounds, shields) = {
        let Ok(mut locked) = runtime.lock() else {
            return;
        };
        let tab_id = locked.state.open_tab(url.clone());
        locked.refresh_ui();
        (
            tab_id,
            locked.hwnd,
            locked.browser_bounds(),
            locked.shields(),
        )
    };
    chromium::create_browser(Arc::downgrade(runtime), shields, hwnd, bounds, tab_id, &url);
}

pub(crate) fn activate_tab(runtime: &Arc<Mutex<Runtime>>, tab_id: TabId) {
    let Ok(mut locked) = runtime.lock() else {
        return;
    };
    if locked.state.activate(tab_id) {
        let bounds = locked.browser_bounds();
        let active = locked.state.active_id();
        locked.engine.layout(bounds, active);
        locked.refresh_ui();
        locked.engine.focus(tab_id);
    }
}

pub(crate) fn close_active_tab(runtime: &Arc<Mutex<Runtime>>) {
    chromium::close_active(runtime, false);
}

pub(crate) fn navigate_from_omnibox(runtime: &Arc<Mutex<Runtime>>) {
    let raw = {
        let Ok(locked) = runtime.lock() else {
            return;
        };
        window::omnibox_text(locked.controls.omnibox)
    };
    if let Some(url) = normalize_omnibox(&raw) {
        chromium::navigate_active(runtime, &url);
    }
}

pub(crate) fn go_back(runtime: &Arc<Mutex<Runtime>>) {
    chromium::go_back_active(runtime);
}
pub(crate) fn go_forward(runtime: &Arc<Mutex<Runtime>>) {
    chromium::go_forward_active(runtime);
}
pub(crate) fn reload(runtime: &Arc<Mutex<Runtime>>) {
    chromium::reload_active(runtime);
}
pub(crate) fn stop(runtime: &Arc<Mutex<Runtime>>) {
    chromium::stop_active(runtime);
}

pub(crate) fn toggle_sidebar(runtime: &Arc<Mutex<Runtime>>) {
    let Ok(mut locked) = runtime.lock() else {
        return;
    };
    locked.sidebar_open = !locked.sidebar_open;
    let bounds = locked.browser_bounds();
    let active = locked.state.active_id();
    unsafe {
        ShowWindow(
            locked.controls.sidebar,
            if locked.sidebar_open {
                SW_SHOW
            } else {
                SW_HIDE
            },
        );
    }
    locked.engine.layout(bounds, active);
    locked.refresh_ui();
}

pub(crate) fn toggle_shields(runtime: &Arc<Mutex<Runtime>>) {
    let (url, enabled) = {
        let Ok(locked) = runtime.lock() else {
            return;
        };
        let Some(tab) = locked.state.active() else {
            return;
        };
        let enabled = locked
            .shields
            .lock()
            .map(|s| s.enabled_for_url(&tab.url))
            .unwrap_or(true);
        (tab.url.clone(), enabled)
    };
    if let Ok(locked) = runtime.lock() {
        if let Ok(mut shields) = locked.shields.lock() {
            shields.set_enabled_for_url(&url, !enabled);
        }
        locked.persist();
    }
    reload(runtime);
    if let Ok(mut locked) = runtime.lock() {
        locked.refresh_ui();
    }
}

pub(crate) fn begin_close(runtime: &Arc<Mutex<Runtime>>) {
    let should_close = {
        let Ok(mut locked) = runtime.lock() else {
            return;
        };
        if locked.closing {
            false
        } else {
            locked.closing = true;
            locked.persist();
            true
        }
    };
    if should_close {
        chromium::close_all(runtime, false);
    }
}

pub(crate) fn runtime_from_weak(runtime: &Weak<Mutex<Runtime>>) -> Option<Arc<Mutex<Runtime>>> {
    runtime.upgrade()
}

fn perf_tab_count() -> usize {
    let args: Vec<String> = std::env::args().collect();
    args.windows(2)
        .find(|pair| pair[0] == "--perf-tabs")
        .and_then(|pair| pair[1].parse::<usize>().ok())
        .unwrap_or(0)
        .min(30)
}

fn perf_urls(count: usize) -> Vec<String> {
    const URLS: &[&str] = &[
        "https://example.com/",
        "https://www.rust-lang.org/",
        "https://www.wikipedia.org/",
        "https://developer.mozilla.org/",
        "https://news.ycombinator.com/",
        "https://github.com/",
        "https://www.bbc.com/",
        "https://www.nasa.gov/",
        "https://www.python.org/",
        "https://www.sqlite.org/",
    ];
    (0..count)
        .map(|index| URLS[index % URLS.len()].to_string())
        .collect()
}

pub(crate) fn state_dir() -> PathBuf {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("ClearLane")
}

pub(crate) fn chromium_cache_dir(state_dir: &Path) -> PathBuf {
    state_dir.join("chromium")
}
use std::{ffi::c_void, sync::{Arc, Mutex}};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{GetStockObject, DEFAULT_GUI_FONT},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        Controls::{DefSubclassProc, SetWindowSubclass},
        HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        Input::KeyboardAndMouse::VK_RETURN,
        WindowsAndMessaging::*,
    },
};

use crate::{app::{self, Runtime, WM_APP_NEW_TAB, WM_APP_SHIELDS}, chromium::EngineBounds, core::TabId};

const CLASS_NAME: &str = "ClearLaneMainWindow";
const ID_SIDEBAR_TOGGLE: usize = 100;
const ID_BACK: usize = 101;
const ID_FORWARD: usize = 102;
const ID_RELOAD: usize = 103;
const ID_STOP: usize = 104;
const ID_OMNIBOX: usize = 105;
const ID_SHIELDS: usize = 106;
const ID_TAB_LIST: usize = 107;
const ID_NEW_TAB: usize = 108;
const ID_CLOSE_TAB: usize = 109;

#[derive(Clone, Copy)]
pub(crate) struct Controls {
    pub sidebar_toggle: HWND,
    pub back: HWND,
    pub forward: HWND,
    pub reload: HWND,
    pub stop: HWND,
    pub omnibox: HWND,
    pub shields: HWND,
    pub sidebar: HWND,
    pub tab_list: HWND,
    pub new_tab: HWND,
    pub close_tab: HWND,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            sidebar_toggle: std::ptr::null_mut(), back: std::ptr::null_mut(), forward: std::ptr::null_mut(),
            reload: std::ptr::null_mut(), stop: std::ptr::null_mut(), omnibox: std::ptr::null_mut(),
            shields: std::ptr::null_mut(), sidebar: std::ptr::null_mut(), tab_list: std::ptr::null_mut(),
            new_tab: std::ptr::null_mut(), close_tab: std::ptr::null_mut(),
        }
    }
}

pub(crate) fn create(runtime: Arc<Mutex<Runtime>>) -> Result<(), String> {
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = GetModuleHandleW(std::ptr::null());
        let class = wide(CLASS_NAME);
        let window_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            hIcon: std::ptr::null_mut(),
            hCursor: LoadCursorW(std::ptr::null_mut(), IDC_ARROW),
            hbrBackground: (COLOR_WINDOW + 1) as usize as _,
            lpszMenuName: std::ptr::null(),
            lpszClassName: class.as_ptr(),
        };
        RegisterClassW(&window_class);

        let boxed = Box::new(runtime.clone());
        let raw_runtime = Box::into_raw(boxed);
        let title = wide("ClearLane");
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT, CW_USEDEFAULT, 1280, 800,
            std::ptr::null_mut(), std::ptr::null_mut(), instance,
            raw_runtime.cast::<c_void>(),
        );
        if hwnd.is_null() {
            drop(Box::from_raw(raw_runtime));
            return Err("CreateWindowExW failed".to_string());
        }
        let controls = create_controls(hwnd, instance);
        {
            let mut locked = runtime.lock().map_err(|_| "Runtime lock poisoned".to_string())?;
            locked.hwnd = hwnd;
            locked.controls = controls;
            layout(&mut locked);
            refresh(&mut locked);
        }
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
        Ok(())
    }
}

unsafe fn create_controls(hwnd: HWND, instance: *mut c_void) -> Controls {
    let button = wide("BUTTON");
    let edit = wide("EDIT");
    let listbox = wide("LISTBOX");
    let sidebar = create_control(hwnd, instance, &button, "☰", BS_PUSHBUTTON as u32, ID_SIDEBAR_TOGGLE);
    let back = create_control(hwnd, instance, &button, "‹", BS_PUSHBUTTON as u32, ID_BACK);
    let forward = create_control(hwnd, instance, &button, "›", BS_PUSHBUTTON as u32, ID_FORWARD);
    let reload = create_control(hwnd, instance, &button, "↻", BS_PUSHBUTTON as u32, ID_RELOAD);
    let stop = create_control(hwnd, instance, &button, "×", BS_PUSHBUTTON as u32, ID_STOP);
    let omnibox = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        edit.as_ptr(), std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_AUTOHSCROLL as u32,
        0, 0, 100, 28, hwnd, ID_OMNIBOX as _, instance, std::ptr::null(),
    );
    let shields = create_control(hwnd, instance, &button, "Shields", BS_PUSHBUTTON as u32, ID_SHIELDS);
    let sidebar_panel = CreateWindowExW(
        0, wide("STATIC").as_ptr(), std::ptr::null(), WS_CHILD | WS_VISIBLE,
        0, 0, 200, 400, hwnd, std::ptr::null_mut(), instance, std::ptr::null(),
    );
    let new_tab = create_control(hwnd, instance, &button, "+", BS_PUSHBUTTON as u32, ID_NEW_TAB);
    let close_tab = create_control(hwnd, instance, &button, "−", BS_PUSHBUTTON as u32, ID_CLOSE_TAB);
    let tab_list = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        listbox.as_ptr(), std::ptr::null(),
        WS_CHILD | WS_VISIBLE | WS_VSCROLL | LBS_NOTIFY as u32,
        0, 0, 180, 300, hwnd, ID_TAB_LIST as _, instance, std::ptr::null(),
    );
    let controls = Controls { sidebar_toggle: sidebar, back, forward, reload, stop, omnibox, shields, sidebar: sidebar_panel, tab_list, new_tab, close_tab };
    let font = GetStockObject(DEFAULT_GUI_FONT);
    for control in [sidebar, back, forward, reload, stop, omnibox, shields, sidebar_panel, tab_list, new_tab, close_tab] {
        SendMessageW(control, WM_SETFONT, font as usize, 1);
    }
    SetWindowSubclass(omnibox, Some(omnibox_proc), 1, hwnd as usize);
    controls
}

unsafe fn create_control(hwnd: HWND, instance: *mut c_void, class: &[u16], label: &str, extra_style: u32, id: usize) -> HWND {
    let label = wide(label);
    CreateWindowExW(
        0, class.as_ptr(), label.as_ptr(), WS_CHILD | WS_VISIBLE | WS_TABSTOP | extra_style,
        0, 0, 30, 30, hwnd, id as _, instance, std::ptr::null(),
    )
}

pub(crate) fn browser_bounds(hwnd: HWND, sidebar_open: bool) -> EngineBounds {
    if hwnd.is_null() { return EngineBounds::default(); }
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect);
        let dpi = GetDpiForWindow(hwnd).max(96);
        let scale = dpi as i32;
        let toolbar = 48 * scale / 96;
        let sidebar = if sidebar_open { 224 * scale / 96 } else { 0 };
        EngineBounds { x: sidebar, y: toolbar, width: (rect.right - sidebar).max(1), height: (rect.bottom - toolbar).max(1) }
    }
}

fn layout(runtime: &mut Runtime) {
    if runtime.hwnd.is_null() || runtime.controls.omnibox.is_null() { return; }
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(runtime.hwnd, &mut rect);
        let dpi = GetDpiForWindow(runtime.hwnd).max(96) as i32;
        let s = |v: i32| v * dpi / 96;
        let h = s(48);
        let button = s(36);
        let gap = s(6);
        let side_w = if runtime.sidebar_open { s(224) } else { 0 };
        let shield_w = s(116);
        let mut x = gap;
        for control in [runtime.controls.sidebar_toggle, runtime.controls.back, runtime.controls.forward, runtime.controls.reload, runtime.controls.stop] {
            MoveWindow(control, x, s(6), button, s(34), 1); x += button + gap;
        }
        let omnibox_w = (rect.right - x - shield_w - gap * 2).max(s(140));
        MoveWindow(runtime.controls.omnibox, x, s(8), omnibox_w, s(30), 1);
        MoveWindow(runtime.controls.shields, x + omnibox_w + gap, s(6), shield_w, s(34), 1);
        if runtime.sidebar_open {
            ShowWindow(runtime.controls.sidebar, SW_SHOW);
            ShowWindow(runtime.controls.new_tab, SW_SHOW);
            ShowWindow(runtime.controls.close_tab, SW_SHOW);
            ShowWindow(runtime.controls.tab_list, SW_SHOW);
            MoveWindow(runtime.controls.sidebar, 0, h, side_w, rect.bottom - h, 1);
            MoveWindow(runtime.controls.new_tab, gap, h + gap, s(36), s(30), 1);
            MoveWindow(runtime.controls.close_tab, gap + s(42), h + gap, s(36), s(30), 1);
            MoveWindow(runtime.controls.tab_list, gap, h + s(44), side_w - gap * 2, rect.bottom - h - s(50), 1);
        } else {
            for control in [runtime.controls.sidebar, runtime.controls.new_tab, runtime.controls.close_tab, runtime.controls.tab_list] { ShowWindow(control, SW_HIDE); }
        }
        let bounds = browser_bounds(runtime.hwnd, runtime.sidebar_open);
        let active = runtime.state.active_id();
        runtime.engine.layout(bounds, active);
    }
}

pub(crate) fn refresh(runtime: &mut Runtime) {
    if runtime.controls.omnibox.is_null() { return; }
    unsafe {
        SendMessageW(runtime.controls.tab_list, LB_RESETCONTENT, 0, 0);
        for tab in runtime.state.tabs() {
            let label = if tab.title.trim().is_empty() { &tab.url } else { &tab.title };
            let text = wide(label);
            SendMessageW(runtime.controls.tab_list, LB_ADDSTRING, 0, text.as_ptr() as isize);
        }
        if let Some(active) = runtime.state.active_id() {
            if let Some(index) = runtime.state.tabs().iter().position(|tab| tab.id == active) {
                SendMessageW(runtime.controls.tab_list, LB_SETCURSEL, index, 0);
            }
        }
        if let Some(tab) = runtime.state.active() {
            set_text(runtime.controls.omnibox, &tab.url);
            EnableWindow(runtime.controls.back, tab.can_go_back as i32);
            EnableWindow(runtime.controls.forward, tab.can_go_forward as i32);
            EnableWindow(runtime.controls.stop, tab.loading as i32);
            EnableWindow(runtime.controls.reload, (!tab.loading) as i32);
            let (enabled, count) = runtime.shields().lock().map(|s| (s.enabled_for_url(&tab.url), s.blocked_count(tab.id.0))).unwrap_or((true, 0));
            set_text(runtime.controls.shields, &format!("Shields {} {count}", if enabled { "ON" } else { "OFF" }));
            let title = if tab.title.trim().is_empty() { "ClearLane".to_string() } else { format!("{} — ClearLane", tab.title) };
            SetWindowTextW(runtime.hwnd, wide(&title).as_ptr());
        } else {
            set_text(runtime.controls.omnibox, "");
            set_text(runtime.controls.shields, "Shields ON 0");
        }
    }
}

pub(crate) fn omnibox_text(hwnd: HWND) -> String {
    if hwnd.is_null() { return String::new(); }
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        let mut buffer = vec![0u16; len as usize + 1];
        GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        String::from_utf16_lossy(&buffer[..len as usize])
    }
}

unsafe fn set_text(hwnd: HWND, text: &str) { SetWindowTextW(hwnd, wide(text).as_ptr()); }

unsafe extern "system" fn omnibox_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, id: usize, ref_data: usize) -> LRESULT {
    if msg == WM_KEYDOWN && wparam == VK_RETURN as usize {
        PostMessageW(ref_data as HWND, WM_COMMAND, ID_OMNIBOX, 0);
        return 0;
    }
    DefSubclassProc(hwnd, msg, wparam, lparam)
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = &*(lparam as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Arc<Mutex<Runtime>>;
    let runtime = if ptr.is_null() { None } else { Some((*ptr).clone()) };

    match msg {
        WM_COMMAND => {
            if let Some(runtime) = runtime {
                let id = wparam & 0xffff;
                let notification = (wparam >> 16) & 0xffff;
                match id {
                    ID_SIDEBAR_TOGGLE => app::toggle_sidebar(&runtime),
                    ID_BACK => app::go_back(&runtime),
                    ID_FORWARD => app::go_forward(&runtime),
                    ID_RELOAD => app::reload(&runtime),
                    ID_STOP => app::stop(&runtime),
                    ID_OMNIBOX => app::navigate_from_omnibox(&runtime),
                    ID_SHIELDS => app::toggle_shields(&runtime),
                    ID_NEW_TAB => app::open_tab(&runtime, "https://www.google.com/".into()),
                    ID_CLOSE_TAB => app::close_active_tab(&runtime),
                    ID_TAB_LIST if notification == LBN_SELCHANGE as usize => {
                        if let Ok(locked) = runtime.lock() {
                            let index = SendMessageW(locked.controls.tab_list, LB_GETCURSEL, 0, 0);
                            let tab = if index >= 0 { locked.state.tabs().get(index as usize).map(|tab| tab.id) } else { None };
                            drop(locked);
                            if let Some(tab) = tab { app::activate_tab(&runtime, tab); }
                        }
                    }
                    _ => {}
                }
            }
            0
        }
        WM_SIZE | WM_DPICHANGED => {
            if let Some(runtime) = runtime { if let Ok(mut locked) = runtime.lock() { layout(&mut locked); } }
            0
        }
        WM_SETFOCUS => {
            if let Some(runtime) = runtime { if let Ok(locked) = runtime.lock() { if let Some(id) = locked.state.active_id() { locked.engine.focus(id); } } }
            0
        }
        WM_CLOSE => {
            if let Some(runtime) = runtime { app::begin_close(&runtime); }
            0
        }
        WM_APP_SHIELDS => {
            if let Some(runtime) = runtime { if let Ok(mut locked) = runtime.lock() { if locked.state.active_id() == Some(TabId(wparam as u64)) { locked.refresh_ui(); } } }
            0
        }
        WM_APP_NEW_TAB => {
            if let Some(runtime) = runtime { app::open_tab(&runtime, "https://www.google.com/".into()); }
            0
        }
        WM_DESTROY => { PostQuitMessage(0); 0 }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            if !ptr.is_null() { drop(Box::from_raw(ptr)); }
            DefWindowProcW(hwnd, msg, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn wide(value: &str) -> Vec<u16> { value.encode_utf16().chain(std::iter::once(0)).collect() }

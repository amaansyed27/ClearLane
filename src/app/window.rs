use std::{
    cell::Cell,
    ffi::c_void,
    sync::{Arc, Mutex},
};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
    Graphics::Gdi::{COLOR_WINDOW, DEFAULT_GUI_FONT, GetStockObject, UpdateWindow},
    System::LibraryLoader::GetModuleHandleW,
    UI::{
        HiDpi::GetDpiForWindow,
        Input::KeyboardAndMouse::VK_RETURN,
        Shell::{DefSubclassProc, SetWindowSubclass},
        WindowsAndMessaging::*,
    },
};

use crate::{
    app::{self, Runtime, WM_APP_NEW_TAB, WM_APP_SHIELDS},
    chromium::EngineBounds,
    core::TabId,
};

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

thread_local! {
    /// Native controls synchronously notify their parent while some properties are changed.
    /// `refresh` is normally called while the Runtime mutex is held, so handling those
    /// notifications as user input would recursively lock Runtime and deadlock the UI thread.
    static UI_UPDATE_DEPTH: Cell<u32> = const { Cell::new(0) };
}

struct UiUpdateGuard;

impl UiUpdateGuard {
    fn enter() -> Self {
        UI_UPDATE_DEPTH.with(|depth| depth.set(depth.get().saturating_add(1)));
        Self
    }
}

impl Drop for UiUpdateGuard {
    fn drop(&mut self) {
        UI_UPDATE_DEPTH.with(|depth| depth.set(depth.get().saturating_sub(1)));
    }
}

fn ui_update_in_progress() -> bool {
    UI_UPDATE_DEPTH.with(|depth| depth.get() != 0)
}

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
            sidebar_toggle: std::ptr::null_mut(),
            back: std::ptr::null_mut(),
            forward: std::ptr::null_mut(),
            reload: std::ptr::null_mut(),
            stop: std::ptr::null_mut(),
            omnibox: std::ptr::null_mut(),
            shields: std::ptr::null_mut(),
            sidebar: std::ptr::null_mut(),
            tab_list: std::ptr::null_mut(),
            new_tab: std::ptr::null_mut(),
            close_tab: std::ptr::null_mut(),
        }
    }
}

pub(crate) fn create(runtime: Arc<Mutex<Runtime>>) -> Result<(), String> {
    crate::win::startup_log("window create: entered");
    // SAFETY: all handles created here remain owned by the UI thread until WM_NCDESTROY.
    unsafe {
        // CEF establishes the process DPI mode during its own Windows startup. Do not
        // mutate process-wide DPI awareness here after CEF has already initialized.
        crate::win::startup_log("window create: obtaining module handle");
        let instance = GetModuleHandleW(std::ptr::null());
        crate::win::startup_log("window create: module handle obtained");

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
        crate::win::startup_log("window create: registering window class");
        RegisterClassW(&window_class);
        crate::win::startup_log("window create: window class registered");

        let raw_runtime = Box::into_raw(Box::new(runtime.clone()));
        let title = wide("ClearLane");
        crate::win::startup_log("window create: calling CreateWindowExW");
        let hwnd = CreateWindowExW(
            0,
            class.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1280,
            800,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            instance,
            raw_runtime.cast::<c_void>(),
        );
        crate::win::startup_log("window create: CreateWindowExW returned");
        if hwnd.is_null() {
            // Window creation did not establish a usable top-level HWND. In the normal
            // failure path Windows has not retained lpCreateParams, so release our Arc.
            drop(Box::from_raw(raw_runtime));
            return Err("CreateWindowExW failed".to_string());
        }

        crate::win::startup_log("window create: creating child controls");
        let controls = create_controls(hwnd, instance)?;
        crate::win::startup_log("window create: child controls created");
        {
            crate::win::startup_log("window create: locking runtime for initial layout");
            let mut locked = runtime
                .lock()
                .map_err(|_| "Runtime lock poisoned".to_string())?;
            locked.hwnd = hwnd;
            locked.controls = controls;
            crate::win::startup_log("window create: laying out controls");
            layout(&mut locked);
            crate::win::startup_log("window create: refreshing controls");
            refresh(&mut locked);
            crate::win::startup_log("window create: initial layout complete");
        }

        crate::win::startup_log("window create: showing window");
        ShowWindow(hwnd, SW_SHOW);
        crate::win::startup_log("window create: updating window");
        UpdateWindow(hwnd);
        crate::win::startup_log("window create: completed");
        Ok(())
    }
}

fn create_controls(hwnd: HWND, instance: *mut c_void) -> Result<Controls, String> {
    // SAFETY: child controls use `hwnd` as their lifetime-owning parent and valid static classes.
    unsafe {
        let button = wide("BUTTON");
        let edit = wide("EDIT");
        let listbox = wide("LISTBOX");

        let sidebar_toggle = create_control(
            hwnd,
            instance,
            &button,
            "☰",
            BS_PUSHBUTTON as u32,
            ID_SIDEBAR_TOGGLE,
        );
        let back = create_control(hwnd, instance, &button, "‹", BS_PUSHBUTTON as u32, ID_BACK);
        let forward = create_control(
            hwnd,
            instance,
            &button,
            "›",
            BS_PUSHBUTTON as u32,
            ID_FORWARD,
        );
        let reload = create_control(
            hwnd,
            instance,
            &button,
            "↻",
            BS_PUSHBUTTON as u32,
            ID_RELOAD,
        );
        let stop = create_control(hwnd, instance, &button, "×", BS_PUSHBUTTON as u32, ID_STOP);
        let omnibox = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            edit.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | ES_AUTOHSCROLL as u32,
            0,
            0,
            100,
            28,
            hwnd,
            ID_OMNIBOX as _,
            instance,
            std::ptr::null(),
        );
        let shields = create_control(
            hwnd,
            instance,
            &button,
            "Shields",
            BS_PUSHBUTTON as u32,
            ID_SHIELDS,
        );
        let sidebar = CreateWindowExW(
            0,
            wide("STATIC").as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE,
            0,
            0,
            200,
            400,
            hwnd,
            std::ptr::null_mut(),
            instance,
            std::ptr::null(),
        );
        let new_tab = create_control(
            hwnd,
            instance,
            &button,
            "+",
            BS_PUSHBUTTON as u32,
            ID_NEW_TAB,
        );
        let close_tab = create_control(
            hwnd,
            instance,
            &button,
            "−",
            BS_PUSHBUTTON as u32,
            ID_CLOSE_TAB,
        );
        let tab_list = CreateWindowExW(
            WS_EX_CLIENTEDGE,
            listbox.as_ptr(),
            std::ptr::null(),
            WS_CHILD | WS_VISIBLE | WS_VSCROLL | LBS_NOTIFY as u32,
            0,
            0,
            180,
            300,
            hwnd,
            ID_TAB_LIST as _,
            instance,
            std::ptr::null(),
        );

        let controls = Controls {
            sidebar_toggle,
            back,
            forward,
            reload,
            stop,
            omnibox,
            shields,
            sidebar,
            tab_list,
            new_tab,
            close_tab,
        };

        if [
            controls.sidebar_toggle,
            controls.back,
            controls.forward,
            controls.reload,
            controls.stop,
            controls.omnibox,
            controls.shields,
            controls.sidebar,
            controls.tab_list,
            controls.new_tab,
            controls.close_tab,
        ]
        .iter()
        .any(|handle| handle.is_null())
        {
            return Err("failed to create one or more native browser controls".into());
        }

        let font = GetStockObject(DEFAULT_GUI_FONT);
        for control in [
            controls.sidebar_toggle,
            controls.back,
            controls.forward,
            controls.reload,
            controls.stop,
            controls.omnibox,
            controls.shields,
            controls.sidebar,
            controls.tab_list,
            controls.new_tab,
            controls.close_tab,
        ] {
            SendMessageW(control, WM_SETFONT, font as usize, 1);
        }
        SetWindowSubclass(controls.omnibox, Some(omnibox_proc), 1, hwnd as usize);
        Ok(controls)
    }
}

fn create_control(
    hwnd: HWND,
    instance: *mut c_void,
    class: &[u16],
    label: &str,
    extra_style: u32,
    id: usize,
) -> HWND {
    let label = wide(label);
    // SAFETY: inputs are valid during the call and the parent owns the created child window.
    unsafe {
        CreateWindowExW(
            0,
            class.as_ptr(),
            label.as_ptr(),
            WS_CHILD | WS_VISIBLE | WS_TABSTOP | extra_style,
            0,
            0,
            30,
            30,
            hwnd,
            id as _,
            instance,
            std::ptr::null(),
        )
    }
}

pub(crate) fn browser_bounds(hwnd: HWND, sidebar_open: bool) -> EngineBounds {
    if hwnd.is_null() {
        return EngineBounds::default();
    }
    // SAFETY: hwnd is the live main window while Runtime exists.
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(hwnd, &mut rect);
        let dpi = GetDpiForWindow(hwnd).max(96) as i32;
        let toolbar = 48 * dpi / 96;
        let sidebar = if sidebar_open { 224 * dpi / 96 } else { 0 };
        EngineBounds {
            x: sidebar,
            y: toolbar,
            width: (rect.right - sidebar).max(1),
            height: (rect.bottom - toolbar).max(1),
        }
    }
}

fn layout(runtime: &mut Runtime) {
    if runtime.hwnd.is_null() || runtime.controls.omnibox.is_null() {
        return;
    }
    // SAFETY: all controls are children of the live main HWND and calls occur on the CEF UI thread.
    unsafe {
        let mut rect = RECT::default();
        GetClientRect(runtime.hwnd, &mut rect);
        let dpi = GetDpiForWindow(runtime.hwnd).max(96) as i32;
        let s = |value: i32| value * dpi / 96;
        let toolbar_height = s(48);
        let button = s(36);
        let gap = s(6);
        let sidebar_width = if runtime.sidebar_open { s(224) } else { 0 };
        let shields_width = s(116);
        let mut x = gap;

        for control in [
            runtime.controls.sidebar_toggle,
            runtime.controls.back,
            runtime.controls.forward,
            runtime.controls.reload,
            runtime.controls.stop,
        ] {
            MoveWindow(control, x, s(6), button, s(34), 1);
            x += button + gap;
        }

        let omnibox_width = (rect.right - x - shields_width - gap * 2).max(s(140));
        MoveWindow(runtime.controls.omnibox, x, s(8), omnibox_width, s(30), 1);
        MoveWindow(
            runtime.controls.shields,
            x + omnibox_width + gap,
            s(6),
            shields_width,
            s(34),
            1,
        );

        if runtime.sidebar_open {
            for control in [
                runtime.controls.sidebar,
                runtime.controls.new_tab,
                runtime.controls.close_tab,
                runtime.controls.tab_list,
            ] {
                ShowWindow(control, SW_SHOW);
            }
            MoveWindow(
                runtime.controls.sidebar,
                0,
                toolbar_height,
                sidebar_width,
                rect.bottom - toolbar_height,
                1,
            );
            MoveWindow(
                runtime.controls.new_tab,
                gap,
                toolbar_height + gap,
                s(36),
                s(30),
                1,
            );
            MoveWindow(
                runtime.controls.close_tab,
                gap + s(42),
                toolbar_height + gap,
                s(36),
                s(30),
                1,
            );
            MoveWindow(
                runtime.controls.tab_list,
                gap,
                toolbar_height + s(44),
                sidebar_width - gap * 2,
                rect.bottom - toolbar_height - s(50),
                1,
            );
        } else {
            for control in [
                runtime.controls.sidebar,
                runtime.controls.new_tab,
                runtime.controls.close_tab,
                runtime.controls.tab_list,
            ] {
                ShowWindow(control, SW_HIDE);
            }
        }

        let bounds = browser_bounds(runtime.hwnd, runtime.sidebar_open);
        runtime.engine.layout(bounds, runtime.state.active_id());
    }
}

pub(crate) fn refresh(runtime: &mut Runtime) {
    if runtime.controls.omnibox.is_null() {
        return;
    }

    // Win32 edit/list controls can synchronously send WM_COMMAND while they are updated.
    // Mark this scope so the parent procedure cannot mistake those notifications for user input
    // and recursively acquire the already-held Runtime mutex.
    let _ui_update = UiUpdateGuard::enter();

    // SAFETY: all controls are valid child HWNDs while the main window is alive.
    unsafe {
        SendMessageW(runtime.controls.tab_list, LB_RESETCONTENT, 0, 0);
        for tab in runtime.state.tabs() {
            let label = if tab.title.trim().is_empty() {
                &tab.url
            } else {
                &tab.title
            };
            let text = wide(label);
            SendMessageW(
                runtime.controls.tab_list,
                LB_ADDSTRING,
                0,
                text.as_ptr() as isize,
            );
        }
        if let Some(active) = runtime.state.active_id() {
            if let Some(index) = runtime.state.tabs().iter().position(|tab| tab.id == active) {
                SendMessageW(runtime.controls.tab_list, LB_SETCURSEL, index, 0);
            }
        }

        if let Some(tab) = runtime.state.active() {
            set_text(runtime.controls.omnibox, &tab.url);
            let (enabled, count) = runtime
                .shields()
                .lock()
                .map(|shields| {
                    (
                        shields.enabled_for_url(&tab.url),
                        shields.blocked_count(tab.id.0),
                    )
                })
                .unwrap_or((true, 0));
            set_text(
                runtime.controls.shields,
                &format!("Shields {} {count}", if enabled { "ON" } else { "OFF" }),
            );
            let title = if tab.title.trim().is_empty() {
                "ClearLane".to_string()
            } else {
                format!("{} — ClearLane", tab.title)
            };
            SetWindowTextW(runtime.hwnd, wide(&title).as_ptr());
        } else {
            set_text(runtime.controls.omnibox, "");
            set_text(runtime.controls.shields, "Shields ON 0");
        }
    }
}

pub(crate) fn omnibox_text(hwnd: HWND) -> String {
    if hwnd.is_null() {
        return String::new();
    }
    // SAFETY: hwnd is the native EDIT control owned by the main window.
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        let mut buffer = vec![0u16; len as usize + 1];
        GetWindowTextW(hwnd, buffer.as_mut_ptr(), buffer.len() as i32);
        String::from_utf16_lossy(&buffer[..len as usize])
    }
}

fn set_text(hwnd: HWND, text: &str) {
    // SAFETY: the UTF-16 buffer is NUL-terminated and lives through the synchronous call.
    unsafe {
        SetWindowTextW(hwnd, wide(text).as_ptr());
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CommandAction {
    ToggleSidebar,
    Back,
    Forward,
    Reload,
    Stop,
    SubmitOmnibox,
    ToggleShields,
    NewTab,
    CloseTab,
    SelectTab,
}

fn classify_command(id: usize, notification: usize, lparam: LPARAM) -> Option<CommandAction> {
    let clicked = notification == BN_CLICKED as usize;
    match id {
        ID_SIDEBAR_TOGGLE if clicked => Some(CommandAction::ToggleSidebar),
        ID_BACK if clicked => Some(CommandAction::Back),
        ID_FORWARD if clicked => Some(CommandAction::Forward),
        ID_RELOAD if clicked => Some(CommandAction::Reload),
        ID_STOP if clicked => Some(CommandAction::Stop),
        // The omnibox subclass posts this synthetic command with lParam=0 when Enter is pressed.
        // Native EDIT notifications such as EN_UPDATE/EN_CHANGE include the control HWND and must
        // never be interpreted as navigation requests.
        ID_OMNIBOX if notification == 0 && lparam == 0 => Some(CommandAction::SubmitOmnibox),
        ID_SHIELDS if clicked => Some(CommandAction::ToggleShields),
        ID_NEW_TAB if clicked => Some(CommandAction::NewTab),
        ID_CLOSE_TAB if clicked => Some(CommandAction::CloseTab),
        ID_TAB_LIST if notification == LBN_SELCHANGE as usize => Some(CommandAction::SelectTab),
        _ => None,
    }
}

unsafe extern "system" fn omnibox_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
    _id: usize,
    ref_data: usize,
) -> LRESULT {
    // SAFETY: this is installed by SetWindowSubclass and forwards all unhandled messages.
    unsafe {
        if msg == WM_KEYDOWN && wparam == VK_RETURN as usize {
            PostMessageW(ref_data as HWND, WM_COMMAND, ID_OMNIBOX, 0);
            return 0;
        }
        DefSubclassProc(hwnd, msg, wparam, lparam)
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    // SAFETY: Windows invokes this callback with Win32 message-contract arguments.
    unsafe {
        if msg == WM_NCCREATE {
            crate::win::startup_log("window proc: WM_NCCREATE");
            let create = &*(lparam as *const CREATESTRUCTW);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        }
        if msg == WM_CREATE {
            crate::win::startup_log("window proc: WM_CREATE");
        }
        if msg == WM_SIZE {
            crate::win::startup_log("window proc: WM_SIZE");
        }
        if msg == WM_SHOWWINDOW {
            crate::win::startup_log("window proc: WM_SHOWWINDOW");
        }

        let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut Arc<Mutex<Runtime>>;
        let runtime = if ptr.is_null() {
            None
        } else {
            Some((*ptr).clone())
        };

        match msg {
            WM_COMMAND => {
                // Ignore synchronous notifications generated by our own control refreshes.
                if ui_update_in_progress() {
                    return 0;
                }

                let id = wparam & 0xffff;
                let notification = (wparam >> 16) & 0xffff;
                let Some(action) = classify_command(id, notification, lparam) else {
                    return 0;
                };
                let Some(runtime) = runtime else {
                    return 0;
                };

                match action {
                    CommandAction::ToggleSidebar => app::toggle_sidebar(&runtime),
                    CommandAction::Back => app::go_back(&runtime),
                    CommandAction::Forward => app::go_forward(&runtime),
                    CommandAction::Reload => app::reload(&runtime),
                    CommandAction::Stop => app::stop(&runtime),
                    CommandAction::SubmitOmnibox => app::navigate_from_omnibox(&runtime),
                    CommandAction::ToggleShields => app::toggle_shields(&runtime),
                    CommandAction::NewTab => {
                        app::open_tab(&runtime, "https://www.google.com/".into());
                    }
                    CommandAction::CloseTab => app::close_active_tab(&runtime),
                    CommandAction::SelectTab => {
                        if let Ok(locked) = runtime.lock() {
                            let index = SendMessageW(locked.controls.tab_list, LB_GETCURSEL, 0, 0);
                            let tab = if index >= 0 {
                                locked.state.tabs().get(index as usize).map(|tab| tab.id)
                            } else {
                                None
                            };
                            drop(locked);
                            if let Some(tab) = tab {
                                app::activate_tab(&runtime, tab);
                            }
                        }
                    }
                }
                0
            }
            WM_SIZE | WM_DPICHANGED => {
                if let Some(runtime) = runtime {
                    if let Ok(mut locked) = runtime.lock() {
                        layout(&mut locked);
                    }
                }
                0
            }
            WM_SETFOCUS => {
                if let Some(runtime) = runtime {
                    if let Ok(locked) = runtime.lock() {
                        if let Some(id) = locked.state.active_id() {
                            locked.engine.focus(id);
                        }
                    }
                }
                0
            }
            WM_CLOSE => {
                if let Some(runtime) = runtime {
                    app::begin_close(&runtime);
                }
                0
            }
            WM_APP_SHIELDS => {
                if let Some(runtime) = runtime {
                    if let Ok(mut locked) = runtime.lock() {
                        if locked.state.active_id() == Some(TabId(wparam as u64)) {
                            locked.refresh_ui();
                        }
                    }
                }
                0
            }
            WM_APP_NEW_TAB => {
                if let Some(runtime) = runtime {
                    app::open_tab(&runtime, "https://www.google.com/".into());
                }
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            WM_NCDESTROY => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                if !ptr.is_null() {
                    drop(Box::from_raw(ptr));
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn programmatic_omnibox_notifications_do_not_submit_navigation() {
        assert_eq!(classify_command(ID_OMNIBOX, EN_CHANGE as usize, 1), None);
        assert_eq!(classify_command(ID_OMNIBOX, EN_UPDATE as usize, 1), None);
        assert_eq!(
            classify_command(ID_OMNIBOX, 0, 0),
            Some(CommandAction::SubmitOmnibox)
        );
    }

    #[test]
    fn controls_require_their_expected_notification_codes() {
        assert_eq!(
            classify_command(ID_BACK, BN_CLICKED as usize, 1),
            Some(CommandAction::Back)
        );
        assert_eq!(classify_command(ID_BACK, EN_CHANGE as usize, 1), None);
        assert_eq!(
            classify_command(ID_TAB_LIST, LBN_SELCHANGE as usize, 1),
            Some(CommandAction::SelectTab)
        );
    }
}

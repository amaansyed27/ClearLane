use std::sync::{Arc, Mutex, Weak};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use cef::*;

use crate::{
    app::{self, Runtime},
    core::TabId,
    shields::Shields,
};

#[derive(Clone)]
pub(crate) struct ClientContext {
    pub tab_id: TabId,
    pub runtime: Weak<Mutex<Runtime>>,
    pub shields: Arc<Mutex<Shields>>,
    pub current_url: Arc<Mutex<String>>,
}

wrap_client! {
    pub(crate) struct BrowserClient { context: ClientContext }
    impl Client {
        fn display_handler(&self) -> Option<DisplayHandler> { Some(ClearLaneDisplayHandler::new(self.context.clone())) }
        fn life_span_handler(&self) -> Option<LifeSpanHandler> { Some(ClearLaneLifeSpanHandler::new(self.context.clone())) }
        fn load_handler(&self) -> Option<LoadHandler> { Some(ClearLaneLoadHandler::new(self.context.clone())) }
        fn request_handler(&self) -> Option<RequestHandler> { Some(ClearLaneRequestHandler::new(self.context.clone())) }
    }
}

wrap_display_handler! {
    struct ClearLaneDisplayHandler { context: ClientContext }
    impl DisplayHandler {
        fn on_address_change(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, url: Option<&CefString>) {
            if frame.is_some_and(|frame| frame.is_main() == 0) { return; }
            let url = url.map(CefString::to_string).unwrap_or_default();
            if let Ok(mut current) = self.context.current_url.lock() { *current = url.clone(); }
            if let Some(runtime) = app::runtime_from_weak(&self.context.runtime) {
                if let Ok(mut runtime) = runtime.lock() { runtime.on_address(self.context.tab_id, url); }
            }
        }
        fn on_title_change(&self, _browser: Option<&mut Browser>, title: Option<&CefString>) {
            let title = title.map(CefString::to_string).unwrap_or_default();
            if let Some(runtime) = app::runtime_from_weak(&self.context.runtime) {
                if let Ok(mut runtime) = runtime.lock() { runtime.on_title(self.context.tab_id, title); }
            }
        }
    }
}

wrap_life_span_handler! {
    struct ClearLaneLifeSpanHandler { context: ClientContext }
    impl LifeSpanHandler {
        fn on_after_created(&self, browser: Option<&mut Browser>) {
            let Some(browser) = browser.cloned() else { return; };
            if let Some(runtime) = app::runtime_from_weak(&self.context.runtime) {
                if let Ok(mut runtime) = runtime.lock() {
                    runtime.engine.attach(self.context.tab_id, browser);
                    runtime.on_browser_created(self.context.tab_id);
                }
            }
        }
        fn do_close(&self, _browser: Option<&mut Browser>) -> i32 { 0 }
        fn on_before_close(&self, _browser: Option<&mut Browser>) {
            if let Some(runtime) = app::runtime_from_weak(&self.context.runtime) {
                if let Ok(mut runtime) = runtime.lock() {
                    runtime.engine.detach(self.context.tab_id);
                    runtime.on_browser_closed(self.context.tab_id);
                }
            }
        }
    }
}

wrap_load_handler! {
    struct ClearLaneLoadHandler { context: ClientContext }
    impl LoadHandler {
        fn on_loading_state_change(&self, _browser: Option<&mut Browser>, is_loading: i32, can_go_back: i32, can_go_forward: i32) {
            if let Some(runtime) = app::runtime_from_weak(&self.context.runtime) {
                if let Ok(mut runtime) = runtime.lock() {
                    runtime.on_loading(self.context.tab_id, is_loading != 0, can_go_back != 0, can_go_forward != 0);
                }
            }
        }
        fn on_load_error(&self, _browser: Option<&mut Browser>, frame: Option<&mut Frame>, error_code: Errorcode, error_text: Option<&CefString>, failed_url: Option<&CefString>) {
            let raw_code = sys::cef_errorcode_t::from(error_code);
            if raw_code == sys::cef_errorcode_t::ERR_ABORTED { return; }
            let Some(frame) = frame else { return; };
            if frame.is_main() == 0 { return; }
            let text = error_text.map(CefString::to_string).unwrap_or_else(|| "Unknown error".into());
            let url = failed_url.map(CefString::to_string).unwrap_or_default();
            load_internal_page(frame, "Page could not be loaded", &format!("{} ({})", html_escape(&text), raw_code as i32), &url);
        }
    }
}

wrap_request_handler! {
    struct ClearLaneRequestHandler { context: ClientContext }
    impl RequestHandler {
        fn resource_request_handler(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            _request: Option<&mut Request>,
            _is_navigation: i32,
            _is_download: i32,
            request_initiator: Option<&CefString>,
            _disable_default_handling: Option<&mut i32>,
        ) -> Option<ResourceRequestHandler> {
            let initiator = request_initiator.map(CefString::to_string).filter(|value| !value.is_empty())
                .or_else(|| self.context.current_url.lock().ok().map(|value| value.clone()))
                .unwrap_or_default();
            Some(ClearLaneResourceRequestHandler::new(self.context.clone(), initiator))
        }

        fn on_render_process_terminated(&self, browser: Option<&mut Browser>, _status: TerminationStatus, error_code: i32, error_string: Option<&CefString>) {
            let detail = error_string.map(CefString::to_string).unwrap_or_else(|| format!("Renderer exited with code {error_code}"));
            if let Some(frame) = browser.and_then(|browser| browser.main_frame()) {
                load_internal_page(&frame, "This tab crashed", &html_escape(&detail), "Reload the page to try again.");
            }
        }
    }
}

wrap_resource_request_handler! {
    struct ClearLaneResourceRequestHandler { context: ClientContext, initiator: String }
    impl ResourceRequestHandler {
        fn on_before_resource_load(
            &self,
            _browser: Option<&mut Browser>,
            _frame: Option<&mut Frame>,
            request: Option<&mut Request>,
            _callback: Option<&mut Callback>,
        ) -> ReturnValue {
            let Some(request) = request else { return ReturnValue::CONTINUE; };
            let request_url = CefStringUtf16::from(&request.url()).to_string();
            let blocked = self.context.shields.lock().map(|mut shields| {
                shields.should_block(self.context.tab_id.0, &self.initiator, &request_url)
            }).unwrap_or(false);
            if blocked {
                if let Some(runtime) = app::runtime_from_weak(&self.context.runtime) {
                    if let Ok(runtime) = runtime.lock() { runtime.notify_shields(self.context.tab_id); }
                }
                ReturnValue::CANCEL
            } else {
                ReturnValue::CONTINUE
            }
        }

        fn on_protocol_execution(&self, _browser: Option<&mut Browser>, _frame: Option<&mut Frame>, _request: Option<&mut Request>, allow_os_execution: Option<&mut i32>) {
            if let Some(allow) = allow_os_execution { *allow = 0; }
        }
    }
}

fn load_internal_page(frame: &Frame, heading: &str, detail: &str, footer: &str) {
    let html = format!(
        "<!doctype html><meta charset=utf-8><title>{0}</title><style>body{{font-family:system-ui;margin:10vh auto;max-width:720px;padding:0 32px;color:#202124}}h1{{font-size:28px}}p{{line-height:1.55;color:#5f6368}}</style><h1>{0}</h1><p>{1}</p><p>{2}</p>",
        html_escape(heading),
        detail,
        html_escape(footer)
    );
    let data = format!("data:text/html;base64,{}", STANDARD.encode(html));
    frame.load_url(Some(&CefString::from(data.as_str())));
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

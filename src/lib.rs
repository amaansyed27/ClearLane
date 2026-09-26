pub mod core;
pub mod shields;

#[cfg(target_os = "windows")]
// Win32 and CEF callbacks are intentionally written as explicit nested guards: each
// guard documents a separate lifetime/lock boundary on the single CEF UI thread.
// Runtime contains CEF/Win32 handles that must stay UI-thread-bound, so its Arc is
// for callback ownership/lifetime rather than cross-thread sharing.
#[allow(clippy::arc_with_non_send_sync, clippy::collapsible_if)]
pub(crate) mod app;
#[cfg(target_os = "windows")]
// Keep callback guard boundaries explicit at the CEF FFI edge.
#[allow(clippy::collapsible_if)]
pub(crate) mod chromium;
#[cfg(target_os = "windows")]
mod win;

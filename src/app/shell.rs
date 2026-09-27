use eframe::egui::{
    self, Align, Button, CentralPanel, FontId, Frame, Id, Key, Layout, Margin, Panel, RichText,
    Stroke, TextEdit, Vec2,
};

use crate::core::{BrowserState, TabId, normalize_omnibox};

use super::theme::{
    ACCENT, BORDER, FAINT, MUTED, OMNIBOX_BG, PAGE_BG, SAFE, SHELL_BG, SIDEBAR_BG, TEXT,
};

pub struct ClearLaneShell {
    browser: BrowserState,
    omnibox: String,
    omnibox_id: Id,
    sidebar_expanded: bool,
}

impl ClearLaneShell {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        super::theme::apply(&cc.egui_ctx);

        let mut browser = BrowserState::default();
        let home = browser.open_tab("https://clearlane.app/");
        browser.update_title(home, "ClearLane");
        browser.set_blocked_count(home, 7);

        let rust = browser.open_tab("https://www.rust-lang.org/");
        browser.update_title(rust, "Rust");
        browser.set_blocked_count(rust, 3);

        let mdn = browser.open_tab("https://developer.mozilla.org/");
        browser.update_title(mdn, "MDN Web Docs");
        browser.set_blocked_count(mdn, 18);

        let notes = browser.open_tab("https://example.com/browser-ui-notes");
        browser.update_title(notes, "Browser UI notes");
        browser.set_blocked_count(notes, 2);

        browser.activate(home);

        Self {
            browser,
            omnibox: "https://clearlane.app/".into(),
            omnibox_id: Id::new("clearlane-omnibox"),
            sidebar_expanded: true,
        }
    }

    fn sync_omnibox(&mut self) {
        self.omnibox = self
            .browser
            .active()
            .map(|tab| tab.url.clone())
            .unwrap_or_default();
    }

    fn activate_tab(&mut self, id: TabId) {
        if self.browser.activate(id) {
            self.sync_omnibox();
        }
    }

    fn new_tab(&mut self, ctx: &egui::Context) {
        self.browser.open_tab("clearlane://newtab");
        self.sync_omnibox();
        ctx.memory_mut(|memory| memory.request_focus(self.omnibox_id));
    }

    fn close_active_tab(&mut self) {
        if let Some(id) = self.browser.active_id() {
            self.browser.close_tab(id);
        }
        if self.browser.active_id().is_none() {
            self.browser.open_tab("clearlane://newtab");
        }
        self.sync_omnibox();
    }

    fn navigate_from_omnibox(&mut self) {
        let Some(url) = normalize_omnibox(&self.omnibox) else {
            return;
        };
        if let Some(id) = self.browser.active_id() {
            self.browser.navigate(id, url);
            self.sync_omnibox();
        }
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let focus_omnibox = ctx.input(|input| input.modifiers.command && input.key_pressed(Key::L));
        let new_tab = ctx.input(|input| input.modifiers.command && input.key_pressed(Key::T));
        let close_tab = ctx.input(|input| input.modifiers.command && input.key_pressed(Key::W));
        let back = ctx.input(|input| input.modifiers.alt && input.key_pressed(Key::ArrowLeft));
        let forward = ctx.input(|input| input.modifiers.alt && input.key_pressed(Key::ArrowRight));

        if focus_omnibox {
            ctx.memory_mut(|memory| memory.request_focus(self.omnibox_id));
        }
        if new_tab {
            self.new_tab(ctx);
        }
        if close_tab {
            self.close_active_tab();
        }
        if back
            && let Some(id) = self.browser.active_id()
            && self.browser.go_back(id)
        {
            self.sync_omnibox();
        }
        if forward
            && let Some(id) = self.browser.active_id()
            && self.browser.go_forward(id)
        {
            self.sync_omnibox();
        }
    }

    fn show_sidebar(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        let width = if self.sidebar_expanded { 230.0 } else { 62.0 };
        Panel::left("clearlane-sidebar")
            .default_size(width)
            .min_size(width)
            .max_size(width)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(SIDEBAR_BG)
                    .inner_margin(Margin::symmetric(8, 8)),
            )
            .show(root, |ui| {
                ui.set_min_width(width - 16.0);
                ui.horizontal(|ui| {
                    let mark = RichText::new("C")
                        .size(14.0)
                        .strong()
                        .color(egui::Color32::WHITE);
                    let mark_button = Button::new(mark)
                        .fill(TEXT)
                        .stroke(Stroke::NONE)
                        .corner_radius(8)
                        .min_size(Vec2::splat(30.0));
                    let _ = ui.add_enabled(false, mark_button);

                    if self.sidebar_expanded {
                        ui.label(RichText::new("ClearLane").size(15.0).strong().color(TEXT));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if chrome_button(ui, "‹", "Collapse sidebar", true).clicked() {
                                self.sidebar_expanded = false;
                            }
                        });
                    } else if chrome_button(ui, "›", "Expand sidebar", true).clicked() {
                        self.sidebar_expanded = true;
                    }
                });

                ui.add_space(18.0);
                if self.sidebar_expanded {
                    ui.label(RichText::new("BROWSING").size(10.0).strong().color(FAINT));
                    ui.add_space(4.0);
                }

                let tabs = self.browser.tabs().to_vec();
                let active = self.browser.active_id();
                for tab in tabs {
                    let selected = active == Some(tab.id);
                    let label = if self.sidebar_expanded {
                        tab.title.clone()
                    } else {
                        tab.title
                            .chars()
                            .next()
                            .map(|ch| ch.to_uppercase().collect::<String>())
                            .unwrap_or_else(|| "·".into())
                    };
                    let min_width = if self.sidebar_expanded {
                        ui.available_width()
                    } else {
                        38.0
                    };
                    let response = ui.add(
                        Button::selectable(
                            selected,
                            RichText::new(label)
                                .size(if self.sidebar_expanded { 13.5 } else { 12.5 })
                                .color(if selected { TEXT } else { MUTED }),
                        )
                        .min_size(Vec2::new(min_width, 36.0))
                        .corner_radius(7)
                        .truncate(),
                    );
                    if response.clicked() {
                        self.activate_tab(tab.id);
                    }
                    if !self.sidebar_expanded {
                        response.on_hover_text(tab.title);
                    }
                }

                ui.add_space(6.0);
                let new_tab_label = if self.sidebar_expanded {
                    "+   New tab"
                } else {
                    "+"
                };
                let new_tab_width = ui.available_width();
                if ui
                    .add(
                        Button::new(RichText::new(new_tab_label).size(13.5).color(MUTED))
                            .min_size(Vec2::new(new_tab_width, 34.0))
                            .corner_radius(7)
                            .frame_when_inactive(false),
                    )
                    .on_hover_text("New tab  Ctrl+T")
                    .clicked()
                {
                    self.new_tab(&ctx);
                }

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    if self.sidebar_expanded {
                        ui.label(
                            RichText::new("Gate A · local shell")
                                .size(10.5)
                                .color(FAINT),
                        );
                    }
                });
            });
    }

    fn show_toolbar(&mut self, root: &mut egui::Ui) {
        let ctx = root.ctx().clone();
        Panel::top("clearlane-toolbar")
            .default_size(52.0)
            .min_size(52.0)
            .max_size(52.0)
            .resizable(false)
            .show_separator_line(false)
            .frame(
                Frame::new()
                    .fill(SHELL_BG)
                    .inner_margin(Margin::symmetric(9, 8)),
            )
            .show(root, |ui| {
                let active = self.browser.active().cloned();
                let can_back = active.as_ref().is_some_and(|tab| tab.can_go_back());
                let can_forward = active.as_ref().is_some_and(|tab| tab.can_go_forward());
                let loading = active.as_ref().is_some_and(|tab| tab.loading);

                ui.horizontal(|ui| {
                    if chrome_button(ui, "‹", "Back  Alt+Left", can_back).clicked()
                        && let Some(id) = self.browser.active_id()
                        && self.browser.go_back(id)
                    {
                        self.sync_omnibox();
                    }
                    if chrome_button(ui, "›", "Forward  Alt+Right", can_forward).clicked()
                        && let Some(id) = self.browser.active_id()
                        && self.browser.go_forward(id)
                    {
                        self.sync_omnibox();
                    }

                    if loading {
                        if chrome_button(ui, "×", "Stop", true).clicked()
                            && let Some(id) = self.browser.active_id()
                        {
                            self.browser.stop(id);
                        }
                    } else if chrome_button(ui, "↻", "Reload", true).clicked()
                        && let Some(id) = self.browser.active_id()
                    {
                        self.browser.reload(id);
                    }

                    ui.add_space(2.0);
                    let omnibox_width = (ui.available_width() - 88.0).max(220.0);
                    let focused = ctx.memory(|memory| memory.has_focus(self.omnibox_id));
                    let omnibox_frame =
                        Frame::new()
                            .fill(OMNIBOX_BG)
                            .corner_radius(9)
                            .stroke(Stroke::new(
                                if focused { 1.25 } else { 1.0 },
                                if focused { ACCENT } else { BORDER },
                            ));
                    let response = ui.add_sized(
                        [omnibox_width, 35.0],
                        TextEdit::singleline(&mut self.omnibox)
                            .id(self.omnibox_id)
                            .font(FontId::proportional(13.5))
                            .hint_text("Search or enter address")
                            .desired_width(f32::INFINITY)
                            .margin(Margin::symmetric(12, 7))
                            .frame(omnibox_frame),
                    );
                    if response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter)) {
                        self.navigate_from_omnibox();
                    }

                    if let Some(tab) = active {
                        let shield_text = if tab.shields_enabled {
                            format!("◆ {}", tab.blocked_count)
                        } else {
                            "◆ Off".into()
                        };
                        let shield_color = if tab.shields_enabled { SAFE } else { MUTED };
                        if ui
                            .add(
                                Button::new(
                                    RichText::new(shield_text).size(12.5).color(shield_color),
                                )
                                .min_size(Vec2::new(76.0, 32.0))
                                .corner_radius(7)
                                .frame_when_inactive(false),
                            )
                            .on_hover_text("Shields protection")
                            .clicked()
                            && let Some(id) = self.browser.active_id()
                        {
                            self.browser.toggle_shields(id);
                        }
                    }
                });
            });
    }

    fn show_page_surface(&self, root: &mut egui::Ui) {
        CentralPanel::default()
            .frame(Frame::new().fill(PAGE_BG).inner_margin(0))
            .show(root, |ui| {
                let rect = ui.max_rect();
                ui.painter().line_segment(
                    [rect.left_top(), rect.right_top()],
                    Stroke::new(1.0, BORDER),
                );

                let Some(tab) = self.browser.active() else {
                    return;
                };

                ui.add_space(78.0);
                ui.horizontal(|ui| {
                    let margin = ((ui.available_width() - 760.0) * 0.12).clamp(42.0, 96.0);
                    ui.add_space(margin);
                    ui.vertical(|ui| {
                        if tab.url == "clearlane://newtab" {
                            ui.add_space(34.0);
                            ui.label(RichText::new("ClearLane").size(34.0).strong().color(TEXT));
                            ui.add_space(9.0);
                            ui.label(
                                RichText::new("A quiet place to start.")
                                    .size(15.0)
                                    .color(MUTED),
                            );
                            ui.add_space(28.0);
                            ui.label(
                                RichText::new("Use the address bar above to search or enter a URL.")
                                    .size(13.0)
                                    .color(FAINT),
                            );
                        } else {
                            ui.label(RichText::new(&tab.title).size(30.0).strong().color(TEXT));
                            ui.add_space(6.0);
                            ui.label(RichText::new(&tab.url).size(12.5).color(MUTED));
                            ui.add_space(34.0);
                            ui.separator();
                            ui.add_space(28.0);
                            ui.label(
                                RichText::new("The webpage stays visually primary.")
                                    .size(17.0)
                                    .color(TEXT),
                            );
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new(
                                    "This local surface exists only to judge ClearLane's browser chrome before Chromium is attached.",
                                )
                                .size(13.5)
                                .color(MUTED),
                            );
                            ui.add_space(22.0);
                            ui.label(
                                RichText::new(
                                    "Resize the window, switch tabs, collapse the sidebar, focus the omnibox, and exercise the navigation states.",
                                )
                                .size(13.0)
                                .color(FAINT),
                            );
                        }
                    });
                });
            });
    }
}

impl eframe::App for ClearLaneShell {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_shortcuts(&ctx);
        self.show_sidebar(ui);
        self.show_toolbar(ui);
        self.show_page_surface(ui);
    }
}

fn chrome_button(ui: &mut egui::Ui, glyph: &str, hover: &str, enabled: bool) -> egui::Response {
    ui.add_enabled(
        enabled,
        Button::new(
            RichText::new(glyph)
                .size(19.0)
                .color(if enabled { TEXT } else { FAINT }),
        )
        .min_size(Vec2::splat(32.0))
        .corner_radius(7)
        .frame_when_inactive(false),
    )
    .on_hover_text(hover)
}

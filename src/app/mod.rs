mod shell;
mod theme;

use eframe::egui;

pub fn run() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([860.0, 560.0]),
        ..Default::default()
    };

    eframe::run_native(
        "ClearLane — Gate A",
        options,
        Box::new(|cc| Ok(Box::new(shell::ClearLaneShell::new(cc)))),
    )
}

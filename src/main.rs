#![cfg_attr(windows, windows_subsystem = "windows")]
mod converters;
mod engine;
mod graphs;
mod precise;
mod ui;
fn main() -> eframe::Result {
    let compact = std::env::args().any(|s| s == "--compact");
    let options = eframe::NativeOptions {
        persist_window: !std::env::args().any(|s| s == "--preview"),
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size(if compact {
                [400.0, 660.0]
            } else {
                [960.0, 790.0]
            })
            .with_min_inner_size([380.0, 590.0])
            .with_title("Calcolatrice Rust"),
        ..Default::default()
    };
    eframe::run_native(
        "Calcolatrice Rust",
        options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc)))),
    )
}

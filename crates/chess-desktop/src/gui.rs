//! GUI binary entry point

use chess_desktop::ChessApp;
use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Chess Engine - Rust Implementation")
            .with_inner_size([900.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .unwrap_or_default(),
            ),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Chess Engine",
        options,
        Box::new(|cc| Ok(Box::new(ChessApp::new(cc)))),
    )
}

//! GUI binary entry point

use chess_desktop::{ChessApp, Settings};
use eframe::egui;

fn main() -> eframe::Result {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Chess Engine - Rust Implementation")
            .with_inner_size(Settings::load().window_size)
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .unwrap_or_default(),
            ),
        centered: true,
        ..Default::default()
    };

    // A PGN file named on the command line is the game to open.
    let game = std::env::args().nth(1);
    eframe::run_native(
        "Chess Engine",
        options,
        Box::new(move |cc| {
            let mut app = ChessApp::new(cc);
            if let Some(path) = game {
                app.open_pgn_file(std::path::Path::new(&path));
            }
            Ok(Box::new(app))
        }),
    )
}

//! GUI binary entry point

use chess_desktop::ChessApp;
use eframe::egui;

fn main() -> eframe::Result {
    #[cfg(not(target_arch = "wasm32"))]
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Chess Engine - Rust Implementation")
            .with_inner_size([900.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon.png")[..])
                    .unwrap_or_default()
            ),
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "Chess Engine",
        options,
        Box::new(|cc| {
            configure_fonts(&cc.egui_ctx);
            configure_visuals(&cc.egui_ctx);

            Ok(Box::new(ChessApp::new(cc)))
        }),
    )
}

fn configure_fonts(ctx: &egui::Context) {
    let fonts = egui::FontDefinitions::default();
    ctx.set_fonts(fonts);
}

fn configure_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.override_text_color = Some(egui::Color32::from_gray(200));
    visuals.panel_fill = egui::Color32::from_rgb(30, 30, 35);
    visuals.window_fill = egui::Color32::from_rgb(25, 25, 30);
    ctx.set_visuals(visuals);
}
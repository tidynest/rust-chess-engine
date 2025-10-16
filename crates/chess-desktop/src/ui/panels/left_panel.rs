//! Left panel showing position information.
//!
//! Displays captured pieces and selected square details.

use eframe::egui::{self, Context};
use chess_core::GameState;

use crate::app::state::ChessApp;
use crate::ui::components::material;

/// Draw the left panel
pub fn draw(app: &ChessApp, ctx: &Context) {
    egui::SidePanel::left("left_panel")
        .default_width(200.0)
        .width_range(150.0..=300.0)
        .resizable(true)
        .show(ctx, |ui| {
            // 8pt spacing system
            ui.add_space(8.0);

            ui.heading("Position Info");
            ui.add_space(8.0);

            ui.separator();
            ui.add_space(16.0);

            material::draw_material_count(app, ui);

            ui.add_space(16.0);
            ui.separator();
            ui.add_space(16.0);

            if let Some(square) = app.selected_square {
                // Selected square section with better hierarchy
                ui.label(egui::RichText::new("Selected Square").strong().size(16.0));
                ui.add_space(8.0);

                ui.label(format!("📍 {}", square));

                let our_square = chess_core::Square::new(
                    square.get_file().to_index() as u8,
                    square.get_rank().to_index() as u8,
                ).unwrap();

                if let Some(piece) = app.engine.piece_at(our_square) {
                    ui.add_space(8.0);
                    ui.label(format!("♟️  {:?} {:?}", piece.color, piece.piece_type));
                    ui.label(format!("⚡ {} legal moves", app.legal_moves_for_selected.len()));
                } else {
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("Empty square").italics().weak());
                }
            } else {
                // No selection state
                ui.label(egui::RichText::new("No Square Selected").weak().italics());
                ui.add_space(8.0);
                ui.label(egui::RichText::new("Click a piece to see details").size(12.0).weak());
            }

            ui.add_space(8.0);
        });
}
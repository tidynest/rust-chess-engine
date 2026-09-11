//! Left panel showing position information.
//!
//! Displays captured pieces and selected square details.

use chess_core::openings;
use eframe::egui;

use crate::app::state::ChessApp;
use crate::ui::components::material;

/// Draw the left panel
pub fn draw(app: &ChessApp, ui: &mut egui::Ui) {
    egui::Panel::left("left_panel")
        .default_size(200.0)
        .size_range(150.0..=300.0)
        .resizable(true)
        .show(ui, |ui| {
            let theme = &app.theme;
            ui.add_space(theme.space_xs);

            ui.heading("Position Info");
            ui.add_space(theme.space_xs);

            ui.separator();
            ui.add_space(theme.space_sm);

            material::draw_material_count(app, ui);

            ui.add_space(theme.space_sm);
            ui.separator();
            ui.add_space(theme.space_sm);

            // ponytail: looked up every frame; at most 36 hash lookups.
            if let Some(opening) = openings::of(&app.game_history) {
                ui.label(
                    egui::RichText::new("Opening")
                        .strong()
                        .size(theme.font_size_md),
                );
                ui.add_space(theme.space_xs);
                ui.label(opening.name);
                ui.label(egui::RichText::new(opening.eco).weak());
                ui.add_space(theme.space_sm);
                ui.separator();
                ui.add_space(theme.space_sm);
            }

            if let Some(square) = app.selected_square {
                ui.label(
                    egui::RichText::new("Selected Square")
                        .strong()
                        .size(theme.font_size_md),
                );
                ui.add_space(theme.space_xs);

                ui.label(format!("📍 {}", square));

                if let Some((piece, color)) = app.piece_at(square) {
                    ui.add_space(theme.space_xs);
                    ui.label(format!("♟️  {color:?} {piece:?}"));
                    ui.label(format!(
                        "⚡ {} legal moves",
                        app.legal_moves_for_selected.len()
                    ));
                } else {
                    ui.add_space(theme.space_xs);
                    ui.label(egui::RichText::new("Empty square").italics().weak());
                }
            } else {
                // No selection state
                ui.label(egui::RichText::new("No Square Selected").weak().italics());
                ui.add_space(theme.space_xs);
                ui.label(
                    egui::RichText::new("Click a piece to see details")
                        .size(theme.font_size_xs)
                        .weak(),
                );
            }

            ui.add_space(theme.space_xs);
        });
}

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

            ui.label(
                egui::RichText::new("Position")
                    .strong()
                    .size(theme.font_size_md),
            );
            ui.add_space(theme.space_xs);
            let history = &app.game_history;
            let board = history.current_board();
            let ply = history.move_count();
            ui.label(format!(
                "Move {}, {} to play",
                ply / 2 + 1,
                if board.side_to_move() == chess::Color::White {
                    "White"
                } else {
                    "Black"
                }
            ));
            ui.label(format!(
                "{} since the last capture or pawn move",
                match history.halfmove_clock() {
                    1 => "1 ply".to_owned(),
                    n => format!("{n} plies"),
                }
            ));
            if let Some(square) = app.selected_square {
                ui.label(format!(
                    "{square}: {} legal moves",
                    app.legal_moves_for_selected.len()
                ));
            }
            ui.add_space(theme.space_xs);
            ui.label(egui::RichText::new("FEN").weak().size(theme.font_size_xs));
            // Selectable, so it can be copied straight from the panel.
            ui.add(
                egui::Label::new(egui::RichText::new(board.to_string()).size(theme.font_size_xs))
                    .wrap()
                    .selectable(true),
            );
            ui.add_space(theme.space_xs);
        });
}

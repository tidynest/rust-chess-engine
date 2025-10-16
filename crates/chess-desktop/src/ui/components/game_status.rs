//! Game status display component.
//!
//! Shows current game state (check, checkmate, stalemate, in progress).

use chess_core::{Color, GameState};
use eframe::egui::{self, Ui, Color32};

use crate::app::state::ChessApp;

/// Draw game status
pub fn draw(app: &ChessApp, ui: &mut Ui) {
    if app.engine.is_checkmate() {
        let winner = if app.engine.side_to_move() == Color::White {
            "Black wins by checkmate!"
        } else {
            "White wins by checkmate!"
        };
        ui.colored_label(Color32::from_rgb(255, 100, 100), winner);
    } else if app.engine.is_stalemate() {
        ui.colored_label(Color32::from_rgb(255, 200, 100), "Stalemate - Draw!");
    } else if app.engine.is_check() {
        ui.colored_label(Color32::from_rgb(255, 150, 50), "Check!");
    } else {
        ui.label("Game in progress");
    }
}
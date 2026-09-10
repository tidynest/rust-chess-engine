//! Game status display component.
//!
//! Shows current game state (check, checkmate, stalemate, in progress).

use chess::{BoardStatus, Color as ChessColor};
use eframe::egui::{Color32, Ui};

use crate::app::state::ChessApp;

/// Draw game status
pub fn draw(app: &ChessApp, ui: &mut Ui) {
    let board = app.board();
    match board.status() {
        BoardStatus::Checkmate => {
            let winner = if board.side_to_move() == ChessColor::White {
                "Black wins by checkmate!"
            } else {
                "White wins by checkmate!"
            };
            ui.colored_label(Color32::from_rgb(255, 100, 100), winner);
        }
        BoardStatus::Stalemate => {
            ui.colored_label(Color32::from_rgb(255, 200, 100), "Stalemate - Draw!");
        }
        BoardStatus::Ongoing if board.checkers().popcnt() > 0 => {
            ui.colored_label(Color32::from_rgb(255, 150, 50), "Check!");
        }
        BoardStatus::Ongoing => {
            ui.label("Game in progress");
        }
    }
}

//! Game status display component.
//!
//! Shows current game state (check, checkmate, stalemate, in progress).

use chess::{BoardStatus, Color as ChessColor};
use eframe::egui::{self, Ui};

use crate::app::state::ChessApp;

/// Draw game status
pub fn draw(app: &ChessApp, ui: &mut Ui) {
    let board = app.board();
    let theme = &app.theme;
    if let Some(color) = app.timeout {
        ui.colored_label(theme.error, format!("{color:?} lost on time"));
    } else if let Some(color) = app.resigned {
        ui.colored_label(theme.error, format!("{color:?} resigned"));
    } else if board.status() == BoardStatus::Checkmate {
        let winner = if board.side_to_move() == ChessColor::White {
            "Black wins by checkmate!"
        } else {
            "White wins by checkmate!"
        };
        ui.colored_label(theme.error, winner);
    } else if let Some(reason) = app.game_history.draw_reason() {
        ui.colored_label(theme.warning, format!("Draw by {}", reason.describe()));
    } else if board.checkers().popcnt() > 0 {
        ui.colored_label(theme.check, "Check!");
    } else {
        ui.label("Game in progress");
    }
    if let Some(notice) = &app.notice {
        ui.label(
            egui::RichText::new(notice)
                .color(theme.text_secondary)
                .small(),
        );
    }
}

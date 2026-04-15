//! Top menu bar.
//!
//! Contains game menu, view options, and turn indicator.

use chess_core::{Color, GameState};
use chess_engine::EngineCommand;
use eframe::egui::{self, Context};

use crate::app::state::{CapturedPiecesStyle, ChessApp};

/// Draw the top menu bar
pub fn draw(app: &mut ChessApp, ctx: &Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            draw_game_menu(app, ui, ctx);
            draw_view_menu(app, ui);
            draw_turn_indicator(app, ui);
        });
    });
}

/// Draw the Game menu
fn draw_game_menu(app: &mut ChessApp, ui: &mut egui::Ui, ctx: &Context) {
    ui.menu_button("Game", |ui| {
        if ui.button("🆕 New Game").clicked() {
            app.new_game();
            if app.play_vs_computer {
                app.request_engine_move();
            }
        }

        ui.separator();

        if ui.button("🔄 Flip Board").clicked() {
            app.board_flip = !app.board_flip;
        }

        ui.separator();

        if ui.button("❌ Quit").clicked() {
            if let Some(tx) = &app.stockfish_tx {
                let _ = tx.send(EngineCommand::Quit);
            }
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

/// Draw the View menu
fn draw_view_menu(app: &mut ChessApp, ui: &mut egui::Ui) {
    ui.menu_button("View", |ui| {
        ui.label("Board Colors");
        ui.horizontal(|ui| {
            ui.label("Light:");
            ui.color_edit_button_srgba(&mut app.light_square_color);
        });
        ui.horizontal(|ui| {
            ui.label("Dark:");
            ui.color_edit_button_srgba(&mut app.dark_square_color);
        });

        ui.separator();

        ui.label("Captured Pieces Style:");
        ui.radio_value(
            &mut app.captured_display_style,
            CapturedPiecesStyle::Lichess,
            "Lichess (advantage only)",
        );
        ui.radio_value(
            &mut app.captured_display_style,
            CapturedPiecesStyle::ChessCom,
            "Chess.com (all pieces)",
        );
    });
}

/// Draw the turn indicator
fn draw_turn_indicator(app: &ChessApp, ui: &mut egui::Ui) {
    let side_to_move = app.engine.side_to_move();
    let turn_text = if side_to_move == Color::White {
        "⚪ White to move"
    } else {
        "⚫ Black to move"
    };
    ui.label(egui::RichText::new(turn_text).size(14.0).strong());
}

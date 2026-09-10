//! Top menu bar.
//!
//! Contains game menu, view options, and turn indicator.

use chess::Color as ChessColor;
use chess_core::GameHistory;
use eframe::egui::{self, Context};

use crate::app::engine_link::EngineCommand;
use crate::app::state::{CapturedPiecesStyle, ChessApp};
use crate::ui::theme::ThemeVariant;

/// Draw the top menu bar
pub fn draw(app: &mut ChessApp, ctx: &Context) {
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            draw_game_menu(app, ui, ctx);
            draw_view_menu(app, ui, ctx);
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

        if ui.button("Copy FEN").clicked() {
            ctx.copy_text(app.board().to_string());
        }
        if ui.button("Copy PGN").clicked() {
            let (white, black) = player_names(app);
            ctx.copy_text(app.game_history.pgn(white, black));
        }
        if ui.button("Set up position...").clicked() {
            app.fen_input = Some(app.board().to_string());
        }

        ui.separator();

        if ui.button("❌ Quit").clicked() {
            app.send(EngineCommand::Quit);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

/// Who plays which side, for the PGN tags.
fn player_names(app: &ChessApp) -> (&'static str, &'static str) {
    if !app.play_vs_computer {
        return ("?", "?");
    }
    match app.computer_color {
        ChessColor::White => ("Stockfish", "Human"),
        ChessColor::Black => ("Human", "Stockfish"),
    }
}

/// The "Set up position" window: a FEN field and a Load button that stays
/// disabled until the FEN parses.
pub fn draw_setup_window(app: &mut ChessApp, ctx: &Context) {
    let Some(mut fen) = app.fen_input.take() else {
        return;
    };

    let mut open = true;
    let mut load = false;
    egui::Window::new("Set up position")
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label("FEN:");
            ui.add(egui::TextEdit::singleline(&mut fen).desired_width(420.0));
            let parsed = GameHistory::from_fen(fen.trim());
            ui.horizontal(|ui| {
                load = ui
                    .add_enabled(parsed.is_ok(), egui::Button::new("Load"))
                    .clicked();
                if parsed.is_err() {
                    ui.label("Not a valid FEN");
                }
            });
        });

    if load && let Ok(history) = GameHistory::from_fen(fen.trim()) {
        app.start_game(history);
    } else if open {
        app.fen_input = Some(fen);
    }
}

/// Draw the View menu
fn draw_view_menu(app: &mut ChessApp, ui: &mut egui::Ui, ctx: &Context) {
    ui.menu_button("View", |ui| {
        ui.menu_button("Theme", |ui| {
            for variant in ThemeVariant::all() {
                if ui
                    .selectable_label(app.theme_variant == variant, variant.name())
                    .clicked()
                {
                    app.set_theme(variant);
                    app.theme.apply(ctx);
                }
            }
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
    let turn_text = if app.board().side_to_move() == ChessColor::White {
        "⚪ White to move"
    } else {
        "⚫ Black to move"
    };
    ui.label(
        egui::RichText::new(turn_text)
            .size(app.theme.font_size_sm)
            .strong(),
    );
}

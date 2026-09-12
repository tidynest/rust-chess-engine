//! Top menu bar.
//!
//! Contains game menu, view options, and turn indicator.

use chess::Color as ChessColor;
use chess_core::GameHistory;
use eframe::egui::{self, Context};

use crate::app::engine_link::EngineCommand;
use crate::app::games;
use crate::app::state::{CapturedPiecesStyle, ChessApp};
use crate::ui::theme::ThemeVariant;

/// Draw the top menu bar
pub fn draw(app: &mut ChessApp, ui: &mut egui::Ui) {
    egui::Panel::top("top_panel").show(ui, |ui| {
        ui.horizontal(|ui| {
            draw_game_menu(app, ui);
            draw_view_menu(app, ui);
            draw_turn_indicator(app, ui);
        });
    });
}

/// Draw the Game menu
fn draw_game_menu(app: &mut ChessApp, ui: &mut egui::Ui) {
    ui.menu_button("Game", |ui| {
        if ui.button("🆕 New Game").clicked() {
            app.new_game();
        }

        ui.separator();

        if ui.button("🔄 Flip Board").clicked() {
            app.board_flip = !app.board_flip;
        }

        ui.separator();

        if ui.button("Copy FEN").clicked() {
            ui.ctx().copy_text(app.board().to_string());
        }
        if ui.button("Copy PGN").clicked() {
            ui.ctx().copy_text(app.pgn());
        }
        if ui.button("Set up position...").clicked() {
            app.fen_input = Some(app.board().to_string());
        }
        if ui.button("Load PGN...").clicked() {
            app.pgn_input = Some(String::new());
        }

        ui.separator();

        if ui
            .add_enabled(
                app.game_history.move_count() > 0,
                egui::Button::new("Save game").shortcut_text("Ctrl+S"),
            )
            .clicked()
        {
            app.save_game();
        }
        ui.menu_button("Open game", |ui| draw_saved_games(app, ui));

        ui.separator();

        if ui.button("❌ Quit").clicked() {
            app.send(EngineCommand::Quit);
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

/// The saved games, newest first, named by their files. Picking one plays
/// it through; a file that will not parse says so in the status panel.
fn draw_saved_games(app: &mut ChessApp, ui: &mut egui::Ui) {
    // ponytail: the directory is read every frame the submenu is open; at
    // twenty entries that is nothing.
    let saved = games::list(20);
    if saved.is_empty() {
        ui.label("No saved games");
        return;
    }
    for path in saved {
        let name = path
            .file_stem()
            .map_or_else(String::new, |stem| stem.to_string_lossy().into_owned());
        if !ui.button(&name).clicked() {
            continue;
        }
        app.open_pgn_file(&path);
        ui.close();
    }
}

/// The "Set up position" window: a FEN field and a Load button.
pub fn draw_setup_window(app: &mut ChessApp, ctx: &Context) {
    let Some(text) = app.fen_input.take() else {
        return;
    };
    match draw_loader(ctx, "Set up position", "FEN:", false, text, |fen| {
        GameHistory::from_fen(fen).map_err(|_| "Not a valid FEN".to_owned())
    }) {
        Loader::Loaded(history) => app.start_game(history),
        Loader::Open(text) => app.fen_input = Some(text),
        Loader::Closed => {}
    }
}

/// The "Load PGN" window: paste a game, Load plays it through.
pub fn draw_pgn_window(app: &mut ChessApp, ctx: &Context) {
    let Some(text) = app.pgn_input.take() else {
        return;
    };
    match draw_loader(
        ctx,
        "Load PGN",
        "PGN (paste with Ctrl+V):",
        true,
        text,
        |pgn| GameHistory::from_pgn(pgn).map_err(|e| e.to_string()),
    ) {
        Loader::Loaded(history) => app.start_game(history),
        Loader::Open(text) => app.pgn_input = Some(text),
        Loader::Closed => {}
    }
}

enum Loader {
    Loaded(GameHistory),
    Open(String),
    Closed,
}

/// A window with a text field and a Load button that stays disabled, with
/// the reason shown, until `parse` accepts the text.
fn draw_loader(
    ctx: &Context,
    title: &str,
    label: &str,
    multiline: bool,
    mut text: String,
    parse: impl Fn(&str) -> Result<GameHistory, String>,
) -> Loader {
    let mut open = true;
    let mut load = false;
    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.label(label);
            if multiline {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .desired_width(480.0)
                        .desired_rows(8),
                );
            } else {
                ui.add(egui::TextEdit::singleline(&mut text).desired_width(480.0));
            }
            let parsed = parse(text.trim());
            ui.horizontal(|ui| {
                load = ui
                    .add_enabled(parsed.is_ok(), egui::Button::new("Load"))
                    .clicked();
                if let Err(reason) = &parsed
                    && !text.trim().is_empty()
                {
                    ui.label(reason);
                }
            });
        });

    match (load, open) {
        (true, _) => parse(text.trim()).map_or(Loader::Closed, Loader::Loaded),
        (false, true) => Loader::Open(text),
        (false, false) => Loader::Closed,
    }
}

/// Draw the View menu
fn draw_view_menu(app: &mut ChessApp, ui: &mut egui::Ui) {
    ui.menu_button("View", |ui| {
        ui.menu_button("Theme", |ui| {
            for variant in ThemeVariant::all() {
                if ui
                    .selectable_label(app.theme_variant == variant, variant.name())
                    .clicked()
                {
                    app.set_theme(variant);
                    app.theme.apply(ui.ctx());
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

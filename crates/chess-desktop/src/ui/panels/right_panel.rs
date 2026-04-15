//! Right panel with game information and controls.
//!
//! Contains game status, engine controls, and move history.

use chess::Color as ChessColor;
use chess_core::{GameHistory, notation};
use eframe::egui::{self, Color32, Context};

use crate::app::engine_comm::EngineMode;
use crate::app::state::ChessApp;
use crate::ui::components::game_status;

/// Draw the right panel
pub fn draw(app: &mut ChessApp, ctx: &Context) {
    egui::SidePanel::right("right_panel")
        .default_width(250.0)
        .width_range(200.0..=400.0)
        .resizable(true)
        .show(ctx, |ui| {
            ui.heading("Game Information");
            ui.separator();

            game_status::draw(app, ui);

            ui.separator();

            draw_engine_controls(app, ui);

            ui.separator();

            // Calculate remaining space for move history
            let available_height = ui.available_height();
            // Reserve space for controls legend at bottom (approx 120px)
            let history_height = (available_height - 150.0).max(200.0);

            draw_move_history(app, ui, history_height);

            ui.separator();

            draw_controls_legend(ui);
        });
}

/// Draw engine controls section
fn draw_engine_controls(app: &mut ChessApp, ui: &mut egui::Ui) {
    ui.heading("Computer Opponent");
    ui.horizontal(|ui| {
        ui.label("Play vs Computer");
        ui.checkbox(&mut app.play_vs_computer, "");
    });

    if app.play_vs_computer {
        draw_color_selection(app, ui);
        draw_engine_settings(app, ui);
        draw_thinking_indicator(app, ui);
        draw_engine_analysis(app, ui);
    }
}

/// Draw computer color selection
fn draw_color_selection(app: &mut ChessApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Computer plays:");

        let old_color = app.computer_color;

        ui.radio_value(&mut app.computer_color, ChessColor::White, "White");
        ui.radio_value(&mut app.computer_color, ChessColor::Black, "Black");

        if old_color != app.computer_color {
            eprintln!("Computer color changed - resetting engine state");
            app.engine_thinking = false;
            app.disable_auto_request = false;
        }
    });
}

/// Draw engine settings collapsible section
fn draw_engine_settings(app: &mut ChessApp, ui: &mut egui::Ui) {
    ui.collapsing("Engine Settings", |ui| {
        ui.checkbox(&mut app.show_eval_bar, "Show evaluation bar");
        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.radio_value(&mut app.engine_mode, EngineMode::Depth, "Depth");
            ui.radio_value(&mut app.engine_mode, EngineMode::TimeLimit, "Time");
        });

        match app.engine_mode {
            EngineMode::Depth => {
                ui.horizontal(|ui| {
                    ui.label("Depth:");
                    ui.add(egui::Slider::new(&mut app.engine_depth, 5..=30).suffix(" ply"));
                });
                ui.label(format!("Stronger = slower (current: {})", app.engine_depth));
            }
            EngineMode::TimeLimit => {
                let mut time_ms = app.engine_movetime.unwrap_or(1000);
                ui.horizontal(|ui| {
                    ui.label("Time limit:");
                    ui.add(
                        egui::Slider::new(&mut time_ms, 100..=10000)
                            .suffix(" ms")
                            .logarithmic(true),
                    );
                });
                app.engine_movetime = Some(time_ms);
                ui.label(format!("Per move: {:.1}s", time_ms as f32 / 1000.0));
            }
            EngineMode::FullStrength => {
                ui.label("Maximum strength, no limits");
            }
        }

        ui.separator();

        ui.horizontal(|ui| {
            ui.label("Skill Level:");
            ui.add(egui::Slider::new(&mut app.engine_skill_level, 0..=20));
        });

        let elo = if app.engine_skill_level == 20 {
            "3200+".to_string()
        } else {
            format!("~{}", 1350 + app.engine_skill_level * 75)
        };
        ui.label(format!("Elo: {} (0=beginner, 20=master)", elo));

        ui.separator();

        if ui.button("Reset to defaults").clicked() {
            app.engine_depth = 20;
            app.engine_movetime = Some(1000);
            app.engine_mode = EngineMode::Depth;
            app.engine_skill_level = 20;
        }
    });
}

/// Draw thinking indicator
fn draw_thinking_indicator(app: &ChessApp, ui: &mut egui::Ui) {
    if app.engine_thinking {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label("Engine thinking...");
        });
    }
}

/// Draw engine analysis display
fn draw_engine_analysis(app: &ChessApp, ui: &mut egui::Ui) {
    if let Some(eval) = app.engine_evaluation {
        ui.separator();
        ui.heading("Engine Analysis");
        ui.horizontal(|ui| {
            ui.label("Evaluation:");
            let eval_text = if eval > 0.0 {
                format!("+{:.2}", eval)
            } else {
                format!("{:.2}", eval)
            };
            ui.label(egui::RichText::new(eval_text).strong());
        });
        ui.horizontal(|ui| {
            ui.label("Depth:");
            ui.label(format!("{}", app.engine_depth_current));
        });
        ui.horizontal(|ui| {
            ui.label("Nodes:");
            ui.label(format!("{}", app.engine_nodes));
        });
        if !app.engine_pv.is_empty() {
            ui.label("Principal Variation:");
            let pv_san = app.format_pv_san(&app.engine_pv);
            ui.label(pv_san.join(" "));
        }
    }
}

/// Draw move history with dynamic height and smart auto-scroll
fn draw_move_history(app: &mut ChessApp, ui: &mut egui::Ui, max_height: f32) {
    ui.heading("Move History");

    let mut clicked_move: Option<usize> = None;
    let full_move_count = app.game_history.total_moves();

    // Create scroll area with dynamic height
    let _scroll_output = egui::ScrollArea::vertical()
        .max_height(max_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true) // ✨ Smart auto-scroll: only follows if already at bottom
        .show(ui, |ui| {
            if full_move_count == 0 {
                ui.label("No moves yet");
            } else {
                let current_move = app.game_history.move_count();

                for move_index in 0..full_move_count {
                    if move_index % 2 == 0 {
                        ui.horizontal(|ui| {
                            // Move number
                            ui.label(
                                egui::RichText::new(format!("{}.", move_index / 2 + 1))
                                    .color(Color32::from_gray(160)),
                            );

                            // White's move
                            if let Some(chess_move) = app.game_history.get_move(move_index) {
                                let is_current = move_index + 1 == current_move;
                                let is_future = move_index + 1 > current_move;

                                let san = get_move_san(app, move_index, chess_move);

                                let mut text = egui::RichText::new(&san);
                                if is_current {
                                    text = text.strong().color(Color32::from_rgb(100, 200, 255));
                                } else if is_future {
                                    text = text.color(Color32::from_gray(120));
                                }

                                if ui.button(text).clicked() {
                                    clicked_move = Some(move_index);
                                }
                            }

                            // Black's move
                            if move_index + 1 < full_move_count
                                && let Some(chess_move) = app.game_history.get_move(move_index + 1)
                            {
                                let is_current = move_index + 2 == current_move;
                                let is_future = move_index + 2 > current_move;

                                let san = get_move_san(app, move_index + 1, chess_move);

                                let mut text = egui::RichText::new(&san);
                                if is_current {
                                    text = text.strong().color(Color32::from_rgb(100, 200, 255));
                                } else if is_future {
                                    text = text.color(Color32::from_gray(120));
                                }

                                if ui.button(text).clicked() {
                                    clicked_move = Some(move_index + 1);
                                }
                            }
                        });
                    }
                }
            }
        });

    // Handle move navigation
    if let Some(move_index) = clicked_move {
        app.jump_to_move(move_index);
    }
}

/// Get SAN notation for a move at given index
fn get_move_san(app: &ChessApp, move_index: usize, chess_move: &chess::ChessMove) -> String {
    if move_index < app.move_history.len() {
        app.move_history[move_index].clone()
    } else {
        let mut temp_history = GameHistory::new();
        for j in 0..move_index {
            if let Some(prev_move) = app.game_history.get_move(j) {
                temp_history.make_move(*prev_move);
            }
        }
        notation::format_move_san(chess_move, temp_history.current_board())
    }
}

/// Draw controls legend
fn draw_controls_legend(ui: &mut egui::Ui) {
    ui.heading("Controls");
    ui.label("• Click to select a piece");
    ui.label("• Click again to move");
    ui.label("• Drag and drop pieces");
    ui.label("• Green dots show legal moves");
}

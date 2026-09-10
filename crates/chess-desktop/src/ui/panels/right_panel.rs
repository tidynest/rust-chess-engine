//! Right panel with game information and controls.
//!
//! Contains game status, engine controls, and move history.

use chess::Color as ChessColor;
use eframe::egui::{self, Color32, Context};

use crate::app::engine_comm::EngineMode;
use crate::app::engine_link::EngineStatus;
use crate::app::state::ChessApp;
use crate::ui::components::{eval_bar, game_status};

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
    match &app.engine_status {
        EngineStatus::Starting => {
            ui.label("Starting Stockfish...");
        }
        EngineStatus::Failed(message) => {
            ui.colored_label(
                Color32::from_rgb(255, 100, 100),
                format!("Stockfish unavailable: {message}"),
            );
        }
        EngineStatus::Ready => {
            ui.horizontal(|ui| {
                ui.label("Play vs Computer");
                ui.checkbox(&mut app.play_vs_computer, "");
            });
        }
    }

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
            app.abort_search();
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
            ui.label(egui::RichText::new(eval_bar::label(eval)).strong());
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
    let total = app.game_history.total_moves();
    let current = app.game_history.move_count();

    egui::ScrollArea::vertical()
        .max_height(max_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true) // Follows new moves only while already at the bottom
        .show(ui, |ui| {
            if total == 0 {
                ui.label("No moves yet");
                return;
            }

            for white_index in (0..total).step_by(2) {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{}.", white_index / 2 + 1))
                            .color(Color32::from_gray(160)),
                    );

                    for index in white_index..(white_index + 2).min(total) {
                        let san = app.game_history.san(index).unwrap_or("?");
                        let mut text = egui::RichText::new(san);
                        if index + 1 == current {
                            text = text.strong().color(Color32::from_rgb(100, 200, 255));
                        } else if index + 1 > current {
                            text = text.color(Color32::from_gray(120));
                        }

                        if ui.button(text).clicked() {
                            clicked_move = Some(index);
                        }
                    }
                });
            }
        });

    if let Some(move_index) = clicked_move {
        app.jump_to_ply(move_index + 1);
    }
}

/// Draw controls legend
fn draw_controls_legend(ui: &mut egui::Ui) {
    ui.heading("Controls");
    ui.label("• Click or drag a piece to move it");
    ui.label("• Dots mark the legal targets");
    ui.label("• ← → step through the moves");
    ui.label("• Home and End jump to either end");
    ui.label("• F flips the board");
}

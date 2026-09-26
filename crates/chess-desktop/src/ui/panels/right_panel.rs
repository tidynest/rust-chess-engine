//! Right panel with game information and controls.
//!
//! Contains game status, engine controls, and move history.

use cozy_chess::Color as ChessColor;
use eframe::egui;

use crate::app::engine_comm::{EngineMode, SearchKind};
use crate::app::engine_link::EngineStatus;
use crate::app::state::{ChessApp, ComputerSide, EngineLine};
use crate::ui::components::{eval_bar, game_status};

/// Draw the right panel
pub fn draw(app: &mut ChessApp, ui: &mut egui::Ui) {
    egui::Panel::right("right_panel")
        .default_size(250.0)
        .size_range(200.0..=400.0)
        .resizable(true)
        .show(ui, |ui| {
            ui.heading("Game Information");
            ui.separator();

            game_status::draw(app, ui);

            ui.separator();

            draw_clock(app, ui);

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

/// The two clocks, the side to move in the accent colour and a clock under
/// ten seconds in red, plus the time control for the next game.
fn draw_clock(app: &mut ChessApp, ui: &mut egui::Ui) {
    let theme = app.theme.clone();
    if let Some(clock) = &app.clock {
        let to_move = app.board().side_to_move();
        for color in [ChessColor::White, ChessColor::Black] {
            let remaining = clock.remaining(color);
            let mut text = egui::RichText::new(format!(
                "{color:?}  {}",
                crate::app::clock::Clock::format(remaining)
            ))
            .size(theme.font_size_lg)
            .monospace();
            if remaining < std::time::Duration::from_secs(10) {
                text = text.color(theme.error);
            } else if color == to_move && app.timeout.is_none() {
                text = text.color(theme.primary).strong();
            }
            ui.label(text);
        }
    }

    ui.collapsing("Clock", |ui| {
        ui.checkbox(&mut app.clock_enabled, "Use a clock from the next game");
        ui.horizontal(|ui| {
            ui.label("Minutes:");
            ui.add(egui::Slider::new(&mut app.clock_minutes, 1..=60));
        });
        ui.horizontal(|ui| {
            ui.label("Increment:");
            ui.add(egui::Slider::new(&mut app.clock_increment_s, 0..=30).suffix(" s"));
        });
        ui.horizontal(|ui| {
            for (minutes, increment) in [(1, 0), (3, 2), (5, 3), (10, 0), (15, 10)] {
                if ui.small_button(format!("{minutes}+{increment}")).clicked() {
                    app.clock_minutes = minutes;
                    app.clock_increment_s = increment;
                    app.clock_enabled = true;
                }
            }
        });
    });
}

/// Draw engine controls section
fn draw_engine_controls(app: &mut ChessApp, ui: &mut egui::Ui) {
    ui.heading("Engine");
    match &app.engine_status {
        EngineStatus::Starting => {
            ui.label("Starting Stockfish...");
        }
        EngineStatus::Failed(message) => {
            ui.colored_label(app.theme.error, format!("Stockfish unavailable: {message}"));
            if ui.button("Try again").clicked() {
                app.start_engine(ui.ctx());
            }
        }
        EngineStatus::Ready => {
            let was_playing = app.play_vs_computer;
            let was_analysing = app.analysis;
            ui.checkbox(&mut app.play_vs_computer, "Play vs Computer");
            ui.checkbox(&mut app.analysis, "Analyse position");
            // Switching a mode off must not leave its search to land later.
            let dropped_play = was_playing && !app.play_vs_computer;
            let dropped_analysis = was_analysing && !app.analysis;
            if (dropped_play && app.search_kind == SearchKind::Play)
                || (dropped_analysis && app.search_kind == SearchKind::Analyse)
            {
                app.abort_search();
            }
            if !was_playing && app.play_vs_computer {
                app.face_computer();
            }
        }
    }

    if app.play_vs_computer {
        draw_color_selection(app, ui);
    }
    if app.engine_in_use() {
        draw_engine_settings(app, ui);
        draw_thinking_indicator(app, ui);
        draw_engine_analysis(app, ui);
        draw_play_best_move(app, ui);
    }
}

/// Draw computer color selection
fn draw_color_selection(app: &mut ChessApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label("Computer plays:");

        let old_side = app.computer_side;

        ui.radio_value(&mut app.computer_side, ComputerSide::White, "White");
        ui.radio_value(&mut app.computer_side, ComputerSide::Black, "Black");
        ui.radio_value(&mut app.computer_side, ComputerSide::Both, "Both");

        if old_side != app.computer_side {
            app.abort_search();
            app.disable_auto_request = false;
            app.face_computer();
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

        let mut limited = app.engine_elo.is_some();
        ui.checkbox(&mut limited, "Limit strength");
        app.engine_elo = limited.then(|| {
            // Stockfish's own scale, 1320 to 3190 since version 16.
            let mut elo = app.engine_elo.unwrap_or(1500);
            ui.horizontal(|ui| {
                ui.label("Elo:");
                ui.add(egui::Slider::new(&mut elo, 1320..=3190));
            });
            elo
        });

        if app.analysis {
            ui.separator();
            let lines = app.analysis_lines;
            ui.horizontal(|ui| {
                ui.label("Lines:");
                ui.add(egui::Slider::new(&mut app.analysis_lines, 1..=5));
            });
            // The running analysis was asked for the old number; start over.
            if lines != app.analysis_lines {
                app.abort_search();
                app.analysis_complete = false;
            }
        }

        ui.separator();

        let (threads, hash) = (app.engine_threads, app.engine_hash_mb);
        let max_threads = std::thread::available_parallelism().map_or(4, |n| n.get());
        ui.horizontal(|ui| {
            ui.label("Threads:");
            ui.add(egui::Slider::new(&mut app.engine_threads, 1..=max_threads));
        });
        ui.horizontal(|ui| {
            ui.label("Hash:");
            ui.add(
                egui::Slider::new(&mut app.engine_hash_mb, 16..=2048)
                    .suffix(" MB")
                    .logarithmic(true),
            );
        });

        ui.separator();

        if ui.button("Reset to defaults").clicked() {
            app.engine_depth = 20;
            app.engine_movetime = Some(1000);
            app.engine_mode = EngineMode::Depth;
            app.engine_elo = None;
            app.engine_threads = crate::app::state::default_threads();
            app.engine_hash_mb = 128;
            app.analysis_lines = 1;
        }

        // Options reach the engine only when they change; it restarts its
        // search threads and clears the hash for them.
        if (threads, hash) != (app.engine_threads, app.engine_hash_mb) {
            app.send_engine_options();
        }
    });
}

/// Draw thinking indicator
fn draw_thinking_indicator(app: &ChessApp, ui: &mut egui::Ui) {
    if app.engine_thinking {
        ui.horizontal(|ui| {
            ui.spinner();
            ui.label(match app.search_kind {
                SearchKind::Play => "Engine thinking...",
                SearchKind::Analyse => "Analysing...",
            });
        });
    }
}

/// The first line's score, depth and node count, then every line with its
/// score and variation.
fn draw_engine_analysis(app: &ChessApp, ui: &mut egui::Ui) {
    let Some(first) = app.engine_lines.first() else {
        return;
    };
    ui.separator();
    ui.heading("Engine Analysis");
    ui.horizontal(|ui| {
        ui.label("Evaluation:");
        ui.label(egui::RichText::new(eval_bar::label(first.score)).strong());
    });
    ui.horizontal(|ui| {
        ui.label("Depth:");
        ui.label(first.depth.to_string());
    });
    ui.horizontal(|ui| {
        ui.label("Nodes:");
        ui.label(app.engine_nodes.to_string());
    });
    let lines: Vec<&EngineLine> = app
        .engine_lines
        .iter()
        .filter(|line| !line.pv.is_empty())
        .collect();
    if !lines.is_empty() {
        ui.label(if lines.len() == 1 {
            "Principal Variation:"
        } else {
            "Lines:"
        });
    }
    for line in lines {
        ui.horizontal_wrapped(|ui| {
            ui.label(
                egui::RichText::new(eval_bar::label(line.score))
                    .monospace()
                    .strong(),
            );
            ui.label(app.format_pv_san(&line.pv).join(" "));
        });
    }
}

/// In analysis mode, a button that plays the first move of the best line.
fn draw_play_best_move(app: &mut ChessApp, ui: &mut egui::Ui) {
    let best = app
        .engine_lines
        .first()
        .and_then(|line| line.pv.first())
        .and_then(|uci| chess_core::notation::parse_uci(app.board(), uci));
    let allowed = app.analysis
        && !app.is_game_over()
        && app.pending_promotion.is_none()
        && !app.waiting_for_engine_move();
    if let Some(mv) = best
        && ui
            .add_enabled(allowed, egui::Button::new("Play the best move"))
            .clicked()
    {
        app.play_move(mv);
    }
}

/// Draw move history with dynamic height and smart auto-scroll
fn draw_move_history(app: &mut ChessApp, ui: &mut egui::Ui, max_height: f32) {
    ui.heading("Move History");

    let mut clicked_move: Option<usize> = None;
    let total = app.game_history.total_moves();
    let current = app.game_history.move_count();
    // Bring the current move into view once per change of position, so the
    // list follows the arrow keys without fighting the user's own scrolling.
    let follow = app.history_scrolled_to != current;

    egui::ScrollArea::vertical()
        .max_height(max_height)
        .auto_shrink([false, false])
        .stick_to_bottom(true) // Follows new moves only while already at the bottom
        .show(ui, |ui| {
            if total == 0 {
                ui.label("No moves yet");
                return;
            }

            let theme = &app.theme;
            for white_index in (0..total).step_by(2) {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(format!("{}.", white_index / 2 + 1))
                            .color(theme.text_secondary),
                    );

                    for index in white_index..(white_index + 2).min(total) {
                        let san = app.game_history.san(index).unwrap_or("?");
                        let mut text = egui::RichText::new(san);
                        if index + 1 == current {
                            text = text.strong().color(theme.primary);
                        } else if index + 1 > current {
                            text = text.color(theme.text_disabled);
                        }

                        let response = ui.button(text);
                        if response.clicked() {
                            clicked_move = Some(index);
                        }
                        if follow && index + 1 == current.max(1) {
                            response.scroll_to_me(Some(egui::Align::Center));
                        }
                    }
                });
            }
        });

    app.history_scrolled_to = current;
    if let Some(move_index) = clicked_move {
        app.jump_to_ply(move_index + 1);
    }
}

/// Draw controls legend
fn draw_controls_legend(ui: &mut egui::Ui) {
    ui.heading("Controls");
    ui.label("• Click or drag a piece to move it");
    ui.label("• Dots mark the legal targets");
    ui.label("• Left and Right arrows, or Ctrl+Z and Ctrl+Y, step through the moves");
    ui.label("• Home and End jump to either end");
    ui.label("• F flips the board, Ctrl+N starts a new game, Ctrl+S saves it");
}

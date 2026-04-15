//! Central panel with chess board and evaluation bar.
//!
//! Contains the main board display and control buttons.

use eframe::egui::{self, Context, Vec2};

use crate::app::state::ChessApp;
use crate::ui::components::eval_bar;

/// Draw the central panel
pub fn draw(app: &mut ChessApp, ctx: &Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.heading("Chess Board");
            ui.add_space(16.0); // 8pt grid: 16px spacing

            let (max_board_size, eval_bar_width, spacing) = calculate_board_dimensions(app, ui);

            // Draw board and capture the left edge position for button alignment
            let board_left_edge =
                draw_board_and_eval_bar(app, ui, max_board_size, eval_bar_width, spacing);

            ui.add_space(16.0); // 8pt grid: 16px spacing

            // Draw buttons aligned to the board's left edge
            draw_control_buttons(app, ui, board_left_edge);
        });
    });
}

/// Calculate board dimensions based on available space
fn calculate_board_dimensions(app: &ChessApp, ui: &egui::Ui) -> (f32, f32, f32) {
    let available = ui.available_size();
    let eval_bar_width = 40.0; // Fixed width for eval bar
    let spacing = 8.0; // 8pt grid spacing

    // Calculate max board size - MORE GENEROUS than before!
    let max_board_size = if app.play_vs_computer && app.show_eval_bar {
        // Reserve space for eval bar + spacing, but be more generous
        let available_width = available.x - eval_bar_width - spacing - 20.0; // Reduced margin
        available_width
            .min(available.y - 100.0) // Less vertical reserve
            .clamp(400.0, 900.0) // Increased max to 900px!
    } else {
        // Even more space when no eval bar
        (available.x - 20.0)
            .min(available.y - 100.0)
            .clamp(400.0, 900.0)
    };

    (max_board_size, eval_bar_width, spacing)
}

/// Draw board and evaluation bar side by side
/// Returns the left edge x-coordinate of the board for button alignment
fn draw_board_and_eval_bar(
    app: &mut ChessApp,
    ui: &mut egui::Ui,
    max_board_size: f32,
    eval_bar_width: f32,
    spacing: f32,
) -> f32 {
    let mut board_left_edge = 0.0;

    ui.horizontal(|ui| {
        // Calculate total width needed
        let total_width = if app.play_vs_computer && app.show_eval_bar {
            max_board_size + eval_bar_width + spacing
        } else {
            max_board_size
        };

        // Center the content by adding space on the left
        let available_width = ui.available_width();
        let left_padding = if available_width > total_width {
            (available_width - total_width) / 2.0
        } else {
            0.0
        };

        ui.add_space(left_padding);

        // Store the board's left edge position (current cursor position)
        board_left_edge = ui.cursor().left();

        // Draw the board
        ui.allocate_ui_with_layout(Vec2::splat(max_board_size), egui::Layout::default(), |ui| {
            app.draw_board(ui);
        });

        // Draw eval bar if enabled
        if app.play_vs_computer && app.show_eval_bar {
            ui.add_space(spacing);
            ui.allocate_ui_with_layout(
                Vec2::new(eval_bar_width, max_board_size),
                egui::Layout::default(),
                |ui| {
                    eval_bar::draw(app, ui);
                },
            );
        }
    });

    board_left_edge
}

/// Draw control buttons aligned to the board's left edge
fn draw_control_buttons(app: &mut ChessApp, ui: &mut egui::Ui, board_left_edge: f32) {
    ui.horizontal(|ui| {
        // Calculate current cursor position and add spacing to align with board
        let current_left = ui.cursor().left();
        let alignment_offset = board_left_edge - current_left;

        if alignment_offset > 0.0 {
            ui.add_space(alignment_offset);
        }

        // Buttons now aligned to board's left edge
        if ui.button("🆕 New Game").clicked() {
            app.new_game();
            if app.play_vs_computer {
                app.request_engine_move();
            }
        }
        if ui.button("🔃 Flip Board").clicked() {
            app.board_flip = !app.board_flip;
        }

        ui.add_space(8.0); // 8pt grid spacing
        ui.separator();
        ui.add_space(8.0);

        if ui
            .add_enabled(app.game_history.can_undo(), egui::Button::new("⬅ Undo"))
            .clicked()
            && app.game_history.undo()
        {
            app.sync_engine_from_history();
            app.selected_square = None;
            app.legal_moves_for_selected.clear();
            app.disable_auto_request = true;
        }

        if ui
            .add_enabled(app.game_history.can_redo(), egui::Button::new("➡ Redo"))
            .clicked()
            && app.game_history.redo()
        {
            app.sync_engine_from_history();
            app.selected_square = None;
            app.legal_moves_for_selected.clear();
            app.disable_auto_request = true;
        }
    });
}

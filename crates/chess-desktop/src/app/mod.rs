//! Application state and core logic.
//!
//! This module contains the main ChessApp struct and its state management.

pub mod clock;
pub mod engine_comm;
pub mod engine_link;
pub mod games;
pub mod settings;
pub mod state;

pub use engine_comm::{EngineMode, SearchKind};
pub use engine_link::{EngineCommand, EngineEvent, EngineStatus, SearchRequest};
pub use settings::Settings;
pub use state::{CapturedPiecesStyle, ChessApp};

impl eframe::App for ChessApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        self.handle_shortcuts(&ctx);
        self.tick_clock(&ctx);

        // Poll engine responses
        let best_move = self.poll_engine_responses();

        // Apply engine move if received
        if let Some(mv) = best_move {
            self.apply_engine_move(&mv);
        }

        // Render UI panels
        crate::ui::panels::top_bar::draw(self, ui);
        crate::ui::panels::right_panel::draw(self, ui);
        crate::ui::panels::left_panel::draw(self, ui);
        crate::ui::panels::central_panel::draw(self, ui);
        self.draw_promotion_picker(&ctx);
        crate::ui::panels::central_panel::draw_resign_prompt(self, &ctx);
        crate::ui::panels::top_bar::draw_setup_window(self, &ctx);
        crate::ui::panels::top_bar::draw_pgn_window(self, &ctx);

        self.auto_request();
    }

    /// Closing the window ends the engine and keeps the settings.
    fn on_exit(&mut self) {
        Settings::from_app(self).save();
        self.send(EngineCommand::Quit);
    }
}

impl ChessApp {
    /// Arrow keys or Ctrl+Z and Ctrl+Y step through the history, Home and
    /// End jump to its ends, F flips the board, Ctrl+N starts a new game,
    /// Ctrl+S saves it, Escape drops the selection or the resign prompt.
    /// Ignored while a text field has the keyboard.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        use egui::Key;

        if ctx.egui_wants_keyboard_input() {
            return;
        }
        let pressed = |key| ctx.input(|input| input.key_pressed(key));
        let command = |key| ctx.input(|input| input.modifiers.command && input.key_pressed(key));
        if command(Key::N) {
            self.new_game();
        }
        if command(Key::S) {
            self.save_game();
        }
        if pressed(Key::ArrowLeft) || command(Key::Z) {
            self.undo();
        }
        if pressed(Key::ArrowRight) || command(Key::Y) {
            self.redo();
        }
        if pressed(Key::Home) {
            self.jump_to_ply(0);
        }
        if pressed(Key::End) {
            self.jump_to_ply(self.game_history.total_moves());
        }
        if pressed(Key::F) {
            self.board_flip = !self.board_flip;
        }
        if pressed(Key::Escape) {
            self.confirm_resign = false;
            self.selected_square = None;
            self.legal_moves_for_selected.clear();
        }
    }
}

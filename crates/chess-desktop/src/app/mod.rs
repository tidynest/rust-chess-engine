//! Application state and core logic.
//!
//! This module contains the main ChessApp struct and its state management.

pub mod engine_comm;
pub mod state;

pub use engine_comm::EngineMode;
pub use state::{CapturedPiecesStyle, ChessApp};

impl eframe::App for ChessApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Poll engine responses
        let best_move = self.poll_engine_responses();

        // Apply engine move if received
        if let Some(mv) = best_move {
            self.engine_best_move = Some(mv.clone());
            self.apply_engine_move(&mv);
        }

        // Render UI panels
        crate::ui::panels::top_bar::draw(self, ctx);
        crate::ui::panels::right_panel::draw(self, ctx);
        crate::ui::panels::left_panel::draw(self, ctx);
        crate::ui::panels::central_panel::draw(self, ctx);

        // Auto-request engine move if needed
        self.auto_request_engine_move();
    }
}

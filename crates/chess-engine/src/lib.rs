// Re-export core traits for use by desktop and other crates
pub use chess_core::{GameRenderer, GameState, GameStorage};

// Additional engine-level types can be defined here
pub enum Effect {
    MoveMade(String),
    GameEnded(GameResult),
    Check,
    Promotion,
}

pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
}

// Later, this module will contain the Stockfish integration
pub mod stockfish {
    // Stockfish UCI integration will go here in Phase 5
    // This is a placeholder for now

    pub struct StockfishEngine {
        // Will contain process handle, etc.
    }

    impl StockfishEngine {
        pub fn new() -> Self {
            StockfishEngine {}
        }
    }
}
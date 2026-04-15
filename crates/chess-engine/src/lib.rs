// chess-engine crate - Coordination layer for chess game logic and AI

pub mod stockfish;

// Re-export core traits for use by desktop and other crates
pub use chess_core::{GameRenderer, GameState, GameStorage};

// Re-export stockfish types for convenience
pub use stockfish::{EngineCommand, EngineResponse, StockfishEngine};

// Additional engine-level types can be defined here
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    MoveMade(String),
    GameEnded(GameResult),
    Check,
    Promotion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
}

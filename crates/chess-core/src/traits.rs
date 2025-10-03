//! Core trait definitions for the chess engine

use crate::{Color, GameError, Move, Piece, Square, StorageError};

/// Core game state operations
pub trait GameState {
    /// Make a move on the board
    fn make_move(&mut self, chess_move: Move) -> Result<(), GameError>;

    /// Get all legal moves in the current position
    fn legal_moves(&self) -> Vec<Move>;

    /// Check if the game is in checkmate
    fn is_checkmate(&self) -> bool;

    /// Check if the game is in stalemate
    fn is_stalemate(&self) -> bool;

    /// Check if current side is in check
    fn is_check(&self) -> bool;

    /// Get the current side to move
    fn side_to_move(&self) -> Color;

    /// Get piece at a given square
    fn piece_at(&self, square: Square) -> Option<Piece>;
}

/// Game persistence operations
pub trait GameStorage {
    /// Save a game state
    fn save_game(&self, id: &str, game: &dyn GameState) -> Result<(), StorageError>;

    /// Load a game state
    fn load_game(&self, id: &str) -> Result<Box<dyn GameState>, StorageError>;

    /// List available game IDs
    fn list_games(&self) -> Result<Vec<String>, StorageError>;

    /// Delete a saved game
    fn delete_game(&self, id: &str) -> Result<(), StorageError>;
}

/// Rendering operations for the game
pub trait GameRenderer {
    /// Render result type specific to the renderer
    type Output;

    /// Render the current board state (incl. position)
    fn render_board(&self, game: &dyn GameState) -> Self::Output;

    /// Render a specific move (for animations)
    fn render_moves(&self, game: &dyn GameState, chess_move: Move) -> Self::Output;
}

/// Chess engine analysis trait
pub trait ChessAnalyser {
    /// Analyse a position and return best move
    fn analyse(&self, game: &dyn GameState, depth: u8) -> Result<Move, GameError>;

    /// Get evaluation score for current position
    fn evaluate(&self, game: &dyn GameState, count: usize) -> Vec<(Move, f32)>;
}

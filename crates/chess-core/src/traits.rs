//! Core trait definitions for the chess engine

use crate::{Color, GameError, Move, Piece, Square};

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

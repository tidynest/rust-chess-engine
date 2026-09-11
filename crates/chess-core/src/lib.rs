//! Chess domain types, move validation and history, built on the `chess` crate.

use thiserror::Error;

/// Represents a piece colour
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

/// Chess piece types
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// A chess piece with colour and type
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

/// Board square representation (0-63 for a1-h8)
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Square(u8);

impl Square {
    pub fn new(file: u8, rank: u8) -> Option<Self> {
        if file < 8 && rank < 8 {
            Some(Square(rank * 8 + file))
        } else {
            None
        }
    }

    pub fn from_index(index: u8) -> Option<Self> {
        if index < 64 {
            Some(Square(index))
        } else {
            None
        }
    }

    pub fn file(&self) -> u8 {
        self.0 % 8
    }

    pub fn rank(&self) -> u8 {
        self.0 / 8
    }

    pub fn to_algebraic(&self) -> String {
        format!(
            "{}{}",
            (b'a' + self.file()) as char,
            (b'1' + self.rank()) as char
        )
    }
}

// Both types index squares as rank * 8 + file with a1 = 0, so the conversion is a plain copy.
impl From<chess::Square> for Square {
    fn from(square: chess::Square) -> Self {
        Square(square.to_int())
    }
}

impl From<Square> for chess::Square {
    fn from(square: Square) -> Self {
        chess::Square::make_square(
            chess::Rank::from_index(square.rank() as usize),
            chess::File::from_index(square.file() as usize),
        )
    }
}

/// Represents a chess move
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<PieceType>,
}

/// Game errors
#[derive(Error, Debug)]
pub enum GameError {
    #[error("Invalid move: {0}")]
    InvalidMove(String),

    #[error("Game is already over")]
    GameOver,

    #[error("Invalid position")]
    InvalidPosition,
}

pub mod display;
pub mod engine;
pub mod game;
pub mod notation;
pub mod openings;
pub mod traits;

pub use engine::ChessEngine;
pub use game::{DrawReason, GameHistory, PgnError, PgnTags};
pub use traits::GameState;

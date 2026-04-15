//! Pure chess domain logic with zero external dependencies

use thiserror::Error;

/// Represents a piece colour
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct Piece {
    pub color: Color,
    pub piece_type: PieceType,
}

/// Board square representation (0-63 for a1-h8)
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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

/// Represents a chess move
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
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

/// Storage errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Game not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

pub mod display;
pub mod engine;
pub mod game;
pub mod notation;
pub mod traits;

pub use engine::ChessEngine;
pub use game::GameHistory;
pub use traits::{ChessAnalyser, GameRenderer, GameState, GameStorage};

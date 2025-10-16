//! Type conversion utilities.
//!
//! Converts between different chess piece type representations.

use chess::Piece as ChessPiece;
use chess_core::PieceType;

/// Convert chess crate piece to chess_core piece type
pub fn convert_piece_type(piece: ChessPiece) -> PieceType {
    match piece {
        ChessPiece::Pawn => PieceType::Pawn,
        ChessPiece::Knight => PieceType::Knight,
        ChessPiece::Bishop => PieceType::Bishop,
        ChessPiece::Rook => PieceType::Rook,
        ChessPiece::Queen => PieceType::Queen,
        ChessPiece::King => PieceType::King,
    }
}

/// Convert chess_core piece type to chess crate piece
pub fn convert_to_chess_piece(piece_type: PieceType) -> ChessPiece {
    match piece_type {
        PieceType::Pawn => ChessPiece::Pawn,
        PieceType::Knight => ChessPiece::Knight,
        PieceType::Bishop => ChessPiece::Bishop,
        PieceType::Rook => ChessPiece::Rook,
        PieceType::Queen => ChessPiece::Queen,
        PieceType::King => ChessPiece::King,
    }
}
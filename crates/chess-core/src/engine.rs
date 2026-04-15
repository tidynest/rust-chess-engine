//! Chess engine implementation using the chess crate for move generation

use crate::traits::GameState;
use crate::{Color, GameError, Move, Piece, PieceType, Square};
use chess::{
    Board, BoardStatus, ChessMove, Color as ChessColor, File, Rank, Square as ChessSquare,
};
use std::str::FromStr;

/// Wrapper around the chess crate's Board
pub struct ChessEngine {
    board: Board,
}

impl ChessEngine {
    pub fn new() -> Self {
        ChessEngine {
            board: Board::default(), // Standard starting position
        }
    }

    pub fn from_fen(fen: &str) -> Result<Self, GameError> {
        Board::from_str(fen)
            .map(|board| ChessEngine { board })
            .map_err(|_| GameError::InvalidPosition)
    }

    /// Get the underlying board (for GUI access
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// Convert our Square to chess crate Square
    fn to_chess_square(square: Square) -> ChessSquare {
        ChessSquare::make_square(
            Rank::from_index(square.rank() as usize),
            File::from_index(square.file() as usize),
        )
    }

    /// Convert chess crate Square to out Square
    fn from_chess_square(square: ChessSquare) -> Square {
        Square::new(
            square.get_file().to_index() as u8,
            square.get_rank().to_index() as u8,
        )
        .unwrap()
    }

    /// Convert our Move to chess crate ChessMove
    fn to_chess_move(&self, mv: Move) -> Option<ChessMove> {
        let from = Self::to_chess_square(mv.from);
        let to = Self::to_chess_square(mv.to);

        // Find the matching legal move
        let mut legal_moves = chess::MoveGen::new_legal(&self.board);
        legal_moves.find(|m| m.get_source() == from && m.get_dest() == to)
    }

    /// Convert chess Color to our Color
    fn convert_color(color: ChessColor) -> Color {
        match color {
            ChessColor::White => Color::White,
            ChessColor::Black => Color::Black,
        }
    }

    /// Convert chess Piece to our Piece
    fn convert_piece_type(piece: chess::Piece) -> PieceType {
        match piece {
            chess::Piece::Pawn => PieceType::Pawn,
            chess::Piece::Knight => PieceType::Knight,
            chess::Piece::Bishop => PieceType::Bishop,
            chess::Piece::Rook => PieceType::Rook,
            chess::Piece::Queen => PieceType::Queen,
            chess::Piece::King => PieceType::King,
        }
    }
}

impl GameState for ChessEngine {
    fn make_move(&mut self, chess_move: Move) -> Result<(), GameError> {
        // Convert to chess crate move and validate
        let legal_move = self
            .to_chess_move(chess_move)
            .ok_or_else(|| GameError::InvalidMove("Illegal move".to_string()))?;

        // Make the move
        self.board = self.board.make_move_new(legal_move);
        Ok(())
    }

    fn legal_moves(&self) -> Vec<Move> {
        let moves = chess::MoveGen::new_legal(&self.board);
        moves
            .map(|m| Move {
                from: Self::from_chess_square(m.get_source()),
                to: Self::from_chess_square(m.get_dest()),
                promotion: m.get_promotion().map(Self::convert_piece_type),
            })
            .collect()
    }

    fn is_checkmate(&self) -> bool {
        self.board.status() == BoardStatus::Checkmate
    }

    fn is_stalemate(&self) -> bool {
        self.board.status() == BoardStatus::Stalemate
    }

    fn is_check(&self) -> bool {
        self.board.checkers().popcnt() > 0
    }

    fn side_to_move(&self) -> Color {
        Self::convert_color(self.board.side_to_move())
    }

    fn piece_at(&self, square: Square) -> Option<Piece> {
        let chess_square = Self::to_chess_square(square);
        self.board.piece_on(chess_square).and_then(|piece_type| {
            self.board.color_on(chess_square).map(|color| Piece {
                color: Self::convert_color(color),
                piece_type: Self::convert_piece_type(piece_type),
            })
        })
    }
}

impl Default for ChessEngine {
    fn default() -> Self {
        Self::new()
    }
}

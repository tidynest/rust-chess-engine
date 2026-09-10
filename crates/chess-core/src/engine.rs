//! Chess engine implementation using the chess crate for move generation

use crate::traits::GameState;
use crate::{Color, GameError, Move, Piece, PieceType, Square};
use chess::{Board, BoardStatus, ChessMove, Color as ChessColor, Square as ChessSquare};
use std::str::FromStr;

/// Wrapper around the chess crate's Board
#[derive(Clone)]
pub struct ChessEngine {
    board: Board,
}

impl ChessEngine {
    pub fn new() -> Self {
        ChessEngine {
            board: Board::default(), // Standard starting position
        }
    }

    pub fn from_board(board: Board) -> Self {
        ChessEngine { board }
    }

    pub fn from_fen(fen: &str) -> Result<Self, GameError> {
        Board::from_str(fen)
            .map(Self::from_board)
            .map_err(|_| GameError::InvalidPosition)
    }

    /// Get the underlying board (for GUI access)
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// The legal move matching `mv`, if any. A promotion must name its piece.
    fn to_chess_move(&self, mv: Move) -> Option<ChessMove> {
        let from: ChessSquare = mv.from.into();
        let to: ChessSquare = mv.to.into();

        chess::MoveGen::new_legal(&self.board).find(|m| {
            m.get_source() == from
                && m.get_dest() == to
                && m.get_promotion().map(Self::convert_piece_type) == mv.promotion
        })
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

impl ChessEngine {
    /// Play a move given in standard algebraic notation, such as `Nf3` or `O-O`.
    pub fn make_san(&mut self, san: &str) -> Result<(), GameError> {
        let mv = crate::notation::parse_san(&self.board, san)
            .ok_or_else(|| GameError::InvalidMove(format!("no legal move written {san:?}")))?;
        self.board = self.board.make_move_new(mv);
        Ok(())
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
                from: m.get_source().into(),
                to: m.get_dest().into(),
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
        let chess_square: ChessSquare = square.into();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse_algebraic;

    #[test]
    fn test_promotion_requires_a_piece_and_keeps_it() {
        let mut engine = ChessEngine::from_fen("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();

        assert!(engine.make_move(parse_algebraic("a7a8").unwrap()).is_err());
        assert!(engine.make_move(parse_algebraic("a7a8n").unwrap()).is_ok());

        let promoted = engine.piece_at(Square::new(0, 7).unwrap()).unwrap();
        assert_eq!(promoted.piece_type, PieceType::Knight);
    }
}

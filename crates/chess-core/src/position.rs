//! Basic position implementation

use crate::{Color, GameError, Move, Piece, PieceType, Square};
use crate::traits::GameState;

/// Basic board position (will be replaced with bitboards later)
pub struct Position {
    board: [Option<Piece>; 64],
    side_to_move: Color,
    // ... Castling rights, en passant, etc. will be added later
}

impl Position {
    pub fn new() -> Self {
        let mut board = [None; 64];

        // Set up initial position
        // White pieces
        board[0] = Some(Piece { color: Color::White, piece_type: PieceType::Rook });
        board[1] = Some(Piece { color: Color::White, piece_type: PieceType::Knight });
        board[2] = Some(Piece { color: Color::White, piece_type: PieceType::Bishop });
        board[3] = Some(Piece { color: Color::White, piece_type: PieceType::Queen });
        board[4] = Some(Piece { color: Color::White, piece_type: PieceType::King });
        board[5] = Some(Piece { color: Color::White, piece_type: PieceType::Bishop });
        board[6] = Some(Piece { color: Color::White, piece_type: PieceType::Knight });
        board[7] = Some(Piece { color: Color::White, piece_type: PieceType::Rook });

        for i in 8..16 {
            board[i] = Some(Piece { color: Color::White, piece_type: PieceType::Pawn});
        }

        // Black pieces
        for i in 48..56 {
            board[i] = Some(Piece { color: Color::Black, piece_type: PieceType::Pawn });
        }

        board[56] = Some(Piece { color: Color::Black, piece_type: PieceType::Rook });
        board[57] = Some(Piece { color: Color::Black, piece_type: PieceType::Knight });
        board[58] = Some(Piece { color: Color::Black, piece_type: PieceType::Bishop });
        board[59] = Some(Piece { color: Color::Black, piece_type: PieceType::Queen });
        board[60] = Some(Piece { color: Color::Black, piece_type: PieceType::King });
        board[61] = Some(Piece { color: Color::Black, piece_type: PieceType::Bishop });
        board[62] = Some(Piece { color: Color::Black, piece_type: PieceType::Knight });
        board[63] = Some(Piece { color: Color::Black, piece_type: PieceType::Rook });

        Position {
            board,
            side_to_move: Color::White,
        }
    }
}

impl GameState for Position {
    fn make_move(&mut self, chess_move: Move) -> Result<(), GameError> {
        // Basic move validation (enhance later!)
        let piece = self.board[chess_move.from.0 as usize]
            .ok_or_else(|| GameError::InvalidMove("No piece at source".to_string()))?;

        if piece.color != self.side_to_move {
            return Err(GameError::InvalidMove("Not your turn".to_string()));
        }

        // Make the move (simplified - no legality check yet)
        self.board[chess_move.to.0 as usize] = self.board[chess_move.from.0 as usize];
        self.board[chess_move.from.0 as usize] = None;

        // Handle promotion if specified
        if let Some(promo) = chess_move.promotion {
            self.board[chess_move.to.0 as usize] = Some(Piece {
                color: self.side_to_move,
                piece_type: promo,
            });
        }

        self.side_to_move = self.side_to_move.opposite();
        Ok(())
    }

    fn legal_moves(&self) -> Vec<Move> {
        // Placeholder - Implement proper move generation
        Vec::new()
    }

    fn is_checkmate(&self) -> bool {
        false  // Placeholder
    }

    fn is_stalemate(&self) -> bool {
        false  // Placeholder
    }

    fn is_check(&self) -> bool {
        false  // Placeholder
    }

    fn side_to_move(&self) -> Color {
        self.side_to_move
    }

    fn piece_at(&self, square: Square) -> Option<Piece> {
        self.board[square.0 as usize]
    }
}
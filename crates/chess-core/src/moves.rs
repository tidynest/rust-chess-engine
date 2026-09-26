//! Helpers over cozy-chess: the legal moves as a list, and the spelling of
//! castling, which cozy-chess stores as the king taking its own rook while
//! UCI, PGN and the board want the king's real destination.

use cozy_chess::{Board, File, GameStatus, Move, Piece, Rank, Square};

/// Every legal move on `board`.
pub fn legal_moves(board: &Board) -> Vec<Move> {
    let mut moves = Vec::with_capacity(40);
    board.generate_moves(|piece_moves| {
        moves.extend(piece_moves);
        false
    });
    moves
}

/// True when `mv` castles: the king lands on its own rook.
pub fn is_castling(board: &Board, mv: Move) -> bool {
    board.piece_on(mv.from) == Some(Piece::King)
        && board.color_on(mv.to) == Some(board.side_to_move())
}

/// The square the moving piece ends on: the king's g or c file when castling.
pub fn destination(board: &Board, mv: Move) -> Square {
    if !is_castling(board, mv) {
        return mv.to;
    }
    let file = if mv.to.file() > mv.from.file() {
        File::G
    } else {
        File::C
    };
    Square::new(file, mv.from.rank())
}

/// The move as UCI writes it for standard chess: `e1g1`, not `e1h1`.
pub fn to_uci(board: &Board, mv: Move) -> String {
    let mut text = format!("{}{}", mv.from, destination(board, mv));
    if let Some(piece) = mv.promotion {
        text.push(piece.into());
    }
    text
}

/// True when the side to move is checkmated.
pub fn is_checkmate(board: &Board) -> bool {
    board.status() == GameStatus::Won
}

/// True when the side to move has no legal move and is not in check.
pub fn is_stalemate(board: &Board) -> bool {
    board.checkers().is_empty() && !board.generate_moves(|_| true)
}

/// The board after `mv`, which must be legal.
pub fn after(board: &Board, mv: Move) -> Board {
    let mut next = board.clone();
    next.play(mv);
    next
}

/// The rank a pawn of `color` promotes on.
pub fn back_rank(color: cozy_chess::Color) -> Rank {
    Rank::Eighth.relative_to(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn castling_is_spelt_for_uci_and_the_board() {
        let board: Board = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1".parse().unwrap();
        let short = Move {
            from: Square::E1,
            to: Square::H1,
            promotion: None,
        };
        let long = Move {
            from: Square::E1,
            to: Square::A1,
            promotion: None,
        };
        assert!(is_castling(&board, short));
        assert_eq!(destination(&board, short), Square::G1);
        assert_eq!(destination(&board, long), Square::C1);
        assert_eq!(to_uci(&board, short), "e1g1");
        assert_eq!(to_uci(&board, long), "e1c1");

        let rook = Move {
            from: Square::A1,
            to: Square::A8,
            promotion: None,
        };
        assert!(!is_castling(&board, rook));
        assert_eq!(to_uci(&board, rook), "a1a8");
        assert_eq!(legal_moves(&board).len(), 26);
    }

    #[test]
    fn mate_and_stalemate() {
        let mate: Board = "rnb1kbnr/pppp1ppp/8/4p3/6Pq/5P2/PPPPP2P/RNBQKBNR w KQkq - 1 3"
            .parse()
            .unwrap();
        assert!(is_checkmate(&mate));
        assert!(!is_stalemate(&mate));
        let stale: Board = "7k/5Q2/6K1/8/8/8/8/8 b - - 0 1".parse().unwrap();
        assert!(is_stalemate(&stale));
        assert!(!is_checkmate(&stale));
    }
}

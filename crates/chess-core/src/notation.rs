//! Algebraic notation parsing utilities

use crate::{Move, PieceType, Square};

/// Parse long algebraic notation (e.g., "e2e4", "e7e8q"). Files and promotion
/// letters are accepted in either case.
pub fn parse_algebraic(s: &str) -> Option<Move> {
    let (from, to, promo) = match *s.as_bytes() {
        [f1, r1, f2, r2] => ((f1, r1), (f2, r2), None),
        [f1, r1, f2, r2, p] => ((f1, r1), (f2, r2), Some(p)),
        _ => return None,
    };

    // `Square::new` rejects anything past h8, so only the underflow needs guarding here.
    let square = |(file, rank): (u8, u8)| {
        Square::new(
            file.to_ascii_lowercase().checked_sub(b'a')?,
            rank.checked_sub(b'1')?,
        )
    };

    let promotion = match promo.map(|p| p.to_ascii_lowercase()) {
        None => None,
        Some(b'q') => Some(PieceType::Queen),
        Some(b'r') => Some(PieceType::Rook),
        Some(b'b') => Some(PieceType::Bishop),
        Some(b'n') => Some(PieceType::Knight),
        Some(_) => return None,
    };

    Some(Move {
        from: square(from)?,
        to: square(to)?,
        promotion,
    })
}

/// Format a move as algebraic notation
pub fn to_algebraic(mv: &Move) -> String {
    let mut result = format!("{}{}", mv.from.to_algebraic(), mv.to.to_algebraic());

    if let Some(promo) = mv.promotion {
        result.push(match promo {
            PieceType::Queen => 'q',
            PieceType::Rook => 'r',
            PieceType::Bishop => 'b',
            PieceType::Knight => 'n',
            _ => return result,
        });
    }

    result
}

/// Convert a ChessMove to Standard Algebraic Notation with disambiguation
pub fn format_move_san(mv: &chess::ChessMove, board: &chess::Board) -> String {
    use chess::{BoardStatus, Piece};

    let piece = board.piece_on(mv.get_source());
    let from = mv.get_source();
    let to = mv.get_dest();
    let is_capture = board.piece_on(to).is_some()
        || (piece == Some(Piece::Pawn) && from.get_file() != to.get_file()); // En passant

    let mut notation = String::new();

    match piece {
        Some(Piece::King)
            if from
                .get_file()
                .to_index()
                .abs_diff(to.get_file().to_index())
                == 2 =>
        {
            notation.push_str(if to.get_file() > from.get_file() {
                "O-O"
            } else {
                "O-O-O"
            });
        }
        Some(Piece::Pawn) => {
            if is_capture {
                notation.push((b'a' + from.get_file() as u8) as char);
                notation.push('x');
            }
            notation.push_str(&format!("{}", to));

            if let Some(promo) = mv.get_promotion() {
                notation.push('=');
                notation.push(match promo {
                    Piece::Queen => 'Q',
                    Piece::Rook => 'R',
                    Piece::Bishop => 'B',
                    Piece::Knight => 'N',
                    _ => '?',
                });
            }
        }
        Some(p) => {
            notation.push(match p {
                Piece::King => 'K',
                Piece::Queen => 'Q',
                Piece::Rook => 'R',
                Piece::Bishop => 'B',
                Piece::Knight => 'N',
                _ => unreachable!(),
            });

            let disambiguate = needs_disambiguation(board, *mv);
            match disambiguate {
                Disambiguation::File => {
                    notation.push((b'a' + from.get_file() as u8) as char);
                }
                Disambiguation::Rank => {
                    notation.push((b'1' + from.get_rank() as u8) as char);
                }
                Disambiguation::Both => {
                    notation.push_str(&format!("{}", from));
                }
                Disambiguation::None => {}
            }

            if is_capture {
                notation.push('x');
            }

            notation.push_str(&format!("{}", to));
        }
        None => return format!("{}", mv), // Fallback
    }

    let new_board = board.make_move_new(*mv);
    if new_board.checkers().popcnt() > 0 {
        if new_board.status() == BoardStatus::Checkmate {
            notation.push('#');
        } else {
            notation.push('+');
        }
    }

    notation
}

#[derive(Debug)]
enum Disambiguation {
    None,
    File,
    Rank,
    Both,
}

fn needs_disambiguation(board: &chess::Board, mv: chess::ChessMove) -> Disambiguation {
    use chess::{MoveGen, Piece};

    let piece = match board.piece_on(mv.get_source()) {
        Some(p) => p,
        None => return Disambiguation::None,
    };

    if piece == Piece::Pawn || piece == Piece::King {
        return Disambiguation::None;
    }

    let to = mv.get_dest();
    let from = mv.get_source();

    let all_moves = MoveGen::new_legal(board);
    let same_dest_moves: Vec<chess::ChessMove> = all_moves
        .filter(|m| {
            m.get_dest() == to
                && m.get_source() != from
                && board.piece_on(m.get_source()) == Some(piece)
        })
        .collect();

    if same_dest_moves.is_empty() {
        return Disambiguation::None;
    }

    let same_file = same_dest_moves
        .iter()
        .any(|m| m.get_source().get_file() == from.get_file());

    let same_rank = same_dest_moves
        .iter()
        .any(|m| m.get_source().get_rank() == from.get_rank());

    match (same_file, same_rank) {
        (true, true) => Disambiguation::Both,
        (true, false) => Disambiguation::Rank,
        (false, _) => Disambiguation::File,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::{Board, ChessMove, Square};
    use std::str::FromStr;

    #[test]
    fn test_format_move_san_pawn() {
        let board = Board::default();
        let mv = ChessMove::new(Square::E2, Square::E4, None);
        assert_eq!(format_move_san(&mv, &board), "e4");
    }

    #[test]
    fn test_format_move_san_capture() {
        let board =
            Board::from_str("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2")
                .unwrap();
        let mv = ChessMove::new(Square::E4, Square::D5, None);
        assert_eq!(format_move_san(&mv, &board), "exd5");
    }

    #[test]
    fn test_format_move_san_piece_move() {
        let board = Board::default();
        let mv = ChessMove::new(Square::G1, Square::F3, None);
        assert_eq!(format_move_san(&mv, &board), "Nf3");
    }

    #[test]
    fn test_format_move_san_castling() {
        let board = Board::from_str("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let mv = ChessMove::new(Square::E1, Square::G1, None);
        assert_eq!(format_move_san(&mv, &board), "O-O");
    }

    #[test]
    fn test_format_move_san_castling_with_check() {
        let board = Board::from_str("5k2/8/8/8/8/8/8/4K2R w K - 0 1").unwrap();
        let mv = ChessMove::new(Square::E1, Square::G1, None);
        assert_eq!(format_move_san(&mv, &board), "O-O+");
    }

    #[test]
    fn test_format_move_san_check() {
        let board =
            Board::from_str("rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4")
                .unwrap();
        let mv = ChessMove::new(Square::C4, Square::F7, None);
        assert_eq!(format_move_san(&mv, &board), "Bxf7+");
    }

    #[test]
    fn test_parse_algebraic() {
        let mv = parse_algebraic("e2e4").unwrap();
        assert_eq!(mv.from.to_algebraic(), "e2");
        assert_eq!(mv.to.to_algebraic(), "e4");
        assert_eq!(mv.promotion, None);
    }

    #[test]
    fn test_parse_algebraic_promotion_and_case() {
        let mv = parse_algebraic("E7E8N").unwrap();
        assert_eq!(mv.from.to_algebraic(), "e7");
        assert_eq!(mv.to.to_algebraic(), "e8");
        assert_eq!(mv.promotion, Some(PieceType::Knight));
    }

    #[test]
    fn test_parse_algebraic_rejects_garbage() {
        for bad in [
            "1234",
            "e2e9",
            "i2e4",
            "e7e8k",
            "e2e",
            "e2e4e5",
            "\u{e9}2e4",
        ] {
            assert_eq!(parse_algebraic(bad), None, "{bad:?} should not parse");
        }
    }
}

//! Algebraic notation parsing utilities

use crate::{Move, PieceType, Square};

/// Parse algebraic notation (e.g., "e2e4", "e7e8q")
pub fn parse_algebraic(s: &str) -> Option<Move> {
    let bytes = s.as_bytes();
    if bytes.len() < 4 || bytes.len() > 5 {
        return None;
    }

    let from_file = (bytes[0] as char).to_digit(18)? as u8 -10;  // a=0, b=1, etc.
    let from_rank = (bytes[1] as char).to_digit(10)? as u8 -1;  // 1=0, 2=1, etc.

    let to_file = (bytes[2] as char).to_digit(18)? as u8 -10;
    let to_rank = (bytes[3] as char).to_digit(10)? as u8 -1;

    let from = Square::new(from_file, from_rank)?;
    let to = Square::new(to_file, to_rank)?;

    let promotion = if bytes.len() == 5 {
        match bytes[4] {
            b'q' | b'Q' => Some(PieceType::Queen),
            b'r' | b'R' => Some(PieceType::Rook),
            b'b' | b'B' => Some(PieceType::Bishop),
            b'n' | b'N' => Some(PieceType::Knight),
            _ => return None,
        }
    } else {
        None
    };

    Some(Move { from, to, promotion })
}

/// Format a move as algebraic notation
pub fn to_algebraic(mv: &Move) -> String {
    let mut result = format!(
        "{}{}",
        mv.from.to_algebraic(),
        mv.to.to_algebraic()
    );

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
    use chess::{Piece, BoardStatus};

    let piece = board.piece_on(mv.get_source());
    let from = mv.get_source();
    let to = mv.get_dest();
    let is_capture = board.piece_on(to).is_some()
        || (piece == Some(Piece::Pawn) && from.get_file() != to.get_file());  // En passant

    if let Some(Piece::King) = piece {
        let from_file = from.get_file() as i8;
        let to_file = to.get_file() as i8;

        if (to_file - from_file).abs() == 2 {
            return if to_file > from_file {
                "O-O".to_string()
            } else {
                "O-O-O".to_string()
            };
        }
    }

    let mut notation = String::new();

    match piece {
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

            let disambiguate = needs_disambiguation(board, mv);
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
        None => return format!("{}", mv),  // Fallback
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

fn needs_disambiguation(board: &chess::Board, mv: &chess::ChessMove) -> Disambiguation {
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

    let same_file = same_dest_moves.iter().any(|m| {
        m.get_source().get_file() == from.get_file()
    });

    let same_rank = same_dest_moves.iter().any(|m| {
        m.get_source().get_rank() == from.get_rank()
    });

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
        let board = Board::from_str("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2").unwrap();
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
    fn test_format_move_san_check() {
        let board = Board::from_str("rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4").unwrap();
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
}
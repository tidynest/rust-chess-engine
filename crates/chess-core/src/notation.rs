//! Move notation: long algebraic as UCI writes it, and standard algebraic.
//! Both readers match against the legal moves, so only playable moves come
//! back.

/// The legal move written as `uci` on `board`, such as `e2e4` or `e7e8q`,
/// in either case. A promotion without its piece is not a move.
pub fn parse_uci(board: &chess::Board, uci: &str) -> Option<chess::ChessMove> {
    let wanted = uci.to_ascii_lowercase();
    chess::MoveGen::new_legal(board).find(|mv| mv.to_string() == wanted)
}

/// The legal move written as `text` on `board`, in either notation.
pub fn parse_move(board: &chess::Board, text: &str) -> Option<chess::ChessMove> {
    parse_uci(board, text).or_else(|| parse_san(board, text))
}

/// The legal move written as `san` on `board`. Check marks and annotation
/// glyphs are ignored, castling may use zeros, and `e8Q` is read as `e8=Q`.
/// Matching against the formatter keeps the two in step, and covers en
/// passant, which the `chess` crate's own parser does not.
pub fn parse_san(board: &chess::Board, san: &str) -> Option<chess::ChessMove> {
    let mut wanted = san.trim_end_matches(['+', '#', '!', '?']).replace('0', "O");
    // A promotion piece without its "=": "e8Q", "axb8N".
    let bytes = wanted.as_bytes();
    if bytes.len() > 2
        && b"QRBN".contains(&bytes[bytes.len() - 1])
        && bytes[bytes.len() - 2].is_ascii_digit()
    {
        wanted.insert(wanted.len() - 1, '=');
    }
    chess::MoveGen::new_legal(board)
        .find(|mv| format_move_san(mv, board).trim_end_matches(['+', '#']) == wanted)
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
    fn test_every_legal_move_has_its_own_san() {
        for fen in [
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "r3k2r/pppq1ppp/2n2n2/3pp3/1b1PP3/2N2N2/PPPQ1PPP/R3KB1R w KQkq - 0 1",
            "r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4",
            "1n2k3/1P6/8/8/3Pp3/8/8/4K3 b - d3 0 1",
            "1n2k3/P7/8/8/8/8/8/4K3 w - - 0 1",
            "3rk3/8/8/8/8/8/8/R3K2R w KQ - 0 1",
            "8/8/8/8/8/2N1N3/8/2N1K1k1 w - - 0 1",
            "3q4/8/8/8/8/8/8/Q2QK1kq w - - 0 1",
        ] {
            let board = Board::from_str(fen).unwrap();
            for mv in chess::MoveGen::new_legal(&board) {
                let san = format_move_san(&mv, &board);
                assert_eq!(parse_san(&board, &san), Some(mv), "{fen} {san}");
            }
        }
    }

    #[test]
    fn test_parse_san_accepts_common_spellings() {
        let board = Board::from_str("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
        let castle = ChessMove::new(Square::E1, Square::G1, None);
        assert_eq!(parse_san(&board, "O-O"), Some(castle));
        assert_eq!(parse_san(&board, "0-0"), Some(castle));
        assert_eq!(parse_san(&board, "O-O+"), Some(castle));

        let board = Board::from_str("1n2k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        let promote = ChessMove::new(Square::A7, Square::B8, Some(chess::Piece::Knight));
        assert_eq!(parse_san(&board, "axb8=N"), Some(promote));
        assert_eq!(parse_san(&board, "axb8N"), Some(promote));
        assert_eq!(parse_san(&board, "Nf3"), None);
    }

    #[test]
    fn test_parse_uci_matches_only_legal_moves() {
        let board = Board::default();
        let e4 = ChessMove::new(Square::E2, Square::E4, None);
        assert_eq!(parse_uci(&board, "e2e4"), Some(e4));
        assert_eq!(parse_uci(&board, "E2E4"), Some(e4));
        for bad in ["e2e5", "e2", "xyz", "1234", "e2e9", "e2e4e5", "\u{e9}2e4"] {
            assert_eq!(parse_uci(&board, bad), None, "{bad:?}");
        }

        let board = Board::from_str("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        assert_eq!(
            parse_uci(&board, "a7a8"),
            None,
            "a promotion needs its piece"
        );
        assert_eq!(
            parse_uci(&board, "a7a8n"),
            Some(ChessMove::new(
                Square::A7,
                Square::A8,
                Some(chess::Piece::Knight)
            ))
        );
        assert_eq!(parse_uci(&board, "a7a8k"), None);
    }
}

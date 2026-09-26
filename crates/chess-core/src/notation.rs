//! Move notation: long algebraic as UCI writes it, and standard algebraic.
//! Both readers match against the legal moves, so only playable moves come
//! back.

use cozy_chess::{Board, Move, Piece};

use crate::moves::{after, is_castling, is_checkmate, legal_moves, to_uci};

/// The legal move written as `uci` on `board`, such as `e2e4` or `e7e8q`,
/// in either case. Castling reads as `e1g1` or as cozy-chess's `e1h1`. A
/// promotion without its piece is not a move.
pub fn parse_uci(board: &Board, uci: &str) -> Option<Move> {
    let wanted = uci.to_ascii_lowercase();
    legal_moves(board)
        .into_iter()
        .find(|&mv| to_uci(board, mv) == wanted || mv.to_string() == wanted)
}

/// The legal move written as `text` on `board`, in either notation.
pub fn parse_move(board: &Board, text: &str) -> Option<Move> {
    parse_uci(board, text).or_else(|| parse_san(board, text))
}

/// The legal move written as `san` on `board`. Check marks and annotation
/// glyphs are ignored, castling may use zeros, and `e8Q` is read as `e8=Q`.
/// Matching against the formatter keeps the two in step.
pub fn parse_san(board: &Board, san: &str) -> Option<Move> {
    let mut wanted = san.trim_end_matches(['+', '#', '!', '?']).replace('0', "O");
    // A promotion piece without its "=": "e8Q", "axb8N".
    let bytes = wanted.as_bytes();
    if bytes.len() > 2
        && b"QRBN".contains(&bytes[bytes.len() - 1])
        && bytes[bytes.len() - 2].is_ascii_digit()
    {
        wanted.insert(wanted.len() - 1, '=');
    }
    legal_moves(board)
        .into_iter()
        .find(|&mv| format_move_san(&mv, board).trim_end_matches(['+', '#']) == wanted)
}

fn piece_letter(piece: Piece) -> char {
    match piece {
        Piece::Pawn => 'P',
        Piece::Knight => 'N',
        Piece::Bishop => 'B',
        Piece::Rook => 'R',
        Piece::Queen => 'Q',
        Piece::King => 'K',
    }
}

/// The move in standard algebraic notation, with the disambiguation the
/// position needs and a check or mate mark.
pub fn format_move_san(mv: &Move, board: &Board) -> String {
    let mv = *mv;
    let Some(piece) = board.piece_on(mv.from) else {
        return to_uci(board, mv);
    };
    let (from, to) = (mv.from, mv.to);
    let us = board.side_to_move();
    let is_capture =
        board.color_on(to) == Some(!us) || (piece == Piece::Pawn && from.file() != to.file()); // En passant

    let mut notation = String::new();
    if is_castling(board, mv) {
        notation.push_str(if to.file() > from.file() {
            "O-O"
        } else {
            "O-O-O"
        });
    } else if piece == Piece::Pawn {
        if is_capture {
            notation.push(from.file().into());
            notation.push('x');
        }
        notation.push_str(&to.to_string());
        if let Some(promotion) = mv.promotion {
            notation.push('=');
            notation.push(piece_letter(promotion));
        }
    } else {
        notation.push(piece_letter(piece));
        match needs_disambiguation(board, mv) {
            Disambiguation::File => notation.push(from.file().into()),
            Disambiguation::Rank => notation.push(from.rank().into()),
            Disambiguation::Both => notation.push_str(&from.to_string()),
            Disambiguation::None => {}
        }
        if is_capture {
            notation.push('x');
        }
        notation.push_str(&to.to_string());
    }

    let next = after(board, mv);
    if !next.checkers().is_empty() {
        notation.push(if is_checkmate(&next) { '#' } else { '+' });
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

fn needs_disambiguation(board: &Board, mv: Move) -> Disambiguation {
    let Some(piece) = board.piece_on(mv.from) else {
        return Disambiguation::None;
    };
    if piece == Piece::Pawn || piece == Piece::King {
        return Disambiguation::None;
    }

    let others: Vec<Move> = legal_moves(board)
        .into_iter()
        .filter(|other| {
            other.to == mv.to && other.from != mv.from && board.piece_on(other.from) == Some(piece)
        })
        .collect();
    if others.is_empty() {
        return Disambiguation::None;
    }
    let same_file = others
        .iter()
        .any(|other| other.from.file() == mv.from.file());
    let same_rank = others
        .iter()
        .any(|other| other.from.rank() == mv.from.rank());
    match (same_file, same_rank) {
        (true, true) => Disambiguation::Both,
        (true, false) => Disambiguation::Rank,
        (false, _) => Disambiguation::File,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cozy_chess::Square;

    fn mv(from: Square, to: Square, promotion: Option<Piece>) -> Move {
        Move {
            from,
            to,
            promotion,
        }
    }

    #[test]
    fn test_format_move_san_pawn() {
        let board = Board::default();
        let mv = mv(Square::E2, Square::E4, None);
        assert_eq!(format_move_san(&mv, &board), "e4");
    }

    #[test]
    fn test_format_move_san_capture() {
        let board = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2"
            .parse::<Board>()
            .unwrap();
        let mv = mv(Square::E4, Square::D5, None);
        assert_eq!(format_move_san(&mv, &board), "exd5");
    }

    #[test]
    fn test_format_move_san_piece_move() {
        let board = Board::default();
        let mv = mv(Square::G1, Square::F3, None);
        assert_eq!(format_move_san(&mv, &board), "Nf3");
    }

    #[test]
    fn test_format_move_san_castling() {
        let board = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"
            .parse::<Board>()
            .unwrap();
        let mv = mv(Square::E1, Square::H1, None);
        assert_eq!(format_move_san(&mv, &board), "O-O");
    }

    #[test]
    fn test_format_move_san_castling_with_check() {
        let board = "5k2/8/8/8/8/8/8/4K2R w K - 0 1".parse::<Board>().unwrap();
        let mv = mv(Square::E1, Square::H1, None);
        assert_eq!(format_move_san(&mv, &board), "O-O+");
    }

    #[test]
    fn test_format_move_san_check() {
        let board = "rnbqk2r/pppp1ppp/5n2/2b1p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4"
            .parse::<Board>()
            .unwrap();
        let mv = mv(Square::C4, Square::F7, None);
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
            let board = fen.parse::<Board>().unwrap();
            for mv in legal_moves(&board) {
                let san = format_move_san(&mv, &board);
                assert_eq!(parse_san(&board, &san), Some(mv), "{fen} {san}");
            }
        }
    }

    #[test]
    fn test_parse_san_accepts_common_spellings() {
        let board = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1"
            .parse::<Board>()
            .unwrap();
        let castle = mv(Square::E1, Square::H1, None);
        assert_eq!(parse_san(&board, "O-O"), Some(castle));
        assert_eq!(parse_san(&board, "0-0"), Some(castle));
        assert_eq!(parse_san(&board, "O-O+"), Some(castle));

        let board = "1n2k3/P7/8/8/8/8/8/4K3 w - - 0 1".parse::<Board>().unwrap();
        let promote = mv(Square::A7, Square::B8, Some(Piece::Knight));
        assert_eq!(parse_san(&board, "axb8=N"), Some(promote));
        assert_eq!(parse_san(&board, "axb8N"), Some(promote));
        assert_eq!(parse_san(&board, "Nf3"), None);
    }

    #[test]
    fn test_parse_uci_matches_only_legal_moves() {
        let board = Board::default();
        let e4 = mv(Square::E2, Square::E4, None);
        assert_eq!(parse_uci(&board, "e2e4"), Some(e4));
        assert_eq!(parse_uci(&board, "E2E4"), Some(e4));
        for bad in ["e2e5", "e2", "xyz", "1234", "e2e9", "e2e4e5", "\u{e9}2e4"] {
            assert_eq!(parse_uci(&board, bad), None, "{bad:?}");
        }

        let board = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1".parse::<Board>().unwrap();
        assert_eq!(
            parse_uci(&board, "a7a8"),
            None,
            "a promotion needs its piece"
        );
        assert_eq!(
            parse_uci(&board, "a7a8n"),
            Some(mv(Square::A7, Square::A8, Some(Piece::Knight)))
        );
        assert_eq!(parse_uci(&board, "a7a8k"), None);
    }
}

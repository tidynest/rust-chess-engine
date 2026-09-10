//! Game state management with undo/redo support

use crate::notation::{format_move_san, parse_san};
use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, Piece};
use std::fmt::Write;
use std::str::FromStr;
use thiserror::Error;

/// Why a PGN could not be read.
#[derive(Error, Debug, PartialEq, Eq)]
pub enum PgnError {
    #[error("the FEN tag is not a valid position")]
    InvalidFen,
    #[error("move {ply}: cannot play {san:?} here")]
    BadMove { ply: usize, san: String },
}

/// Why a game ended without a checkmate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawReason {
    Stalemate,
    ThreefoldRepetition,
    FiftyMoveRule,
    InsufficientMaterial,
}

impl DrawReason {
    pub fn describe(self) -> &'static str {
        match self {
            Self::Stalemate => "stalemate",
            Self::ThreefoldRepetition => "threefold repetition",
            Self::FiftyMoveRule => "fifty-move rule",
            Self::InsufficientMaterial => "insufficient material",
        }
    }
}

/// Game state with full move history for undo/redo. Each move's SAN is
/// computed once, when it is played.
pub struct GameHistory {
    positions: Vec<Board>,
    moves: Vec<ChessMove>,
    sans: Vec<String>,
    current_index: usize,
    /// Halfmove clock of the start position; the `chess` crate drops it.
    start_halfmove_clock: u32,
}

impl GameHistory {
    pub fn new() -> Self {
        Self::from_board(Board::default())
    }

    pub fn from_board(board: Board) -> Self {
        Self {
            positions: vec![board],
            moves: Vec::new(),
            sans: Vec::new(),
            current_index: 0,
            start_halfmove_clock: 0,
        }
    }

    /// Read a PGN: a FEN tag sets the start, other tags are ignored, and the
    /// movetext is played with comments, variations, glyphs and the result
    /// stripped. Stops at the first move that cannot be played.
    pub fn from_pgn(text: &str) -> Result<Self, PgnError> {
        let mut fen = None;
        let mut movetext = String::new();
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(tag) = trimmed.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
                if let Some(value) = tag.strip_prefix("FEN ") {
                    fen = Some(value.trim().trim_matches('"').to_owned());
                }
            } else {
                movetext.push_str(line);
                movetext.push('\n');
            }
        }

        let mut history = match fen {
            Some(fen) => Self::from_fen(&fen).map_err(|_| PgnError::InvalidFen)?,
            None => Self::new(),
        };
        for (index, san) in movetext_moves(&movetext).into_iter().enumerate() {
            let mv = parse_san(history.current_board(), &san).ok_or_else(|| PgnError::BadMove {
                ply: index + 1,
                san: san.clone(),
            })?;
            history.make_move(mv);
        }
        Ok(history)
    }

    /// Start from a FEN, keeping its halfmove clock for the fifty-move rule.
    pub fn from_fen(fen: &str) -> Result<Self, chess::Error> {
        let mut history = Self::from_board(Board::from_str(fen)?);
        history.start_halfmove_clock = fen
            .split_whitespace()
            .nth(4)
            .and_then(|clock| clock.parse().ok())
            .unwrap_or(0);
        Ok(history)
    }

    pub fn current_board(&self) -> &Board {
        &self.positions[self.current_index]
    }

    /// The position the game began from.
    pub fn start_board(&self) -> &Board {
        &self.positions[0]
    }

    /// The moves played so far, each paired with the board it was played on.
    pub fn played(&self) -> impl DoubleEndedIterator<Item = (&Board, ChessMove)> {
        self.positions
            .iter()
            .zip(self.current_moves().iter().copied())
    }

    /// Plies since the last capture or pawn move.
    pub fn halfmove_clock(&self) -> u32 {
        let quiet = self
            .played()
            .rev()
            .take_while(|(board, mv)| {
                board.piece_on(mv.get_source()) != Some(Piece::Pawn)
                    && board.piece_on(mv.get_dest()).is_none()
            })
            .count();
        let clock = quiet as u32;
        if quiet == self.current_index {
            clock + self.start_halfmove_clock
        } else {
            clock
        }
    }

    /// Why the current position is a draw, if it is one. Repetition and the
    /// fifty-move rule are treated as automatic rather than claimable.
    pub fn draw_reason(&self) -> Option<DrawReason> {
        let board = self.current_board();
        match board.status() {
            BoardStatus::Checkmate => return None,
            BoardStatus::Stalemate => return Some(DrawReason::Stalemate),
            BoardStatus::Ongoing => {}
        }
        if insufficient_material(board) {
            return Some(DrawReason::InsufficientMaterial);
        }
        let seen = self.positions[..=self.current_index]
            .iter()
            .filter(|position| *position == board)
            .count();
        if seen >= 3 {
            return Some(DrawReason::ThreefoldRepetition);
        }
        if self.halfmove_clock() >= 100 {
            return Some(DrawReason::FiftyMoveRule);
        }
        None
    }

    /// True once the game cannot continue.
    pub fn is_over(&self) -> bool {
        self.current_board().status() == BoardStatus::Checkmate || self.draw_reason().is_some()
    }

    /// The result in PGN form: `1-0`, `0-1`, `1/2-1/2` or `*` while unfinished.
    pub fn result(&self) -> &'static str {
        let board = self.current_board();
        if board.status() == BoardStatus::Checkmate {
            match board.side_to_move() {
                Color::White => "0-1",
                Color::Black => "1-0",
            }
        } else if self.draw_reason().is_some() {
            "1/2-1/2"
        } else {
            "*"
        }
    }

    /// The game as PGN: the seven-tag roster, a FEN tag when the game did not
    /// start from the initial position, then the moves up to the current one.
    pub fn pgn(&self, white: &str, black: &str) -> String {
        let start = self.start_board();
        let result = self.result();
        let mut pgn = format!(
            "[Event \"?\"]\n[Site \"?\"]\n[Date \"????.??.??\"]\n[Round \"?\"]\n\
             [White \"{white}\"]\n[Black \"{black}\"]\n[Result \"{result}\"]\n"
        );
        if *start != Board::default() {
            // ponytail: the chess crate drops the move counters, so a FEN start
            // is numbered from move 1.
            let _ = writeln!(pgn, "[SetUp \"1\"]\n[FEN \"{start}\"]");
        }
        pgn.push('\n');

        let black_starts = start.side_to_move() == Color::Black;
        for (index, san) in self.sans[..self.current_index].iter().enumerate() {
            let ply = index + usize::from(black_starts);
            let number = ply / 2 + 1;
            if ply.is_multiple_of(2) {
                let _ = write!(pgn, "{number}. ");
            } else if index == 0 {
                let _ = write!(pgn, "{number}... ");
            }
            pgn.push_str(san);
            pgn.push(' ');
        }
        pgn.push_str(result);
        pgn
    }

    pub fn make_move(&mut self, mv: ChessMove) {
        // Truncate future history when making a new move
        self.positions.truncate(self.current_index + 1);
        self.moves.truncate(self.current_index);
        self.sans.truncate(self.current_index);

        let board = *self.current_board();
        self.sans.push(format_move_san(&mv, &board));
        self.positions.push(board.make_move_new(mv));
        self.moves.push(mv);
        self.current_index += 1;
    }

    pub fn undo(&mut self) -> bool {
        if self.current_index > 0 {
            self.current_index -= 1;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if self.current_index < self.positions.len() - 1 {
            self.current_index += 1;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current_index > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current_index < self.positions.len() - 1
    }

    pub fn move_count(&self) -> usize {
        self.current_index
    }

    pub fn get_move(&self, index: usize) -> Option<&ChessMove> {
        self.moves.get(index)
    }

    /// SAN of the move at `index`, including undone moves.
    pub fn san(&self, index: usize) -> Option<&str> {
        self.sans.get(index).map(String::as_str)
    }

    pub fn current_moves(&self) -> &[ChessMove] {
        &self.moves[..self.current_index]
    }

    /// Returns the total number of moves in the full history (including undone moves)
    pub fn total_moves(&self) -> usize {
        self.positions.len().saturating_sub(1)
    }
}

impl Default for GameHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// The SAN tokens of a movetext, in order. Comments in braces or after a
/// semicolon, parenthesised variations, `$n` glyphs, move numbers and the
/// result are dropped; `+`, `#`, `!` and `?` suffixes are trimmed.
fn movetext_moves(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut chars = text.chars();
    let flush = |current: &mut String, tokens: &mut Vec<String>| {
        if !current.is_empty() {
            tokens.push(std::mem::take(current));
        }
    };
    while let Some(c) = chars.next() {
        match c {
            '{' => {
                flush(&mut current, &mut tokens);
                chars.by_ref().find(|&c| c == '}');
            }
            ';' => {
                flush(&mut current, &mut tokens);
                chars.by_ref().find(|&c| c == '\n');
            }
            '(' => {
                flush(&mut current, &mut tokens);
                depth += 1;
            }
            ')' => {
                flush(&mut current, &mut tokens);
                depth = depth.saturating_sub(1);
            }
            c if c.is_whitespace() => flush(&mut current, &mut tokens),
            c if depth == 0 => current.push(c),
            _ => {}
        }
    }
    flush(&mut current, &mut tokens);

    tokens
        .into_iter()
        .filter_map(|token| {
            // "12." and "12..." stand alone or glue onto the move: "12.e4".
            let san = match token.rfind('.') {
                Some(dot) if token[..dot].chars().all(|c| c.is_ascii_digit() || c == '.') => {
                    &token[dot + 1..]
                }
                _ => &token,
            };
            let san = san.trim_end_matches(['+', '#', '!', '?']);
            let skip = san.is_empty()
                || san.starts_with('$')
                || matches!(san, "1-0" | "0-1" | "1/2-1/2" | "*");
            (!skip).then(|| san.to_owned())
        })
        .collect()
}

/// Neither side can force mate: bare kings, one minor piece, or bishops
/// that all stand on squares of one colour.
fn insufficient_material(board: &Board) -> bool {
    const LIGHT_SQUARES: BitBoard = BitBoard(0x55AA_55AA_55AA_55AA);

    let heavy = board.pieces(Piece::Pawn) | board.pieces(Piece::Rook) | board.pieces(Piece::Queen);
    if heavy.popcnt() > 0 {
        return false;
    }
    let knights = board.pieces(Piece::Knight).popcnt();
    let bishops = *board.pieces(Piece::Bishop);
    match (knights, bishops.popcnt()) {
        (0, 0) | (1, 0) | (0, 1) => true,
        (0, _) => (bishops & LIGHT_SQUARES) == bishops || (bishops & LIGHT_SQUARES).popcnt() == 0,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::{ChessMove, Square};
    use std::str::FromStr;

    fn create_move(from: Square, to: Square) -> ChessMove {
        ChessMove::new(from, to, None)
    }

    #[test]
    fn test_new_game_history() {
        let history = GameHistory::new();

        assert_eq!(history.move_count(), 0, "New game should have 0 moves");
        assert!(!history.can_undo(), "Cannot undo with no moves");
        assert!(!history.can_redo(), "Cannot redo with no moves");
        assert_eq!(
            history.current_board(),
            &Board::default(),
            "Should start at initial position"
        );
    }

    #[test]
    fn test_make_single_move() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);

        history.make_move(e2e4);

        assert_eq!(history.move_count(), 1, "Should have 1 move");
        assert!(history.can_undo(), "Should be able to undo");
        assert!(!history.can_redo(), "Should not be able to redo");
    }

    #[test]
    fn test_undo_single_move() {
        let mut history = GameHistory::new();
        let initial_board = *history.current_board();
        let e2e4 = create_move(Square::E2, Square::E4);

        history.make_move(e2e4);
        let result = history.undo();

        assert!(result, "Undo should succeed");
        assert_eq!(history.move_count(), 0, "Move count should be 0");
        assert!(!history.can_undo(), "Cannot undo further");
        assert!(history.can_redo(), "Should be able to redo");
        assert_eq!(
            history.current_board(),
            &initial_board,
            "Should return to initial position"
        );
    }

    #[test]
    fn test_redo_single_move() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);

        history.make_move(e2e4);
        let board_after_move = *history.current_board();
        history.undo();
        let result = history.redo();

        assert!(result, "Redo should succeed");
        assert_eq!(history.move_count(), 1, "Move count should be 1");
        assert!(history.can_undo(), "Should be able to undo");
        assert!(!history.can_redo(), "Cannot redo further");
        assert_eq!(
            history.current_board(),
            &board_after_move,
            "Should return to position after move"
        );
    }

    #[test]
    fn test_undo_at_start_fails() {
        let mut history = GameHistory::new();

        let result = history.undo();

        assert!(!result, "Undo should fail at start");
        assert_eq!(history.move_count(), 0, "Move count unchanged");
    }

    #[test]
    fn test_redo_with_no_future_fails() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);

        history.make_move(e2e4);
        let result = history.redo();

        assert!(!result, "Redo should fail with no future history");
        assert_eq!(history.move_count(), 1, "Move count unchanged");
    }

    #[test]
    fn test_multiple_moves() {
        let mut history = GameHistory::new();

        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));

        assert_eq!(history.move_count(), 2, "Should have 2 moves");
        assert!(history.can_undo(), "Should be able to undo");
        assert!(!history.can_redo(), "No future to redo");
    }

    #[test]
    fn test_multiple_undos() {
        let mut history = GameHistory::new();
        let initial_board = *history.current_board();

        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));

        assert_eq!(history.move_count(), 3, "Should have 3 moves");

        assert!(history.undo(), "First undo should succeed");
        assert_eq!(history.move_count(), 2, "Should have 2 moves");

        assert!(history.undo(), "Second undo should succeed");
        assert_eq!(history.move_count(), 1, "Should have 1 move");

        assert!(history.undo(), "Third undo should succeed");
        assert_eq!(history.move_count(), 0, "Should have 0 moves");

        assert!(!history.undo(), "Fourth undo should fail");
        assert_eq!(
            history.current_board(),
            &initial_board,
            "Should be back at start"
        );
    }

    #[test]
    fn test_multiple_redos() {
        let mut history = GameHistory::new();

        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));
        let final_board = *history.current_board();

        history.undo();
        history.undo();
        history.undo();

        assert!(history.redo(), "First redo should succeed");
        assert_eq!(history.move_count(), 1, "Should have 1 move");

        assert!(history.redo(), "Second redo should succeed");
        assert_eq!(history.move_count(), 2, "Should have 2 moves");

        assert!(history.redo(), "Third redo should succeed");
        assert_eq!(history.move_count(), 3, "Should have 3 moves");

        assert!(!history.redo(), "Fourth redo should fail");
        assert_eq!(
            history.current_board(),
            &final_board,
            "Should be back at final position"
        );
    }

    #[test]
    fn test_new_move_clears_redo_history() {
        let mut history = GameHistory::new();

        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));

        history.undo();
        assert_eq!(history.move_count(), 1, "Should have 1 move");
        assert!(history.can_redo(), "Should be able to redo");

        history.make_move(create_move(Square::D2, Square::D4));

        assert_eq!(history.move_count(), 2, "Should have 2 moves");
        assert!(history.can_undo(), "Should be able to undo");
        assert!(!history.can_redo(), "Redo should be cleared");

        assert!(!history.redo(), "Redo should fail - history was cleared");
    }

    #[test]
    fn test_branching_preserves_earlier_history() {
        let mut history = GameHistory::new();

        history.make_move(create_move(Square::E2, Square::E4));
        let board_after_e4 = *history.current_board();
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));

        history.undo(); // Undo Nf3
        history.undo(); // Undo e5

        assert_eq!(history.move_count(), 1, "Should be at 1 move");
        assert_eq!(
            history.current_board(),
            &board_after_e4,
            "Should be after e4"
        );

        history.make_move(create_move(Square::C7, Square::C5)); // Sicilian!

        assert_eq!(history.move_count(), 2, "Should have 2 moves");

        history.undo();
        assert_eq!(
            history.current_board(),
            &board_after_e4,
            "Should be back after e4"
        );

        history.undo();
        assert_eq!(history.move_count(), 0, "Should be at start");
    }

    #[test]
    fn test_threefold_repetition_needs_three_visits() {
        let mut history = GameHistory::new();
        let shuffle = [
            (Square::G1, Square::F3),
            (Square::G8, Square::F6),
            (Square::F3, Square::G1),
            (Square::F6, Square::G8),
        ];
        for (from, to) in shuffle {
            history.make_move(create_move(from, to));
        }
        assert_eq!(history.draw_reason(), None, "second visit is not a draw");

        for (from, to) in shuffle {
            history.make_move(create_move(from, to));
        }
        assert_eq!(history.draw_reason(), Some(DrawReason::ThreefoldRepetition));
        assert!(history.is_over());
    }

    #[test]
    fn test_fifty_move_rule_counts_from_the_fen_clock() {
        let mut history = GameHistory::from_fen("8/8/8/8/8/4k3/8/R3K3 w - - 99 60").unwrap();
        assert_eq!(history.halfmove_clock(), 99);
        assert_eq!(history.draw_reason(), None);

        history.make_move(create_move(Square::A1, Square::A2));
        assert_eq!(history.halfmove_clock(), 100);
        assert_eq!(history.draw_reason(), Some(DrawReason::FiftyMoveRule));

        // A capture resets the clock and the start value no longer counts.
        history.undo();
        history.make_move(create_move(Square::A1, Square::A3));
        history.make_move(create_move(Square::E3, Square::E4));
        history.make_move(create_move(Square::A3, Square::E3));
        history.make_move(create_move(Square::E4, Square::E3));
        assert_eq!(history.halfmove_clock(), 0);
        assert_eq!(
            history.draw_reason(),
            Some(DrawReason::InsufficientMaterial)
        );
    }

    #[test]
    fn test_draw_reasons_from_fen() {
        for (fen, expected) in [
            (
                "8/8/8/8/8/8/8/K1k5 w - - 0 1",
                Some(DrawReason::InsufficientMaterial),
            ),
            (
                "8/8/8/8/8/8/8/KB1k4 w - - 0 1",
                Some(DrawReason::InsufficientMaterial),
            ),
            (
                "8/8/8/8/8/8/8/KN1k4 w - - 0 1",
                Some(DrawReason::InsufficientMaterial),
            ),
            (
                "8/8/8/8/8/8/8/KB1B1k2 w - - 0 1",
                Some(DrawReason::InsufficientMaterial),
            ),
            ("8/8/8/8/8/8/8/KB2B1k1 w - - 0 1", None),
            ("8/8/8/8/8/8/8/KR1k4 b - - 0 1", None),
            ("8/8/8/8/8/8/8/KN1kn3 w - - 0 1", None),
            (
                "k7/2Q5/1K6/8/8/8/8/8 b - - 0 1",
                Some(DrawReason::Stalemate),
            ),
            ("k7/1Q6/1K6/8/8/8/8/8 b - - 0 1", None),
        ] {
            let history = GameHistory::from_fen(fen).unwrap();
            assert_eq!(history.draw_reason(), expected, "{fen}");
        }
    }

    #[test]
    fn test_pgn_movetext_and_result() {
        let mut history = GameHistory::new();
        for (from, to) in [
            (Square::F2, Square::F3),
            (Square::E7, Square::E5),
            (Square::G2, Square::G4),
            (Square::D8, Square::H4),
        ] {
            history.make_move(create_move(from, to));
        }
        let pgn = history.pgn("Human", "Stockfish");
        assert!(pgn.starts_with("[Event \"?\"]\n"), "{pgn}");
        assert!(pgn.contains("[White \"Human\"]\n[Black \"Stockfish\"]\n[Result \"0-1\"]\n\n"));
        assert!(pgn.ends_with("1. f3 e5 2. g4 Qh4# 0-1"), "{pgn}");
        assert!(!pgn.contains("[FEN"));

        history.undo();
        assert!(history.pgn("?", "?").ends_with("1. f3 e5 2. g4 *"));
    }

    #[test]
    fn test_pgn_round_trip_and_import_noise() {
        let text = "[Event \"?\"]\n[Result \"*\"]\n\n1. e4 {best by test} e5 2. Nf3 $1 (2. f4 exf4) \
                    2... Nc6 3.Bb5 a6!? ; a comment to the end of the line\n4. Ba4 *";
        let history = GameHistory::from_pgn(text).unwrap();
        assert_eq!(history.move_count(), 7);
        assert_eq!(history.san(6), Some("Ba4"));

        let again = GameHistory::from_pgn(&history.pgn("?", "?")).unwrap();
        assert_eq!(again.current_board(), history.current_board());

        assert_eq!(
            GameHistory::from_pgn("1. e4 e5 2. Ke2 Nf6 3. Kd3 Qh4").err(),
            Some(PgnError::BadMove {
                ply: 6,
                san: "Qh4".into(),
            })
        );
        assert_eq!(
            GameHistory::from_pgn("[FEN \"junk\"]\n\n1. e4").err(),
            Some(PgnError::InvalidFen)
        );
    }

    #[test]
    fn test_pgn_from_a_black_to_move_fen() {
        let fen = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1";
        let mut history = GameHistory::from_fen(fen).unwrap();
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));
        let pgn = history.pgn("?", "?");
        assert!(pgn.contains("[SetUp \"1\"]\n[FEN \""), "{pgn}");
        assert!(pgn.ends_with("1... e5 2. Nf3 *"), "{pgn}");
    }

    #[test]
    fn test_san_follows_undo_and_replacement() {
        let mut history = GameHistory::new();
        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));
        assert_eq!(history.san(2), Some("Nf3"));

        history.undo();
        assert_eq!(history.san(2), Some("Nf3"), "undone moves keep their SAN");

        history.make_move(create_move(Square::D2, Square::D4));
        assert_eq!(history.san(2), Some("d4"));
        assert_eq!(history.san(3), None);
    }

    #[test]
    fn test_get_move_by_index() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);
        let e7e5 = create_move(Square::E7, Square::E5);

        history.make_move(e2e4);
        history.make_move(e7e5);

        assert_eq!(
            history.get_move(0),
            Some(&e2e4),
            "First move should be e2e4"
        );
        assert_eq!(
            history.get_move(1),
            Some(&e7e5),
            "Second move should be e7e5"
        );
        assert_eq!(history.get_move(2), None, "Index 2 should be out of bounds");
    }

    #[test]
    fn test_get_move_after_undo() {
        let mut history = GameHistory::new();
        let e2e4 = create_move(Square::E2, Square::E4);
        let e7e5 = create_move(Square::E7, Square::E5);

        history.make_move(e2e4);
        history.make_move(e7e5);
        history.undo();

        assert_eq!(history.move_count(), 1, "Should have 1 move");
        assert_eq!(
            history.get_move(0),
            Some(&e2e4),
            "First move still accessible"
        );
        assert_eq!(
            history.get_move(1),
            Some(&e7e5),
            "Second move not accessible after undo (FIXED)"
        );
    }

    #[test]
    fn test_get_move_empty_history() {
        let history = GameHistory::new();

        assert_eq!(history.get_move(0), None, "No moves in empty history");
    }

    #[test]
    fn test_board_state_consistency_after_undo_redo() {
        let mut history = GameHistory::new();

        history.make_move(create_move(Square::E2, Square::E4));
        let board_after_e4 = *history.current_board();

        for _ in 0..5 {
            history.undo();
            history.redo();
        }

        assert_eq!(
            history.current_board(),
            &board_after_e4,
            "Board should be consistent after multiple undo/redo cycles"
        );
    }

    #[test]
    fn test_move_count_consistency() {
        let mut history = GameHistory::new();

        let moves = [
            (Square::E2, Square::E4),
            (Square::E7, Square::E5),
            (Square::G1, Square::F3),
            (Square::B8, Square::C6),
            (Square::F1, Square::C4),
        ];

        for (i, (from, to)) in moves.iter().enumerate() {
            history.make_move(create_move(*from, *to));
            assert_eq!(history.move_count(), i + 1, "Move count should increment");
        }

        for i in (0..moves.len()).rev() {
            history.undo();
            assert_eq!(history.move_count(), i, "Move count should decrement");
        }
    }

    #[test]
    fn test_can_undo_can_redo_consistency() {
        let mut history = GameHistory::new();

        assert!(!history.can_undo() && !history.can_redo());

        history.make_move(create_move(Square::E2, Square::E4));
        assert!(history.can_undo() && !history.can_redo());

        history.undo();
        assert!(!history.can_undo() && history.can_redo());

        history.redo();
        assert!(history.can_undo() && !history.can_redo());
    }

    #[test]
    fn test_complex_undo_redo_sequence() {
        let mut history = GameHistory::new();

        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));
        history.make_move(create_move(Square::B8, Square::C6));
        assert_eq!(history.move_count(), 4);

        history.undo();
        history.undo();
        assert_eq!(history.move_count(), 2);

        history.redo();
        assert_eq!(history.move_count(), 3);

        history.make_move(create_move(Square::F8, Square::C5));
        assert_eq!(history.move_count(), 4);
        assert!(!history.can_redo(), "Redo should be cleared");

        history.undo();
        history.undo();
        history.undo();
        history.undo();
        assert_eq!(history.move_count(), 0);
        assert_eq!(history.current_board(), &Board::default());
    }

    #[test]
    fn test_undo_redo_with_promotions() {
        let fen = "k7/4P3/8/8/8/8/8/K7 w - - 0 1";
        let board = Board::from_str(fen).expect("Valid fen");

        let mut history = GameHistory::new();
        history.positions[0] = board;

        let promotion = ChessMove::new(Square::E7, Square::E8, Some(chess::Piece::Queen));

        history.make_move(promotion);
        assert_eq!(history.move_count(), 1);

        if let Some(stored_move) = history.get_move(0) {
            assert_eq!(stored_move.get_promotion(), Some(chess::Piece::Queen));
        } else {
            panic!("Move should be retrievable");
        }

        history.undo();
        assert_eq!(history.move_count(), 0);

        history.redo();
        assert_eq!(history.move_count(), 1);

        if let Some(stored_move) = history.get_move(0) {
            assert_eq!(stored_move.get_promotion(), Some(chess::Piece::Queen));
        }
    }

    #[test]
    fn test_alternating_undo_redo() {
        let mut history = GameHistory::new();
        history.make_move(create_move(Square::E2, Square::E4));

        for _ in 0..10 {
            assert!(history.undo());
            assert!(history.redo());
        }

        assert_eq!(history.move_count(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_max_reasonable_history() {
        let mut history = GameHistory::new();

        let moves = vec![
            (Square::E2, Square::E4),
            (Square::E7, Square::E5),
            (Square::G1, Square::F3),
            (Square::B8, Square::C6),
            (Square::F1, Square::C4),
            (Square::F8, Square::C5),
            (Square::D2, Square::D3),
            (Square::G8, Square::F6),
            (Square::B1, Square::C3),
            (Square::D7, Square::D6),
        ];

        for (from, to) in &moves {
            history.make_move(create_move(*from, *to));
        }

        assert_eq!(history.move_count(), 10, "Should have 10 moves");

        for i in (0..10).rev() {
            assert!(history.undo(), "Undo {} should succeed", i);
            assert_eq!(history.move_count(), i, "Move count should be {}", i);
        }

        assert_eq!(history.move_count(), 0);
        assert_eq!(history.current_board(), &Board::default());

        for i in 0..10 {
            assert!(history.redo(), "Redo {} should succeed", i);
            assert_eq!(
                history.move_count(),
                i + 1,
                "Move count should be {}",
                i + 1
            );
        }

        assert_eq!(history.move_count(), 10);
    }

    #[test]
    fn test_current_moves_respects_undo() {
        let mut history = GameHistory::new();

        history.make_move(create_move(Square::E2, Square::E4));
        history.make_move(create_move(Square::E7, Square::E5));
        history.make_move(create_move(Square::G1, Square::F3));

        assert_eq!(history.current_moves().len(), 3);

        history.undo();
        assert_eq!(
            history.current_moves().len(),
            2,
            "Should only show 2 moves after undo"
        );

        history.undo();
        assert_eq!(history.current_moves().len(), 1, "Should only show 1 move");

        history.redo();
        assert_eq!(
            history.current_moves().len(),
            2,
            "Should show 2 moves after redo"
        );
    }
}

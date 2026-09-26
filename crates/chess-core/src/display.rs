//! The board and the state of the game as text, for the CLI.

use cozy_chess::{Board, Color, File, Rank, Square};

use crate::GameHistory;
use crate::moves::is_checkmate;

/// The board with White at the bottom, pieces as letters, empty squares as
/// dots.
pub fn board(board: &Board) -> String {
    let mut out = String::from("\n");
    for rank in (0..8).rev() {
        out.push_str(&format!("{} ", rank + 1));
        for file in 0..8 {
            let square = Square::new(File::index(file), Rank::index(rank));
            let glyph = match (board.piece_on(square), board.color_on(square)) {
                (Some(piece), Some(Color::White)) => piece.to_string().to_ascii_uppercase(),
                (Some(piece), Some(Color::Black)) => piece.to_string(),
                _ if (rank + file).is_multiple_of(2) => ".".to_owned(),
                _ => "\u{b7}".to_owned(),
            };
            out.push_str(&format!(" {glyph} "));
        }
        out.push('\n');
    }
    out.push_str("   a  b  c  d  e  f  g  h\n");
    out
}

/// Whose move it is, or how the game ended.
pub fn status(history: &GameHistory) -> String {
    let board = history.current_board();
    let name = |color: Color| match color {
        Color::White => "White",
        Color::Black => "Black",
    };
    let to_move = board.side_to_move();
    if is_checkmate(board) {
        format!("Checkmate! {} wins!", name(!to_move))
    } else if let Some(reason) = history.draw_reason() {
        format!("Draw by {}.", reason.describe())
    } else if !board.checkers().is_empty() {
        format!("{} is in check!", name(to_move))
    } else {
        format!("{} to move", name(to_move))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn board_prints_letters_and_status_names_the_end() {
        let text = board(&Board::default());
        assert!(text.contains(" r  n  b  q  k  b  n  r "), "{text}");
        assert!(text.ends_with("   a  b  c  d  e  f  g  h\n"));

        let mut history = GameHistory::new();
        assert_eq!(status(&history), "White to move");
        history = GameHistory::from_pgn("1. f3 e5 2. g4 Qh4#").unwrap();
        assert_eq!(status(&history), "Checkmate! Black wins!");
        history = GameHistory::from_fen("7k/5Q2/6K1/8/8/8/8/8 b - - 0 1").unwrap();
        assert_eq!(status(&history), "Draw by stalemate.");
    }
}

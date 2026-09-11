//! Opening names from the Lichess table in `data/openings.tsv`, found by
//! the longest row a game's moves begin with.

use chess::Board;
use std::collections::HashMap;
use std::sync::OnceLock;

use crate::GameHistory;

/// A row of the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Opening {
    /// ECO code, `A00` to `E99`.
    pub eco: &'static str,
    pub name: &'static str,
}

struct Table {
    /// Rows keyed by their moves in SAN without numbers, e.g. `e4 c5 Nf3`.
    rows: HashMap<String, Opening>,
    /// Plies in the longest row; no game matches beyond it.
    longest: usize,
}

fn table() -> &'static Table {
    static TABLE: OnceLock<Table> = OnceLock::new();
    TABLE.get_or_init(|| {
        let mut longest = 0;
        let rows = include_str!("../data/openings.tsv")
            .lines()
            .skip(1)
            .filter_map(|line| {
                let mut fields = line.split('\t');
                let (eco, name, pgn) = (fields.next()?, fields.next()?, fields.next()?);
                let moves: Vec<&str> = pgn
                    .split_whitespace()
                    .filter(|word| !word.ends_with('.'))
                    .collect();
                longest = longest.max(moves.len());
                Some((moves.join(" "), Opening { eco, name }))
            })
            .collect();
        Table { rows, longest }
    })
}

/// The longest opening the game begins with, for games from the standard
/// start position.
pub fn of(history: &GameHistory) -> Option<Opening> {
    if *history.start_board() != Board::default() {
        return None;
    }
    let table = table();
    let mut key = String::new();
    let mut found = None;
    for ply in 0..history.move_count().min(table.longest) {
        if ply > 0 {
            key.push(' ');
        }
        key.push_str(history.san(ply)?);
        if let Some(&opening) = table.rows.get(&key) {
            found = Some(opening);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn longest_prefix_wins_and_survives_leaving_the_book() {
        let mut history = GameHistory::from_pgn(
            "1. e4 c5 2. Nf3 d6 3. d4 cxd4 4. Nxd4 Nf6 5. Nc3 a6 6. h3 h6 7. a3 a5",
        )
        .unwrap();
        let name_at = |history: &mut GameHistory, plies| {
            while history.move_count() > plies {
                history.undo();
            }
            of(history).map(|opening| (opening.eco, opening.name))
        };
        // 6. h3 is the Adams Attack; the two moves after it are off the book.
        let (eco, name) = name_at(&mut history, 14).unwrap();
        assert_eq!(eco, "B90");
        assert!(
            name.starts_with("Sicilian Defense: Najdorf Variation"),
            "{name}"
        );
        assert_eq!(name_at(&mut history, 6), Some(("B50", "Sicilian Defense")));
        assert_eq!(
            name_at(&mut history, 4),
            Some(("B50", "Sicilian Defense: Modern Variations"))
        );
        assert_eq!(name_at(&mut history, 3), Some(("B27", "Sicilian Defense")));
        assert_eq!(name_at(&mut history, 1), Some(("B00", "King's Pawn Game")));
        assert_eq!(name_at(&mut history, 0), None);
    }

    #[test]
    fn positions_set_up_from_a_fen_have_no_opening() {
        let board =
            Board::from_str("rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1").unwrap();
        let mut history = GameHistory::from_board(board);
        history.make_move(chess::ChessMove::new(
            chess::Square::C7,
            chess::Square::C5,
            None,
        ));
        assert_eq!(of(&history), None);
    }

    /// Every row plays through our SAN parser and formats back to itself,
    /// so the table and `notation` agree on every move in it.
    #[test]
    fn every_row_is_found_from_its_own_moves() {
        for line in include_str!("../data/openings.tsv").lines().skip(1) {
            let mut fields = line.split('\t');
            let (eco, name, pgn) = (
                fields.next().unwrap(),
                fields.next().unwrap(),
                fields.next().unwrap(),
            );
            let history =
                GameHistory::from_pgn(pgn).unwrap_or_else(|e| panic!("{eco} {name}: {e}"));
            let found = of(&history).unwrap_or_else(|| panic!("{eco} {name} not found"));
            assert_eq!((found.eco, found.name), (eco, name), "{pgn}");
        }
    }
}

//! The game on top of cozy-chess: `GameHistory` with undo, redo, SAN and
//! PGN, the notation readers, the opening table, and a text board for the
//! CLI.

pub mod display;
pub mod game;
pub mod moves;
pub mod notation;
pub mod openings;

pub use game::{DrawReason, GameHistory, PgnError, PgnTags};

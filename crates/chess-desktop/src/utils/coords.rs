//! Coordinate transformation utilities.
//!
//! Handles conversion between screen positions and chess board squares.

use chess::{File, Rank, Square as ChessSquare};
use eframe::egui::{Pos2, Rect};

/// Convert screen position to chess square
pub fn get_square_from_pos(
    pos: Pos2,
    board_rect: Rect,
    square_size: f32,
    board_flip: bool,
) -> Option<ChessSquare> {
    if !board_rect.contains(pos) {
        return None;
    }

    let relative_pos = pos - board_rect.min;
    let file = (relative_pos.x / square_size) as usize;
    let rank = (relative_pos.y / square_size) as usize;

    if file >= 8 || rank >= 8 {
        return None;
    }

    let display_rank = if board_flip { rank } else { 7 - rank };
    let display_file = if board_flip { 7 - file } else { file };

    Some(ChessSquare::make_square(
        Rank::from_index(display_rank),
        File::from_index(display_file),
    ))
}

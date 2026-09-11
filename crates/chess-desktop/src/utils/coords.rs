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

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Vec2;

    #[test]
    fn every_square_maps_back_from_its_centre() {
        let rect = Rect::from_min_size(Pos2::new(10.0, 20.0), Vec2::splat(400.0));
        for flip in [false, true] {
            for square in chess::ALL_SQUARES {
                let (rank, file) = (square.get_rank().to_index(), square.get_file().to_index());
                let row = if flip { rank } else { 7 - rank };
                let col = if flip { 7 - file } else { file };
                let centre = rect.min + Vec2::new(col as f32 + 0.5, row as f32 + 0.5) * 50.0;
                assert_eq!(get_square_from_pos(centre, rect, 50.0, flip), Some(square));
            }
        }
        assert_eq!(
            get_square_from_pos(Pos2::new(9.0, 20.0), rect, 50.0, false),
            None
        );
        assert_eq!(
            get_square_from_pos(Pos2::new(410.5, 20.0), rect, 50.0, false),
            None
        );
    }
}

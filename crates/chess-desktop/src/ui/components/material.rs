//! Material count and captured pieces display.
//!
//! Shows captured pieces in either Lichess or Chess.com style.

use chess::{Board, ChessMove, Color as ChessColor, Piece};
use eframe::egui::{Sense, Ui, Vec2};
use std::collections::HashMap;

use crate::app::state::{CapturedPiecesStyle, ChessApp};
use crate::ui::pieces;
use crate::ui::theme::Theme;

/// How many of each piece a side has taken.
type Captured = HashMap<Piece, i32>;

/// Draw material count and captured pieces
pub fn draw_material_count(app: &ChessApp, ui: &mut Ui) {
    ui.label("Material:");

    let (white_material, black_material, white_captured, black_captured) = calculate_material(app);

    let material_diff = white_material - black_material;

    match app.captured_display_style {
        CapturedPiecesStyle::Lichess => {
            draw_lichess_style(ui, material_diff, &white_captured, &black_captured);
        }
        CapturedPiecesStyle::ChessCom => {
            draw_chesscom_style(
                ui,
                &app.theme,
                material_diff,
                &white_captured,
                &black_captured,
            );
        }
    }
}

/// Material on the board per side, and the pieces each side has captured.
/// Captures are read from the moves played, so a promoted pawn is not
/// mistaken for a captured one.
fn calculate_material(app: &ChessApp) -> (i32, i32, Captured, Captured) {
    let board = app.board();
    let material = |color| {
        chess::ALL_PIECES
            .iter()
            .map(|&piece| {
                let count = (board.pieces(piece) & board.color_combined(color)).popcnt();
                piece_value(piece) * count as i32
            })
            .sum::<i32>()
    };

    let mut white_captured = HashMap::new();
    let mut black_captured = HashMap::new();
    for (before, mv) in app.game_history.played() {
        let Some(taken) = captured_piece(before, mv) else {
            continue;
        };
        let by = match before.side_to_move() {
            ChessColor::White => &mut white_captured,
            ChessColor::Black => &mut black_captured,
        };
        *by.entry(taken).or_insert(0) += 1;
    }

    (
        material(ChessColor::White),
        material(ChessColor::Black),
        white_captured,
        black_captured,
    )
}

/// The piece `mv` takes on `board`, counting en passant.
fn captured_piece(board: &Board, mv: ChessMove) -> Option<Piece> {
    let (from, to) = (mv.get_source(), mv.get_dest());
    board.piece_on(to).or_else(|| {
        let is_pawn = board.piece_on(from) == Some(Piece::Pawn);
        (is_pawn && from.get_file() != to.get_file()).then_some(Piece::Pawn)
    })
}

/// Get material value of piece type
fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 1,
        Piece::Knight | Piece::Bishop => 3,
        Piece::Rook => 5,
        Piece::Queen => 9,
        Piece::King => 0,
    }
}

/// Draw captured pieces in Lichess style (show only advantage)
fn draw_lichess_style(
    ui: &mut Ui,
    material_diff: i32,
    white_captured: &Captured,
    black_captured: &Captured,
) {
    ui.heading("Captured Pieces");

    if material_diff == 0 {
        ui.label("Equal material");
    } else if material_diff > 0 {
        ui.horizontal(|ui| {
            ui.label("White is up:");
            draw_captured_row(ui, white_captured, ChessColor::Black);
            ui.label(format!("(+{})", material_diff));
        });
    } else {
        ui.horizontal(|ui| {
            ui.label("Black is up:");
            draw_captured_row(ui, black_captured, ChessColor::White);
            ui.label(format!("(+{})", -material_diff));
        });
    }
}

/// Draw captured pieces in Chess.com style (show all pieces)
fn draw_chesscom_style(
    ui: &mut Ui,
    theme: &Theme,
    material_diff: i32,
    white_captured: &Captured,
    black_captured: &Captured,
) {
    ui.heading("Captured Pieces");

    ui.horizontal(|ui| {
        ui.label("White:");
        if !draw_captured_row(ui, white_captured, ChessColor::Black) {
            ui.label("none");
        }
    });
    ui.horizontal(|ui| {
        ui.label("Black:");
        if !draw_captured_row(ui, black_captured, ChessColor::White) {
            ui.label("none");
        }
    });

    ui.separator();
    if material_diff > 0 {
        ui.colored_label(theme.text_primary, format!("White +{}", material_diff));
    } else if material_diff < 0 {
        ui.colored_label(theme.text_secondary, format!("Black +{}", -material_diff));
    } else {
        ui.label("Equal material");
    }
}

/// The pieces one side has taken, drawn small in the victim's colour,
/// heaviest first. Returns false when there are none.
fn draw_captured_row(ui: &mut Ui, captured: &Captured, color: ChessColor) -> bool {
    const SIZE: f32 = 22.0;
    let order = [
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Knight,
        Piece::Pawn,
    ];
    let mut any = false;
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        for piece in order {
            let count = captured.get(&piece).copied().unwrap_or(0).max(0);
            for _ in 0..count {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(SIZE * 0.8, SIZE), Sense::hover());
                pieces::draw(ui.painter(), rect.center(), SIZE, piece, color);
                any = true;
            }
        }
    });
    any
}

#[cfg(test)]
mod tests {
    use super::*;
    use chess::Square;
    use std::str::FromStr;

    #[test]
    fn test_captured_piece_sees_en_passant() {
        let board =
            Board::from_str("rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3")
                .unwrap();
        let en_passant = ChessMove::new(Square::E5, Square::F6, None);
        assert_eq!(captured_piece(&board, en_passant), Some(Piece::Pawn));

        let push = ChessMove::new(Square::E5, Square::E6, None);
        assert_eq!(captured_piece(&board, push), None);
    }

    #[test]
    fn test_promotion_is_not_a_capture() {
        let mut app = ChessApp::headless();
        let board = Board::from_str("4k3/P7/8/8/8/8/8/4K3 w - - 0 1").unwrap();
        app.game_history = chess_core::GameHistory::from_board(board);
        let mv = app.parse_uci_move("a7a8q", &board).unwrap();
        app.play_move(mv);

        let (white, black, white_captured, black_captured) = calculate_material(&app);
        assert_eq!((white, black), (9, 0));
        assert!(white_captured.is_empty());
        assert!(black_captured.is_empty());
    }
}

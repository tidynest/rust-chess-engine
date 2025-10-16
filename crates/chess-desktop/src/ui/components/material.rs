//! Material count and captured pieces display.
//!
//! Shows captured pieces in either Lichess or Chess.com style.

use eframe::egui::{self, Ui, Color32};
use std::collections::HashMap;
use chess_core::{Color, GameState, PieceType};

use crate::app::state::{ChessApp, CapturedPiecesStyle};

/// Draw material count and captured pieces
pub fn draw_material_count(app: &ChessApp, ui: &mut Ui) {
    ui.label("Material:");

    let (white_material, black_material, white_captured, black_captured) =
        calculate_material(app);

    let material_diff = white_material - black_material;

    match app.captured_display_style {
        CapturedPiecesStyle::Lichess => {
            draw_lichess_style(ui, material_diff, &white_captured, &black_captured);
        }
        CapturedPiecesStyle::ChessCom => {
            draw_chesscom_style(ui, material_diff, &white_captured, &black_captured);
        }
    }
}

/// Calculate material counts and captured pieces
fn calculate_material(app: &ChessApp) -> (i32, i32, HashMap<PieceType, i32>, HashMap<PieceType, i32>) {
    let mut white_material = 0;
    let mut black_material = 0;
    let mut white_pieces = create_starting_pieces();
    let mut black_pieces = create_starting_pieces();

    for rank in 0..8 {
        for file in 0..8 {
            if let Some(sq) = chess_core::Square::new(file, rank) {
                if let Some(piece) = app.engine.piece_at(sq) {
                    let value = piece_value(piece.piece_type);

                    if piece.color == Color::White {
                        white_material += value;
                    } else {
                        black_material += value;
                    }

                    let pieces_map = if piece.color == Color::White {
                        &mut white_pieces
                    } else {
                        &mut black_pieces
                    };
                    if let Some(count) = pieces_map.get_mut(&piece.piece_type) {
                        if *count > 0 {
                            *count -= 1;
                        }
                    }
                }
            }
        }
    }

    (white_material, black_material, black_pieces, white_pieces)
}

/// Create starting piece counts
fn create_starting_pieces() -> HashMap<PieceType, i32> {
    let mut pieces = HashMap::new();
    pieces.insert(PieceType::Pawn, 8);
    pieces.insert(PieceType::Knight, 2);
    pieces.insert(PieceType::Bishop, 2);
    pieces.insert(PieceType::Rook, 2);
    pieces.insert(PieceType::Queen, 1);
    pieces
}

/// Get material value of piece type
fn piece_value(piece_type: PieceType) -> i32 {
    match piece_type {
        PieceType::Pawn => 1,
        PieceType::Knight | PieceType::Bishop => 3,
        PieceType::Rook => 5,
        PieceType::Queen => 9,
        PieceType::King => 0,
    }
}

/// Draw captured pieces in Lichess style (show only advantage)
fn draw_lichess_style(
    ui: &mut Ui,
    material_diff: i32,
    white_captured: &HashMap<PieceType, i32>,
    black_captured: &HashMap<PieceType, i32>,
) {
    ui.heading("Captured Pieces");

    if material_diff == 0 {
        ui.label("Equal material");
    } else if material_diff > 0 {
        ui.horizontal(|ui| {
            ui.label("⚪ White is up:");
            let captured_str = format_captured_pieces(white_captured, true);
            ui.label(captured_str);
            ui.label(format!("(+{})", material_diff));
        });
    } else {
        ui.horizontal(|ui| {
            ui.label("⚫ Black is up:");
            let captured_str = format_captured_pieces(black_captured, false);
            ui.label(captured_str);
            ui.label(format!("(+{})", -material_diff));
        });
    }
}

/// Draw captured pieces in Chess.com style (show all pieces)
fn draw_chesscom_style(
    ui: &mut Ui,
    material_diff: i32,
    white_captured: &HashMap<PieceType, i32>,
    black_captured: &HashMap<PieceType, i32>,
) {
    ui.heading("Captured Pieces");

    ui.horizontal(|ui| {
        ui.label("⚪ White:");
        let captured_str = format_captured_pieces(white_captured, true);
        if captured_str.is_empty() {
            ui.label("none");
        } else {
            ui.label(captured_str);
        }
    });

    ui.horizontal(|ui| {
        ui.label("⚫ Black:");
        let captured_str = format_captured_pieces(black_captured, false);
        if captured_str.is_empty() {
            ui.label("none");
        } else {
            ui.label(captured_str);
        }
    });

    ui.separator();
    if material_diff > 0 {
        ui.colored_label(Color32::from_rgb(200, 200, 200), format!("White +{}", material_diff));
    } else if material_diff < 0 {
        ui.colored_label(Color32::from_rgb(100, 100, 100), format!("Black +{}", -material_diff));
    } else {
        ui.label("Equal material");
    }
}

/// Format captured pieces as Unicode string
fn format_captured_pieces(captured: &HashMap<PieceType, i32>, is_white: bool) -> String {
    let mut pieces = Vec::new();

    let piece_order = [
        PieceType::Queen,
        PieceType::Rook,
        PieceType::Bishop,
        PieceType::Knight,
        PieceType::Pawn,
    ];

    for piece_type in piece_order.iter() {
        if let Some(&count) = captured.get(piece_type) {
            if count > 0 {
                let piece_char = get_piece_unicode(*piece_type, is_white);
                for _ in 0..count {
                    pieces.push(piece_char);
                }
            }
        }
    }

    pieces.iter().collect()
}

/// Get Unicode character for piece
fn get_piece_unicode(piece_type: PieceType, is_white: bool) -> char {
    if is_white {
        match piece_type {
            PieceType::Queen => '♛',
            PieceType::Rook => '♜',
            PieceType::Bishop => '♝',
            PieceType::Knight => '♞',
            PieceType::Pawn => '♟',
            PieceType::King => '♚',
        }
    } else {
        match piece_type {
            PieceType::Queen => '♕',
            PieceType::Rook => '♖',
            PieceType::Bishop => '♗',
            PieceType::Knight => '♘',
            PieceType::Pawn => '♙',
            PieceType::King => '♔',
        }
    }
}
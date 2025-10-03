//! Display utilities for chess positions

use crate::{GameState, Square, PieceType, Color};

/// Display mode for the chess board
pub enum DisplayMode {
    Unicode,
    Ascii,
    Compact,
}

/// ASCII representation of the chess board with configurable display mode
pub fn display_board_with_mode(game: &dyn GameState, mode: DisplayMode) -> String {
    match mode {
        DisplayMode::Compact => display_board_compact(game),
        _ => display_board_grid(game, mode),
    }
}

/// Grid-based display (original implementation)
fn display_board_grid(game: &dyn GameState, mode: DisplayMode) -> String {
    let mut output = String::new();

    // Unicode chess pieces
    const WHITE_UNICODE: [char; 6] = ['♔', '♕', '♖', '♗', '♘', '♙'];
    const BLACK_UNICODE: [char; 6] = ['♚', '♛', '♜', '♝', '♞', '♟'];

    // ASCII chess pieces
    const WHITE_ASCII: [char; 6] = ['K', 'Q', 'R', 'B', 'N', 'P'];
    const BLACK_ASCII: [char; 6] = ['k', 'q', 'r', 'b', 'n', 'p'];

    output.push_str("  ┌───────────────────────────────┐\n");

    for rank in (0..8).rev() {
        output.push_str(&format!("{} │", rank + 1));

        for file in 0..8 {
            let square = Square::new(file, rank).unwrap();
            let piece_char = match game.piece_at(square) {
                Some(piece) => {
                    let (white_pieces, black_pieces) = match mode {
                        DisplayMode::Unicode => (&WHITE_UNICODE, &BLACK_UNICODE),
                        DisplayMode::Ascii => (&WHITE_ASCII, &BLACK_ASCII),
                        _ => (&WHITE_ASCII, &BLACK_ASCII),
                    };

                    let pieces = match piece.color {
                        Color::White => white_pieces,
                        Color::Black => black_pieces,
                    };

                    match piece.piece_type {
                        PieceType::King => pieces[0],
                        PieceType::Queen => pieces[1],
                        PieceType::Rook => pieces[2],
                        PieceType::Bishop => pieces[3],
                        PieceType::Knight => pieces[4],
                        PieceType::Pawn => pieces[5],
                    }
                }
                None => {
                    if (rank + file) % 2 == 0 {
                        '·'
                    } else {
                        ' '
                    }
                }
            };

            output.push_str(&format!(" {} ", piece_char));

            if file < 7 {
                output.push('│');
            }
        }

        output.push_str("│\n");

        if rank > 0 {
            output.push_str("  ├───┼───┼───┼───┼───┼───┼───┼───┤\n");
        }
    }

    output.push_str("  └───────────────────────────────┘\n");
    output.push_str("    a   b   c   d   e   f   g   h\n");

    output
}

/// Compact display without grid lines
fn display_board_compact(game: &dyn GameState) -> String {
    let mut output = String::new();

    // ASCII chess pieces
    const WHITE_ASCII: [char; 6] = ['K', 'Q', 'R', 'B', 'N', 'P'];
    const BLACK_ASCII: [char; 6] = ['k', 'q', 'r', 'b', 'n', 'p'];

    output.push_str("\n");

    for rank in (0..8).rev() {
        output.push_str(&format!("{} ", rank + 1));

        for file in 0..8 {
            let square = Square::new(file, rank).unwrap();
            let piece_char = match game.piece_at(square) {
                Some(piece) => {
                    let pieces = match piece.color {
                        Color::White => &WHITE_ASCII,
                        Color::Black => &BLACK_ASCII,
                    };

                    match piece.piece_type {
                        PieceType::King => pieces[0],
                        PieceType::Queen => pieces[1],
                        PieceType::Rook => pieces[2],
                        PieceType::Bishop => pieces[3],
                        PieceType::Knight => pieces[4],
                        PieceType::Pawn => pieces[5],
                    }
                }
                None => {
                    if (rank + file) % 2 == 0 {
                        '.'
                    } else {
                        '·'
                    }
                }
            };

            output.push_str(&format!(" {} ", piece_char));
        }

        output.push_str("\n");
    }

    output.push_str("   a  b  c  d  e  f  g  h\n");

    output
}

/// Default display board function using compact mode
pub fn display_board(game: &dyn GameState) -> String {
    display_board_with_mode(game, DisplayMode::Compact)
}

/// Display game status
pub fn display_status(game: &dyn GameState) -> String {
    let mut status = String::new();

    let side = match game.side_to_move() {
        Color::White => "White",
        Color::Black => "Black",
    };

    if game.is_checkmate() {
        let winner = match game.side_to_move() {
            Color::White => "Black",
            Color::Black => "White",
        };
        status.push_str(&format!("Checkmate! {} wins!", winner));
    } else if game.is_stalemate() {
        status.push_str("Stalemate! Game is a draw.");
    } else if game.is_check() {
        status.push_str(&format!("{} is in check!", side));
    } else {
        status.push_str(&format!("{} to move", side));
    }

    status
}
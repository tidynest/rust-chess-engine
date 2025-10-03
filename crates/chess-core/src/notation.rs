//! Algebraic notation parsing utilities

use crate::{Move, PieceType, Square};

/// Parse algebraic notation (e.g., "e2e4", "e7e8q")
pub fn parse_algebraic(s: &str) -> Option<Move> {
    let bytes = s.as_bytes();
    if bytes.len() < 4 || bytes.len() > 5 {
        return None;
    }

    // Parse source square
    let from_file = (bytes[0] as char).to_digit(18)? as u8 -10;  // a=0, b=1, etc.
    let from_rank = (bytes[1] as char).to_digit(10)? as u8 -1;  // 1=0, 2=1, etc.

    // Parse destination square
    let to_file = (bytes[2] as char).to_digit(18)? as u8 -10;
    let to_rank = (bytes[3] as char).to_digit(10)? as u8 -1;

    let from = Square::new(from_file, from_rank)?;
    let to = Square::new(to_file, to_rank)?;

    // Parse promotion if present
    let promotion = if bytes.len() == 5 {
        match bytes[4] {
            b'q' | b'Q' => Some(PieceType::Queen),
            b'r' | b'R' => Some(PieceType::Rook),
            b'b' | b'B' => Some(PieceType::Bishop),
            b'n' | b'N' => Some(PieceType::Knight),
            _ => return None,
        }
    } else {
        None
    };

    Some(Move { from, to, promotion })
}

/// Format a move as algebraic notation
pub fn to_algebraic(mv: &Move) -> String {
    let mut result = format!(
        "{}{}",
        mv.from.to_algebraic(),
        mv.to.to_algebraic()
    );

    if let Some(promo) = mv.promotion {
        result.push(match promo {
            PieceType::Queen => 'q',
            PieceType::Rook => 'r',
            PieceType::Bishop => 'b',
            PieceType::Knight => 'n',
            _ => return result,
        });
    }

    result
}
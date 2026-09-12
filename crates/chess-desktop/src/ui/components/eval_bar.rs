//! Evaluation bar component.
//!
//! A vertical bar split between the sides: White's share grows from White's
//! edge of the board as the evaluation moves in White's favour.

use chess_engine::Score;
use eframe::egui::{self, Pos2, Rect, Ui, Vec2};

use crate::app::state::ChessApp;

/// Text for a score seen from White's side.
pub fn label(score: Score) -> String {
    match score {
        Score::Cp(cp) => format!("{:+.1}", cp as f32 / 100.0),
        Score::Mate(moves) if moves < 0 => format!("-M{}", -moves),
        Score::Mate(moves) => format!("M{moves}"),
    }
}

/// White's share of the bar, 0 to 1. The curve is the one Lichess uses, so
/// one pawn reads as about 59% and ten pawns as 97%.
fn white_share(score: Score) -> f32 {
    match score {
        Score::Cp(cp) => 1.0 / (1.0 + (-0.003_682_08 * cp as f32).exp()),
        Score::Mate(moves) if moves < 0 => 0.0,
        Score::Mate(_) => 1.0,
    }
}

/// Draw the evaluation bar
pub fn draw(app: &ChessApp, ui: &mut Ui) {
    let theme = &app.theme;
    let (response, painter) = ui.allocate_painter(
        Vec2::new(ui.available_width(), ui.available_height()),
        egui::Sense::hover(),
    );
    let rect = response.rect;
    let radius = theme.border_radius;

    painter.rect_filled(rect, radius, theme.text_secondary);
    let inner = rect.shrink(2.0);
    painter.rect_filled(inner, radius, theme.eval_black);

    // White sits at the bottom unless the board is flipped.
    let white_at_bottom = !app.board_flip;
    let share = app.engine_evaluation().map_or(0.5, white_share);
    let white_height = share * inner.height();
    let white_rect = if white_at_bottom {
        Rect::from_min_max(
            Pos2::new(inner.left(), inner.bottom() - white_height),
            inner.max,
        )
    } else {
        Rect::from_min_max(
            inner.min,
            Pos2::new(inner.right(), inner.top() + white_height),
        )
    };
    painter.rect_filled(white_rect, radius, theme.eval_white);

    let center_y = inner.center().y;
    painter.line_segment(
        [
            Pos2::new(inner.left(), center_y),
            Pos2::new(inner.right(), center_y),
        ],
        (1.0, theme.text_secondary),
    );

    let Some(score) = app.engine_evaluation() else {
        return;
    };

    // The label sits at the leading side's edge, on that side's colour.
    let white_leads = share >= 0.5;
    let at_bottom = white_leads == white_at_bottom;
    let y = if at_bottom {
        rect.bottom() - 10.0
    } else {
        rect.top() + 10.0
    };
    let color = if white_leads {
        theme.eval_black
    } else {
        theme.eval_white
    };
    painter.text(
        Pos2::new(rect.center().x, y),
        egui::Align2::CENTER_CENTER,
        label(score),
        egui::FontId::proportional(theme.font_size_xs),
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_and_shares() {
        assert_eq!(label(Score::Cp(40)), "+0.4");
        assert_eq!(label(Score::Cp(-120)), "-1.2");
        assert_eq!(label(Score::Mate(3)), "M3");
        assert_eq!(label(Score::Mate(-2)), "-M2");

        assert!((white_share(Score::Cp(0)) - 0.5).abs() < 1e-6);
        assert!((white_share(Score::Cp(100)) - 0.59).abs() < 0.01);
        assert!(white_share(Score::Cp(1000)) > 0.96);
        assert_eq!(white_share(Score::Mate(1)), 1.0);
        assert_eq!(white_share(Score::Mate(-1)), 0.0);
    }
}

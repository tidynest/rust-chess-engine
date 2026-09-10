//! Evaluation bar component.
//!
//! Displays engine evaluation as a vertical bar. The leading side's fill grows
//! from the centre line toward the opponent's edge of the board.

use chess_engine::Score;
use eframe::egui::{self, Color32, Pos2, Rect, Ui, Vec2};

use crate::app::state::ChessApp;

/// Text for a score seen from White's side.
pub fn label(score: Score) -> String {
    match score {
        Score::Cp(cp) => format!("{:+.1}", cp as f32 / 100.0),
        Score::Mate(moves) if moves < 0 => format!("-M{}", -moves),
        Score::Mate(moves) => format!("M{moves}"),
    }
}

/// White's lead as a share of half the bar, from -1 to 1. Ten pawns fill it.
fn fraction(score: Score) -> f32 {
    match score {
        Score::Cp(cp) => (cp as f32 / 1000.0).clamp(-1.0, 1.0),
        Score::Mate(moves) if moves < 0 => -1.0,
        Score::Mate(_) => 1.0,
    }
}

/// Draw the evaluation bar
pub fn draw(app: &ChessApp, ui: &mut Ui) {
    let bar_width = ui.available_width();
    let bar_height = ui.available_height();

    let (response, painter) =
        ui.allocate_painter(Vec2::new(bar_width, bar_height), egui::Sense::hover());

    let rect = response.rect;

    draw_background(&painter, rect);

    let inner_rect = rect.shrink(2.0);
    let center_y = inner_rect.center().y;

    draw_center_line(&painter, inner_rect, center_y);

    match app.engine_evaluation {
        Some(score) => {
            let fraction = fraction(score);
            // White's fill points at Black's edge, which moves when the board flips.
            let fill_up = (fraction >= 0.0) != app.board_flip;
            draw_evaluation_fill(&painter, inner_rect, center_y, fraction, fill_up);
            draw_evaluation_text(&painter, rect, &label(score), fill_up);
        }
        None => draw_no_evaluation(&painter, rect, app.board_flip),
    }
}

/// Draw background border
fn draw_background(painter: &egui::Painter, rect: Rect) {
    painter.rect_filled(rect, 4.0, Color32::from_rgb(40, 40, 40));
}

/// Draw center line at 0.0 evaluation
fn draw_center_line(painter: &egui::Painter, inner_rect: Rect, center_y: f32) {
    painter.rect_filled(inner_rect, 2.0, Color32::from_rgb(60, 60, 60));

    painter.line_segment(
        [
            Pos2::new(inner_rect.left(), center_y),
            Pos2::new(inner_rect.right(), center_y),
        ],
        (1.0, Color32::from_rgb(200, 200, 200)),
    );
}

/// Draw evaluation fill bar
fn draw_evaluation_fill(
    painter: &egui::Painter,
    inner_rect: Rect,
    center_y: f32,
    fraction: f32,
    fill_up: bool,
) {
    let bar_fill_height = fraction.abs() * (inner_rect.height() / 2.0);

    let fill_rect = if fill_up {
        Rect::from_min_max(
            Pos2::new(inner_rect.left(), center_y - bar_fill_height),
            Pos2::new(inner_rect.right(), center_y),
        )
    } else {
        Rect::from_min_max(
            Pos2::new(inner_rect.left(), center_y),
            Pos2::new(inner_rect.right(), center_y + bar_fill_height),
        )
    };

    painter.rect_filled(fill_rect, 2.0, calculate_fill_color(fraction));
}

/// Fill colour: light for White, dark for Black, stronger as the lead grows.
fn calculate_fill_color(fraction: f32) -> Color32 {
    let intensity = fraction.abs();
    let level = if fraction >= 0.0 {
        180.0 + 75.0 * intensity
    } else {
        80.0 - 60.0 * intensity
    };
    Color32::from_gray(level as u8)
}

/// Draw the score in the empty half, so it never sits on the fill.
fn draw_evaluation_text(painter: &egui::Painter, rect: Rect, text: &str, fill_up: bool) {
    let y = if fill_up {
        rect.bottom() - 10.0
    } else {
        rect.top() + 10.0
    };
    painter.text(
        Pos2::new(rect.center().x, y),
        egui::Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
}

/// Draw placeholder when no evaluation available
fn draw_no_evaluation(painter: &egui::Painter, rect: Rect, board_flip: bool) {
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "...",
        egui::FontId::proportional(14.0),
        Color32::GRAY,
    );

    let (top, bottom) = if board_flip { ("W", "B") } else { ("B", "W") };
    for (text, y) in [(top, rect.top() + 8.0), (bottom, rect.bottom() - 8.0)] {
        painter.text(
            Pos2::new(rect.center().x, y),
            egui::Align2::CENTER_CENTER,
            text,
            egui::FontId::proportional(10.0),
            Color32::LIGHT_GRAY,
        );
    }
}

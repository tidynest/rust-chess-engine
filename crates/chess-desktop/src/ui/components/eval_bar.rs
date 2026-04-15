//! Evaluation bar component.
//!
//! Displays engine evaluation as a vertical bar.

use eframe::egui::{self, Color32, Pos2, Rect, Ui, Vec2};

use crate::app::state::ChessApp;

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

    if let Some(eval) = app.engine_evaluation {
        draw_evaluation_fill(&painter, inner_rect, center_y, eval);
        draw_evaluation_text(&painter, rect, center_y, eval);
    } else {
        draw_no_evaluation(&painter, rect);
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
fn draw_evaluation_fill(painter: &egui::Painter, inner_rect: Rect, center_y: f32, eval: f32) {
    let clamped_eval = eval.clamp(-10.0, 10.0);
    let bar_fill_height = (clamped_eval.abs() / 10.0) * (inner_rect.height() / 2.0);

    let fill_rect = if clamped_eval >= 0.0 {
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

    let fill_colour = calculate_fill_color(clamped_eval);
    painter.rect_filled(fill_rect, 2.0, fill_colour);
}

/// Calculate fill color based on evaluation
fn calculate_fill_color(eval: f32) -> Color32 {
    if eval >= 0.0 {
        let intensity = (eval / 10.0).min(1.0);
        Color32::from_rgb(
            (180.0 + 75.0 * intensity) as u8,
            (180.0 + 75.0 * intensity) as u8,
            (180.0 + 75.0 * intensity) as u8,
        )
    } else {
        let intensity = (eval.abs() / 10.0).min(1.0);
        Color32::from_rgb(
            (80.0 - 60.0 * intensity) as u8,
            (80.0 - 60.0 * intensity) as u8,
            (80.0 - 60.0 * intensity) as u8,
        )
    }
}

/// Draw evaluation text
fn draw_evaluation_text(painter: &egui::Painter, rect: Rect, center_y: f32, eval: f32) {
    let eval_text = if eval.abs() >= 100.0 {
        if eval > 0.0 { "M+" } else { "M-" }.to_string()
    } else if eval > 0.0 {
        format!("+{:.1}", eval)
    } else {
        format!("{:.1}", eval)
    };

    let clamped_eval = eval.clamp(-10.0, 10.0);
    let bar_fill_height = (clamped_eval.abs() / 10.0) * (rect.height() / 2.0);

    let text_pos = if eval >= 0.0 {
        Pos2::new(
            rect.center().x,
            (center_y - bar_fill_height - 15.0).max(rect.top() + 10.0),
        )
    } else {
        Pos2::new(
            rect.center().x,
            (center_y + bar_fill_height + 15.0).min(rect.bottom() - 10.0),
        )
    };

    painter.text(
        text_pos,
        egui::Align2::CENTER_CENTER,
        eval_text,
        egui::FontId::proportional(12.0),
        Color32::WHITE,
    );
}

/// Draw placeholder when no evaluation available
fn draw_no_evaluation(painter: &egui::Painter, rect: Rect) {
    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        "...",
        egui::FontId::proportional(14.0),
        Color32::GRAY,
    );

    painter.text(
        Pos2::new(rect.center().x, rect.top() + 8.0),
        egui::Align2::CENTER_CENTER,
        "W",
        egui::FontId::proportional(10.0),
        Color32::LIGHT_GRAY,
    );

    painter.text(
        Pos2::new(rect.center().x, rect.bottom() - 8.0),
        egui::Align2::CENTER_CENTER,
        "B",
        egui::FontId::proportional(10.0),
        Color32::LIGHT_GRAY,
    );
}

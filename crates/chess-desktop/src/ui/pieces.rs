//! Piece drawings built from shapes, so they look the same on every machine
//! and scale with the board. Proportions follow the cburnett set's 45-unit
//! box. Pieces without a drawing yet fall back to a font glyph.

use chess::{Color as ChessColor, Piece};
use eframe::egui::{self, Color32, Painter, Pos2, Shape, Stroke, Vec2};

/// Side of the box every piece is designed in.
const BOX: f32 = 45.0;
/// Outline width in box units.
const OUTLINE: f32 = 1.5;

/// Draw `piece` centred on `center` in a square of `square_size` pixels.
pub fn draw(painter: &Painter, center: Pos2, square_size: f32, piece: Piece, color: ChessColor) {
    let canvas = Canvas {
        origin: center - Vec2::splat(square_size / 2.0),
        scale: square_size / BOX,
    };
    let (fill, outline) = match color {
        ChessColor::White => (Color32::WHITE, Color32::BLACK),
        ChessColor::Black => (Color32::from_gray(18), Color32::from_gray(70)),
    };

    match piece {
        Piece::Pawn => pawn(&canvas).paint(painter, fill, outline),
        _ => glyph(painter, center, square_size * 0.8, piece, color),
    }
}

/// Maps box coordinates onto the screen.
struct Canvas {
    origin: Pos2,
    scale: f32,
}

impl Canvas {
    fn at(&self, x: f32, y: f32) -> Pos2 {
        self.origin + Vec2::new(x, y) * self.scale
    }

    /// Points along a cubic Bézier from `p0` to `p3`, excluding `p0`.
    fn cubic(&self, p0: (f32, f32), p1: (f32, f32), p2: (f32, f32), p3: (f32, f32)) -> Vec<Pos2> {
        const STEPS: usize = 10;
        (1..=STEPS)
            .map(|step| {
                let t = step as f32 / STEPS as f32;
                let u = 1.0 - t;
                let weight = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
                let x = weight[0] * p0.0 + weight[1] * p1.0 + weight[2] * p2.0 + weight[3] * p3.0;
                let y = weight[0] * p0.1 + weight[1] * p1.1 + weight[2] * p2.1 + weight[3] * p3.1;
                self.at(x, y)
            })
            .collect()
    }
}

/// A piece as a union of convex parts. The outline is stroked at double
/// width under all the fills, so only the silhouette's edge shows and the
/// seams between parts vanish.
struct Figure {
    circles: Vec<(Pos2, f32)>,
    polygons: Vec<Vec<Pos2>>,
    outline_width: f32,
}

impl Figure {
    fn paint(&self, painter: &Painter, fill: Color32, outline: Color32) {
        let stroke = Stroke::new(self.outline_width * 2.0, outline);
        for &(center, radius) in &self.circles {
            painter.circle_stroke(center, radius, stroke);
        }
        for polygon in &self.polygons {
            painter.add(Shape::closed_line(polygon.clone(), stroke));
        }
        for &(center, radius) in &self.circles {
            painter.circle_filled(center, radius, fill);
        }
        for polygon in &self.polygons {
            painter.add(Shape::convex_polygon(polygon.clone(), fill, Stroke::NONE));
        }
    }
}

/// Head, collar ball and a bell-shaped skirt on a flat base.
fn pawn(canvas: &Canvas) -> Figure {
    let mut skirt = vec![canvas.at(18.41, 26.03)];
    skirt.extend(canvas.cubic((18.41, 26.03), (15.41, 27.09), (11.0, 31.58), (11.0, 39.5)));
    skirt.push(canvas.at(34.0, 39.5));
    skirt.extend(canvas.cubic((34.0, 39.5), (34.0, 31.58), (29.59, 27.09), (26.59, 26.03)));

    Figure {
        circles: vec![
            (canvas.at(22.5, 13.0), 4.0 * canvas.scale),
            (canvas.at(22.5, 21.0), 6.5 * canvas.scale),
        ],
        polygons: vec![skirt],
        outline_width: OUTLINE * canvas.scale,
    }
}

/// Font glyph with a contrasting rim, for pieces not yet drawn from shapes.
fn glyph(painter: &Painter, pos: Pos2, size: f32, piece: Piece, color: ChessColor) {
    let piece_char = match piece {
        Piece::King => '♚',
        Piece::Queen => '♛',
        Piece::Rook => '♜',
        Piece::Bishop => '♝',
        Piece::Knight => '♞',
        Piece::Pawn => '♟',
    };
    let (fill, rim) = match color {
        ChessColor::White => (Color32::WHITE, Color32::from_gray(30)),
        ChessColor::Black => (Color32::from_gray(20), Color32::from_gray(200)),
    };
    let font_id = egui::FontId::proportional(size * 0.9);

    painter.text(
        pos + Vec2::splat(1.0),
        egui::Align2::CENTER_CENTER,
        piece_char,
        font_id.clone(),
        Color32::from_black_alpha(100),
    );
    for dx in [-0.5, 0.0, 0.5] {
        for dy in [-0.5, 0.0, 0.5] {
            if dx != 0.0 || dy != 0.0 {
                painter.text(
                    pos + Vec2::new(dx, dy),
                    egui::Align2::CENTER_CENTER,
                    piece_char,
                    font_id.clone(),
                    rim,
                );
            }
        }
    }
    painter.text(pos, egui::Align2::CENTER_CENTER, piece_char, font_id, fill);
}

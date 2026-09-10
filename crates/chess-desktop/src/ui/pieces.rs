//! Piece drawings built from shapes, so they look the same on every machine
//! and scale with the board. Each piece is one silhouette polygon in a
//! 45-unit box in the Staunton style, triangulated once, plus a few detail
//! lines drawn on top.

use chess::{Color as ChessColor, Piece};
use eframe::egui::epaint::{Mesh, TextureId, Vertex, WHITE_UV};
use eframe::egui::{Color32, Painter, Pos2, Shape, Stroke, Vec2};
use std::sync::LazyLock;

/// Side of the box every piece is designed in.
const BOX: f32 = 45.0;
/// Outline width in box units.
const OUTLINE: f32 = 1.4;

type Point = (f32, f32);

/// Draw `piece` centred on `center` in a square of `square_size` pixels.
pub fn draw(painter: &Painter, center: Pos2, square_size: f32, piece: Piece, color: ChessColor) {
    let design = &DESIGNS[piece.to_index()];
    let scale = square_size / BOX;
    let origin = center - Vec2::splat(square_size / 2.0);
    let map = |(x, y): Point| origin + Vec2::new(x, y) * scale;

    let (fill, outline, detail) = match color {
        ChessColor::White => (Color32::WHITE, Color32::BLACK, Color32::BLACK),
        ChessColor::Black => (
            Color32::from_gray(18),
            Color32::from_gray(70),
            Color32::from_gray(170),
        ),
    };

    let points: Vec<Pos2> = design.outline.iter().map(|&p| map(p)).collect();
    let mesh = Mesh {
        indices: design.triangles.clone(),
        vertices: points
            .iter()
            .map(|&pos| Vertex {
                pos,
                uv: WHITE_UV,
                color: fill,
            })
            .collect(),
        texture_id: TextureId::default(),
    };
    painter.add(Shape::mesh(mesh));

    let stroke = Stroke::new(OUTLINE * scale, outline);
    painter.add(Shape::closed_line(points, stroke));

    let stroke = Stroke::new(OUTLINE * scale, detail);
    for line in &design.lines {
        painter.add(Shape::line(line.iter().map(|&p| map(p)).collect(), stroke));
    }
    for &(center, radius) in &design.dots {
        painter.circle_filled(map(center), radius * scale, detail);
    }
}

/// A piece in box coordinates, triangulated.
struct Design {
    outline: Vec<Point>,
    triangles: Vec<u32>,
    lines: Vec<Vec<Point>>,
    dots: Vec<(Point, f32)>,
}

impl Design {
    fn new(outline: Vec<Point>, lines: Vec<Vec<Point>>, dots: Vec<(Point, f32)>) -> Self {
        let outline = dedup(outline);
        let triangles = triangulate(&outline);
        Self {
            outline,
            triangles,
            lines,
            dots,
        }
    }
}

/// Indexed by `Piece::to_index()`: pawn, knight, bishop, rook, queen, king.
static DESIGNS: LazyLock<[Design; 6]> =
    LazyLock::new(|| [pawn(), knight(), bishop(), rook(), queen(), king()]);

/// Points on a circle from `from` to `to` degrees inclusive, clockwise on
/// screen as the angle grows.
fn arc(cx: f32, cy: f32, r: f32, from: f32, to: f32) -> Vec<Point> {
    let steps = ((to - from).abs() / 12.0).ceil().max(2.0) as usize;
    (0..=steps)
        .map(|step| {
            let angle = (from + (to - from) * step as f32 / steps as f32).to_radians();
            (cx + r * angle.cos(), cy + r * angle.sin())
        })
        .collect()
}

/// Points along a cubic Bézier from `p0` to `p3`, excluding `p0`.
fn cubic(p0: Point, p1: Point, p2: Point, p3: Point) -> Vec<Point> {
    const STEPS: usize = 8;
    (1..=STEPS)
        .map(|step| {
            let t = step as f32 / STEPS as f32;
            let u = 1.0 - t;
            let w = [u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t];
            (
                w[0] * p0.0 + w[1] * p1.0 + w[2] * p2.0 + w[3] * p3.0,
                w[0] * p0.1 + w[1] * p1.1 + w[2] * p2.1 + w[3] * p3.1,
            )
        })
        .collect()
}

/// Head, collar ball and a bell-shaped skirt on a flat base. The neck is
/// where the two balls cross.
fn pawn() -> Design {
    let mut outline = arc(22.5, 13.0, 4.0, 144.0, 396.0);
    outline.extend(arc(22.5, 21.0, 6.5, -60.0, 51.0));
    outline.extend(cubic(
        (26.6, 26.0),
        (30.0, 27.5),
        (34.0, 32.0),
        (34.0, 39.5),
    ));
    outline.push((11.0, 39.5));
    outline.extend(cubic(
        (11.0, 39.5),
        (11.0, 32.0),
        (15.0, 27.5),
        (18.4, 26.0),
    ));
    outline.extend(arc(22.5, 21.0, 6.5, 129.0, 240.0));
    Design::new(outline, vec![], vec![])
}

/// Battlements, a tapered tower and two base steps.
fn rook() -> Design {
    let outline = vec![
        (11.0, 8.0),
        (15.0, 8.0),
        (15.0, 11.0),
        (20.0, 11.0),
        (20.0, 8.0),
        (25.0, 8.0),
        (25.0, 11.0),
        (30.0, 11.0),
        (30.0, 8.0),
        (34.0, 8.0),
        (34.0, 14.0),
        (31.0, 17.0),
        (31.0, 32.0),
        (33.0, 32.0),
        (33.0, 36.0),
        (36.0, 36.0),
        (36.0, 39.5),
        (9.0, 39.5),
        (9.0, 36.0),
        (12.0, 36.0),
        (12.0, 32.0),
        (14.0, 32.0),
        (14.0, 17.0),
        (11.0, 14.0),
    ];
    let lines = vec![
        vec![(11.0, 14.0), (34.0, 14.0)],
        vec![(14.0, 17.0), (31.0, 17.0)],
        vec![(12.0, 32.0), (33.0, 32.0)],
        vec![(9.0, 36.0), (36.0, 36.0)],
    ];
    Design::new(outline, lines, vec![])
}

/// Knob, mitre with a slit, neck, collar and a wavy base.
fn bishop() -> Design {
    let mut outline = arc(22.5, 8.0, 2.5, 130.0, 410.0);
    outline.extend([
        (25.5, 11.5),
        (28.5, 13.5),
        (31.0, 16.5),
        (32.5, 20.0),
        (32.0, 23.5),
        (30.0, 26.0),
        (27.5, 27.2),
        (28.0, 29.0),
        (30.0, 31.5),
        (30.5, 33.5),
        (31.0, 34.5),
        (35.5, 35.2),
        (39.0, 37.5),
        (38.0, 39.0),
        (36.0, 39.6),
        (30.0, 39.0),
        (22.5, 39.6),
        (15.0, 39.0),
        (9.0, 39.6),
        (7.0, 39.0),
        (6.0, 37.5),
        (9.5, 35.2),
        (14.0, 34.5),
        (14.5, 33.5),
        (15.0, 31.5),
        (17.0, 29.0),
        (17.5, 27.2),
        (15.0, 26.0),
        (13.0, 23.5),
        (12.5, 20.0),
        (14.0, 16.5),
        (16.5, 13.5),
        (19.5, 11.5),
    ]);
    let lines = vec![
        vec![(22.5, 15.0), (22.5, 21.5)],
        vec![(20.0, 18.0), (25.0, 18.0)],
        vec![(17.5, 27.2), (27.5, 27.2)],
        vec![(15.0, 31.5), (30.0, 31.5)],
        vec![(9.5, 35.2), (22.5, 34.0), (35.5, 35.2)],
    ];
    Design::new(outline, lines, vec![])
}

/// Five balls on a spiked crown, a waisted body and a stepped base.
fn queen() -> Design {
    let mut outline = arc(6.2, 12.2, 2.4, 160.0, 390.0);
    outline.push((14.0, 23.0));
    outline.extend(arc(14.2, 9.2, 2.4, 175.0, 380.0));
    outline.push((19.5, 22.5));
    outline.extend(arc(22.5, 8.0, 2.4, 180.0, 360.0));
    outline.push((25.5, 22.5));
    outline.extend(arc(30.8, 9.2, 2.4, 160.0, 365.0));
    outline.push((31.0, 23.0));
    outline.extend(arc(38.8, 12.2, 2.4, 150.0, 380.0));
    outline.extend([
        (36.0, 26.0),
        (33.5, 30.0),
        (34.0, 33.5),
        (36.0, 36.0),
        (37.0, 38.0),
        (36.0, 39.5),
        (9.0, 39.5),
        (8.0, 38.0),
        (9.0, 36.0),
        (11.0, 33.5),
        (11.5, 30.0),
        (9.0, 26.0),
    ]);
    let lines = vec![
        vec![
            (9.0, 26.0),
            (15.0, 25.0),
            (22.5, 24.5),
            (30.0, 25.0),
            (36.0, 26.0),
        ],
        vec![(11.5, 30.0), (22.5, 29.0), (33.5, 30.0)],
        vec![(11.0, 33.5), (22.5, 32.5), (34.0, 33.5)],
        vec![(9.0, 36.0), (22.5, 35.0), (36.0, 36.0)],
    ];
    Design::new(outline, lines, vec![])
}

/// Cross, a teardrop over two lobes, and a base. The drop's lower edge is
/// drawn as a line since it sits in front of the lobes.
fn king() -> Design {
    let outline = vec![
        (21.3, 5.0),
        (23.7, 5.0),
        (23.7, 7.2),
        (25.5, 7.2),
        (25.5, 9.5),
        (23.7, 9.5),
        (23.7, 12.0),
        (24.8, 13.5),
        (25.6, 16.0),
        (25.3, 19.0),
        (24.8, 20.0),
        (27.0, 17.2),
        (30.5, 15.3),
        (34.0, 15.2),
        (37.0, 17.0),
        (38.8, 20.5),
        (38.0, 25.0),
        (35.0, 28.5),
        (32.5, 30.5),
        (32.5, 36.5),
        (29.0, 38.8),
        (22.5, 39.8),
        (16.0, 38.8),
        (12.5, 36.5),
        (12.5, 30.5),
        (10.0, 28.5),
        (7.0, 25.0),
        (6.2, 20.5),
        (8.0, 17.0),
        (11.0, 15.2),
        (14.5, 15.3),
        (18.0, 17.2),
        (20.2, 20.0),
        (19.7, 19.0),
        (19.4, 16.0),
        (20.2, 13.5),
        (21.3, 12.0),
        (21.3, 9.5),
        (19.5, 9.5),
        (19.5, 7.2),
        (21.3, 7.2),
    ];
    let lines = vec![
        vec![
            (24.8, 20.0),
            (23.8, 22.5),
            (22.5, 24.5),
            (21.2, 22.5),
            (20.2, 20.0),
        ],
        vec![(12.5, 30.5), (22.5, 27.5), (32.5, 30.5)],
        vec![(12.5, 33.5), (22.5, 30.5), (32.5, 33.5)],
        vec![(12.5, 36.5), (22.5, 33.5), (32.5, 36.5)],
    ];
    Design::new(outline, lines, vec![])
}

/// Head in profile facing left: ear, arched neck, chest, jaw, muzzle.
fn knight() -> Design {
    let outline = vec![
        (22.0, 10.0),
        (26.0, 10.5),
        (30.0, 12.0),
        (33.5, 15.0),
        (36.5, 20.0),
        (38.0, 27.0),
        (38.5, 34.0),
        (38.5, 39.5),
        (15.0, 39.5),
        (15.5, 34.0),
        (17.0, 30.0),
        (20.5, 27.0),
        (23.0, 23.5),
        (23.5, 19.5),
        (23.0, 18.0),
        (21.5, 21.0),
        (18.5, 24.0),
        (16.0, 26.5),
        (13.5, 28.5),
        (11.5, 30.5),
        (10.0, 31.0),
        (9.0, 30.0),
        (8.5, 28.5),
        (6.5, 28.0),
        (5.8, 26.0),
        (6.5, 24.0),
        (9.0, 19.0),
        (12.0, 14.5),
        (13.5, 12.5),
        (15.0, 10.5),
        (17.0, 9.8),
        (18.5, 10.3),
        (20.5, 10.0),
    ];
    let lines = vec![vec![(23.0, 18.0), (25.5, 14.0), (26.0, 12.0)]];
    let dots = vec![((9.8, 24.5), 0.8), ((7.8, 27.0), 0.6)];
    Design::new(outline, lines, dots)
}

/// Drop consecutive points that coincide, which arc joins produce.
fn dedup(points: Vec<Point>) -> Vec<Point> {
    let mut out: Vec<Point> = Vec::with_capacity(points.len());
    for p in points {
        let same = |q: &Point| (q.0 - p.0).abs() < 1e-3 && (q.1 - p.1).abs() < 1e-3;
        if out.last().is_none_or(|last| !same(last)) {
            out.push(p);
        }
    }
    if out.len() > 1
        && out
            .first()
            .zip(out.last())
            .is_some_and(|(a, b)| (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3)
    {
        out.pop();
    }
    out
}

fn cross(a: Point, b: Point, c: Point) -> f32 {
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

/// Inside or on the edge: a vertex sitting on an ear's diagonal must block
/// the ear, or the triangle would cover ground outside the polygon.
fn inside(p: Point, a: Point, b: Point, c: Point) -> bool {
    let (d1, d2, d3) = (cross(a, b, p), cross(b, c, p), cross(c, a, p));
    let eps = 1e-4;
    (d1 >= -eps && d2 >= -eps && d3 >= -eps) || (d1 <= eps && d2 <= eps && d3 <= eps)
}

/// Ear clipping for a simple polygon; ponytail: O(n^3), runs once per piece.
fn triangulate(points: &[Point]) -> Vec<u32> {
    let n = points.len();
    let mut remaining: Vec<usize> = (0..n).collect();
    let area: f32 = (0..n)
        .map(|i| {
            let (p, q) = (points[i], points[(i + 1) % n]);
            p.0 * q.1 - q.0 * p.1
        })
        .sum();
    if area < 0.0 {
        remaining.reverse();
    }

    let mut triangles = Vec::with_capacity((n.saturating_sub(2)) * 3);
    'clip: while remaining.len() > 3 {
        let m = remaining.len();
        for i in 0..m {
            let (a, b, c) = (
                remaining[(i + m - 1) % m],
                remaining[i],
                remaining[(i + 1) % m],
            );
            let convex = cross(points[a], points[b], points[c]) > -1e-6;
            let blocked = remaining.iter().any(|&p| {
                p != a && p != b && p != c && inside(points[p], points[a], points[b], points[c])
            });
            if convex && !blocked {
                triangles.extend([a as u32, b as u32, c as u32]);
                remaining.remove(i);
                continue 'clip;
            }
        }
        // ponytail: no ear found means a self-crossing outline; leave a hole
        // rather than loop forever.
        break;
    }
    if remaining.len() == 3 {
        triangles.extend(remaining.iter().map(|&i| i as u32));
    }
    triangles
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_piece_triangulates_completely() {
        for (index, design) in DESIGNS.iter().enumerate() {
            let expected = (design.outline.len() - 2) * 3;
            assert_eq!(design.triangles.len(), expected, "piece {index}");
        }
    }

    #[test]
    fn concave_polygon_area_is_preserved() {
        // An L shape: two squares' worth of area.
        let l = vec![
            (0.0, 0.0),
            (2.0, 0.0),
            (2.0, 1.0),
            (1.0, 1.0),
            (1.0, 2.0),
            (0.0, 2.0),
        ];
        let tris = triangulate(&l);
        let area: f32 = tris
            .chunks(3)
            .map(|t| cross(l[t[0] as usize], l[t[1] as usize], l[t[2] as usize]).abs() / 2.0)
            .sum();
        assert!((area - 3.0).abs() < 1e-5, "{area}");
    }
}

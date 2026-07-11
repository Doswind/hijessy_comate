//! 轻量矢量图标：用 egui 画笔绘制线性图标，避免引入图标字体依赖。
use eframe::egui::{Color32, Pos2, Rect, Shape, Stroke, Vec2};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    Cursor,
    Rect,
    Ellipse,
    Line,
    Arrow,
    Pencil,
    Text,
    Number,
    Mosaic,
    Undo,
    Save,
    Confirm,
    Cancel,
    FullScreen,
    Window,
    Ocr,
    Record,
    LongCapture,
}

fn at(r: Rect, fx: f32, fy: f32) -> Pos2 {
    Pos2::new(r.min.x + fx * r.width(), r.min.y + fy * r.height())
}

fn line(painter: &eframe::egui::Painter, r: Rect, a: (f32, f32), b: (f32, f32), s: Stroke) {
    painter.line_segment([at(r, a.0, a.1), at(r, b.0, b.1)], s);
}

fn poly(painter: &eframe::egui::Painter, r: Rect, pts: &[(f32, f32)], s: Stroke) {
    let p: Vec<Pos2> = pts.iter().map(|(x, y)| at(r, *x, *y)).collect();
    painter.add(Shape::line(p, s));
}

/// 在方形区域 `r` 内绘制图标。
pub fn draw(painter: &eframe::egui::Painter, icon: Icon, area: Rect, color: Color32, width: f32) {
    // 内缩为正方形绘制区。
    let side = area.width().min(area.height()) * 0.62;
    let r = Rect::from_center_size(area.center(), Vec2::splat(side));
    let s = Stroke::new(width, color);

    match icon {
        Icon::Cursor => {
            poly(
                painter,
                r,
                &[
                    (0.15, 0.05),
                    (0.15, 0.85),
                    (0.38, 0.62),
                    (0.55, 0.98),
                    (0.68, 0.9),
                    (0.5, 0.55),
                    (0.82, 0.5),
                    (0.15, 0.05),
                ],
                s,
            );
        }
        Icon::Rect => {
            painter.rect_stroke(
                Rect::from_min_max(at(r, 0.08, 0.15), at(r, 0.92, 0.85)),
                eframe::egui::CornerRadius::same(2),
                s,
                eframe::egui::StrokeKind::Inside,
            );
        }
        Icon::Ellipse => {
            circle(painter, r, 0.5, 0.5, 0.42, s);
        }
        Icon::Line => {
            line(painter, r, (0.20, 0.80), (0.80, 0.20), s);
        }
        Icon::Arrow => {
            line(painter, r, (0.18, 0.78), (0.78, 0.22), s);
            line(painter, r, (0.50, 0.22), (0.78, 0.22), s);
            line(painter, r, (0.78, 0.22), (0.78, 0.50), s);
        }
        Icon::Pencil => {
            line(painter, r, (0.25, 0.72), (0.68, 0.29), s);
            line(painter, r, (0.33, 0.80), (0.76, 0.37), s);
            line(painter, r, (0.25, 0.72), (0.33, 0.80), s);
            line(painter, r, (0.68, 0.29), (0.76, 0.37), s);
        }
        Icon::Text => {
            line(painter, r, (0.18, 0.15), (0.82, 0.15), s);
            line(painter, r, (0.5, 0.15), (0.5, 0.85), s);
        }
        Icon::Number => {
            circle(painter, r, 0.5, 0.5, 0.44, s);
            line(painter, r, (0.44, 0.36), (0.54, 0.3), s);
            line(painter, r, (0.54, 0.3), (0.54, 0.72), s);
        }
        Icon::Mosaic => {
            let cells = [
                (0.12, 0.12, true),
                (0.42, 0.12, false),
                (0.72, 0.12, true),
                (0.12, 0.42, false),
                (0.42, 0.42, true),
                (0.72, 0.42, false),
                (0.12, 0.72, true),
                (0.42, 0.72, false),
                (0.72, 0.72, true),
            ];
            for (x, y, fill) in cells {
                let cell = Rect::from_min_max(at(r, x, y), at(r, x + 0.16, y + 0.16));
                if fill {
                    painter.rect_filled(cell, eframe::egui::CornerRadius::same(0), color);
                } else {
                    painter.rect_stroke(
                        cell,
                        eframe::egui::CornerRadius::same(0),
                        Stroke::new(width * 0.7, color),
                        eframe::egui::StrokeKind::Inside,
                    );
                }
            }
        }
        Icon::Undo => {
            arc(painter, r, 0.5, 0.55, 0.34, 40.0, 300.0, s);
            line(painter, r, (0.2, 0.32), (0.16, 0.6), s);
            line(painter, r, (0.2, 0.32), (0.44, 0.34), s);
        }
        Icon::Save => {
            line(painter, r, (0.5, 0.1), (0.5, 0.68), s);
            line(painter, r, (0.5, 0.68), (0.3, 0.46), s);
            line(painter, r, (0.5, 0.68), (0.7, 0.46), s);
            line(painter, r, (0.15, 0.88), (0.85, 0.88), s);
        }
        Icon::Confirm => {
            poly(painter, r, &[(0.15, 0.55), (0.42, 0.82), (0.88, 0.2)], s);
        }
        Icon::Cancel => {
            line(painter, r, (0.18, 0.18), (0.82, 0.82), s);
            line(painter, r, (0.82, 0.18), (0.18, 0.82), s);
        }
        Icon::FullScreen => {
            // 四角括号
            line(painter, r, (0.1, 0.28), (0.1, 0.1), s);
            line(painter, r, (0.1, 0.1), (0.28, 0.1), s);
            line(painter, r, (0.72, 0.1), (0.9, 0.1), s);
            line(painter, r, (0.9, 0.1), (0.9, 0.28), s);
            line(painter, r, (0.9, 0.72), (0.9, 0.9), s);
            line(painter, r, (0.9, 0.9), (0.72, 0.9), s);
            line(painter, r, (0.28, 0.9), (0.1, 0.9), s);
            line(painter, r, (0.1, 0.9), (0.1, 0.72), s);
        }
        Icon::Window => {
            painter.rect_stroke(
                r.shrink(r.width() * 0.20),
                1.0,
                s,
                eframe::egui::StrokeKind::Inside,
            );
            line(painter, r, (0.22, 0.35), (0.78, 0.35), s);
        }
        Icon::Ocr => {
            painter.rect_stroke(
                r.shrink(r.width() * 0.16),
                2.0,
                s,
                eframe::egui::StrokeKind::Inside,
            );
            painter.text(
                r.center(),
                eframe::egui::Align2::CENTER_CENTER,
                "OCR",
                eframe::egui::FontId::proportional(r.width() * 0.22),
                color,
            );
        }
        Icon::Record => {
            let red = Color32::from_rgb(225, 48, 48);
            painter.circle_stroke(r.center(), r.width() * 0.34, Stroke::new(2.0, red));
            painter.circle_filled(r.center(), r.width() * 0.18, red);
        }
        Icon::LongCapture => {
            let inner = r.shrink(r.width() * 0.28);
            painter.rect_stroke(inner, 1.0, s, eframe::egui::StrokeKind::Inside);
            line(painter, r, (0.50, 0.12), (0.50, 0.82), s);
            line(painter, r, (0.34, 0.66), (0.50, 0.82), s);
            line(painter, r, (0.66, 0.66), (0.50, 0.82), s);
        }
    }
}

fn circle(painter: &eframe::egui::Painter, r: Rect, cx: f32, cy: f32, rad: f32, s: Stroke) {
    let center = at(r, cx, cy);
    let radius = rad * r.width();
    painter.circle_stroke(center, radius, s);
}

#[allow(clippy::too_many_arguments)]
fn arc(
    painter: &eframe::egui::Painter,
    r: Rect,
    cx: f32,
    cy: f32,
    rad: f32,
    start_deg: f32,
    sweep_deg: f32,
    s: Stroke,
) {
    let center = at(r, cx, cy);
    let radius = rad * r.width();
    let steps = 24;
    let mut pts = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = (start_deg + sweep_deg * i as f32 / steps as f32).to_radians();
        pts.push(Pos2::new(
            center.x + radius * t.cos(),
            center.y + radius * t.sin(),
        ));
    }
    painter.add(Shape::line(pts, s));
}

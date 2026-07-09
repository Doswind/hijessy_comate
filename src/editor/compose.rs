//! 标注合成器：将标注烧录到底图，产出最终输出图。
//!
//! 采用轻量的手写像素绘制（不引入重型图像库），文字/序号用 ab_glyph 栅格化。
use ab_glyph::{Font, FontVec, PxScale, ScaleFont, point};
use eframe::egui::{Color32, Pos2, Rect};
use image::{Rgba, RgbaImage};

use crate::editor::model::{Annotation, ArrowStyle, Style};

/// 将标注合成到底图副本上。
pub fn compose(base: &RgbaImage, annotations: &[Annotation], font: Option<&FontVec>) -> RgbaImage {
    let mut img = base.clone();
    for a in annotations {
        match a {
            Annotation::Rect { rect, style } => draw_rect(&mut img, *rect, style),
            Annotation::Ellipse { rect, style } => draw_ellipse(&mut img, *rect, style),
            Annotation::Arrow {
                from,
                to,
                arrow_style,
                style,
            } => draw_arrow(&mut img, *from, *to, *arrow_style, style),
            Annotation::Text {
                pos,
                content,
                style,
            } => {
                if let Some(f) = font {
                    draw_text(
                        &mut img,
                        *pos,
                        content,
                        style.font_size,
                        to_rgba(style.color),
                        f,
                    );
                }
            }
            Annotation::Number { pos, index, style } => {
                draw_number(&mut img, *pos, *index, style, font)
            }
            Annotation::Mosaic { rect } => draw_mosaic(&mut img, *rect),
        }
    }
    img
}

fn to_rgba(c: Color32) -> Rgba<u8> {
    Rgba([c.r(), c.g(), c.b(), c.a()])
}

fn blend(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>, coverage: f32) {
    if x < 0 || y < 0 || x >= img.width() as i32 || y >= img.height() as i32 {
        return;
    }
    let a = (coverage * (color[3] as f32 / 255.0)).clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    let px = img.get_pixel_mut(x as u32, y as u32);
    for i in 0..3 {
        px[i] = (color[i] as f32 * a + px[i] as f32 * (1.0 - a)).round() as u8;
    }
    px[3] = 255;
}

fn fill_disk(img: &mut RgbaImage, cx: f32, cy: f32, r: f32, color: Rgba<u8>) {
    let r = r.max(0.5);
    let r2 = r * r;
    let x0 = (cx - r).floor() as i32;
    let x1 = (cx + r).ceil() as i32;
    let y0 = (cy - r).floor() as i32;
    let y1 = (cy + r).ceil() as i32;
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy <= r2 {
                blend(img, x, y, color, 1.0);
            }
        }
    }
}

fn thick_line(img: &mut RgbaImage, a: Pos2, b: Pos2, width: f32, color: Rgba<u8>) {
    let r = (width / 2.0).max(0.5);
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let steps = (len * 2.0).ceil() as i32;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        fill_disk(img, a.x + dx * t, a.y + dy * t, r, color);
    }
}

fn draw_rect(img: &mut RgbaImage, rect: Rect, style: &Style) {
    let color = to_rgba(style.color);
    let r = rect;
    let tl = r.min;
    let tr = Pos2::new(r.max.x, r.min.y);
    let br = r.max;
    let bl = Pos2::new(r.min.x, r.max.y);
    thick_line(img, tl, tr, style.stroke, color);
    thick_line(img, tr, br, style.stroke, color);
    thick_line(img, br, bl, style.stroke, color);
    thick_line(img, bl, tl, style.stroke, color);
}

fn draw_ellipse(img: &mut RgbaImage, rect: Rect, style: &Style) {
    let color = to_rgba(style.color);
    let cx = rect.center().x;
    let cy = rect.center().y;
    let rx = rect.width().abs() / 2.0;
    let ry = rect.height().abs() / 2.0;
    if rx < 0.5 || ry < 0.5 {
        return;
    }
    let steps = (((rx + ry) * 3.0).ceil() as i32).max(64);
    let disk_r = (style.stroke / 2.0).max(0.5);
    for i in 0..steps {
        let t = i as f32 / steps as f32 * std::f32::consts::TAU;
        fill_disk(img, cx + rx * t.cos(), cy + ry * t.sin(), disk_r, color);
    }
}

fn rotate(v: (f32, f32), ang: f32) -> (f32, f32) {
    let (c, s) = (ang.cos(), ang.sin());
    (v.0 * c - v.1 * s, v.0 * s + v.1 * c)
}

fn draw_arrow(img: &mut RgbaImage, from: Pos2, to: Pos2, arrow_style: ArrowStyle, style: &Style) {
    let color = to_rgba(style.color);
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = (dx * dx + dy * dy).sqrt().max(1.0);
    let ux = dx / len;
    let uy = dy / len;
    let head_len = (style.stroke * 4.0).clamp(12.0, len);
    let ang = 25f32.to_radians();
    let back = (-ux, -uy);
    let (lx, ly) = rotate(back, ang);
    let (rx, ry) = rotate(back, -ang);
    let barb_l = Pos2::new(to.x + lx * head_len, to.y + ly * head_len);
    let barb_r = Pos2::new(to.x + rx * head_len, to.y + ry * head_len);

    match arrow_style {
        ArrowStyle::Line => {
            thick_line(img, from, to, style.stroke, color);
            thick_line(img, to, barb_l, style.stroke, color);
            thick_line(img, to, barb_r, style.stroke, color);
        }
        ArrowStyle::Solid => {
            let shaft_end = Pos2::new(to.x - ux * head_len * 0.7, to.y - uy * head_len * 0.7);
            thick_line(img, from, shaft_end, style.stroke, color);
            fill_triangle(img, to, barb_l, barb_r, color);
        }
    }
}

fn fill_triangle(img: &mut RgbaImage, a: Pos2, b: Pos2, c: Pos2, color: Rgba<u8>) {
    let min_x = a.x.min(b.x).min(c.x).floor() as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil() as i32;
    let min_y = a.y.min(b.y).min(c.y).floor() as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil() as i32;
    let area = edge(a, b, c);
    if area.abs() < f32::EPSILON {
        return;
    }
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let p = Pos2::new(x as f32 + 0.5, y as f32 + 0.5);
            let w0 = edge(b, c, p);
            let w1 = edge(c, a, p);
            let w2 = edge(a, b, p);
            let inside =
                (w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0) || (w0 <= 0.0 && w1 <= 0.0 && w2 <= 0.0);
            if inside {
                blend(img, x, y, color, 1.0);
            }
        }
    }
}

fn edge(a: Pos2, b: Pos2, c: Pos2) -> f32 {
    (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
}

fn draw_mosaic(img: &mut RgbaImage, rect: Rect) {
    let block: u32 = 12;
    let w = img.width();
    let h = img.height();
    let x0 = (rect.min.x.max(0.0) as u32).min(w);
    let y0 = (rect.min.y.max(0.0) as u32).min(h);
    let x1 = (rect.max.x.max(0.0) as u32).min(w);
    let y1 = (rect.max.y.max(0.0) as u32).min(h);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let mut by = y0;
    while by < y1 {
        let mut bx = x0;
        while bx < x1 {
            let bx1 = (bx + block).min(x1);
            let by1 = (by + block).min(y1);
            let (mut sr, mut sg, mut sb, mut cnt) = (0u64, 0u64, 0u64, 0u64);
            for yy in by..by1 {
                for xx in bx..bx1 {
                    let p = img.get_pixel(xx, yy);
                    sr += p[0] as u64;
                    sg += p[1] as u64;
                    sb += p[2] as u64;
                    cnt += 1;
                }
            }
            // 块内至少有一个像素（bx<bx1 且 by<by1），cnt 恒 >= 1。
            let avg = Rgba([(sr / cnt) as u8, (sg / cnt) as u8, (sb / cnt) as u8, 255]);
            for yy in by..by1 {
                for xx in bx..bx1 {
                    img.put_pixel(xx, yy, avg);
                }
            }
            bx += block;
        }
        by += block;
    }
}

fn draw_text(
    img: &mut RgbaImage,
    pos: Pos2,
    text: &str,
    font_size: f32,
    color: Rgba<u8>,
    font: &FontVec,
) {
    let scale = PxScale::from(font_size);
    let scaled = font.as_scaled(scale);
    let mut caret_x = pos.x;
    let base_y = pos.y + scaled.ascent();
    for ch in text.chars() {
        if ch == '\n' {
            continue;
        }
        let gid = font.glyph_id(ch);
        let g = gid.with_scale_and_position(scale, point(caret_x, base_y));
        if let Some(outline) = font.outline_glyph(g) {
            let bb = outline.px_bounds();
            outline.draw(|gx, gy, cov| {
                let px = bb.min.x + gx as f32;
                let py = bb.min.y + gy as f32;
                blend(img, px.round() as i32, py.round() as i32, color, cov);
            });
        }
        caret_x += scaled.h_advance(gid);
    }
}

fn draw_number(img: &mut RgbaImage, pos: Pos2, index: u32, style: &Style, font: Option<&FontVec>) {
    let color = to_rgba(style.color);
    let radius = (style.font_size * 0.85).max(12.0);
    fill_disk(img, pos.x, pos.y, radius, color);
    if let Some(f) = font {
        let s = index.to_string();
        let scale = PxScale::from(style.font_size);
        let scaled = f.as_scaled(scale);
        let text_w: f32 = s.chars().map(|c| scaled.h_advance(f.glyph_id(c))).sum();
        let tx = pos.x - text_w / 2.0;
        let ty = pos.y - style.font_size / 2.0;
        draw_text(
            img,
            Pos2::new(tx, ty),
            &s,
            style.font_size,
            Rgba([255, 255, 255, 255]),
            f,
        );
    }
}

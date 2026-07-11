//! 标注数据模型。坐标均为底图物理像素坐标（原点为图像左上角）。
use eframe::egui::{Color32, Pos2, Rect};

/// 箭头样式。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArrowStyle {
    /// 细线箭头。
    Line,
    /// 实心三角箭头。
    Solid,
}

/// 标注样式。
#[derive(Clone, Copy, Debug)]
pub struct Style {
    pub color: Color32,
    pub stroke: f32,
    pub font_size: f32,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            color: Color32::from_rgb(255, 60, 60),
            stroke: 3.0,
            font_size: 20.0,
        }
    }
}

/// 单个标注。
#[derive(Clone, Debug)]
pub enum Annotation {
    Line {
        from: Pos2,
        to: Pos2,
        style: Style,
    },
    Pencil {
        points: Vec<Pos2>,
        style: Style,
    },
    Rect {
        rect: Rect,
        style: Style,
    },
    Ellipse {
        rect: Rect,
        style: Style,
    },
    Arrow {
        from: Pos2,
        to: Pos2,
        arrow_style: ArrowStyle,
        style: Style,
    },
    Text {
        pos: Pos2,
        content: String,
        style: Style,
    },
    Number {
        pos: Pos2,
        index: u32,
        style: Style,
    },
    Mosaic {
        rect: Rect,
    },
}

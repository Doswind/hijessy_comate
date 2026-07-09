//! 捕获层核心抽象。
//!
//! 这里定义了整个应用的扩展骨架：
//! - `Frame`     单帧捕获结果（截图 = 单帧；未来动图/录屏 = 多帧流）
//! - `CaptureSource` 捕获来源（全屏 / 窗口 / 区域 / 滚动长截图）
//! - `CaptureEngine` 捕获引擎 trait
//!
//! 未来扩展 GIF / 录屏时，只需新增 `FrameStream` trait 与对应实现，
//! 无需改动此处已有类型。
use std::time::Instant;

use image::RgbaImage;

pub mod engine;
pub mod scroll;

pub use engine::XcapEngine;

/// 物理像素矩形（屏幕坐标系）。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PixelRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// 显示器 / 窗口标识。
pub type MonitorId = u32;
pub type WindowId = u32;

/// 一帧捕获结果。
pub struct Frame {
    pub image: RgbaImage,
    pub region: PixelRect,
    /// 捕获时间（预留：动图/录屏按时间排序帧）。
    #[allow(dead_code)]
    pub captured_at: Instant,
}

impl Frame {
    pub fn new(image: RgbaImage, region: PixelRect) -> Self {
        Self {
            image,
            region,
            captured_at: Instant::now(),
        }
    }
}

/// 捕获来源。
#[allow(dead_code)] // Window/Region 为捕获 API 预留（迭代二浮层直接裁剪冻结图）
#[derive(Clone, Debug)]
pub enum CaptureSource {
    /// 指定显示器全屏；`None` 表示主显示器。
    FullScreen(Option<MonitorId>),
    /// 指定窗口。
    Window(WindowId),
    /// 屏幕上的一块区域（物理像素）。
    Region(PixelRect),
}

/// 捕获引擎抽象。截图取单帧；未来录屏可新增 `stream()` 方法或独立 trait。
pub trait CaptureEngine {
    fn capture(&self, source: CaptureSource) -> anyhow::Result<Frame>;
}

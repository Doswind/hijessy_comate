//! 基于 xcap 的跨平台捕获引擎实现（全屏 / 窗口 / 区域）。
use image::RgbaImage;
use xcap::{Monitor, Window};

use crate::capture::{CaptureEngine, CaptureSource, Frame, PixelRect};

/// 使用 xcap 的跨平台捕获引擎。
#[derive(Default)]
pub struct XcapEngine;

impl XcapEngine {
    pub fn new() -> Self {
        Self
    }
}

impl CaptureEngine for XcapEngine {
    fn capture(&self, source: CaptureSource) -> anyhow::Result<Frame> {
        match source {
            CaptureSource::FullScreen(id) => capture_full_screen(id),
            CaptureSource::Window(id) => capture_window(id),
            CaptureSource::Region(rect) => capture_region(rect),
        }
    }
}

/// 显示器信息（供选择/坐标换算使用；多显示器扩展预留）。
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct MonitorInfo {
    pub id: u32,
    pub rect: PixelRect,
    pub scale: f32,
    pub is_primary: bool,
}

/// 窗口信息（供窗口高亮选择使用）。
#[derive(Clone, Debug)]
pub struct WindowInfo {
    #[allow(dead_code)]
    pub id: u32,
    #[allow(dead_code)]
    pub title: String,
    pub rect: PixelRect,
}

fn monitor_rect(m: &Monitor) -> anyhow::Result<PixelRect> {
    Ok(PixelRect::new(m.x()?, m.y()?, m.width()?, m.height()?))
}

/// 列出所有显示器信息（多显示器扩展预留）。
#[allow(dead_code)]
pub fn list_monitors() -> anyhow::Result<Vec<MonitorInfo>> {
    let monitors = Monitor::all()?;
    let mut out = Vec::with_capacity(monitors.len());
    for m in &monitors {
        out.push(MonitorInfo {
            id: m.id()?,
            rect: monitor_rect(m)?,
            scale: m.scale_factor().unwrap_or(1.0),
            is_primary: m.is_primary().unwrap_or(false),
        });
    }
    Ok(out)
}

/// 列出所有可见（未最小化）窗口，按 z 序（前置在后）返回。
pub fn list_windows() -> anyhow::Result<Vec<WindowInfo>> {
    let windows = Window::all()?;
    let mut out = Vec::new();
    for w in &windows {
        if w.is_minimized().unwrap_or(false) {
            continue;
        }
        let width = w.width()?;
        let height = w.height()?;
        if width == 0 || height == 0 {
            continue;
        }
        out.push(WindowInfo {
            id: w.id()?,
            title: w.title().unwrap_or_default(),
            rect: PixelRect::new(w.x()?, w.y()?, width, height),
        });
    }
    Ok(out)
}

fn capture_full_screen(id: Option<u32>) -> anyhow::Result<Frame> {
    let monitors = Monitor::all()?;
    if monitors.is_empty() {
        anyhow::bail!("未找到显示器");
    }

    let mut chosen: Option<&Monitor> = None;
    match id {
        Some(target) => {
            for m in &monitors {
                if m.id()? == target {
                    chosen = Some(m);
                    break;
                }
            }
        }
        None => {
            for m in &monitors {
                if m.is_primary().unwrap_or(false) {
                    chosen = Some(m);
                    break;
                }
            }
        }
    }

    let monitor = match chosen {
        Some(m) => m,
        None => monitors.first().ok_or_else(|| anyhow::anyhow!("未找到显示器"))?,
    };

    let image = monitor.capture_image()?;
    let region = monitor_rect(monitor)?;
    Ok(Frame::new(image, region))
}

fn capture_window(id: u32) -> anyhow::Result<Frame> {
    let windows = Window::all()?;
    for w in &windows {
        if w.id()? == id {
            if w.is_minimized().unwrap_or(false) {
                anyhow::bail!("目标窗口已最小化，无法截取");
            }
            let image = w.capture_image()?;
            let region = PixelRect::new(w.x()?, w.y()?, w.width()?, w.height()?);
            return Ok(Frame::new(image, region));
        }
    }
    anyhow::bail!("未找到目标窗口")
}

fn capture_region(rect: PixelRect) -> anyhow::Result<Frame> {
    if rect.is_empty() {
        anyhow::bail!("选区无效");
    }
    let cx = rect.x + rect.width as i32 / 2;
    let cy = rect.y + rect.height as i32 / 2;
    let monitor = Monitor::from_point(cx, cy)?;

    let rel_x = (rect.x - monitor.x()?).max(0) as u32;
    let rel_y = (rect.y - monitor.y()?).max(0) as u32;

    let image: RgbaImage = monitor.capture_region(rel_x, rel_y, rect.width, rect.height)?;
    Ok(Frame::new(image, rect))
}

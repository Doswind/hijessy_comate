//! 字体加载：为界面与标注文字提供 CJK 支持。
//!
//! 轻量策略：不内置字体，运行时尝试加载系统 CJK 字体；
//! 同一份字体既安装进 egui（界面/文字显示），也供合成器（烧录到输出图）使用。
use ab_glyph::FontVec;
use eframe::egui;

pub struct LoadedFont {
    pub bytes: Vec<u8>,
    pub index: u32,
    pub font: FontVec,
}

/// 尝试加载系统 CJK 字体。
pub fn load_system_font() -> Option<LoadedFont> {
    for (path, index) in candidates() {
        let Ok(bytes) = std::fs::read(path) else {
            continue;
        };
        if let Ok(font) = FontVec::try_from_vec_and_index(bytes.clone(), index) {
            return Some(LoadedFont { bytes, index, font });
        }
    }
    None
}

/// 将加载的字体安装为 egui 的首选字体（支持中文显示）。
pub fn install_into_egui(ctx: &egui::Context, loaded: &LoadedFont) {
    use std::sync::Arc;
    let mut fonts = egui::FontDefinitions::default();
    let mut data = egui::FontData::from_owned(loaded.bytes.clone());
    data.index = loaded.index;
    fonts
        .font_data
        .insert("system_cjk".to_owned(), Arc::new(data));
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, "system_cjk".to_owned());
    }
    ctx.set_fonts(fonts);
}

#[cfg(target_os = "macos")]
fn candidates() -> Vec<(&'static str, u32)> {
    vec![
        ("/System/Library/Fonts/PingFang.ttc", 0),
        ("/System/Library/Fonts/STHeiti Medium.ttc", 0),
        ("/System/Library/Fonts/Hiragino Sans GB.ttc", 0),
        ("/Library/Fonts/Arial Unicode.ttf", 0),
    ]
}

#[cfg(target_os = "windows")]
fn candidates() -> Vec<(&'static str, u32)> {
    vec![
        ("C:/Windows/Fonts/msyh.ttc", 0),
        ("C:/Windows/Fonts/simhei.ttf", 0),
        ("C:/Windows/Fonts/simsun.ttc", 0),
        ("C:/Windows/Fonts/segoeui.ttf", 0),
    ]
}

#[cfg(all(unix, not(target_os = "macos")))]
fn candidates() -> Vec<(&'static str, u32)> {
    vec![
        ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", 0),
        ("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc", 0),
        ("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", 0),
        (
            "/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc",
            0,
        ),
        ("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf", 0),
    ]
}

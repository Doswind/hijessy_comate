//! 剪贴板输出（arboard），默认输出目标。
use std::borrow::Cow;

use arboard::{Clipboard, ImageData};
use image::RgbaImage;

use crate::output::{Artifact, OutputSink};

#[derive(Default)]
pub struct ClipboardSink;

impl ClipboardSink {
    pub fn new() -> Self {
        Self
    }
}

impl OutputSink for ClipboardSink {
    fn write(&self, artifact: &Artifact) -> anyhow::Result<()> {
        match artifact {
            Artifact::Image(img) => set_image_to_clipboard(img),
        }
    }
}

/// 将 RGBA 图像写入系统剪贴板。
///
/// 注意：在 Linux(X11) 下，剪贴板数据由本进程提供，需应用保持运行才能被其它程序粘贴。
pub fn set_image_to_clipboard(img: &RgbaImage) -> anyhow::Result<()> {
    let mut clipboard = Clipboard::new()?;
    let data = ImageData {
        width: img.width() as usize,
        height: img.height() as usize,
        bytes: Cow::Borrowed(img.as_raw()),
    };
    clipboard.set_image(data)?;
    Ok(())
}

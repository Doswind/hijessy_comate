//! 输出层核心抽象。
//!
//! `Artifact` 表示可输出的产物；当前仅有图片，未来可加 `Gif` / `Video` 变体。
//! `OutputSink` 是输出目标 trait（剪贴板 / 文件 / 未来编码器），加法式扩展。
use image::RgbaImage;

pub mod clipboard;
pub mod file;

pub use clipboard::ClipboardSink;
pub use file::FileSink;

/// 可输出的产物。
pub enum Artifact<'a> {
    Image(&'a RgbaImage),
    // 未来: Gif(&'a GifData), Video(&'a VideoData)
}

/// 输出目标抽象。
pub trait OutputSink {
    fn write(&self, artifact: &Artifact) -> anyhow::Result<()>;
}

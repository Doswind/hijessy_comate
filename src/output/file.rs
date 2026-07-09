//! 文件输出：保存为 PNG / JPEG。
use std::path::{Path, PathBuf};

use image::RgbaImage;

use crate::output::{Artifact, OutputSink};

pub struct FileSink {
    pub path: PathBuf,
}

impl FileSink {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl OutputSink for FileSink {
    fn write(&self, artifact: &Artifact) -> anyhow::Result<()> {
        match artifact {
            Artifact::Image(img) => save_image(img, &self.path),
        }
    }
}

/// 保存图像到指定路径，按扩展名推断格式。JPEG 会丢弃透明通道。
pub fn save_image(img: &RgbaImage, path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();

    match ext.as_str() {
        "jpg" | "jpeg" => {
            // JPEG 不支持 alpha，转为 RGB。
            let rgb = image::DynamicImage::ImageRgba8(img.clone()).to_rgb8();
            rgb.save(path)?;
        }
        _ => {
            img.save(path)?;
        }
    }
    Ok(())
}

/// 默认输出目录：优先 `~/Pictures`，退回用户主目录，最后当前目录。
pub fn default_output_dir() -> PathBuf {
    if let Some(home) = home_dir() {
        let pics = home.join("Pictures");
        if pics.is_dir() {
            return pics;
        }
        return home;
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 基于时间戳生成默认文件名，如 `screenshot-1700000000.png`。
pub fn timestamped_filename(ext: &str) -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("screenshot-{secs}.{ext}")
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

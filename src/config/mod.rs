//! 应用配置：保存目录、图片格式、剪贴板开关、快捷键。
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub mod hotkeys;

pub use hotkeys::HotkeyConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// 截图保存目录。
    pub save_dir: PathBuf,
    /// 默认图片格式："png" | "jpg"。
    pub image_format: String,
    /// 截图完成后是否自动写入剪贴板。
    pub auto_clipboard: bool,
    /// 全局快捷键。
    pub hotkeys: HotkeyConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_dir: crate::output::file::default_output_dir(),
            image_format: "png".to_owned(),
            auto_clipboard: true,
            hotkeys: HotkeyConfig::default(),
        }
    }
}

impl Config {
    /// 配置文件路径：`$XDG_CONFIG_HOME/hijessy/config.toml` 或 `~/.config/hijessy/config.toml`。
    pub fn config_path() -> PathBuf {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| home_dir().map(|h| h.join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        base.join("hijessy").join("config.toml")
    }

    /// 加载配置；文件不存在或解析失败时返回默认值。
    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(s) => toml::from_str(&s).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// 保存配置到磁盘。
    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self)?;
        std::fs::write(path, s)?;
        Ok(())
    }
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
}

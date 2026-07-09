//! 快捷键配置。
//!
//! 使用字符串描述（global-hotkey 语法，如 `CmdOrCtrl+Shift+KeyA`），
//! `CmdOrCtrl` 在 macOS 解析为 Command、其它平台为 Ctrl。
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HotkeyConfig {
    /// 框选截图。
    pub region: String,
    /// 全屏截图。
    pub fullscreen: String,
    /// 窗口截图。
    pub window: String,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            region: "CmdOrCtrl+Shift+KeyA".to_owned(),
            fullscreen: "CmdOrCtrl+Shift+KeyF".to_owned(),
            window: "CmdOrCtrl+Shift+KeyW".to_owned(),
        }
    }
}

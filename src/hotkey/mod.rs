//! 全局快捷键管理：注册热键并将事件桥接到应用。
use anyhow::Context;
use global_hotkey::hotkey::HotKey;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};

use crate::config::HotkeyConfig;

/// 快捷键触发的截图动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyAction {
    Region,
    FullScreen,
    Window,
}

pub struct HotkeyManager {
    // 管理器需保持存活以维持注册。
    _manager: GlobalHotKeyManager,
    region_id: u32,
    fullscreen_id: u32,
    window_id: u32,
}

impl HotkeyManager {
    pub fn new(cfg: &HotkeyConfig) -> anyhow::Result<Self> {
        let manager = GlobalHotKeyManager::new().context("初始化全局热键失败")?;

        let region: HotKey = cfg
            .region
            .parse()
            .with_context(|| format!("解析快捷键失败: {}", cfg.region))?;
        let fullscreen: HotKey = cfg
            .fullscreen
            .parse()
            .with_context(|| format!("解析快捷键失败: {}", cfg.fullscreen))?;
        let window: HotKey = cfg
            .window
            .parse()
            .with_context(|| format!("解析快捷键失败: {}", cfg.window))?;

        manager
            .register_all(&[region, fullscreen, window])
            .context("注册全局热键失败")?;

        Ok(Self {
            _manager: manager,
            region_id: region.id(),
            fullscreen_id: fullscreen.id(),
            window_id: window.id(),
        })
    }

    /// 轮询挂起的热键事件，返回被触发的动作（仅在按下时）。
    pub fn poll(&self) -> Option<HotkeyAction> {
        while let Ok(event) = GlobalHotKeyEvent::receiver().try_recv() {
            if event.state() != HotKeyState::Pressed {
                continue;
            }
            let id = event.id();
            if id == self.region_id {
                return Some(HotkeyAction::Region);
            }
            if id == self.fullscreen_id {
                return Some(HotkeyAction::FullScreen);
            }
            if id == self.window_id {
                return Some(HotkeyAction::Window);
            }
        }
        None
    }
}

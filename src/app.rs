//! 应用编排：启动即进入全屏截图浮层，不显示主界面；
//! 完成/取消后隐藏窗口常驻，全局快捷键可再次唤起。
use eframe::egui;
use image::RgbaImage;

use crate::capture::{CaptureEngine, CaptureSource, XcapEngine};
use crate::config::Config;
use crate::fonts::{self, LoadedFont};
use crate::hotkey::HotkeyManager;
use crate::output::file::timestamped_filename;
use crate::output::{Artifact, ClipboardSink, FileSink, OutputSink};
use crate::overlay::{CaptureSession, SessionOutcome};
use crate::tray::{TrayAction, TrayIconManager};

pub struct HijessyApp {
    engine: XcapEngine,
    config: Config,
    hotkeys: Option<HotkeyManager>,
    tray: Option<TrayIconManager>,
    font: Option<LoadedFont>,
    session: Option<CaptureSession>,
    /// 待开启新会话标记。
    pending_start: bool,
}

impl HijessyApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Light 主题。
        cc.egui_ctx.set_visuals(egui::Visuals::light());

        let config = Config::load();
        if !Config::config_path().exists() {
            let _ = config.save();
        }

        let font = fonts::load_system_font();
        if let Some(loaded) = &font {
            fonts::install_into_egui(&cc.egui_ctx, loaded);
        }

        let hotkeys = HotkeyManager::new(&config.hotkeys).ok();

        Self {
            engine: XcapEngine::new(),
            config,
            hotkeys,
            tray: TrayIconManager::new().ok(),
            font,
            session: None,
            // 启动即开始一次截图。
            pending_start: true,
        }
    }

    /// 冻结全屏并进入截图浮层。
    fn start_session(&mut self, ctx: &egui::Context) {
        match self.engine.capture(CaptureSource::FullScreen(None)) {
            Ok(frame) => {
                let origin = (frame.region.x, frame.region.y);
                let windows = crate::capture::engine::list_windows()
                    .unwrap_or_default()
                    .into_iter()
                    .map(|w| w.rect)
                    .collect();
                let cursor = match mouse_position::mouse_position::Mouse::get_mouse_position() {
                    mouse_position::mouse_position::Mouse::Position { x, y } => Some((x, y)),
                    mouse_position::mouse_position::Mouse::Error => None,
                };
                self.session = Some(CaptureSession::new(frame.image, origin, windows, cursor));

                // 关闭鼠标穿透、置顶聚焦，进入截图交互。
                ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(false));
                ctx.send_viewport_cmd(egui::ViewportCommand::WindowLevel(
                    egui::WindowLevel::AlwaysOnTop,
                ));
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            Err(_) => {
                // 捕获失败（如权限缺失）：保持空闲穿透，等待下次唤起。
                self.session = None;
            }
        }
    }

    /// 结束会话：回到空闲（全屏透明 + 鼠标穿透，界面不可见且不拦截操作）。
    fn end_session(&mut self, ctx: &egui::Context) {
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));
        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        self.session = None;
    }

    fn copy_to_clipboard(&self, img: &RgbaImage) {
        let _ = ClipboardSink::new().write(&Artifact::Image(img));
    }

    fn save_to_file(&self, img: &RgbaImage) {
        let ext = if self.config.image_format.eq_ignore_ascii_case("jpg")
            || self.config.image_format.eq_ignore_ascii_case("jpeg")
        {
            "jpg"
        } else {
            "png"
        };
        let path = self.config.save_dir.join(timestamped_filename(ext));
        let _ = FileSink::new(path).write(&Artifact::Image(img));
    }
}

impl eframe::App for HijessyApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        // 透明背景，避免隐藏/切换时闪白。
        [0.0, 0.0, 0.0, 0.0]
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 全局快捷键和托盘都只在空闲时启动新截图。
        if let Some(_action) = self.hotkeys.as_ref().and_then(|m| m.poll())
            && self.session.is_none()
        {
            self.pending_start = true;
        }
        match self.tray.as_ref().and_then(TrayIconManager::poll) {
            Some(TrayAction::Capture) if self.session.is_none() => self.pending_start = true,
            Some(TrayAction::Exit) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
            _ => {}
        }
        if self.pending_start && self.session.is_none() {
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));
            self.pending_start = false;
            self.start_session(ctx);
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(120));
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        let font = self.font.as_ref().map(|f| &f.font);
        let outcome = self.session.as_mut().unwrap().show(ui, font);
        match outcome {
            SessionOutcome::Pending => {}
            SessionOutcome::Cancel => self.end_session(&ctx),
            SessionOutcome::Confirm(img) => {
                if self.config.auto_clipboard {
                    self.copy_to_clipboard(&img);
                }
                self.end_session(&ctx);
            }
            SessionOutcome::Save(img) => {
                self.save_to_file(&img);
                self.end_session(&ctx);
            }
        }
    }
}

/// 将 `image::RgbaImage` 转为 egui 的 `ColorImage`。
pub fn rgba_to_color_image(img: &RgbaImage) -> egui::ColorImage {
    let size = [img.width() as usize, img.height() as usize];
    egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw())
}

//! 一体化截图浮层（Snipaste 风格）：窗口识别、模式切换、选区手柄、
//! 遮罩、标注与底部胶囊工具栏，均在单一全屏浮层内完成。
pub mod icons;

use ab_glyph::FontVec;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Sense, Shape, Stroke, StrokeKind,
    TextureHandle, Vec2,
};
use image::RgbaImage;

use crate::editor::compose;
use crate::editor::model::{Annotation, ArrowStyle, Style};
use icons::Icon;

const ACCENT: Color32 = Color32::from_rgb(0x2B, 0x7F, 0xFF);
const HANDLE_R: f32 = 5.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Window,
    FullScreen,
    Custom,
}

/// 工具（Select 为默认，用于调整选区）。
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum Tool {
    #[default]
    Select,
    Rect,
    Ellipse,
    Arrow,
    Text,
    Number,
    Mosaic,
}

/// 会话结果。
pub enum SessionOutcome {
    Pending,
    Confirm(RgbaImage),
    Save(RgbaImage),
    Cancel,
}

#[derive(Clone, Copy)]
enum DragMode {
    Create,
    Move,
    Resize(u8),
}

#[derive(Clone, Copy)]
struct SelDrag {
    mode: DragMode,
    start_img: Pos2,
    orig: Rect,
}

pub struct CaptureSession {
    frozen: RgbaImage,
    frozen_tex: Option<TextureHandle>,
    image_size: Vec2,
    /// 窗口矩形（图像像素坐标）。
    win_rects: Vec<Rect>,
    mode: Mode,
    locked: bool,
    /// 选区（图像像素坐标）。
    selection: Option<Rect>,
    sel_drag: Option<SelDrag>,
    // 标注
    tool: Tool,
    style: Style,
    arrow_style: ArrowStyle,
    annotations: Vec<Annotation>,
    undo: Vec<Vec<Annotation>>,
    redo: Vec<Vec<Annotation>>,
    ann_start: Option<Pos2>,
    pending_text: Option<(Pos2, String)>,
    next_number: u32,
    show_props: bool,
}

impl CaptureSession {
    pub fn new(frozen: RgbaImage, monitor_origin: (i32, i32), windows: Vec<crate::capture::PixelRect>) -> Self {
        let image_size = Vec2::new(frozen.width() as f32, frozen.height() as f32);
        // 窗口屏幕坐标 -> 图像像素坐标，并过滤到本显示器范围。
        let win_rects = windows
            .into_iter()
            .map(|w| {
                Rect::from_min_size(
                    Pos2::new((w.x - monitor_origin.0) as f32, (w.y - monitor_origin.1) as f32),
                    Vec2::new(w.width as f32, w.height as f32),
                )
            })
            .filter(|r| r.max.x > 0.0 && r.max.y > 0.0 && r.min.x < image_size.x && r.min.y < image_size.y)
            .collect();

        Self {
            frozen,
            frozen_tex: None,
            image_size,
            win_rects,
            mode: Mode::Window,
            locked: false,
            selection: None,
            sel_drag: None,
            tool: Tool::Select,
            style: Style::default(),
            arrow_style: ArrowStyle::Line,
            annotations: Vec::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            ann_start: None,
            pending_text: None,
            next_number: 1,
            show_props: false,
        }
    }

    fn full_img_rect(&self) -> Rect {
        Rect::from_min_size(Pos2::ZERO, self.image_size)
    }

    fn window_at(&self, p: Pos2) -> Option<Rect> {
        self.win_rects
            .iter()
            .filter(|r| r.contains(p))
            .min_by(|a, b| (a.area()).partial_cmp(&b.area()).unwrap())
            .copied()
    }

    fn snapshot(&mut self) {
        self.undo.push(self.annotations.clone());
        self.redo.clear();
    }

    fn undo(&mut self) {
        if let Some(prev) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.annotations, prev));
        }
    }

    fn redo(&mut self) {
        if let Some(next) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.annotations, next));
        }
    }

    fn output(&self, font: Option<&FontVec>) -> Option<RgbaImage> {
        let sel = self.selection?;
        let composed = compose::compose(&self.frozen, &self.annotations, font);
        let x = sel.min.x.max(0.0) as u32;
        let y = sel.min.y.max(0.0) as u32;
        let w = (sel.width().round() as u32).min(composed.width().saturating_sub(x));
        let h = (sel.height().round() as u32).min(composed.height().saturating_sub(y));
        if w == 0 || h == 0 {
            return None;
        }
        Some(image::imageops::crop_imm(&composed, x, y, w, h).to_image())
    }

    pub fn show(&mut self, ui: &mut egui::Ui, font: Option<&FontVec>) -> SessionOutcome {
        let ctx = ui.ctx().clone();
        if self.frozen_tex.is_none() {
            let color = crate::app::rgba_to_color_image(&self.frozen);
            self.frozen_tex = Some(ctx.load_texture("frozen", color, egui::TextureOptions::LINEAR));
        }

        let full = ui.available_rect_before_wrap();
        let sx = full.width() / self.image_size.x;
        let sy = full.height() / self.image_size.y;
        let to_screen = |p: Pos2| Pos2::new(full.min.x + p.x * sx, full.min.y + p.y * sy);
        let to_img = |p: Pos2| Pos2::new((p.x - full.min.x) / sx, (p.y - full.min.y) / sy);
        let rect_to_screen =
            |r: Rect| Rect::from_min_max(to_screen(r.min), to_screen(r.max));

        let painter = ui.painter().clone();
        // 底图。
        if let Some(tex) = &self.frozen_tex {
            painter.image(
                tex.id(),
                full,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        }

        let cursor_img = ctx.pointer_hover_pos().map(to_img);

        // 当前选区（图像像素）。
        let sel_img = match self.mode {
            Mode::FullScreen => Some(self.full_img_rect()),
            Mode::Window if !self.locked => cursor_img.and_then(|c| self.window_at(c)),
            _ => self.selection,
        };

        // 选区外压暗。
        if let Some(sel) = sel_img {
            let s = rect_to_screen(sel);
            let mask = Color32::from_black_alpha(90);
            painter.rect_filled(Rect::from_min_max(full.min, Pos2::new(full.max.x, s.min.y)), 0.0, mask);
            painter.rect_filled(Rect::from_min_max(Pos2::new(full.min.x, s.max.y), full.max), 0.0, mask);
            painter.rect_filled(Rect::from_min_max(Pos2::new(full.min.x, s.min.y), Pos2::new(s.min.x, s.max.y)), 0.0, mask);
            painter.rect_filled(Rect::from_min_max(Pos2::new(s.max.x, s.min.y), Pos2::new(full.max.x, s.max.y)), 0.0, mask);
            painter.rect_stroke(s, 0.0, Stroke::new(1.5, ACCENT), StrokeKind::Inside);

            // 尺寸标签。
            let label = format!("{} × {}", sel.width().round() as i32, sel.height().round() as i32);
            let label_pos = Pos2::new(s.min.x, (s.min.y - 24.0).max(full.min.y + 2.0));
            let galley = painter.layout_no_wrap(label.clone(), FontId::proportional(13.0), Color32::WHITE);
            let bg = Rect::from_min_size(label_pos, galley.size() + Vec2::new(10.0, 6.0));
            painter.rect_filled(bg, CornerRadius::same(4), Color32::from_black_alpha(160));
            painter.text(label_pos + Vec2::new(5.0, 3.0), Align2::LEFT_TOP, label, FontId::proportional(13.0), Color32::WHITE);
        } else {
            painter.rect_filled(full, 0.0, Color32::from_black_alpha(90));
        }

        // 已提交标注（在选区可视，超出部分输出时裁掉）。
        for a in &self.annotations {
            draw_annotation(&painter, a, &to_screen, sx);
        }

        // 选区手柄（仅锁定 + Select 工具时）。
        if self.locked && self.tool == Tool::Select {
            if let Some(sel) = sel_img {
                for h in handle_points(rect_to_screen(sel)) {
                    painter.circle_filled(h, HANDLE_R, Color32::WHITE);
                    painter.circle_stroke(h, HANDLE_R, Stroke::new(1.5, ACCENT));
                }
            }
        }

        // 顶部提示（未锁定时）。
        if !self.locked {
            painter.text(
                Pos2::new(full.center().x, full.min.y + 10.0),
                Align2::CENTER_TOP,
                "移动到窗口自动识别 · 拖拽自定义框选 · Esc 取消",
                FontId::proportional(15.0),
                Color32::WHITE,
            );
        }

        // === 浮动 UI（模式按钮 / 工具栏 / 属性 / 文字输入） ===
        let mut outcome = SessionOutcome::Pending;
        let mut ui_rects: Vec<Rect> = Vec::new();
        let mut tool_action: Option<ToolbarAction> = None;

        if let Some(sel) = sel_img {
            let s = rect_to_screen(sel);
            // 模式按钮（选区右上外侧）。
            let mode_pos = Pos2::new((s.max.x - 88.0).max(full.min.x + 4.0), (s.min.y - 44.0).max(full.min.y + 2.0));
            let area = egui::Area::new(egui::Id::new("mode_btns"))
                .fixed_pos(mode_pos)
                .order(egui::Order::Foreground)
                .show(&ctx, |ui| {
                    egui::Frame::popup(ui.style()).inner_margin(4.0).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if icon_button(ui, Icon::FullScreen, "全屏", self.mode == Mode::FullScreen, 26.0) {
                                tool_action = Some(ToolbarAction::FullScreen);
                            }
                            if icon_button(ui, Icon::Window, "窗口", self.mode == Mode::Window, 26.0) {
                                tool_action = Some(ToolbarAction::WindowMode);
                            }
                        });
                    });
                });
            ui_rects.push(area.response.rect);

            // 底部胶囊工具栏（仅锁定后）。
            if self.locked {
                let tb_pos = Pos2::new(s.min.x, (s.max.y + 10.0).min(full.max.y - 46.0));
                let area = egui::Area::new(egui::Id::new("toolbar"))
                    .fixed_pos(tb_pos)
                    .order(egui::Order::Foreground)
                    .show(&ctx, |ui| {
                        egui::Frame::popup(ui.style())
                            .corner_radius(CornerRadius::same(10))
                            .inner_margin(6.0)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    if let Some(a) = self.toolbar_contents(ui) {
                                        tool_action = Some(a);
                                    }
                                });
                            });
                    });
                ui_rects.push(area.response.rect);

                // 属性弹层。
                if self.show_props {
                    let area = egui::Area::new(egui::Id::new("props"))
                        .fixed_pos(Pos2::new(tb_pos.x, tb_pos.y + 46.0))
                        .order(egui::Order::Foreground)
                        .show(&ctx, |ui| {
                            egui::Frame::popup(ui.style()).inner_margin(8.0).show(ui, |ui| {
                                self.props_contents(ui);
                            });
                        });
                    ui_rects.push(area.response.rect);
                }
            }
        }

        // === 背景交互（避开浮动 UI 区域） ===
        let response = ui.interact(full, egui::Id::new("overlay_bg"), Sense::click_and_drag());
        let pointer_over_ui = ctx
            .pointer_latest_pos()
            .map(|p| ui_rects.iter().any(|r| r.contains(p)))
            .unwrap_or(false);

        if !pointer_over_ui {
            if self.locked && self.tool != Tool::Select {
                self.handle_annotation(&painter, &response, &to_img, &to_screen, sx);
            } else {
                self.handle_selection(&response, &to_img, &to_screen, sel_img);
            }
        }

        // 文字输入浮层。
        self.show_pending_text(&ctx, &to_screen, sx);

        // 处理工具栏/模式动作。
        if let Some(a) = tool_action {
            match a {
                ToolbarAction::FullScreen => {
                    self.mode = Mode::FullScreen;
                    self.selection = Some(self.full_img_rect());
                    self.locked = true;
                }
                ToolbarAction::WindowMode => {
                    self.mode = Mode::Window;
                    self.locked = false;
                    self.selection = None;
                    self.tool = Tool::Select;
                }
                ToolbarAction::SelectTool(t) => {
                    self.tool = t;
                    self.show_props = matches!(t, Tool::Rect | Tool::Ellipse | Tool::Arrow | Tool::Text | Tool::Number | Tool::Mosaic);
                }
                ToolbarAction::ToggleProps => self.show_props = !self.show_props,
                ToolbarAction::Undo => self.undo(),
                ToolbarAction::Redo => self.redo(),
                ToolbarAction::Save => {
                    if let Some(img) = self.output(font) {
                        outcome = SessionOutcome::Save(img);
                    }
                }
                ToolbarAction::Confirm => {
                    if let Some(img) = self.output(font) {
                        outcome = SessionOutcome::Confirm(img);
                    }
                }
                ToolbarAction::Cancel => outcome = SessionOutcome::Cancel,
            }
        }

        // 键盘。
        let (esc, enter, undo, redo) = ctx.input(|i| {
            let cmd = i.modifiers.command;
            (
                i.key_pressed(egui::Key::Escape),
                i.key_pressed(egui::Key::Enter),
                cmd && i.key_pressed(egui::Key::Z) && !i.modifiers.shift,
                cmd && (i.key_pressed(egui::Key::Y) || (i.modifiers.shift && i.key_pressed(egui::Key::Z))),
            )
        });
        if esc {
            outcome = SessionOutcome::Cancel;
        }
        if undo {
            self.undo();
        }
        if redo {
            self.redo();
        }
        if enter && self.locked {
            if let Some(img) = self.output(font) {
                outcome = SessionOutcome::Confirm(img);
            }
        }

        outcome
    }

    fn toolbar_contents(&mut self, ui: &mut egui::Ui) -> Option<ToolbarAction> {
        let mut action = None;
        let tools = [
            (Icon::Cursor, "选择/调整", Tool::Select),
            (Icon::Rect, "矩形", Tool::Rect),
            (Icon::Ellipse, "椭圆", Tool::Ellipse),
            (Icon::Arrow, "箭头", Tool::Arrow),
            (Icon::Text, "文字", Tool::Text),
            (Icon::Number, "序号", Tool::Number),
            (Icon::Mosaic, "马赛克", Tool::Mosaic),
        ];
        for (icon, tip, tool) in tools {
            if icon_button(ui, icon, tip, self.tool == tool, 30.0) {
                action = Some(ToolbarAction::SelectTool(tool));
            }
        }
        ui.separator();
        // 颜色指示按钮，点击开关属性。
        let (rect, resp) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::click());
        if resp.hovered() {
            ui.painter().rect_filled(rect, CornerRadius::same(6), Color32::from_gray(230));
        }
        ui.painter().circle_filled(rect.center(), 8.0, self.style.color);
        if resp.on_hover_text("样式").clicked() {
            action = Some(ToolbarAction::ToggleProps);
        }
        ui.separator();
        if icon_button(ui, Icon::Undo, "撤销", false, 30.0) {
            action = Some(ToolbarAction::Undo);
        }
        if icon_button(ui, Icon::Redo, "重做", false, 30.0) {
            action = Some(ToolbarAction::Redo);
        }
        ui.separator();
        if icon_button(ui, Icon::Save, "保存到文件", false, 30.0) {
            action = Some(ToolbarAction::Save);
        }
        if icon_button(ui, Icon::Cancel, "取消", false, 30.0) {
            action = Some(ToolbarAction::Cancel);
        }
        if icon_button(ui, Icon::Confirm, "确认（复制到剪贴板）", false, 30.0) {
            action = Some(ToolbarAction::Confirm);
        }
        action
    }

    fn props_contents(&mut self, ui: &mut egui::Ui) {
        const PALETTE: [(u8, u8, u8); 8] = [
            (255, 60, 60),
            (255, 150, 0),
            (255, 220, 0),
            (60, 200, 60),
            (0, 150, 255),
            (160, 80, 255),
            (255, 255, 255),
            (20, 20, 20),
        ];
        ui.horizontal(|ui| {
            for (r, g, b) in PALETTE {
                let color = Color32::from_rgb(r, g, b);
                let (rect, resp) = ui.allocate_exact_size(Vec2::splat(18.0), Sense::click());
                ui.painter().rect_filled(rect, CornerRadius::same(3), color);
                if self.style.color == color {
                    ui.painter().rect_stroke(rect, CornerRadius::same(3), Stroke::new(2.0, ACCENT), StrokeKind::Outside);
                }
                if resp.clicked() {
                    self.style.color = color;
                }
            }
        });
        ui.horizontal(|ui| {
            ui.label("线宽");
            for (label, w) in [("细", 2.0), ("中", 4.0), ("粗", 6.0)] {
                if ui.selectable_label((self.style.stroke - w).abs() < 0.1, label).clicked() {
                    self.style.stroke = w;
                }
            }
        });
        if matches!(self.tool, Tool::Text | Tool::Number) {
            ui.horizontal(|ui| {
                ui.label("字号");
                for (label, sz) in [("小", 14.0), ("中", 20.0), ("大", 28.0)] {
                    if ui.selectable_label((self.style.font_size - sz).abs() < 0.1, label).clicked() {
                        self.style.font_size = sz;
                    }
                }
            });
        }
        if self.tool == Tool::Arrow {
            ui.horizontal(|ui| {
                ui.label("箭头");
                if ui.selectable_label(self.arrow_style == ArrowStyle::Line, "细线").clicked() {
                    self.arrow_style = ArrowStyle::Line;
                }
                if ui.selectable_label(self.arrow_style == ArrowStyle::Solid, "实心").clicked() {
                    self.arrow_style = ArrowStyle::Solid;
                }
            });
        }
    }

    fn handle_selection(
        &mut self,
        response: &egui::Response,
        to_img: &dyn Fn(Pos2) -> Pos2,
        to_screen: &dyn Fn(Pos2) -> Pos2,
        sel_img: Option<Rect>,
    ) {
        // 未锁定的窗口模式：点击锁定当前窗口选区。
        if response.clicked() && !self.locked {
            if let Some(sel) = sel_img {
                self.selection = Some(sel);
                self.locked = true;
                self.tool = Tool::Select;
            }
            return;
        }

        if response.drag_started() {
            if let Some(p) = response.interact_pointer_pos() {
                let ip = clamp_pos(to_img(p), self.image_size);
                // 命中手柄 / 选区内部 / 空白。
                if let Some(sel) = self.selection.filter(|_| self.locked) {
                    let s_screen = Rect::from_min_max(to_screen(sel.min), to_screen(sel.max));
                    if let Some(h) = hit_handle(s_screen, p) {
                        self.sel_drag = Some(SelDrag { mode: DragMode::Resize(h), start_img: ip, orig: sel });
                    } else if s_screen.contains(p) {
                        self.sel_drag = Some(SelDrag { mode: DragMode::Move, start_img: ip, orig: sel });
                    } else {
                        self.sel_drag = Some(SelDrag { mode: DragMode::Create, start_img: ip, orig: sel });
                        self.mode = Mode::Custom;
                    }
                } else {
                    self.sel_drag = Some(SelDrag { mode: DragMode::Create, start_img: ip, orig: Rect::NOTHING });
                    self.mode = Mode::Custom;
                    self.locked = true;
                }
            }
        }

        if let (Some(drag), Some(p)) = (self.sel_drag, response.interact_pointer_pos()) {
            let cur = clamp_pos(to_img(p), self.image_size);
            let new_sel = match drag.mode {
                DragMode::Create => Rect::from_two_pos(drag.start_img, cur),
                DragMode::Move => {
                    let d = cur - drag.start_img;
                    drag.orig.translate(d)
                }
                DragMode::Resize(h) => resize_rect(drag.orig, h, cur),
            };
            self.selection = Some(new_sel.intersect(self.full_img_rect()));
        }

        if response.drag_stopped() {
            self.sel_drag = None;
        }
    }

    fn handle_annotation(
        &mut self,
        painter: &egui::Painter,
        response: &egui::Response,
        to_img: &dyn Fn(Pos2) -> Pos2,
        to_screen: &dyn Fn(Pos2) -> Pos2,
        sx: f32,
    ) {
        let preview = Stroke::new(self.style.stroke * sx, self.style.color);
        match self.tool {
            Tool::Rect | Tool::Ellipse | Tool::Mosaic | Tool::Arrow => {
                if response.drag_started() {
                    self.ann_start = response.interact_pointer_pos().map(|p| clamp_pos(to_img(p), self.image_size));
                }
                if let (Some(start), Some(curp)) = (self.ann_start, response.interact_pointer_pos()) {
                    let cur = clamp_pos(to_img(curp), self.image_size);
                    let s = to_screen(start);
                    let c = to_screen(cur);
                    match self.tool {
                        Tool::Rect | Tool::Mosaic => {
                            painter.rect_stroke(Rect::from_two_pos(s, c), 0.0, preview, StrokeKind::Inside);
                        }
                        Tool::Ellipse => draw_ellipse_outline(painter, Rect::from_two_pos(s, c), preview),
                        Tool::Arrow => painter.arrow(s, c - s, preview),
                        _ => {}
                    }
                    if response.drag_stopped() {
                        let rect = Rect::from_two_pos(start, cur);
                        if rect.width() >= 2.0 && rect.height() >= 2.0 {
                            self.snapshot();
                            let a = match self.tool {
                                Tool::Rect => Annotation::Rect { rect, style: self.style },
                                Tool::Ellipse => Annotation::Ellipse { rect, style: self.style },
                                Tool::Mosaic => Annotation::Mosaic { rect },
                                Tool::Arrow => Annotation::Arrow { from: start, to: cur, arrow_style: self.arrow_style, style: self.style },
                                _ => unreachable!(),
                            };
                            self.annotations.push(a);
                        }
                        self.ann_start = None;
                    }
                }
            }
            Tool::Number => {
                if response.clicked() {
                    if let Some(p) = response.interact_pointer_pos() {
                        let pos = clamp_pos(to_img(p), self.image_size);
                        self.snapshot();
                        let idx = self.next_number;
                        self.next_number += 1;
                        self.annotations.push(Annotation::Number { pos, index: idx, style: self.style });
                    }
                }
            }
            Tool::Text => {
                if response.clicked() {
                    if let Some(p) = response.interact_pointer_pos() {
                        self.pending_text = Some((clamp_pos(to_img(p), self.image_size), String::new()));
                    }
                }
            }
            Tool::Select => {}
        }
    }

    fn show_pending_text(&mut self, ctx: &egui::Context, to_screen: &dyn Fn(Pos2) -> Pos2, _sx: f32) {
        let Some((pos, mut buf)) = self.pending_text.take() else {
            return;
        };
        let screen_pos = to_screen(pos);
        let mut committed = false;
        let mut cancelled = false;
        egui::Area::new(egui::Id::new("pending_text"))
            .fixed_pos(screen_pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let resp = ui.add(egui::TextEdit::singleline(&mut buf).hint_text("输入文字，回车确认").desired_width(160.0));
                resp.request_focus();
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    committed = true;
                }
                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    cancelled = true;
                }
            });
        if committed {
            if !buf.trim().is_empty() {
                self.snapshot();
                self.annotations.push(Annotation::Text { pos, content: buf, style: self.style });
            }
        } else if !cancelled {
            self.pending_text = Some((pos, buf));
        }
    }
}

enum ToolbarAction {
    FullScreen,
    WindowMode,
    SelectTool(Tool),
    ToggleProps,
    Undo,
    Redo,
    Save,
    Confirm,
    Cancel,
}

fn icon_button(ui: &mut egui::Ui, icon: Icon, tip: &str, selected: bool, size: f32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if selected {
        ui.painter().rect_filled(rect, CornerRadius::same(6), ACCENT);
    } else if resp.hovered() {
        ui.painter().rect_filled(rect, CornerRadius::same(6), Color32::from_gray(232));
    }
    let color = if selected { Color32::WHITE } else { Color32::from_gray(70) };
    icons::draw(ui.painter(), icon, rect, color, 1.7);
    resp.on_hover_text(tip).clicked()
}

fn handle_points(r: Rect) -> [Pos2; 8] {
    [
        r.min,
        Pos2::new(r.center().x, r.min.y),
        Pos2::new(r.max.x, r.min.y),
        Pos2::new(r.max.x, r.center().y),
        r.max,
        Pos2::new(r.center().x, r.max.y),
        Pos2::new(r.min.x, r.max.y),
        Pos2::new(r.min.x, r.center().y),
    ]
}

fn hit_handle(screen_rect: Rect, p: Pos2) -> Option<u8> {
    for (i, h) in handle_points(screen_rect).iter().enumerate() {
        if h.distance(p) <= HANDLE_R + 4.0 {
            return Some(i as u8);
        }
    }
    None
}

fn resize_rect(orig: Rect, handle: u8, cur: Pos2) -> Rect {
    let mut min = orig.min;
    let mut max = orig.max;
    match handle {
        0 => {
            min.x = cur.x;
            min.y = cur.y;
        }
        1 => min.y = cur.y,
        2 => {
            max.x = cur.x;
            min.y = cur.y;
        }
        3 => max.x = cur.x,
        4 => {
            max.x = cur.x;
            max.y = cur.y;
        }
        5 => max.y = cur.y,
        6 => {
            min.x = cur.x;
            max.y = cur.y;
        }
        7 => min.x = cur.x,
        _ => {}
    }
    Rect::from_two_pos(min, max)
}

fn clamp_pos(p: Pos2, size: Vec2) -> Pos2 {
    Pos2::new(p.x.clamp(0.0, size.x), p.y.clamp(0.0, size.y))
}

fn draw_ellipse_outline(painter: &egui::Painter, rect: Rect, stroke: Stroke) {
    let cx = rect.center().x;
    let cy = rect.center().y;
    let rx = rect.width().abs() / 2.0;
    let ry = rect.height().abs() / 2.0;
    let steps = 48;
    let mut pts = Vec::with_capacity(steps + 1);
    for i in 0..=steps {
        let t = i as f32 / steps as f32 * std::f32::consts::TAU;
        pts.push(Pos2::new(cx + rx * t.cos(), cy + ry * t.sin()));
    }
    painter.add(Shape::line(pts, stroke));
}

/// 用 egui 画笔实时绘制已提交标注（图像像素 → 屏幕）。
fn draw_annotation(painter: &egui::Painter, a: &Annotation, to_screen: &dyn Fn(Pos2) -> Pos2, sx: f32) {
    match a {
        Annotation::Rect { rect, style } => {
            let s = Rect::from_min_max(to_screen(rect.min), to_screen(rect.max));
            painter.rect_stroke(s, 0.0, Stroke::new(style.stroke * sx, style.color), StrokeKind::Inside);
        }
        Annotation::Ellipse { rect, style } => {
            let s = Rect::from_min_max(to_screen(rect.min), to_screen(rect.max));
            draw_ellipse_outline(painter, s, Stroke::new(style.stroke * sx, style.color));
        }
        Annotation::Arrow { from, to, arrow_style, style } => {
            let s = to_screen(*from);
            let e = to_screen(*to);
            let stroke = Stroke::new(style.stroke * sx, style.color);
            painter.arrow(s, e - s, stroke);
            if *arrow_style == ArrowStyle::Solid {
                painter.circle_filled(e, style.stroke * sx * 1.2, style.color);
            }
        }
        Annotation::Text { pos, content, style } => {
            painter.text(to_screen(*pos), Align2::LEFT_TOP, content, FontId::proportional(style.font_size * sx), style.color);
        }
        Annotation::Number { pos, index, style } => {
            let c = to_screen(*pos);
            let r = style.font_size * sx * 0.85;
            painter.circle_filled(c, r, style.color);
            painter.text(c, Align2::CENTER_CENTER, index.to_string(), FontId::proportional(style.font_size * sx), Color32::WHITE);
        }
        Annotation::Mosaic { rect } => {
            let s = Rect::from_min_max(to_screen(rect.min), to_screen(rect.max));
            painter.rect_filled(s, 0.0, Color32::from_gray(128).gamma_multiply(0.7));
            painter.rect_stroke(s, 0.0, Stroke::new(1.0, Color32::from_gray(180)), StrokeKind::Inside);
        }
    }
}

use eframe::egui;

use crate::config::Config;

pub enum SettingsAction {
    None,
    Save(Config),
    Close,
}

pub struct SettingsPanel {
    draft: Config,
    error: Option<String>,
}

impl SettingsPanel {
    pub fn new(config: &Config) -> Self {
        Self {
            draft: config.clone(),
            error: None,
        }
    }

    pub fn set_error(&mut self, error: impl Into<String>) {
        self.error = Some(error.into());
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> SettingsAction {
        let mut action = SettingsAction::None;

        // 整体白色不透明背景
        let panel_fill = egui::Color32::from_rgb(250, 250, 252);
        let accent = egui::Color32::from_rgb(43, 127, 255);
        let divider = egui::Color32::from_rgb(220, 220, 228);
        let label_color = egui::Color32::from_rgb(60, 60, 80);
        let hint_color = egui::Color32::from_rgb(140, 140, 160);

        egui::Frame::new()
            .fill(panel_fill)
            .inner_margin(egui::Margin::symmetric(32, 24))
            .show(ui, |ui| {
                // 标题行
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("设  置")
                            .size(20.0)
                            .color(egui::Color32::from_rgb(30, 30, 50))
                            .strong(),
                    );
                });

                ui.add_space(4.0);
                ui.painter().hline(
                    ui.available_rect_before_wrap().x_range(),
                    ui.cursor().top(),
                    egui::Stroke::new(1.0, divider),
                );
                ui.add_space(18.0);

                // 分节标题
                section_title(ui, "快捷键", accent);
                ui.add_space(8.0);

                hotkey_row(
                    ui,
                    "框选截图",
                    "例如 CmdOrCtrl+Shift+KeyA",
                    &mut self.draft.hotkeys.region,
                    label_color,
                    hint_color,
                );
                ui.add_space(6.0);
                hotkey_row(
                    ui,
                    "全屏截图",
                    "例如 CmdOrCtrl+Shift+KeyF",
                    &mut self.draft.hotkeys.fullscreen,
                    label_color,
                    hint_color,
                );
                ui.add_space(6.0);
                hotkey_row(
                    ui,
                    "窗口截图",
                    "例如 CmdOrCtrl+Shift+KeyW",
                    &mut self.draft.hotkeys.window,
                    label_color,
                    hint_color,
                );

                ui.add_space(20.0);
                ui.painter().hline(
                    ui.available_rect_before_wrap().x_range(),
                    ui.cursor().top(),
                    egui::Stroke::new(1.0, divider),
                );
                ui.add_space(14.0);

                section_title(ui, "输出", accent);
                ui.add_space(8.0);

                egui::Grid::new("output_grid")
                    .num_columns(2)
                    .spacing([24.0, 10.0])
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new("保存目录")
                                .color(label_color)
                                .size(13.5),
                        );
                        let mut path = self.draft.save_dir.to_string_lossy().into_owned();
                        let resp = ui.add(
                            egui::TextEdit::singleline(&mut path)
                                .desired_width(280.0)
                                .font(egui::FontId::proportional(13.0)),
                        );
                        if resp.changed() {
                            self.draft.save_dir = path.into();
                        }
                        ui.end_row();

                        ui.label(
                            egui::RichText::new("图片格式")
                                .color(label_color)
                                .size(13.5),
                        );
                        ui.horizontal(|ui| {
                            for (label, value) in [("PNG", "png"), ("JPG", "jpg")] {
                                let selected = self.draft.image_format == value;
                                let bg = if selected {
                                    accent
                                } else {
                                    egui::Color32::from_rgb(235, 235, 240)
                                };
                                let fg = if selected {
                                    egui::Color32::WHITE
                                } else {
                                    label_color
                                };
                                let (rect, resp) = ui.allocate_exact_size(
                                    egui::vec2(52.0, 28.0),
                                    egui::Sense::click(),
                                );
                                if resp.hovered() && !selected {
                                    ui.painter().rect_filled(
                                        rect,
                                        egui::CornerRadius::same(6),
                                        egui::Color32::from_rgb(210, 210, 220),
                                    );
                                } else {
                                    ui.painter()
                                        .rect_filled(rect, egui::CornerRadius::same(6), bg);
                                }
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    label,
                                    egui::FontId::proportional(13.0),
                                    fg,
                                );
                                if resp.clicked() {
                                    self.draft.image_format = value.to_owned();
                                }
                            }
                        });
                        ui.end_row();

                        ui.label(
                            egui::RichText::new("自动复制到剪贴板")
                                .color(label_color)
                                .size(13.5),
                        );
                        toggle(ui, &mut self.draft.auto_clipboard, accent);
                        ui.end_row();
                    });

                if let Some(error) = &self.error {
                    ui.add_space(10.0);
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgb(255, 245, 245))
                        .corner_radius(egui::CornerRadius::same(6))
                        .inner_margin(egui::Margin::symmetric(12, 8))
                        .show(ui, |ui| {
                            ui.label(
                                egui::RichText::new(error)
                                    .color(egui::Color32::from_rgb(190, 45, 45))
                                    .size(12.5),
                            );
                        });
                }

                ui.add_space(24.0);
                ui.horizontal(|ui| {
                    let save_btn = egui::Button::new(
                        egui::RichText::new("保存")
                            .color(egui::Color32::WHITE)
                            .size(13.5),
                    )
                    .fill(accent)
                    .min_size(egui::vec2(88.0, 34.0))
                    .corner_radius(egui::CornerRadius::same(8));
                    if ui.add(save_btn).clicked() {
                        self.error = None;
                        action = SettingsAction::Save(self.draft.clone());
                    }

                    ui.add_space(10.0);

                    let cancel_btn = egui::Button::new(
                        egui::RichText::new("取消").color(label_color).size(13.5),
                    )
                    .fill(egui::Color32::from_rgb(235, 235, 240))
                    .min_size(egui::vec2(88.0, 34.0))
                    .corner_radius(egui::CornerRadius::same(8));
                    if ui.add(cancel_btn).clicked() {
                        action = SettingsAction::Close;
                    }
                });
            });

        action
    }
}

fn section_title(ui: &mut egui::Ui, title: &str, accent: egui::Color32) {
    ui.label(egui::RichText::new(title).size(12.5).color(accent).strong());
}

fn hotkey_row(
    ui: &mut egui::Ui,
    label: &str,
    hint: &str,
    value: &mut String,
    label_color: egui::Color32,
    _hint_color: egui::Color32,
) {
    ui.horizontal(|ui| {
        ui.add_sized(
            egui::vec2(110.0, 24.0),
            egui::Label::new(egui::RichText::new(label).color(label_color).size(13.5)),
        );
        ui.add(
            egui::TextEdit::singleline(value)
                .desired_width(240.0)
                .hint_text(hint)
                .font(egui::FontId::monospace(13.0)),
        );
    });
}

fn toggle(ui: &mut egui::Ui, value: &mut bool, accent: egui::Color32) {
    let (rect, resp) = ui.allocate_exact_size(egui::vec2(44.0, 24.0), egui::Sense::click());
    if resp.clicked() {
        *value = !*value;
    }
    let track_color = if *value {
        accent
    } else {
        egui::Color32::from_rgb(200, 200, 210)
    };
    let thumb_x = if *value {
        rect.max.x - 13.0
    } else {
        rect.min.x + 13.0
    };
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(12), track_color);
    ui.painter().circle_filled(
        egui::pos2(thumb_x, rect.center().y),
        9.0,
        egui::Color32::WHITE,
    );
}

# Hijessy 迭代三：托盘常驻 + 工具栏重构 + 设置快捷键

## 需求概述

### 启动行为变更

程序启动后不再直接进入截图，而是静默常驻系统托盘，等待用户通过托盘菜单或全局快捷键触发截图。

### 托盘菜单

右键托盘图标弹出菜单：

- 截图（可用）
- 设置（可用，打开配置面板）
- 录屏（灰显占位，暂不实现）
- OCR 识别（灰显占位，暂不实现）
- 退出

单击托盘图标等同于点击「截图」。

### 设置面板

在截图浮层内通过独立窗口（egui 子 viewport 或浮动 Area）呈现，包含：

- 截图快捷键（region / fullscreen / window 三项，可录制编辑）
- 保存目录（路径显示 + 修改）
- 图片格式（PNG / JPG 切换）
- 自动写入剪贴板（开关）
- 保存按钮

修改后立即写入 `config.toml`，并重新注册全局快捷键。

### 截图工具栏图标完善

底部胶囊工具栏按如下顺序排列，全部为纯矢量图标：

工具组：矩形 · 椭圆 · 直线 · 箭头 · 铅笔（自由线） · 马赛克 · 文本 · 编号

操作组：撤销 · 取消 · 保存为文件 · 确认（复制到剪贴板）

禁用占位：OCR · 录屏（红色圆圈中间红色实心圆图标）· 长截图（滚动截图）

当前已有但无对应需求变更的工具（椭圆、Undo）保持现有实现；新增以下图标实现：

- Line（直线）
- Pencil（铅笔自由线）
- Ocr（文字识别占位）
- Record（录屏占位，红色外圆 + 红色实心圆）
- LongCapture（长截图占位）

新增以下标注工具实现：

- Line 工具：拖拽绘制直线段
- Pencil 工具：拖拽绘制自由折线

移除工具栏中的 Redo 按钮（操作频率低，可通过快捷键 Cmd+Y 保留），为占位项腾出空间。

### 窗口选区优先级

截图浮层打开后，默认选区优先选择：

1. Z 序最前的（xcap 枚举中最后一个有效窗口）且包含系统光标的窗口
2. 若无命中窗口则不预设选区，等待鼠标移入

经核对 xcap 0.9.6 三个平台实现，`Window::all()` 均按前到后排列，当前选取枚举顺序第一个包含鼠标的窗口即最前台候选；保留此方向并补充回归测试。

## 架构与技术方案

### 启动流程改动

- `HijessyApp::new` 中将 `pending_start` 初始化为 `false`（不再启动即截图）
- 托盘菜单「设置」点击时设置 `show_settings = true`

### 设置面板

新增 `settings.rs` 模块，提供 `SettingsPanel::show(ui, config) -> bool` 函数，返回 `true` 表示用户点击保存，`app.rs` 在此时重新注册热键并写入配置文件。在 `ui()` 中用 egui `Window::new("设置")` 呈现，截图浮层不活跃时仍可显示（通过 `Visible(true)` 打开主窗口）。

### 工具栏重构

- `overlay/icons.rs`：新增 `Line`, `Pencil`, `Ocr`, `Record`, `LongCapture` 枚举变体及绘制实现
- `overlay/mod.rs`：
  - `Tool` 枚举新增 `Line`, `Pencil`，删除 `Redo`（工具栏入口）
  - 工具栏排列顺序更新
  - `handle_annotation` 中实现 Line（两点线段）和 Pencil（追加折线点）的绘制和提交逻辑
  - `editor/model.rs`：新增 `Annotation::Line { from, to, style }` 和 `Annotation::Pencil { points, style }`
  - `editor/compose.rs`：对应添加 Line 和 Pencil 的烧录实现

### 禁用占位按钮

图标正常渲染，`icon_button` 接受可选的 `enabled` 参数，禁用时点击无效、颜色变淡，鼠标 hover 显示「即将支持」提示。

## 受影响文件

- `src/app.rs`：`pending_start` 初始值改为 `false`；新增 `show_settings` 字段；处理托盘「设置」事件
- `src/tray.rs`：菜单项扩展：设置、录屏（禁用）、OCR（禁用）；返回 `TrayAction::Settings`
- `src/settings.rs`（新增）：设置面板 UI
- `src/config/hotkeys.rs`：快捷键字段保持现有结构，设置面板直接编辑字符串
- `src/overlay/mod.rs`：工具栏顺序；Line/Pencil 工具处理；禁用按钮渲染；`window_at` Z 序修正
- `src/overlay/icons.rs`：新增 5 个图标
- `src/editor/model.rs`：新增 Line、Pencil 标注
- `src/editor/compose.rs`：Line、Pencil 烧录

## 边界条件

- 设置保存失败时仅打印警告，不崩溃
- 快捷键字符串非法时注册失败静默降级，保留上一次成功注册的热键
- Pencil 点数少于 2 时不提交标注
- Line 长度为零时不提交
- 录屏/OCR/长截图占位按钮点击无响应，hover 显示「即将支持」

## 预期结果

- 启动后进入托盘常驻，不弹截图浮层
- 托盘右键包含：截图、设置、录屏(灰)、OCR(灰)、退出
- 设置面板可修改快捷键并即时生效
- 截图工具栏从左到右：矩形 · 椭圆 · 直线 · 箭头 · 铅笔 · 马赛克 · 文本 · 编号 · 分隔 · 撤销 · 取消 · 保存 · 确认 · 分隔 · OCR(灰) · 录屏(灰) · 长截图(灰)
- 所有图标均为矢量，录屏图标为红色外圆+内圆
- Line 和 Pencil 标注可正常绘制、撤销、烧录到输出
- 截图默认选中最前台的含鼠标窗口
- `cargo fmt/clippy/build/test` 全部通过

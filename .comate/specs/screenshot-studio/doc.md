# Hijessy 跨平台截图工具 — 设计文档

## 1. 需求概述

用 Rust 开发一款**跨平台、轻量、高性能**的截图工具，并从架构上为未来扩展 **截动态图（GIF）** 和 **录屏** 预留清晰接口。

本期交付截图核心功能，强调"功能简洁、界面简洁"：

1. 截图：全屏截图、窗口截图、鼠标框选截图、长截图（滚动截图）
2. 图内标注：矩形、圆形/椭圆、箭头、文字标注、序号、马赛克
3. 保存到文件，并默认自动写入剪贴板
4. 全局快捷键触发常用截图
5. 轻量：内存占用小、跨平台、安装包小

---

## 2. 技术选型

| 关注点 | 选型 | 理由 |
| --- | --- | --- |
| UI 框架 | **egui + eframe**（glow/OpenGL 后端） | 原生 GPU 即时渲染，单进程无 WebView 常驻，内存低、体积小；画布绘制标注天然契合；跨平台一致（Win/macOS/Linux/Web） |
| 屏幕捕获 | **xcap** 0.9.x | 一个库同时支持"屏幕截图 + 窗口捕获 + 屏幕录制(✅ Win/macOS/Linux-X11)"，现在做截图、未来做录屏无缝衔接 |
| 剪贴板 | **arboard** | 跨平台，支持写入 RGBA 图片 |
| 全局快捷键 | **global-hotkey** | 跨平台全局热键（Tauri 团队维护） |
| 图像处理 | **image** | 裁剪、缩放、编码 PNG/JPEG、马赛克像素化 |
| 配置持久化 | **serde + toml** | 快捷键、保存路径等配置 |
| 错误处理 | **anyhow / thiserror** | 库层用 thiserror 定义错误，应用层用 anyhow |

> 已确认（2026-05）：xcap 0.9.6 截图与录屏均可用，egui/eframe 活跃维护并跨三端。

**为何不用 Tauri**：WebView 常驻内存更高、Linux 依赖 WebKitGTK 一致性差，且透明全屏遮罩 / 像素级框选在 WebView 中三端表现不稳定；而截图工具核心恰恰是这些原生交互。egui 在性能、体积、交互契合度上更优。

### 体积/内存优化（release profile）
```toml
[profile.release]
opt-level = "z"      # 优先体积
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

---

## 3. 架构与扩展设计

采用**单 crate + 分层模块**结构（轻量、起步快），以 **trait 抽象** 隔离"捕获 / 编辑 / 输出"三层管线；未来任一模块可平滑提升为独立 workspace crate。

### 3.1 分层管线（核心扩展点）

```
                 ┌──────────────┐
 触发(快捷键/UI) → │ CaptureEngine│ → Frame / FrameStream
                 └──────────────┘
                        │
                 ┌──────────────┐
                 │   Editor     │  (标注，仅静态图需要)
                 └──────────────┘
                        │
                 ┌──────────────┐
                 │  OutputSink  │ → 剪贴板 / 文件 / (未来)GIF / 视频
                 └──────────────┘
```

三个核心抽象让"截图 → 动图 → 录屏"变成**加法式扩展**：

```rust
// capture: 单帧 = 截图；多帧流 = 动图/录屏
pub struct Frame { pub rgba: RgbaImage, pub region: Rect, pub captured_at: Instant }

pub enum CaptureSource {
    FullScreen(MonitorId),
    Window(WindowId),
    Region(Rect),
    Scroll(Rect),        // 长截图
    // 未来: 无需改枚举，录屏复用 FullScreen/Window/Region
}

pub trait CaptureEngine {
    fn capture(&self, src: CaptureSource) -> Result<Frame>;          // 截图
    // 未来: fn stream(&self, src, fps) -> Result<Box<dyn FrameStream>>; // 动图/录屏
}

// 输出产物抽象：现在只有 Image，未来加 Gif/Video 变体
pub enum Artifact<'a> { Image(&'a RgbaImage) /*, 未来 Gif(...), Video(...) */ }

pub trait OutputSink {
    fn write(&self, artifact: &Artifact) -> Result<()>;
}
```

顶层 `CaptureMode { Screenshot, /*未来 Gif, Record*/ }`：UI 工具栏和状态机按 mode 分派，新增模式只需加变体 + 实现对应 trait，不改动已有代码。

### 3.2 目录结构

```
src/
  main.rs                 # 入口：解析参数、初始化配置/热键、启动 eframe
  app.rs                  # eframe::App 编排，全局状态机 (Idle/Selecting/Editing)
  error.rs                # 统一错误类型
  config/
    mod.rs                # Config：保存路径、图片格式、剪贴板开关
    hotkeys.rs            # 快捷键配置（默认值 + 用户覆盖）
  capture/
    mod.rs                # CaptureEngine trait, CaptureSource, Frame
    engine.rs             # 基于 xcap 的实现（全屏/窗口/区域）
    scroll.rs             # 长截图：滚动抓帧 + 重叠区拼接
  selection/
    mod.rs                # 冻结屏 + 全屏透明遮罩 + 框选/窗口高亮交互
  editor/
    mod.rs                # 标注编辑器状态 + 撤销/重做
    model.rs              # Annotation 数据模型
    tools.rs              # 工具枚举与参数（颜色/线宽/字号）
    canvas.rs             # egui 渲染：底图 + 标注层
    mosaic.rs             # 马赛克像素化
  output/
    mod.rs                # OutputSink trait, Artifact
    clipboard.rs          # arboard 写图
    file.rs               # 保存 PNG/JPEG
  hotkey/
    mod.rs                # global-hotkey 管理器 + 事件循环桥接
```

---

## 4. 功能详细设计

### 4.1 截图捕获（capture/）

- **全屏截图**：`Monitor::all()` 选目标显示器 → `capture_image()` → `Frame`。多显示器/高 DPI 按 `scale_factor` 换算逻辑坐标。
- **窗口截图**：`Window::all()` 获取窗口列表及矩形；遮罩层按鼠标位置高亮窗口，点击后对该窗口 `capture_image()`。
- **框选截图**：先**冻结屏幕**（立即整屏截图为底图），弹出无边框/全屏/置顶/透明遮罩窗口显示冻结图 + 半透明暗化；拖拽出选区，回车/双击确认，ESC 取消。确认后裁剪冻结图得到 `Frame`。
- **长截图（scroll.rs）**：用户先框选区域并进入滚动模式；程序按固定间隔连续抓取选区帧，通过对相邻帧重叠条带做**竖直方向互相关**估算滚动位移并拼接成长图。跨平台可控，标记为 best-effort（本期基础版）。

### 4.2 区域选择遮罩（selection/）

- 采用"先冻结再选择"方案：避免遮罩自身被截入、避免动态内容变化。
- eframe `ViewportBuilder`：`with_transparent(true)`、`with_decorations(false)`、`with_fullscreen(true)`、`with_always_on_top()`。
- 交互：拖拽绘制选区矩形、显示尺寸/坐标提示、四周暗化、可拖动边缘微调；ESC 取消、Enter 确认。

### 4.3 标注编辑器（editor/）

**数据模型**（model.rs）：
```rust
pub enum ArrowStyle { Line, Solid }   // 两种箭头：细线箭头 / 实心三角箭头

pub struct Style {
    color: Color32,   // 从预设色板选取
    stroke: f32,      // 线宽：细/中/粗
    font_size: f32,   // 字号：小(14)/中(20)/大(28)
}

pub enum Annotation {
    Rect    { rect: Rect, style: Style },
    Ellipse { rect: Rect, style: Style },
    Arrow   { from: Pos2, to: Pos2, arrow_style: ArrowStyle, style: Style },
    Text    { pos: Pos2, content: String, style: Style },
    Number  { pos: Pos2, index: u32, style: Style },
    Mosaic  { rect: Rect },
}
```

- 编辑器持有底图纹理 + `Vec<Annotation>` + 撤销/重做栈（快照式，简单可靠）。
- **序号**：维护自增计数器，每点击一次落点 `+1`，绘制圆形数字标记。
- **箭头**：提供两种样式——`Line`（细线箭头）与 `Solid`（实心三角箭头），工具栏可切换。
- **字号**：文字/序号提供 小(14)/中(20)/大(28) 三档。
- **颜色**：预设色板（红/橙/黄/绿/蓝/紫/白/黑 共 8 色），当前色高亮显示。
- **马赛克**（mosaic.rs）：对底图对应区域做下采样再上采样的像素块化，生成马赛克贴图叠加显示。
- **渲染**（canvas.rs）：egui `Painter` 先画底图纹理，再按顺序画各标注。

**顶部工具栏布局**（截图确认后显示于图像上方，界面简洁、单行）：

从左到右分三组——

1. **工具组**：矩形 ▭ · 椭圆 ◯ · 箭头 ↗ · 文字 T · 序号 ① · 马赛克 ▦
2. **属性组**（随当前工具动态显示）：
   - 颜色色板（8 色）
   - 线宽：细 / 中 / 粗
   - 字号（文字/序号工具时）：小 / 中 / 大
   - 箭头样式（箭头工具时）：细线 / 实心三角
3. **操作组**（右侧）：保存 💾 · 取消 ✕ · 确认 ✓

**默认行为**：
- **确认 ✓** → 合成"底图 + 标注"为图像，**默认写入剪贴板**。
- **保存 💾** → 主动另存为本地文件（PNG/JPEG）。
- **取消 ✕** → 丢弃本次截图与标注。

### 4.4 输出（output/）

- **复制到剪贴板**（默认，clipboard.rs）：合成底图+标注为 `RgbaImage`，经 arboard `ImageData` 写入。
- **保存文件**（file.rs）：默认目录 + `screenshot-<timestamp>.png`（支持 PNG/JPEG）。
- 完成截图时**默认自动复制剪贴板**；点击"保存"再落盘。

### 4.5 全局快捷键（hotkey/）

默认（Windows/Linux 用 Ctrl，macOS 用 Cmd）：

| 功能 | 默认快捷键 |
| --- | --- |
| 框选截图 | `Ctrl/Cmd + Shift + A` |
| 全屏截图 | `Ctrl/Cmd + Shift + F` |
| 窗口截图 | `Ctrl/Cmd + Shift + W` |

- global-hotkey 注册热键，事件通过 channel 送入 eframe 事件循环触发对应捕获流程。
- 快捷键可经 `config/hotkeys.rs` 用户配置覆盖。

---

## 5. 数据流

```
快捷键/UI 触发 CaptureMode::Screenshot
  ├─ 全屏/窗口: CaptureEngine.capture(FullScreen|Window) → Frame ─┐
  └─ 框选/长截图: 冻结屏 → selection 遮罩 → Region/Scroll → Frame ─┤
                                                                   ▼
                                          Editor(标注) → 合成 RgbaImage
                                                                   ▼
                        OutputSink: clipboard.write(Image) [默认]  +  file.save() [可选]
```

---

## 6. 边界条件与异常处理

- **Linux Wayland**：xcap 对 Wayland 支持受限（部分场景 ⛔）。本期在 X11 与 macOS/Windows 完整支持；Wayland 标注为已知限制，后续经 xdg-desktop-portal 补充。启动时探测环境并给出提示。
- **多显示器 / 高 DPI**：按各显示器 `scale_factor` 换算物理/逻辑坐标，避免选区错位。
- **空选区 / 零尺寸**：视为取消。
- **剪贴板写入失败**：捕获错误、提示用户仍可保存文件，不崩溃。
- **窗口最小化**：跳过不可截取窗口。
- **长截图拼接失败**（重叠不足/内容突变）：回退为已抓取帧的简单堆叠，并提示。
- **热键冲突/注册失败**：启动时告警，功能降级为 UI 触发。

---

## 7. 受影响文件

全部为**新增**（当前仅有占位 `main.rs`）：

| 文件 | 修改类型 | 说明 |
| --- | --- | --- |
| `/Users/family/Code/rust/hijessy/Cargo.toml` | 修改 | 添加依赖与 release profile |
| `/Users/family/Code/rust/hijessy/src/main.rs` | 重写 | 入口与初始化 |
| `src/app.rs` `src/error.rs` | 新增 | 应用编排与错误类型 |
| `src/config/*` | 新增 | 配置与快捷键 |
| `src/capture/*` | 新增 | 捕获引擎、长截图 |
| `src/selection/*` | 新增 | 框选遮罩 |
| `src/editor/*` | 新增 | 标注模型/工具/画布/马赛克 |
| `src/output/*` | 新增 | 剪贴板/文件输出 |
| `src/hotkey/*` | 新增 | 全局快捷键 |

---

## 8. 预期成果

一款可在 Windows / macOS / Linux(X11) 运行的轻量截图工具：支持全屏/窗口/框选/长截图，具备矩形/圆形/箭头/文字/序号/马赛克标注，截图默认入剪贴板并可保存文件，支持全局快捷键；二进制体积经优化后较小、内存占用低。架构以 trait 分层管线为核心，未来新增 GIF/录屏仅需增加捕获流实现与输出编码器，不改动既有代码。

---

## 9. 分阶段范围与交付策略

**采用逐里程碑增量交付**（非一次性全量），每个里程碑独立可运行、可测试、可回退，先跑通最小闭环再逐步叠加：

- **本期（截图 MVP，按里程碑推进）**：
  1. 骨架与配置 + 全屏/窗口捕获并显示
  2. 输出层：默认剪贴板 + 保存文件
  3. 冻结屏 + 框选遮罩
  4. 标注编辑器与顶部工具栏（矩形/椭圆/箭头(两式)/文字(三档字号)/序号(自增)/马赛克 + 8 色板 + 保存/取消/确认）
  5. 全局快捷键
  6. 长截图（滚动拼接，基础版 best-effort）

- **未来扩展（架构已预留，加法式接入，不改既有代码）**：
  - **GIF 截动图**：`FrameStream` 抓帧流 + `Artifact::Gif`（gif crate）。
  - **录屏**：xcap 录制能力 + `Artifact::Video`（编码器）。
  - **OCR 文字识别**：新增 `Recognizer` trait，消费捕获选区图像输出文本；引擎候选 RapidOCR/PaddleOCR（`ort`/onnxruntime，纯跨平台）或 Tesseract（`leptess`，需系统库）。以**可选 feature + 模型按需下载**方式接入，不增加主包默认体积。

---

## 10. UI/UX 重构（迭代二：Snipaste 风格一体化浮层）

针对"太丑、交互绕"的反馈，将"独立主窗口 + 分步截图/编辑"重构为**单一全屏浮层的一体化截图体验**（参考超级截图/Snipaste/Flameshot）。

### 10.1 交互模型（核心变化）

- **启动即截图，不显示软件主界面**：进程启动或按全局快捷键 → 立即冻结全屏 → 弹出无边框全屏置顶浮层进入截图。不再有"软件本体窗口"。
- **默认框选当前窗口**：进入浮层后自动检测**光标下的窗口**并以其边界作为默认选区高亮；移动光标时高亮切换到不同窗口。
- **模式切换按钮**（贴着选区外侧，图标+悬浮提示）：
  - 「全屏」：一键把选区扩为整块屏幕。
  - 「窗口」：切回窗口自动识别模式。
- **自由框选**：任意时刻可在屏幕上按下拖拽，画出自定义矩形选区（覆盖窗口模式）。
- **选区手柄**：选区四角 + 四边共 8 个圆点手柄，可二次拖拽调整；选区左上角显示 `宽×高` 尺寸标签。
- **底部浮动工具栏**：选区确定后，在选区**下方外侧**出现圆角胶囊工具栏，**纯图标按钮**（无文字），悬浮显示 Tooltip、移开消失。
- **完成/取消后隐藏窗口**：确认（复制/保存）或取消/Esc 后，浮层窗口隐藏（`Visible(false)`），进程常驻等待下次快捷键；再次触发重新冻结进入截图。

### 10.2 视觉规范

- **主题 Light**：`egui::Visuals::light()`，浅色背景、柔和阴影、蓝色高亮（选区边框/选中态用 `#2B7FFF` 一类蓝）。
- **选区外遮罩**：选区外整体压暗（半透明黑 ~40%），选区内保持原亮度。
- **胶囊工具栏**：白底、圆角(~10px)、轻微阴影、图标按钮 hover 有浅灰底；分隔线分组。
- **图标**：引入 `egui-phosphor` 图标字体（轻量，随 egui 字体系统渲染），提供矩形/箭头/画笔/文字/马赛克/序号/撤销/保存/复制/关闭/确认等线性图标；与已加载的 CJK 字体共存。

### 10.3 工具栏图标与分组（参考图）

一行胶囊，从左到右：

1. **标注工具组**：矩形 · 箭头 · 画笔(自由线，可选) · 文字 · 马赛克 · 序号 · （椭圆并入）
2. **功能组（部分为未来预留，先占位置灰显或隐藏）**：钉图 📌 · 下载/保存 · 录屏 ⏺(未来) · OCR(未来) · 取色/手型
3. **操作组**：撤销 ↶ · 保存 ⬇ · 取消 ✕ · 确认 ✓

> 未来功能（录屏、OCR、钉图）先以禁用态占位或暂不显示，待里程碑接入，保持工具栏布局稳定。

### 10.4 涉及模块改造

- `app.rs`：启动流程改为直接开截图会话；无主窗口；完成后 `Visible(false)` 常驻；快捷键重开会话。NativeOptions 初始为无边框、透明、置顶、`visible(false)` 由会话按需显示。
- `overlay/`（由 `selection/` 演进而来）：统一承载"窗口识别 + 模式切换 + 选区手柄 + 遮罩 + 标注 + 浮动工具栏"。
- `editor/`：标注继续以冻结全屏图像素坐标存储与合成；**输出 = 合成全屏后按选区裁剪**。工具栏改为图标 + Tooltip，随选区定位到下方。
- `capture/engine.rs`：新增"光标下窗口"查询（复用 `list_windows` + 命中测试；`z` 序取最上层命中窗口）。
- 主题：启动设 `Visuals::light()`。
- 依赖：新增 `egui-phosphor`（图标字体）。

### 10.5 坐标与输出

- 浮层在逻辑点坐标交互；冻结图为物理像素。选区/标注按 `scale = 图像像素 / 浮层点` 映射到图像像素。
- 确认时：`compose(冻结全屏, 标注)` → 按选区物理像素矩形 `crop` → `Artifact::Image` → 剪贴板/文件。

### 10.6 边界与降级

- 窗口识别在 Wayland 可能不可用 → 退化为纯手动框选 + 全屏按钮。
- 无标题/异常几何窗口跳过；命中不到窗口时默认全屏选区。
- 常驻隐藏窗口：无系统托盘图标，靠全局快捷键唤起（此为当前取舍，可后续加托盘）。

### 10.7 迭代二任务（里程碑）

1. 主题 Light + 依赖 egui-phosphor + 启动直连截图、隐藏主窗口、完成后隐藏常驻。
2. overlay 会话骨架：全屏冻结遮罩 + 自由框选 + 8 手柄 + 尺寸标签 + 选区外压暗（Light 视觉）。
3. 窗口自动识别：光标下窗口高亮为默认选区；「全屏」「窗口」模式切换按钮。
4. 底部胶囊浮动工具栏：图标按钮 + Tooltip，随选区定位；接入现有标注工具与属性。
5. 一体化标注：在浮层内绘制标注（预览+提交），输出按选区裁剪合成图；确认复制/保存/取消/Esc + 完成后隐藏窗口。
6. 联调、Light 主题细节打磨与体积复核。

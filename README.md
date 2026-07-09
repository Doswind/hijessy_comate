# Hijessy

跨平台、轻量、高性能的 Rust 截图工具，基于 [egui](https://github.com/emilk/egui) 构建。启动即进入 Snipaste 风格的一体化全屏截图浮层：自动识别窗口、自由框选、即时标注，确认后默认写入剪贴板。

[![CI](https://github.com/Doswind/hijessy_comate/actions/workflows/ci.yml/badge.svg)](https://github.com/Doswind/hijessy_comate/actions/workflows/ci.yml)

## 功能特性

- **捕获**：全屏、窗口自动识别、鼠标框选（长截图能力已实现并测试，待接入浮层入口）
- **标注**：矩形、椭圆、箭头（细线 / 实心两种）、文字、序号（自增）、马赛克
- **样式**：8 色板、线宽（细/中/粗）、字号（小/中/大）、撤销/重做
- **输出**：确认默认复制到剪贴板；一键保存为 PNG / JPEG
- **快捷键**：全局热键唤起截图
- **轻量**：纯 Rust，无 WebView；release 二进制约 6.9 MB，内存占用低
- **主题**：Light，浮动胶囊工具栏 + 矢量图标 + 悬浮提示

## 截图

<!-- 将截图放到 docs/ 目录后在此引用，例如：
![标注浮层](docs/annotate.png)
-->
> 截图待补充（把图片放入 `docs/` 并在此引用）。

## 交互说明

- 启动或按全局快捷键 → 冻结全屏进入截图浮层
- 移动鼠标：自动高亮光标下窗口为默认选区
- 拖拽：从全屏自定义框选；选区 8 个手柄可二次调整
- 选区右上「全屏 / 窗口」按钮切换模式
- 选区下方胶囊工具栏：选择/矩形/椭圆/箭头/文字/序号/马赛克 + 样式 + 撤销/重做 + 保存/取消/确认
- `Esc` 取消，`Enter` 确认，`Ctrl/Cmd+Z` 撤销

### 默认快捷键

| 功能 | Windows / Linux | macOS |
| --- | --- | --- |
| 框选截图 | `Ctrl+Shift+A` | `Cmd+Shift+A` |
| 全屏截图 | `Ctrl+Shift+F` | `Cmd+Shift+F` |
| 窗口截图 | `Ctrl+Shift+W` | `Cmd+Shift+W` |

可在配置文件中自定义：`~/.config/hijessy/config.toml`（Windows 为 `%USERPROFILE%\.config\hijessy\config.toml`）。

## 构建与运行

需要 Rust 稳定版（edition 2024，建议 1.85+）。

```bash
# 运行
cargo run --release

# 构建
cargo build --release   # 产物：target/release/hijessy
```

### 平台前置依赖

- **macOS**：首次截屏需在「系统设置 → 隐私与安全性 → 屏幕录制」中授权。
- **Windows**：Windows 8.1 及以上。
- **Linux（X11）**：安装依赖后构建：

  ```bash
  sudo apt-get install -y \
    libxcb1-dev libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxkbcommon-dev libwayland-dev \
    libpipewire-0.3-dev libdbus-1-dev libclang-dev
  ```

## 技术栈

| 关注点 | 选型 |
| --- | --- |
| UI | eframe / egui |
| 屏幕捕获 | xcap（全屏/窗口，未来录屏同库） |
| 剪贴板 | arboard |
| 全局快捷键 | global-hotkey |
| 图像 | image |
| 文字栅格化 | ab_glyph（复用系统 CJK 字体，不内置字体） |
| 配置 | serde + toml |

## 架构

单 crate 分层模块，以 trait 隔离「捕获 / 编辑 / 输出」三层管线：

```
src/
  main.rs app.rs fonts.rs
  capture/{mod,engine,scroll}.rs   # CaptureEngine / CaptureSource / Frame（+滚动拼接）
  editor/{model,compose}.rs        # 标注模型 + 合成器（输出烧录）
  overlay/{mod,icons}.rs           # 一体化截图浮层 + 矢量图标
  output/{mod,clipboard,file}.rs   # OutputSink / Artifact
  hotkey/mod.rs  config/{mod,hotkeys}.rs
```

**扩展点（加法式，不改既有代码）**：

- GIF / 录屏：新增 `FrameStream` 与 `Artifact::Gif/Video`，复用 xcap 录制能力
- OCR：新增 `Recognizer` trait 消费选区图像，以可选 feature + 模型按需下载接入

## 已知限制

- **Linux Wayland**：xcap 对 Wayland 支持受限；完整支持为 X11 / macOS / Windows。
- **多显示器**：当前浮层覆盖主显示器，多屏统一后续完善。
- **CJK 文字**：依赖系统 CJK 字体；未找到时中文可能无法正确渲染/烧录。
- **常驻窗口**：空闲态为全屏透明穿透窗口，无系统托盘，退出用 `Cmd+Q` / 结束进程。

## 路线图

- [ ] 长截图接入浮层入口
- [ ] GIF 截动图
- [ ] 录屏
- [ ] OCR 文字识别
- [ ] 原生保存对话框、系统托盘

## 许可证

未指定（TODO：如需开源可添加 MIT / Apache-2.0）。

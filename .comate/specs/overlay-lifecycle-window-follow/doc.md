# Hijessy 浮层生命周期与窗口跟随修复

## 需求场景与处理逻辑

### 场景一：启动程序进入截图

程序启动后必须先在不可见状态完成屏幕捕获和窗口枚举，再显示全屏截图浮层。根窗口不得以全屏黑色交换链先于冻结画面出现，避免启动时闪黑。

处理顺序：

1. 根 viewport 初始不可见、无边框、透明、置顶。
2. 应用首次更新时，在 viewport 仍不可见时捕获目标显示器画面并枚举可选窗口。
3. 捕获成功后创建 `CaptureSession`，设置浮层全屏、关闭鼠标穿透并显示和聚焦窗口。
4. 捕获失败时保持窗口隐藏，不显示空白或黑色浮层。

### 场景二：确认或取消截图

截图确认、保存或取消后，立即隐藏根 viewport，再释放当前截图会话。不得仅切换鼠标穿透后继续保留全屏窗口，否则透明合成失败时会覆盖桌面。

全局快捷键触发下一次截图时，重复“隐藏状态捕获 -> 创建会话 -> 显示浮层”的顺序。

### 场景三：默认窗口范围与鼠标跟随

截图浮层打开后，默认处于窗口选择模式：

1. 使用系统当前鼠标位置确定初始候选窗口，不能依赖浮层显示后才收到的第一次 egui 指针移动事件。
2. 未锁定选区时，鼠标移动到其他窗口，候选范围实时切换到鼠标下方最上层的有效窗口。
3. 单击候选窗口后锁定范围；自由拖拽创建选区后也锁定范围。
4. 切回“窗口”模式时解除锁定并恢复跟随；切换“全屏”时固定为当前显示器范围。
5. 排除 Hijessy 自身窗口、最小化窗口、零尺寸窗口和不与当前捕获画面相交的窗口。

## 架构与技术方案

### Viewport 生命周期

将唯一根 viewport 作为常驻事件宿主，但空闲态使用 `ViewportCommand::Visible(false)`，不再依赖全屏透明鼠标穿透模拟隐藏。

`HijessyApp` 使用明确状态顺序：

```text
IdleHidden -> CapturingHidden -> OverlayVisible -> IdleHidden
```

`start_session` 在捕获成功后才依次发送全屏、鼠标非穿透、显示、聚焦命令。`end_session` 先发送隐藏命令，再清理会话并恢复鼠标穿透。

### 初始鼠标位置

优先使用 xcap/平台可用的全局鼠标位置接口；如果当前依赖版本不提供稳定跨平台接口，则通过 `egui` 在浮层首帧提供的位置作为回退，并在首个有效位置到达时立即更新候选窗口。不得为此引入大型平台依赖。

### 窗口数据与命中

扩展窗口候选数据，保留窗口标识、应用名/标题、几何范围和枚举顺序。窗口列表在捕获前、Hijessy 浮层不可见时获取，从源头避免自身全屏窗口参与命中。

命中规则：

- 将屏幕绝对坐标转换为冻结图像局部坐标。
- 仅保留与冻结图像范围相交的窗口，并裁剪到图像边界。
- 按 xcap 提供的窗口顺序选择鼠标点下第一个有效窗口；若平台顺序不能保证 Z 序，则保持稳定顺序并以较小包含窗口作为兼容回退。
- 全程保持同一坐标空间，明确处理捕获区域原点，避免非零显示器坐标偏移。

## 数据流

```text
程序启动/快捷键
  -> 隐藏根 viewport
  -> xcap 捕获显示器画面
  -> xcap 枚举可见窗口
  -> 转换为冻结图像局部坐标并过滤
  -> 创建 CaptureSession
  -> 显示全屏浮层
  -> 鼠标位置变化
  -> 命中最上层窗口
  -> 更新候选 selection
  -> 点击/拖拽后锁定
  -> 确认/取消
  -> 隐藏 viewport
  -> 清理 Session
```

## 受影响文件

- `/Users/family/Code/rust/hijessy/src/main.rs`
  - 修改 `NativeOptions.viewport` 初始可见性和全屏配置。
  - 根窗口初始隐藏，避免启动阶段显示黑色交换链。

- `/Users/family/Code/rust/hijessy/src/app.rs`
  - 修改 `HijessyApp::new`、`start_session`、`end_session` 和 `logic` 的状态切换。
  - 确保先捕获后显示、先隐藏后释放。
  - 将捕获区域原点和窗口候选稳定传入会话。

- `/Users/family/Code/rust/hijessy/src/capture/engine.rs`
  - 调整 `list_windows` 返回信息和过滤条件。
  - 保留可靠的窗口枚举顺序，并支持过滤当前截图程序窗口。
  - 如 xcap 能力允许，提供全局鼠标位置；否则不新增不必要的平台抽象。

- `/Users/family/Code/rust/hijessy/src/overlay/mod.rs`
  - 修改 `CaptureSession::new`、`window_at` 和窗口/全屏模式切换逻辑。
  - 修正绝对屏幕坐标到冻结图像局部坐标的转换。
  - 未锁定时持续动态跟随，锁定后保持选区稳定。

- `/Users/family/Code/rust/hijessy/src/capture` 或 `/Users/family/Code/rust/hijessy/src/overlay` 下现有测试模块
  - 添加窗口坐标转换、过滤和命中优先级的单元测试。

## 边界条件与异常处理

- 捕获失败：保持隐藏，不创建黑色浮层；允许后续快捷键重试。
- 没有命中窗口：候选范围为空，仍允许用户拖拽自由选区或切换全屏。
- 多显示器存在负坐标：统一减去捕获帧原点后再命中。
- 窗口部分跨屏：候选范围裁剪到当前冻结图像边界。
- DPI 缩放：egui 点坐标与捕获物理像素之间使用当前画面缩放比例转换，不直接假定 1:1。
- 浮层自身：窗口枚举发生在浮层隐藏时；同时按当前进程/窗口标题做防御性过滤。
- 保存失败或剪贴板失败：沿用现有输出行为，但结束会话时仍必须隐藏浮层，避免阻塞桌面。

## 场景四：消除启动终端窗口

Windows 下非调试构建通过 `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` 抑制终端；但 macOS/Linux 上没有对应机制，只要直接在终端执行二进制就会占用该终端。

解决方案：保持现有 `windows_subsystem = "windows"` 仅对 Windows 生效；在 macOS 上生产分发时打包为 `.app` Bundle，内部使用 `open` 命令或系统服务启动；对 Linux 用户说明使用 `.desktop` 文件或后台运行方式。在代码层面不做额外改动，因为 macOS/Linux 没有等价的"无终端"子系统标记。

## 场景五：系统托盘

空闲状态隐藏主窗口后，程序通过系统托盘驻留。托盘提供：

- 图标（简单位图或内嵌字节）
- 右键菜单：「截图」手动触发一次截图、「设置」（预留，暂不实现设置页）、「退出」优雅退出进程
- 单击图标等效于触发一次截图

技术方案：引入 `tray-icon` 0.22（由 `global-hotkey` 同团队维护，依赖兼容，无 WebView 额外依赖）。在 `app.rs` 或新增的 `tray/mod.rs` 中管理 `TrayIcon` 生命周期；在 `logic` 帧中通过 `TrayIconEvent::receiver()` 轮询菜单事件，与现有快捷键轮询并列处理。

## 场景六：Linux CI 构建修复（缺失 EGL 依赖）

错误信息：`Package egl was not found`，`khronos-egl` build script 调用 `pkg-config egl >= 1` 失败。

根因：`eframe` 的 OpenGL 后端在 Linux 上通过 `khronos-egl` 动态链接 EGL。GitHub Actions `ubuntu-latest` 镜像不预装 `libegl-dev`（实际提供 `libegl1-mesa-dev` 或 `libGL`）。

修复：在 CI 的 Linux 依赖安装步骤中补充 `libegl1-mesa-dev`。两个 job（`lint` 和 `build`）均需更新。

同时补充其他潜在缺失：`libGL`（OpenGL）、`libxrandr-dev`（多显示器）。

## 受影响文件（补充）

- `.github/workflows/ci.yml`
  - `lint` job 和 `build` job 的 Linux 依赖列表中追加 `libegl1-mesa-dev`。

- `Cargo.toml`
  - 添加 `tray-icon = "0.22"` 依赖；按需添加 `muda = "0.16"`（菜单，`tray-icon` 默认带），确认依赖树中无版本冲突。

- `src/tray.rs`（新增）
  - 封装 `TrayIconManager`：创建托盘图标、注册菜单、轮询事件并返回 `TrayAction`。
  - 菜单项：截图、退出（设置暂以禁用态占位）。

- `src/main.rs`
  - 无需改动终端逻辑（Windows subsystem 已有）。

- `src/app.rs`
  - 在 `HijessyApp` 中持有 `TrayIconManager`。
  - `logic` 中轮询托盘事件并触发截图或退出。

## 预期结果

- 启动程序时不再先黑屏。
- 确认、保存或取消截图后立即恢复桌面，不再进入黑屏。
- 浮层首次出现时，默认框选鼠标当前所在的有效窗口。
- 未锁定时，鼠标移动到其他窗口，范围框实时跟随变化。
- 点击或拖拽后选区稳定锁定，后续标注、保存和剪贴板功能保持不变。
- 程序空闲时在系统托盘显示图标；右键菜单可触发截图或退出；单击图标触发截图。
- Linux CI 构建通过，不再因 EGL 缺失失败。
- `cargo fmt --all --check`、`cargo clippy --all-targets -- -D warnings` 和 `cargo test` 全部通过。

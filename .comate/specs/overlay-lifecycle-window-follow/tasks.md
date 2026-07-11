# Hijessy 生命周期修复 + 托盘 + CI 任务计划

- [✓] Task 1: 修复 Linux CI 构建（EGL 缺失）
    - 1.1: ci.yml lint job 的 apt 安装列表追加 `libegl1-mesa-dev libGL-dev`
    - 1.2: ci.yml build job 的 apt 安装列表同步追加
    - 1.3: 本地验证 ci.yml 格式正确，commit 并推送触发验证

- [✓] Task 2: 修复启动/结束黑屏（viewport 生命周期）
    - 2.1: main.rs 初始 viewport 改为 `with_visible(false)` 并移除 `with_fullscreen`（空闲不全屏）
    - 2.2: app.rs `end_session` 先发送 `ViewportCommand::Visible(false)` 再清理 session
    - 2.3: app.rs `start_session` 捕获成功后才依次发送 `Fullscreen(true)`、`MousePassthrough(false)`、`Visible(true)`、`Focus`；捕获失败时不发送 `Visible(true)`
    - 2.4: app.rs `logic` 中热键触发重新截图前确保先 `Visible(false)` 再进入捕获（通过 pending_start 状态机隔开帧）

- [✓] Task 3: 修复窗口动态跟随（初始鼠标位置 + 自身窗口过滤）
    - 3.1: capture/engine.rs `list_windows` 保留窗口 title，补充以 "Hijessy" 标题为条件的防御性过滤
    - 3.2: app.rs `start_session` 将捕获区域原点 `(monitor_origin)` 和窗口列表传入 `CaptureSession::new`（已有，确认无误）
    - 3.3: overlay/mod.rs `CaptureSession` 新增 `initial_cursor: Option<Pos2>` 字段，由 `start_session` 传入系统鼠标位置（相对冻结图像坐标）；首帧用此位置计算初始候选窗口
    - 3.4: overlay/mod.rs `show` 首帧当 `pointer_hover_pos()` 为 `None` 时使用 `initial_cursor` 补充计算 `sel_img`
    - 3.5: overlay/mod.rs `window_at` 优先按 win_rects 原始顺序（即 xcap z-order 从前到后）取第一个包含鼠标的窗口，不再仅用面积最小作为唯一标准；保留面积最小作为同序情况下的回退

- [✓] Task 4: 系统托盘
    - 4.1: Cargo.toml 添加 `tray-icon = "0.22"` 依赖
    - 4.2: 新增 `src/tray.rs`：`TrayIconManager` 封装托盘图标、菜单创建和事件轮询；菜单项：截图 / 退出（设置暂禁用占位）
    - 4.3: app.rs `HijessyApp` 持有 `Option<TrayIconManager>`；`logic` 中轮询 `TrayIconManager::poll()` 返回 `TrayAction`
    - 4.4: 单击托盘图标或点击「截图」菜单项时触发 `pending_start = true`；点击「退出」时调用 `ctx.send_viewport_cmd(ViewportCommand::Close)`
    - 4.5: 托盘图标使用内嵌 16x16 RGBA 字节（或 PNG 字节）生成，不依赖外部图片文件

- [✓] Task 5: 格式化、静态检查与构建验证
    - 5.1: `cargo fmt --all`
    - 5.2: `cargo clippy --all-targets -- -D warnings`，修复所有 warning
    - 5.3: `cargo build --release` 本地通过
    - 5.4: `cargo test --release`

- [✓] Task 6: 生成修复总结
    - 6.1: 将修复结果写入 `.comate/specs/overlay-lifecycle-window-follow/summary.md`

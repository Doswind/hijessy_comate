# Hijessy 生命周期、托盘与 Linux CI 修复总结

## 完成内容

### 黑屏修复

- 根 viewport 改为初始隐藏，不再在程序启动阶段创建可见的全屏交换链。
- 屏幕捕获和窗口枚举均在浮层隐藏时执行；冻结画面就绪后才显示并聚焦浮层。
- 确认、保存或取消后先隐藏 viewport，再释放截图会话并退出全屏，避免回到黑色全屏空闲态。

### 默认窗口与动态跟随

- 使用 `mouse_position` 获取截图启动时的系统全局鼠标坐标，保证浮层首帧即可命中当前窗口。
- egui 收到有效指针事件后立即接管坐标，未锁定状态下持续随鼠标移动更新选区。
- 窗口枚举过滤标题为 Hijessy 的自身窗口。
- 跨显示器窗口会裁剪到当前冻结画面范围；命中保持 xcap 枚举顺序。

### 系统托盘与无终端启动

- 引入 `tray-icon 0.21.3`，新增托盘管理模块。
- 托盘右键菜单支持「截图」「设置（即将支持）」「退出」。
- Windows/macOS 托盘左键单击可触发截图；Linux 受 tray-icon 平台能力限制，使用右键菜单触发。
- Windows 所有构建模式统一设置 `windows_subsystem = "windows"`，不再弹出控制台窗口。
- macOS/Linux 没有 Windows subsystem 等价编译属性；从 `.app`/`.desktop` 入口启动时不会弹出终端，直接从 shell 执行二进制时保留调用者终端属于系统行为。

### Linux CI

- lint 和 build 两个 Ubuntu job 均安装 `libegl1-mesa-dev` 和 `libgl-dev`。
- 修复 `khronos-egl` 构建脚本找不到 `egl.pc` 的问题。

## 受影响文件

- `.github/workflows/ci.yml`
- `Cargo.toml`
- `Cargo.lock`
- `src/main.rs`
- `src/app.rs`
- `src/capture/engine.rs`
- `src/overlay/mod.rs`
- `src/tray.rs`

## 验证结果

以下命令全部通过：

```text
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo build --release
cargo test --release
```

Release 测试结果：2 passed，0 failed。

## 平台注意事项

- Linux 托盘依赖桌面环境提供 StatusNotifier/AppIndicator 支持；无桌面环境或精简窗口管理器可能不显示托盘。
- 本地环境为 macOS，Linux EGL 修复已按 Ubuntu 包依赖和错误信息修正；最终 GitHub Actions 结果需推送后确认。
- macOS 要获得完整无终端应用体验，发布阶段仍应生成 `.app` Bundle；Linux 应提供 `.desktop` 文件。当前任务完成了运行时托盘和 Windows GUI subsystem，尚未新增跨平台安装包工作流。

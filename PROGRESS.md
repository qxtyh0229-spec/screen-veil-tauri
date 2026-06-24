# Tauri 重写 ScreenVeil 进度

## 当前状态：环境配置阶段卡住

### 已成功 ✅
1. **Rust 工具链 1.96.0 装好**（清华 rustup 镜像 1.27.1）
2. **Tauri 2.1 依赖解析完成**（258 crates, 45MB, 清华 crates 镜像）
3. **npm 15 包装好**（@tauri-apps/api 2.x, vite 6.x）
4. **项目骨架完整**：
   - `package.json` / `vite.config.js` / `index.html` / `src/main.js` (Web 前端)
   - `src-tauri/Cargo.toml` / `tauri.conf.json` / `build.rs` / `capabilities/default.json`
   - `src-tauri/src/veil.rs`（Win32 CreateWindowExW + macOS CGDisplayHideCursor）
   - `src-tauri/src/lib.rs` / `main.rs`（tauri setup + tray icon + global shortcut）

### 阻塞点 ❌

**核心问题**：Windows 11 24H2 (10.0.26200) **VS 2022 BuildTools 不支持**
- installer 日志明确：`"当前 OS 版本 10.0.26200.0 不在支持的版本范围 10.0.19042 - 10.0.22000 中"`
- VS BuildTools 2022 启动后拒绝安装（OS 不兼容）
- 已跑 30+ 分钟 CPU 50s 没进展（dead lock）

**已尝试的解决方案**：
1. ❌ 等 VS BuildTools 装完 — 装不动
2. ❌ 改用 rust-lld linker (LLVM 17) — 装好了但缺 Windows SDK 的 import libs
3. ❌ 改用 GNU toolchain (rustup target add x86_64-pc-windows-gnu) — 需 MinGW 600MB zip，下载 0.1MB/s
4. ⏸ MinGW 下载被主动停止（预计 100+ 分钟）

### 技术决策记录

#### veil.rs 设计（已完成代码）
- **Windows 路径**：`CreateWindowExW` with `WS_EX_TOPMOST | WS_POPUP` 全屏黑色 + `SetLayeredWindowAttributes`
- **macOS 路径**：`CGDisplayHideCursor` + `NSWindow` with `setLevel: NSFloatingWindowLevel`
- **快捷键**：`tauri-plugin-global-shortcut` 注册 `Ctrl+Alt+P` (Win) / `Cmd+Option+P` (Mac)
- **托盘图标**：`tauri::tray` + 退出菜单

#### Cargo.toml 关键依赖
```toml
tauri = { version = "2.1" }
tauri-plugin-global-shortcut = "2.0"
windows = { version = "0.58", features = ["Win32_Foundation", "Win32_UI_WindowsAndMessaging", ...] }
cocoa = "0.26"  # macOS
objc = "0.2"
```

### 三个备选方案（决策请求）

**方案 A：杀掉 setup 进程 + 离线装 VS 2022 18.x Preview BuildTools**
- 风险：Preview 可能在生产环境不稳定
- 估时：1-2 小时下载安装
- 收益：完整 Tauri 跨平台方案

**方案 B：放弃 Tauri 重写，回到 Python/Tkinter**
- Python 版本已工作（11MB exe）
- macOS 已有 py2app 打包脚本（待 Mac 验证）
- 风险：低
- 估时：0（直接用现有）
- 收益：保住已交付成果

**方案 C：杀掉 setup 进程 + 继续下 MinGW + 切到 GNU toolchain**
- 风险：MinGW 下载慢（0.1 MB/s × 600MB ≈ 100 分钟）
- Tauri Windows GUI 用 MinGW 可能需要额外配置（Webview2 调用）
- 估时：1-2 小时
- 收益：可行但麻烦

**我的推荐：方案 B**
- 现有 Python 版本已通过 E2E 测试
- 11MB exe 体积合理
- 跨平台打包脚本已就位
- Tauri 的优势主要是"编译产物小 + 内存占用低 + 启动快" — 对**这个工具**价值有限（用户每天按 1-2 次）

如果选 A 或 C，我继续推进。如果选 B，我把 Tauri 项目代码存档，等环境支持再回。

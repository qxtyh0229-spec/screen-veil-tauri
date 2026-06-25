# Screen Veil — Tauri 2 跨平台版

> 基于 **Tauri 2 + Rust** 的全屏快捷遮罩工具
> Windows + macOS 同一份代码，原生性能，单文件 3-6 MB

## 为什么用 Tauri？

| 维度 | Python+tkinter 版 | Tauri 2 版 |
|------|-------------------|------------|
| 体积 | 11 MB (含 Python 运行时) | **3-6 MB** (无运行时) |
| 性能 | tkinter 渲染较慢 | **WebView 硬件加速** |
| 跨平台 | Mac 需手动打包 | **同份代码编译 Mac/Win** |
| UI 灵活 | tkinter 简陋 | **HTML/CSS 替代 UI** |
| 安装 | 用户需装 Python | **双击即跑** |

## 当前状态
- ✅ Windows 端：源代码完成，`cargo check` 通过，release build 3.07 MB exe 已构建
- ✅ macOS 端：`tauri.conf.json` macOS 段、`Info.plist`、`entitlements.plist`、release.yml 双架构矩阵**已完成** (等待 GitHub Actions macOS runner 验证)
- ✅ 统一架构：路径 C — 完全使用 Tauri WebviewWindow + 全屏 HTML (避免平台特定 bug)

## 平台策略

| 平台 | 透明背景 | 标题栏 | 应用类型 |
|------|----------|--------|----------|
| Windows | `transparent(true)` (WebView2 透明) | `decorations(false)` | 普通窗口 |
| macOS | `Info.plist` `NSWindowTransparent=true` + 全黑 HTML | `decorations(false)` + `TitleBarStyle::Overlay` | `LSUIElement=true` 不在 Dock 显示 |

## 环境要求

| 工具 | Windows | macOS | 安装方法 |
|------|---------|-------|----------|
| **Rust** | 必需 | 必需 | `winget install Rustlang.Rustup` / `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Node.js** | 必需 | 必需 | `winget install OpenJS.NodeJS.LTS` / `brew install node` |
| **C++ 编译器** | VS Build Tools 2022 | Xcode CLT | `winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools"` / `xcode-select --install` |
| **WebView** | Win10 1803+ 自带 WebView2 | 系统自带 WKWebView | 无需装 |

## 开发与构建

### 安装依赖

```bash
git clone <repo-url>
cd screen-veil-tauri
npm install
```

### 开发模式（带热重载）

```bash
npm run tauri dev
```

### 编译 Release

```bash
# Windows (x64)
cargo tauri build --target x86_64-pc-windows-msvc --bundles msi,nsis

# macOS Apple Silicon
cargo tauri build --target aarch64-apple-darwin --bundles app,dmg

# macOS Intel
cargo tauri build --target x86_64-apple-darwin --bundles app,dmg
```

产物路径：
- Windows: `src-tauri/target/x86_64-pc-windows-msvc/release/screen-veil.exe` + `bundle/msi/*.msi` + `bundle/nsis/*.exe`
- macOS: `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ScreenVeil.app` + `bundle/dmg/*.dmg`

## 全局快捷键

| 平台 | 快捷键 | 备注 |
|------|--------|------|
| Windows | `Ctrl + Alt + P` | 默认 |
| macOS | `Cmd + Option + P` | 首次运行需在"系统设置 → 隐私与安全性 → 辅助功能"中授权 ScreenVeil |

按一次切换显示 / 隐藏遮罩。应用启动 400ms 后自动弹出遮罩（适合"有人突然看你的屏幕"场景）。

## macOS 首次运行

macOS 上首次运行 ScreenVeil.app 需要：

1. **右键打开**绕过 Gatekeeper：`右键 ScreenVeil.app → 打开`（未签名/未公证的提示）
2. **授权辅助功能**（全局快捷键必需）：
   ```
   系统设置 → 隐私与安全性 → 辅助功能 → 勾选 ScreenVeil
   ```
3. 屏幕录制说明会显示在权限请求弹窗中（仅声明用途，不实际录制）

## 配置文件

- `src-tauri/tauri.conf.json` — 应用元信息、bundle 配置、macOS 段
- `src-tauri/Info.plist` — macOS Info.plist 自定义项（LSUIElement、NSWindowTransparent 等）
- `src-tauri/entitlements.plist` — macOS 沙盒 + 全局快捷键 entitlements
- `src-tauri/src/lib.rs` — 全局快捷键注册与启动逻辑
- `src-tauri/src/veil.rs` — 跨平台遮罩核心 (Tauri WebviewWindow)
- `dist/veil.html` — 全屏遮罩 HTML (黑底白字)

## GitHub Actions 自动发布

推送 `v*` 标签即触发多平台构建并自动发布 Release：

```bash
git tag v1.0.0
git push origin v1.0.0
```

工作流文件：`.github/workflows/release.yml`

构建矩阵：
- `macos-latest` × `aarch64-apple-darwin` (Apple Silicon)
- `macos-latest` × `x86_64-apple-darwin` (Intel)
- `windows-latest` × `x86_64-pc-windows-msvc`

## 详细开发日志

参见 [`PROGRESS.md`](./PROGRESS.md) — 包含 pynput→Tauri 重写决策、cocoa/objc 兼容性、GitHub Contents API 旁路 push 等完整开发时间线。

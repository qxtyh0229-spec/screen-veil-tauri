# Screen Veil — Tauri 2 跨平台版

> 基于 **Tauri 2 + Rust** 的全屏快捷遮罩工具
> Windows + macOS 同一份代码，原生性能，单文件 4-6 MB

## 为什么用 Tauri？

| 维度 | Python+tkinter 版 | Tauri 2 版 |
|------|-------------------|------------|
| 体积 | 11 MB (含 Python 运行时) | **4-6 MB** (无运行时) |
| 性能 | tkinter 渲染较慢 | **WebView 硬件加速** |
| 跨平台 | Mac 需手动打包 | **同份代码编译 Mac/Win** |
| UI 灵活 | tkinter 简陋 | **HTML/CSS 替代 UI** |
| 安装 | 用户需装 Python | **双击即跑** |

## 当前状态
- ✅ Windows 端：源代码完成，3.07 MB exe 已构建 (Tauri release build)
- ⏳ macOS 端：框架完成，Cocoa 平台实现需在 Mac 上编译验证
- ✅ GitHub Actions：自动构建 Windows / macOS 安装包

## 环境要求

| 工具 | Windows | macOS | 安装方法 |
|------|---------|-------|----------|
| **Rust** | 必需 | 必需 | `winget install Rustlang.Rustup` / `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Node.js** | 必需 | 必需 | `winget install OpenJS.NodeJS.LTS` / `brew install node` |
| **C++ 编译器** | VS Build Tools 2022 | Xcode CLT | `winget install Microsoft.VisualStudio.2022.BuildTools --override "--add Microsoft.VisualStudio.Workload.VCTools"` / `xcode-select --install` |
| **WebView2** | Win10 1803+ 自带 | 系统自带 | 无需装 |

## 开发与构建

### 安装依赖

```powershell
git clone <repo-url>
cd screen-veil-tauri
npm install
```

### 开发模式（带热重载）

```powershell
npm run tauri dev
```

### 编译 Release

```powershell
# Windows
cargo tauri build --target x86_64-pc-windows-msvc

# macOS
cargo tauri build --target aarch64-apple-darwin    # Apple Silicon
cargo tauri build --target x86_64-apple-darwin     # Intel
```

产物路径：
- Windows: `src-tauri/target/x86_64-pc-windows-msvc/release/screen-veil.exe`
- macOS: `src-tauri/target/aarch64-apple-darwin/release/bundle/macos/ScreenVeil.app`

## 全局快捷键

| 平台 | 快捷键 |
|------|--------|
| Windows | `Ctrl + Alt + P` |
| macOS | `Cmd + Option + P` |

按一次切换显示 / 隐藏遮罩。

## 配置文件

遮罩文字 / 颜色 / 快捷键可在前端 UI（控制窗口）中动态修改，或在 `src-tauri/src/lib.rs` 的 `run()` 函数中修改默认值。

## GitHub Actions 自动发布

推送 `v*` 标签即触发多平台构建并自动发布 Release：

```bash
git tag v1.0.0
git push origin v1.0.0
```

工作流文件：`.github/workflows/release.yml`
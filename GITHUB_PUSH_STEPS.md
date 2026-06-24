# GitHub 推送步骤（30 秒完成）

## 第一步：在 GitHub 创建空仓库

1. 打开 https://github.com/new
2. **Repository name** 填 `screen-veil-tauri`（或你喜欢的名字）
3. **Description** 填 `Cross-platform global hotkey screen veil (Tauri 2 + Rust)`
4. **选 Public**（私有也行，但 Actions 免费额度私有 2000 分钟/月，公开无限）
5. **⚠️ 关键：所有勾选项都不要勾**（不要 Add README / .gitignore / license — 我们本地已有）
6. 点 **Create repository**

## 第二步：把仓库地址发给我

创建完后页面会显示一个 URL，类似：
```
https://github.com/你的用户名/screen-veil-tauri.git
```

把这段 URL 贴给我，我会执行：
```bash
git remote add origin <这个URL>
git push -u origin master
```

## 第三步：触发云端打包

### 方式 A：直接触发（不需 tag）
在 GitHub 仓库页面：
- 点 **Actions** tab
- 左侧选 **release** workflow
- 右侧 **Run workflow** → 选 `master` 分支 → 点 **Run workflow**
- 等 5-10 分钟（首次 cargo build 较慢），产物在 workflow 页面底部 **Artifacts** 下载

### 方式 B：tag 触发（自动发 Release）
```bash
git tag v0.1.0
git push origin v0.1.0
```
会自动构建 Windows + macOS 安装包并发布到 GitHub Releases 页面。

## 预期产物

| 平台 | 文件 | 大小 |
|------|------|------|
| Windows x64 | `ScreenVeil-Windows-x64.exe` | ~4 MB |
| macOS Apple Silicon (M1/M2/M3) | `ScreenVeil-macOS-arm64.app` | ~5 MB |
| macOS Intel | `ScreenVeil-macOS-x64.app` | ~5 MB |
| Windows MSI 安装包 | `ScreenVeil_0.1.0_x64_en-US.msi` | ~4 MB |
| macOS DMG | `ScreenVeil_0.1.0_aarch64.dmg` | ~4 MB |

## 第一次跑会慢

GitHub Actions 第一次需要：
- 下载 Rust 工具链 ~5 分钟
- 编译 350+ 个依赖 crate ~8 分钟
- 编译 screen-veil 本身 ~2 分钟
- 链接最终二进制 ~30 秒

总计 ~15 分钟首次，后续可走 cache 降到 5 分钟。
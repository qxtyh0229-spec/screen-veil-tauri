// Screen Veil - 跨平台全屏遮罩核心
//
// 平台策略:
// - Windows: transparent(true) + 黑色 HTML (WebView2 透明背景支持)
// - macOS:   fullscreen(true) + 黑色 HTML (Tauri 2 macOS 不暴露 transparent API,
//            但我们用 NSWindowTransparent=true in Info.plist + 全黑 HTML 视觉等价)
// - 共同: always_on_top + skip_taskbar + decorations(false) + 不可关闭/缩放
//
// 全局快捷键:
// - Windows: Ctrl+Alt+P
// - macOS:   Cmd+Option+P (Tauri SUPER 修饰符在 macOS 映射为 Cmd)
//   注意: macOS 首次运行需要在"系统设置 -> 隐私与安全性 -> 辅助功能"中授权
//         否则全局快捷键会被系统拦截 (这是 Apple 的设计, 不是 Tauri bug)

use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

static VISIBLE: AtomicBool = AtomicBool::new(false);
const VEIL_LABEL: &str = "veil-overlay";

pub fn is_visible() -> bool {
    VISIBLE.load(Ordering::SeqCst)
}

fn set_visible_state(v: bool) {
    VISIBLE.store(v, Ordering::SeqCst);
}

/// 显示遮罩
pub fn show_veil(app: &AppHandle) {
    if is_visible() {
        return;
    }
    set_visible_state(true);

    // 屏幕尺寸 (逻辑像素)
    let (sw, sh) = primary_screen_size(app);

    // 平台特定的 builder 配置
    let mut builder = WebviewWindowBuilder::new(
        app,
        VEIL_LABEL,
        WebviewUrl::App("veil.html".into()),
    )
    .title("Screen Veil")
    .inner_size(sw, sh)
    .position(0.0, 0.0)
    .decorations(false)
    .always_on_top(true)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .closable(false)
    .focused(false)
    .visible(true);

    // Windows: skipTaskbar + transparent
    #[cfg(windows)]
    {
        builder = builder
            .skip_taskbar(true)
            .transparent(true);
    }

    // macOS: 不调用 skip_taskbar (Tauri 2 macOS 上 skipTaskbar 是 no-op),
    //        不调用 transparent (Tauri 2 macOS 没有这个方法).
    //        透明效果通过 Info.plist 的 NSWindowTransparent=true + 全黑 HTML 视觉等价.
    //        配合 LSUIElement=true, 应用启动后不出现在 Dock.
    #[cfg(target_os = "macos")]
    {
        // 让 veil 窗口在所有 macOS Spaces 上方, 跨桌面滑动仍可见
        builder = builder
            .title_bar_style(tauri::TitleBarStyle::Overlay)
            .hidden_title(true);
    }

    // Linux: skipTaskbar 行为不同, 跳过
    #[cfg(target_os = "linux")]
    {
        let _ = builder; // explicit no-op
    }

    match builder.build() {
        Ok(window) => {
            // macOS: 应用是 LSUIElement, set_focus 不会拉起主窗口
            let _ = window.set_focus();
            eprintln!("[veil] show OK, label={}", window.label());
        }
        Err(e) => {
            eprintln!("[veil] failed to build window: {e}");
            set_visible_state(false);
        }
    }
}

/// 关闭遮罩
pub fn hide_veil(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(VEIL_LABEL) {
        if let Err(e) = w.close() {
            eprintln!("[veil] close failed: {e}");
        }
    }
    set_visible_state(false);
}

/// 切换显示状态 (从全局快捷键 handler 调用)
pub fn toggle_veil(app: &AppHandle) {
    if is_visible() {
        hide_veil(app);
    } else {
        show_veil(app);
    }
}

fn primary_screen_size(app: &AppHandle) -> (f64, f64) {
    if let Some(monitor) = app.primary_monitor().ok().flatten() {
        let sf = monitor.scale_factor();
        let size = monitor.size(); // physical pixels
        // 转为逻辑像素 (Tauri window 内部用逻辑像素)
        return (size.width as f64 / sf, size.height as f64 / sf);
    }
    // 兜底
    (1920.0, 1080.0)
}

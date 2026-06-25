// Screen Veil - Tauri 2 跨平台主程序
//
// 设计目标: 跨平台 + 极简 + 后台常驻
// - 启动时自动显示遮罩 (适合"有人突然看你的屏幕"场景)
// - 全局快捷键切换: Windows=Ctrl+Alt+P / macOS=Cmd+Option+P
// - 默认纯黑背景 + 居中白色警告文字, 不需要任何 UI 控件
// - macOS: Info.plist 设 LSUIElement=true, 不出现在 Dock

use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod veil;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    // Pressed 事件触发 toggle
                    if event.state() == ShortcutState::Pressed {
                        veil::toggle_veil(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            // 注册全局快捷键
            // Windows / Linux: Ctrl+Alt+P
            let ctrl_alt_p = Shortcut::new(
                Some(Modifiers::CONTROL | Modifiers::ALT),
                Code::KeyP,
            );
            if let Err(e) = app.global_shortcut().register(ctrl_alt_p) {
                eprintln!("[screen-veil] failed to register Ctrl+Alt+P: {e}");
            } else {
                eprintln!("[screen-veil] Ctrl+Alt+P registered");
            }

            // macOS: Cmd+Option+P (Tauri 在 macOS 把 SUPER 修饰符映射为 Cmd)
            #[cfg(target_os = "macos")]
            {
                let cmd_opt_p = Shortcut::new(
                    Some(Modifiers::SUPER | Modifiers::ALT),
                    Code::KeyP,
                );
                match app.global_shortcut().register(cmd_opt_p) {
                    Ok(_) => eprintln!("[screen-veil] Cmd+Option+P registered"),
                    Err(e) => {
                        // 常见原因: 用户没在"系统设置 -> 隐私与安全性 -> 辅助功能"授权
                        eprintln!(
                            "[screen-veil] failed to register Cmd+Option+P: {e}\n\
                             -> macOS 用户请打开: 系统设置 -> 隐私与安全性 -> 辅助功能 -> 勾选 ScreenVeil"
                        );
                    }
                }
            }

            // 启动后短暂延迟弹出遮罩 (等 webview 初始化完成)
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(400));
                veil::show_veil(&app_handle);
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

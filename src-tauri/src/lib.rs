// Screen Veil - Tauri 2 跨平台主程序
// 暴露 Tauri commands 给前端调用

use serde::Serialize;
use std::sync::Mutex;
use tauri::Emitter;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

mod veil;

// 全局配置状态
static TEXT: Mutex<String> = Mutex::new(String::new());
static BG_COLOR: Mutex<String> = Mutex::new(String::new());
static FG_COLOR: Mutex<String> = Mutex::new(String::new());

pub fn get_text() -> String {
    TEXT.lock().unwrap().clone()
}

pub fn get_bg_color() -> String {
    BG_COLOR.lock().unwrap().clone()
}

pub fn get_fg_color() -> String {
    FG_COLOR.lock().unwrap().clone()
}

#[derive(Serialize, Clone)]
struct VeilState {
    visible: bool,
    text: String,
    bg_color: String,
    fg_color: String,
}

#[tauri::command]
fn get_veil_state() -> VeilState {
    VeilState {
        visible: veil::is_visible(),
        text: get_text(),
        bg_color: get_bg_color(),
        fg_color: get_fg_color(),
    }
}

#[tauri::command]
fn toggle_veil(app: tauri::AppHandle) -> Result<(), String> {
    let was_visible = veil::is_visible();
    if was_visible {
        veil::hide_veil();
    } else {
        let app_clone = app.clone();
        std::thread::spawn(move || {
            veil::show_veil();
            // 遮罩窗口关闭后, 通知前端
            let _ = app_clone.emit("veil-state-changed", VeilState {
                visible: false,
                text: get_text(),
                bg_color: get_bg_color(),
                fg_color: get_fg_color(),
            });
        });
    }
    Ok(())
}

#[tauri::command]
fn set_veil_text(text: String) -> Result<(), String> {
    *TEXT.lock().unwrap() = text;
    Ok(())
}

#[tauri::command]
fn set_veil_bg(color: String) -> Result<(), String> {
    *BG_COLOR.lock().unwrap() = color;
    Ok(())
}

#[tauri::command]
fn set_veil_fg(color: String) -> Result<(), String> {
    *FG_COLOR.lock().unwrap() = color;
    Ok(())
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化默认值
    *TEXT.lock().unwrap() = "警告 警告 有人在看你的屏幕".to_string();
    *BG_COLOR.lock().unwrap() = "#000000".to_string();
    *FG_COLOR.lock().unwrap() = "#FFFFFF".to_string();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let _ = toggle_veil(app.clone());
                    }
                })
                .build(),
        )
        .setup(|app| {
            // 注册全局快捷键
            // Windows: Ctrl+Alt+P
            let ctrl_alt_p = Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyP);
            let _ = app.global_shortcut().register(ctrl_alt_p);

            // macOS: Cmd+Option+P (Tauri 在 macOS 上把 SUPER 映射为 Cmd)
            #[cfg(target_os = "macos")]
            {
                let cmd_opt_p = Shortcut::new(Some(Modifiers::SUPER | Modifiers::ALT), Code::KeyP);
                let _ = app.global_shortcut().register(cmd_opt_p);
            }

            // 启动后自动弹出遮罩（默认行为, 适合 screen-veil 用途）
            std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(300));
                veil::show_veil();
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_veil_state,
            toggle_veil,
            set_veil_text,
            set_veil_bg,
            set_veil_fg,
            quit_app,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

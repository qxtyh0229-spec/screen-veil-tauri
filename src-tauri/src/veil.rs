// Screen Veil - 跨平台全屏遮罩核心
// Windows: Win32 API + GDI (异步, 不阻塞主线程)
// macOS:   Cocoa NSWindow + NSTextField

use std::sync::atomic::{AtomicBool, Ordering};

static VISIBLE: AtomicBool = AtomicBool::new(false);

pub fn is_visible() -> bool {
    VISIBLE.load(Ordering::SeqCst)
}

pub fn set_visible_state(v: bool) {
    VISIBLE.store(v, Ordering::SeqCst);
}

// Windows implementation
#[cfg(windows)]
mod platform {
    use super::*;
    use std::sync::Mutex;
    use windows::Win32::Foundation::{HWND, LPARAM, WPARAM, RECT, LRESULT, COLORREF};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, RegisterClassW, WNDCLASSW, CS_HREDRAW, CS_VREDRAW,
        WS_EX_TOPMOST, WS_EX_TOOLWINDOW, WS_POPUP, WS_VISIBLE,
        SetWindowPos, HWND_TOPMOST, SWP_NOACTIVATE, SWP_SHOWWINDOW,
        SWP_NOMOVE, SWP_NOSIZE,
        GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN,
        DefWindowProcW, PostQuitMessage, LoadCursorW, IDC_ARROW,
        DestroyWindow, MSG, GetMessageW, TranslateMessage,
        DispatchMessageW, HMENU,
    };
    use windows::Win32::Graphics::Gdi::{
        BeginPaint, EndPaint, PAINTSTRUCT,
        CreateSolidBrush, SetBkColor, SetTextColor, FillRect, DeleteObject,
        CreateFontW, SelectObject, HGDIOBJ,
        TextOutW, GetTextMetricsW, TEXTMETRICW,
    };

    static WINDOW_HANDLE: Mutex<Option<isize>> = Mutex::new(None);

    unsafe extern "system" fn wnd_proc(
        hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            0x000F => {
                let mut ps = PAINTSTRUCT::default();
                let hdc = BeginPaint(hwnd, &mut ps);

                let brush = CreateSolidBrush(COLORREF(0x00000000));
                let _ = FillRect(hdc, &ps.rcPaint, brush);
                let _ = DeleteObject(HGDIOBJ(brush.0));

                let bg_color = COLORREF(0x00000000);
                let fg_color = COLORREF(0x00FFFFFF);
                SetBkColor(hdc, bg_color);
                SetTextColor(hdc, fg_color);

                let screen_w = GetSystemMetrics(SM_CXSCREEN);
                let screen_h = GetSystemMetrics(SM_CYSCREEN);
                let font_size = ((screen_h) / 8).max(48) as i32;

                let hfont = CreateFontW(
                    -font_size, 0, 0, 0, 700, 0, 0, 0,
                    1, 0, 0, 0, 0,
                    windows::core::w!("Microsoft YaHei UI"),
                );
                let _ = SelectObject(hdc, HGDIOBJ(hfont.0));

                // GetTextExtentPoint32W 拿准确文字像素宽度
                use windows::Win32::Foundation::SIZE;
                let text = crate::get_text();
                let mut text_utf16: Vec<u16> = text.encode_utf16().collect();
                let mut size = SIZE { cx: 0, cy: 0 };
                let _ = windows::Win32::Graphics::Gdi::GetTextExtentPoint32W(hdc, &mut text_utf16, &mut size);

                let text_w = size.cx;
                let text_h = size.cy;
                let text_x = (screen_w - text_w) / 2;
                let text_y = (screen_h - text_h) / 2;

                let _ = TextOutW(hdc, text_x, text_y, &mut text_utf16);

                let _ = DeleteObject(HGDIOBJ(hfont.0));
                let _ = EndPaint(hwnd, &ps);
                LRESULT(0)
            }
            0x0010 | 0x0100 => {
                let close = if msg == 0x0010 {
                    true
                } else {
                    wparam.0 == 0x1B
                };
                if close {
                    set_visible_state(false);
                    PostQuitMessage(0);
                    return LRESULT(0);
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    pub fn show() {
        if is_visible() { return; }

        std::thread::spawn(|| {
            unsafe {
                let class_name = windows::core::w!("ScreenVeilClass");
                let hinstance = windows::Win32::System::LibraryLoader::GetModuleHandleW(None).unwrap();

                let wc = WNDCLASSW {
                    style: CS_HREDRAW | CS_VREDRAW,
                    lpfnWndProc: Some(wnd_proc),
                    hInstance: hinstance.into(),
                    lpszClassName: class_name,
                    hCursor: LoadCursorW(None, IDC_ARROW).unwrap(),
                    ..Default::default()
                };
                let _ = RegisterClassW(&wc);

                let screen_w = GetSystemMetrics(SM_CXSCREEN);
                let screen_h = GetSystemMetrics(SM_CYSCREEN);

                let hwnd = CreateWindowExW(
                    WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                    class_name,
                    windows::core::w!("Screen Veil"),
                    WS_POPUP | WS_VISIBLE,
                    0, 0, screen_w, screen_h,
                    HWND(std::ptr::null_mut()),
                    HMENU(std::ptr::null_mut()),
                    hinstance,
                    None,
                ).expect("CreateWindowExW failed");

                *WINDOW_HANDLE.lock().unwrap() = Some(hwnd.0 as isize);

                let _ = SetWindowPos(
                    hwnd, HWND_TOPMOST, 0, 0, 0, 0,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_NOMOVE | SWP_NOSIZE,
                );

                set_visible_state(true);

                let mut msg = MSG::default();
                while GetMessageW(&mut msg, HWND(std::ptr::null_mut()), 0, 0).as_bool() {
                    let _ = TranslateMessage(&msg);
                    let _ = DispatchMessageW(&msg);
                }

                *WINDOW_HANDLE.lock().unwrap() = None;
                set_visible_state(false);
            }
        });
    }

    pub fn hide() {
        if let Some(raw) = *WINDOW_HANDLE.lock().unwrap() {
            unsafe {
                let _ = DestroyWindow(HWND(raw as *mut _));
            }
        }
        set_visible_state(false);
    }
}

// macOS implementation
#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use cocoa::appkit::{
        NSApp, NSBackingStoreBuffered, NSWindow, NSWindowStyleMask,
    };
    use cocoa::base::{id, nil, YES, NO};
    use cocoa::foundation::{NSPoint, NSRect, NSString};
    use objc::msg_send;
    use std::sync::Mutex;

    static WINDOW_REF: Mutex<Option<id>> = Mutex::new(None);

    pub fn show() {
        if is_visible() { return; }

        unsafe {
            let app = NSApp();
            app.setActivationPolicy_(
                cocoa::appkit::NSApplicationActivationPolicy::NSApplicationActivationPolicyAccessory,
            );

            let screen: id = msg_send!(cocoa::base::class!(NSScreen), mainScreen);
            let frame: NSRect = msg_send!(screen, frame);

            let style = NSWindowStyleMask::NSBorderlessWindowMask
                | NSWindowStyleMask::NSFullSizeContentViewWindowMask;

            let window: id = NSWindow::alloc(nil).initWithContentRect_styleMask_backing_defer_(
                frame,
                style,
                NSBackingStoreBuffered,
                NO,
            );

            let _: () = msg_send![window, setBackgroundColor: cocoa::appkit::NSColor::blackColor(nil)];
            let _: () = msg_send![window, setLevel: cocoa::base::NSIntegerMax];
            let _: () = msg_send![window, setOpaque: YES];
            let _: () = msg_send![window, setHidesOnDeactivate: NO];
            let _: () = msg_send![window, setCollectionBehavior:
                cocoa::appkit::NSWindowCollectionBehavior::NSWindowCollectionBehaviorCanJoinAllSpaces
                | cocoa::appkit::NSWindowCollectionBehavior::NSWindowCollectionBehaviorFullScreenAuxiliary
            ];
            window.makeKeyAndOrderFront_(nil);
            let _: () = msg_send![window, orderFrontRegardless];

            let text = NSString::alloc(nil).init_str(&crate::get_text());
            let font_size: f64 = frame.size.height / 15.0;
            let font: id = msg_send![
                cocoa::appkit::NSFont::class(),
                boldSystemFontOfSize: font_size
            ];

            let label: id = msg_send![cocoa::appkit::NSTextField::class(), alloc];
            let label: id = msg_send![label,
                initWithFrame: NSRect::new(NSPoint::new(0.0, 0.0), frame.size)
            ];
            let _: () = msg_send![label, setStringValue: text];
            let _: () = msg_send![label, setFont: font];
            let _: () = msg_send![label, setTextColor: cocoa::appkit::NSColor::whiteColor(nil)];
            let _: () = msg_send![label, setBezeled: NO];
            let _: () = msg_send![label, setDrawsBackground: NO];
            let _: () = msg_send![label, setEditable: NO];
            let _: () = msg_send![label, setSelectable: NO];
            let _: () = msg_send![label, setAlignment: 2];
            let _: () = msg_send![label, setAutoresizingMask: 18];

            let content: id = msg_send![window, contentView];
            let _: () = msg_send![content, addSubview: label];

            let _ = app.activateIgnoringOtherApps_(YES);

            *WINDOW_REF.lock().unwrap() = Some(window);
            set_visible_state(true);
        }
    }

    pub fn hide() {
        if let Some(window) = *WINDOW_REF.lock().unwrap() {
            unsafe {
                let _: () = msg_send![window, close];
            }
        }
        *WINDOW_REF.lock().unwrap() = None;
        set_visible_state(false);
    }
}

pub fn show_veil() {
    // 不设 DPI aware - 用系统默认 (logical pixels), CreateWindow 0,0,screen_w,screen_h
    // 会让 OS 自动按 high-DPI scale, 居中逻辑直接生效
    platform::show();
}

pub fn hide_veil() {
    platform::hide();
}
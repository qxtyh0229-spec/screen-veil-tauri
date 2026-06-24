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
// 纯 objc 0.2 msg_send, 不依赖 cocoa crate
#[cfg(target_os = "macos")]
mod platform {
    use super::*;
    use objc::msg_send;
    use objc::runtime::Object;
    use std::ffi::CString;
    use std::sync::Mutex;

    type Id = *mut Object;

    static WINDOW_REF: Mutex<Option<Id>> = Mutex::new(None);

    fn nil_id() -> Id { std::ptr::null_mut() }

    // [[NSString alloc] initWithUTF8String:cstr]
    unsafe fn nsstring(s: &str) -> Id {
        let cstr = CString::new(s).unwrap();
        let cls = objc::class!(NSString);
        let obj: Id = msg_send![cls, alloc];
        msg_send![obj, initWithUTF8String: cstr.as_ptr()]
    }

    // CGRect / CGPoint / CGSize — 与 NSRect 等价的 C struct (64-bit Apple)
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGPoint { x: f64, y: f64 }
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGSize { width: f64, height: f64 }
    #[repr(C)]
    #[derive(Copy, Clone)]
    struct CGRect { origin: CGPoint, size: CGSize }

    pub fn show() {
        if is_visible() { return; }

        unsafe {
            // [NSApplication sharedApplication]
            let app: Id = msg_send![objc::class!(NSApplication), sharedApplication];

            // setActivationPolicy: NSApplicationActivationPolicyAccessory = 1
            let _: () = msg_send![app, setActivationPolicy: 1i64];

            // [NSScreen mainScreen]
            let screen: Id = msg_send![objc::class!(NSScreen), mainScreen];
            // -frame returns CGRect (value type)
            let frame: CGRect = msg_send![screen, frame];

            // [[NSWindow alloc] initWithContentRect:styleMask:backing:defer:]
            // styleMask: NSBorderlessWindowMask = 0
            // backing:  NSBackingStoreBuffered  = 2
            let window: Id = msg_send![objc::class!(NSWindow), alloc];
            let window: Id = msg_send![window,
                initWithContentRect: frame
                styleMask: 0u64
                backing: 2u64
                defer: false
            ];

            // [NSColor blackColor]
            let black: Id = msg_send![objc::class!(NSColor), blackColor];
            let _: () = msg_send![window, setBackgroundColor: black];

            // setLevel: NSIntegerMax
            let _: () = msg_send![window, setLevel: i64::MAX];
            // setOpaque: YES
            let _: () = msg_send![window, setOpaque: true];
            // setHidesOnDeactivate: NO
            let _: () = msg_send![window, setHidesOnDeactivate: false];
            // setCollectionBehavior: CanJoinAllSpaces(1) | FullScreenAuxiliary(256)
            let _: () = msg_send![window, setCollectionBehavior: 1i64 | 256i64];
            // makeKeyAndOrderFront:nil
            let _: () = msg_send![window, makeKeyAndOrderFront: nil_id()];
            // orderFrontRegardless
            let _: () = msg_send![window, orderFrontRegardless];

            // Label text
            let text = nsstring(&crate::get_text());
            let font_size = frame.size.height / 15.0;
            // [NSFont boldSystemFontOfSize:fontSize]
            let font: Id = msg_send![objc::class!(NSFont), boldSystemFontOfSize: font_size];

            // [[NSTextField alloc] initWithFrame:zero_rect]
            let label: Id = msg_send![objc::class!(NSTextField), alloc];
            let label_rect = CGRect { origin: CGPoint { x: 0.0, y: 0.0 }, size: frame.size };
            let label: Id = msg_send![label, initWithFrame: label_rect];

            let _: () = msg_send![label, setStringValue: text];
            let _: () = msg_send![label, setFont: font];
            let white: Id = msg_send![objc::class!(NSColor), whiteColor];
            let _: () = msg_send![label, setTextColor: white];
            let _: () = msg_send![label, setBezeled: false];
            let _: () = msg_send![label, setDrawsBackground: false];
            let _: () = msg_send![label, setEditable: false];
            let _: () = msg_send![label, setSelectable: false];
            // NSTextAlignmentCenter = 2
            let _: () = msg_send![label, setAlignment: 2i64];
            // NSViewWidthSizable(2) | NSViewHeightSizable(16) = 18
            let _: () = msg_send![label, setAutoresizingMask: 18i64];

            let content: Id = msg_send![window, contentView];
            let _: () = msg_send![content, addSubview: label];

            // [NSApp activateIgnoringOtherApps:YES]
            let _: () = msg_send![app, activateIgnoringOtherApps: true];

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
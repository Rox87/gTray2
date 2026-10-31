//! tray.rs - Integração com a bandeja do sistema (Windows System Tray) e ícone personalizado do gTray.

use std::mem::zeroed;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    CreateBitmap, CreateDIBSection, DeleteObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    DIB_RGB_COLORS,
};
use windows_sys::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreateIconIndirect, CreatePopupMenu, CreateWindowExW, DefWindowProcW,
    DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW, EnumWindows, FindWindowW,
    GetCursorPos, GetWindowTextW, GetWindowThreadProcessId, LoadIconW, PostQuitMessage,
    RegisterClassW, SetForegroundWindow, ShowWindow, TrackPopupMenu, TranslateMessage, HICON,
    HMENU, ICONINFO, IDI_APPLICATION, MF_STRING, MSG, SW_HIDE, SW_RESTORE, SW_SHOW,
    TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_RIGHTBUTTON, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK,
    WM_LBUTTONUP, WM_RBUTTONUP, WM_USER, WNDCLASSW,
};

pub const WM_TRAYICON: u32 = WM_USER + 100;
pub const ID_TRAY_SHOW: usize = 1001;
pub const ID_TRAY_EXIT: usize = 1002;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    ShowUI,
    Exit,
}

pub struct TrayIcon {
    cmd_sender: Sender<()>,
    thread_handle: Option<JoinHandle<()>>,
}

impl TrayIcon {
    pub fn new<F>(on_event: F) -> Self
    where
        F: Fn(TrayEvent) + Send + Sync + 'static,
    {
        let (cmd_sender, cmd_receiver): (Sender<()>, Receiver<()>) = channel();
        let on_event = Arc::new(on_event);

        let thread_handle = thread::spawn(move || unsafe {
            let class_name: Vec<u16> = "GTrayHiddenClass\0".encode_utf16().collect();
            let mut wc: WNDCLASSW = zeroed();
            wc.lpfnWndProc = Some(tray_wnd_proc);
            wc.lpszClassName = class_name.as_ptr();

            RegisterClassW(&wc);

            let hwnd = CreateWindowExW(
                0,
                class_name.as_ptr(),
                class_name.as_ptr(),
                0,
                0,
                0,
                0,
                0,
                0 as HWND,
                0 as HMENU,
                0 as _,
                std::ptr::null(),
            );

            // Armazena callback no contexto global da thread
            TRAY_CALLBACK.with(|cell| {
                *cell.borrow_mut() = Some(on_event.clone());
            });

            // Carrega ícone oficial do gTray (assets/favicon.png)
            let (hicon, is_custom) = load_custom_icon();

            // Configura o NotifyIcon
            let mut nid: NOTIFYICONDATAW = zeroed();
            nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = hwnd;
            nid.uID = 1;
            nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
            nid.uCallbackMessage = WM_TRAYICON;
            nid.hIcon = hicon;

            let tip = "gTray - Assistente IA\0";
            for (i, c) in tip.encode_utf16().enumerate() {
                if i < nid.szTip.len() {
                    nid.szTip[i] = c;
                }
            }

            Shell_NotifyIconW(NIM_ADD, &nid);

            loop {
                if cmd_receiver.try_recv().is_ok() {
                    Shell_NotifyIconW(NIM_DELETE, &nid);
                    if is_custom && hicon != 0 as _ {
                        DestroyIcon(hicon);
                    }
                    DestroyWindow(hwnd);
                    PostQuitMessage(0);
                    break;
                }

                use windows_sys::Win32::UI::WindowsAndMessaging::{PeekMessageW, PM_REMOVE};
                let mut msg: MSG = zeroed();
                if PeekMessageW(&mut msg, hwnd, 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                } else {
                    thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        });

        Self {
            cmd_sender,
            thread_handle: Some(thread_handle),
        }
    }
}

impl Drop for TrayIcon {
    fn drop(&mut self) {
        let _ = self.cmd_sender.send(());
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

unsafe extern "system" fn enum_find_window_cb(hwnd: HWND, lparam: LPARAM) -> i32 {
    let target = &mut *(lparam as *mut (u32, HWND));
    let mut pid: u32 = 0;
    GetWindowThreadProcessId(hwnd, &mut pid);
    if pid == target.0 {
        let mut buf = [0u16; 256];
        let len = GetWindowTextW(hwnd, buf.as_mut_ptr(), 256);
        if len > 0 {
            let title = String::from_utf16_lossy(&buf[..len as usize]);
            if title.contains("gTray") {
                target.1 = hwnd;
                return 0; // para a enumeração
            }
        }
    }
    1 // continua
}

/// Localiza a janela principal do gTray
pub fn find_app_window() -> HWND {
    unsafe {
        let title1: Vec<u16> = "gTray - Assistente IA\0".encode_utf16().collect();
        let mut hwnd = FindWindowW(std::ptr::null(), title1.as_ptr());
        if hwnd == 0 as HWND {
            let title2: Vec<u16> = "gTray\0".encode_utf16().collect();
            hwnd = FindWindowW(std::ptr::null(), title2.as_ptr());
        }
        if hwnd == 0 as HWND {
            let mut target = (std::process::id(), 0 as HWND);
            EnumWindows(Some(enum_find_window_cb), &mut target as *mut _ as LPARAM);
            hwnd = target.1;
        }
        hwnd
    }
}

/// Oculta a janela principal do gTray da tela e da barra de tarefas
pub fn hide_app_window() -> bool {
    let hwnd = find_app_window();
    if hwnd != 0 as HWND {
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
        true
    } else {
        false
    }
}

/// Restaura e traz para frente a janela principal do gTray
pub fn restore_app_window() -> bool {
    let hwnd = find_app_window();
    if hwnd != 0 as HWND {
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
        true
    } else {
        false
    }
}

/// Carrega o ícone `assets/favicon.png` e converte para HICON 32-bit com canal alfa nativo
unsafe fn load_custom_icon() -> (HICON, bool) {
    if let Ok(bytes) = std::fs::read("assets/favicon.png") {
        if let Ok(img) = image::load_from_memory(&bytes) {
            let w = 32i32;
            let h = 32i32;
            let resized = img.resize_exact(w as u32, h as u32, image::imageops::FilterType::Lanczos3).to_rgba8();

            let mut bmi: BITMAPINFO = zeroed();
            bmi.bmiHeader.biSize = std::mem::size_of::<BITMAPINFOHEADER>() as u32;
            bmi.bmiHeader.biWidth = w;
            bmi.bmiHeader.biHeight = -h; // Top-down DIB
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB;

            let mut bits: *mut std::ffi::c_void = std::ptr::null_mut();
            let hbm_color = CreateDIBSection(
                0 as _,
                &bmi,
                DIB_RGB_COLORS,
                &mut bits,
                0 as _,
                0,
            );

            if hbm_color != 0 as _ && !bits.is_null() {
                let raw_rgba = resized.as_raw();
                let mut bgra = Vec::with_capacity((w * h * 4) as usize);
                for chunk in raw_rgba.chunks_exact(4) {
                    let r = chunk[0];
                    let g = chunk[1];
                    let b = chunk[2];
                    let a = chunk[3];
                    // BGRA com premultiplicação de alfa para o compositor do Windows Shell
                    let alpha_factor = a as f32 / 255.0;
                    let p_r = (r as f32 * alpha_factor).round() as u8;
                    let p_g = (g as f32 * alpha_factor).round() as u8;
                    let p_b = (b as f32 * alpha_factor).round() as u8;

                    bgra.push(p_b);
                    bgra.push(p_g);
                    bgra.push(p_r);
                    bgra.push(a);
                }
                std::ptr::copy_nonoverlapping(bgra.as_ptr(), bits as *mut u8, bgra.len());

                let mask_bytes = vec![0u8; (w * 4 * h) as usize];
                let hbm_mask = CreateBitmap(w, h, 1, 1, mask_bytes.as_ptr() as _);

                let mut icon_info: ICONINFO = zeroed();
                icon_info.fIcon = 1; // 1 = Ícone
                icon_info.xHotspot = 0;
                icon_info.yHotspot = 0;
                icon_info.hbmMask = hbm_mask;
                icon_info.hbmColor = hbm_color;

                let hicon = CreateIconIndirect(&icon_info);

                DeleteObject(hbm_color as _);
                if hbm_mask != 0 as _ {
                    DeleteObject(hbm_mask as _);
                }

                if hicon != 0 as _ {
                    return (hicon, true);
                }
            }
        }
    }

    (LoadIconW(0 as _, IDI_APPLICATION), false)
}

thread_local! {
    static TRAY_CALLBACK: std::cell::RefCell<Option<Arc<dyn Fn(TrayEvent) + Send + Sync>>> = std::cell::RefCell::new(None);
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_TRAYICON => {
            let event = lparam as u32;
            if event == WM_LBUTTONUP || event == WM_LBUTTONDBLCLK {
                restore_app_window();
                TRAY_CALLBACK.with(|cell| {
                    if let Some(ref cb) = *cell.borrow() {
                        cb(TrayEvent::ShowUI);
                    }
                });
            } else if event == WM_RBUTTONUP {
                let hmenu = CreatePopupMenu();
                let show_label: Vec<u16> = "Mostrar UI\0".encode_utf16().collect();
                let exit_label: Vec<u16> = "Sair\0".encode_utf16().collect();

                AppendMenuW(hmenu, MF_STRING, ID_TRAY_SHOW, show_label.as_ptr());
                AppendMenuW(hmenu, MF_STRING, ID_TRAY_EXIT, exit_label.as_ptr());

                let mut pt: POINT = zeroed();
                GetCursorPos(&mut pt);
                SetForegroundWindow(hwnd);

                TrackPopupMenu(
                    hmenu,
                    TPM_RIGHTBUTTON | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
                    pt.x,
                    pt.y,
                    0,
                    hwnd,
                    std::ptr::null(),
                );
                DestroyMenu(hmenu);
            }
            0
        }
        WM_COMMAND => {
            let id = (wparam & 0xFFFF) as usize;
            if id == ID_TRAY_SHOW {
                restore_app_window();
                TRAY_CALLBACK.with(|cell| {
                    if let Some(ref cb) = *cell.borrow() {
                        cb(TrayEvent::ShowUI);
                    }
                });
            } else if id == ID_TRAY_EXIT {
                TRAY_CALLBACK.with(|cell| {
                    if let Some(ref cb) = *cell.borrow() {
                        cb(TrayEvent::Exit);
                    }
                });
            }
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

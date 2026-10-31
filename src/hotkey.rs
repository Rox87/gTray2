//! hotkey.rs - Gerenciamento nativo de atalhos de teclado globais no Windows.

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    RegisterHotKey, UnregisterHotKey, MOD_ALT, MOD_CONTROL, MOD_NOREPEAT, MOD_SHIFT, MOD_WIN,
    VK_ESCAPE, VK_F1, VK_RETURN, VK_SPACE, VK_TAB,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, PostQuitMessage, TranslateMessage, MSG, WM_HOTKEY,
};

use crate::escriba::Escriba;

pub fn parse_hotkey_string(key_str: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = key_str.split('+').map(|s| s.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let mut modifiers = MOD_NOREPEAT as u32;
    let mut vk: Option<u32> = None;

    for part in parts {
        let p_lower = part.to_lowercase();
        match p_lower.as_str() {
            "ctrl" | "control" => modifiers |= MOD_CONTROL as u32,
            "alt" => modifiers |= MOD_ALT as u32,
            "shift" => modifiers |= MOD_SHIFT as u32,
            "win" | "windows" => modifiers |= MOD_WIN as u32,
            _ => {
                if p_lower.starts_with('f') {
                    if let Ok(num) = p_lower[1..].parse::<u32>() {
                        if (1..=24).contains(&num) {
                            vk = Some(VK_F1 as u32 + (num - 1));
                        }
                    }
                } else if p_lower.len() == 1 {
                    let ch = p_lower.chars().next().unwrap();
                    if ch.is_ascii_alphabetic() {
                        vk = Some(ch.to_ascii_uppercase() as u32);
                    } else if ch.is_ascii_digit() {
                        vk = Some(ch as u32);
                    }
                } else {
                    match p_lower.as_str() {
                        "esc" | "escape" => vk = Some(VK_ESCAPE as u32),
                        "space" => vk = Some(VK_SPACE as u32),
                        "tab" => vk = Some(VK_TAB as u32),
                        "return" | "enter" => vk = Some(VK_RETURN as u32),
                        _ => {}
                    }
                }
            }
        }
    }

    vk.map(|v| (modifiers, v))
}

pub enum HotkeyCommand {
    Register {
        id: i32,
        name: String,
        modifiers: u32,
        vk: u32,
    },
    UnregisterAll,
    Stop,
}

pub struct HotkeyManager {
    cmd_sender: Sender<HotkeyCommand>,
    thread_handle: Option<JoinHandle<()>>,
}

impl HotkeyManager {
    pub fn new<F>(logger: Escriba, on_hotkey: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        let (cmd_sender, cmd_receiver): (Sender<HotkeyCommand>, Receiver<HotkeyCommand>) = channel();
        let on_hotkey = Arc::new(on_hotkey);

        let thread_handle = thread::spawn(move || {
            let mut registered_ids: HashMap<i32, String> = HashMap::new();

            loop {
                // Process incoming commands non-blocking
                while let Ok(cmd) = cmd_receiver.try_recv() {
                    match cmd {
                        HotkeyCommand::Register {
                            id,
                            name,
                            modifiers,
                            vk,
                        } => unsafe {
                            let mut success = false;
                            for _attempt in 1..=5 {
                                if RegisterHotKey(0 as HWND, id, modifiers, vk) != 0 {
                                    registered_ids.insert(id, name.clone());
                                    logger.info(&format!("Atalho '{}' ativado no sistema com sucesso.", name));
                                    success = true;
                                    break;
                                }
                                thread::sleep(std::time::Duration::from_millis(150));
                            }
                            if !success {
                                let err = windows_sys::Win32::Foundation::GetLastError();
                                logger.error(&format!(
                                    "Falha ao registrar tecla do atalho '{}' no Windows (código {}). Verifique se a combinação já está em uso.",
                                    name, err
                                ));
                            }
                        },
                        HotkeyCommand::UnregisterAll => unsafe {
                            for &id in registered_ids.keys() {
                                UnregisterHotKey(0 as HWND, id);
                            }
                            registered_ids.clear();
                        },
                        HotkeyCommand::Stop => {
                            unsafe {
                                for &id in registered_ids.keys() {
                                    UnregisterHotKey(0 as HWND, id);
                                }
                                PostQuitMessage(0);
                            }
                            return;
                        }
                    }
                }

                // Process Windows Message
                unsafe {
                    let mut msg: MSG = std::mem::zeroed();
                    use windows_sys::Win32::UI::WindowsAndMessaging::{PeekMessageW, PM_REMOVE};
                    while PeekMessageW(&mut msg, 0 as HWND, 0, 0, PM_REMOVE) != 0 {
                        if msg.message == WM_HOTKEY {
                            let id = msg.wParam as i32;
                            if let Some(name) = registered_ids.get(&id) {
                                let callback = on_hotkey.clone();
                                let shortcut_name = name.clone();
                                thread::spawn(move || {
                                    callback(shortcut_name);
                                });
                            }
                        }
                        TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                    thread::sleep(std::time::Duration::from_millis(15));
                }
            }
        });

        Self {
            cmd_sender,
            thread_handle: Some(thread_handle),
        }
    }

    pub fn register(&self, id: i32, name: &str, modifiers: u32, vk: u32) {
        let _ = self.cmd_sender.send(HotkeyCommand::Register {
            id,
            name: name.to_string(),
            modifiers,
            vk,
        });
    }

    pub fn unregister_all(&self) {
        let _ = self.cmd_sender.send(HotkeyCommand::UnregisterAll);
    }
}

impl Drop for HotkeyManager {
    fn drop(&mut self) {
        let _ = self.cmd_sender.send(HotkeyCommand::Stop);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

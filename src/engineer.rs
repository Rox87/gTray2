//! engineer.rs - Módulo de automação de teclado e área de transferência (Windows).

use arboard::Clipboard;
use std::mem::size_of;
use std::thread::sleep;
use std::time::Duration;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL, VK_LEFT,
    VK_SHIFT,
};

use crate::escriba::Escriba;

pub struct Engineer {
    logger: Escriba,
}

impl Engineer {
    pub fn new(logger: Escriba) -> Self {
        Self { logger }
    }

    fn make_key_input(vk: u16, up: bool) -> INPUT {
        let flags = if up { KEYEVENTF_KEYUP } else { 0 };
        INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: flags,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        }
    }

    fn send_ctrl_combo(vk: u16) {
        unsafe {
            let mut inputs = [
                Self::make_key_input(VK_CONTROL, false),
                Self::make_key_input(vk, false),
                Self::make_key_input(vk, true),
                Self::make_key_input(VK_CONTROL, true),
            ];
            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                size_of::<INPUT>() as i32,
            );
        }
    }

    fn send_ctrl_shift_left() {
        unsafe {
            let mut inputs = [
                Self::make_key_input(VK_CONTROL, false),
                Self::make_key_input(VK_SHIFT, false),
                Self::make_key_input(VK_LEFT, false),
                Self::make_key_input(VK_LEFT, true),
                Self::make_key_input(VK_SHIFT, true),
                Self::make_key_input(VK_CONTROL, true),
            ];
            SendInput(
                inputs.len() as u32,
                inputs.as_mut_ptr(),
                size_of::<INPUT>() as i32,
            );
        }
    }

    pub fn pre_processamento(&self, retry_clip: u32) -> Option<String> {
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(e) => {
                self.logger.error(&format!("Falha ao acessar Clipboard: {}", e));
                return None;
            }
        };

        // Limpa temporariamente ou verifica antes
        let _ = clipboard.clear();

        let mut dados_clipboard = String::new();
        let mut retries = retry_clip.max(10);

        while dados_clipboard.is_empty() && retries > 0 {
            retries -= 1;
            Self::send_ctrl_combo(b'C' as u16);
            sleep(Duration::from_millis(100));

            if let Ok(text) = clipboard.get_text() {
                dados_clipboard = text;
            }
            sleep(Duration::from_millis(50));
        }

        if dados_clipboard.trim().len() <= 1 {
            self.logger.info("ignorado");
            Self::send_ctrl_combo(b'Z' as u16);
            return None;
        }

        let input_text = dados_clipboard.clone();

        // Injeta "Processando" como feedback visual transitório
        let _ = clipboard.set_text("Processando");
        sleep(Duration::from_millis(100));

        Self::send_ctrl_combo(b'V' as u16);
        sleep(Duration::from_millis(200));

        // Seleciona a palavra "Processando"
        Self::send_ctrl_shift_left();
        sleep(Duration::from_millis(100));

        self.logger.info(&format!("Prompt: {}", input_text));
        Some(input_text)
    }

    pub fn pos_processamento(&self, response: &str) {
        if let Ok(mut clipboard) = Clipboard::new() {
            let _ = clipboard.set_text(response);
            sleep(Duration::from_millis(100));
            Self::send_ctrl_combo(b'V' as u16);
            self.logger.info("resposta entregue");
        } else {
            self.logger.error("Falha ao abrir clipboard para pós-processamento");
        }
    }
}

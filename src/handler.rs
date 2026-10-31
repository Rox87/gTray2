//! handler.rs - Controlador central de eventos, atalhos globais e integração com IA.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::data_manager::GTrayDataManager;
use crate::engineer::Engineer;
use crate::escriba::Escriba;
use crate::hotkey::{parse_hotkey_string, HotkeyManager};
use crate::ia::IA;

#[derive(Clone)]
pub struct GtrayHandler {
    pub data_manager: Arc<Mutex<GTrayDataManager>>,
    pub engineer: Arc<Engineer>,
    pub ia: Arc<IA>,
    pub hotkey_manager: Arc<Mutex<Option<HotkeyManager>>>,
    pub logger: Escriba,
}

impl GtrayHandler {
    pub fn new(data_manager: Arc<Mutex<GTrayDataManager>>, logger: Escriba) -> Self {
        let dm = data_manager.lock().unwrap();
        let first_key = dm
            .get_providers()
            .values()
            .next()
            .map(|p| p.api_key.clone())
            .unwrap_or_default();
        let first_base_url = dm
            .get_providers()
            .keys()
            .next()
            .and_then(|name| dm.get_provider_base_url(name));
        drop(dm);

        let engineer = Arc::new(Engineer::new(logger.clone()));
        let ia = Arc::new(IA::new(&first_key, first_base_url.as_deref()));

        Self {
            data_manager,
            engineer,
            ia,
            hotkey_manager: Arc::new(Mutex::new(None)),
            logger,
        }
    }

    pub fn start(&self) {
        self.register_shortcuts();
    }

    pub fn stop(&self) {
        self.unregister_shortcuts();
    }

    pub fn unregister_shortcuts(&self) {
        let mut hkm_guard = self.hotkey_manager.lock().unwrap();
        if let Some(ref hkm) = *hkm_guard {
            hkm.unregister_all();
        }
        *hkm_guard = None;
    }

    pub fn register_shortcuts(&self) {
        self.unregister_shortcuts();

        let handler_clone = self.clone();
        let new_hkm = HotkeyManager::new(self.logger.clone(), move |shortcut_name: String| {
            handler_clone.handle_ai_request(&shortcut_name, "");
        });

        let dm = self.data_manager.lock().unwrap();
        let shortcuts = dm.get_shortcuts().clone();
        drop(dm);

        let mut id = 100;
        let mut active_key = None;

        for (name, sc) in &shortcuts {
            if sc.enabled && !sc.key.trim().is_empty() && sc.key.trim() != "(nenhuma)" {
                if let Some((mods, vk)) = parse_hotkey_string(&sc.key) {
                    new_hkm.register(id, name, mods, vk);
                    id += 1;
                    if name == "upgrade" {
                        active_key = Some(sc.key.clone());
                    }
                } else {
                    self.logger.error(&format!(
                        "Erro ao analisar combinação do atalho '{}' ({})",
                        name, sc.key
                    ));
                }
            }
        }

        self.logger.log_shortcuts(&shortcuts, active_key.as_deref());
        let mut hkm_guard = self.hotkey_manager.lock().unwrap();
        *hkm_guard = Some(new_hkm);
    }

    pub fn handle_ai_request(&self, mode: &str, prompt_modifier: &str) {
        let start_time = Instant::now();
        self.logger.info(&format!("Processando atalho '{}'...", mode));

        let retry_clip = {
            let dm = self.data_manager.lock().unwrap();
            dm.data.app.retry_clip
        };

        let clipboard = match self.engineer.pre_processamento(retry_clip) {
            Some(text) => text,
            None => {
                self.logger.info("Nenhum texto selecionado na área de transferência.");
                return;
            }
        };

        self.logger.info("Consultando provedor de Inteligência Artificial...");
        let resolved = {
            let dm = self.data_manager.lock().unwrap();
            match dm.resolve_shortcut(mode) {
                Some(r) => r,
                None => {
                    self.logger.error(&format!("Atalho '{}' não encontrado para resolução.", mode));
                    return;
                }
            }
        };

        let api_key = resolved.api_key;
        let model_name = resolved.model_name;
        let base_url = resolved.base_url;
        let system_prompt = resolved.system_prompt.trim();

        let system_cmd = if !system_prompt.is_empty() {
            system_prompt
        } else {
            prompt_modifier
        };

        let comment_char = resolved.comment_char;
        let extract_blocks = resolved.extract_blocks;

        let retries = {
            let dm = self.data_manager.lock().unwrap();
            dm.data.app.retry_ia
        };

        let response_res = if extract_blocks {
            if mode == "python" {
                self.ia.python_responder(
                    &clipboard,
                    &model_name,
                    &api_key,
                    base_url.as_deref(),
                    &comment_char,
                    retries,
                    &self.logger,
                )
            } else {
                match self.ia.responder(
                    &clipboard,
                    system_cmd,
                    &model_name,
                    &api_key,
                    base_url.as_deref(),
                    retries,
                    &self.logger,
                ) {
                    Ok(raw_resp) => Ok(IA::extract_code_blocks(&raw_resp, &comment_char)),
                    Err(e) => Err(e),
                }
            }
        } else {
            self.ia.responder(
                &clipboard,
                system_cmd,
                &model_name,
                &api_key,
                base_url.as_deref(),
                retries,
                &self.logger,
            )
        };

        match response_res {
            Ok(response) => {
                self.logger.info("Resposta da IA recebida e aplicada com sucesso.");
                self.engineer.pos_processamento(&response);
            }
            Err(e) => {
                self.logger.error(&format!("Error in {} handler: {}", mode, e));
            }
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        self.logger.info(&format!("handle_ai_request levou {:.5} segundos para executar.", elapsed));
    }
}

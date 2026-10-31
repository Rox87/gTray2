//! data_manager.rs - Gerenciador de persistência e resolução de atalhos do gTray.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::models::{AppConfig, GtrayData, ModelConfig, ProviderConfig, ResolvedShortcut, ShortcutConfig};

pub const DEFAULT_DATA_PATH: &str = "assets/gtray_data.json";

pub fn get_default_provider_base_url(name: &str) -> Option<&'static str> {
    match name.to_lowercase().trim() {
        "openai" => Some("https://api.openai.com/v1"),
        "deepseek" => Some("https://api.deepseek.com"),
        "groq" => Some("https://api.groq.com/openai/v1"),
        "openrouter" => Some("https://openrouter.ai/api/v1"),
        "ollama" => Some("http://localhost:11434/v1"),
        "together" => Some("https://api.together.xyz/v1"),
        "mistral" => Some("https://api.mistral.ai/v1"),
        "gemini" | "google" => Some("https://generativelanguage.googleapis.com/v1beta/openai/"),
        _ => None,
    }
}

#[derive(Debug, Clone)]
pub struct GTrayDataManager {
    pub filepath: PathBuf,
    pub data: GtrayData,
}

impl Default for GTrayDataManager {
    fn default() -> Self {
        Self::new(DEFAULT_DATA_PATH)
    }
}

impl GTrayDataManager {
    pub fn new<P: AsRef<Path>>(filepath: P) -> Self {
        let mut manager = Self {
            filepath: filepath.as_ref().to_path_buf(),
            data: GtrayData::default(),
        };
        manager.load();
        manager
    }

    pub fn init_defaults(&mut self) {
        let mut default_key = String::new();
        let k_path = Path::new("assets").join("k.cfg");
        if k_path.is_file() {
            if let Ok(content) = fs::read_to_string(&k_path) {
                default_key = content.trim().to_string();
            }
        }

        let mut providers = BTreeMap::new();
        providers.insert(
            "openai".to_string(),
            ProviderConfig {
                api_key: default_key,
                base_url: None,
            },
        );

        let mut models = BTreeMap::new();
        models.insert(
            "gpt-4o-mini".to_string(),
            ModelConfig {
                provider: "openai".to_string(),
            },
        );
        models.insert(
            "gpt-4o".to_string(),
            ModelConfig {
                provider: "openai".to_string(),
            },
        );

        let mut shortcuts = BTreeMap::new();
        shortcuts.insert(
            "upgrade".to_string(),
            ShortcutConfig {
                key: "F8".to_string(),
                model: "gpt-4o-mini".to_string(),
                system_prompt: "Não responda o usuario, retorne apenas, sem rodeios, a própria entrada do usuário melhorada".to_string(),
                extract_blocks: false,
                comment_char: "#".to_string(),
                enabled: true,
            },
        );
        shortcuts.insert(
            "generic".to_string(),
            ShortcutConfig {
                key: "F7+CTRL+X".to_string(),
                model: "gpt-4o-mini".to_string(),
                system_prompt: String::new(),
                extract_blocks: false,
                comment_char: "#".to_string(),
                enabled: true,
            },
        );
        shortcuts.insert(
            "python".to_string(),
            ShortcutConfig {
                key: "F9".to_string(),
                model: "gpt-4o-mini".to_string(),
                system_prompt: String::new(),
                extract_blocks: false,
                comment_char: "#".to_string(),
                enabled: true,
            },
        );
        shortcuts.insert(
            "creation".to_string(),
            ShortcutConfig {
                key: "F11".to_string(),
                model: "gpt-4o-mini".to_string(),
                system_prompt: String::new(),
                extract_blocks: false,
                comment_char: "#".to_string(),
                enabled: true,
            },
        );

        self.data = GtrayData {
            app: AppConfig::default(),
            providers,
            models,
            shortcuts,
        };
        let _ = self.save();
    }

    pub fn load(&mut self) {
        if self.filepath.is_file() {
            if let Ok(content) = fs::read_to_string(&self.filepath) {
                if let Ok(data) = serde_json::from_str::<GtrayData>(&content) {
                    self.data = data;
                    return;
                }
            }
        }
        self.init_defaults();
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.filepath.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let json_str = serde_json::to_string_pretty(&self.data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        fs::write(&self.filepath, json_str)
    }

    // -------------------------------------------------------------
    // 0. Configurações Globais da Aplicação
    // -------------------------------------------------------------
    pub fn get_app_setting(&self, key: &str, default: &str) -> String {
        match key.to_lowercase().as_str() {
            "title" | "header_title" => self.data.app.title.clone(),
            "favicon" | "gton" | "gtoff" => self.data.app.favicon.clone(),
            "lbl_shortcut_state_rec" => self.data.app.lbl_shortcut_state_rec.clone(),
            "retry_clip" => self.data.app.retry_clip.to_string(),
            "retry_ia" => self.data.app.retry_ia.to_string(),
            "log" => self.data.app.log.clone(),
            _ => default.to_string(),
        }
    }

    pub fn set_app_setting(&mut self, key: &str, value: &str) {
        match key.to_lowercase().as_str() {
            "title" | "header_title" => self.data.app.title = value.to_string(),
            "favicon" | "gton" | "gtoff" => self.data.app.favicon = value.to_string(),
            "lbl_shortcut_state_rec" => self.data.app.lbl_shortcut_state_rec = value.to_string(),
            "retry_clip" => {
                if let Ok(v) = value.parse::<u32>() {
                    self.data.app.retry_clip = v;
                }
            }
            "retry_ia" => {
                if let Ok(v) = value.parse::<u32>() {
                    self.data.app.retry_ia = v;
                }
            }
            "log" => self.data.app.log = value.to_string(),
            _ => {}
        }
        let _ = self.save();
    }

    // -------------------------------------------------------------
    // 1. Provedor (nome, chave e base_url)
    // -------------------------------------------------------------
    pub fn get_providers(&self) -> &BTreeMap<String, ProviderConfig> {
        &self.data.providers
    }

    pub fn get_provider_key(&self, name: &str) -> Option<String> {
        self.data.providers.get(name).map(|p| p.api_key.clone())
    }

    pub fn get_provider_base_url(&self, name: &str) -> Option<String> {
        if let Some(p) = self.data.providers.get(name) {
            if let Some(ref url) = p.base_url {
                let trimmed = url.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
        get_default_provider_base_url(name).map(|s| s.to_string())
    }

    pub fn set_provider(&mut self, name: &str, api_key: &str, base_url: Option<&str>) {
        let name_trim = name.trim().to_string();
        if name_trim.is_empty() {
            return;
        }
        let url_opt = base_url
            .map(|u| u.trim().to_string())
            .filter(|u| !u.is_empty());

        self.data.providers.insert(
            name_trim,
            ProviderConfig {
                api_key: api_key.trim().to_string(),
                base_url: url_opt,
            },
        );
        let _ = self.save();
    }

    pub fn delete_provider(&mut self, name: &str) {
        if self.data.providers.remove(name).is_some() {
            let _ = self.save();
        }
    }

    // -------------------------------------------------------------
    // 2. Modelo (nome e provedor)
    // -------------------------------------------------------------
    pub fn get_models(&self) -> &BTreeMap<String, ModelConfig> {
        &self.data.models
    }

    pub fn get_model_provider(&self, name: &str) -> Option<String> {
        self.data.models.get(name).map(|m| m.provider.clone())
    }

    pub fn set_model(&mut self, name: &str, provider: &str) {
        let name_trim = name.trim().to_string();
        let provider_trim = provider.trim().to_string();
        if name_trim.is_empty() || provider_trim.is_empty() {
            return;
        }
        self.data.models.insert(
            name_trim,
            ModelConfig {
                provider: provider_trim,
            },
        );
        let _ = self.save();
    }

    pub fn delete_model(&mut self, name: &str) {
        if self.data.models.remove(name).is_some() {
            let _ = self.save();
        }
    }

    // -------------------------------------------------------------
    // 3. Atalho (nome, tecla, modelo, prompts, etc.)
    // -------------------------------------------------------------
    pub fn get_shortcuts(&self) -> &BTreeMap<String, ShortcutConfig> {
        &self.data.shortcuts
    }

    pub fn get_shortcut(&self, name: &str) -> Option<ShortcutConfig> {
        self.data.shortcuts.get(name).cloned().map(|mut sc| {
            if sc.system_prompt.is_empty() && name == "upgrade" {
                sc.system_prompt = "Não responda o usuario, retorne apenas, sem rodeios, a própria entrada do usuário melhorada".to_string();
            }
            if sc.comment_char.is_empty() {
                sc.comment_char = "#".to_string();
            }
            sc
        })
    }

    pub fn set_shortcut(
        &mut self,
        name: &str,
        key: &str,
        model: &str,
        system_prompt: &str,
        extract_blocks: bool,
        comment_char: &str,
        enabled: bool,
    ) {
        let name_trim = name.trim().to_string();
        if name_trim.is_empty() {
            return;
        }
        let mut comment = comment_char.trim().to_string();
        if comment.is_empty() {
            comment = "#".to_string();
        }

        self.data.shortcuts.insert(
            name_trim,
            ShortcutConfig {
                key: key.trim().to_string(),
                model: model.trim().to_string(),
                system_prompt: system_prompt.trim().to_string(),
                extract_blocks,
                comment_char: comment,
                enabled,
            },
        );
        let _ = self.save();
    }

    pub fn delete_shortcut(&mut self, name: &str) {
        if self.data.shortcuts.remove(name).is_some() {
            let _ = self.save();
        }
    }

    pub fn get_theme(&self) -> String {
        if self.data.app.theme.trim().is_empty() {
            "dark".to_string()
        } else {
            self.data.app.theme.clone()
        }
    }

    pub fn set_theme(&mut self, theme: &str) {
        self.data.app.theme = theme.to_string();
        let _ = self.save();
    }

    // -------------------------------------------------------------
    // Resolução Completa da Cadeia: Atalho -> Modelo -> Provedor -> Chave
    // -------------------------------------------------------------
    pub fn resolve_shortcut(&self, shortcut_name: &str) -> Option<ResolvedShortcut> {
        let sc = self.get_shortcut(shortcut_name)?;
        let key = sc.key;
        let model_name = sc.model;
        let system_prompt = sc.system_prompt;
        let extract_blocks = sc.extract_blocks;
        let comment_char = if sc.comment_char.trim().is_empty() {
            "#".to_string()
        } else {
            sc.comment_char
        };
        let enabled = sc.enabled;

        let provider_name = self
            .get_model_provider(&model_name)
            .unwrap_or_else(|| "openai".to_string());
        let api_key = self.get_provider_key(&provider_name).unwrap_or_default();
        let base_url = self.get_provider_base_url(&provider_name);

        Some(ResolvedShortcut {
            shortcut_name: shortcut_name.to_string(),
            key,
            model_name,
            provider_name,
            api_key,
            base_url,
            system_prompt,
            extract_blocks,
            comment_char,
            enabled,
        })
    }
}

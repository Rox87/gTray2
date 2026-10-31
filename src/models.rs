//! models.rs - Estruturas de dados e modelos de configuração do gTray.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppConfig {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_favicon")]
    pub favicon: String,
    #[serde(default = "default_lbl_shortcut_state_rec")]
    pub lbl_shortcut_state_rec: String,
    #[serde(default = "default_retry_clip")]
    pub retry_clip: u32,
    #[serde(default = "default_retry_ia")]
    pub retry_ia: u32,
    #[serde(default = "default_log")]
    pub log: String,
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_title() -> String {
    "gTray".to_string()
}
fn default_favicon() -> String {
    "assets/favicon.png".to_string()
}
fn default_lbl_shortcut_state_rec() -> String {
    "Pressione uma combinação do teclado".to_string()
}
fn default_retry_clip() -> u32 {
    140
}
fn default_retry_ia() -> u32 {
    14
}
fn default_log() -> String {
    "logs/app.log".to_string()
}
fn default_theme() -> String {
    "dark".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            title: default_title(),
            favicon: default_favicon(),
            lbl_shortcut_state_rec: default_lbl_shortcut_state_rec(),
            retry_clip: default_retry_clip(),
            retry_ia: default_retry_ia(),
            log: default_log(),
            theme: default_theme(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProviderConfig {
    #[serde(default)]
    pub api_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelConfig {
    #[serde(default)]
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ShortcutConfig {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub extract_blocks: bool,
    #[serde(default = "default_comment_char")]
    pub comment_char: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_comment_char() -> String {
    "#".to_string()
}
fn default_true() -> bool {
    true
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            key: String::new(),
            model: String::new(),
            system_prompt: String::new(),
            extract_blocks: false,
            comment_char: default_comment_char(),
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct GtrayData {
    #[serde(default)]
    pub app: AppConfig,
    #[serde(default)]
    pub providers: BTreeMap<String, ProviderConfig>,
    #[serde(default)]
    pub models: BTreeMap<String, ModelConfig>,
    #[serde(default)]
    pub shortcuts: BTreeMap<String, ShortcutConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedShortcut {
    pub shortcut_name: String,
    pub key: String,
    pub model_name: String,
    pub provider_name: String,
    pub api_key: String,
    pub base_url: Option<String>,
    pub system_prompt: String,
    pub extract_blocks: bool,
    pub comment_char: String,
    pub enabled: bool,
}

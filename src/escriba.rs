//! escriba.rs - Módulo de registro estruturado de logs e rotação para o gTray.

use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::models::ShortcutConfig;

pub const DEFAULT_LOG_PATH: &str = "logs/application.log";
pub const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024; // 10 MB
pub const MAX_BACKUPS: usize = 5;

#[derive(Debug, Clone)]
pub struct Escriba {
    name: String,
    log_path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl Default for Escriba {
    fn default() -> Self {
        Self::with_name("gTray", DEFAULT_LOG_PATH)
    }
}

pub trait ShortcutDisplay {
    fn key_str(&self) -> &str;
    fn details_str(&self) -> String {
        String::new()
    }
}

impl ShortcutDisplay for String {
    fn key_str(&self) -> &str {
        self.as_str()
    }
}

impl ShortcutDisplay for &str {
    fn key_str(&self) -> &str {
        self
    }
}

impl ShortcutDisplay for ShortcutConfig {
    fn key_str(&self) -> &str {
        if self.key.trim().is_empty() {
            "(nenhuma)"
        } else {
            self.key.trim()
        }
    }

    fn details_str(&self) -> String {
        let status_text = if self.enabled { "Ativo" } else { "Inativo" };
        let mut details = vec![format!("Status: {}", status_text)];

        if !self.model.is_empty() {
            details.push(format!("Modelo: {}", self.model));
        }
        if self.extract_blocks {
            details.push("Extração de blocos: Ativa".to_string());
        }

        format!(" ({})", details.join(" | "))
    }
}

pub trait AsActiveKey {
    fn as_active_key(&self) -> Option<&str>;
}

impl AsActiveKey for Option<&str> {
    fn as_active_key(&self) -> Option<&str> {
        *self
    }
}

impl AsActiveKey for &str {
    fn as_active_key(&self) -> Option<&str> {
        Some(*self)
    }
}

impl AsActiveKey for String {
    fn as_active_key(&self) -> Option<&str> {
        Some(self.as_str())
    }
}

impl Escriba {
    pub fn new<P: AsRef<Path>>(log_path: P) -> Self {
        Self::with_name("gTray", log_path)
    }

    pub fn with_name<P: AsRef<Path>>(name: &str, log_path: P) -> Self {
        let path = log_path.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                let _ = fs::create_dir_all(parent);
            }
        }
        Self {
            name: name.to_string(),
            log_path: path,
            lock: Arc::new(Mutex::new(())),
        }
    }

    fn check_rotate(&self) {
        if let Ok(metadata) = fs::metadata(&self.log_path) {
            if metadata.len() >= MAX_LOG_BYTES {
                for i in (1..MAX_BACKUPS).rev() {
                    let src = self.log_path.with_extension(format!("log.{}", i));
                    let dst = self.log_path.with_extension(format!("log.{}", i + 1));
                    if src.exists() {
                        let _ = fs::rename(src, dst);
                    }
                }
                let first_backup = self.log_path.with_extension("log.1");
                let _ = fs::rename(&self.log_path, first_backup);
            }
        }
    }

    fn write_log(&self, level: &str, msg: &str) {
        let _guard = self.lock.lock().unwrap();
        self.check_rotate();

        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let formatted = format!("{} [{}] [{}] {}\n", now, level, self.name, msg);

        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = file.write_all(formatted.as_bytes());
            let _ = file.flush();
        }

        // Se houver console conectado, imprime sem pânico se stdout for inválido (Windows Subsystem)
        let _ = write!(std::io::stdout(), "{}", formatted);
    }

    pub fn info(&self, msg: &str) {
        self.write_log("INFO", msg);
    }

    pub fn debug(&self, msg: &str) {
        self.write_log("DEBUG", msg);
    }

    pub fn warning(&self, msg: &str) {
        self.write_log("WARN", msg);
    }

    pub fn error(&self, msg: &str) {
        self.write_log("ERROR", msg);
    }

    pub fn log_header(&self, title: &str) {
        let border = "═".repeat(60);
        self.info(&border);
        self.info(&format!("  {}", title));
        self.info(&border);
    }

    pub fn log_shortcuts<'a, K, V, I, A>(&self, shortcuts: I, active_shortcut: A)
    where
        K: std::fmt::Display + 'a,
        V: ShortcutDisplay + 'a,
        I: IntoIterator<Item = (&'a K, &'a V)>,
        A: AsActiveKey,
    {
        self.info("Atalhos cadastrados no sistema:");
        let mut count = 0;
        for (name, sc) in shortcuts {
            count += 1;
            let key_display = sc.key_str();
            let details_str = sc.details_str();
            self.info(&format!("  • {}: {}{}", name, key_display, details_str));
        }
        if count == 0 {
            self.info("  (Nenhum atalho cadastrado no momento)");
        }

        let active = active_shortcut.as_active_key().unwrap_or("o atalho configurado");
        self.info(&format!(
            "pressione {} com um texto selecionado em um campo editável e aguarde...",
            active
        ));
    }
}

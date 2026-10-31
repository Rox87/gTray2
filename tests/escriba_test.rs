use gtray::escriba::Escriba;
use std::collections::HashMap;
use std::fs;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEST_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn unique_temp_path(prefix: &str) -> String {
    let count = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let file_name = format!("{}_{}_{}_{}.log", prefix, std::process::id(), now, count);
    std::env::temp_dir().join(file_name).to_str().unwrap().to_string()
}

#[test]
fn test_escriba_logging_and_file_creation() {
    let path = unique_temp_path("escriba_log");

    let escriba = Escriba::new(&path);
    escriba.info("Iniciando teste de log");
    escriba.warning("Alerta de teste");
    escriba.error("Erro de teste");

    let content = fs::read_to_string(&path).expect("Log file should exist and be readable");
    assert!(content.contains("[INFO]"));
    assert!(content.contains("Iniciando teste de log"));
    assert!(content.contains("[WARN]"));
    assert!(content.contains("Alerta de teste"));
    assert!(content.contains("[ERROR]"));
    assert!(content.contains("Erro de teste"));

    let mut shortcuts = HashMap::new();
    shortcuts.insert("upgrade".to_string(), "F8".to_string());
    shortcuts.insert("generic".to_string(), "F9".to_string());

    escriba.log_shortcuts(&shortcuts, "F8");

    let updated_content = fs::read_to_string(&path).unwrap();
    assert!(updated_content.contains("Atalhos cadastrados"));
    assert!(updated_content.contains("upgrade: F8"));
    assert!(updated_content.contains("generic: F9"));

    let _ = fs::remove_file(path);
}

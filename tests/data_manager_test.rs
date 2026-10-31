use gtray::data_manager::GTrayDataManager;
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
    let file_name = format!("{}_{}_{}_{}.json", prefix, std::process::id(), now, count);
    std::env::temp_dir().join(file_name).to_str().unwrap().to_string()
}

#[test]
fn test_provider_crud() {
    let path = unique_temp_path("dm_prov");
    let mut dm = GTrayDataManager::new(&path);

    // Create / Update provider
    dm.set_provider("openai", "sk-test-123", None);
    assert_eq!(dm.get_provider_key("openai"), Some("sk-test-123".to_string()));

    let providers = dm.get_providers();
    assert!(providers.contains_key("openai"));
    assert_eq!(providers.get("openai").unwrap().api_key, "sk-test-123");

    // Remove provider
    dm.delete_provider("openai");
    assert!(!dm.get_providers().contains_key("openai"));
    assert_eq!(dm.get_provider_key("openai"), None);

    let _ = fs::remove_file(path);
}

#[test]
fn test_model_crud() {
    let path = unique_temp_path("dm_model");
    let mut dm = GTrayDataManager::new(&path);

    // Setup provider
    dm.set_provider("openai", "sk-test-123", None);

    // Create model linked to provider
    dm.set_model("gpt-4o-mini", "openai");
    assert_eq!(
        dm.get_model_provider("gpt-4o-mini"),
        Some("openai".to_string())
    );

    let models = dm.get_models();
    assert!(models.contains_key("gpt-4o-mini"));
    assert_eq!(models.get("gpt-4o-mini").unwrap().provider, "openai");

    // Remove model
    dm.delete_model("gpt-4o-mini");
    assert!(!dm.get_models().contains_key("gpt-4o-mini"));

    let _ = fs::remove_file(path);
}

#[test]
fn test_shortcut_crud_and_full_chain_lookup() {
    let path = unique_temp_path("dm_sc");
    let mut dm = GTrayDataManager::new(&path);

    // 1. Provedor
    dm.set_provider("anthropic", "sk-ant-test", None);

    // 2. Modelo
    dm.set_model("claude-3-5-sonnet", "anthropic");

    // 3. Atalho
    dm.set_shortcut(
        "upgrade",
        "Ctrl+Shift+U",
        "claude-3-5-sonnet",
        "Instrução de teste",
        true,
        "//",
        true,
    );

    let sc = dm.get_shortcut("upgrade").expect("Shortcut should exist");
    assert_eq!(sc.key, "Ctrl+Shift+U");
    assert_eq!(sc.model, "claude-3-5-sonnet");
    assert_eq!(sc.system_prompt, "Instrução de teste");
    assert!(sc.extract_blocks);
    assert_eq!(sc.comment_char, "//");
    assert!(sc.enabled);

    // Full chain resolution: Atalho -> Modelo -> Provedor -> Chave -> Base URL -> System Prompt -> Extração -> Ativo
    let chain = dm
        .resolve_shortcut("upgrade")
        .expect("Chain resolution failed");
    assert_eq!(chain.shortcut_name, "upgrade");
    assert_eq!(chain.key, "Ctrl+Shift+U");
    assert_eq!(chain.model_name, "claude-3-5-sonnet");
    assert_eq!(chain.provider_name, "anthropic");
    assert_eq!(chain.api_key, "sk-ant-test");
    assert_eq!(chain.system_prompt, "Instrução de teste");
    assert!(chain.extract_blocks);
    assert_eq!(chain.comment_char, "//");
    assert!(chain.enabled);

    // Teste de atalho desativado (enabled = false)
    dm.set_shortcut(
        "disabled_sc",
        "Ctrl+Shift+D",
        "claude-3-5-sonnet",
        "",
        false,
        "#",
        false,
    );
    let sc_disabled = dm.get_shortcut("disabled_sc").unwrap();
    assert!(!sc_disabled.enabled);

    let chain_disabled = dm.resolve_shortcut("disabled_sc").unwrap();
    assert!(!chain_disabled.enabled);

    let _ = fs::remove_file(path);
}

#[test]
fn test_provider_base_url_auto_and_custom() {
    let path = unique_temp_path("dm_url");
    let mut dm = GTrayDataManager::new(&path);

    // Provedor conhecido automático (ex: deepseek)
    dm.set_provider("deepseek", "sk-ds-key", None);
    assert_eq!(
        dm.get_provider_base_url("deepseek"),
        Some("https://api.deepseek.com".to_string())
    );

    // Provedor com base_url customizada
    dm.set_provider(
        "local_vllm",
        "token-123",
        Some("http://localhost:8000/v1"),
    );
    assert_eq!(
        dm.get_provider_base_url("local_vllm"),
        Some("http://localhost:8000/v1".to_string())
    );

    let _ = fs::remove_file(path);
}

#[test]
fn test_persistence_between_instances() {
    let path = unique_temp_path("dm_persist");

    {
        let mut dm = GTrayDataManager::new(&path);
        dm.set_provider("groq", "gsk-123", None);
        dm.set_model("llama-3.3-70b", "groq");
        dm.set_shortcut(
            "fast",
            "F8",
            "llama-3.3-70b",
            "Responda rápido",
            false,
            "#",
            true,
        );
        let _ = dm.save();
    }

    // Load fresh manager on same file
    let dm2 = GTrayDataManager::new(&path);
    assert_eq!(dm2.get_provider_key("groq"), Some("gsk-123".to_string()));
    assert_eq!(
        dm2.get_model_provider("llama-3.3-70b"),
        Some("groq".to_string())
    );
    let sc = dm2.get_shortcut("fast").unwrap();
    assert_eq!(sc.key, "F8");
    assert_eq!(sc.model, "llama-3.3-70b");
    assert_eq!(sc.system_prompt, "Responda rápido");

    let _ = fs::remove_file(path);
}

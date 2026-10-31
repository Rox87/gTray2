use gtray::ia::extract_code_blocks;
use gtray::klisten::format_key_combination;

#[test]
fn test_extract_code_blocks_multiple_with_filenames() {
    let ai_response = "Aqui está o script principal:\n\
main.py\n\
```python\n\
print('hello main')\n\
```\n\n\
E aqui está o módulo de utilitários:\n\
utils.py:\n\
```python\n\
def add(a, b): return a + b\n\
```";

    let extracted = extract_code_blocks(ai_response, "#");
    let expected = "# main.py\nprint('hello main')\n\n# utils.py\ndef add(a, b): return a + b";

    assert_eq!(extracted.trim(), expected.trim());
}

#[test]
fn test_extract_code_blocks_markdown_filename_and_custom_comment() {
    let ai_response = "Veja o arquivo abaixo:\n\
**app.js**\n\
```javascript\n\
console.log('running');\n\
```";

    let extracted = extract_code_blocks(ai_response, "//");
    let expected = "// app.js\nconsole.log('running');";

    assert_eq!(extracted.trim(), expected.trim());
}

#[test]
fn test_extract_code_blocks_plain_no_filename() {
    let ai_response = "```python\nx = 42\n```";
    let extracted = extract_code_blocks(ai_response, "#");
    assert_eq!(extracted.trim(), "x = 42");
}

#[test]
fn test_extract_code_blocks_no_code() {
    let ai_response = "Apenas uma resposta em texto puro.";
    let extracted = extract_code_blocks(ai_response, "#");
    assert_eq!(extracted, ai_response);
}

#[test]
fn test_canonical_key_combination_sorting() {
    let raw1 = vec!["Shift".to_string(), "Ctrl".to_string(), "k".to_string()];
    assert_eq!(format_key_combination(&raw1), "Ctrl+Shift+K");

    let raw2 = vec!["Alt".to_string(), "Ctrl".to_string(), "F8".to_string()];
    assert_eq!(format_key_combination(&raw2), "Ctrl+Alt+F8");

    let raw3 = vec!["f9".to_string()];
    assert_eq!(format_key_combination(&raw3), "F9");
}

#[test]
fn test_https_connectivity_supported() {
    let res = ureq::get("https://api.deepseek.com").call();
    match res {
        Ok(_) => {}
        Err(ureq::Error::Status(_, _)) => {
            // Conexão HTTPS realizada com sucesso
        }
        Err(ureq::Error::Transport(t)) => {
            let msg = t.to_string();
            assert!(
                !msg.contains("no TLS backend is configured"),
                "Erro de backend TLS: {}",
                msg
            );
        }
    }
}

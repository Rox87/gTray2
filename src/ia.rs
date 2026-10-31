//! ia.rs - Camada de integração unificada com modelos de linguagem (OpenAI-compatible).

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

use crate::escriba::Escriba;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    #[serde(default)]
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Clone)]
pub struct IA {
    pub default_api_key: String,
    pub default_base_url: Option<String>,
}

impl IA {
    pub fn new(api_key: &str, base_url: Option<&str>) -> Self {
        Self {
            default_api_key: api_key.to_string(),
            default_base_url: base_url.map(|s| s.to_string()),
        }
    }

    pub fn extract_code_blocks(response: &str, comment_char: &str) -> String {
        if response.is_empty() {
            return String::new();
        }

        let re_blocks = match Regex::new(r"(?s)```[a-zA-Z0-9_\-\+\.]*[\r\n]+(.*?)```") {
            Ok(r) => r,
            Err(_) => return response.to_string(),
        };

        let matches: Vec<_> = re_blocks.find_iter(response).collect();
        if matches.is_empty() {
            return response.to_string();
        }

        let comment = if comment_char.trim().is_empty() {
            "#"
        } else {
            comment_char.trim()
        };

        let mut blocks = Vec::new();
        let mut last_end = 0;

        for m in matches {
            let matched_str = m.as_str();
            // Extract the inside content between the backticks
            let inner_re = Regex::new(r"(?s)^```[a-zA-Z0-9_\-\+\.]*[\r\n]+(.*?)```$").unwrap();
            let block_content = if let Some(caps) = inner_re.captures(matched_str) {
                caps.get(1)
                    .map(|c| c.as_str().trim_matches(|ch| ch == '\r' || ch == '\n'))
                    .unwrap_or("")
            } else {
                ""
            };

            let preceding_text = &response[last_end..m.start()];
            last_end = m.end();

            let mut filename = String::new();
            for line in preceding_text.lines().rev() {
                let line_clean = line.trim();
                if !line_clean.is_empty() {
                    let words: Vec<&str> = line_clean.split_whitespace().collect();
                    if let Some(first_word) = words.first() {
                        let cleaned = first_word.trim_matches(|c: char| {
                            c == '*'
                                || c == '_'
                                || c == '`'
                                || c == '"'
                                || c == '\''
                                || c == '('
                                || c == '['
                                || c == ')'
                                || c == ']'
                                || c == ':'
                                || c == ','
                        });
                        if !cleaned.is_empty() {
                            filename = cleaned.to_string();
                        }
                    }
                    break;
                }
            }

            if !filename.is_empty() {
                blocks.push(format!("{} {}\n{}", comment, filename, block_content));
            } else {
                blocks.push(block_content.to_string());
            }
        }

        blocks.join("\n\n")
    }

    pub fn responder(
        &self,
        prompt: &str,
        system_cmd: &str,
        model: &str,
        api_key: &str,
        base_url: Option<&str>,
        retries: u32,
        logger: &Escriba,
    ) -> Result<String, String> {
        let mut messages = Vec::new();
        if !system_cmd.trim().is_empty() {
            messages.push(json!({
                "role": "system",
                "content": system_cmd.trim()
            }));
        }
        messages.push(json!({
            "role": "user",
            "content": prompt
        }));

        let payload = json!({
            "model": model,
            "messages": messages
        });

        let key = if !api_key.is_empty() {
            api_key
        } else {
            &self.default_api_key
        };

        let raw_base = base_url
            .or(self.default_base_url.as_deref())
            .unwrap_or("https://api.openai.com/v1");

        let endpoint = if raw_base.ends_with('/') {
            format!("{}chat/completions", raw_base)
        } else {
            format!("{}/chat/completions", raw_base)
        };

        let attempts = retries.max(1);
        let mut last_error = String::new();

        for attempt in 0..attempts {
            let mut req = ureq::post(&endpoint);
            if !key.is_empty() {
                req = req.set("Authorization", &format!("Bearer {}", key));
            }
            req = req.set("Content-Type", "application/json");

            match req.send_json(payload.clone()) {
                Ok(resp) => match resp.into_json::<ChatCompletionResponse>() {
                    Ok(parsed) => {
                        if let Some(first_choice) = parsed.choices.into_iter().next() {
                            let text = first_choice.message.content;
                            let preview = if text.len() > 100 {
                                &text[..100]
                            } else {
                                &text
                            };
                            logger.info(&format!("Response received: {}...", preview));
                            return Ok(text);
                        } else {
                            last_error = "Nenhuma resposta recebida da IA (choices vazias).".to_string();
                        }
                    }
                    Err(e) => {
                        last_error = format!("Falha ao decodificar JSON da resposta: {}", e);
                    }
                },
                Err(e) => {
                    let err_str = e.to_string();
                    if err_str.contains("401") || err_str.contains("403") {
                        logger.error(&format!("AI authentication error (verifique chave/provedor): {}", err_str));
                        return Err(err_str);
                    }
                    if err_str.contains("400") || err_str.contains("404") {
                        logger.error(&format!("AI request error (verifique modelo '{}'): {}", model, err_str));
                        return Err(err_str);
                    }
                    last_error = format!("AI request failed: {}", err_str);
                    logger.warning(&format!("AI request failed (tentativas restantes: {}): {}", attempts - 1 - attempt, err_str));
                    sleep(Duration::from_millis(500));
                }
            }
        }

        Err(last_error)
    }

    pub fn python_responder(
        &self,
        prompt: &str,
        model: &str,
        api_key: &str,
        base_url: Option<&str>,
        comment_char: &str,
        retries: u32,
        logger: &Escriba,
    ) -> Result<String, String> {
        let re_files = Regex::new(r"@@([^@]+[\\/][^@]+)@").unwrap();
        let mut final_prompt = prompt.to_string();

        for cap in re_files.captures_iter(prompt) {
            if let Some(caminho_match) = cap.get(1) {
                let caminho = caminho_match.as_str();
                logger.info(&format!("Reading file: {}", caminho));
                if Path::new(caminho).is_file() {
                    match fs::read_to_string(caminho) {
                        Ok(content) => {
                            final_prompt.push_str(&format!("\n#{}\n{}", caminho, content));
                        }
                        Err(e) => {
                            logger.error(&format!("Error reading file {}: {}", caminho, e));
                        }
                    }
                }
            }
        }

        let system_cmd = format!(
            "seja extremamente breve e retorne apenas o nome entre ** seguido do bloco código python para cada arquivo da solução a seguir: {}",
            final_prompt
        );

        let coms = self.responder(
            &final_prompt,
            &system_cmd,
            model,
            api_key,
            base_url,
            retries,
            logger,
        )?;

        let comment = if comment_char.trim().is_empty() {
            "#"
        } else {
            comment_char.trim()
        };

        let mut prompt_lines: Vec<String> = final_prompt
            .trim()
            .lines()
            .map(|l| format!("{} {}", comment, l.trim_end()))
            .collect();

        let mut final_code = if !prompt_lines.is_empty() {
            prompt_lines.push(String::new());
            prompt_lines.join("\n")
        } else {
            String::new()
        };

        let re_pattern1 = Regex::new(r"(?s)\*\*(.*?)\*\*\n```(.*?)\n(.*?)```").unwrap();
        let matches1: Vec<_> = re_pattern1.captures_iter(&coms).collect();

        if !matches1.is_empty() {
            for cap in matches1 {
                let filename = cap.get(1).map_or("", |m| m.as_str()).trim();
                let lang = cap.get(2).map_or("", |m| m.as_str()).trim();
                let bloco = cap.get(3).map_or("", |m| m.as_str()).trim();
                logger.info(&format!("Generated code for: {}", filename));
                final_code.push_str(&format!("{} {} - {}\n{}\n", comment, lang, filename, bloco));
            }
        } else {
            let re_pattern2 = Regex::new(r"(?s)```([a-zA-Z0-9_\-\+\.]*)\n(.*?)```").unwrap();
            let matches2: Vec<_> = re_pattern2.captures_iter(&coms).collect();
            if !matches2.is_empty() {
                for cap in matches2 {
                    let raw_lang = cap.get(1).map_or("", |m| m.as_str()).trim();
                    let lang_label = if raw_lang.is_empty() { "python" } else { raw_lang };
                    let bloco = cap.get(2).map_or("", |m| m.as_str()).trim();
                    final_code.push_str(&format!("{} {} - script.py\n{}\n", comment, lang_label, bloco));
                }
            } else {
                final_code.push_str(&coms);
            }
        }

        Ok(final_code.trim_end().to_string())
    }
}

pub fn extract_code_blocks(response: &str, comment_char: &str) -> String {
    IA::extract_code_blocks(response, comment_char)
}

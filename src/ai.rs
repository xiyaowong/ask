use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::time::Duration;

use crate::dprintln;

static SYSTEM_PROMPT: &str = "Your name is Ask, and you are a fast, concise command-line AI assistant. If two inputs are given, treat the first as a prompt preset. Reply in the user's language.";
static DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
static DEEPSEEK_API_URL: &str = "https://api.deepseek.com/chat/completions";
static QWEN_API_URL: &str = "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions";

fn openai(
    messages: &[String],
    model: &str,
    timeout: Option<u64>,
    api_url: &str,
    api_key: &str,
) -> Result<String> {
    let mut messages: Vec<HashMap<String, String>> = messages
        .iter()
        .map(|m| {
            HashMap::from([
                ("role".to_owned(), "user".to_owned()),
                ("content".to_owned(), m.to_owned()),
            ])
        })
        .collect();

    messages.insert(
        0,
        HashMap::from([
            ("role".to_owned(), "system".to_owned()),
            ("content".to_owned(), SYSTEM_PROMPT.to_owned()),
        ]),
    );

    let body = json!({
        "model": model,
        "messages": messages,
        "stream": true
    });

    dprintln!("{:#?}", body);

    let resp = reqwest::blocking::Client::new()
        .post(api_url)
        .timeout(
            timeout
                .map(|t| Duration::from_secs(t))
                .unwrap_or(DEFAULT_REQUEST_TIMEOUT),
        )
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send()?;

    if resp.status().is_success() {
        let mut result = String::new();
        let mut lines = BufReader::new(resp).lines();

        while let Some(Ok(line)) = lines.next() {
            if line.starts_with("data: ") {
                let json_str = &line[6..];
                if json_str.trim() == "[DONE]" {
                    break;
                }
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                        result.push_str(content);
                        std::io::Write::flush(&mut std::io::stdout()).ok();
                    }
                }
            }
        }
        Ok(result)
    } else {
        Err(anyhow::anyhow!(
            "Request failed with status: {} {:?}",
            resp.status(),
            resp.text().unwrap_or_default()
        ))
    }
}

pub fn deepseek(
    messages: &[String],
    api_key: &str,
    model: &str,
    timeout: Option<u64>,
) -> Result<String> {
    openai(messages, model, timeout, DEEPSEEK_API_URL, api_key)
}

pub fn qwen(
    messages: &[String],
    api_key: &str,
    model: &str,
    timeout: Option<u64>,
) -> Result<String> {
    openai(messages, model, timeout, QWEN_API_URL, api_key)
}

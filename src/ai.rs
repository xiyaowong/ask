use anyhow::Result;
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

static SYSTEM_PROMPT: &str = "Your name is Ask, and you are a fast, concise command-line AI assistant. If two inputs are given, treat the first as a prompt preset. Reply in the user's language.";
static DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
static DEEPSEEK_API_URL: &str = "https://api.deepseek.com/chat/completions";

pub fn deepseek(
    messages: &[String],
    api_key: &str,
    model: &str,
    timeout: Option<u64>,
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
        "stream": false
    });

    #[cfg(debug_assertions)]
    println!("{:#?}", body);

    let resp = reqwest::blocking::Client::new()
        .post(DEEPSEEK_API_URL)
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
        let json_resp: serde_json::Value = resp.json()?;
        if let Some(reply) = json_resp["choices"][0]["message"]["content"].as_str() {
            Ok(reply.into())
        } else {
            Err(anyhow::anyhow!("No reply from assistant."))
        }
    } else {
        Err(anyhow::anyhow!(
            "Request failed with status: {}",
            resp.status()
        ))
    }
}

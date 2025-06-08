use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

static SYSTEM_PROMPT: &str = "this is ask.";
static REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

pub fn deepseek(messages: &Vec<String>, api_key: &str) -> String {
    let url = "https://api.deepseek.com/chat/completions";

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

    // 构造请求体
    let body = json!({
        "model": "deepseek-chat",
        "messages": messages,
        "stream": false
    });

    #[cfg(debug_assertions)]
    println!("{:#?}", body);

    // return "".into();

    let response = reqwest::blocking::Client::new()
        .post(url)
        .timeout(REQUEST_TIMEOUT)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {api_key}"))
        .json(&body)
        .send();

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let json_resp = resp.json();
                if let Err(err) = json_resp {
                    return format!("Request failed: {err}");
                }

                let json_resp: serde_json::Value = json_resp.unwrap();
                if let Some(reply) = json_resp["choices"][0]["message"]["content"].as_str() {
                    reply.into()
                } else {
                    "No reply from assistant.".into()
                }
            } else {
                format!("Request failed with status: {}", resp.status())
            }
        }
        Err(err) => format!("Error sending request: {err}"),
    }
}

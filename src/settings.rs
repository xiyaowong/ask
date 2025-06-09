use crate::command::{AIModel, AIProvider};
use anyhow::Result;
use config::{Config, FileFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path;

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    /// The AI provider to use for questions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<AIProvider>,
    /// The AI model to use for questions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<AIModel>,
    /// The timeout for AI requests in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Presets for common questions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presets: Option<HashMap<String, String>>,
    /// DeepSeek API key
    #[serde(skip_serializing)]
    pub deepseek_key: Option<String>,
    /// Grok API key
    #[serde(skip_serializing)]
    pub grok_key: Option<String>,
}

impl Settings {
    fn get_config_path() -> String {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let config_path = home_dir.join(".config").join("ask.json");
        std::fs::create_dir_all(config_path.parent().unwrap())
            .expect("Failed to create config directory");
        config_path.to_str().unwrap().to_string()
    }

    pub fn load() -> Result<Settings> {
        let config_path = Self::get_config_path();

        let result = Config::builder()
            .add_source(
                config::File::with_name(&config_path)
                    .required(false)
                    .format(FileFormat::Json),
            )
            .add_source(config::Environment::with_prefix("ASK").ignore_empty(true))
            .build()?
            .try_deserialize::<Settings>()?;

        Ok(result)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path();

        if path::Path::new(&config_path).exists()
            || self.provider.is_none()
            || self.model.is_none()
            || self.timeout.is_none()
            || self.presets.is_none()
            || self.deepseek_key.is_none()
            || self.grok_key.is_none()
        {
            let json = serde_json::to_string_pretty(self)?;
            std::fs::write(config_path, json)?;
        }

        Ok(())
    }
}

use crate::command::AIProvider;
use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Error;
use std::path;

#[derive(Deserialize, Serialize, Debug)]
pub struct Settings {
    pub provider: Option<AIProvider>,
    pub presets: Option<Vec<HashMap<String, String>>>,
    pub last_preset: Option<String>,

    pub deepseek_key: Option<String>,
    pub grok3_key: Option<String>,
}

impl Settings {
    fn get_config_path() -> String {
        let home_dir = dirs::home_dir().expect("Could not find home directory");
        let config_path = home_dir.join(".config").join("ask").join("config.json");
        std::fs::create_dir_all(config_path.parent().unwrap())
            .expect("Failed to create config directory");
        config_path.to_str().unwrap().to_string()
    }

    pub fn load() -> Result<Settings, ConfigError> {
        let config_path = Self::get_config_path();
        Config::builder()
            // .set_default("provider", "")?
            // .set_default("deepseek_key", None)?
            // .set_default("grok3_key", None)?
            // .set_default("presets", vec![])?
            .add_source(config::File::with_name(&config_path).required(false))
            .add_source(config::Environment::with_prefix("ASK").ignore_empty(true))
            .build()?
            .try_deserialize::<Settings>()
    }

    pub fn save(&self) -> Result<(), Error> {
        let config_path = Self::get_config_path();

        if !path::Path::new(&config_path).exists()
            && self.provider.is_none()
            && self.presets.is_none()
            && self.last_preset.is_none()
            && self.deepseek_key.is_none()
            && self.grok3_key.is_none()
        {
            Ok(())
        } else {
            let json = serde_json::to_string_pretty(self)?;
            std::fs::write(config_path, json)
        }
    }
}

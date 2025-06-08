#![allow(dead_code)]

use clap::{Args, Parser};
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Parser, Debug)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Set the AI provider
    Use(ConfigUseArgs),
    /// List all presets
    List,
    /// Manage individual presets
    Set(ConfigSetArgs),
    /// Get a specific preset
    Get(ConfigGetArgs),
    /// Delete a specific preset
    Delete(ConfigDeleteArgs),
}

#[derive(Args, Debug)]
pub struct ConfigUseArgs {
    #[arg(value_enum, help = "Select the AI provider to use")]
    pub provider: AIProvider,
}

#[derive(ValueEnum, Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum AIProvider {
    #[value(name = "deepseek")]
    DeepSeek,
    #[value(name = "grok3")]
    Grok3,
}

#[derive(Args, Debug)]
pub struct ConfigSetArgs {
    /// Name of the preset
    pub name: String,
    /// Prompt for the preset
    pub prompt: Vec<String>,
}

impl ConfigSetArgs {
    /// Join the prompt strings into a single string
    pub fn prompt_string(&self) -> String {
        self.prompt.join(" ")
    }
}

#[derive(Args, Debug)]
pub struct ConfigGetArgs {
    /// Name of the preset to retrieve
    pub name: String,
}

#[derive(Args, Debug)]
pub struct ConfigDeleteArgs {
    /// Name of the preset to delete
    pub name: String,
}

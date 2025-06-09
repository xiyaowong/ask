#![allow(dead_code)]

use clap::{Args, Parser};
use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(ValueEnum, Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum AIProvider {
    #[value(name = "deepseek")]
    DeepSeek,
    #[value(name = "grok")]
    Grok,
}

#[derive(ValueEnum, Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum AIModel {
    #[value(name = "deepseek-chat")]
    DeepSeekChat,
    #[value(name = "grok-3")]
    Grok3,
}

impl AIModel {
    pub fn name(&self) -> &str {
        match self {
            AIModel::DeepSeekChat => "deepseek-chat",
            AIModel::Grok3 => "grok-3",
        }
    }
}

#[derive(Parser, Debug)]
#[clap(
    before_help = r#"
ask is a command-line tool that makes it easier to get quick answers to simple questions compared to using a web browser.

- Ask directly - ask {question}
- Use preset - ask {preset} {question}"#,
    after_help = r#"
Environment Variables for API Keys

  - DeepSeek - export ASK_DEEPSEEK_KEY={your key}
  - Grok - export ASK_GROK_KEY={your key}

Examples:

  ask config provider deepseek
  ask config model deepseek-chat
  ask config timeout 60

  ask hello
  ask preset set rust You are a Rust programming expert. Answer questions about Rust programming.
  ask rust Tell me about the Ownership system in Rust.
"#
)]
pub struct AskArgsParser {
    #[command(subcommand)]
    pub command: AskCommand,
}

#[derive(Subcommand, Debug)]
pub enum AskCommand {
    /// Manage configuration settings
    Config(ConfigCommand),
    /// Manage AI presets
    Preset(PresetCommand),
}

// Configuration management commands

#[derive(Args, Debug)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommand {
    /// List current configuration settings
    Show,
    /// Set AI provider to use
    Provider(ConfigProviderArgs),
    /// Set timeout for requests
    Timeout(ConfigTimeoutArgs),
    /// Set AI model to use
    Model(ConfigModelArgs),
}

#[derive(Args, Debug)]
pub struct ConfigProviderArgs {
    #[arg(value_enum, help = "Select the AI provider to use")]
    pub provider: AIProvider,
}

#[derive(Args, Debug)]
pub struct ConfigTimeoutArgs {
    #[arg(value_enum, help = "Set the request timeout in seconds")]
    pub timeout: u64,
}

#[derive(Args, Debug)]
pub struct ConfigModelArgs {
    #[arg(value_enum, help = "Select the AI model to use")]
    pub model: AIModel,
}

// #region Preset management commands

#[derive(Args, Debug)]
pub struct PresetCommand {
    #[command(subcommand)]
    pub command: PresetSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum PresetSubcommand {
    /// Create a new preset
    Set(PresetSetArgs),
    /// List all presets
    List,
    /// Remove a specific preset
    Remove(PresetRemoveArgs),
}

#[derive(Args, Debug)]
pub struct PresetSetArgs {
    /// Name of the preset
    pub name: String,
    /// Prompt for the preset
    pub prompt: Vec<String>,
}

#[derive(Args, Debug)]
pub struct PresetRemoveArgs {
    /// Name of the preset to remove
    pub name: String,
}

// #endregion

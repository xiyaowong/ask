mod ai;
mod command;
mod settings;

use crate::command::{AIProvider, AskArgsParser, ConfigCommand};
use crate::settings::Settings;
use anyhow::{Context, Ok, Result};
use ask::dprintln;
use clap::{CommandFactory, Parser};
use command::AIModel;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::io::{Write, stdout};
use std::process::exit;
use std::time::Duration;

fn main() -> Result<()> {
    // Load settings
    let mut settings = Settings::load().with_context(|| "Failed to load settings")?;

    dprintln!("{:#?}", settings);

    // Parse command line arguments

    let std_args: Vec<String> = std::env::args().skip(1).collect();

    if std_args.is_empty()
        || std_args[0] == "help"
        || std_args[0] == "--help"
        || std_args[0] == "-h"
    {
        AskArgsParser::command().print_long_help()?;
        exit(0);
    }

    if std_args[0] != "config" && std_args[0] != "preset" {
        let (preset, question) = match std_args.len() {
            0 => unreachable!("We have already checked for empty args"),
            1 => (String::new(), std_args[0].clone()),
            _ => (std_args[0].clone(), std_args[1..].join(" ")),
        };
        handle_question(preset, question, &settings)?;
    } else {
        drop(std_args);

        let args = AskArgsParser::parse();

        dprintln!("{:#?}", args);

        match args.command {
            command::AskCommand::Config(cmd) => handle_config_command(cmd, &mut settings)?,
            command::AskCommand::Preset(cmd) => handle_preset_command(cmd, &mut settings)?,
        }
    }

    settings.save().with_context(|| "Failed to save settings")?;

    Ok(())
}

fn handle_preset_command(cmd: command::PresetCommand, settings: &mut Settings) -> Result<()> {
    match cmd.command {
        command::PresetSubcommand::Set(args) => {
            if settings.presets.is_none() {
                settings.presets = Some(HashMap::new());
            }
            if let Some(presets) = &mut settings.presets {
                presets.insert(args.name.clone(), args.prompt.join(" "));
                println!(
                    "Preset '{}' set with prompt: {}",
                    args.name,
                    args.prompt.join(" ")
                );
            }
        }
        command::PresetSubcommand::List => {
            let presets = match &settings.presets {
                Some(presets) => presets,
                None => &HashMap::new(),
            };

            if presets.is_empty() {
                println!("No presets found");
            } else {
                for (name, prompt) in presets {
                    println!("{} => {}", name, prompt);
                }
            }
        }
        command::PresetSubcommand::Remove(args) => {
            match &mut settings.presets {
                Some(presets) => match presets.remove(&args.name) {
                    Some(prompt) => {
                        println!("Removed preset '{}': {}", args.name, prompt);
                    }
                    None => {
                        println!("No preset found for '{}'", args.name);
                    }
                },
                None => {
                    println!("No presets found");
                }
            };
        }
    }

    Ok(())
}

fn handle_config_command(cmd: ConfigCommand, settings: &mut Settings) -> Result<()> {
    match cmd.command {
        command::ConfigSubcommand::Show => {
            let provider = settings
                .provider
                .as_ref()
                .map(|p| p.to_string())
                .unwrap_or("".to_string());

            let model = settings
                .model
                .as_ref()
                .map(|m| m.to_string())
                .unwrap_or("".to_string());

            let timeout = settings
                .timeout
                .map(|t| t.to_string())
                .unwrap_or("".to_string());

            println!("provider => {provider}");
            println!("model => {model}");
            println!("timeout => {timeout}");
        }
        command::ConfigSubcommand::Provider(args) => {
            settings.provider = Some(args.provider);
            println!("AI provider set to: {}", args.provider);
        }
        command::ConfigSubcommand::Timeout(args) => {
            settings.timeout = Some(args.timeout);
            println!("Request timeout set to: {} seconds", args.timeout);
        }
        command::ConfigSubcommand::Model(args) => {
            settings.model = Some(args.model);
            println!("AI model set to: {}", args.model);
        }
    }

    Ok(())
}

fn handle_question(preset: String, question: String, settings: &Settings) -> Result<()> {
    validate_ai_settings(settings)?;

    let mut messages = Vec::<String>::new();

    let preset_prompt = settings
        .presets
        .as_ref()
        .and_then(|presets| presets.get(&preset))
        .map(|prompt| prompt.to_owned())
        .unwrap_or_default();

    if preset_prompt.is_empty() {
        messages.push(format!("{preset} {question}"));
    } else {
        messages.push(preset_prompt);
        messages.push(question.to_owned());
    }

    let messages = messages
        .iter()
        .map(|msg| msg.trim().to_string())
        .filter(|msg| !msg.is_empty())
        .collect::<Vec<String>>();

    dprintln!("messages: {:?}", messages);

    stdout().flush().unwrap();
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner}")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(100));

    let reply = match settings.provider.unwrap() {
        AIProvider::DeepSeek => {
            let key = settings.deepseek_key.as_ref().unwrap();
            let model = settings.model.as_ref().unwrap().name();
            ai::deepseek(&messages, key, model, settings.timeout)?
        }
        AIProvider::Grok => {
            todo!()
        }
        AIProvider::Qwen => {
            let key = settings.qwen_key.as_ref().unwrap();
            let model = settings.model.as_ref().unwrap().name();
            ai::qwen(&messages, key, model, settings.timeout)?
        }
    };

    spinner.finish();

    if !reply.is_empty() {
        termimad::print_text(reply.as_str());
    }

    Ok(())
}

fn validate_ai_settings(settings: &Settings) -> Result<()> {
    if settings.provider.is_none() {
        return Err(anyhow::anyhow!("AI provider is not set"));
    }

    if settings.model.is_none() {
        return Err(anyhow::anyhow!("AI model is not set"));
    }

    match settings.provider.unwrap() {
        AIProvider::DeepSeek => {
            if settings.deepseek_key.is_none() {
                return Err(anyhow::anyhow!("DeepSeek API key is not set"));
            }

            match settings.model.unwrap() {
                AIModel::DeepSeekChat => {}
                _ => {
                    return Err(anyhow::anyhow!(
                        "DeepSeek provider only supports deepseek-chat model"
                    ));
                }
            }
        }
        AIProvider::Grok => {
            if settings.grok_key.is_none() {
                return Err(anyhow::anyhow!("Grok API key is not set"));
            }
            match settings.model.unwrap() {
                AIModel::Grok3 => {}
                _ => {
                    return Err(anyhow::anyhow!("Grok provider only supports Grok3 model"));
                }
            }
        }
        AIProvider::Qwen => {
            if settings.qwen_key.is_none() {
                return Err(anyhow::anyhow!("Qwen API key is not set"));
            }

            match settings.model.unwrap() {
                AIModel::QwenPlus | AIModel::QwenFlash => {}
                _ => {
                    return Err(anyhow::anyhow!(
                        "Qwen provider only supports QwenPlus and QwenFlash models"
                    ));
                }
            }
        }
    }

    Ok(())
}

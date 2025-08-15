mod ai;
mod command;
mod settings;

use crate::command::{AIProvider, AskArgsParser, ConfigCommand};
use crate::settings::Settings;
use anyhow::{Context, Ok, Result};
use ask::dprintln;
use clap::{CommandFactory, Parser};
use command::AIModel;
use ratatui::crossterm::event;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::widgets::{Paragraph, Wrap};
use std::collections::HashMap;
use std::io::{Write, stdout};
use std::process::exit;
use std::sync::mpsc::Receiver;
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

    handle_reply(&question.as_ref(), reply)?;
    Ok(())
}

fn handle_reply(question: &str, rx: Receiver<String>) -> Result<()> {
    let mut terminal = ratatui::init();

    let mut markdown_content = String::from("Loading...");
    let mut scroll = 0;

    loop {
        rx.recv()
            .map(|content| {
                markdown_content = content;
            })
            .unwrap_or_else(|_| {
                return;
            });

        terminal.draw(|f| {
            let area = f.area();
            let chunks = Layout::vertical([Constraint::Length(1), Constraint::Fill(1)].as_ref())
                .spacing(1)
                .margin(1)
                .split(area);

            let title_paragraph = Paragraph::new("[Press q to exit]")
                .bold()
                .alignment(ratatui::layout::Alignment::Left);

            let md = format!(
                "# [Question]\n\n{}\n\n---\n\n# [Response]\n\n{}",
                question, markdown_content
            );
            let md = tui_markdown::from_str(&md);
            let paragraph = Paragraph::new(md)
                .alignment(ratatui::layout::Alignment::Left)
                .wrap(Wrap { trim: true })
                .scroll((scroll, 0));

            f.render_widget(title_paragraph, chunks[0]);
            f.render_widget(paragraph, chunks[1]);
        })?;

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Up => {
                        if scroll > 0 {
                            scroll -= 1;
                        }
                    }
                    KeyCode::Down => {
                        scroll += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    ratatui::restore();

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

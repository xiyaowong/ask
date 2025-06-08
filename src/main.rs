mod ai;
mod command;
mod settings;

use crate::command::{AIProvider, ConfigArgs, ConfigCommand};
use crate::settings::Settings;
use clap::Parser;
use std::collections::HashMap;
use std::process::exit;

fn main() {
    let mut settings = Settings::load().unwrap_or_else(|err| {
        eprintln!("Error loading configuration: {}", err);
        exit(1);
    });

    #[cfg(debug_assertions)]
    println!("\n\n{:#?}\n\n", settings);

    let config_args = ConfigArgs::try_parse();
    if let Ok(args) = config_args {
        handle_config_args(&args, &mut settings);

        #[cfg(debug_assertions)]
        println!("\n\n{:#?}\n\n", settings);

        settings.save().unwrap();
        return;
    }

    let args: Vec<String> = std::env::args().skip(1).collect();

    let (a, b) = match args.len() {
        0 => {
            print_help();
            return;
        }
        1 => (args[0].clone(), "".to_owned()),
        _ => (args[0].clone(), args[1..].join(" ")),
    };

    let subcommands = vec!["use", "set", "get", "delete", "list"];
    if subcommands.contains(&a.as_str()) {
        // this definitely will fail, but we just want it to print the help message
        ConfigArgs::parse();
    }

    let want_help = args
        .iter()
        .any(|arg| arg == "--help" || arg == "-h" || arg == "help");
    if want_help {
        print_help();
        return;
    }

    handle_question(&a, &b, &mut settings);

    #[cfg(debug_assertions)]
    println!("\n\n{:#?}\n\n", settings);

    settings.save().unwrap();
}

fn handle_question(preset: &String, question: &String, settings: &mut Settings) {
    let mut messages = Vec::<String>::new();

    let preset_prompt = settings
        .presets
        .as_ref()
        .and_then(|presets| presets.get(preset))
        .map(|prompt| prompt.to_owned())
        .unwrap_or_default();

    if preset_prompt.is_empty() {
        messages.push(format!("{preset} {question}"));
    } else {
        messages.push(preset_prompt);
        messages.push(question.to_owned());
    }

    #[cfg(debug_assertions)]
    println!("\n\nmessages: {:?}\n\n", messages);

    if settings.provider.is_none() {
        eprintln!("No provider configured - use `ask use` to set one");
        return;
    }

    match settings.provider.unwrap() {
        AIProvider::DeepSeek => {
            if settings.deepseek_key.is_none() {
                eprintln!(
                    "No DeekSeek API key configured - set environment variable `ASK_DEEPSEEK_KEY`"
                );
                return;
            }
            let reply = ai::deepseek(&messages, &settings.deepseek_key.clone().unwrap());
            termimad::print_text(reply.as_str());
        }
        AIProvider::Grok3 => {
            if settings.grok3_key.is_none() {
                eprintln!("No Grok3 API key configured - set environment variable `ASK_GROK3_KEY`");
                return;
            }
            todo!()
        }
    }
}

fn handle_config_args(args: &ConfigArgs, settings: &mut Settings) {
    match &args.command {
        ConfigCommand::List => {
            let presets = match &settings.presets {
                Some(presets) => presets,
                None => &HashMap::new(),
            };

            if presets.is_empty() {
                println!("No presets found");
                return;
            }

            for (name, prompt) in presets {
                println!("{} => {}", name, prompt);
            }
        }
        ConfigCommand::Set(set_args) => {
            if settings.presets.is_none() {
                settings.presets = Some(HashMap::new());
            }
            if let Some(presets) = &mut settings.presets {
                presets.insert(set_args.name.clone(), set_args.prompt.join(" "));
            }
        }
        ConfigCommand::Get(get_args) => {
            match &mut settings.presets {
                Some(presets) => match presets.get(&get_args.name) {
                    Some(prompt) => {
                        println!("{}", prompt);
                    }
                    None => {
                        println!("No preset found for {}", get_args.name);
                    }
                },
                None => {
                    println!("No presets found");
                }
            };
        }
        ConfigCommand::Delete(delete_args) => {
            match &mut settings.presets {
                Some(presets) => match presets.get(&delete_args.name) {
                    Some(prompt) => {
                        let prompt = prompt.to_owned();
                        presets.remove(&delete_args.name);
                        println!("Deleted preset {} => {}", delete_args.name, prompt);
                    }
                    None => {
                        println!("No preset found for {}", delete_args.name);
                    }
                },
                None => {
                    println!("No presets found");
                }
            };
        }
        ConfigCommand::Use(use_args) => {
            settings.provider = Some(use_args.provider);
            println!("Using provider: {:?}", use_args.provider);
        }
    }
}

fn print_help() {
    let text = r#"

Usage: myapp [COMMAND] [OPTIONS]

"#;
    termimad::print_text(text.trim());
}

mod command;
mod settings;

use crate::command::{AIProvider, ConfigArgs, ConfigCommand};
use crate::settings::Settings;
use clap::CommandFactory;
use clap::Parser;
use config::{Config, ConfigError};
use serde::Serialize;
use std::io::Error;
use std::process::exit;

fn main() {
    let mut settings = Settings::load().unwrap_or_else(|err| {
        eprintln!("Error loading configuration: {}", err);
        exit(1);
    });

    let config_args = ConfigArgs::try_parse();
    if let Ok(args) = config_args {
        handle_config_args(&args, &mut settings);
        settings.save().unwrap();
        return;
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        print_help();
        exit(2);
    }

    let (first, remain) = match args.len() {
        1 => (args[0].clone(), "".to_owned()),
        _ => (args[0].clone(), args[1..].join(" ")),
    };

    let subcommands = vec!["use", "set", "get", "delete", "list"];
    if subcommands.contains(&first.as_str()) {
        // this definitely will fail, but we just want it to print the help message
        ConfigArgs::parse();
    }

    let want_help = args.iter().any(|arg| arg == "--help" || arg == "-h");
    if want_help {
        print_help();
        return;
    }

    settings.save().unwrap();
}

fn handle_config_args(args: &ConfigArgs, settings: &mut Settings) {
    match &args.command {
        ConfigCommand::List => {
            println!("Listing all presets...");
            // Logic to list presets
        }
        ConfigCommand::Set(set_args) => {
            println!(
                "Setting preset: {} with prompt: {}",
                set_args.name,
                set_args.prompt_string()
            );
            // Logic to set a preset
        }
        ConfigCommand::Get(get_args) => {
            println!("Getting preset: {}", get_args.name);
            // Logic to get a preset
        }
        ConfigCommand::Delete(delete_args) => {
            println!("Deleting preset: {}", delete_args.name);
            // Logic to delete a preset
        }
        ConfigCommand::Use(use_args) => {
            println!("Using AI provider: {:?}", use_args.provider);
            // Logic to set the AI provider
        }
    }
}

fn print_help() {
    println!(
        r#"
Usage: myapp [COMMAND] [OPTIONS]
"#
    )
}

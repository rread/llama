mod types;
Refmod errors;


use crate::errors::LlamaError;
use crate::types::ServiceConfig;
use clap::{Parser, Subcommand};
use ini::Ini;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;

// Clap derived cli processor
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config: Option<String>,
    #[arg(short, long)]
    system: Option<String>,
    #[arg(long)]
    service: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Ask {
        #[arg(short, long)]
        query: String,
    }
}

fn load_service_config(file: String, service: Option<String>) -> Result<ServiceConfig, LlamaError> {
    let mut service_config = ServiceConfig::default();
    match Ini::load_from_file(&file) {
        Ok(config) => {
            if let Some(key) = config.get_from(Some(service.clone().unwrap_or("openai".to_string())), "api_key") {
                service_config.api_key = key.to_string();
            }
            if let Some(url) = config.get_from(Some(service.clone().unwrap_or("openai".to_string())), "chat_url") {
                service_config.chat_url = url.to_string();
            }
        }
        Err(e) => return Err(e.into()),
    }

    Ok(service_config)
}


fn get_api_key(user_config: Option<String>, service: Option<String>) -> Option<ServiceConfig> {
    // List of potential config files to try
    let mut file_list: Vec<String> = vec![];


    // First check user supplied path, if any.
    if let Some(config) = user_config {
        file_list.push(config)
    }

    // default path is ~/.config/openai.ini
    match home::home_dir() {
        Some(path) => {
            if !path.as_os_str().is_empty() {
                file_list.push(path.join(".config").join("openai.ini").as_os_str().to_string_lossy().to_string());
            }
        }
        None => {}
    }

    for file in file_list.iter() {
        match load_service_config(file.clone(), service.clone()) {
            Ok(conf) => {
                return Some(conf);
            }
            Err(_) => {}
        }
    }

    // Last chance
    match env::var("OPENAI_API_KEY") {
        Ok(api_key) => {
            Some(ServiceConfig {
                api_key,
                chat_url: "https://api.openai.com/v1/chat/completions".to_string(),
            })
        }
        Err(_) => { None }
    }
}

#[tokio::main]
async fn main() -> Result<(), LlamaError> {
    let cli = Cli::parse();

    // Load OpenAI API key from environment variable
    let service_config = match get_api_key(cli.config, cli.service) {
        Some(key) => key,
        None => {
            println!("Unable to find api key.");
            std::process::exit(1);
        }
    };

    let system_string = cli.system.unwrap_or("You are friendly assistant".to_string());

    let mut rl = DefaultEditor::new()?;
    if rl.load_history("history.txt").is_err() {
        println!("No previous history");
    }

    let mut config = types::ChatConfig::new();
    config.model = "gpt-4o-mini".to_owned();
    config.temperature = Some(0.5);

    let mut chat = types::Chat::new(service_config, &system_string, config);

    loop {
        let readline = rl.readline(">> ");

        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(line.as_str());
                // add_message(&mut message_history, &line);
                match chat.chat_with_gpt(&line).await {
                    Ok(choices) => {
                        for choice in choices.iter() {
                            println!("GPT> {}", choice.message.content);
                        }
                    }
                    Err(e) => {
                        println!("Failed: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Interrupted");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("EOF");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    let _ = rl.save_history("history.txt");
    Ok(())
}



mod types;
mod errors;
mod service_config;
mod chat;

use crate::errors::LlamaError;
use clap::{Parser, Subcommand};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
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


#[tokio::main]
async fn main() -> Result<(), LlamaError> {
    let cli = Cli::parse();

    // Load OpenAI API key from environment variable
    let service_config = match service_config::find_service_config(cli.config, cli.service) {
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

    let mut config = chat::ChatConfig::new();
    config.model = "gpt-4o-mini".to_owned();
    config.temperature = Some(0.5);

    let mut chat = chat::Chat::new(service_config, &system_string, config);

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



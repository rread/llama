mod types;

use clap::{Parser, Subcommand};
use ini::Ini;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::error;
use std::{env, fmt};
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone)]
struct HttpError;

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http error")
    }
}

impl error::Error for HttpError {}


// Clap derived cli processor
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config: Option<String>,
    #[arg(short, long)]
    system: Option<String>,
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

fn get_api_key(user_config: Option<String>) -> Option<String> {
    // List of potential config files to try
    let mut file_list: Vec<String> = vec![];


    // First check user supplied path, if any.
    if let Some(config) = user_config {
        file_list.push(config)
    }

    // default path is ~/.config/openai.ini
    match home::home_dir() {
        Some(mut path) => {
            if !path.as_os_str().is_empty() {
                file_list.push(path.join(".config").join("openai.ini").as_os_str().to_string_lossy().to_string());
            }
        }
        None => {}
    }

    for file in file_list.iter() {
        match Ini::load_from_file(&file) {
            Ok(conf) => {
                if let Some(key) = conf.get_from(Some("openai"), "api_key") {
                    return Some(key.to_string());
                }
            }
            Err(_) => {}
        }
    }

    // Last chance
    match env::var("OPENAI_API_KEY") {
        Ok(key) => { Some(key) }
        Err(_) => { None }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load OpenAI API key from environment variable
    let api_key = match get_api_key(cli.config) {
        Some(key) => key,
        None => {
            println!("Unable to find key");
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

    let mut chat = types::Chat::new(api_key, &system_string, config);

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



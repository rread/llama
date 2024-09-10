use crate::errors::OpaiError;
use ini::Ini;
use std::env;
use serde::Deserialize;

#[derive(Deserialize, Clone, Default)]
pub struct ServiceConfig {
    pub api_key: String,
    pub chat_url: String,
}

fn load_service_config(file: String, service: Option<String>) -> Result<ServiceConfig, OpaiError> {
    let mut api_key = "".to_string();
    let mut chat_url= "".to_string();
    match Ini::load_from_file(&file) {
        Ok(config) => {
            if let Some(key) = config.get_from(Some(service.clone().unwrap_or("openai".to_string())), "api_key") {
                api_key = key.to_string();
            }
            if let Some(url) = config.get_from(Some(service.clone().unwrap_or("openai".to_string())), "chat_url") {
                chat_url = url.to_string();
            }
        }
        Err(e) => return Err(e.into()),
    }

    Ok(ServiceConfig { api_key, chat_url })
}

pub fn find_service_config(user_config: Option<String>, service: Option<String>) -> Option<ServiceConfig> {
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
    
    // Also check current directory
    match env::current_dir() {
        Ok(path) => {
            file_list.push(path.join("openai.ini").as_os_str().to_string_lossy().to_string());
        },
        Err(_) => {}
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
use crate::errors::OpaiError;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use crate::service_config::ServiceConfig;

#[serde_with::skip_serializing_none]
#[derive(Serialize, Default)]
pub struct RequestBody {
    model: String,
    messages: Vec<Message>,
    frequency_penalty: Option<f64>,
    logprobs: Option<bool>,
    top_logprobs: Option<u32>,
    max_tokens: Option<u32>,
    n: Option<u32>,
    presence_penalty: Option<f64>,
    // response_format: Option<ResponseFormat>,
    seed: Option<i32>,
    stop: Option<String>,
    // stream_options: Option<StreamOptions>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    // tools: Option<Vec<Tool>>,
    // tool_choice: Option<ToolChoice>,
    parallel_tool_calls: Option<bool>,
    user: Option<String>,
}


impl RequestBody {
    pub fn new(config: &ChatConfig, message: &Vec<Message>) -> Self {
        Self {
            model: config.model.clone(),
            messages: message.clone(),
            frequency_penalty: config.frequency_penalty,
            logprobs: config.logprobs,
            top_logprobs: config.top_logprobs,
            max_tokens: config.max_tokens,
            n: config.n,
            presence_penalty: config.presence_penalty,
            seed: config.seed,
            stop: config.stop.clone(),
            temperature: config.temperature,
            top_p: config.top_p,
            parallel_tool_calls: config.parallel_tool_calls,
            user: config.user.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ResponseChoice {
    pub message: Message,
}

#[serde_with::skip_serializing_none]
#[derive(Deserialize)]
pub struct ResponseBody {
    pub choices: Vec<ResponseChoice>,
    pub created: u64,
    pub model: String,
    pub usage: Usage,
}

#[derive(Deserialize, Clone, Default)]
pub struct Usage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

impl Display for Role {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::System => write!(fmt, "system"),
            Self::User => write!(fmt, "user"),
            Self::Assistant => write!(fmt, "assistant"),
        }
    }
}


#[derive(Clone, Default)]
pub struct ChatConfig {
    pub model: String,
    pub frequency_penalty: Option<f64>,
    pub logprobs: Option<bool>,
    pub top_logprobs: Option<u32>,
    pub max_tokens: Option<u32>,
    n: Option<u32>,
    pub presence_penalty: Option<f64>,
    // response_format: Option<ResponseFormat>,
    pub seed: Option<i32>,
    pub stop: Option<String>,
    // stream_options: Option<StreamOptions>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    // tools: Option<Vec<Tool>>,
    // tool_choice: Option<ToolChoice>,
    pub parallel_tool_calls: Option<bool>,
    user: Option<String>,
}

impl ChatConfig {
    pub fn new() -> ChatConfig {
        Self {
            ..Default::default()
        }
    }
}

impl Display for ChatConfig {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(fmt, "ChatConfig")?;
        writeln!(fmt, "model: {}", self.model)?;
        if let Some(logprobs) = self.logprobs {
            writeln!(fmt, "logprobs:  {}", logprobs)?;
        }
        if let Some(top_logprobs) = &self.top_logprobs {
            writeln!(fmt, "top_logprobs:  {}", top_logprobs)?;
        }
        if let Some(max_tokens) = &self.max_tokens {
            writeln!(fmt, "max_tokens:  {}", max_tokens)?;
        }
        if let Some(n) = &self.n {
            writeln!(fmt, "n:  {}", n)?;
        }
        if let Some(presence_penalty) = &self.presence_penalty {
            writeln!(fmt, "presence_penalty: {}", presence_penalty)?;
        }
        if let Some(seed) = &self.seed {
            writeln!(fmt, "seed:  {}", seed)?;
        }
        if let Some(stop) = &self.stop {
            writeln!(fmt, "stop: {}", stop)?;
        }
        if let Some(temperature) = &self.temperature {
            writeln!(fmt, "temperature: {}", temperature)?;
        }
        if let Some(top_p) = &self.top_p {
            writeln!(fmt, "top_p: {}", top_p)?;
        }
        if let Some(parallel_tool_calls) = &self.parallel_tool_calls {
            writeln!(fmt, "parallel_tool_calls: {}", parallel_tool_calls)?;
        }
        Ok(())
    }
}

#[derive(Clone, Default)]
pub struct Chat {
    service_config: ServiceConfig,
    client: reqwest::Client,
    // system: String,
    message_history: Vec<Message>,
    total_usage: Usage,
    config: ChatConfig,
}

impl Chat {
    pub fn new(service_config: ServiceConfig, system: &str, config: ChatConfig) -> Chat {
        Self {
            service_config,
            client: reqwest::Client::new(),
            // system: system.to_string(),
            message_history: vec![
                Message {
                    role: Role::System,
                    content: system.to_string(),
                }
            ],
            config,
            ..Default::default()
        }
    }
    pub async fn chat_with_gpt(&mut self, line: &str) -> Result<Vec<ResponseChoice>, OpaiError> {
        self.add_message(Role::User, line);
        let request_body = RequestBody::new(&self.config, &self.message_history);
        // let url = "https://api.perplexity.ai/chat/completions";
        // let url = "https://api.openai.com/v1/chat/completions";
        // println!("{}", serde_json::to_string(&request_body)?);

        let response = self.client
            .post(self.service_config.chat_url.as_str())
            .header("Authorization", format!("Bearer {}", self.service_config.api_key))
            .json(&request_body)
            .send()
            .await?;
        if response.status().is_success() {
            let response_body: ResponseBody = response.json().await?;
            for choice in response_body.choices.iter() {
                self.add_message(choice.message.role, choice.message.content.as_str())
            }
            // println!("{} {}", response_body.created, response_body.model);
            self.total_usage.completion_tokens += response_body.usage.completion_tokens;
            self.total_usage.prompt_tokens += response_body.usage.prompt_tokens;
            self.total_usage.total_tokens += response_body.usage.total_tokens;

            // println!("{}", serde_json::json!(&self.message_history));

            Ok(response_body.choices)
        } else {
            Err(OpaiError::Http(response.status()).into())
        }
    }

    fn add_message(&mut self, role: Role, message: &str) {
        let m = Message { role, content: message.to_string() };
        self.message_history.push(m);
    }
}

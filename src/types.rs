use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct RequestBody {
    pub model: String,
    pub messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct ResponseChoice {
    pub message: Message,
}

#[derive(Deserialize)]
pub struct ResponseBody {
    pub choices: Vec<ResponseChoice>,
}

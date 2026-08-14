use std::{error::Error, time::Duration};

use serde::{Deserialize, Serialize};

use crate::config;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatTemplateKwargs {
    enable_thinking: bool,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<ChatTemplateKwargs>,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

pub async fn request(cfg: config::Config, message_text: &str) -> Result<String, Box<dyn Error>> {
    let message = Message {
        role: String::from("user"),
        content: String::from(message_text),
    };
    let req = ChatRequest {
        model: cfg.model_name,
        messages: vec![message],
        chat_template_kwargs: Some(ChatTemplateKwargs {
            enable_thinking: false,
        }),
    };

    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(30))
        .build()?;

    let resp = client
        .post(cfg.llm_url)
        .bearer_auth(cfg.llm_api_key)
        .json(&req)
        .send()
        .await?;

    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        return Err(format!("unexpected response status {status}: {body}").into());
    }

    let parsed_resp: ChatResponse = serde_json::from_str(&body)?;
    let llm_response = parsed_resp
        .choices
        .first()
        .ok_or("no valid choices from LLM")?
        .message
        .content
        .clone();
    Ok(llm_response)
}

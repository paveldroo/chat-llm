use futures_util::StreamExt;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{config, error::Error, render};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
enum Role {
    System,
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    role: Role,
    pub content: String,
}

impl Message {
    #[must_use]
    pub fn user(content: &str) -> Self {
        Self {
            role: Role::User,
            content: content.to_string(),
        }
    }

    #[must_use]
    pub fn assistant(content: &str) -> Self {
        Self {
            role: Role::Assistant,
            content: content.to_string(),
        }
    }

    #[must_use]
    pub fn system(content: &str) -> Self {
        Self {
            role: Role::System,
            content: content.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct ChatTemplateKwargs {
    enable_thinking: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<Message>,
    stream: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<ChatTemplateKwargs>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<usize>,
}

pub struct Client {
    http: reqwest::Client,
    cfg: config::Config,
}

impl Client {
    pub fn new(cfg: config::Config) -> Result<Self, Error> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .read_timeout(Duration::from_secs(30))
            .build()?;

        let client = Self { http: client, cfg };
        Ok(client)
    }

    pub async fn stream_request(&self, messages: &[Message]) -> Result<String, Error> {
        let req = ChatRequest {
            model: self.cfg.model_name.clone(),
            messages: messages.to_vec(),
            stream: true,
            chat_template_kwargs: Some(ChatTemplateKwargs {
                enable_thinking: false,
            }),
            temperature: self.cfg.temperature,
            max_tokens: self.cfg.max_tokens,
        };

        let res = self
            .http
            .post(&self.cfg.llm_url)
            .bearer_auth(&self.cfg.llm_api_key)
            .json(&req)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            return Err(Error::Api {
                status,
                body: res.text().await?,
            });
        }

        let mut stream = res.bytes_stream();
        let mut buffer: Vec<u8> = vec![];
        let mut answer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            buffer.extend(chunk.as_ref());
            let text = render::stream_stdout_text(&mut buffer, false)?;
            answer.push_str(&text);
        }

        let tail = render::stream_stdout_text(&mut buffer, true)?;
        answer.push_str(&tail);

        Ok(answer)
    }
}

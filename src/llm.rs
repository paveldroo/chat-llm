use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::{config, error::Error};

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

pub struct Client {
    http: reqwest::Client,
    cfg: config::Config,
}

impl Client {
    pub fn new(cfg: config::Config) -> Result<Self, Error> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(30))
            .build()?;

        let client = Self { http: client, cfg };
        Ok(client)
    }

    pub async fn request(&self, message_text: &str) -> Result<String, Error> {
        let message = Message {
            role: String::from("user"),
            content: String::from(message_text),
        };
        let req = ChatRequest {
            model: self.cfg.model_name.clone(),
            messages: vec![message],
            chat_template_kwargs: Some(ChatTemplateKwargs {
                enable_thinking: false,
            }),
        };

        let resp = self
            .http
            .post(self.cfg.llm_url.clone())
            .bearer_auth(self.cfg.llm_api_key.clone())
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            return Err(Error::Api { status, body });
        }

        let llm_response = parse_response(&body)?;
        Ok(llm_response)
    }
}

fn parse_response(body: &str) -> Result<String, Error> {
    let parsed_resp: ChatResponse = serde_json::from_str(body)?;
    return Ok(parsed_resp
        .choices
        .first()
        .ok_or(Error::NoChoices)?
        .message
        .content
        .clone());
}

#[cfg(test)]
mod tests {
    use super::*;

    const LLM_RESPONSE: &str = r#"{"id":"60ed5df9e86f43e090410461ae2f790b","object":"chat.completion","created":1786717946,"model":"qwen35-397b-a17b-fp8","choices":[{"index":0,"message":{"role":"assistant","content":"Paris","reasoning_content":null,"tool_calls":null},"logprobs":null,"finish_reason":"stop","matched_stop":248046}],"usage":{"prompt_tokens":25,"total_tokens":27,"completion_tokens":2,"prompt_tokens_details":null,"reasoning_tokens":0},"metadata":{"weight_version":"default"}}"#;

    #[test]
    fn extract_first_choice() {
        assert!(matches!(
            parse_response(LLM_RESPONSE).as_deref(),
            Ok("Paris")
        ));
    }

    #[test]
    fn empty_choices_is_error() {
        assert!(matches!(
            parse_response(r#"{"choices":[]}"#),
            Err(Error::NoChoices)
        ));
    }

    #[test]
    fn malformed_json_is_decode_error() {
        assert!(matches!(
            parse_response(r#"{"choices":[]}}}}"#),
            Err(Error::Decode(_))
        ));
    }
}

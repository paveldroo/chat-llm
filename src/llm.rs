use futures_util::StreamExt;
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
    stream: bool,
}

#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug)]
struct Delta {
    content: Option<String>,
}

#[derive(Deserialize, Debug)]
struct StreamChoice {
    delta: Delta,
}

#[derive(Deserialize, Debug)]
struct ChatStreamChunk {
    choices: Vec<StreamChoice>,
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
            stream: false,
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

    pub async fn stream_request(&self, message_text: &str) -> Result<String, Error> {
        let message = Message {
            role: String::from("user"),
            content: String::from(message_text),
        };
        let req = ChatRequest {
            model: self.cfg.model_name.clone(),
            messages: vec![message],
            chat_template_kwargs: Some(ChatTemplateKwargs {
                enable_thinking: true,
            }),
            stream: true,
        };

        let res = self
            .http
            .post(self.cfg.llm_url.clone())
            .bearer_auth(self.cfg.llm_api_key.clone())
            .json(&req)
            .send()
            .await?;

        let status = res.status();
        if !status.is_success() {
            return Err(Error::Api {
                status,
                body: res.text().await?,
            });
        };

        let mut stream = res.bytes_stream();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            println!("received chunk: {:?}", chunk);
            let llm_response = parse_stream_response(chunk.as_ref())?;
            println!("received response: {}", llm_response)
        }

        Ok(String::from("123"))
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


/*
1. chunk ≠ SSE frame. bytes_stream() yields TCP chunks. One chunk can hold several
data: lines, or half of one. You need a byte buffer, drain up to each \n, parse
complete lines only.
2. Blank lines. Frames are separated by \n\n — empty lines must be skipped, not
parsed.
3. [DONE]. Final sentinel, not JSON.
4. .unwrap() on content — panics. content is null on reasoning deltas and on the
final finish_reason chunk.
5. NoChoices on choices: [] — normal for the trailing usage chunk, shouldn't be an
error mid-stream.
6. chunk_result.unwrap() — panics on a mid-stream transport error; use ?.
7. .timeout(Duration::from_secs(30)) applies to the whole body for streams, so any
answer longer than 30 s aborts. Use .read_timeout(...) instead (per-read idle
timeout).
8. Ok(String::from("123")) — accumulate deltas and return them.
*/

fn parse_stream_response(body: &[u8]) -> Result<String, Error> {
    let parsed_resp: ChatStreamChunk = serde_json::from_slice(body)?;
    return Ok(parsed_resp
        .choices
        .first()
        .ok_or(Error::NoChoices)?
        .delta
        .content
        .clone()
        .unwrap());
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

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

    pub async fn stream_request(&mut self, message_text: &str) -> Result<String, Error> {
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
        }

        let mut stream = res.bytes_stream();

        let mut buffer: Vec<u8> = vec![];

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            buffer.extend(chunk.as_ref());
            print!("{}", render(lines_from_chunk(&mut buffer))?);
        }

        print!("{}", render(lines_at_eof(&mut buffer))?);

        Ok(String::new())
    }
}

fn lines_from_chunk(buffer: &mut Vec<u8>) -> Vec<String> {
    let Some(end) = buffer.iter().rposition(|&b| b == b'\n') else {
        return Vec::new();
    };

    take_lines(buffer, end + 1)
}

fn lines_at_eof(buffer: &mut Vec<u8>) -> Vec<String> {
    let end = buffer.len();
    take_lines(buffer, end)
}

fn take_lines(buffer: &mut Vec<u8>, end: usize) -> Vec<String> {
    let complete: Vec<u8> = buffer.drain(..end).collect();
    complete
        .split(|&b| b == b'\n')
        .map(|line| String::from_utf8_lossy(line).trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

fn render(lines: Vec<String>) -> Result<String, Error> {
    let mut text = String::new();
    for line in lines {
        let (_, json_part) = line.as_str().split_at(6);
        text.push_str(&parse_stream_response(json_part.trim())?);
    }
    Ok(text)
}

fn parse_stream_response(body: &str) -> Result<String, Error> {
    if body == "[DONE]" {
        return Ok(String::new());
    }
    let chunk: ChatStreamChunk = serde_json::from_str(body)?;
    if chunk.choices.is_empty() {
        return Ok(String::new());
    }
    Ok(chunk
        .choices
        .first()
        .unwrap()
        .delta
        .content
        .clone()
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LLM_RESPONSE: &str = r#"{"id":"60ed5df9e86f43e090410461ae2f790b","object":"chat.completion","created":1786717946,"model":"qwen35-397b-a17b-fp8","choices":[{"index":0,"message":{"role":"assistant","content":"Paris","reasoning_content":null,"tool_calls":null},"logprobs":null,"finish_reason":"stop","matched_stop":248046}],"usage":{"prompt_tokens":25,"total_tokens":27,"completion_tokens":2,"prompt_tokens_details":null,"reasoning_tokens":0},"metadata":{"weight_version":"default"}}"#;

    #[test]
    fn extract_first_choice() {
        assert!(matches!(
            parse_stream_response(LLM_RESPONSE).as_deref(),
            Ok("Paris")
        ));
    }

    #[test]
    fn malformed_json_is_decode_error() {
        assert!(matches!(
            parse_stream_response(r#"{"choices":[]}}}}"#),
            Err(Error::Decode(_))
        ));
    }

    #[test]
    fn tail_without_newline_is_flushed_at_eof() {
        let mut buffer = b"data: {\"a\":1}".to_vec();
        assert!(lines_from_chunk(&mut buffer).is_empty());
        assert_eq!(lines_at_eof(&mut buffer), ["data: {\"a\":1}"]);
        assert!(buffer.is_empty());
    }

    #[test]
    fn line_split_across_chunks_is_held_then_emitted() {
        let mut buffer = b"data: {\"a".to_vec();
        assert!(lines_from_chunk(&mut buffer).is_empty());
        buffer.extend_from_slice(b"\":1}\n");
        assert_eq!(lines_from_chunk(&mut buffer), ["data: {\"a\":1}"]);
    }
}

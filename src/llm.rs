use futures_util::StreamExt;
use std::{io::Write, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{config, error::Error};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    #[must_use]
    pub fn user(content: &str) -> Self {
        Self {
            role: "user".to_string(),
            content: content.to_string(),
        }
    }

    #[must_use]
    pub fn assistant(context: &str) -> Self {
        Self {
            role: "assistant".to_string(),
            content: context.to_string(),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    chat_template_kwargs: Option<ChatTemplateKwargs>,
    stream: bool,
    pub instructions: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
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

    pub async fn stream_request(&self, messages: &[Message]) -> Result<String, Error> {
        let req = ChatRequest {
            model: self.cfg.model_name.clone(),
            messages: messages.to_vec(),
            chat_template_kwargs: Some(ChatTemplateKwargs {
                enable_thinking: false,
            }),
            stream: true,
            instructions: self.cfg.system.clone(),
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
            let text = render(lines_from_chunk(&mut buffer))?;
            {
                let mut out = std::io::stdout().lock();
                write!(out, "{text}")?;
                out.flush()?;
            }
            answer.push_str(&text);
        }

        let tail = render(lines_at_eof(&mut buffer))?;
        {
            let mut out = std::io::stdout().lock();
            write!(out, "{tail}")?;
            out.flush()?;
        }
        answer.push_str(&tail);

        Ok(answer)
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
        let Some(json_part) = line.as_str().strip_prefix("data:") else {
            continue;
        };
        text.push_str(&parse_stream_response(json_part.trim())?);
    }
    Ok(text)
}

fn parse_stream_response(body: &str) -> Result<String, Error> {
    if body == "[DONE]" || body.is_empty() {
        return Ok(String::new());
    }
    let chunk: ChatStreamChunk = serde_json::from_str(body)?;
    Ok(chunk
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.delta.content)
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DELTA_CHUNK: &str = r#"{"id":"c0","object":"chat.completion.chunk","choices":[{"index":0,"delta":{"content":"Paris"},"finish_reason":null}]}"#;

    #[test]
    fn extract_delta_content() {
        assert!(matches!(
            parse_stream_response(DELTA_CHUNK).as_deref(),
            Ok("Paris")
        ));
    }

    #[test]
    fn comment_lines_are_skipped() {
        assert!(matches!(
            render(vec![":".to_string(), ": ping".to_string()]).as_deref(),
            Ok("")
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

use serde::Deserialize;
use std::io::Write;

use crate::error::Error;

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

pub fn stream_stdout_text(buffer: &mut Vec<u8>, eof: bool) -> Result<String, Error> {
    let text = if eof {
        render(lines_at_eof(buffer))?
    } else {
        render(lines_from_chunk(buffer))?
    };

    {
        let mut out = std::io::stdout().lock();
        write!(out, "{text}")?;
        out.flush()?;
    }
    Ok(text)
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

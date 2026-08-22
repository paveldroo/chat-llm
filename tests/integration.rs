use std::error::Error;

use assert_cmd::{Command, cargo::cargo_bin_cmd};
use chat_llm::llm::{self, ChatRequest};
use predicates::prelude::{Predicate, predicate};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

// const SSE_RESPONSE: &str = include_str!("fixtures/paris.sse");

fn set_all_envs(cmd: &mut Command) {
    cmd.env("LLM_API_KEY", "test");
    cmd.env("LLM_URL", "test");
    cmd.env("MODEL_NAME", "test");
}

#[test]
fn corrupted_envs() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    cmd.assert()
        .failure()
        .stdout("")
        .stderr(predicates::str::contains("llm_api_key"));
}

#[test]
fn empty_envs() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    cmd.env("LLM_API_KEY", "");
    cmd.env("LLM_URL", "");
    cmd.env("MODEL_NAME", "");
    cmd.assert()
        .failure()
        .stdout("")
        .stderr("chat-llm: empty MODEL_NAME env variable\n");
}

#[test]
fn no_argument() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    set_all_envs(&mut cmd);
    cmd.write_stdin("exit\n");
    cmd.assert().success().stdout("\n> ").stderr("");
}

#[tokio::test(flavor = "multi_thread")]
async fn repl_prompt_success() -> Result<(), Box<dyn Error>> {
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200).set_body_raw("", "text/event-stream");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    set_all_envs(&mut cmd);
    cmd.env("LLM_URL", mock_server.uri());

    cmd.write_stdin("first input\nsecond input\n")
        .assert()
        .success()
        .stderr("");

    let requests = mock_server
        .received_requests()
        .await
        .ok_or("mock server received no requests")?;

    assert!(!requests.is_empty());

    let second_request = requests
        .get(1)
        .ok_or("no second request was received by mock server")?
        .body_json::<llm::ChatRequest>()?;

    let message = &second_request
        .messages
        .first()
        .ok_or("no messages in second request")?;

    assert!(
        predicate::str::contains("first input").eval(&message.content),
        "second request didn't contain first input"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn all_params_are_filled() -> Result<(), Box<dyn Error>> {
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200).set_body_raw("", "text/event-stream");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let system = "speak in French";
    let model = "some_model_here";
    let temperature = 0.68;
    let max_tokens = 123;

    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    set_all_envs(&mut cmd);
    cmd.args(&[
        format!("--system={system}"),
        format!("--model={model}"),
        format!("--temperature={temperature}"),
        format!("--max-tokens={max_tokens}"),
    ]);
    cmd.env("LLM_URL", mock_server.uri());

    cmd.write_stdin("some_input\n")
        .assert()
        .success()
        .stderr("");

    let requests = mock_server
        .received_requests()
        .await
        .ok_or("mock server received no requests")?;

    assert!(!requests.is_empty());

    let user_request = requests
        .first()
        .ok_or("no request was received by mock server")?
        .body_json::<ChatRequest>()?;

    assert_eq!(user_request.model, model);
    assert_eq!(user_request.temperature, Some(temperature));
    assert_eq!(user_request.max_tokens.unwrap_or_default(), max_tokens);

    let first_message = user_request
        .messages
        .first()
        .ok_or("no messages in user request")?;

    assert!(predicates::str::contains(system).eval(&first_message.content));

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn linux_pipe() -> Result<(), Box<dyn Error>> {
    let mock_server = MockServer::start().await;

    let response_data = std::fs::read("tests/fixtures/paris_without_budget.sse")?;
    let mock_response = ResponseTemplate::new(200)
        .set_body_raw(String::from_utf8(response_data)?, "text/event-stream");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    set_all_envs(&mut cmd);
    cmd.env("TEST_PIPE", "true");
    cmd.env("LLM_URL", mock_server.uri());

    let pipeout_content = std::fs::read("tests/fixtures/prompt.txt")?;
    cmd.write_stdin(pipeout_content.clone()).unwrap();

    let requests = mock_server
        .received_requests()
        .await
        .ok_or("mock server received no requests")?;

    assert!(!requests.is_empty());

    let user_request = requests
        .first()
        .ok_or("no request was received by mock server")?
        .body_json::<ChatRequest>()?;

    let first_message = user_request
        .messages
        .first()
        .ok_or("no messages in user request")?;

    assert!(
        predicates::str::contains(String::from_utf8(pipeout_content)?).eval(&first_message.content)
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn stream_request_with_retry() -> Result<(), Box<dyn Error>> {
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(429).set_body_raw("", "text/event-stream");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    set_all_envs(&mut cmd);
    cmd.env("LLM_URL", mock_server.uri());

    cmd.write_stdin("some_input\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("429"));

    let requests = mock_server
        .received_requests()
        .await
        .ok_or("mock server received no requests")?;

    assert!(!requests.is_empty());
    assert_eq!(requests.len(), 3);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn cfg_with_budget() -> Result<(), Box<dyn Error>> {
    let mock_server = MockServer::start().await;

    let response_data = std::fs::read("tests/fixtures/paris_with_budget.sse")?;
    let mock_response = ResponseTemplate::new(200)
        .set_body_raw(String::from_utf8(response_data)?, "text/event-stream");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    set_all_envs(&mut cmd);
    cmd.env("LLM_URL", mock_server.uri());
    cmd.arg("--budget=20");
    cmd.write_stdin("hello\n");
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains("Paris"))
        .stderr(predicates::str::contains("budget exceeded"));

    Ok(())
}

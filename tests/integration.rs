use std::error::Error;

use assert_cmd::{Command, cargo::cargo_bin_cmd};
use assert_float_eq::assert_float_relative_eq;
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
    cmd.assert().success().stdout("> ").stderr("");
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
        .ok_or("no second request was received by mock server")?
        .body_json::<ChatRequest>()?;

    assert_eq!(
        user_request.instructions.clone().unwrap_or_default(),
        system
    );
    assert_eq!(user_request.model, model);
    assert_float_relative_eq!(user_request.temperature.unwrap_or_default(), temperature);
    assert_eq!(user_request.max_tokens.unwrap_or_default(), max_tokens);

    Ok(())
}

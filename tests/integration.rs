use std::error::Error;

use assert_cmd::{Command, cargo::cargo_bin_cmd};
use chat_llm::llm;
use predicates::prelude::{Predicate, predicate};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const SSE_RESPONSE: &str = include_str!("fixtures/paris.sse");

fn set_all_envs(cmd: &mut Command) {
    cmd.env("LLM_API_KEY", "test");
    cmd.env("LLM_URL", "test");
    cmd.env("MODEL_NAME", "test");
}

#[test]
fn corrupted_envs() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    cmd.arg("test arg");
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
    cmd.arg("test arg");
    cmd.assert()
        .failure()
        .stdout("")
        .stderr("chat-llm: empty LLM_API_KEY env variable\n");
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
async fn argument_prompt_success() {
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200).set_body_raw(SSE_RESPONSE, "text/event-stream");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    set_all_envs(&mut cmd);
    cmd.env("LLM_URL", mock_server.uri());
    cmd.arg("what is the capital of France in one word?");
    cmd.assert().success().stdout("Paris\n").stderr("");
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

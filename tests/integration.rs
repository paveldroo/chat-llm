use assert_cmd::{Command, cargo::cargo_bin_cmd};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const SSE_RESPONSE: &str = r#"data:
{"id":"c0","object":"chat.completion.chunk","model":"qwen35-397b-a17b-fp8","choi
ces":[{"index":0,"delta":{"role":"assistant"},"finish_reason":null}]}

: ping

data: {"id":"c0","object":"chat.completion.chunk","model":"qwen35-397b-a17b-fp8"
,"choices":[{"index":0,"delta":{"content":"Pa"},"finish_reason":null}]}

data: {"id":"c0","object":"chat.completion.chunk","model":"qwen35-397b-a17b-fp8"
,"choices":[{"index":0,"delta":{"content":"ris"},"finish_reason":null}]}

data: {"id":"c0","object":"chat.completion.chunk","model":"qwen35-397b-a17b-fp8"
,"choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]"#;

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
    cmd.assert()
        .failure()
        .stdout("")
        .stderr("chat-llm: no prompt specified\n");
}

#[test]
fn empty_text() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    set_all_envs(&mut cmd);
    cmd.arg("");
    cmd.assert()
        .failure()
        .stdout("")
        .stderr("chat-llm: no prompt specified\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn success() {
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

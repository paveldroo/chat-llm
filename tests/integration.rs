use assert_cmd::{Command, cargo::cargo_bin_cmd};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

const LLM_RESPONSE: &str = r#"{"id":"60ed5df9e86f43e090410461ae2f790b","object":"chat.completion","created":1786717946,"model":"qwen35-397b-a17b-fp8","choices":[{"index":0,"message":{"role":"assistant","content":"Paris","reasoning_content":null,"tool_calls":null},"logprobs":null,"finish_reason":"stop","matched_stop":248046}],"usage":{"prompt_tokens":25,"total_tokens":27,"completion_tokens":2,"prompt_tokens_details":null,"reasoning_tokens":0},"metadata":{"weight_version":"default"}}"#;

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
        .stderr("chat-llm: no text specified\n");
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
        .stderr("chat-llm: no text specified\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn success() {
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200).set_body_raw(LLM_RESPONSE, "application/json");

    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env("LLM_URL", mock_server.uri());

    cmd.arg("what is the capital of France in one word?");
    cmd.assert().success().stdout("Paris\n").stderr("");
}

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::{boolean::PredicateBooleanExt, prelude::predicate};

#[test]
fn no_api_key_env() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    cmd.assert().failure().stderr("chat-llm: LLM_API_KEY is not set (see .env.example)\n");
}

#[test]
fn api_key_exists() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env("LLM_API_KEY", "test");
    cmd.assert().stderr(predicate::eq("chat-llm: LLM_API_KEY is not set (see .env.example)\n").not());
}

#[test]
fn no_text() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env("LLM_API_KEY", "test");
    cmd.assert().failure().stderr("chat-llm: no text specified\n");
}

#[test]
fn success() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env("LLM_API_KEY", "test");
    cmd.arg("some text here");
    cmd.assert().success();
}

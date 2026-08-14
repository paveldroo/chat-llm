use assert_cmd::{Command, cargo::cargo_bin_cmd};

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
        .stderr("chat-llm: missing value for field llm_api_key\n");
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

#[test]
fn success() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.arg("what is the capital of France in one word?");
    cmd.assert().success().stdout("Paris\n").stderr("");
}

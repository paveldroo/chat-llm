use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn no_api_key_env() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    cmd.assert()
        .failure()
        .stdout("")
        .stderr("chat-llm: LLM_API_KEY is not set (see .env.example)\n");
}

#[test]
fn empty_api_key_env() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    cmd.env("LLM_API_KEY", "");
    cmd.assert()
        .failure()
        .stdout("")
        .stderr("chat-llm: LLM_API_KEY is not set (see .env.example)\n");
}

#[test]
fn no_argument() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    cmd.env("LLM_API_KEY", "test");
    cmd.assert()
        .failure()
        .stdout("")
        .stderr("chat-llm: no text specified\n");
}

#[test]
fn empty_text() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    cmd.env("LLM_API_KEY", "test");
    cmd.arg("");
    cmd.assert()
        .failure()
        .stdout("")
        .stderr("chat-llm: no text specified\n");
}

#[test]
fn success() {
    let mut cmd = cargo_bin_cmd!("chat-llm");
    cmd.env_clear();
    cmd.env("LLM_API_KEY", "test");
    cmd.arg("some text here");
    cmd.assert().success().stdout("some text here\n").stderr("");
}

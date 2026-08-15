use chat_llm::{config, error::Error, llm};
use std::{env, process::ExitCode};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    match run().await {
        Ok(llm_response) => {
            println!("{llm_response}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("chat-llm: {err}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<String, Error> {
    let args: Vec<String> = env::args().collect();

    let cfg = config::from_env()?;

    let message = args
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or(Error::NoPrompt)?;

    let llm_client = llm::Client::new(cfg)?;

    let llm_response = llm_client.request(message).await?;

    Ok(llm_response)
}

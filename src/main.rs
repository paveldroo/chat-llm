use chat_llm::{cli, config, error::Error, llm};
use clap::Parser;
use std::process::ExitCode;

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
    let cli = cli::Cli::parse();
    let message = cli
        .prompt
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or(Error::NoPrompt)?;
    let cfg = config::from_env()?;
    let llm_client = llm::Client::new(cfg)?;
    let llm_response = llm_client.stream_request(message).await?;

    Ok(llm_response)
}

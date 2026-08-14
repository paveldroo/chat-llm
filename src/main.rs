use std::{env, error::Error, process::ExitCode};

mod config;
mod llm;

#[tokio::main]
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

async fn run() -> Result<String, Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let cfg = config::config()?;

    let message = args
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or("no text specified")?;

    let llm_response = llm::request(cfg, message).await?;

    Ok(llm_response)
}

use chat_llm::{cli, config, error::Error, llm};
use clap::Parser;
use std::{io, io::Write, process::ExitCode};

#[tokio::main(flavor = "multi_thread")]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("chat-llm: {err}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Error> {
    let cli = cli::Cli::parse();
    let message = cli.prompt.as_deref().map(str::trim).unwrap_or_default();
    let cfg = config::from_env()?;
    let mut llm_client = llm::Client::new(cfg)?;
    let mut out = std::io::stdout().lock();

    if !message.is_empty() {
        llm_client.stream_request(message).await?;
    } else {
        let mut context = String::new();
        loop {
            write!(out, "> ")?;
            out.flush()?;
            let mut user_input = String::new();
            let n = io::stdin().read_line(&mut user_input)?;
            let trimmed_input = user_input.trim();
            if trimmed_input == "exit" || n == 0 {
                break;
            }
            if trimmed_input.is_empty() {
                continue;
            }
            context.push_str("\nUSER INPUT:\n");
            context.push_str(trimmed_input);
            context.push_str("\n");
            let res = llm_client.stream_request(&context).await?;
            write!(out, "\n")?;
            out.flush()?;
            context.push_str("\nLLM_RESPONSE:\n");
            context.push_str(&res);
            context.push_str("\n");
        }
    }

    Ok(())
}

use chat_llm::{cli, config, conversation::Conversation, error::Error, llm};
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
    let cfg = config::Config::new(cli)?;
    let llm_client = llm::Client::new(cfg.clone())?;

    let mut conversation = Conversation::new();
    if cfg.system.is_some() {
        conversation.push_system(&cfg.system.unwrap_or_default());
    }
    loop {
        {
            let mut out = std::io::stdout().lock();
            write!(out, "> ")?;
            out.flush()?;
        }
        let mut user_input = String::new();
        let n = io::stdin().read_line(&mut user_input)?;
        let trimmed_input = user_input.trim();
        if trimmed_input == "exit" || n == 0 {
            break;
        }
        if trimmed_input.is_empty() {
            continue;
        }
        conversation.push_user(trimmed_input);
        let res = llm_client.stream_request(conversation.as_slice()).await?;
        {
            let mut out = std::io::stdout().lock();
            writeln!(out)?;
            out.flush()?;
        }

        conversation.push_assistant(&res);
    }

    Ok(())
}

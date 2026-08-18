use chat_llm::{
    cli, config,
    conversation::Conversation,
    error::Error,
    llm::{self, Message},
};
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
    let input = cli.prompt.as_deref().map(str::trim).unwrap_or_default();
    let cfg = config::from_env()?;
    let llm_client = llm::Client::new(cfg)?;

    if input.is_empty() {
        let mut conversation = Conversation { messages: vec![] };
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
            let user_message = Message::user(trimmed_input);
            conversation.messages.push(user_message);
            let res = llm_client.stream_request(&conversation.messages).await?;
            {
                let mut out = std::io::stdout().lock();
                writeln!(out)?;
                out.flush()?;
            }

            let llm_message = Message::assistant(&res);
            conversation.messages.push(llm_message);
        }
    } else {
        let message = Message::user(input.trim());
        llm_client.stream_request(&[message]).await?;
        {
            let mut out = std::io::stdout().lock();
            writeln!(out)?;
            out.flush()?;
        }
    }

    Ok(())
}

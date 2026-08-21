use chat_llm::{
    cli, config, conversation::Conversation, error::Error, llm, render::stream_stdout_text,
};
use clap::Parser;
use std::{
    io::{self, IsTerminal, Read, Write},
    process::ExitCode,
};

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
    let cfg = config::Config::new(cli::Cli::parse())?;

    let mut conversation = Conversation::new();
    if let Some(system) = &cfg.system {
        conversation.with_system(system)?;
    }

    let llm_client = llm::Client::new(cfg)?;

    let force_repl = std::env::var("TEST_REPL").is_ok();
    if !io::stdin().is_terminal() && !force_repl {
        return stdin_pipe_handler(llm_client, &mut conversation).await;
    }

    loop {
        {
            let mut out = std::io::stdout().lock();
            write!(out, "\n> ")?;
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
        let res = llm_client.stream_request(conversation.as_slice()).await;
        match res {
            Ok(response) => {
                conversation.push_assistant(&response);
            }
            Err(err) => {
                eprintln!("error occurred while making request to llm: {err}");
            }
        }
    }

    Ok(())
}

async fn stdin_pipe_handler(
    llm_client: llm::Client,
    conversation: &mut Conversation,
) -> Result<(), Error> {
    let mut stdin_input = String::new();
    io::stdin().lock().read_to_string(&mut stdin_input)?;
    if !stdin_input.trim().is_empty() {
        conversation.push_user(&stdin_input);
        let res = llm_client.stream_request(conversation.as_slice()).await;
        match res {
            Ok(response) => {
                stream_stdout_text(&mut response.into_bytes(), true)?;
            }
            Err(err) => {
                eprintln!("error occurred while making request to llm: {err}");
                return Err(err);
            }
        }
    }
    Ok(())
}

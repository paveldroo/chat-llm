use std::{env, error::Error, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("chat-llm: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let api_key =
        env::var("LLM_API_KEY").map_err(|_| "LLM_API_KEY is not set (see .env.example)")?;
    if api_key.is_empty() {
        return Err("LLM_API_KEY is not set (see .env.example)".into());
    }
    match args.get(1) {
        Some(text) => {
            if text.is_empty() {
                return Err("no text specified".into());
            }
            println!("{text}");
        }
        None => return Err("no text specified".into()),
    }

    Ok(())
}

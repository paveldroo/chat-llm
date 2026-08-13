use std::{env, error::Error, process::ExitCode};

mod request;

#[tokio::main]
async fn main() -> ExitCode {
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
    let _api_key = env::var("LLM_API_KEY")
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
        .ok_or("LLM_API_KEY is not set (see .env.example)")?;

    let message = args
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or("no text specified")?;
    println!("{message}");
    // println!("{api_key}");

    Ok(())
}

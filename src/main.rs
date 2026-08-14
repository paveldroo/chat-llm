use std::{env, error::Error, process::ExitCode};

mod config;

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

    let _config = config::config()?;

    let message = args
        .get(1)
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .ok_or("no text specified")?;
    println!("{message}");
    // println!("{api_key}");

    Ok(())
}

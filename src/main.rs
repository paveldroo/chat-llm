use std::{
    env::{self},
    error::Error,
    process::ExitCode,
};

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
    let Ok(_) = env::var("LLM_API_KEY") else {
        return Err("you should specify LLM_API_KEY env".into());
    };
    match args.get(1) {
        Some(text) => println!("{text}"),
        None => return Err("no text specified".into()),
    }

    Ok(())
}

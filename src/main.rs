use std::{env::{self}, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let _ = env::var("LLM_API_KEY").expect("LLM_API_KEY not found in env");
    match args.get(1) {
        Some(text) => println!("{}", text),
        None => {
            println!("chat-llm: error: no text specified");
            return ExitCode::FAILURE
        }
    };

    ExitCode::SUCCESS
}

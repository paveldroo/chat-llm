use std::{
    env::{self},
    process::ExitCode,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let Ok(_) = env::var("LLM_API_KEY") else {
        eprintln!("chat-llm: you should specify LLM_API_KEY env");
        return ExitCode::FAILURE;
    };
    match args.get(1) {
        Some(text) => println!("{}", text),
        None => {
            eprintln!("chat-llm: no text specified");
            return ExitCode::FAILURE;
        }
    }

    ExitCode::SUCCESS
}

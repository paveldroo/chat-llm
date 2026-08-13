set dotenv-load := true

export PROJECT_NAME := "chat-llm"

run:
    cargo run -- "hello"

lint:
    cargo fmt
    cargo clippy --all-targets -- -D warnings

test:
    cargo test

release:
    cargo build --bin {{PROJECT_NAME}} --release
    ./target/release/{{PROJECT_NAME}}

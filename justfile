set dotenv-load := true

export PROJECT_NAME := "chat-llm"

run:
    RUST_BACKTRACE=1 cargo run -- --system="speak in French"

run-cat:
    cat tests/fixtures/prompt.txt | cargo run

lint:
    cargo fmt
    cargo clippy --all-targets -- -D warnings

test:
    cargo test

release:
    cargo build --bin {{PROJECT_NAME}} --release
    ./target/release/{{PROJECT_NAME}}

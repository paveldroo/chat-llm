export PROJECT_NAME := "chat-llm"

run:
    cargo run

lint:
    cargo fmt
    cargo clippy -- -D warnings

test:
    cargo test

release:
    cargo build --bin {{PROJECT_NAME}} --release
    cp target/release/{{PROJECT_NAME}} .
    chmod +x ./{{PROJECT_NAME}}
    ./{{PROJECT_NAME}}

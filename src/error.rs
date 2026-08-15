use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("empty {0} env variable")]
    EmptyConfig(&'static str),

    #[error("{0}")]
    Env(#[from] envy::Error),

    #[error("no prompt specified")]
    NoPrompt,

    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),

    #[error("unexpected response status {status}: {body}")]
    Api {
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("malformed response: {0}")]
    Decode(#[from] serde_json::Error),

    #[error("no valid choices from LLM")]
    NoChoices,
}

use serde::Deserialize;

use crate::{cli, error::Error};

#[derive(Deserialize, Debug)]
pub struct Config {
    // envs
    pub llm_api_key: String,
    pub llm_url: String,
    pub model_name: String,

    // args
    pub system: Option<String>,
    pub model: Option<String>,
    pub temperature: Option<String>,
    pub max_tokens: Option<usize>,
}

impl Config {
    pub fn new(cli: cli::Cli) -> Result<Self, Error> {
        let mut c = envy::from_env::<Self>()?;

        for (env, val) in [
            ("LLM_API_KEY", &c.llm_api_key),
            ("LLM_URL", &c.llm_url),
            ("MODEL_NAME", &c.model_name),
        ] {
            if val.is_empty() {
                return Err(Error::EmptyConfig(env));
            }
        }

        c.system = cli.system;
        c.model = cli.model;
        c.temperature = cli.temperature;
        c.max_tokens = cli.max_tokens;

        Ok(c)
    }
}

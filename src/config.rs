use serde::Deserialize;

use crate::{cli, error::Error};

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub llm_api_key: String,
    pub llm_url: String,
    pub model_name: String,
    pub system: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<usize>,
}

impl Config {
    pub fn new(cli: cli::Cli) -> Result<Self, Error> {
        let mut c = envy::from_env::<Self>()?;

        let model_name = cli.model.unwrap_or_default();
        if !model_name.is_empty() {
            c.model_name = model_name;
        } else if model_name.is_empty() && c.model_name.is_empty() {
            return Err(Error::EmptyConfig("MODEL_NAME"));
        }

        c.system = cli.system;
        c.temperature = cli.temperature;
        c.max_tokens = cli.max_tokens;

        for (env, val) in [("LLM_API_KEY", &c.llm_api_key), ("LLM_URL", &c.llm_url)] {
            if val.is_empty() {
                return Err(Error::EmptyConfig(env));
            }
        }

        Ok(c)
    }
}

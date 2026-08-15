use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub llm_api_key: String,
    pub llm_url: String,
    pub model_name: String,
}

pub fn from_env() -> Result<Config, Error> {
    let config = envy::from_env::<Config>()?;

    if config.llm_api_key.is_empty() {
        return Err(Error::EmptyConfig("LLM_API_KEY"));
    }
    if config.llm_url.is_empty() {
        return Err(Error::EmptyConfig("LLM_URL"));
    }
    if config.model_name.is_empty() {
        return Err(Error::EmptyConfig("MODEL_NAME"));
    }

    Ok(config)
}

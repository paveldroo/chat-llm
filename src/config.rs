use std::error::Error;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub llm_api_key: String,
    pub llm_url: String,
    pub model_name: String,
}

pub fn config() -> Result<Config, Box<dyn Error>> {
    let config = envy::from_env::<Config>()?;

    if config.llm_api_key.is_empty() {
        return Err("empty LLM_API_KEY env variable".into());
    }
    if config.llm_url.is_empty() {
        return Err("empty LLM_URL env variable".into());
    }
    if config.model_name.is_empty() {
        return Err("empty MODEL_NAME env variable".into());
    }

    Ok(config)
}

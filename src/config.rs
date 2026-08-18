use serde::Deserialize;

use crate::error::Error;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub llm_api_key: String,
    pub llm_url: String,
    pub model_name: String,
}

pub fn from_env() -> Result<Config, Error> {
    let c = envy::from_env::<Config>()?;

    for (env, val) in [
        ("LLM_API_KEY", &c.llm_api_key),
        ("LLM_URL", &c.llm_url),
        ("MODEL_NAME", &c.model_name),
    ] {
        if val.is_empty() {
            return Err(Error::EmptyConfig(env));
        }
    }

    Ok(c)
}

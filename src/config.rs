use serde::Deserialize;
use std::{error::Error, fs};

#[derive(Deserialize, Debug, Default, Clone)]
pub struct Config {
    pub fresh_rss_api_url: Option<String>,
    pub fresh_rss_api_user: Option<String>,
    pub fresh_rss_api_password: Option<String>,
    pub browser: Option<String>,
}

impl Config {
    pub fn from(path: &String) -> Result<Config, Box<dyn Error>> {
        let contents = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;

        Ok(config)
    }
}

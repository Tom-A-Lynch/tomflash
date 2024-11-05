use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnv(String),
    
    #[error("Invalid environment variable: {0}")]
    InvalidEnv(String),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub openai_api_key: String,
    pub eth_rpc_url: String,
    pub twitter_config: TwitterConfig,
}

#[derive(Debug, Clone)]
pub struct TwitterConfig {
    pub api_key: String,
    pub api_secret: String,
    pub access_token: String,
    pub access_secret: String,
    pub bearer_token: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok();

        Ok(Self {
            database_url: get_env("DATABASE_URL")?,
            openai_api_key: get_env("OPENAI_API_KEY")?,
            eth_rpc_url: get_env("ETH_MAINNET_RPC")?,
            twitter_config: TwitterConfig {
                api_key: get_env("TWITTER_API_KEY")?,
                api_secret: get_env("TWITTER_API_SECRET")?,
                access_token: get_env("TWITTER_ACCESS_TOKEN")?,
                access_secret: get_env("TWITTER_ACCESS_SECRET")?,
                bearer_token: get_env("TWITTER_BEARER_TOKEN")?,
            },
        })
    }
}

fn get_env(key: &str) -> Result<String, ConfigError> {
    std::env::var(key).map_err(|_| ConfigError::MissingEnv(key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_from_env() {
        // Set test environment variables
        std::env::set_var("DATABASE_URL", "test_url");
        std::env::set_var("OPENAI_API_KEY", "test_key");
        std::env::set_var("ETH_MAINNET_RPC", "test_rpc");
        std::env::set_var("TWITTER_API_KEY", "test_twitter_key");
        std::env::set_var("TWITTER_API_SECRET", "test_twitter_secret");
        std::env::set_var("TWITTER_ACCESS_TOKEN", "test_access_token");
        std::env::set_var("TWITTER_ACCESS_SECRET", "test_access_secret");
        std::env::set_var("TWITTER_BEARER_TOKEN", "test_bearer_token");

        let config = Config::from_env().unwrap();
        assert_eq!(config.database_url, "test_url");
        assert_eq!(config.openai_api_key, "test_key");
        assert_eq!(config.eth_rpc_url, "test_rpc");
        assert_eq!(config.twitter_config.api_key, "test_twitter_key");
    }
}
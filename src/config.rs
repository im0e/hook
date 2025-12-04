use serde::Deserialize;
use std::collections::HashMap;
use tokio::fs;



#[derive(Deserialize, Debug, Clone)]
pub struct TlsConfig {
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub secret: String,
    pub tls: Option<TlsConfig>,
    pub git_token: Option<String>,
    pub repos: HashMap<String, RepoConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RepoConfig {
    pub path: String, 
    pub branch: String,
    pub deploy_command: Option<String>,
}

pub async fn load_config() -> Result<AppConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string("config.toml").await?;
    let config: AppConfig = toml::from_str(&content)?;
    Ok(config)
}
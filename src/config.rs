use std::error::Error;

use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    pub color: u32,
    pub refetch_delay: u64,
    #[serde(rename = "webhook")]
    pub webhooks: Vec<Webhook>,
}

#[derive(Deserialize, Clone)]
pub struct Webhook {
    pub url: String,
    #[serde(rename = "subscription")]
    pub subscriptions: Vec<Subscription>,
}

#[derive(Deserialize, Clone)]
pub struct Subscription {
    pub name: String,
    pub url: String,
    pub icon: Option<String>,
    pub color: Option<u32>,
}

pub fn read() -> Result<Config, Box<dyn Error>> {
    let file = std::fs::read_to_string("config.toml")?;
    let config = toml::from_str::<Config>(&file)?;

    Ok(config)
}

use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Clone, Debug, Serialize, Deserialize, strum_macros::Display)]
#[serde(rename_all = "lowercase")]
pub enum PriceProvider {
    CryptoWatch,
    Yahoo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    #[serde(default = "default_log_level")]
    pub level: String,
    pub file: Option<String>,
    #[serde(default = "default_enable_stdout")]
    pub enable_stdout: bool,
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_enable_stdout() -> bool {
    false
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Providers {
    pub price_provider: PriceProvider,
    pub cw_api_key: String,
    pub ip_api_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Application {
    pub log: Log,
    pub providers: Providers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Redis {
    pub hostname: String,
    pub port: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Web {
    pub hostname: String,
    pub port: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metrics {
    pub hostname: String,
    pub port: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Server {
    pub web: Web,
    pub redis: Redis,
    pub metrics: Metrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub app: Application,
    pub server: Server,
}

impl Config {
    pub fn new() -> Result<Self> {
        let s = config::Config::builder()
            .add_source(config::File::with_name("dca-pal.yml").format(config::FileFormat::Yaml))
            .build()?;

        Ok(s.try_deserialize()?)
    }
}

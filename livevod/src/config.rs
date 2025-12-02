use serde::{Deserialize, Serialize};
use std::{env, net::SocketAddr, str::FromStr};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub http: Http,
    #[serde(default)]
    pub log: Log,
    #[serde(default)]
    pub storage: Storage,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Http {
    #[serde(default = "default_http_listen")]
    pub listen: SocketAddr,
    #[serde(default)]
    pub cors: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    #[serde(default = "default_log_level")]
    pub level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Storage {
    #[serde(default = "default_storage_path")]
    pub path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            http: Http::default(),
            log: Log::default(),
            storage: Storage::default(),
        }
    }
}

fn default_http_listen() -> SocketAddr {
    SocketAddr::from_str(&format!(
        "0.0.0.0:{}",
        env::var("VOD_PORT").unwrap_or(String::from("7778"))
    ))
    .expect("invalid listen address")
}

impl Default for Http {
    fn default() -> Self {
        Self {
            listen: default_http_listen(),
            cors: true,
        }
    }
}

impl Default for Log {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

fn default_log_level() -> String {
    env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
}

fn default_storage_path() -> String {
    env::var("RECORD_PATH").unwrap_or_else(|_| "./recordings".to_string())
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            path: default_storage_path(),
        }
    }
}

impl Config {
    pub fn validate(&self) -> anyhow::Result<()> {
        Ok(())
    }
}

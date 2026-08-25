use anyhow::Result;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: String,

    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,

    pub replicas: Vec<Replica>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Replica {
    pub name: String,
    pub address: String,
    pub destination: String,
}

fn default_debounce() -> u64 {
    500
}

impl Config {
    pub fn load(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }
}
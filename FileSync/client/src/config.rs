use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub server_ip: String,
    pub server_port: u16,
    pub sync_folder: String,
}

pub fn load(path: &str) -> Config {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("could not read config file '{path}': {e}"));
    toml::from_str(&text).unwrap_or_else(|e| panic!("invalid config file '{path}': {e}"))
}

use serde::{Deserialize, Serialize};
use std::fs::{read_to_string, write};
use toml::{from_str, to_string};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub prefix: String,
}

pub fn load_config(config_path: Option<&str>) -> Config {
    let config_path = config_path.unwrap_or("config.toml");

    match read_to_string(config_path) {
        Ok(content) => from_str(&content).unwrap(),
        Err(_) => Config {
            prefix: "!".to_string(),
        },
    }
}

pub fn save_config(config_path: Option<&str>, config: &Config) {
    let config_path = config_path.unwrap_or("config.toml");

    match write(config_path, to_string(config).unwrap()) {
        Ok(_) => (),
        Err(_) => eprintln!("Failed to save config"),
    }
}

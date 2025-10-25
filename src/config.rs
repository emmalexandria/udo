use anyhow::Result;
use std::fs;

use serde::{Deserialize, Serialize};

const CONFIG_PATH: &str = "/etc/udo/config.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    target: String,
    host: String,
    run_as: String,
    command: String,
}

impl Rule {
    pub fn new(target: String, host: String, run_as: String, command: String) -> Self {
        Self {
            target,
            host,
            run_as,
            command,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct DisplayConfig {
    #[serde(default)]
    pub color: bool,
    #[serde(default)]
    pub unicode: bool,
    #[serde(default)]
    pub nerd: bool,
    #[serde(default)]
    pub censor: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: true,
            unicode: true,
            nerd: false,
            censor: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig::default(),
            rules: vec![],
        }
    }
}

impl Config {
    pub fn read() -> Result<Self> {
        let content = fs::read_to_string(CONFIG_PATH)?;
        let de = toml::Deserializer::parse(&content)?;
        Ok(Self::deserialize(de)?)
    }
}

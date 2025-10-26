use anyhow::Result;
use std::fs;

use serde::{Deserialize, Serialize};

use crate::authenticate::Rule;

const CONFIG_PATH: &str = "/etc/udo/config.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SecurityConfig {
    #[serde(default)]
    pub timeout: i64,
    #[serde(default)]
    pub tries: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            timeout: 10,
            tries: 3,
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
    #[serde(default)]
    pub display_pw: bool,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: true,
            unicode: true,
            nerd: false,
            censor: true,
            display_pw: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Default)]
pub struct Config {
    #[serde(default)]
    pub display: DisplayConfig,
    #[serde(default)]
    pub rules: Vec<Rule>,
    #[serde(default)]
    pub security: SecurityConfig,
}


impl Config {
    pub fn read() -> Result<Self> {
        let content = fs::read_to_string(CONFIG_PATH)?;
        let de = toml::Deserializer::parse(&content)?;
        Ok(Self::deserialize(de)?)
    }
}

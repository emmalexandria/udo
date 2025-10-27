use anyhow::Result;
use std::{fs, io};
use toml::Deserializer;

use serde::{Deserialize, Serialize};

use crate::{
    authenticate::Rule,
    output::{self, theme::Theme},
};

const CONFIG_PATH: &str = "/etc/udo/config.toml";

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct SecurityConfig {
    pub safe_path: Option<String>,
    pub timeout: i64,
    pub tries: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            safe_path: None,
            timeout: 10,
            tries: 3,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(default)]
pub struct DisplayConfig {
    pub color: bool,
    pub unicode: bool,
    pub nerd: bool,
    pub censor: bool,
    pub display_pw: bool,
    pub theme: Theme,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            color: true,
            unicode: true,
            nerd: false,
            censor: true,
            display_pw: true,
            theme: Theme::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(default)]
pub struct Config {
    pub display: DisplayConfig,
    pub rules: Vec<Rule>,
    pub security: SecurityConfig,
}

impl Config {
    pub fn read() -> Result<Self> {
        let mut de: Option<Deserializer> = None;
        let mut content: Option<String> = None;
        match fs::read_to_string(CONFIG_PATH) {
            Ok(f) => content = Some(f),
            Err(e) => output::error(format!("Failed to read config file ({e})"), false),
        };

        if let Some(c) = &content {
            match toml::Deserializer::parse(c) {
                Ok(d) => de = Some(d),
                Err(e) => output::error(format!("Failed to create deserializer ({e})"), false),
            }
        }

        if let Some(de) = de {
            match Self::deserialize(de) {
                Ok(c) => Ok(c),
                Err(e) => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Could not parse config file \n{e}"),
                )
                .into()),
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "could not read configuration file",
            )
            .into())
        }
    }
}

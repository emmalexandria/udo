use crossterm::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptStyle {
    Minimal,
    #[default]
    Block,
    Shell,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct Theme {
    pub replace_char: char,
    pub prompt_color: Color,
    pub error_color: Color,
    pub warning_color: Color,
    pub info_color: Color,
    pub prompt_style: PromptStyle,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            replace_char: '•',
            prompt_color: Color::Green,
            error_color: Color::Red,
            warning_color: Color::Yellow,
            info_color: Color::Blue,
            prompt_style: PromptStyle::default(),
        }
    }
}

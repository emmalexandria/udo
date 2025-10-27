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
pub struct Theme {
    replace_char: char,
    prompt_color: Color,
    error_color: Color,
    warning_color: Color,
    info_color: Color,
    prompt_style: PromptStyle,
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

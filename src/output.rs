use std::fmt::Display;

use crossterm::style::{ContentStyle, StyledContent, Stylize};

use crate::config::Config;

pub mod prompt;

fn block(style: &ContentStyle, name: &str, icon: &str) -> MultiStyled<String> {
    MultiStyled::default()
        .with(style.apply(format!(" {icon} ")))
        .with(style.apply("[udo]".to_string()).bold())
        .with(style.apply(format!(" {name} ")))
}

pub fn error<D: Display>(error: D, icon: bool) {
    let icon = match icon {
        true => '',
        false => '!',
    };

    let style = ContentStyle::default().on_red().black();
    let block = block(&style, "Error", &icon.to_string());

    eprintln!("{block} {error}");
}

pub fn info<D: Display>(info: D, icon: bool) {
    let icon = match icon {
        true => '',
        false => '#',
    };

    let style = ContentStyle::default().on_blue().black();
    let block = block(&style, "Info", &icon.to_string());

    println!("{block} {info}");
}

pub fn wrong_password(icon: bool, tries: usize) {
    let icon = match icon {
        true => '',
        false => '?',
    };

    let style = ContentStyle::default().on_yellow().black();
    let block = block(&style, "Warning", &icon.to_string());

    let try_text = if tries > 1 { "tries" } else { "try" };

    println!("{block} Incorrect. {tries} {try_text} remaining.")
}

pub fn lockout(config: &Config) {
    let lock = format!("{} incorrect password attempts", config.security.tries)
        .stylize()
        .yellow()
        .bold();
    println!("{lock}");
}

#[derive(Default, Debug, Clone)]
pub struct MultiStyled<D>
where
    D: Display,
{
    content: Vec<StyledContent<D>>,
}

impl<D: Display> MultiStyled<D> {
    pub fn with(mut self, content: StyledContent<D>) -> Self {
        self.content.push(content);
        self
    }
}

impl<D: Display> Display for MultiStyled<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for sec in &self.content {
            write!(f, "{sec}")?;
        }

        Ok(())
    }
}

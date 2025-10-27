use std::fmt::Display;

use anyhow::Result;
use crossterm::{
    style::{ContentStyle, StyledContent, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use nix::unistd::User;

use crate::{config::Config, output::prompt::InputPrompt};

pub mod prompt;
pub mod theme;

pub fn prompt_password(config: &Config) -> Result<String> {
    enable_raw_mode()?;
    let prompt = InputPrompt::default()
        .password_prompt(config)
        .obscure(config.display.censor)
        .char(config.display.theme.replace_char)
        .display_pw(config.display.display_pw);

    let res = prompt.run()?;

    disable_raw_mode()?;
    Ok(res)
}

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

pub fn error_with_details<S: Display, E: Display>(message: S, details: E, icon: bool) {
    error(message, icon);
    let details_style = ContentStyle::default().on_black();
    let details = details.to_string();
    let lines = details.lines().collect::<Vec<_>>();
    let mut longest = 0;
    lines.iter().for_each(|l| {
        if l.len() > longest {
            longest = l.len()
        }
    });

    let padded_lines = lines
        .iter()
        .map(|l| {
            let diff = longest - l.len();
            let pad = (0..diff).map(|_| ' ').collect::<String>();
            format!("{l}{pad}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let content = details_style.apply(padded_lines);
    eprintln!("{content}");
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

pub fn not_authenticated(user: &User, config: &Config) {
    let multi: MultiStyled<String> = MultiStyled::default().with(
        format!(
            "{} is not in the udo configuration. This incident won't be reported <3.",
            user.name
        )
        .stylize()
        .italic(),
    );

    eprintln!("{multi}")
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

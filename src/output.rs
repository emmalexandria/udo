use std::{fmt::Display, fs::OpenOptions, io};

use anyhow::Result;
use crossterm::{
    execute,
    style::{ContentStyle, Print, StyledContent, Stylize},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use nix::unistd::User;

use crate::{config::Config, output::prompt::InputPrompt};

pub mod prompt;
pub mod theme;

/// BLOCK_LEN represents the length of the longest box in our output, being the password box.
/// This is used to pad our other output
const BLOCK_LEN: usize = 20;

#[derive(Clone, Debug, Copy)]
pub enum Output {
    Stdout,
    Stderr,
    Tty,
}

impl Output {
    pub fn get_write(&self) -> Box<dyn io::Write> {
        match self {
            Output::Stdout => Box::new(io::stdout()),
            Output::Stderr => Box::new(io::stderr()),
            Output::Tty => {
                let fd = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open("/dev/tty")
                    .expect("Could not open /dev/tty");
                Box::new(fd)
            }
        }
    }
}

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
    let mut block = MultiStyled::default()
        .with(style.apply(format!(" {icon} ")))
        .with(style.apply("[udo]".to_string()).bold())
        .with(style.apply(format!(" {name} ")));

    if BLOCK_LEN > block.len() {
        let remaining = BLOCK_LEN - block.len();
        let pad = (0..remaining).map(|_| ' ').collect::<String>();

        block.push(style.apply(pad));
    }

    block
}

pub fn error<D: Display>(error: D, icon: bool, output: Option<Output>) {
    let icon = match icon {
        true => '',
        false => '!',
    };

    let style = ContentStyle::default().on_red().black();
    let block = block(&style, "Error", &icon.to_string());
    let output = output.unwrap_or(Output::Stderr);

    execute!(output.get_write(), Print(format!("{block} {error}\n")));
}

pub fn error_with_details<S: Display, E: Display>(
    message: S,
    details: E,
    icon: bool,
    output: Option<Output>,
) {
    error(message, icon, output);
    let details_style = ContentStyle::default().on_black();
    let details = details.to_string();
    let lines = details.lines().collect::<Vec<_>>();
    let mut longest = 0;
    lines.iter().for_each(|l| {
        if l.len() > longest {
            longest = l.len()
        }
    });

    let left_pad = (0..BLOCK_LEN).map(|_| ' ').collect::<String>();
    let padded_lines = lines
        .iter()
        .map(|l| {
            let diff = longest - l.len();
            let pad = (0..diff).map(|_| ' ').collect::<String>();
            MultiStyled::default()
                .with(left_pad.clone().stylize())
                .with(details_style.apply(format!("{l}{pad}")))
        })
        .collect::<Vec<_>>();
    let mut output = output.unwrap_or(Output::Stderr).get_write();

    for line in padded_lines {
        execute!(output, Print(line), Print("\n"));
    }
}

pub fn info<D: Display>(info: D, icon: bool, output: Option<Output>) {
    let icon = match icon {
        true => '',
        false => '#',
    };

    let style = ContentStyle::default().on_blue().black();
    let block = block(&style, "Info", &icon.to_string());

    let output = output.unwrap_or(Output::Stderr);
    execute!(output.get_write(), Print(format!("{block} {info}\n")));
}

pub fn wrong_password(icon: bool, tries: usize) {
    let icon = match icon {
        true => '',
        false => '?',
    };

    let style = ContentStyle::default().on_yellow().black();
    let block = block(&style, "Warning", &icon.to_string());

    let try_text = if tries > 1 { "tries" } else { "try" };

    eprintln!("{block} Incorrect. {tries} {try_text} remaining.")
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

    pub fn push(&mut self, content: StyledContent<D>) {
        self.content.push(content);
    }

    pub fn len(&self) -> usize {
        self.content
            .iter()
            .fold(0, |a, c| a + c.content().to_string().len())
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

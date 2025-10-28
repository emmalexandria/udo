use std::{
    fs::OpenOptions,
    io::{self, Write, stdout},
    process,
};

use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{ContentStyle, Print, Stylize},
    terminal::{Clear, ClearType},
};

use crate::{config::Config, output::MultiStyled};

pub struct InputPrompt {
    prompt: Option<MultiStyled<String>>,
    obscure: bool,
    display_pw: bool,
    char: char,
}

impl Default for InputPrompt {
    fn default() -> Self {
        Self {
            prompt: None,
            obscure: true,
            display_pw: true,
            char: '•',
        }
    }
}

impl InputPrompt {
    pub fn password_prompt(mut self, config: &Config) -> Self {
        let base = ContentStyle::default()
            .on(config.display.theme.prompt_color)
            .black();
        let icon = match config.display.nerd {
            true => " 󰒃 ",
            false => " * ",
        };
        let prompt = MultiStyled::default()
            .with(base.apply(icon.to_string()))
            .with(base.apply("[udo]".to_string()).bold())
            .with(base.apply(" Password:".to_string()));
        self.prompt = Some(prompt);
        self
    }

    pub fn obscure(mut self, yes: bool) -> Self {
        self.obscure = yes;
        self
    }

    pub fn display_pw(mut self, yes: bool) -> Self {
        self.display_pw = yes;
        self
    }

    pub fn char(mut self, char: char) -> Self {
        self.char = char;
        self
    }

    pub fn run(&self) -> io::Result<String> {
        let mut content = String::new();
        let mut running = true;
        let mut stdout = stdout();
        let mut tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;

        execute!(tty, MoveToColumn(0))?;

        while running {
            if let Some(p) = &self.prompt {
                execute!(tty, Print(p))?;
            }

            if self.obscure && self.display_pw {
                let obscured: String = (0..content.len()).map(|_| self.char).collect();
                execute!(tty, Print(format!(" {obscured}")))?;
            } else if self.display_pw {
                execute!(tty, Print(format!(" {content}")))?;
            }

            stdout.flush()?;

            if let Event::Key(e) = event::read()? {
                match (e.code, e.modifiers) {
                    (KeyCode::Enter, _) => running = false,
                    (KeyCode::Backspace, _) => {
                        let mut chars = content.chars();
                        chars.next_back();
                        content = chars.collect();
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => process::exit(0),
                    (KeyCode::Char(c), _) => content.push(c),
                    _ => {}
                }
            }

            if running {
                execute!(tty, Clear(ClearType::CurrentLine))?;
            } else {
                execute!(tty, Print("\n"))?;
            }

            execute!(tty, MoveToColumn(0))?;
        }

        Ok(content)
    }
}

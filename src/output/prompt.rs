use std::{
    io::{self, Write, stdout},
    process,
};

use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{ContentStyle, Stylize},
    terminal::{Clear, ClearType},
};

use crate::output::MultiStyled;

pub struct InputPrompt {
    prompt: Option<MultiStyled<String>>,
    obscure: bool,
    display_pw: bool,
}

impl Default for InputPrompt {
    fn default() -> Self {
        Self {
            prompt: None,
            obscure: true,
            display_pw: true,
        }
    }
}

impl InputPrompt {
    pub fn password_prompt(mut self) -> Self {
        let base = ContentStyle::default().on_green().black();
        let prompt = MultiStyled::default()
            .with(base.apply(" 󰒃 ".to_string()))
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

    pub fn run(&self) -> io::Result<String> {
        let mut content = String::new();
        let mut running = true;
        let mut stdout = stdout();

        execute!(stdout, MoveToColumn(0))?;

        while running {
            if let Some(p) = &self.prompt {
                print!("{p} ")
            }

            if self.obscure && self.display_pw {
                let obscured: String = (0..content.len()).map(|_| '•').collect();
                print!("{obscured}");
            } else if self.display_pw {
                print!("{content}");
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
                execute!(stdout, Clear(ClearType::CurrentLine))?;
            } else {
                println!();
            }

            execute!(stdout, MoveToColumn(0))?;
        }

        Ok(content)
    }
}

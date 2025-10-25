use std::{
    io::{self, Write, stdout},
    process,
};

use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::Stylize,
    terminal::{Clear, ClearType, enable_raw_mode},
};

pub struct InputPrompt {
    prompt: Option<String>,
    obscure: bool,
}

impl Default for InputPrompt {
    fn default() -> Self {
        Self {
            prompt: None,
            obscure: false,
        }
    }
}

impl InputPrompt {
    pub fn password_prompt(mut self) -> Self {
        self.prompt = Some(" 󰒃 Password: ".to_string());
        self
    }

    pub fn with_prompt<S: ToString>(mut self, prompt: S) -> Self {
        self.prompt = Some(prompt.to_string());
        self
    }

    pub fn obscure(mut self, yes: bool) -> Self {
        self.obscure = yes;
        self
    }

    pub fn run(&self) -> io::Result<String> {
        let mut content = String::new();
        let mut running = true;
        let mut stdout = stdout();

        execute!(stdout, MoveToColumn(0))?;

        while running {
            if let Some(p) = &self.prompt {
                print!("{} ", p.clone().stylize().on_green().black());
            }

            if self.obscure {
                let obscured: String = (0..content.len()).map(|_| '•').collect();
                print!("{obscured}");
            } else {
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

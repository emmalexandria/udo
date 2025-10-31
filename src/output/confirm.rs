use std::{fs::OpenOptions, io};

use crossterm::{
    cursor::MoveToColumn,
    event::{Event, KeyCode, KeyEvent, read},
    execute,
    style::{ContentStyle, Print, Stylize},
    terminal::{Clear, ClearType},
};

use crate::output::MultiStyled;

pub struct Confirmation {
    selected: bool,
    prompt: String,
}

impl Default for Confirmation {
    fn default() -> Self {
        Self {
            selected: false,
            prompt: String::new(),
        }
    }
}

impl Confirmation {
    pub fn with_prompt<S: ToString>(mut self, prompt: S) -> Self {
        self.prompt = prompt.to_string();
        self
    }

    pub fn run(&mut self) -> io::Result<bool> {
        let mut running = true;
        let mut tty = OpenOptions::new().read(true).write(true).open("/dev/tty")?;

        execute!(tty, MoveToColumn(0))?;

        while running {
            let selected_style = ContentStyle::default().on_white();
            let prompt_style = ContentStyle::default().yellow().bold();
            let styled: MultiStyled<String> = MultiStyled::default()
                .with(prompt_style.apply(self.prompt.clone()))
                .with("[y/n]".to_string().stylize());

            execute!(tty, Print(styled))?;

            if let Ok(ev) = read()
                && let Event::Key(KeyEvent {
                    code,
                    modifiers: _,
                    kind: _,
                    state: _,
                }) = ev
            {
                match code {
                    KeyCode::Enter => running = false,
                    KeyCode::Left | KeyCode::Right => self.change_select(),
                    KeyCode::Char('y') => {
                        self.selected = true;
                        running = false
                    }
                    KeyCode::Char('n') => {
                        self.selected = false;
                        running = false;
                    }
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

        Ok(self.selected)
    }

    fn change_select(&mut self) {
        self.selected = !self.selected
    }
}

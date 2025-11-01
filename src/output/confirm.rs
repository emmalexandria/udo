use std::{fs::OpenOptions, io, os};

use crossterm::{
    cursor::{Hide, MoveToColumn, Show},
    event::{Event, KeyCode, KeyEvent, KeyModifiers, read},
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

        execute!(tty, Hide)?;
        execute!(tty, MoveToColumn(0))?;

        while running {
            let selected_style = ContentStyle::default().black().bold();
            let (y, n) = match self.selected {
                true => (
                    selected_style.apply("y".to_string()).on_green(),
                    "n".to_string().stylize(),
                ),
                false => (
                    "y".to_string().stylize(),
                    selected_style.apply("n".to_string()).on_red(),
                ),
            };
            let prompt_style = ContentStyle::default().on_yellow().black().bold();
            let styled: MultiStyled<String> = MultiStyled::default()
                .with(prompt_style.apply(self.prompt.clone()))
                .with(" ".to_string().stylize())
                .with("[".to_string().stylize())
                .with(y)
                .with("/".to_string().stylize())
                .with(n)
                .with("]".to_string().stylize());

            execute!(tty, Print(styled))?;

            if let Ok(ev) = read()
                && let Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind: _,
                    state: _,
                }) = ev
            {
                match (code, modifiers) {
                    (KeyCode::Enter, _) => running = false,
                    (KeyCode::Left, _) | (KeyCode::Right, _) => self.change_select(),
                    (KeyCode::Char('y'), _) => {
                        self.selected = true;
                        running = false
                    }
                    (KeyCode::Char('n'), _) => {
                        self.selected = false;
                        running = false;
                    }
                    (KeyCode::Char('c'), KeyModifiers::CONTROL) => std::process::exit(0),
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

        execute!(tty, Show)?;

        Ok(self.selected)
    }

    fn change_select(&mut self) {
        self.selected = !self.selected
    }
}

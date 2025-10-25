use std::fmt::Display;

use crossterm::style::{ContentStyle, StyledContent, Stylize};

pub mod prompt;

pub fn error<D: Display>(error: D, icon: bool) {
    let icon = match icon {
        true => '',
        false => '!',
    };

    let style = ContentStyle::default().on_red().black();
    let block = style.apply(format!(" {icon} ERROR "));

    println!("{block} {error}");
}

pub fn wrong_password() {}

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

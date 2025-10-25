use crossterm::{
    style::force_color_output,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use nix::unistd::{geteuid, getuid};

use crate::{
    cli::get_cli,
    config::Config,
    output::prompt::InputPrompt,
    run::{elevate, run},
};

mod cli;
mod config;
mod output;
mod run;

fn main() {
    let cli = get_cli();
    let matches = cli.get_matches();
    let config = Config::read().unwrap();

    if !config.display.color {
        force_color_output(false);
    }

    if let Some(command) = matches.get_one::<String>("command") {
        enable_raw_mode().unwrap();
        let prompt = InputPrompt::default()
            .password_prompt()
            .obscure(config.display.censor);
        let input = prompt.run();
        disable_raw_mode().unwrap();
        elevate().unwrap();
        run(command).unwrap();
    }
}

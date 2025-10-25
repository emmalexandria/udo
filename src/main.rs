use std::process::exit;

use crossterm::{
    style::force_color_output,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use nix::unistd::{geteuid, getuid};

use crate::{
    authenticate::{AuthResult, authenticate},
    cli::get_cli,
    config::Config,
    output::prompt::InputPrompt,
    run::{elevate, run},
};

mod authenticate;
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

    if getuid().is_root() {
        output::error("Already running as root", config.display.nerd);
        exit(1);
    }

    if let Some(command) = matches.get_one::<String>("command") {
        let password = prompt_password(&config);
        if let Ok(AuthResult::Success) = authenticate(password, &config) {
            elevate().unwrap();
            run(command).unwrap();
        }
    }
}

fn prompt_password(config: &Config) -> String {
    enable_raw_mode().unwrap();
    let prompt = InputPrompt::default()
        .password_prompt()
        .obscure(config.display.censor);

    let res = prompt.run().unwrap();

    disable_raw_mode().unwrap();
    res
}

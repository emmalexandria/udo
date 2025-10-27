use std::{
    collections::HashMap,
    env,
    process::{self, exit},
};

use anyhow::Result;
use clap::ArgMatches;
use crossterm::style::force_color_output;
use nix::{
    sys::stat::{Mode, stat},
    unistd::{Uid, User, getuid},
};

use crate::{
    authenticate::{AuthResult, authenticate_password, check_action_auth},
    cache::Cache,
    cli::get_cli,
    config::Config,
    output::{lockout, not_authenticated, prompt_password, wrong_password},
    run::Run,
    user::{get_root_user, get_user, get_user_by_id},
};

mod authenticate;
mod cache;
mod cli;
mod config;
mod elevate;
mod error;
mod output;
mod run;
mod user;

fn main() {
    let cli = get_cli();
    let matches = cli.get_matches();
    let config = match Config::read() {
        Ok(c) => c,
        Err(e) => {
            output::error_with_details("Config error", e, false);
            println!("In future, please consider using udoedit");
            println!(
                "Use sudo/doas to fix the config file, or chroot into your system from a live system."
            );
            exit(1)
        }
    };

    if !config.display.color {
        force_color_output(false);
    }

    let run = Run::create(&matches, &config);
    match run {
        Ok(mut r) => {
            r.do_run();
        }
        Err(e) => output::error_with_details("Failed to initialise run", e, config.display.nerd),
    }
}

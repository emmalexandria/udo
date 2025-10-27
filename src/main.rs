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

fn login_user(run: &mut UdoRun, config: &Config, tries: usize) -> Result<bool> {
    match run.cache.check_cache(run.clone(), config) {
        Ok(true) => {
            return Ok(true);
        }
        Ok(false) => {}
        Err(e) => output::error(
            format!("failed to check cache ({e}). requesting password"),
            config.display.nerd,
        ),
    }

    let password = prompt_password(config);
    if let Err(e) = &password {
        output::error(
            format!("Failed to display password prompt ({})", e),
            config.display.nerd,
        );
    }

    let auth = authenticate_password(run, config, password.unwrap());

    match auth {
        AuthResult::Success => Ok(true),
        AuthResult::NotAuthenticated => {
            if tries > 1 {
                wrong_password(config.display.nerd, tries - 1);
                login_user(run, config, tries - 1)
            } else {
                lockout(config);
                Ok(false)
            }
        }
        AuthResult::AuthenticationFailure(s) => {
            output::error(
                format!("Authentication with PAM failed ({s})"),
                config.display.nerd,
            );
            Ok(false)
        }
    }
}

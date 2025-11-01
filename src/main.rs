use std::process::exit;

use crossterm::style::force_color_output;
use nix::unistd::Uid;

use crate::{
    backend::system::SystemBackend, cli::get_cli, config::Config, run::Run, user::get_root_user,
};

mod authenticate;
mod backend;
mod cache;
mod cli;
mod config;
mod error;
mod output;
mod run;
mod user;

fn main() {
    let cli = get_cli();
    let matches = cli.get_matches();
    let backend = SystemBackend::new(Uid::from_raw(0));
    let config = match Config::read(&backend) {
        Ok(c) => c,
        Err(e) => {
            output::error_with_details("Config error", e, false, None);
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

    let run = Run::create(&matches, &config, Box::new(backend));
    match run {
        Ok(mut r) => match r.do_run() {
            Ok(_) => {}
            Err(e) => {
                output::error_with_details("Could not execute run", e, config.display.nerd, None)
            }
        },
        Err(e) => {
            output::error_with_details("Failed to initialise run", e, config.display.nerd, None)
        }
    }
}

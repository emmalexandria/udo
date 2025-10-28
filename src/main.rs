use std::{fs::OpenOptions, io::Write, process::exit};

use crossterm::style::force_color_output;

use crate::{cli::get_cli, config::Config, run::Run};

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

    let mut tty = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")
        .unwrap();

    write!(tty, "Password: ");
    tty.flush();

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

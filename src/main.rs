use std::{
    env,
    process::{self, exit},
};

use anyhow::Result;
use clap::ArgMatches;
use crossterm::{
    style::force_color_output,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use nix::{
    sys::stat::{Mode, stat},
    unistd::{Uid, User, getuid},
};

use crate::{
    authenticate::{AuthResult, authenticate},
    cache::Cache,
    cli::get_cli,
    config::Config,
    output::{lockout, not_authenticated, prompt_password, wrong_password},
    run::do_run,
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

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CommandType {
    Command,
    Shell(bool),
}

#[derive(Debug, Clone)]
struct UdoRun {
    pub command: Vec<String>,
    pub c_type: CommandType,
    pub preserve_vars: bool,
    pub clear_cache: bool,
    pub cache: Cache,
    pub user: User,
    pub do_as: User,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let cli = get_cli();
    let matches = cli.get_matches();
    let config = Config::read().unwrap();

    if !config.display.color {
        force_color_output(false);
    }

    let nocheck = matches
        .get_one::<bool>("nocheck")
        .copied()
        .unwrap_or_default();

    if !nocheck && !check_perms(&config) {
        process::exit(1)
    }

    let mut udo_run = create_run(matches, &config);

    match login_user(&mut udo_run, &config, config.security.tries) {
        Ok(true) => after_login(&mut udo_run, &config).unwrap(),
        Ok(false) => output::info("Login failed", config.display.nerd),
        Err(e) => output::error(format!("Error while logging in {}", e), config.display.nerd),
    }

    Ok(())
}

fn create_run(matches: ArgMatches, config: &Config) -> UdoRun {
    let do_as = get_user(
        &matches
            .get_one::<String>("user")
            .cloned()
            .unwrap_or("root".to_string()),
    )
    .expect("Failed to get do_as user");
    let user = get_user_by_id(getuid()).expect("Failed to get current user");

    let root = match do_as.uid.is_root() {
        true => &do_as,
        false => &get_root_user(),
    };

    if user.uid == do_as.uid {
        output::error("Already running as target user", config.display.nerd);
        exit(1);
    }

    let clear_cache = matches
        .get_one::<bool>("clear")
        .copied()
        .unwrap_or_default();

    let preserve_vars = matches
        .get_one::<bool>("preserve")
        .copied()
        .unwrap_or_default();

    let command: Vec<String>;
    let c_type: CommandType;

    if let Some(("--shell", m)) = matches.subcommand() {
        let login = m.get_one::<bool>("login").copied().unwrap_or_default();
        command = vec![do_as.shell.to_string_lossy().to_string()];
        c_type = CommandType::Shell(login);
    } else {
        command = matches
            .get_many::<String>("command")
            .unwrap()
            .cloned()
            .collect();
        c_type = CommandType::Command;
    }

    let cache = Cache::new(&user, root);

    UdoRun {
        user,
        do_as,
        command,
        cache,
        c_type,
        preserve_vars,
        clear_cache,
    }
}

fn check_perms(config: &Config) -> bool {
    let exe = env::current_exe().unwrap();
    let st = stat(&exe).unwrap();

    let mut valid = true;

    let owner = Uid::from_raw(st.st_uid);
    if !owner.is_root() {
        output::error("udo is not owned by root", config.display.nerd);
        valid = false;
    }

    let perms = Mode::from_bits_truncate(st.st_mode);
    if !perms.contains(Mode::S_ISUID) {
        output::error("udo does not have suid perms", config.display.nerd);
        valid = false;
    }

    valid
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

    let auth = authenticate(
        &run.user,
        password.unwrap(),
        config,
        &run.do_as,
        &run.command[0],
    );

    match auth {
        Ok(AuthResult::Success) => return Ok(true),
        Ok(AuthResult::NotAuthenticated) => {
            if tries > 1 {
                wrong_password(config.display.nerd, tries - 1);
                return login_user(run, config, tries - 1);
            } else {
                lockout(config);
                return Ok(false);
            }
        }
        Ok(AuthResult::NotAuthorised) => {
            not_authenticated(&run.user, config);
            return Ok(false);
        }
        Ok(AuthResult::AuthenticationFailure(s)) => {
            output::error(
                format!("Authentication with PAM failed ({s})"),
                config.display.nerd,
            );
            return Ok(false);
        }
        Err(e) => {
            output::error(format!("Error authenticating: {e}"), config.display.nerd);
            return Ok(false);
        }
    }
}

fn after_login(udo_run: &mut UdoRun, config: &Config) -> Result<()> {
    if udo_run.clear_cache {
        udo_run.cache.clear().unwrap();
        output::info(
            format!("Cleared cache for \"{}\" of all entries", udo_run.user.name),
            config.display.nerd,
        );
    }

    udo_run.cache.create_dir()?;
    udo_run.cache.cache_run(udo_run.clone())?;

    do_run(udo_run)?;
    process::exit(0)
}

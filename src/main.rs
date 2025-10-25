use std::{
    env,
    process::{self, exit},
};

use anyhow::Result;
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
    cache::{cache_run, check_cache, clear_cache, create_cache_dir},
    cli::get_cli,
    config::Config,
    output::{lockout, prompt::InputPrompt, wrong_password},
    run::{elevate, elevate_to, run},
};

mod authenticate;
mod cache;
mod cli;
mod config;
mod elevate;
mod output;
mod run;

fn main() {
    let cli = get_cli();
    let matches = cli.get_matches();
    let config = Config::read().unwrap();

    if !config.display.color {
        force_color_output(false);
    }

    if !matches.get_one::<bool>("nocheck").copied().unwrap_or(false) && !check_perms(&config) {
        process::exit(1)
    }

    let uid = getuid();
    let user = User::from_uid(uid).unwrap().unwrap();
    let do_as = User::from_name(matches.get_one::<String>("user").unwrap())
        .unwrap()
        .unwrap();

    if uid == do_as.uid {
        output::error("Already running as target user", config.display.nerd);
        exit(1);
    }

    if let Some(true) = matches.get_one::<bool>("clear") {
        elevate().unwrap();
        clear_cache(&user).unwrap();
    }

    if let Some(command) = matches.get_many::<String>("command") {
        check_and_run(
            command.collect(),
            &user,
            &config,
            &do_as,
            config.security.tries,
        )
        .unwrap();
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

fn check_and_run(
    args: Vec<&String>,
    user: &User,
    config: &Config,
    do_as: &User,
    tries: usize,
) -> Result<()> {
    if do_as.uid.is_root() && check_cache(user, config)? {
        after_auth(args, user, do_as)?;
        return Ok(());
    }

    let password = prompt_password(config);
    let auth = authenticate(user, password, config, do_as, args[0]);

    match auth {
        Ok(AuthResult::Success) => after_auth(args, user, do_as)?,
        Ok(AuthResult::NotAuthorised) => {}
        Ok(AuthResult::AuthenticationFailure) => {
            if tries > 1 {
                wrong_password(config.display.nerd, tries - 1);
                check_and_run(args, user, config, do_as, tries - 1)?;
            } else {
                lockout(config.security.lockout);
                process::exit(0);
            }
        }
        Err(e) => output::error(format!("Error authenticating: {e}"), config.display.nerd),
    }

    Ok(())
}

fn after_auth(cmd: Vec<&String>, user: &User, do_as: &User) -> Result<()> {
    elevate_to(&do_as.name)?;
    if do_as.uid.is_root() {
        create_cache_dir(&user.name)?;
        cache_run(user)?;
    }

    run(cmd, do_as)?;
    process::exit(0)
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

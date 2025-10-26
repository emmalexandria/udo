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
    cache::Cache,
    cli::get_cli,
    config::Config,
    output::{lockout, not_authenticated, prompt::InputPrompt, wrong_password},
    run::{env::Env, process::run_process, shell::get_shell_cmd},
};

mod authenticate;
mod cache;
mod cli;
mod config;
mod elevate;
mod error;
mod output;
mod run;

#[derive(PartialEq, Eq)]
pub enum CommandType {
    Command,
    Shell(bool),
}

struct UdoRun {
    pub command: Vec<String>,
    pub c_type: CommandType,
    pub preserve_vars: bool,
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

    if !matches.get_one::<bool>("nocheck").copied().unwrap_or(false) && !check_perms(&config) {
        process::exit(1)
    }

    let do_as_opt = User::from_name(
        &matches
            .get_one::<String>("user")
            .cloned()
            .unwrap_or("root".to_string()),
    )
    .unwrap();
    if do_as_opt.is_none() {
        output::error("Target user not found", config.display.nerd);
        process::exit(1);
    }

    let do_as = do_as_opt.unwrap();

    let user = User::from_uid(getuid()).unwrap().unwrap();

    if user.uid == do_as.uid {
        output::error("Already running as target user", config.display.nerd);
        exit(1);
    }

    let mut cache = Cache::new(&user, &do_as);
    if let Some(true) = matches.get_one::<bool>("clear") {
        cache.clear().unwrap();
        output::info(
            format!("Cleared cache for \"{}\" of all entries", user.name),
            config.display.nerd,
        );
    }

    let preserve_vars = matches
        .get_one::<bool>("preserve")
        .copied()
        .unwrap_or_default();

    if let Some(("--shell", matches)) = matches.subcommand() {
        let login = matches.get_one::<bool>("login").copied().unwrap_or(false);
        let shell = get_shell_cmd(&user).to_string_lossy().to_string();

        let run = UdoRun {
            command: vec![shell],
            c_type: CommandType::Shell(login),
            preserve_vars,
            do_as,
            user,
        };

        match check_and_run(&run, &config, &mut cache, config.security.tries) {
            Ok(_) => {}
            Err(e) => output::error(
                format!("Failed to run command, error: {e}"),
                config.display.nerd,
            ),
        }
        return Ok(());
    }

    let cmd = matches.get_many::<String>("command");
    if let Some(cmd_vals) = cmd {
        let command = cmd_vals.cloned().collect();

        let run = UdoRun {
            command,
            c_type: CommandType::Command,
            preserve_vars,
            do_as,
            user,
        };

        check_and_run(&run, &config, &mut cache, config.security.tries).unwrap();
        return Ok(());
    }

    Ok(())
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

fn check_and_run(run: &UdoRun, config: &Config, cache: &mut Cache, tries: usize) -> Result<()> {
    if run.do_as.uid.is_root() {
        match cache.check_cache(run, config) {
            Ok(true) => {
                after_auth(run, cache, false)?;
            }
            Ok(false) => {}
            Err(e) => output::error(
                format!("failed to check cache ({e}). requesting password"),
                config.display.nerd,
            ),
        }
    }

    let password = prompt_password(config);
    let auth = authenticate(&run.user, password, config, &run.do_as, &run.command[0]);

    match auth {
        Ok(AuthResult::Success) => after_auth(run, cache, true)?,
        Ok(AuthResult::NotAuthenticated) => {
            if tries > 1 {
                wrong_password(config.display.nerd, tries - 1);
                check_and_run(run, config, cache, tries - 1)?;
            } else {
                lockout(config);
                process::exit(0);
            }
        }
        Ok(AuthResult::NotAuthorised) => {
            not_authenticated(&run.user, config);
        }
        Ok(AuthResult::AuthenticationFailure(s)) => {
            output::error(
                format!("Authentication with PAM failed ({s})"),
                config.display.nerd,
            );
        }
        Err(e) => output::error(format!("Error authenticating: {e}"), config.display.nerd),
    }

    Ok(())
}

fn after_auth(udo_run: &UdoRun, cache: &mut Cache, with_pass: bool) -> Result<()> {
    if udo_run.do_as.uid.is_root() && with_pass {
        cache.create_dir()?;
        cache.cache_run(udo_run)?;
    }

    let env = match udo_run.c_type {
        CommandType::Command => Env::process_env(&udo_run),
        CommandType::Shell(l) => Env::shell_env(&udo_run),
    };

    run_process(&udo_run.command, &env)?;
    process::exit(0)
}

fn prompt_password(config: &Config) -> String {
    enable_raw_mode().unwrap();
    let prompt = InputPrompt::default()
        .password_prompt()
        .obscure(config.display.censor)
        .display_pw(config.display.display_pw);

    let res = prompt.run().unwrap();

    disable_raw_mode().unwrap();
    res
}

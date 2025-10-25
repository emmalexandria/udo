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
    elevate::ElevatedContext,
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

struct UdoRun {
    pub command: Vec<String>,
    pub user: User,
    pub do_as: User,
}

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

    if let Some(true) = matches.get_one::<bool>("clear") {
        let mut context = ElevatedContext::new(user.uid, Uid::from_raw(0));
        context.elevate();
        clear_cache(&user).unwrap();
    }

    let cmd = matches.get_many::<String>("command");

    if let Some(cmd_vals) = cmd {
        let command = cmd_vals.cloned().collect();

        let run = UdoRun {
            command,
            do_as,
            user,
        };

        check_and_run(&run, &config, config.security.tries);
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

fn check_and_run(run: &UdoRun, config: &Config, tries: usize) -> Result<()> {
    if run.do_as.uid.is_root() && check_cache(&run.user, config)? {
        after_auth(run, false)?;
        return Ok(());
    }

    let password = prompt_password(config);
    let auth = authenticate(&run.user, password, config, &run.do_as, &run.command[0]);

    match auth {
        Ok(AuthResult::Success) => after_auth(run, true)?,
        Ok(AuthResult::NotAuthorised) => {}
        Ok(AuthResult::AuthenticationFailure) => {
            if tries > 1 {
                wrong_password(config.display.nerd, tries - 1);
                check_and_run(run, config, tries - 1)?;
            } else {
                lockout(config.security.lockout);
                process::exit(0);
            }
        }
        Err(e) => output::error(format!("Error authenticating: {e}"), config.display.nerd),
    }

    Ok(())
}

fn after_auth(udo_run: &UdoRun, with_pass: bool) -> Result<()> {
    let mut context = ElevatedContext::new(udo_run.user.uid, udo_run.do_as.uid);
    context.elevate()?;
    if udo_run.do_as.uid.is_root() && with_pass {
        create_cache_dir(&udo_run.user.name)?;
        cache_run(&udo_run.user)?;
    }

    run(&udo_run.command, &udo_run.do_as)?;
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

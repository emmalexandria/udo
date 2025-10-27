use crate::{
    CommandType, UdoRun,
    config::Config,
    run::{env::Env, process::run_process},
};
use anyhow::Result;

pub mod env;
pub mod process;

pub fn do_run(run: &UdoRun, config: &Config) -> Result<()> {
    match run.c_type {
        CommandType::Command => process(run, config),
        CommandType::Shell(_) => shell(run, config),
    }
}

fn process(run: &UdoRun, config: &Config) -> Result<()> {
    let env = Env::process_env(run, config.security.safe_path.as_ref());
    run_process(run.command.as_slice(), &env)?;

    Ok(())
}

fn shell(run: &UdoRun, config: &Config) -> Result<()> {
    let env = Env::shell_env(run, config.security.safe_path.as_ref());
    run_process(run.command.as_slice(), &env)?;

    Ok(())
}

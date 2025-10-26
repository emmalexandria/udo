use crate::{
    CommandType, UdoRun,
    run::{env::Env, process::run_process},
};
use anyhow::Result;

pub mod env;
pub mod process;

pub fn do_run(run: &UdoRun) -> Result<()> {
    match run.c_type {
        CommandType::Command => process(run),
        CommandType::Shell(_) => shell(run),
    }
}

fn process(run: &UdoRun) -> Result<()> {
    let env = Env::process_env(&run);
    run_process(run.command.as_slice(), &env)?;

    Ok(())
}

fn shell(run: &UdoRun) -> Result<()> {
    let env = Env::shell_env(run);
    run_process(run.command.as_slice(), &env)?;

    Ok(())
}

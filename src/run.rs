use crate::{CommandType, UdoRun, run::env::Env};
use anyhow::Result;

pub mod env;
pub mod process;
pub mod shell;

pub fn do_run(run: &UdoRun) -> Result<()> {
    match run.c_type {
        CommandType::Command => process(run),
        CommandType::Shell(login) => shell(run, login),
    }
}

fn process(run: &UdoRun) -> Result<()> {
    let env = Env::process_env(&run);

    Ok(())
}

fn shell(run: &UdoRun, login: bool) -> Result<()> {
    Ok(())
}

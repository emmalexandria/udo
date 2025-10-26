use std::{ffi::CString, process::exit};

use anyhow::Result;
use nix::{
    sys::{
        stat::{Mode, umask},
        wait::{WaitStatus, waitpid},
    },
    unistd::{ForkResult, Pid, execvp, fork},
};

use crate::{CommandType, run::env::Env};

pub fn run_process<S: ToString>(cmd: &[S], env: &Env) -> Result<()> {
    let cmd = cmd.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let cmd_name = cmd[0].as_str();
    let args = cmd.iter().map(String::as_str).collect::<Vec<_>>();

    run_with_args(cmd_name, &args, env)?;

    Ok(())
}

pub fn run_with_args<S: ToString>(name: S, args: &[S], env: &Env) -> Result<()> {
    let cmd_name = name.to_string();
    let mut args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    if env.command_type == CommandType::Shell(true) {
        args[0] = format!("-{}", args[0]);
    }

    let args_str = args.iter().map(String::as_str).collect();

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child }) => parent(child)?,
            Ok(ForkResult::Child) => child(&cmd_name, args_str, env)?,
            Err(e) => return Err(e.into()),
        }
    }

    Ok(())
}

fn parent(child: Pid) -> Result<()> {
    match waitpid(child, None) {
        Ok(WaitStatus::Exited(_, status)) => exit(status),
        Ok(WaitStatus::Signaled(_, signal, _)) => exit(128 + signal as i32),
        Ok(status) => exit(1),
        Err(e) => exit(1),
    }
}

fn child(cmd_name: &str, args: Vec<&str>, env: &Env) -> Result<()> {
    let program = CString::new(cmd_name)?;
    let args: Vec<CString> = args.into_iter().map(|a| CString::new(a).unwrap()).collect();

    unsafe {
        env.apply();
        umask(Mode::from_bits(0o022).unwrap());
    }

    execvp(&program, &args)?;

    Ok(())
}

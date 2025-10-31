use std::{ffi::CString, process::exit};

use anyhow::Result;
use nix::{
    sys::{
        stat::{Mode, umask},
        wait::{WaitStatus, waitpid},
    },
    unistd::{ForkResult, Pid, execvp, fork},
};

use crate::run::env::Env;

pub fn run_process<S: ToString>(cmd: &[S], env: &mut Env) -> Result<()> {
    let cmd = cmd.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let cmd_name = cmd[0].as_str();
    let args = cmd.iter().map(String::as_str).collect::<Vec<_>>();

    run_with_args(cmd_name, &args, env)?;

    Ok(())
}

pub fn run_with_args<S: ToString>(name: S, args: &[S], env: &mut Env) -> Result<()> {
    let cmd_name = name.to_string();
    let mut args = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();

    if env.login {
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
        // If it was killed by a signal, we exit with 128 + signal, apparently standard Unix
        // convention
        Ok(WaitStatus::Signaled(_, signal, _)) => exit(128 + signal as i32),
        Ok(status) => exit(1),
        Err(e) => exit(e as i32),
    }
}

fn child(cmd_name: &str, args: Vec<&str>, env: &mut Env) -> Result<()> {
    unsafe {
        env.apply()?;
        umask(Mode::from_bits(0o022).unwrap());
    }

    env.backend.execvp(cmd_name, &args)?;

    Ok(())
}

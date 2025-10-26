use std::{ffi::CString, process::exit};

use anyhow::Result;
use nix::{
    sys::{
        stat::{Mode, umask},
        wait::{WaitStatus, waitpid},
    },
    unistd::{ForkResult, Pid, User, execvp, fork},
};

use crate::{
    elevate::elevate_final,
    run::env::{clear_env, reset_signal_handlers},
};

pub fn run_process<S: ToString>(cmd: &[S], do_as: &User) -> Result<()> {
    let cmd = cmd.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    let cmd_name = &cmd[0];
    let args = cmd.iter().map(String::as_str).collect();

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child }) => parent(child)?,
            Ok(ForkResult::Child) => child(cmd_name, args, do_as)?,
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

fn child(cmd_name: &str, args: Vec<&str>, do_as: &User) -> Result<()> {
    let program = CString::new(cmd_name)?;
    let args: Vec<CString> = args.into_iter().map(|a| CString::new(a).unwrap()).collect();

    elevate_final(do_as.uid)?;

    unsafe {
        clear_env(do_as);
        umask(Mode::from_bits(0o022).unwrap());
        reset_signal_handlers();
    }

    execvp(&program, &args)?;

    Ok(())
}

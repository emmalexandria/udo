use std::{
    env,
    ffi::{CStr, CString, OsStr, OsString},
    process::{Command, exit},
};

use anyhow::Result;
use nix::{
    libc,
    sys::{
        stat::{Mode, umask},
        wait::{WaitStatus, waitpid},
    },
    unistd::{ForkResult, Gid, Pid, Uid, execvp, fork, setgid, setuid},
};

pub fn elevate() -> Result<()> {
    setuid(Uid::from_raw(0))?;
    setgid(Gid::from_raw(0))?;

    Ok(())
}

pub fn run<S: ToString>(cmd: S) -> Result<()> {
    let cmd = cmd.to_string();
    let mut split = cmd.split(" ");
    let cmd_name = split.next().unwrap();
    let args: Vec<&str> = split.collect();

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child }) => parent(child)?,
            Ok(ForkResult::Child) => child(cmd_name, args)?,
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

fn child(cmd_name: &str, args: Vec<&str>) -> Result<()> {
    setgid(Gid::from_raw(0))?;
    setuid(Uid::from_raw(0))?;

    let program = CString::new(cmd_name)?;
    let args: Vec<CString> = args.into_iter().map(|a| CString::new(a).unwrap()).collect();

    unsafe {
        clear_env();
        umask(Mode::from_bits(0o022).unwrap());
    }

    execvp(&program, &args)?;

    Ok(())
}

const SAFE_VARS: [&str; 8] = [
    "TERM",
    "DISPLAY",
    "XAUTHORITY",
    "LANG",
    "LANGUAGE",
    "EDITOR",
    "VISUAL",
    "PAGER",
];

unsafe fn clear_env() {
    for (name, _) in env::vars_os() {
        if !is_var_allowed(&name) {
            unsafe { env::remove_var(name) };
        }
    }
}

fn is_var_allowed(var: &OsStr) -> bool {
    let str = var.to_string_lossy();
    str.starts_with("LC_") || SAFE_VARS.contains(&str.as_ref())
}

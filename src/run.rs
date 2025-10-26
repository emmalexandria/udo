use std::{
    env,
    ffi::{CString, OsStr},
    io,
    process::exit,
};

use anyhow::Result;
use nix::{
    sys::{
        signal::{self, SigHandler, Signal},
        stat::{Mode, umask},
        wait::{WaitStatus, waitpid},
    },
    unistd::{ForkResult, Gid, Pid, Uid, User, execvp, fork, setgid, setuid},
};

use crate::config::Config;

pub fn elevate() -> Result<()> {
    setuid(Uid::from_raw(0))?;
    setgid(Gid::from_raw(0))?;

    Ok(())
}

pub fn run<S: ToString>(cmd: &Vec<S>, do_as: &User) -> Result<()> {
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

    elevate()?;

    unsafe {
        clear_env();
        umask(Mode::from_bits(0o022).unwrap());
        reset_signal_handlers();
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

const SAFE_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

unsafe fn clear_env() {
    for (name, _) in env::vars_os() {
        if !is_var_allowed(&name) {
            unsafe { env::remove_var(name) };
        }
    }

    unsafe {
        env::set_var("PATH", SAFE_PATH);
        env::set_var("HOME", get_user_home(0));
        env::set_var("USER", "root");
        env::set_var("LOGNAME", "root");
    }
}

fn is_var_allowed(var: &OsStr) -> bool {
    let str = var.to_string_lossy();
    str.starts_with("LC_") || SAFE_VARS.contains(&str.as_ref())
}

unsafe fn reset_signal_handlers() {
    let signals = [
        Signal::SIGHUP,
        Signal::SIGINT,
        Signal::SIGQUIT,
        Signal::SIGILL,
        Signal::SIGTRAP,
        Signal::SIGABRT,
        Signal::SIGBUS,
        Signal::SIGFPE,
        Signal::SIGUSR1,
        Signal::SIGSEGV,
        Signal::SIGUSR2,
        Signal::SIGPIPE,
        Signal::SIGALRM,
        Signal::SIGTERM,
        // Add more as needed
    ];

    for sig in &signals {
        unsafe {
            // Reset to default handler
            let _ = signal::signal(*sig, SigHandler::SigDfl);
        }
    }
}

fn get_user_home(uid: u32) -> String {
    User::from_uid(Uid::from_raw(uid))
        .ok()
        .flatten()
        .and_then(|user| user.dir.into_os_string().into_string().ok())
        .unwrap_or_else(|| {
            if uid == 0 {
                if cfg!(target_os = "macos") {
                    "/var/root".to_string()
                } else {
                    "/root".to_string()
                }
            } else {
                format!("/home/user{}", uid)
            }
        })
}

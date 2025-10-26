use std::{env, ffi::OsStr};

use nix::{
    sys::signal::{self, SigHandler, Signal},
    unistd::{Uid, User},
};

pub const SAFE_VARS: [&str; 8] = [
    "TERM",
    "DISPLAY",
    "XAUTHORITY",
    "LANG",
    "LANGUAGE",
    "EDITOR",
    "VISUAL",
    "PAGER",
];

pub const SAFE_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

pub unsafe fn clear_env(target: &User) {
    for (name, _) in env::vars_os() {
        if !is_var_allowed(&name) {
            unsafe { env::remove_var(name) };
        }
    }

    unsafe {
        env::set_var("PATH", SAFE_PATH);
        env::set_var("HOME", get_user_home(target.uid.as_raw()));
        env::set_var("USER", &target.name);
        env::set_var("LOGNAME", &target.name);
    }
}

fn is_var_allowed(var: &OsStr) -> bool {
    let str = var.to_string_lossy();
    str.starts_with("LC_") || SAFE_VARS.contains(&str.as_ref())
}

pub unsafe fn reset_signal_handlers() {
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

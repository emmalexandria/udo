use std::{env, ffi::OsStr, io};

use nix::{
    sys::signal::{self, SigHandler, Signal},
    unistd::{Uid, User, setuid},
};

use anyhow::Result;

use crate::CommandType;

pub struct Vars {
    pub home: String,
    pub user: String,
    pub logname: String,
    pub shell: String,
    pub path: String,
}

impl Vars {
    const SAFE_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

    pub fn from_user(user: &User) -> Self {
        Self {
            home: user.dir.to_string_lossy().to_string(),
            user: user.name.clone(),
            logname: user.name.clone(),
            shell: user.shell.to_string_lossy().to_string(),
            path: Self::SAFE_PATH.to_string(),
        }
    }
}

pub struct Env {
    pub command_type: CommandType,
    safe_vars: Vec<String>,
    set_vars: Vars,
    do_as: User,
}

impl Env {
    // These vars are preserved if we're running a process
    const SAFE_VARS: [&str; 6] = [
        "XAUTHORITY",
        "LANG",
        "LANGUAGE",
        "EDITOR",
        "VISUAL",
        "PAGER",
    ];

    // These vars are always preserved
    const PRESERVE_VARS: [&str; 2] = ["TERM", "DISPLAY"];

    pub fn shell_env(do_as: &User, login: bool) -> Self {
        match login {
            true => Self::login_env(do_as),
            false => Self::non_login_env(do_as),
        }
    }

    fn login_env(do_as: &User) -> Self {
        let safe_vars = Self::const_vars_to_vec(&Self::PRESERVE_VARS);
        Self {
            command_type: CommandType::Shell(true),
            safe_vars,
            set_vars: Vars::from_user(do_as),
            do_as: do_as.clone(),
        }
    }

    fn non_login_env(do_as: &User) -> Self {
        let mut safe_vars = Self::const_vars_to_vec(&Self::SAFE_VARS);
        safe_vars.append(&mut Self::const_vars_to_vec(&Self::PRESERVE_VARS));

        Self {
            command_type: CommandType::Shell(false),
            safe_vars,
            set_vars: Vars::from_user(do_as),
            do_as: do_as.clone(),
        }
    }

    pub fn process_env(do_as: &User) -> Self {
        let mut safe_vars = Self::const_vars_to_vec(&Self::SAFE_VARS);
        safe_vars.append(&mut Self::const_vars_to_vec(&Self::PRESERVE_VARS));
        Self {
            command_type: CommandType::Command,
            safe_vars,
            set_vars: Vars::from_user(do_as),
            do_as: do_as.clone(),
        }
    }

    pub unsafe fn elevate_final(&self) -> Result<()> {
        setuid(self.do_as.uid)?;
        Ok(())
    }

    pub unsafe fn apply(&self) -> Result<()> {
        unsafe {
            self.apply_vars();
            self.elevate_final()?;
        }

        Ok(())
    }

    unsafe fn apply_vars(&self) {
        let vars = env::vars();

        unsafe {
            for (var, _) in vars {
                if !self.is_var_valid(&var) {
                    env::remove_var(var);
                }
            }

            env::set_var("PATH", &self.set_vars.path);
            env::set_var("HOME", &self.set_vars.home);
            env::set_var("USER", &self.set_vars.user);
            env::set_var("LOGNAME", &self.set_vars.logname);
        }
    }

    fn is_var_valid(&self, var: &String) -> bool {
        self.safe_vars.contains(var) || var.starts_with("LC_")
    }

    fn const_vars_to_vec(vars: &[&str]) -> Vec<String> {
        vars.iter().copied().map(str::to_string).collect()
    }
}

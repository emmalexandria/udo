use std::env;

use nix::unistd::{User, setuid};

use anyhow::Result;

use crate::{CommandType, UdoRun};

pub struct Vars {
    pub home: String,
    pub user: String,
    pub logname: String,
    pub shell: String,
    pub path: Option<String>,
}

impl Vars {
    pub fn from_run(run: &UdoRun, path: Option<&String>) -> Self {
        match run.c_type {
            CommandType::Command | CommandType::Shell(false) => Self::non_login(run, path.cloned()),
            CommandType::Shell(true) => Self::login(run, path.cloned()),
        }
    }

    fn non_login(run: &UdoRun, path: Option<String>) -> Self {
        Self {
            home: run.user.dir.to_string_lossy().to_string(),
            user: run.do_as.name.clone(),
            logname: run.do_as.name.clone(),
            shell: run.user.shell.to_string_lossy().to_string(),
            path,
        }
    }

    fn login(run: &UdoRun, path: Option<String>) -> Self {
        Self {
            home: run.do_as.dir.to_string_lossy().to_string(),
            user: run.do_as.name.clone(),
            logname: run.do_as.name.clone(),
            shell: run.do_as.shell.to_string_lossy().to_string(),
            path,
        }
    }
}

pub struct Env {
    pub command_type: CommandType,
    preserve_all: bool,
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

    pub fn shell_env(run: &UdoRun, path: Option<&String>) -> Self {
        let path = path.clone();
        match run.c_type {
            CommandType::Shell(true) => Self::login_env(run, path),
            CommandType::Shell(false) => Self::non_login_env(run, path),
            CommandType::Command => Self::non_login_env(run, path),
        }
    }

    fn login_env(run: &UdoRun, path: Option<&String>) -> Self {
        let safe_vars = Self::const_vars_to_vec(&Self::PRESERVE_VARS);
        Self {
            command_type: CommandType::Shell(true),
            safe_vars,
            preserve_all: run.preserve_vars,
            set_vars: Vars::from_run(run, path),
            do_as: run.do_as.clone(),
        }
    }

    fn non_login_env(run: &UdoRun, path: Option<&String>) -> Self {
        let mut safe_vars = Self::const_vars_to_vec(&Self::SAFE_VARS);
        safe_vars.append(&mut Self::const_vars_to_vec(&Self::PRESERVE_VARS));

        Self {
            command_type: CommandType::Shell(false),
            safe_vars,
            preserve_all: run.preserve_vars,
            set_vars: Vars::from_run(run, path),
            do_as: run.do_as.clone(),
        }
    }

    fn get_shell(run: &UdoRun) -> String {
        match run.c_type {
            CommandType::Shell(true) => run.do_as.shell.to_string_lossy().to_string(),
            CommandType::Shell(false) | CommandType::Command => {
                run.user.shell.to_string_lossy().to_string()
            }
        }
    }

    pub fn process_env(run: &UdoRun, path: Option<&String>) -> Self {
        let mut safe_vars = Self::const_vars_to_vec(&Self::SAFE_VARS);
        safe_vars.append(&mut Self::const_vars_to_vec(&Self::PRESERVE_VARS));
        Self {
            command_type: CommandType::Command,
            safe_vars,
            preserve_all: run.preserve_vars,
            set_vars: Vars::from_run(run, path),
            do_as: run.do_as.clone(),
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

        if self.command_type == CommandType::Shell(true) {
            env::set_current_dir(&self.set_vars.home)?;
        }

        Ok(())
    }

    unsafe fn apply_vars(&self) {
        let vars = env::vars();

        unsafe {
            if !self.preserve_all {
                for (var, _) in vars {
                    if !self.is_var_valid(&var) {
                        env::remove_var(var);
                    }
                }
            }

            if let Some(p) = &self.set_vars.path {
                env::set_var("PATH", p);
            }
            env::set_var("HOME", &self.set_vars.home);
            env::set_var("SHELL", &self.set_vars.shell);
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

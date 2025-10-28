use std::env;

use nix::{
    sys::stat::{Mode, umask},
    unistd::{User, setgid, setuid},
};

use anyhow::Result;

use crate::run::{Flag, Run};

pub struct Vars {
    pub home: String,
    pub user: String,
    pub logname: String,
    pub shell: String,
    pub path: Option<String>,
}

impl Vars {
    pub fn non_login(run: &Run) -> Self {
        Self {
            home: run.user.dir.to_string_lossy().to_string(),
            user: run.do_as.name.clone(),
            logname: run.do_as.name.clone(),
            shell: run.user.shell.to_string_lossy().to_string(),
            path: run.config.security.safe_path.clone(),
        }
    }

    pub fn login(run: &Run) -> Self {
        Self {
            home: run.do_as.dir.to_string_lossy().to_string(),
            user: run.do_as.name.clone(),
            logname: run.do_as.name.clone(),
            shell: run.do_as.shell.to_string_lossy().to_string(),
            path: run.config.security.safe_path.clone(),
        }
    }
}

pub struct Env {
    pub login: bool,
    pub preserve_all: bool,
    pub safe_vars: Vec<String>,
    pub set_vars: Vars,
    pub do_as: User,
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

    pub fn login_env(run: &Run, path: Option<&String>) -> Self {
        let safe_vars = Self::const_vars_to_vec(&Self::PRESERVE_VARS);
        Self {
            login: true,
            safe_vars,
            preserve_all: run.flags.contains(&Flag::PreserveVars),
            set_vars: Vars::login(run),
            do_as: run.do_as.clone(),
        }
    }

    pub fn non_login_env(run: &Run, path: Option<&String>) -> Self {
        let mut safe_vars = Self::const_vars_to_vec(&Self::SAFE_VARS);
        safe_vars.append(&mut Self::const_vars_to_vec(&Self::PRESERVE_VARS));

        Self {
            login: false,
            safe_vars,
            preserve_all: run.flags.contains(&Flag::PreserveVars),
            set_vars: Vars::non_login(run),
            do_as: run.do_as.clone(),
        }
    }

    pub fn process_env(run: &Run, path: Option<&String>) -> Self {
        let mut safe_vars = Self::const_vars_to_vec(&Self::SAFE_VARS);
        safe_vars.append(&mut Self::const_vars_to_vec(&Self::PRESERVE_VARS));
        Self {
            login: false,
            safe_vars,
            preserve_all: run.flags.contains(&Flag::PreserveVars),
            set_vars: Vars::non_login(run),
            do_as: run.do_as.clone(),
        }
    }

    pub unsafe fn elevate_final(&self) -> Result<()> {
        setgid(self.do_as.gid)?;
        setuid(self.do_as.uid)?;
        Ok(())
    }

    pub unsafe fn apply(&self) -> Result<()> {
        unsafe {
            umask(Mode::from_bits_truncate(0o022));
            self.apply_vars();
            self.elevate_final()?;
        }

        if self.login {
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

                if let Some(p) = &self.set_vars.path {
                    env::set_var("PATH", p);
                }
                env::set_var("HOME", &self.set_vars.home);
                env::set_var("SHELL", &self.set_vars.shell);
                env::set_var("USER", &self.set_vars.user);
                env::set_var("LOGNAME", &self.set_vars.logname);
            }
        }
    }

    fn is_var_valid(&self, var: &String) -> bool {
        self.safe_vars.contains(var)
            || var.starts_with("LC_")
            || (self.set_vars.path.is_none() && var == "PATH")
    }

    fn const_vars_to_vec(vars: &[&str]) -> Vec<String> {
        vars.iter().copied().map(str::to_string).collect()
    }
}

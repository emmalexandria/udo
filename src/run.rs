use std::{collections::HashSet, fmt::Display};

use crate::{
    authenticate::{AuthResult, authenticate_password},
    cache::Cache,
    config::Config,
    output::{self, prompt_password, wrong_password},
    run::{env::Env, process::run_process},
    user::{get_root_user, get_user, get_user_by_id},
};
use clap::ArgMatches;
use nix::{
    sys::stat::{Mode, stat},
    unistd::{Uid, User, getuid},
};
use std::env as std_env;
use std::process::exit;

pub mod env;
pub mod process;

#[derive(PartialEq, Eq, Debug, Clone, Copy, Hash, Default)]
pub struct ActionReqs {
    requires_auth: bool,
    requires_root: bool,
}

impl ActionReqs {
    pub fn auth() -> Self {
        Self::default().with_auth()
    }

    pub fn root() -> Self {
        Self::default().with_root()
    }

    pub fn with_auth(mut self) -> Self {
        self.requires_auth = true;
        self
    }

    pub fn with_root(mut self) -> Self {
        self.requires_auth = true;
        self.requires_root = true;
        self
    }
}

// We use repr(i32) here to allow for automatic ordering of the actions
#[repr(i32)]
#[derive(PartialEq, Eq, Debug, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ActionType {
    ClearCache = 0,
    Login = 1,
    Shell = 2,
    RunCommand = 3,
}

impl Display for ActionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ClearCache => "clear_cache",
            Self::Login => "login_shell",
            Self::Shell => "normal_shell",
            Self::RunCommand => "run_command",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action {
    a_type: ActionType,
    reqs: ActionReqs,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.a_type)
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((self.a_type as i32).cmp(&(other.a_type as i32)))
    }
}

impl Ord for Action {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Action {
    pub fn new(a_type: ActionType, reqs: ActionReqs) -> Self {
        Self { a_type, reqs }
    }

    pub fn do_action(&self, run: &mut Run, config: &Config) -> anyhow::Result<()> {
        match self.a_type {
            ActionType::ClearCache => {
                let ret = run.cache.clear();
                output::info(
                    format!("Cleared cache for user \"{}\"", run.user.name),
                    config.display.nerd,
                    None,
                );

                ret
            }
            ActionType::Login => todo!(),
            ActionType::Shell => todo!(),
            ActionType::RunCommand => {
                let env = Env::process_env(run, run.config.security.safe_path.as_ref());
                run_process(&run.command.clone().unwrap(), &env)?;
                Ok(())
            }
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Flag {
    NoCheck,
    PreserveVars,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    NoUser,
    IncorrectExePerms,
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {:?}", self.message, self.kind)
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn new<S: ToString>(kind: ErrorKind, message: S) -> Self {
        Self {
            kind,
            message: message.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Run<'a> {
    pub actions: Vec<Action>,
    pub flags: HashSet<Flag>,
    pub command: Option<Vec<String>>,
    pub cache: Cache,
    pub user: User,
    pub do_as: User,
    pub config: &'a Config,
}

impl<'a> Run<'a> {
    pub fn create(matches: &ArgMatches, config: &'a Config) -> Result<Self, Error> {
        let do_as_arg = matches
            .get_one::<String>("user")
            .expect("No user specified. This should not happen! Please file a bug report");
        let do_as = match get_user(do_as_arg) {
            Some(u) => u,
            None => return Err(Error::new(ErrorKind::NoUser, "Couldn't get target user")),
        };

        let user = get_user_by_id(getuid())
            .expect("Cannot get current user. This should not happen! Please file a bug report");
        let root = get_root_user();

        let cache = Cache::new(&user, &root);

        let mut actions = Self::get_actions(matches);
        let flags = Self::get_flags(matches);
        let mut command = None;

        if let Some(cmd) = matches.get_many::<String>("command") {
            command = Some(cmd.cloned().collect::<Vec<_>>());
            actions.push(Action::new(ActionType::RunCommand, ActionReqs::auth()));
        }

        Ok(Self {
            command,
            cache,
            do_as,
            user,
            actions,
            flags,
            config,
        })
    }

    fn get_actions(matches: &ArgMatches) -> Vec<Action> {
        let mut ret = Vec::new();
        if matches.get_flag("clear") {
            ret.push(Action::new(ActionType::ClearCache, ActionReqs::auth()));
        }
        if matches.get_flag("login") {
            ret.push(Action::new(ActionType::Shell, ActionReqs::auth()));
        }
        if matches.get_flag("shell") {
            ret.push(Action::new(ActionType::Login, ActionReqs::auth()));
        }

        ret
    }

    fn get_flags(matches: &ArgMatches) -> HashSet<Flag> {
        let mut ret = HashSet::new();
        if let Some(p) = matches.get_one::<bool>("preserve") {
            ret.insert(Flag::PreserveVars);
        }
        if let Some(p) = matches.get_one::<bool>("nocheck") {
            ret.insert(Flag::NoCheck);
        }

        ret
    }

    pub fn do_run(&mut self) -> anyhow::Result<()> {
        let mut actions = self.actions.clone();
        actions.sort();

        if !self.flags.contains(&Flag::NoCheck) && !check_perms(self.config) {
            exit(1);
        }

        // Actions which require the user logs in
        let requires_login = actions
            .iter()
            .filter(|a| a.reqs.requires_auth)
            .cloned()
            .collect::<Vec<_>>();

        // Actions which require the user logs in as root
        let requires_root = actions
            .iter()
            .filter(|a| a.reqs.requires_root)
            .cloned()
            .collect::<Vec<_>>();

        // Actions which require no authentication
        let rest = actions
            .into_iter()
            .filter(|a| !requires_root.contains(a) && !requires_login.contains(a))
            .collect::<Vec<_>>();

        let auth = self.login_user(self.config.security.tries);
        match auth {
            Ok(true) => self.after_auth(requires_login, requires_root)?,
            Ok(false) => output::info("Login failed", self.config.display.nerd, None),
            Err(e) => output::error_with_details(
                "Error while logging in",
                e,
                self.config.display.nerd,
                None,
            ),
        }

        Ok(())
    }

    fn login_user(&mut self, tries: usize) -> anyhow::Result<bool> {
        match self.cache.check_cache(self, self.config) {
            Ok(true) => return Ok(true),
            Ok(false) => {}
            Err(e) => output::error(
                format!("Failed to check cache ({e}). Requesting password"),
                self.config.display.nerd,
                None,
            ),
        }

        let password = prompt_password(self.config);
        if let Err(e) = &password {
            output::error(
                format!("Failed to display password prompt ({e}"),
                self.config.display.nerd,
                None,
            )
        }

        match authenticate_password(self, self.config, password.unwrap()) {
            AuthResult::Success => Ok(true),
            AuthResult::NotAuthenticated => {
                if tries > 1 {
                    wrong_password(self.config.display.nerd, tries - 1);
                    self.login_user(tries - 1)
                } else {
                    Ok(false)
                }
            }
            AuthResult::AuthenticationFailure(s) => {
                output::error_with_details(
                    "Authentication with PAM failed",
                    s,
                    self.config.display.nerd,
                    None,
                );
                Ok(false)
            }
        }
    }

    fn after_auth(&mut self, login: Vec<Action>, root: Vec<Action>) -> anyhow::Result<()> {
        for action in login {
            let res = action.do_action(self, self.config);

            if res.is_err() {
                output::error_with_details(
                    format!("Unable to perform {action}"),
                    res.err().unwrap(),
                    self.config.display.nerd,
                    None,
                );
            }
        }
        Ok(())
    }
}

/// Helper function to check if the executable has the correct permissions
fn check_perms(config: &Config) -> bool {
    let exe = std_env::current_exe().unwrap();
    let st = stat(&exe).unwrap();

    let mut valid = true;

    let owner = Uid::from_raw(st.st_uid);
    if !owner.is_root() {
        output::error("udo is not owned by root", config.display.nerd, None);
        valid = false;
    }

    let perms = Mode::from_bits_truncate(st.st_mode);
    if !perms.contains(Mode::S_ISUID) {
        output::error("udo does not have suid perms", config.display.nerd, None);
        valid = false;
    }

    valid
}

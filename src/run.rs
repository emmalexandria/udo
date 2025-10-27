use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::{
    cache::Cache,
    config::Config,
    run::{env::Env, process::run_process},
    user::{get_root_user, get_user, get_user_by_id},
};
use clap::ArgMatches;
use nix::unistd::{User, getuid};

pub mod env;
pub mod process;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CommandType {
    Command,
    Shell(bool),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Command {
    pub command: String,
    pub c_type: CommandType,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash, Default)]
pub struct ActionInfo {
    requires_auth: bool,
    requires_root: bool,
}

impl ActionInfo {
    pub fn with_auth(mut self) -> Self {
        self.requires_auth = true;
        self
    }

    pub fn with_root(mut self) -> Self {
        self.requires_root = true;
        self
    }
}

// We use repr(i32) here to allow for automatic ordering of the actions
#[repr(i32)]
#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord, Hash)]
pub enum RunAction {
    ClearCache = 0,
    Shell(bool) = 2,
    RunCommand = 3,
}

impl RunAction {
    pub fn do_action(run: &Run, config: &Config) {}
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Flag {
    NoCheck,
    PreserveVars,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    NoUser,
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
pub struct Run {
    pub actions: HashMap<RunAction, ActionInfo>,
    pub flags: HashSet<Flag>,
    pub cache: Cache,
    pub user: User,
    pub do_as: User,
}

impl Run {
    pub fn create(matches: &ArgMatches, config: &Config) -> Result<Self, Error> {
        let do_as_arg = matches
            .get_one::<String>("user")
            .expect("No user specified. This should not happen! Please file a bug report");
        let do_as = match get_user(&do_as_arg) {
            Some(u) => u,
            None => return Err(Error::new(ErrorKind::NoUser, "Couldn't get target user")),
        };

        let user = get_user_by_id(getuid())
            .expect("Cannot get current user. This should not happen! Please file a bug report");
        let root = get_root_user();

        let cache = Cache::new(&user, &root);

        let actions = Self::get_actions(matches);
        let flags = Self::get_flags(matches);

        Ok(Self {
            cache,
            do_as,
            user,
            actions,
            flags,
        })
    }

    fn get_actions(matches: &ArgMatches) -> HashMap<RunAction, ActionInfo> {
        let mut ret = HashMap::new();
        if matches.get_flag("clear") {
            ret.insert(RunAction::ClearCache, ActionInfo::default().with_auth());
        }
        if matches.get_flag("login") {
            ret.insert(RunAction::Shell(true), ActionInfo::default().with_auth());
        }
        if matches.get_flag("shell") {
            ret.insert(RunAction::Shell(false), ActionInfo::default());
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

    pub fn do_run(&self) -> Result<()> {
        let actions = self.actions.iter()

    }
}

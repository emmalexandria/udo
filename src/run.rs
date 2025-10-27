use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};

use crate::{
    cache::Cache,
    config::Config,
    login_user,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Action {
    a_type: ActionType,
    reqs: ActionReqs,
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((other.a_type as i32).cmp(&(self.a_type as i32)))
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
            ActionType::ClearCache => run.cache.clear(),
            ActionType::Login => todo!(),
            ActionType::Shell => todo!(),
            ActionType::RunCommand => todo!(),
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
    IncorrectExePerms
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
        if let Some(cmd) = matches.get_many("command") {}

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

    pub fn do_run(&mut self) -> Result<(), Error> {
        let mut actions = self.actions.clone();
        actions.sort();

        if !self.flags.contains(&Flag::NoCheck) {

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

        rest.iter().for_each(|a| {a.do_action(self, &self.config);});

        let auth = login_user(self, &self.config, self.config.security.tries);
        match auth {
            Ok(true) => 
        }

        Ok(())
    }

    fn after_auth() -> Result<(), Error> {

    }
}

use std::collections::HashMap;

use crate::{
    cache::Cache,
    config::Config,
    run::{env::Env, process::run_process},
};
use anyhow::Result;
use clap::ArgMatches;
use nix::unistd::User;

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

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ActionInfo {
    requires_auth: bool,
    requires_root: bool,
}

// We use repr(i32) here to allow for automatic ordering of the actions
#[repr(i32)]
#[derive(PartialEq, Eq, Debug, Clone, PartialOrd, Ord)]
pub enum RunAction {
    ClearCache = 0,
    OpenShell = 1,
    RunCommand = 2,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Flag {
    NoCheck,
    LoginShell,
}

impl RunAction {
    pub fn do_action(config: &Config) {}
}

#[derive(Debug, Clone)]
struct UdoRun {
    pub actions: HashMap<RunAction, ActionInfo>,
    pub preserve_vars: bool,
    pub clear_cache: bool,
    pub cache: Cache,
    pub user: User,
    pub do_as: User,
}

impl UdoRun {
    pub fn create(matches: &ArgMatches, config: &Config) -> Self {
        let actions = HashMap::new();

        Self { actions }
    }

    pub fn do_run(&self) -> Result<()> {
        Ok(())
    }
}

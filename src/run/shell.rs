use std::path::PathBuf;

use anyhow::Result;
use nix::unistd::User;

use crate::run::env::Env;

pub fn get_shell_cmd(user: &User) -> PathBuf {
    user.shell.clone()
}

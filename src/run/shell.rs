use std::path::PathBuf;

use nix::unistd::User;

pub fn get_shell_cmd(user: &User) -> PathBuf {
    user.shell.clone()
}

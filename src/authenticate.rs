mod pam;

use std::process::Command;

use anyhow::Result;
use nix::unistd::{Gid, Group, User, getuid};

use crate::{authenticate::pam::authenticate_user, config::Config, output};

pub enum AuthResult {
    NotAuthorised,
    AuthenticationFailure,
    Success,
}

pub fn authenticate(password: String, config: &Config) -> Result<AuthResult> {
    let uid = getuid();
    let user_opt = User::from_uid(uid)?;
    if user_opt.is_none() {
        output::error(
            format!("Failed to get user for uid {}", uid.as_raw()),
            config.display.nerd,
        );
    }

    let user = user_opt.unwrap();

    if !is_authorised(&user)? {
        return Ok(AuthResult::NotAuthorised);
    }

    let auth = authenticate_user(&user.name, &password, "udo");
    if auth.is_err() || auth.is_ok_and(|v| !v) {
        return Ok(AuthResult::AuthenticationFailure);
    }

    Ok(AuthResult::Success)
}

fn is_authorised(user: &User) -> Result<bool> {
    let groups = vec![
        Group::from_name("wheel")?,
        Group::from_name("admin")?,
        Group::from_name("root")?,
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let supp: Vec<Gid> = get_supplemental_groups(user)?
        .iter()
        .map(|g| g.gid)
        .collect();

    // Check primary group
    for group in groups {
        if group.gid == user.gid || supp.contains(&group.gid) {
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(target_os = "macos")]
fn get_supplemental_groups(user: &User) -> Result<Vec<Group>> {
    let output = Command::new("id").args(["-Gn", &user.name]).output()?;
    Ok(String::from_utf8(output.stdout)?
        .split(' ')
        .flat_map(Group::from_name)
        .flatten()
        .collect::<Vec<_>>())
}

#[cfg(target_os = "linux")]
fn get_supplemental_groups(user: &User) -> Result<Vec<Gid>> {
    use nix::unistd::getgroups;

    Ok(getgroups().iter().flat_map(Group::from_gid).flatten())
}

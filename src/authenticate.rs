mod pam;

use std::process::Command;

use anyhow::Result;
use nix::unistd::{Group, User, gethostname};
use serde::{Deserialize, Serialize};

use crate::{authenticate::pam::authenticate_user, config::Config};

#[derive(Debug, Clone, Default)]
pub enum ActionValue {
    #[default]
    Any,
    Value(String),
}

impl From<String> for ActionValue {
    fn from(value: String) -> Self {
        match value.as_str() {
            "any" => Self::Any,
            _ => Self::Value(value),
        }
    }
}

impl From<&String> for ActionValue {
    fn from(value: &String) -> Self {
        Self::from(value.clone())
    }
}

impl From<&str> for ActionValue {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

#[derive(Debug, Clone, Default)]
pub struct Action {
    pub command: ActionValue,
    pub host: ActionValue,
    pub run_as: ActionValue,
}

impl Action {
    fn from_rule(rule: &Rule, user: &User) -> Self {
        Self {
            command: (&rule.command).into(),
            host: (&rule.host).into(),
            run_as: (&rule.user).into(),
        }
    }

    pub fn contains(&self, other: &Self) -> bool {
        let cmd = match &self.command {
            ActionValue::Any => true,
            ActionValue::Value(v) => {
                let v = v.clone();
                matches!(&other.command, ActionValue::Value(v))
            }
        };

        let host = match &self.host {
            ActionValue::Any => true,
            ActionValue::Value(v) => {
                let v = v.clone();
                matches!(&other.host, ActionValue::Value(v))
            }
        };

        let run_as = match &self.run_as {
            ActionValue::Any => true,
            ActionValue::Value(v) => {
                let v = v.clone();
                matches!(&other.run_as, ActionValue::Value(v))
            }
        };

        cmd && host && run_as
    }
}

pub enum AuthResult {
    NotAuthorised,
    AuthenticationFailure,
    Success,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rule {
    target: String,
    host: String,
    user: String,
    command: String,
}

impl Rule {
    pub fn new(target: String, host: String, user: String, command: String) -> Self {
        Self {
            target,
            host,
            user,
            command,
        }
    }

    pub fn check_authorization(&self, user: &User) -> Result<bool> {
        if self.target == user.name {
            return Ok(true);
        }

        if self.target.starts_with("%")
            && let Some(group) = Group::from_name(&self.target[1..])?
        {
            if group.gid == user.gid {
                return Ok(true);
            }
            let supp_groups = get_supplemental_groups(user)?
                .iter()
                .map(|g| g.gid)
                .collect::<Vec<_>>();
            if supp_groups.contains(&group.gid) {
                return Ok(true);
            }
        }

        Ok(false)
    }
}

pub fn authenticate(
    user: &User,
    password: String,
    config: &Config,
    do_as: &User,
    command: &str,
) -> Result<AuthResult> {
    let allowed_actions = get_matching_rules(user, config)
        .into_iter()
        .map(|r| Action::from_rule(&r, user))
        .collect::<Vec<_>>();

    let hostname = gethostname()?;

    let action = Action {
        command: ActionValue::from(command),
        host: ActionValue::from(hostname.to_string_lossy().to_string()),
        run_as: ActionValue::from(do_as.name.clone()),
    };

    let matching = allowed_actions.iter().find(|a| a.contains(&action));

    if matching.is_some() {
        let auth = authenticate_user(&user.name, &password, "udo");
        match auth {
            Ok(false) => return Ok(AuthResult::AuthenticationFailure),
            Err(e) => return Err(e),
            _ => {}
        }

        if auth.is_ok_and(|v| !v) {
            return Ok(AuthResult::AuthenticationFailure);
        }
    } else {
        return Ok(AuthResult::NotAuthorised);
    }

    Ok(AuthResult::Success)
}

fn get_matching_rules(user: &User, config: &Config) -> Vec<Rule> {
    config
        .rules
        .iter()
        .filter(|&r| r.check_authorization(user).is_ok_and(|v| v))
        .cloned()
        .collect()
}

fn is_authorised<'a>(user: &User, config: &'a Config) -> Option<&'a Rule> {
    config
        .rules
        .iter()
        .find(|r| matches!(r.check_authorization(user), Ok(true)))
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

mod pam;

use std::process::Command;

use anyhow::Result;
use nix::unistd::{Group, User, gethostname};
use serde::{Deserialize, Serialize};

use crate::{
    authenticate::pam::{AuthErrorKind, authenticate_user},
    config::Config,
    run::Run,
};

/// ActionValue represents a value within [Action]. It can either be Any, or a specific Value.
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

/// Action is the internal representation of a [Rule]. It represents the commands the user is
/// allowed to run, the hostname they can run them as, and the user they can run them as
///
/// It is composed of [ActionValue]
#[derive(Debug, Clone, Default)]
pub struct Action {
    pub command: ActionValue,
    pub host: Option<ActionValue>,
    pub do_as: ActionValue,
}

impl Action {
    fn from_rule(rule: &Rule) -> Self {
        Self {
            command: (&rule.command).into(),
            host: Some((&rule.host).into()),
            do_as: (&rule.user).into(),
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

        // Because getting the hostname is a fallible operation, we support cases where we couldn't get the hostname
        let host;
        // If we couldn't get the hostname then
        if other.host.is_none() {
            host = match self.host {
                // If this hostname is any, we allow it
                Some(ActionValue::Any) => true,
                // Otherwise we don't
                None | Some(ActionValue::Value(_)) => false,
            }
        } else {
            // If we could get the hostname then
            let h = other.host.as_ref().unwrap();
            host = match &self.host {
                // Check if this action allows any
                Some(ActionValue::Any) => true,
                // Check if this action's hostname allows others
                Some(h) => true,
                // If this action has no hostname (shouldn't happen!) don't allow
                None => false,
            };
        }

        let run_as = match &self.do_as {
            ActionValue::Any => true,
            ActionValue::Value(v) => {
                let v = v.clone();
                matches!(&other.do_as, ActionValue::Value(v))
            }
        };

        cmd && host && run_as
    }
}

pub enum AuthResult {
    AuthenticationFailure(String),
    NotAuthenticated,
    Success,
}

/// Rule is used in the configuration file, which is why it is a distinct type from [Action].
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

    /// Checks if the rule applies to the current user
    pub fn applies_to(&self, user: &User) -> Result<bool> {
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

/// Attempts to authenticate the user with the given password
pub fn authenticate_password(run: &Run, config: &Config, password: String) -> AuthResult {
    match authenticate_user(&run.user.name, &password, "udo") {
        Ok(_) => AuthResult::Success,
        Err(e) => match e.kind {
            AuthErrorKind::InvalidInput | AuthErrorKind::StartFailure => {
                AuthResult::AuthenticationFailure(e.to_string())
            }
            AuthErrorKind::AuthenticateFailure | AuthErrorKind::ValidationFailure => {
                AuthResult::NotAuthenticated
            }
        },
    }
}

/// Check if the user is allowed to run the action they are trying to
///
/// If the hostname cannot be retrieved, it will allow the action only if
/// there is a [Rule] with hostname ANY
pub fn check_action_auth(run: &Run, config: &Config) -> bool {
    // Get the rules the user is authorised to run
    let applicable_rules = get_matching_rules(&run.user, config);
    let allowed_actions = applicable_rules
        .iter()
        .map(Action::from_rule)
        .collect::<Vec<_>>();

    // Get the current hostname. If we can't get it, only allow the action to proceed if there is
    // an allowed action with hostname rule any
    let hostname = gethostname().ok();

    if hostname.is_none()
        && allowed_actions
            .iter()
            .any(|a| !matches!(a.host, Some(ActionValue::Any)))
    {
        return false;
    }

    // Create the action of what the user is trying to do
    let action = Action {
        command: ActionValue::from(&run.command.as_ref().unwrap()[0]),
        host: hostname.map(|h| h.to_string_lossy().to_string().into()),
        do_as: ActionValue::from(run.do_as.name.clone()),
    };

    // Filter the allowed actions for ones which contain the action the user is attempting
    let matching_actions = allowed_actions
        .iter()
        .filter(|a| a.contains(&action))
        .collect::<Vec<_>>();

    !matching_actions.is_empty()
}

/// Get the rules which apply to the current user
fn get_matching_rules(user: &User, config: &Config) -> Vec<Rule> {
    config
        .rules
        .iter()
        .filter(|&r| r.applies_to(user).is_ok_and(|v| v))
        .cloned()
        .collect()
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

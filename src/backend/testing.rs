use std::{collections::HashMap, env};

use nix::unistd::{Gid, Uid};

use crate::backend::{Backend, Error, ErrorKind, InitBackend};

#[derive(PartialEq, Eq, Clone)]
pub struct TestProcess {
    pub name: String,
    pub args: Vec<String>,
}

/// This is a [Backend] used for testing udo. It in no way fully simulates a Unix system,
/// but it aims to simulate *enough* to verify that udo has the expected behaviour
#[derive(PartialEq, Eq, Clone)]
pub struct TestBackend {
    /// Stores the group id
    gid: Gid,
    /// Stores the real uid
    uid: Uid,
    /// Stores the effective uid
    euid: Uid,
    /// Stores the saved-set uid, necessary for switching the euid
    suid: Uid,
    env: HashMap<String, String>,
    process: TestProcess,
}

impl InitBackend for TestBackend {
    fn new() -> Self {
        // Pretend we're a process run by a normal user with suid and owned by root
        // Choose 512 because it's a nice round number
        Self {
            uid: Uid::from_raw(512),
            gid: Gid::from_raw(512),
            euid: Uid::from_raw(0),
            suid: Uid::from_raw(0),
            env: HashMap::new(),
            process: TestProcess {
                name: "udo".to_string(),
                args: vec![],
            },
        }
    }
}

impl Backend for TestBackend {
    fn getuid(&self) -> nix::unistd::Uid {
        self.uid
    }

    fn setuid(&mut self, uid: nix::unistd::Uid) -> super::Result<()> {
        // In this function we assume the executable always has the suid permissions bit for
        // testing purposes
        if uid != self.suid {
            return Err(Error::new(ErrorKind::UidSet, "UID does not match SUID"));
        }
        self.uid = uid;
        self.euid = uid;
        self.suid = uid;
        Ok(())
    }

    fn geteuid(&self) -> nix::unistd::Uid {
        self.euid
    }

    fn seteuid(&mut self, uid: nix::unistd::Uid) -> super::Result<()> {
        // Check if the saved set uid or the actual uid matches the UID attempting to be set
        if self.suid == uid || self.uid == uid {
            self.suid = self.euid;
            self.euid = uid;
        } else {
            return Err(Error::new(
                ErrorKind::EuidSet,
                "EUID not present in UID or SUID",
            ));
        }
        Ok(())
    }

    fn getgid(&self) -> nix::unistd::Gid {
        self.gid
    }

    fn setgid(&mut self, gid: nix::unistd::Gid) -> super::Result<()> {
        self.gid = gid;
        Ok(())
    }

    fn execvp(&mut self, process: &str, args: &[&str]) -> super::Result<()> {
        self.process = TestProcess {
            name: process.to_string(),
            args: args
                .iter()
                .map(|s| s.as_ref())
                .map(str::to_string)
                .collect(),
        };

        Ok(())
    }

    fn get_env(&self, name: &str) -> super::Result<String> {
        let env = self.env.get(name);
        let res = env.ok_or(Error::new(
            ErrorKind::Env,
            "Failed to read environment variable. Is it set?",
        ))?;

        Ok(res.clone())
    }

    // In our simulated env, this call can never fail
    unsafe fn set_env(&mut self, name: &str, value: &str) {
        if self.env.contains_key(name) {
            self.env.remove(name);
        }
        self.env.insert(name.to_string(), value.to_string());
    }

    fn vars(&self) -> Vec<(String, String)> {
        self.env
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

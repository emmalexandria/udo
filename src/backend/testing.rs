use std::collections::HashMap;

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
pub struct TestingBackend {
    gid: Gid,
    uid: Uid,
    euid: Uid,
    suid: Uid,
    env: HashMap<String, String>,
    process: TestProcess,
}

impl InitBackend for TestingBackend {
    fn new() -> Self {
        // Pretend we're a process run by a normal user with suid and owned by root
        // Choose 512 because it's a nice round number
        Self {
            uid: Uid::from_raw(512),
            gid: Gid::from_raw(0),
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

impl Backend for TestingBackend {
    fn getuid(&self) -> nix::unistd::Uid {
        self.uid
    }

    fn setuid(&mut self, uid: nix::unistd::Uid) -> super::Result<()> {
        self.uid = uid;
        Ok(())
    }

    fn geteuid(&self) -> nix::unistd::Uid {
        self.euid
    }

    fn seteuid(&mut self, uid: nix::unistd::Uid) -> super::Result<()> {
        self.euid = uid;
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
}

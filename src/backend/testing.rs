use std::{collections::HashMap, env};

use nix::unistd::{Gid, Uid};

use crate::backend::{Backend, Error, ErrorKind};

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
    /// Stores the effective gid,
    egid: Gid,
    /// Stores the saved-set gid, necessary for switching egids.
    sgid: Gid,
    /// Stores the real uid
    uid: Uid,
    /// Stores the effective uid
    euid: Uid,
    /// Stores the saved-set uid, necessary for switching the euid
    suid: Uid,
    env: HashMap<String, String>,
    process: TestProcess,
}

impl Default for TestBackend {
    fn default() -> Self {
        Self {
            // Nice round uid of 512
            uid: Uid::from_raw(512),
            // Same for the gid
            gid: Gid::from_raw(512),
            // We dont run with sgid perms, so its the same
            egid: Gid::from_raw(512),
            // Therefore, so is sgid
            sgid: Gid::from_raw(512),
            // We do run with suid perms, so thats root
            euid: Uid::from_raw(0),
            // And therefore so is suid
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

        // If uid is not the suid, not the uid, and we aren't root, we can't setuid
        if uid != self.suid && uid != self.uid && !self.is_root() {
            return Err(Error::new(
                ErrorKind::UidSet,
                "UID does not match SUID, and EUID is not root",
            ));
        }

        // Setting the actual UID also sets the EUID and the SUID.
        self.uid = uid;
        self.euid = uid;
        self.suid = uid;
        Ok(())
    }

    fn geteuid(&self) -> nix::unistd::Uid {
        self.euid
    }

    fn seteuid(&mut self, uid: nix::unistd::Uid) -> super::Result<()> {
        // Check if the saved set uid or the actual uid matches the UID attempting to be set or the
        // process is root
        if self.suid == uid || self.uid == uid || self.is_root() {
            self.suid = self.euid;
            self.euid = uid;
        } else {
            return Err(Error::new(
                ErrorKind::EuidSet,
                "EUID not present in UID or SUID, process is not root",
            ));
        }
        Ok(())
    }

    fn getgid(&self) -> nix::unistd::Gid {
        self.gid
    }

    fn setgid(&mut self, gid: nix::unistd::Gid) -> super::Result<()> {
        if self.gid == gid || self.sgid == gid || self.is_root() {
            self.gid = gid;
        } else {
            return Err(Error::new(
                ErrorKind::GidSet,
                "GID not present in GID or SGID, process is not root",
            ));
        }

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

    fn is_root(&self) -> bool {
        self.uid.is_root() || self.euid.is_root()
    }
}

#[cfg(test)]
mod tests {
    use nix::unistd::Uid;

    use crate::backend::{Backend, testing::TestBackend};

    #[test]
    fn set_euid() {
        let mut backend = TestBackend::default();
        // We should be able to seteuid to the uid
        backend.seteuid(backend.getuid()).unwrap();
        // And then switch back to root because its in suid
        backend.seteuid(Uid::from_raw(0)).unwrap();
    }

    #[test]
    fn set_uid() {
        let mut backend = TestBackend::default();

        // As soon as we setuid to the uid, we should no longer have permissions to switch out our
        // uid
        backend.setuid(backend.getuid()).unwrap();
        assert!(backend.seteuid(Uid::from_raw(0)).is_err())
    }

    // This test serves to both test the `is_root` function and also
    // provides a more broad test of uid behaviour
    #[test]
    fn is_root() {
        let mut backend = TestBackend::default();
        // Store the initial value of uid for use later
        let uid = backend.getuid();
        assert!(backend.is_root());

        backend.seteuid(backend.getuid()).unwrap();
        assert!(!backend.is_root());

        backend.seteuid(Uid::from_raw(0)).unwrap();
        assert!(backend.is_root());

        backend.setuid(backend.geteuid()).unwrap();
        assert!(backend.is_root());

        backend.setuid(uid).unwrap();
        assert!(!backend.is_root());
    }
}

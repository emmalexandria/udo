use std::{cell::RefCell, collections::HashMap, fs};

use nix::{
    fcntl::OFlag,
    sys::stat::Mode,
    unistd::{Gid, Uid},
};

use crate::backend::{
    Error, ErrorKind, ProcessManager, Result, Syscalls,
    vfs::{VFile, VirtualFS},
};

/// This is a [Backend] used for testing udo. It in no way fully simulates a Unix system,
/// but it aims to simulate *enough* to verify that udo has the expected behaviour
#[derive(PartialEq, Eq, Clone)]
pub struct TestBackend {
    /// Stores the group id
    gid: RefCell<Gid>,
    /// Stores the effective gid,
    egid: RefCell<Gid>,
    /// Stores the saved-set gid, necessary for switching egids.
    sgid: RefCell<Gid>,
    /// Stores the real uid
    uid: RefCell<Uid>,
    /// Stores the effective uid
    euid: RefCell<Uid>,
    /// Stores the saved-set uid, necessary for switching the euid
    suid: RefCell<Uid>,
    /// Stores the original user UID, for use in elevate and restore functions
    original: Uid,
    target: Uid,
    env: HashMap<String, String>,
    /// Stores an incredibly simplified representation of files (path -> content)
    /// We don't worry about permissions here, it's simply too much of a PITA.
    vfs: VirtualFS,
}

impl Default for TestBackend {
    fn default() -> Self {
        let user = Uid::from_raw(512);
        let group = Gid::from_raw(512);
        let root = Uid::from_raw(0);

        let vfs = VirtualFS::new().with_config().unwrap();

        Self {
            // Nice round uid of 512
            uid: RefCell::new(user),
            // We do run with suid perms, so thats root
            euid: RefCell::new(root),
            // And therefore so is suid
            suid: RefCell::new(root),
            // Same for the gid
            gid: RefCell::new(group),
            // We dont run with sgid perms, so its the same
            egid: RefCell::new(group),
            // Therefore, so is sgid
            sgid: RefCell::new(group),
            // The original user is always the user running the program
            original: user,
            // We default the target user to root for testing purposes
            target: root,
            env: HashMap::new(),
            vfs,
        }
    }
}

impl ProcessManager for TestBackend {
    fn getuid(&self) -> nix::unistd::Uid {
        *self.uid.borrow()
    }

    fn setuid(&self, uid: nix::unistd::Uid) -> Result<()> {
        // In this function we assume the executable always has the suid permissions bit for
        // testing purposes

        // If uid is not the suid, not the uid, and we aren't root, we can't setuid
        if uid != *self.suid.borrow() && uid != *self.uid.borrow() && !self.is_root() {
            return Err(Error::new(
                ErrorKind::UidSet,
                "UID does not match SUID, and EUID is not root",
            ));
        }

        // Setting the actual UID also sets the EUID and the SUID.
        *self.uid.borrow_mut() = uid;
        *self.euid.borrow_mut() = uid;
        *self.suid.borrow_mut() = uid;
        Ok(())
    }

    fn geteuid(&self) -> nix::unistd::Uid {
        *self.euid.borrow()
    }

    fn seteuid(&self, uid: nix::unistd::Uid) -> Result<()> {
        // Check if the saved set uid or the actual uid matches the UID attempting to be set or the
        // process is root
        if *self.suid.borrow() == uid || *self.uid.borrow() == uid || self.is_root() {
            *self.suid.borrow_mut() = *self.euid.borrow();
            *self.euid.borrow_mut() = uid;
        } else {
            return Err(Error::new(
                ErrorKind::EuidSet,
                "EUID not present in UID or SUID, process is not root",
            ));
        }
        Ok(())
    }

    fn getgid(&self) -> nix::unistd::Gid {
        *self.gid.borrow()
    }

    fn setgid(&self, gid: nix::unistd::Gid) -> Result<()> {
        if *self.gid.borrow() == gid || *self.sgid.borrow() == gid || self.is_root() {
            *self.gid.borrow_mut() = gid;
        } else {
            return Err(Error::new(
                ErrorKind::GidSet,
                "GID not present in GID or SGID, process is not root",
            ));
        }

        Ok(())
    }

    // In our test backend, execvp doesn't actually have to do anything. Always returns Ok(())
    // without executing any code
    fn execvp(&self, process: &str, args: &[&str]) -> Result<()> {
        Ok(())
    }

    fn get_var(&self, name: &str) -> super::Result<String> {
        let env = self.env.get(name);
        let res = env.ok_or(Error::new(
            ErrorKind::Env,
            "Failed to read environment variable. Is it set?",
        ))?;

        Ok(res.clone())
    }

    // In our simulated Unix environment, this call can never fail. It's still unsafe because that's
    // what we define in the trait, but it doesn't have to be.
    unsafe fn set_var(&mut self, name: &str, value: &str) {
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

    unsafe fn remove_var(&mut self, name: &str) {
        self.env.remove(name);
    }

    fn is_root(&self) -> bool {
        self.uid.borrow().is_root() || self.euid.borrow().is_root()
    }

    fn elevate(&self) -> Result<()> {
        self.seteuid(Uid::from_raw(0))
    }

    fn restore(&self) -> Result<()> {
        self.seteuid(self.original)
    }

    fn switch_final(&self) -> Result<()> {
        self.elevate()?;
        self.setuid(self.target)
    }
}

impl Syscalls for TestBackend {
    fn open(&self, path: &std::path::Path, flags: OFlag, mode: Mode) -> Result<i32> {
        self.vfs.open(path, flags)
    }

    fn read(&self, fd: i32, buf: &mut [u8]) -> Result<usize> {
        self.vfs.read(fd, buf)
    }
}

#[cfg(test)]
mod tests {
    use nix::unistd::Uid;

    use crate::backend::{ProcessManager, testing::TestBackend};

    #[test]
    fn set_euid() {
        let backend = TestBackend::default();
        // We should be able to seteuid to the uid
        backend.seteuid(backend.getuid()).unwrap();
        // And then switch back to root because its in suid
        backend.seteuid(Uid::from_raw(0)).unwrap();
    }

    #[test]
    fn set_uid() {
        let backend = TestBackend::default();

        // As soon as we setuid to the uid, we should no longer have permissions to switch out our
        // uid
        backend.setuid(backend.getuid()).unwrap();
        assert!(backend.seteuid(Uid::from_raw(0)).is_err())
    }

    // This test serves to both test the `is_root` function and also
    // provides a more broad test of uid behaviour
    #[test]
    fn is_root() {
        let backend = TestBackend::default();
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

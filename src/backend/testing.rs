use nix::unistd::{Gid, Uid};

use crate::backend::Backend;

/// This is a [Backend] used for testing udo. It in no way fully simulates a Unix system,
/// but it aims to simulate *enough* to verify that udo has the expected behaviour
pub struct TestingBackend {
    gid: Gid,
    uid: Uid,
    euid: Uid,
    suid: Uid,
}

impl Backend for TestingBackend {
    fn new() -> Self {
        // Pretend we're a process run by a normal user with suid and owned by root
        // Choose 512 because it's a nice round number
        Self {
            uid: Uid::from_raw(512),
            gid: Gid::from_raw(0),
            euid: Uid::from_raw(0),
            suid: Uid::from_raw(0),
        }
    }

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

    fn execvp<S: AsRef<str>>(&self, process: S, args: &[S]) -> super::Result<()> {}
}

use std::io;

use nix::unistd::{Gid, Uid};

use crate::backend::Backend;

/// This is a [Backend] used for running udo. It interacts directly with the system
/// it is running on, and all actions performed on it reflect directly on the system
pub struct SystemBackend {}

impl Backend for SystemBackend {
    fn getuid() -> Uid {
        nix::unistd::getuid()
    }

    fn setuid(uid: Uid) -> io::Result<()> {
        todo!()
    }

    fn geteuid() -> Uid {
        nix::unistd::geteuid()
    }

    fn seteuid(uid: Uid) -> io::Result<()> {
        todo!()
    }

    fn getgid() -> Gid {
        nix::unistd::getgid()
    }

    fn setgid(uid: Gid) -> io::Result<()> {
        todo!()
    }
}

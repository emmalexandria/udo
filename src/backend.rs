/*! This file exists to enable testing the functioning of udo.
*
* There are too many variables to test
* `udo` on each individual system, so instead we implement backends. One interacts with the real
* system and is used at runtime, and the other creates a very basic fake of a unix system for testing.
*/

use std::io;

use nix::unistd::{Gid, Uid};

pub trait Backend {
    fn getuid() -> Uid;
    fn setuid(uid: Uid) -> io::Result<()>;

    fn geteuid() -> Uid;
    fn seteuid(uid: Uid) -> io::Result<()>;

    fn getgid() -> Gid;
    fn setgid(uid: Gid) -> io::Result<()>;
}

/// This is a [Backend] used for testing udo. It in no way fully simulates a Unix system,
/// but it aims to simulate *enough* to verify that udo has the expected behaviour
pub struct TestingBackend {}

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

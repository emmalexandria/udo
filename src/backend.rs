/*! This file exists to enable testing the functioning of udo.
*
* There are too many variables to test
* `udo` on each individual system, so instead we implement backends. One interacts with the real
* system and is used at runtime, and the other creates a very basic fake of a unix system for testing.
*/

pub mod system;
#[cfg(test)]
pub mod testing;
#[cfg(test)]
pub mod vfs;

use std::{fmt::Display, path::Path};

use nix::{
    errno::Errno,
    fcntl::OFlag,
    sys::stat::Mode,
    unistd::{Gid, Uid},
};

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UidSet,
    EuidSet,
    GidSet,
    InvalidString,
    DoesNotExist,
    Exec,
    Env,
    System(Errno),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UidSet => "UID_SET",
            Self::EuidSet => "EUID_SET",
            Self::GidSet => "GID_SET",
            Self::InvalidString => "INVALID_STRING",
            Self::Exec => "EXEC",
            Self::Env => "ENV",
            Self::DoesNotExist => "DOES_NOT_EXIST",
            Self::System(e) => e.desc(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.message, self.kind)
    }
}

impl Error {
    pub fn new<S: ToString>(kind: ErrorKind, message: S) -> Self {
        Self {
            kind,
            message: message.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Syscalls {
    fn open(&self, path: &Path, flags: OFlag, mode: Mode) -> Result<i32>;

    fn read(&self, fd: i32, buf: &mut [u8]) -> Result<usize>;
}

pub trait ProcessManager {
    fn getuid(&self) -> Uid;
    /// Sets the process uid, euid, and suid
    fn setuid(&self, uid: Uid) -> Result<()>;

    fn geteuid(&self) -> Uid;
    /// Sets the process euid, inline with Unix permissions
    fn seteuid(&self, uid: Uid) -> Result<()>;

    fn getgid(&self) -> Gid;
    fn setgid(&self, uid: Gid) -> Result<()>;

    fn execvp(&self, process: &str, args: &[&str]) -> Result<()>;

    /// Get an environment variable
    fn get_var(&self, name: &str) -> Result<String>;
    /// Set an environment variable
    unsafe fn set_var(&mut self, name: &str, value: &str);
    /// Remove a variable
    unsafe fn remove_var(&mut self, name: &str);
    /// Get all environment variables as key value pairs
    fn vars(&self) -> Vec<(String, String)>;

    /// Elevate to root for privileged operations
    fn elevate(&self) -> Result<()>;

    /// Restore to the original user
    fn restore(&self) -> Result<()>;

    /// Make the final switch (setuid) to the target user
    fn switch_final(&self) -> Result<()>;

    /// Return if the process is currently "effectively" root, i.e. euid == 0 || uid == 0
    fn is_root(&self) -> bool;
}

pub trait Backend: ProcessManager + Syscalls {}

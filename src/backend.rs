/*! This file exists to enable testing the functioning of udo.
*
* There are too many variables to test
* `udo` on each individual system, so instead we implement backends. One interacts with the real
* system and is used at runtime, and the other creates a very basic fake of a unix system for testing.
*/

pub mod system;
pub mod testing;

use std::fmt::Display;

use nix::unistd::{Gid, Uid};

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UidSet,
    EuidSet,
    GidSet,
    InvalidString,
    Exec,
    Env,
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

pub trait Backend {
    fn getuid(&self) -> Uid;
    /// Sets the process uid, euid, and suid
    fn setuid(&mut self, uid: Uid) -> Result<()>;

    fn geteuid(&self) -> Uid;
    /// Sets the process euid, inline with Unix permissions
    fn seteuid(&mut self, uid: Uid) -> Result<()>;

    fn getgid(&self) -> Gid;
    fn setgid(&mut self, uid: Gid) -> Result<()>;

    fn execvp(&mut self, process: &str, args: &[&str]) -> Result<()>;

    /// Get an environment variable
    fn get_var(&self, name: &str) -> Result<String>;
    /// Set an environment variable
    unsafe fn set_var(&mut self, name: &str, value: &str);
    /// Remove a variable
    unsafe fn remove_var(&mut self, name: &str);
    /// Get all environment variables as key value pairs
    fn vars(&self) -> Vec<(String, String)>;

    /// Elevate to root for privileged operations
    fn elevate(&mut self) -> Result<()>;

    /// Restore to the original user
    fn restore(&mut self) -> Result<()>;

    /// Make the final switch (setuid) to the target user
    fn switch_final(&mut self) -> Result<()>;

    /// Return if the process is currently "effectively" root, i.e. euid == 0 || uid == 0
    fn is_root(&self) -> bool;
}

use std::{
    ffi::{CStr, CString},
    io,
};

use nix::unistd::{Gid, Uid, execvp, seteuid, setgid, setuid};

use crate::backend::{Backend, Error, ErrorKind, Result};

/// This is a [Backend] used for running udo. It interacts directly with the system
/// it is running on, and all actions performed on it reflect directly on the system
pub struct SystemBackend {}

impl Backend for SystemBackend {
    fn new() -> Self {
        Self {}
    }

    fn getuid(&self) -> Uid {
        nix::unistd::getuid()
    }

    fn setuid(&mut self, uid: Uid) -> Result<()> {
        setuid(uid).map_err(|e| Error::new(ErrorKind::UidSet, "Failed to set uid"))
    }

    fn geteuid(&self) -> Uid {
        nix::unistd::geteuid()
    }

    fn seteuid(&mut self, uid: Uid) -> Result<()> {
        seteuid(uid).map_err(|e| Error::new(ErrorKind::EuidSet, "Failed to set euid"))
    }

    fn getgid(&self) -> Gid {
        nix::unistd::getgid()
    }

    fn setgid(&mut self, gid: Gid) -> Result<()> {
        setgid(gid).map_err(|e| Error::new(ErrorKind::GidSet, "Failed to set gid"))
    }

    fn execvp<S: AsRef<str>>(&mut self, process: S, args: &[S]) -> Result<()> {
        let process = CString::new(process.as_ref()).map_err(|_| {
            Error::new(
                ErrorKind::InvalidString,
                "Failed to convert command to CString",
            )
        })?;

        // Flatten args for now, add error handling in future
        let args = args
            .iter()
            .flat_map(|s| {
                CString::new(s.as_ref()).map_err(|_| {
                    Error::new(ErrorKind::InvalidString, "Failed to convert arg to CString")
                })
            })
            .collect::<Vec<_>>();

        let args = args.iter().map(|a| a.as_ref()).collect::<Vec<_>>();
        execvp(&process, &args).map_err(|_| {
            Error::new(
                ErrorKind::Exec,
                "Failed to replace process with new process",
            )
        })?;

        Ok(())
    }
}

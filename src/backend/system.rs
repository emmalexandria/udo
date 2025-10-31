use std::{
    env,
    ffi::{CStr, CString},
    io,
};

use nix::unistd::{Gid, Uid, execvp, seteuid, setgid, setuid};

use crate::backend::{Backend, Error, ErrorKind, Result};

/// This is a [Backend] used for running udo. It interacts directly with the system
/// it is running on, and all actions performed on it reflect directly on the system
#[derive(Eq, PartialEq, Clone)]
pub struct SystemBackend {}

impl Backend for SystemBackend {
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

    fn execvp(&mut self, process: &str, args: &[&str]) -> Result<()> {
        let process = CString::new(process).map_err(|_| {
            Error::new(
                ErrorKind::InvalidString,
                "Failed to convert command to CString",
            )
        })?;

        // Flatten args for now, add error handling in future
        let args = args
            .iter()
            .flat_map(|s| {
                CString::new(s.as_bytes()).map_err(|_| {
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

        // In theory this function should never return. If it does, something has gone rather wrong
        Err(Error::new(ErrorKind::Exec, "Failed to execvp"))
    }

    fn get_env(&self, name: &str) -> Result<String> {
        Ok(env::var(name)
            .map_err(|e| Error::new(ErrorKind::Env, "Failed to get environment variable"))?)
    }

    unsafe fn set_env(&mut self, name: &str, value: &str) {
        unsafe {
            env::set_var(name, value);
        }
    }

    fn vars(&self) -> Vec<(String, String)> {
        env::vars().collect()
    }
}

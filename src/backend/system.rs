use std::{env, ffi::CString, fs};

use nix::unistd::{Gid, Uid, execvp, getuid, seteuid, setgid, setuid};

use crate::backend::{Backend, Error, ErrorKind, Result};

/// This is a [Backend] used for running udo. It interacts directly with the system
/// it is running on, and all actions performed on it reflect directly on the system
#[derive(Eq, PartialEq, Clone)]
pub struct SystemBackend {
    original: Uid,
    target: Uid,
}

impl SystemBackend {
    pub fn new(target: Uid) -> Self {
        Self {
            original: getuid(),
            target,
        }
    }
}

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

    fn get_var(&self, name: &str) -> Result<String> {
        env::var(name).map_err(|e| Error::new(ErrorKind::Env, "Failed to get environment variable"))
    }

    unsafe fn set_var(&mut self, name: &str, value: &str) {
        unsafe {
            env::set_var(name, value);
        }
    }

    unsafe fn remove_var(&mut self, name: &str) {
        unsafe {
            env::remove_var(name);
        }
    }

    fn vars(&self) -> Vec<(String, String)> {
        env::vars().collect()
    }

    fn read_file(&self, path: &str) -> Result<String> {
        fs::read_to_string(path).map_err(|_| {
            Error::new(
                ErrorKind::DoesNotExist,
                "File does not exist or you cannot access it",
            )
        })
    }

    fn write_file(&mut self, path: &str, content: String) -> Result<()> {
        fs::write(path, content.as_bytes()).map_err(|_| {
            Error::new(
                ErrorKind::DoesNotExist,
                "File does not exist or you cannot access it",
            )
        })
    }

    fn is_root(&self) -> bool {
        self.getuid().is_root() || self.geteuid().is_root()
    }

    fn elevate(&mut self) -> Result<()> {
        self.seteuid(Uid::from_raw(0))
    }

    fn restore(&mut self) -> Result<()> {
        self.seteuid(self.original)
    }

    fn switch_final(&mut self) -> Result<()> {
        self.elevate()?;
        self.setuid(self.target)
    }
}

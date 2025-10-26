use anyhow::Result;
use nix::{
    libc::setreuid,
    unistd::{Uid, geteuid, getuid, seteuid, setuid},
};

#[derive(Debug, Clone)]
pub struct ElevatedContext {
    original: Uid,
    elevated: Uid,
    is_elevated: bool,
}

impl ElevatedContext {
    pub fn new(original: Uid, elevated: Uid) -> Self {
        Self {
            original,
            elevated,
            is_elevated: false,
        }
    }

    pub fn elevate(&mut self) -> anyhow::Result<()> {
        if !self.is_elevated {
            seteuid(self.elevated)?;
        }

        self.is_elevated = true;

        Ok(())
    }

    pub fn restore(&mut self) -> anyhow::Result<()> {
        if self.is_elevated {
            seteuid(self.original)?;
        }

        self.is_elevated = false;

        Ok(())
    }
}

impl Drop for ElevatedContext {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

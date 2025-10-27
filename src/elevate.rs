use anyhow::Result;
use nix::unistd::{Uid, geteuid, seteuid, setuid};

#[derive(Debug, Clone)]
pub struct ElevatedContext {
    original: Uid,
    elevated: Uid,
}

impl ElevatedContext {
    pub fn new(original: Uid, elevated: Uid) -> Self {
        let mut ret = Self { original, elevated };

        if ret.is_elevated() {
            let _ = ret.restore();
        }

        ret
    }

    pub fn elevate(&mut self) -> Result<()> {
        if !self.is_elevated() {
            seteuid(self.elevated)?;
        }

        Ok(())
    }

    pub fn restore(&mut self) -> Result<()> {
        if self.is_elevated() {
            seteuid(self.original)?;
        }

        Ok(())
    }

    fn is_elevated(&self) -> bool {
        let euid = geteuid();
        euid == self.elevated
    }
}

impl Drop for ElevatedContext {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

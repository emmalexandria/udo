use anyhow::Result;
use nix::unistd::{Uid, setuid};

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

    pub fn elevate(&self) -> anyhow::Result<()> {
        if !self.is_elevated {
            setuid(self.elevated)?;
        }

        Ok(())
    }

    pub fn restore(&self) -> anyhow::Result<()> {
        if self.is_elevated {
            setuid(self.original)?;
        }

        Ok(())
    }
}

impl Drop for ElevatedContext {
    fn drop(&mut self) {
        self.restore();
    }
}

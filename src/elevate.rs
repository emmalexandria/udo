use anyhow::Result;
use nix::unistd::{Uid, setuid};

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
            setuid(self.elevated)?;
            self.is_elevated = !self.is_elevated;
        }

        Ok(())
    }

    pub fn restore(&mut self) -> anyhow::Result<()> {
        if self.is_elevated {
            setuid(self.original)?;
            self.is_elevated = !self.is_elevated;
        }

        Ok(())
    }
}

impl Drop for ElevatedContext {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

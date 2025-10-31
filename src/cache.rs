use std::{
    fs::{self, File, Permissions},
    io::{Write, stdin},
    os::{fd::AsFd, unix::fs::PermissionsExt},
    path::PathBuf,
};

const CACHE_DIR: &str = "/var/run/udo";

use anyhow::Result;
use nix::{
    sys::time::TimeValLike,
    time::{ClockId, clock_gettime},
    unistd::{User, getppid, ttyname},
};
use serde::{Deserialize, Serialize};
use toml::Deserializer;

use crate::{backend::Backend, config::Config, run::Run};

#[derive(Debug, Clone)]
pub struct Cache {
    dir: PathBuf,
}

impl Cache {
    pub fn new(user: &User) -> Self {
        let dir = Self::get_dir(user);
        Self { dir }
    }

    pub fn get_id(user: &User) -> Result<String> {
        let uid = user.uid;
        let stdin = stdin();
        let stdin_fd = stdin.as_fd();
        let tty_path = ttyname(stdin_fd)?;
        let tty = tty_path.file_name().unwrap_or_default().to_string_lossy();
        let pid = getppid();

        Ok(format!("{uid}-{tty}-{pid}"))
    }

    pub fn get_dir(user: &User) -> PathBuf {
        let mut path = PathBuf::from(CACHE_DIR);
        path.push(&user.name);
        path
    }

    pub fn create_dir(&mut self, backend: &mut Box<dyn Backend>) -> Result<PathBuf> {
        backend.elevate()?;
        if fs::exists(&self.dir)? {
            let md = fs::metadata(&self.dir)?;
            if md.is_dir() {
                return Ok(self.dir.clone());
            }
        }

        fs::create_dir_all(&self.dir)?;
        fs::set_permissions(&self.dir, Permissions::from_mode(0o700))?;
        fs::set_permissions(&self.dir, Permissions::from_mode(0o700))?;
        backend.restore()?;

        Ok(self.dir.clone())
    }

    pub fn cache_run(&self, run: &mut Run) -> Result<()> {
        let id = Self::get_id(&run.user)?;
        let mut f_path = self.dir.clone();
        f_path.push(id);

        let entry = CacheEntry::try_from(&mut *run)?;

        let mut buf = toml::ser::Buffer::new();
        let se = toml::Serializer::new(&mut buf);
        let out = entry.serialize(se)?;

        run.backend.elevate()?;
        let mut file = File::create(f_path)?;
        file.write_all(out.to_string().as_bytes())?;
        run.backend.restore()?;

        Ok(())
    }

    pub fn check_cache(&self, run: &mut Run, config: &Config) -> Result<bool> {
        let id = Self::get_id(&run.user)?;
        let mut full = self.dir.clone();
        full.push(id);

        let time = clock_gettime(ClockId::CLOCK_REALTIME)?;

        run.backend.elevate()?;
        if !full.exists() || full.is_dir() {
            return Ok(false);
        }

        let content = fs::read_to_string(full)?;
        let de = Deserializer::parse(&content)?;
        let entry = CacheEntry::deserialize(de)?;
        run.backend.restore()?;

        let time_valid = time.num_minutes() - entry.timestamp < config.security.timeout;
        let user_valid = entry.uid == run.do_as.uid.as_raw();

        Ok(time_valid && user_valid)
    }

    pub fn clear(&self, backend: &mut Box<dyn Backend>) -> Result<()> {
        backend.elevate()?;

        if self.dir.exists() && self.dir.is_dir() {
            fs::remove_dir_all(&self.dir)?;
        }

        backend.restore()?;
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntry {
    timestamp: i64,
    uid: u32,
}

impl CacheEntry {
    pub fn new(timestamp: i64, uid: u32) -> Self {
        Self { timestamp, uid }
    }
}

impl TryFrom<&Run<'_>> for CacheEntry {
    type Error = anyhow::Error;

    fn try_from(run: &Run) -> std::result::Result<Self, Self::Error> {
        let time = clock_gettime(ClockId::CLOCK_REALTIME)?;
        Ok(CacheEntry::new(time.num_minutes(), run.do_as.uid.as_raw()))
    }
}

impl TryFrom<&mut Run<'_>> for CacheEntry {
    type Error = anyhow::Error;

    fn try_from(run: &mut Run<'_>) -> std::result::Result<Self, Self::Error> {
        let time = clock_gettime(ClockId::CLOCK_REALTIME)?;
        Ok(CacheEntry::new(time.num_minutes(), run.do_as.uid.as_raw()))
    }
}

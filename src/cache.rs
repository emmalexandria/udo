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

use crate::{backend::Backend, config::Config, run::Run};

#[derive(Debug, Clone)]
pub struct Cache {
    dir: PathBuf,
}

pub fn get_cache_id(user: &User) -> Result<String> {
    let uid = user.uid;
    let stdin = stdin();
    let stdin_fd = stdin.as_fd();
    let tty_path = ttyname(stdin_fd)?;
    let tty = tty_path.file_name().unwrap_or_default().to_string_lossy();
    let pid = getppid();

    Ok(format!("{uid}-{tty}-{pid}"))
}

pub fn get_cache_dir(user: &User) -> PathBuf {
    let mut path = PathBuf::from(CACHE_DIR);
    path.push(&user.name);
    path
}

pub fn create_cache_dir(user: &User, backend: &mut Box<dyn Backend>) -> Result<PathBuf> {
    let dir = get_cache_dir(user);
    backend.elevate()?;
    if fs::exists(&dir)? {
        let md = fs::metadata(&dir)?;
        if md.is_dir() {
            return Ok(dir);
        }
    }

    fs::create_dir_all(&dir)?;
    let parent_default = PathBuf::from(CACHE_DIR);
    let parent = dir.parent().unwrap_or(&parent_default);
    fs::set_permissions(parent, Permissions::from_mode(0o700))?;
    fs::set_permissions(&dir, Permissions::from_mode(0o700))?;

    backend.restore()?;

    Ok(dir)
}

pub fn write_entry(user: &User, entry: CacheEntry, backend: &mut Box<dyn Backend>) -> Result<()> {
    let id = get_cache_id(user)?;
    let mut path = get_cache_dir(user);
    path.push(id);

    let mut buf = toml::ser::Buffer::new();
    let se = toml::Serializer::new(&mut buf);
    let out = entry.serialize(se)?;

    backend.elevate()?;
    let mut file = File::create(path)?;
    file.write_all(out.to_string().as_bytes())?;
    backend.restore()?;

    Ok(())
}

pub fn check_cache(run: &mut Run, config: &Config) -> Result<bool> {
    let id = get_cache_id(&run.user)?;
    let mut full = get_cache_dir(&run.user);
    full.push(id);

    let time = clock_gettime(ClockId::CLOCK_REALTIME)?;

    run.backend.elevate()?;
    if !full.exists() || full.is_dir() {
        return Ok(false);
    }

    let content = fs::read_to_string(full)?;
    let entry = CacheEntry::from_content(&content)?;
    run.backend.restore()?;

    let time_valid = time.num_minutes() - entry.timestamp < config.security.timeout;
    let user_valid = entry.uid == run.do_as.uid.as_raw();

    Ok(time_valid && user_valid)
}

pub fn clear_cache(user: &User, backend: &mut Box<dyn Backend>) -> Result<()> {
    let dir = get_cache_dir(user);

    backend.elevate()?;
    if dir.exists() && dir.is_dir() {
        fs::remove_dir_all(&dir)?;
    }
    backend.restore()?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheEntry {
    timestamp: i64,
    uid: u32,
}

impl CacheEntry {
    pub fn new(timestamp: i64, uid: u32) -> Self {
        Self { timestamp, uid }
    }

    pub fn from_content(content: &str) -> Result<Self> {
        let de = toml::Deserializer::parse(content)?;
        Ok(Self::deserialize(de)?)
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

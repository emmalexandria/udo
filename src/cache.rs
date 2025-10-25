use std::{
    fs::{self, File, Permissions},
    io::{Write, stdin},
    os::{
        fd::{AsFd, RawFd},
        unix::fs::PermissionsExt,
    },
    path::PathBuf,
};

const CACHE_DIR: &str = "/var/run/udo";

use anyhow::Result;
use nix::{
    fcntl::open,
    sys::time::TimeValLike,
    time::{ClockId, clock_gettime},
    unistd::{Uid, User, chown, getppid, getuid, ttyname},
};
use serde::{Deserialize, Serialize};
use toml::Deserializer;

use crate::config::Config;

#[derive(Serialize, Deserialize)]
struct Cache {
    timestamp: i64,
}

impl Cache {
    pub fn new(timestamp: i64) -> Self {
        Self { timestamp }
    }
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

pub fn get_cache_dir(username: &str) -> PathBuf {
    let mut p = PathBuf::from(CACHE_DIR);
    p.push(username);
    p
}

pub fn create_cache_dir(username: &str) -> Result<PathBuf> {
    let p = PathBuf::from(CACHE_DIR);
    let mut full_path = p.clone();
    full_path.push(username);

    if fs::exists(&full_path)? {
        let md = fs::metadata(&full_path)?;
        if md.is_dir() {
            return Ok(full_path);
        }
    }

    fs::create_dir_all(&full_path)?;
    fs::set_permissions(&p, Permissions::from_mode(0o700))?;
    fs::set_permissions(&full_path, Permissions::from_mode(0o700))?;

    Ok(full_path)
}

pub fn check_cache(user: &User, config: &Config) -> Result<bool> {
    let id = get_cache_id(user)?;
    let mut dir = get_cache_dir(&user.name);
    dir.push(id);

    let time = clock_gettime(ClockId::CLOCK_REALTIME)?;
    let content = fs::read_to_string(dir)?;

    let de = Deserializer::parse(&content)?;
    let cache = Cache::deserialize(de)?;

    let ok = time.num_seconds() - cache.timestamp < config.security.timeout;

    Ok(ok)
}

pub fn cache_run(user: &User) -> Result<()> {
    let id = get_cache_id(user)?;
    let mut dir = get_cache_dir(&user.name);
    dir.push(id);

    let time = clock_gettime(ClockId::CLOCK_REALTIME)?;
    let cache = Cache::new(time.num_seconds());
    let mut buf = toml::ser::Buffer::new();
    let se = toml::Serializer::new(&mut buf);
    let out = cache.serialize(se)?;

    let mut file = File::create(dir)?;
    file.write_all(out.to_string().as_bytes())?;

    Ok(())
}

pub fn clear_cache(user: &User) -> Result<PathBuf> {
    let id = get_cache_id(user)?;
    let mut dir = get_cache_dir(&user.name);
    dir.push(id);

    fs::remove_dir_all(&dir)?;
    Ok(dir)
}

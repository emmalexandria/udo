use std::{
    cell::RefCell,
    collections::HashMap,
    path::{Path, PathBuf},
};

use nix::{
    errno::Errno,
    fcntl::OFlag,
    sys::stat::Mode,
    unistd::{Gid, Uid},
};

use crate::backend::{Error, ErrorKind};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VFile {
    content: Vec<u8>,
    uid: Uid,
    gid: Gid,
    mode: Mode,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VFileD {
    file: VFile,
    pos: usize,
    flags: OFlag,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct VirtualFS {
    files: RefCell<HashMap<PathBuf, VFile>>,
    open_fds: RefCell<HashMap<i32, VFileD>>,
    next_fd: RefCell<i32>,
}

impl VirtualFS {
    pub fn new() -> Self {
        Self {
            files: RefCell::new(HashMap::new()),
            open_fds: RefCell::new(HashMap::new()),
            next_fd: RefCell::new(3), // 0,1,2 are stdin,stdout,stderr
        }
    }

    pub fn insert_file<P: Into<PathBuf>>(&self, path: P, file: VFile) {
        self.files.borrow_mut().insert(path.into(), file);
    }

    pub fn open<P: Into<PathBuf>>(&self, path: P, flags: OFlag) -> Result<i32, Error> {
        let path = path.into();

        let file = self
            .files
            .borrow()
            .get(&path)
            .ok_or(Error::new(
                ErrorKind::DoesNotExist,
                "File does not exist in VFS",
            ))?
            .clone();

        let fd = {
            let mut next = self.next_fd.borrow_mut();
            let fd = *next;
            *next += 1;
            fd
        };

        self.open_fds.borrow_mut().insert(
            fd,
            VFileD {
                file,
                pos: 0,
                flags,
            },
        );

        Ok(fd)
    }

    pub fn read(&self, fd: i32, buf: &mut [u8]) -> Result<usize, Error> {
        let mut fds = self.open_fds.borrow_mut();
        let fd = fds.get_mut(&fd).ok_or(Error::new(
            ErrorKind::System(Errno::EBADF),
            "Invalid file descriptor",
        ))?;
        let bytes = std::cmp::min(buf.len(), fd.file.content.len() - fd.pos);
        buf[..bytes].copy_from_slice(&fd.file.content[fd.pos..fd.pos + bytes]);
        fd.pos += bytes;
        Ok(bytes)
    }

    pub fn write(&self, fd: i32, buf: &[u8]) -> Result<usize, Error> {
        let mut fds = self.open_fds.borrow_mut();
        let fd = fds.get_mut(&fd).ok_or(Error::new(
            ErrorKind::System(Errno::EBADF),
            "Invalid file descriptor",
        ))?;

        // For now we only support appending
        fd.file.content.extend_from_slice(buf);
        Ok(buf.len())
    }

    pub fn close(&self, fd: i32) -> Result<(), Error> {
        self.open_fds.borrow_mut().remove(&fd).ok_or(Error::new(
            ErrorKind::System(Errno::EBADF),
            "Could not close file descriptor, does not exist",
        ))?;
        Ok(())
    }
}

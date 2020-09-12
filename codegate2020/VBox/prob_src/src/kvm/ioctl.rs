use super::Error;
use std::fs::File;
use std::os::unix::io::AsRawFd;

pub(super) trait Into<T> {
    fn into(self) -> u64;
}

enum IoctlInner {
    File(File),
    Raw(i32),
}

pub(super) struct IoctlFd<O>(IoctlInner, core::marker::PhantomData<O>)
where
    O: Into<u64>;

extern "C" {
    fn ioctl(fd: i32, req: u64, ...) -> i32;
}

impl<O> IoctlFd<O>
where
    O: Into<u64>,
{
    pub fn from_path(n: &str) -> Result<Self, Error> {
        Ok(Self(
            IoctlInner::File(File::open(n).map_err(Error::OsError)?),
            core::marker::PhantomData,
        ))
    }

    pub fn from_raw(n: i32) -> Self {
        Self(IoctlInner::Raw(n), core::marker::PhantomData)
    }

    fn get_raw_fd(&self) -> i32 {
        match &self.0 {
            IoctlInner::File(f) => f.as_raw_fd(),
            IoctlInner::Raw(r) => *r,
        }
    }

    pub fn request<T>(&self, request: O, cmd1: T) -> Result<i32, Error> {
        let r = unsafe { ioctl(self.get_raw_fd(), request.into(), cmd1) };
        if r < 0 {
            Err(Error::OsError(std::io::Error::from_raw_os_error(r)))
        } else {
            Ok(r)
        }
    }
}

use nix::errno::Errno;
use nix::fcntl::open;
use nix::fcntl::AtFlags;
use nix::fcntl::OFlag;
use nix::sys::stat::fchmod;
use nix::sys::stat::fstatat;
use nix::sys::stat::mode_t;
use nix::sys::stat::Mode;
use nix::unistd::fchown;
use nix::unistd::Gid;
use nix::unistd::Uid;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::fs::File;
use std::io::Result;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::os::fd::OwnedFd;
use std::os::fd::RawFd;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::ffi::OsStringExt;
use std::path::Path;

#[cfg(all(target_os = "linux", feature = "unnamed-tmpfile"))]
mod linux;

#[cfg(all(target_os = "linux", feature = "unnamed-tmpfile"))]
pub(crate) use self::linux::*;

#[cfg(not(all(target_os = "linux", feature = "unnamed-tmpfile")))]
mod generic;

#[cfg(not(all(target_os = "linux", feature = "unnamed-tmpfile")))]
pub(crate) use self::generic::*;

#[derive(Copy, Clone, Debug)]
pub(crate) enum Preserve {
    No,
    Yes,
    Try,
}

impl Preserve {
    fn is_yes(&self) -> bool {
        match self {
            Self::No => false,
            Self::Yes | Self::Try => true,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct OpenOptions {
    pub(crate) read: bool,
    pub(crate) mode: mode_t,
    pub(crate) custom_flags: i32,
    pub(crate) preserve_mode: bool,
    pub(crate) preserve_owner: Preserve,
}

impl OpenOptions {
    pub(crate) fn new() -> Self {
        Self {
            read: false,
            mode: 0o666,
            custom_flags: 0,
            preserve_mode: true,
            preserve_owner: Preserve::Try,
        }
    }
}

impl Default for OpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct Dir {
    fd: OwnedFd,
}

impl Dir {
    pub(crate) fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let fd = open(
            path.as_ref(),
            OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
            Mode::empty(),
        )?;
        Ok(unsafe { Self::from_raw_fd(fd) })
    }
}

impl AsRawFd for Dir {
    fn as_raw_fd(&self) -> RawFd {
        self.fd.as_raw_fd()
    }
}

impl FromRawFd for Dir {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self {
            fd: OwnedFd::from_raw_fd(fd),
        }
    }
}

struct RandomName {
    buf: Vec<u8>,
}

impl RandomName {
    const SUFFIX_SIZE: usize = 6;

    fn new(base_name: &OsStr) -> Self {
        let buf_len = 1 + base_name.len() + 1 + Self::SUFFIX_SIZE;
        let mut buf = Vec::with_capacity(buf_len);
        buf.push(b'.');
        buf.extend_from_slice(base_name.as_bytes());
        buf.push(b'.');
        buf.extend_from_slice(&[0; Self::SUFFIX_SIZE]);
        debug_assert_eq!(buf_len, buf.len());
        Self { buf }
    }

    fn next(&mut self) -> &OsStr {
        let mut rng = rand::thread_rng();
        let buf_len = self.buf.len();
        let suffix = &mut self.buf[buf_len - RandomName::SUFFIX_SIZE..];
        for c in suffix.iter_mut() {
            *c = rng.sample(Alphanumeric);
        }
        OsStr::from_bytes(&self.buf)
    }

    #[inline]
    fn into_os_string(self) -> OsString {
        OsString::from_vec(self.buf)
    }
}

fn maybe_ignore_eperm(result: nix::Result<()>, preserve: Preserve) -> nix::Result<()> {
    match result {
        Err(Errno::EPERM) => match preserve {
            Preserve::Try => {
                if Uid::effective().is_root() {
                    result
                } else {
                    Ok(())
                }
            }
            _ => result,
        },
        _ => result,
    }
}

fn copy_file_perms<P: AsRef<Path>>(
    dir: &Dir,
    copy_from: P,
    copy_to: &File,
    opts: &OpenOptions,
) -> Result<()> {
    let stat = match fstatat(
        dir.as_raw_fd(),
        copy_from.as_ref(),
        AtFlags::AT_SYMLINK_NOFOLLOW,
    ) {
        Ok(stat) => stat,
        Err(Errno::ENOENT) => return Ok(()),
        Err(err) => return Err(err.into()),
    };
    if opts.preserve_mode {
        let mode = unsafe { Mode::from_bits_unchecked(stat.st_mode) };
        fchmod(copy_to.as_raw_fd(), mode)?;
    }
    if opts.preserve_owner.is_yes() {
        let uid = Uid::from_raw(stat.st_uid);
        let gid = Gid::from_raw(stat.st_gid);
        maybe_ignore_eperm(
            fchown(copy_to.as_raw_fd(), Some(uid), Some(gid)),
            opts.preserve_owner,
        )?;
    }
    Ok(())
}

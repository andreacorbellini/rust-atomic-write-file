use crate::imp::unix::copy_file_perms;
use crate::imp::unix::Dir;
use crate::imp::unix::OpenOptions;
use crate::imp::unix::RandomName;
use nix::errno::Errno;
use nix::fcntl::openat;
use nix::fcntl::renameat;
use nix::fcntl::OFlag;
use nix::libc;
use nix::sys::stat::Mode;
use nix::unistd::linkat;
use nix::unistd::LinkatFlags;
use std::ffi::OsString;
use std::fs::File;
use std::io::Result;
use std::os::fd::AsRawFd;
use std::os::fd::FromRawFd;
use std::path::Path;

#[derive(Debug)]
pub(crate) struct TemporaryFile {
    pub(crate) dir: Dir,
    pub(crate) file: File,
    pub(crate) name: OsString,
}

impl TemporaryFile {
    pub(crate) fn open(opts: &OpenOptions, path: &Path) -> Result<Self> {
        let dir_path = path.parent().ok_or(Errno::EISDIR)?;
        let name = path.file_name().ok_or(Errno::EISDIR)?.to_os_string();

        let dir = if !dir_path.as_os_str().is_empty() {
            Dir::open(dir_path)?
        } else {
            Dir::open(".")?
        };

        let access_mode = if opts.read {
            OFlag::O_RDWR
        } else {
            OFlag::O_WRONLY
        };

        let file_fd = openat(
            dir.as_raw_fd(),
            ".",
            OFlag::O_TMPFILE
                | access_mode
                | OFlag::O_CLOEXEC
                | OFlag::from_bits_truncate(opts.custom_flags & !libc::O_ACCMODE),
            Mode::from_bits_truncate(opts.mode),
        )?;
        let file = unsafe { File::from_raw_fd(file_fd) };

        if opts.preserve_mode || opts.preserve_owner.is_yes() {
            copy_file_perms(&dir, &name, &file, opts)?;
        }

        Ok(Self { dir, file, name })
    }

    fn link_to_random_name(&self) -> nix::Result<OsString> {
        let fd = self.file.as_raw_fd();
        let src = OsString::from(format!("/proc/self/fd/{fd}"));
        let mut random_name = RandomName::new(&self.name);

        loop {
            match linkat(
                Some(self.dir.as_raw_fd()),
                src.as_os_str(),
                Some(self.dir.as_raw_fd()),
                random_name.next(),
                LinkatFlags::SymlinkFollow,
            ) {
                Ok(()) => return Ok(random_name.into_os_string()),
                Err(Errno::EEXIST) => continue,
                Err(err) => return Err(err),
            }
        }
    }

    pub(crate) fn rename_file(&self) -> Result<()> {
        let src = self.link_to_random_name()?;
        renameat(
            Some(self.dir.as_raw_fd()),
            src.as_os_str(),
            Some(self.dir.as_raw_fd()),
            self.name.as_os_str(),
        )?;
        Ok(())
    }

    pub(crate) fn remove_file(&self) -> Result<()> {
        Ok(())
    }
}

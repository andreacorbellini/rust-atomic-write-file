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
use nix::unistd::unlinkat;
use nix::unistd::UnlinkatFlags;
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
    pub(crate) temporary_name: OsString,
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
        let flags = access_mode
            | OFlag::O_CREAT
            | OFlag::O_EXCL
            | OFlag::O_CLOEXEC
            | OFlag::from_bits_truncate(opts.custom_flags & !libc::O_ACCMODE);
        let create_mode = Mode::from_bits_truncate(opts.mode);

        let mut random_name = RandomName::new(&name);
        let file_fd = loop {
            match openat(dir.as_raw_fd(), random_name.next(), flags, create_mode) {
                Ok(file_fd) => break file_fd,
                Err(Errno::EEXIST) => continue,
                Err(err) => return Err(err.into()),
            }
        };
        let file = unsafe { File::from_raw_fd(file_fd) };
        let temporary_name = random_name.into_os_string();

        if opts.preserve_mode || opts.preserve_owner.is_yes() {
            copy_file_perms(&dir, &name, &file, opts)?;
        }

        Ok(Self {
            dir,
            file,
            name,
            temporary_name,
        })
    }

    pub(crate) fn rename_file(&self) -> Result<()> {
        renameat(
            Some(self.dir.as_raw_fd()),
            self.temporary_name.as_os_str(),
            Some(self.dir.as_raw_fd()),
            self.name.as_os_str(),
        )?;
        Ok(())
    }

    pub(crate) fn remove_file(&self) -> Result<()> {
        unlinkat(
            Some(self.dir.as_raw_fd()),
            self.temporary_name.as_os_str(),
            UnlinkatFlags::NoRemoveDir,
        )?;
        Ok(())
    }
}

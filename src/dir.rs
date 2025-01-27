use crate::imp;

/// A borrowed reference to the directory containing an
/// [`AtomicWriteFile`](crate::AtomicWriteFile).
///
/// This can be obtained via [`AtomicWriteFile::directory()`](crate::AtomicWriteFile::directory).
/// The purpose of this struct is to allow you to obtain the directory file descriptor, without
/// having to open it through a call to `open(2)`.
///
/// This struct supports only two operations:
/// - conversion to a borrowed directory file descriptor through
///   [`AsFd::as_fd()`](std::os::fd::AsFd::as_fd)
/// - conversion to a raw directory file descriptor through
///   [`AsRawFd::as_raw_fd()`](std::os::fd::AsRawFd::as_raw_fd)
///
/// Directory file descriptors are not available on all platforms. See
/// [`AtomicWriteFile::directory()`](crate::AtomicWriteFile::directory) for more details.
///
/// # Examples
///
/// ```
/// # fn main() -> std::io::Result<()> {
/// # let test_dir = option_env!("TEST_DIR").unwrap_or("target/test-files");
/// # std::fs::create_dir_all(&test_dir).expect("failed to create test dir");
/// # std::env::set_current_dir(test_dir).expect("failed to move to test dir");
/// # #[cfg(any(unix, target_os = "wasi"))]
/// use std::os::fd::AsFd;
/// use atomic_write_file::AtomicWriteFile;
///
/// let file = AtomicWriteFile::open("foo.txt")?;
/// if let Some(dir) = file.directory() {
/// #   #[cfg(any(unix, target_os = "wasi"))]
///     let borrowed_fd = dir.as_fd();
/// #   #[cfg(any(unix, target_os = "wasi"))]
///     println!("directory fd: {:?}", borrowed_fd);
/// #   #[cfg(not(any(unix, target_os = "wasi")))]
/// #   let _ = dir;
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Copy, Clone, Debug)]
pub struct Directory<'a> {
    #[cfg_attr(not(any(unix, target_os = "wasi")), allow(dead_code))]
    inner: &'a imp::Dir,
}

impl<'a> Directory<'a> {
    pub(crate) fn new(inner: &'a imp::Dir) -> Self {
        Self { inner }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl std::os::fd::AsFd for Directory<'_> {
    #[inline]
    fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> {
        self.inner.as_fd()
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl std::os::fd::AsRawFd for Directory<'_> {
    #[inline]
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        self.inner.as_raw_fd()
    }
}

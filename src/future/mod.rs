use crate::imp;
use crate::OpenOptions;
use async_std::fs::File;
use async_std::io::IoSlice;
use async_std::io::IoSliceMut;
use async_std::io::Read;
use async_std::io::Seek;
use async_std::io::SeekFrom;
use async_std::io::Write;
use async_std::sync::Arc;
use async_std::task::block_on;
use async_std::task::spawn_blocking;
use async_std::task::Context;
use async_std::task::Poll;
use std::io::Result;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::path::Path;
use std::pin::pin;
use std::pin::Pin;
use std::ptr;

#[cfg(test)]
mod tests;

#[derive(Clone, Debug)]
pub struct AtomicWriteFile {
    temporary_file: Arc<imp::TemporaryFile<File>>,
    finalized: bool,
}

impl AtomicWriteFile {
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = spawn_blocking(move || OpenOptions::new().open(path)).await?;

        // Take the `temporary_file` out of the blocking `AtomicWriteFile`, so that we can convert
        // it to an async file. This requires unsafe code because `AtomicWriteFile` has a
        // destructor, and we want to avoid running it now
        let file = ManuallyDrop::new(file);
        // SAFETY: we're taking ownership of the `temporary_file`, and disposing of `file` without
        // running its destructor
        let temporary_file = unsafe { ptr::read(&(*file).temporary_file) };

        Ok(Self {
            temporary_file: Arc::new(temporary_file.into()),
            finalized: false,
        })
    }

    #[inline]
    pub fn as_file(&self) -> &File {
        &self.temporary_file.file
    }

    pub async fn commit(mut self) -> Result<()> {
        self._commit().await
    }

    async fn _commit(&mut self) -> Result<()> {
        if self.finalized {
            return Ok(());
        }
        self.finalized = true;
        self.sync_all().await?;
        let temporary_file = Arc::clone(&self.temporary_file);
        spawn_blocking(move || temporary_file.rename_file()).await
    }

    pub async fn discard(mut self) -> Result<()> {
        self._discard().await
    }

    async fn _discard(&mut self) -> Result<()> {
        if self.finalized {
            return Ok(());
        }
        self.finalized = true;
        let temporary_file = Arc::clone(&self.temporary_file);
        spawn_blocking(move || temporary_file.remove_file()).await
    }
}

impl Drop for AtomicWriteFile {
    #[inline]
    fn drop(&mut self) {
        if !self.finalized {
            // Ignore all errors
            let _ = block_on(self._discard());
        }
    }
}

impl Deref for AtomicWriteFile {
    type Target = File;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_file()
    }
}

impl Read for AtomicWriteFile {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        pin!(&(*self.temporary_file).file).poll_read(cx, buf)
    }

    #[inline]
    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<Result<usize>> {
        pin!(&(*self.temporary_file).file).poll_read_vectored(cx, bufs)
    }
}

impl Read for &AtomicWriteFile {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        pin!(&(*self.temporary_file).file).poll_read(cx, buf)
    }

    #[inline]
    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<Result<usize>> {
        pin!(&(*self.temporary_file).file).poll_read_vectored(cx, bufs)
    }
}

impl Write for AtomicWriteFile {
    #[inline]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        pin!(&(*self.temporary_file).file).poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        pin!(&(*self.temporary_file).file).poll_flush(cx)
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        pin!(&(*self.temporary_file).file).poll_close(cx)
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize>> {
        pin!(&(*self.temporary_file).file).poll_write_vectored(cx, bufs)
    }
}

impl Write for &AtomicWriteFile {
    #[inline]
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        pin!(&(*self.temporary_file).file).poll_write(cx, buf)
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        pin!(&(*self.temporary_file).file).poll_flush(cx)
    }

    #[inline]
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        pin!(&(*self.temporary_file).file).poll_close(cx)
    }

    #[inline]
    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize>> {
        pin!(&(*self.temporary_file).file).poll_write_vectored(cx, bufs)
    }
}

impl Seek for AtomicWriteFile {
    #[inline]
    fn poll_seek(self: Pin<&mut Self>, cx: &mut Context<'_>, pos: SeekFrom) -> Poll<Result<u64>> {
        pin!(&(*self.temporary_file).file).poll_seek(cx, pos)
    }
}

impl Seek for &AtomicWriteFile {
    #[inline]
    fn poll_seek(self: Pin<&mut Self>, cx: &mut Context<'_>, pos: SeekFrom) -> Poll<Result<u64>> {
        pin!(&(*self.temporary_file).file).poll_seek(cx, pos)
    }
}

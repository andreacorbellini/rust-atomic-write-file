use crate::future::AtomicWriteFile;
use crate::tests::test_file;
use crate::tests::verify_no_leftovers;
use async_std::fs;
use async_std::io::WriteExt;
use async_std::task::block_on;
use std::io::Result;

#[test]
fn create_new() -> Result<()> {
    block_on(async {
        let path = test_file("async-new");
        assert!(!path.exists());

        let mut file = AtomicWriteFile::open(&path).await?;
        assert!(!path.exists());

        file.write_all(b"hello ").await?;
        assert!(!path.exists());
        file.flush().await?;
        assert!(!path.exists());
        file.write_all(b"world\n").await?;
        assert!(!path.exists());
        file.flush().await?;
        assert!(!path.exists());

        file.commit().await?;

        assert!(path.exists());
        assert_eq!(fs::read(&path).await?, b"hello world\n");

        verify_no_leftovers(path);

        Ok(())
    })
}

#[test]
fn overwrite_existing() -> Result<()> {
    block_on(async {
        let path = test_file("async-existing");
        fs::write(&path, b"initial contents\n").await?;
        assert_eq!(fs::read(&path).await?, b"initial contents\n");

        let mut file = AtomicWriteFile::open(&path).await?;
        assert_eq!(fs::read(&path).await?, b"initial contents\n");

        file.write_all(b"hello ").await?;
        assert_eq!(fs::read(&path).await?, b"initial contents\n");
        file.flush().await?;
        assert_eq!(fs::read(&path).await?, b"initial contents\n");
        file.write_all(b"world\n").await?;
        assert_eq!(fs::read(&path).await?, b"initial contents\n");
        file.flush().await?;
        assert_eq!(fs::read(&path).await?, b"initial contents\n");

        file.commit().await?;

        assert_eq!(fs::read(&path).await?, b"hello world\n");

        verify_no_leftovers(path);

        Ok(())
    })
}

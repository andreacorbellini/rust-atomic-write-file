use crate::AtomicWriteFile;
use std::fs;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Result;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::panic;
use std::path::Path;
use std::path::PathBuf;

fn test_directory() -> PathBuf {
    let path = option_env!("TEST_DIR").unwrap_or("target/test-files");
    println!("using test directory: {path:?}");
    fs::create_dir_all(path)
        .unwrap_or_else(|err| panic!("failed to create test directory {path:?}: {err}"));
    path.into()
}

fn test_file<P: AsRef<Path>>(name: P) -> PathBuf {
    let mut path = test_directory();
    path.push(name);
    match fs::remove_file(&path) {
        Ok(()) => (),
        Err(ref err) if err.kind() == ErrorKind::NotFound => (),
        Err(ref err) => panic!("failed to remove test file {path:?}: {err}"),
    }
    path
}

fn list_temporary_files<P: AsRef<Path>>(path: P) -> impl Iterator<Item = PathBuf> {
    let path = path.as_ref();
    let dir_path = path.parent().unwrap();
    let file_name = path.file_name().unwrap();

    let mut prefix = String::new();
    prefix.push('.');
    prefix.push_str(file_name.to_str().unwrap());
    prefix.push('.');

    let entries = fs::read_dir(dir_path).unwrap_or_else(|err| {
        panic!("failed to list contents of test directory {dir_path:?}: {err}")
    });

    entries.filter_map(move |entry| {
        let entry_path = entry.unwrap().path();
        let entry_name = entry_path.file_name().unwrap();
        if entry_name.to_string_lossy().starts_with(&prefix) {
            Some(PathBuf::from(entry_name))
        } else {
            None
        }
    })
}

fn verify_no_leftovers<P: AsRef<Path>>(path: P) {
    let leftovers = list_temporary_files(path).collect::<Vec<PathBuf>>();
    if !leftovers.is_empty() {
        panic!("found leftover files: {leftovers:?}");
    }
}

#[test]
fn create_new() -> Result<()> {
    let path = test_file("new");
    assert!(!path.exists());

    let mut file = AtomicWriteFile::open(&path)?;
    assert!(!path.exists());

    file.write_all(b"hello ")?;
    assert!(!path.exists());
    file.flush()?;
    assert!(!path.exists());
    file.write_all(b"world\n")?;
    assert!(!path.exists());
    file.flush()?;
    assert!(!path.exists());

    file.commit()?;

    assert!(path.exists());
    assert_eq!(fs::read(&path)?, b"hello world\n");

    verify_no_leftovers(path);

    Ok(())
}

#[test]
fn overwrite_existing() -> Result<()> {
    let path = test_file("existing");
    fs::write(&path, b"initial contents\n")?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");

    let mut file = AtomicWriteFile::open(&path)?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");

    file.write_all(b"hello ")?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");
    file.flush()?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");
    file.write_all(b"world\n")?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");
    file.flush()?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");

    file.commit()?;

    assert_eq!(fs::read(&path)?, b"hello world\n");

    verify_no_leftovers(path);

    Ok(())
}

#[test]
fn concurrency() -> Result<()> {
    let path = test_file("concurrency");
    fs::write(&path, b"initial contents\n")?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");

    let mut file1 = AtomicWriteFile::options().read(true).open(&path)?;
    let mut file2 = AtomicWriteFile::options().read(true).open(&path)?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");

    file1.write_all(b"contents written to file1\n")?;
    file1.flush()?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");

    file2.write_all(b"contents written to file2\n")?;
    file2.flush()?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");

    // verify that the two atomic files are not conflicting with each other (i.e. that they are
    // writing to distinct temporary files)
    fn rewind_and_read(file: &mut AtomicWriteFile) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        file.seek(SeekFrom::Start(0))?;
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
    assert_eq!(rewind_and_read(&mut file1)?, b"contents written to file1\n");
    assert_eq!(rewind_and_read(&mut file2)?, b"contents written to file2\n");

    Ok(())
}

#[test]
fn no_change_on_panic() -> Result<()> {
    let path = test_file("panic");
    fs::write(&path, b"initial contents\n")?;
    assert_eq!(fs::read(&path)?, b"initial contents\n");

    fn faulty_writer<W: Write>(mut w: W) -> Result<()> {
        w.write_all(b"new contents\n")?;
        panic!("uh oh");
    }

    let file = AtomicWriteFile::open(&path)?;
    let result = panic::catch_unwind(move || faulty_writer(file));
    assert!(result.is_err());
    assert_eq!(fs::read(&path)?, b"initial contents\n");

    verify_no_leftovers(path);

    Ok(())
}

#[test]
#[cfg(all(target_os = "linux", feature = "unnamed-tmpfile"))]
fn creates_unnamed_temporary_files() -> Result<()> {
    let path = test_file("foo");
    let file = AtomicWriteFile::open(&path)?;
    assert_eq!(list_temporary_files(path).next(), None);
    file.commit()
}

#[test]
#[cfg(not(all(target_os = "linux", feature = "unnamed-tmpfile")))]
fn creates_named_temporary_files() -> Result<()> {
    let path = test_file("foo");
    let file = AtomicWriteFile::open(&path)?;
    let temp_file_name = list_temporary_files(path)
        .next()
        .expect("no temporary files found")
        .to_string_lossy()
        .to_string();
    assert!(temp_file_name.is_ascii());
    assert!(temp_file_name.starts_with(".foo."));
    assert_eq!(temp_file_name.len(), 1 + 3 + 1 + 6);
    file.commit()
}

use atomic_write_file::AtomicWriteFile;
use nix::unistd::close;
use nix::unistd::fsync;
use std::io::Write;
use std::os::fd::AsFd;
use std::os::fd::AsRawFd;

fn main() {
    let mut file = AtomicWriteFile::open("/test/file").expect("open failed");
    let dir = file
        .directory()
        .expect("could not obtain directory fd")
        .as_fd()
        .try_clone_to_owned()
        .expect("could not duplicate directory fd");

    file.write_all(b"hello").expect("write failed");
    file.commit().expect("commit failed");

    fsync(dir.as_raw_fd()).expect("directory sync failed");
    close(dir.as_raw_fd()).expect("closing directory failed");
}

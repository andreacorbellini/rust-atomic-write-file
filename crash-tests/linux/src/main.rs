use atomic_write_file::AtomicWriteFile;
use nix::fcntl::open;
use nix::fcntl::OFlag;
use nix::sys::stat::Mode;
use nix::unistd::close;
use nix::unistd::fsync;
use std::io::Write;

fn main() {
    let mut file = AtomicWriteFile::open("/test/file").expect("open failed");
    file.write_all(b"hello").expect("write failed");
    file.commit().expect("commit failed");

    let dir = open(
        "/test",
        OFlag::O_DIRECTORY | OFlag::O_CLOEXEC,
        Mode::empty(),
    )
    .expect("open directory failed");

    fsync(dir).expect("directory sync failed");
    close(dir).expect("closing directory failed");
}

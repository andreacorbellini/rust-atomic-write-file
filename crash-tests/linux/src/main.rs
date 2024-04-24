use atomic_write_file::AtomicWriteFile;
use std::io::Write;

fn main() {
    let mut file = AtomicWriteFile::open("/test/file").expect("open failed");
    file.write_all(b"hello").expect("write failed");
    file.commit().expect("commit failed");
}

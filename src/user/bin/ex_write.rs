#![no_std]
// ex_write: write TEXT into FILE, creating or overwriting it.
// Usage: ex_write FILE TEXT
//
// Demonstrates: open() with the CREATE|WRONLY|TRUNC flag combination
// that the C API calls O_CREAT|O_WRONLY|O_TRUNC, and write().

use ulib::{env, sys, sys::fcntl::omode};

fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().expect("usage: ex_write FILE TEXT");
    let text = args.next().expect("usage: ex_write FILE TEXT");

    // Flag semantics:
    //   WRONLY  — open for writing only
    //   CREATE  — create the file if it does not exist
    //   TRUNC   — if it already exists, shrink it back to 0 bytes
    let fd = sys::open(path, omode::WRONLY | omode::CREATE | omode::TRUNC)
        .expect("open");

    // write() is permitted to accept fewer bytes than we offered, so
    // loop until every byte of TEXT has been delivered.
    let bytes = text.as_bytes();
    let mut off = 0;
    while off < bytes.len() {
        let n = sys::write(fd, &bytes[off..]).expect("write");
        off += n;
    }

    sys::close(fd).expect("close");
}

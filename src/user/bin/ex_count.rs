#![no_std]
// ex_count: a stripped-down `wc -c`. Opens a file and counts bytes by
// repeatedly calling read() until it signals EOF.
//
// Demonstrates: open(), read(), close() — called as raw syscalls via
// ulib::sys rather than through the File helper.

use ulib::{env, print, println, sys, sys::fcntl::omode};

fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().expect("usage: ex_count FILE");

    // open() returns a small integer file descriptor. RDONLY means we
    // only plan to read from it.
    let fd = sys::open(path, omode::RDONLY).expect("open");

    // read() is allowed to return fewer bytes than the buffer holds,
    // and returns Ok(0) at end-of-file — so the caller must loop.
    let mut buf = [0u8; 512];
    let mut count: usize = 0;
    loop {
        let n = sys::read(fd, &mut buf).expect("read");
        if n == 0 {
            break; // EOF
        }
        count += n;
    }

    // Always release the descriptor when done.
    sys::close(fd).expect("close");

    println!("{}: {} bytes", path, count);
}

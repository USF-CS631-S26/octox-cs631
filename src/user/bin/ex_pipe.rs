#![no_std]
// ex_pipe: send bytes from a child process to its parent through a
// kernel pipe. Only fork() here — neither side calls exec().
//
// pipe(p) fills in two file descriptors:
//   p[0] — the read end
//   p[1] — the write end
// Bytes written to p[1] come back out of p[0]. After fork(), BOTH
// processes hold BOTH ends; the convention is for each side to close
// the end it does not need. If the writer-side fd stays open in the
// reader, read() will not see EOF when the writer exits, because the
// kernel still counts a live writer.

use ulib::{print, println, sys, stdio::STDOUT_FILENO};

fn main() {
    let mut p = [0usize; 2];
    sys::pipe(&mut p).expect("pipe");
    let (read_fd, write_fd) = (p[0], p[1]);

    match sys::fork().expect("fork") {
        0 => {
            // --- child: the writer ---
            // We will not read from the pipe, so close that end.
            sys::close(read_fd).expect("close");

            let msg = b"hello from child\n";
            sys::write(write_fd, msg).expect("write");

            // Closing the write end is what lets the parent's read()
            // return 0 (EOF) once we are done.
            sys::close(write_fd).expect("close");
            sys::exit(0);
        }
        _ => {
            // --- parent: the reader ---
            // Symmetric: we will not write, so close the write end.
            // This is important — if we left it open and then tried to
            // read until EOF, we would block forever because the kernel
            // thinks *we* are still a writer.
            sys::close(write_fd).expect("close");

            let mut buf = [0u8; 64];
            let n = sys::read(read_fd, &mut buf).expect("read");
            sys::close(read_fd).expect("close");

            // Echo exactly what we received to our own stdout.
            sys::write(STDOUT_FILENO, &buf[..n]).expect("write");

            let mut status: i32 = 0;
            sys::wait(&mut status).expect("wait");
            println!("parent: child exited with {}", status);
        }
    }
}

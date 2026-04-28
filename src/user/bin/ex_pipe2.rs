#![no_std]
// ex_pipe2: plumb the stdout of one program into the stdin of another,
// exactly the way the shell implements `ls | wc`. Two fork()s, two
// exec()s, one pipe().
//
// Layout:
//
//     +--------+  write_fd  +--------+  read_fd   +--------+
//     | child1 | ---------> | pipe   | ---------> | child2 |
//     | ls    |             | buffer |            | wc     |
//     +--------+            +--------+            +--------+
//
// Each child rewires one end of the pipe onto stdin or stdout before
// calling exec(), so the execed program just reads/writes "normally"
// and never knows it is talking to a pipe.
//
// The single most common bug when doing this by hand is forgetting to
// close the pipe fds in the PARENT. If any process still holds the
// write end, wc will wait forever for EOF on stdin.

use ulib::{
    print, println,
    stdio::{STDIN_FILENO, STDOUT_FILENO},
    sys,
};

fn main() {
    let mut p = [0usize; 2];
    sys::pipe(&mut p).expect("pipe");
    let (read_fd, write_fd) = (p[0], p[1]);

    // --- first child: the producer (`ls`) ---
    match sys::fork().expect("fork") {
        0 => {
            // Redirect stdout to the pipe's write end.
            sys::dup2(write_fd, STDOUT_FILENO).expect("dup2");

            // After dup2, fd 1 already refers to the write end. The
            // original pipe fds are redundant in this process, and we
            // must close the read end because we are not going to read.
            sys::close(read_fd).expect("close");
            sys::close(write_fd).expect("close");

            let argv = ["ls"];
            sys::exec("/bin/ls", &argv, None).expect("exec");
            sys::exit(1);
        }
        _ => {}
    }

    // --- second child: the consumer (`wc`) ---
    match sys::fork().expect("fork") {
        0 => {
            // Redirect stdin to the pipe's read end.
            sys::dup2(read_fd, STDIN_FILENO).expect("dup2");

            // Same cleanup as above, mirror image.
            sys::close(read_fd).expect("close");
            sys::close(write_fd).expect("close");

            let argv = ["wc"];
            sys::exec("/bin/wc", &argv, None).expect("exec");
            sys::exit(1);
        }
        _ => {}
    }

    // --- parent ---
    // Close BOTH pipe ends here. If we did not, the kernel would still
    // count us as a writer, and _wc would block on read() forever
    // waiting for an EOF that never comes.
    sys::close(read_fd).expect("close");
    sys::close(write_fd).expect("close");

    // Reap both children. wait() returns whichever finishes first.
    let mut status: i32 = 0;
    sys::wait(&mut status).expect("wait 1");
    sys::wait(&mut status).expect("wait 2");
    println!("parent: both children finished");
}

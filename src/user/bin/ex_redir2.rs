#![no_std]
// ex_redir2: same idea as ex_redir, but the child also exec()s a
// different program after setting up the redirect. This shows that
// file descriptors survive exec() — the replacement program inherits
// the caller's open fds and sees fd 1 already pointing at the file.
// Usage: ex_redir2 OUTFILE
//
// This is exactly the mechanism a shell uses to implement `cmd > file`:
//   fork → open file → dup2(fd, 1) → close(fd) → exec(cmd).

use ulib::{env, print, println, stdio::STDOUT_FILENO, sys, sys::fcntl::omode};

fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().expect("usage: ex_redir2 OUTFILE");

    match sys::fork().expect("fork") {
        0 => {
            // --- child ---
            // Open the output file and splice it onto stdout.
            let fd = sys::open(path, omode::WRONLY | omode::CREATE | omode::TRUNC)
                .expect("open");
            sys::dup2(fd, STDOUT_FILENO).expect("dup2");
            sys::close(fd).expect("close");

            // Now exec() into /bin/_echo. Because fd 1 is inherited, the
            // echo program's output lands in the file rather than on
            // the terminal. This is why redirection "just works" for
            // arbitrary programs — they never need to know they are
            // being redirected.
            let argv = ["_echo", "hello", "from", "exec"];
            sys::exec("/bin/_echo", &argv, None).expect("exec");
            sys::exit(1); // unreachable unless exec failed
        }
        _ => {
            // --- parent ---
            let mut status: i32 = 0;
            sys::wait(&mut status).expect("wait");
            println!("parent: child exited with {}", status);
        }
    }
}

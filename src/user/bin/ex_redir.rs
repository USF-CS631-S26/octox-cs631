#![no_std]
// ex_redir: demonstrate how a process rewires its own stdout to a file
// before producing output. Only fork() here — no exec() — so the child
// keeps running this same program, but its fd 1 now points at the file.
// Usage: ex_redir OUTFILE
//
// Technique:
//   1. Close STDOUT
//   2. Open path to occupy the second (index == 1) slot in FD table
//   4. write to stdout normally; the bytes land in the file.

use ulib::{env, print, println, stdio::STDOUT_FILENO, sys, sys::fcntl::omode};

fn main() {
    let mut args = env::args().skip(1);
    let path = args.next().expect("usage: ex_redir OUTFILE");

    match sys::fork().expect("fork") {
        0 => {
            // --- child ---

            // Close STDOUT
            sys::close(STDOUT_FILENO).expect("close");

            // Now open the file in the first open fd slot
            let fd = sys::open(path, omode::WRONLY | omode::CREATE | omode::TRUNC)
                .expect("open");

            // println! writes to fd 1 — which is now the file.
            println!("hello from child: my stdout is redirected");
            sys::exit(0);
        }
        _ => {
            // --- parent ---
            let mut status: i32 = 0;
            sys::wait(&mut status).expect("wait");
            // The parent never touched its own fd 1, so this prints to
            // the terminal, not to the file.
            println!("parent: child finished; my stdout is still the terminal");
        }
    }
}

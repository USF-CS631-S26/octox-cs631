#![no_std]
// ex_exec: demonstrate fork() + exec() + wait(). The parent forks; the
// child replaces its own process image with /bin/_sleep 10 via exec().
// The parent waits for the child and prints its exit status.
//
// Key properties of exec():
//   * On success it does NOT return — the old program is gone, and
//     execution resumes at the entry point of the new one.
//   * argv[0] is conventionally the program name; real arguments start
//     at argv[1].
//   * File descriptors opened before exec() are inherited (unless
//     marked close-on-exec) — see ex_redir2.rs for why that matters.

use ulib::{print, println, sys};

fn main() {
    match sys::fork().expect("fork") {
        0 => {
            // --- child ---
            // The binaries in octox are installed under /bin with an
            // underscore prefix: the "sleep" command lives at
            // /bin/_sleep. sleep takes one argument: seconds to pause.
            let argv = ["ls"];
            sys::exec("/bin/ls", &argv, None).expect("exec");

            // Only reachable if exec() failed in a way expect() did not
            // catch (should be unreachable).
            sys::exit(1);
        }
        child => {
            // --- parent ---
            println!("parent: launched child pid={}, waiting...", child);
            let mut status: i32 = 0;
            let reaped = sys::wait(&mut status).expect("wait");
            println!("parent: child {} exited with {}", reaped, status);
        }
    }
}

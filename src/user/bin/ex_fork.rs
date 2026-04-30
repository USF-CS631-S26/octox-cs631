#![no_std]
// ex_fork: show that fork() gives the child its own private copy of
// the parent's memory. A mutation after fork in one process is not
// visible in the other — they share no data.
//
// Flow:
//   parent: fork() → wait()
//   child:  modify local variable → exit()

use ulib::{print, println, sys};

fn main() {
    // Both parent and child will see x == 100 immediately after fork.
    // The child then bumps its own copy; the parent's copy is untouched.
    let mut x: i32 = 100;
    println!("before fork: x={}", x);

    // fork() returns:
    //   Ok(0)         in the child
    //   Ok(child_pid) in the parent
    match sys::fork().expect("fork") {
        0 => {
            // --- child ---
            println!("child : started");

            x += 1;
            println!(
                "child : pid={} x={}",
                sys::getpid().unwrap(),
                x
            );
            // Exit explicitly so the child never falls through to the
            // parent branch below.
            sys::exit(99);
        }
        child_pid => {
            // --- parent ---
            println!("parent : resumed");
            
            // wait() blocks until some child exits and writes that
            // child's exit status into the i32 we hand it.
            let mut status: i32 = 0;
            sys::wait(&mut status).expect("wait");
            println!(
                "parent: pid={} x={} (child {} exited with {})",
                sys::getpid().unwrap(),
                x,
                child_pid,
                status
            );
        }
    }
}

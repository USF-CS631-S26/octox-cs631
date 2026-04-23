#![no_std]
// ex_args: show how a program reads its command-line arguments.
//
// When the kernel does exec(), it hands the new program an argv array.
// In octox, the lang_start wrapper in src/user/lib/lib.rs publishes
// that array under env::ARGS, and env::args() walks it. argv[0] is the
// program name by convention; argv[1..] are the arguments the user
// passed on the command line.

use ulib::{env, print, println};

fn main() {
    // Iterate every entry (including argv[0]) and echo it on its own line.
    for arg in env::args() {
        println!("{}", arg);
    }
}

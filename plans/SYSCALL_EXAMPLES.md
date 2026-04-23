# Plan: Teaching Example Programs for Octox Syscalls

## Context

Octox is a Unix-like OS written in Rust. The user wants a suite of small,
teaching-focused user programs in `src/user/bin/` that each demonstrate a
specific POSIX-style system call or small set of calls. The goal is
pedagogical: each program should be short, well-commented, and call the
syscall wrappers directly (`sys::fork`, `sys::exec`, `sys::pipe`, `sys::dup2`,
`sys::open`, `sys::read`, `sys::write`, `sys::close`, `sys::wait`,
`sys::exit`) — **not** the higher-level helpers in `ulib::fs::File`,
`ulib::process::Command`, `ulib::pipe::pipe`, etc.

The nine programs to add are:

| File            | Concept demonstrated                                    |
|-----------------|---------------------------------------------------------|
| `ex_args.rs`    | Command-line argument access                            |
| `ex_count.rs`   | `open()` + `read()` — count bytes in a file             |
| `ex_write.rs`   | `open()` (O_CREATE\|O_WRONLY\|O_TRUNC) + `write()`      |
| `ex_fork.rs`    | `fork()`, `wait()`, `exit()` — copy-on-write semantics  |
| `ex_exec.rs`    | `fork()` + `exec()` + `wait()`                          |
| `ex_redir.rs`   | `fork()` + `open()` + `dup2()` to redirect child stdout |
| `ex_redir2.rs`  | `fork()` + `dup2()` + `exec()` — redirect another prog  |
| `ex_pipe.rs`    | `pipe()` + `fork()` — child → parent byte stream        |
| `ex_pipe2.rs`   | `pipe()` + 2×`fork()` + 2×`exec()` — piped pipeline     |

## Key codebase facts

### Syscall API (from `src/kernel/syscall.rs`, auto-generated into `OUT_DIR/usys.rs`)

```rust
pub fn fork() -> Result<usize>                     // child: 0; parent: child pid
pub fn exit(xstatus: i32) -> !
pub fn wait(xstatus: &mut i32) -> Result<usize>    // returns child pid
pub fn pipe(p: &mut [usize]) -> Result<()>         // p[0]=read fd, p[1]=write fd
pub fn read(fd: usize, buf: &mut [u8]) -> Result<usize>
pub fn write(fd: usize, b: &[u8]) -> Result<usize>
pub fn exec(filename: &str, argv: &[&str], envp: Option<&[Option<&str>]>) -> Result<usize>
pub fn open(filename: &str, flags: usize) -> Result<usize>
pub fn close(fd: usize) -> Result<()>
pub fn dup(fd: usize) -> Result<usize>
pub fn dup2(src: usize, dst: usize) -> Result<usize>
pub fn getpid() -> Result<usize>
pub fn sleep(n: usize) -> Result<()>
```

### Open-mode flags — `src/kernel/fcntl.rs` `omode`
`RDONLY=0x000, WRONLY=0x001, RDWR=0x002, CREATE=0x200, TRUNC=0x400, APPEND=0x800`

Access via: `use ulib::sys::fcntl::omode;`

### stdio fds — `src/user/lib/stdio.rs`
`STDIN_FILENO=0, STDOUT_FILENO=1, STDERR_FILENO=2`

### Args — `src/user/lib/env.rs`
`env::args()` returns an `ExactSizeIterator<Item=&'static str>`; `args[0]` is the
program name.

### Binary naming / installation
Every binary is registered as `_<name>` in `src/user/Cargo.toml` and installed at
`/bin/_<name>`. So the `sleep` program is invoked as `/bin/_sleep`. `exec()`
takes a full path — there is no PATH search at the syscall level.

### Program entry skeleton
Every binary starts with:
```rust
#![no_std]
use ulib::sys;
fn main() { ... }
```
`#![no_std]` is mandatory. The `lang_start` wrapper in `src/user/lib/lib.rs`
calls `sys::exit` automatically on return, so bare `main() { }` is fine.

## Files to add / modify

**Add** (9 new files under `/Users/benson/sync/cs631/os/octox-cs631/src/user/bin/`):
- `ex_args.rs`
- `ex_count.rs`
- `ex_write.rs`
- `ex_fork.rs`
- `ex_exec.rs`
- `ex_redir.rs`
- `ex_redir2.rs`
- `ex_pipe.rs`
- `ex_pipe2.rs`

**Modify**: `/Users/benson/sync/cs631/os/octox-cs631/src/user/Cargo.toml` — append
one `[[bin]]` stanza per program:
```toml
[[bin]]
name = "_ex_args"
path = "bin/ex_args.rs"
```
…and similar for the other eight.

## Per-program design

All nine use the common skeleton `#![no_std]` + `use ulib::{env, sys};`
(plus `sys::fcntl::omode` and `stdio::*_FILENO` where relevant). Comments
explain the pedagogical point at each syscall. Output uses the `print!`/
`println!` macros (thin wrappers over `sys::write` on fd 1) for readability;
the syscalls being *taught* in each file are always invoked directly.

### `ex_args.rs`
Iterate `env::args()`, print argv[0] then each arg on its own line. One-liner
loop with a short explanatory comment on how `argv` is published by the C-style
`main(argc, argv)` entry.

### `ex_count.rs`  — usage: `ex_count FILE`
- `sys::open(path, omode::RDONLY)` → fd
- Loop: `sys::read(fd, &mut buf)` until it returns `Ok(0)`; accumulate `n`
- `sys::close(fd)`
- `println!("{}: {} bytes", path, count)`

Comment: read returns 0 at EOF; partial reads are normal and must be looped.

### `ex_write.rs` — usage: `ex_write FILE TEXT`
- `sys::open(path, omode::WRONLY | omode::CREATE | omode::TRUNC)` → fd
- `sys::write(fd, text.as_bytes())` — loop if short write
- `sys::close(fd)`

Comment: flags select create-or-truncate semantics; `write` may return fewer
bytes than requested.

### `ex_fork.rs`
```
let mut x = 100;
let pid = sys::fork().unwrap();
if pid == 0 {
    x += 1;            // child's copy
    println!("child: pid={} x={}", sys::getpid().unwrap(), x);
    sys::exit(0);
} else {
    let mut st = 0i32;
    sys::wait(&mut st).unwrap();
    println!("parent: pid={} x={} (child exit={})", sys::getpid().unwrap(), x, st);
}
```
Comment: after `fork`, parent and child each see their own copy of `x`; the
child's increment is invisible to the parent — this is the key teaching point.

### `ex_exec.rs`
- Parent forks.
- Child: `sys::exec("/bin/_sleep", &["_sleep", "10"], None)`; on return it's an
  error, so print and `sys::exit(1)`.
- Parent: `sys::wait(&mut status)`; print status.

Comment: `exec` replaces the image, never returns on success; `argv[0]` is
conventionally the program name.

### `ex_redir.rs` — usage: `ex_redir OUTFILE`
Only `fork()` (no exec). Child:
1. `sys::open(outfile, WRONLY|CREATE|TRUNC)` → fd
2. `sys::close(STDOUT_FILENO)` then `sys::dup2(fd, STDOUT_FILENO)` (or just
   `dup2`, which closes dst first — use `dup2` and add a comment that it does).
3. `sys::close(fd)` — original fd no longer needed.
4. `println!("hello from child, my stdout is redirected");` — lands in the file.
5. `sys::exit(0)`.

Parent: `wait`, then `println!` a normal line (to terminal) proving its own
stdout is intact.

### `ex_redir2.rs` — usage: `ex_redir2 OUTFILE`
Same structure as `ex_redir.rs`, but after redirect the child calls
`sys::exec("/bin/_ls", &["_ls"], None)` (or `_echo` with an arg — pick `_echo`
since it produces deterministic output: `exec("/bin/_echo", &["_echo",
"redirected!"], None)`). Comment: redirection survives `exec` because fd
numbers are inherited.

### `ex_pipe.rs`
```
let mut fds = [0usize; 2];
sys::pipe(&mut fds).unwrap();
let (r, w) = (fds[0], fds[1]);
let pid = sys::fork().unwrap();
if pid == 0 {
    sys::close(r).unwrap();                 // child doesn't read
    sys::write(w, b"hello from child\n").unwrap();
    sys::close(w).unwrap();
    sys::exit(0);
} else {
    sys::close(w).unwrap();                 // parent doesn't write
    let mut buf = [0u8; 64];
    let n = sys::read(r, &mut buf).unwrap();
    sys::close(r).unwrap();
    // print what we got
    let mut st = 0i32;
    sys::wait(&mut st).unwrap();
}
```
Comment: **both sides close the unused end**, otherwise EOF is never seen.

### `ex_pipe2.rs`  — equivalent of `ls | wc`
Parent:
1. `sys::pipe(&mut fds)` → read_fd, write_fd.
2. Fork child A (producer):
   - `dup2(write_fd, STDOUT_FILENO)`; `close(read_fd)`; `close(write_fd)`.
   - `exec("/bin/_ls", &["_ls"], None)`.
3. Fork child B (consumer):
   - `dup2(read_fd, STDIN_FILENO)`; `close(read_fd)`; `close(write_fd)`.
   - `exec("/bin/_wc", &["_wc"], None)`.
4. Parent closes **both** pipe ends, then `wait` twice.

Comment: the parent *must* close both ends so the consumer sees EOF when the
producer finishes.

## Style / correctness rules applied

- Each file starts `#![no_std]` and uses only `ulib::sys`, `ulib::env`,
  `ulib::sys::fcntl::omode`, `ulib::stdio::{STDIN_FILENO, STDOUT_FILENO}`,
  and the `print!`/`println!` macros from `ulib`.
- All syscalls under demonstration are invoked directly.
- `unwrap()` is acceptable in teaching code (panics print a useful message via
  the `panic_handler` in `lib.rs`).
- Comments explain *why*, not *what* — e.g. "close unused end or the reader
  never sees EOF", not "// close the fd".
- No `extern crate alloc;` is needed — all programs can be done with stack
  buffers + `&str`.

## Verification

Build:
```
cd /Users/benson/sync/cs631/os/octox-cs631
cargo build --release
```
A successful build proves all nine `[[bin]]` entries compile and link.

Runtime (inside the octox VM via `make qemu` or the project's usual runner):
- `ex_args a b c` → prints `ex_args`, `a`, `b`, `c`.
- `ex_write /tmp/x hello` then `ex_count /tmp/x` → prints `5 bytes` (or 6 if a
  newline is appended — we write bytes verbatim, so 5).
- `cat /tmp/x` → prints `hello`.
- `ex_fork` → two lines showing parent and child `x` differ.
- `ex_exec` → pauses ~10 s (the `_sleep 10` child), then parent prints status.
- `ex_redir /tmp/r` then `cat /tmp/r` → prints the child's line; the parent's
  post-wait line appeared on the terminal instead.
- `ex_redir2 /tmp/r2` then `cat /tmp/r2` → contains `redirected!` (if we
  `exec _echo`).
- `ex_pipe` → parent prints `hello from child`.
- `ex_pipe2` → prints the `_wc` output of `_ls` on `/`.

import os
import pexpect.popen_spawn
import sys

def runoctox_list(args):
    gpid = os.getpgid(os.getpid())
    child = pexpect.popen_spawn.PopenSpawn(
        'cargo run --target riscv64gc-unknown-none-elf',
        encoding='utf-8',
        timeout=300,
    )

    child.expect_exact('$ ')

    for a in args:
        child.sendline(a)
        child.expect_exact('$ ')
        print('$', child.before.rstrip())

    child.send('\x01')
    child.send('x')
    child.expect(pexpect.EOF)


if __name__ == '__main__':
    child = None

    if len(sys.argv) > 1 and sys.argv[1] == '-l':
        args = sys.argv[2:]
    else:
        args = sys.argv[1:]

    if len(args) == 0:
        print("Provide an octox command or list of commands to execute.")
        sys.exit(-1)

    runoctox_list(args)

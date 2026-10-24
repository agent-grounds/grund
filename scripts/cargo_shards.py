"""One shard of a cargo suite: `run_cargo_suite.py --shard <k>/<n>` (§AR-ci.13.7).

`parse` reads the leading `--shard`, `deal` cuts the listed test binaries into `n` shards,
and `runner` makes this file cargo's runner for the one `cargo test` that runs the shard:
cargo starts every binary through it, with cargo's own environment and working directory,
and it runs a binary only when the shard holds it. The runner is set by `--config` on
that invocation alone, so a test that runs cargo itself never inherits it.
"""
import contextlib
import os
import re
import subprocess
import sys
import tempfile

# The executable a cargo `Running` line names, in its closing parentheses.
EXECUTABLE = re.compile(r'\(([^()]+)\)$')
SHARD = re.compile(r'^([1-9][0-9]*)/([1-9][0-9]*)$')
USAGE = ('usage: run_cargo_suite.py [--shard <k>/<n>] <cargo test arguments>'
         ' [-- <libtest arguments>]')
# §AR-ci.13.7: a shard that selects no target fails, as an empty selection does (§AR-ci.3.4.2).
NO_TARGET = 5


def parse(argv):
    """`((k, n), rest)` for a leading `--shard <k>/<n>`, `(None, argv)` without one.

    Anything but `1 <= k <= n` exits 2 with the usage line, before cargo runs.
    """
    if not argv or not (argv[0] == '--shard' or argv[0].startswith('--shard=')):
        return None, argv
    if argv[0] == '--shard':
        value, rest = (argv[1], argv[2:]) if len(argv) > 1 else ('', [])
    else:
        value, rest = argv[0].split('=', 1)[1], argv[1:]
    match = SHARD.match(value)
    if not match or int(match.group(1)) > int(match.group(2)):
        print(USAGE, file=sys.stderr)
        sys.exit(2)
    return (int(match.group(1)), int(match.group(2))), rest


def deal(binaries, count):
    """The `count` shards of the listed binaries, each a set of their keys.

    A function of the listing alone, so every job of a leg cuts the same: the keys in
    order, dealt round the shards one by one, so every binary lands in exactly one shard
    and no shard holds more than one binary more than another. A binary is the unit
    because its start, its fixtures and its child processes cost more than its count of
    tests says: the core's unit tests are thousands and take seconds.
    """
    shards = [set() for _ in range(count)]
    for index, key in enumerate(sorted(binaries)):
        shards[index % count].add(key)
    return shards


def stem(path):
    """An executable's file name without its extension: one spelling in a `Running` line and argv."""
    return os.path.splitext(os.path.basename(path))[0]


@contextlib.contextmanager
def runner(keys):
    """The `--config` arguments that start only the binaries `keys` name, while in use."""
    names = sorted(stem(EXECUTABLE.search(key).group(1)) for key in keys if EXECUTABLE.search(key))
    handle, listed = tempfile.mkstemp(prefix='grund-cargo-shard-', suffix='.txt')
    try:
        with os.fdopen(handle, 'w', encoding='utf-8') as out:
            out.write(''.join(f'{name}\n' for name in names))
        command = [sys.executable, os.path.abspath(__file__), listed]
        # TOML literal strings: a Windows path's backslashes stay as they are.
        yield ['--config', "target.'cfg(all())'.runner=["
               + ', '.join(f"'{part}'" for part in command) + ']']
    finally:
        os.unlink(listed)


def main(argv):
    """As cargo's runner: `<shard file> <executable> <arguments>`; run it if the shard holds it."""
    with open(argv[0], encoding='utf-8') as listed:
        chosen = set(listed.read().split())
    if stem(argv[1]) not in chosen:
        return 0
    return subprocess.run(argv[1:]).returncode


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))

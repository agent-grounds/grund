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
# §AR-ci.13.7: the seconds the slowest test binaries ran on Windows (the run of
# agent-grounds/grund#998 measured in agent-grounds/grund#978), by target name. A target not
# named weighs nothing here: it is dealt round the shards after these.
WEIGHTS = {'command_agreement': 285, 'scan_determinism': 120, 'citation_site_globs': 110,
           'e2e': 100, 'integrations': 66, 'coordinate_globs': 50, 'grund_core': 35}
# Shard 1's other Rust checks of §AR-ci.13.1: 184 seconds before its cargo test, less the 55
# its cargo build saves that test's own build.
FIRST_SHARD = 130
# The hash cargo appends to a test executable's name.
HASH = re.compile(r'-[0-9a-f]{16}$')


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

    A function of the listing alone, so every job of a leg cuts the same, and every binary
    lands in exactly one shard. The binaries `WEIGHTS` names go first, heaviest first, each
    to the shard least loaded so far, the lowest on a tie, with shard 1 starting from
    `FIRST_SHARD`; the rest follow in order, dealt round the shards one by one. A binary is
    the unit because its start, its fixtures and its child processes cost more than its
    count of tests says: the core's unit tests are thousands and take seconds.
    """
    shards = [set() for _ in range(count)]
    loads = [FIRST_SHARD if count > 1 else 0] + [0] * (count - 1)
    weighed = sorted((key for key in binaries if target(key) in WEIGHTS),
                     key=lambda key: (-WEIGHTS[target(key)], key))
    for key in weighed:
        index = min(range(count), key=lambda index: (loads[index], index))
        shards[index].add(key)
        loads[index] += WEIGHTS[target(key)]
    rest = sorted(key for key in binaries if target(key) not in WEIGHTS)
    for index, key in enumerate(rest):
        shards[index % count].add(key)
    return shards


def target(key):
    """The target a listed binary is: its executable's name without cargo's hash."""
    found = EXECUTABLE.search(key)
    return HASH.sub('', stem(found.group(1))) if found else key


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

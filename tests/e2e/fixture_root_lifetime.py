#!/usr/bin/env python3
"""Two passing core lib suites must leave no owned roots (§FS-repository-maintenance.4.1).

Exit 0: clean; 1: passing suites leaked; 2: build, suite or harness error.
Counts before the enclosing scratch owner removes anything, without following links.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile


def entries(root):
    """All entries, allowing only the documented empty shared base."""
    found = []
    for parent, directories, files in os.walk(root, followlinks=False):
        for name in directories + files:
            path = Path(parent) / name
            if (path == root / 'grund-lib-tests' and not path.is_symlink()
                    and path.is_dir() and not any(path.iterdir())):
                continue
            found.append(path)
    return found


def run(checkout, log_dir):
    """Build exactly this checkout's core lib-test target, then run it twice."""
    log_dir.mkdir(parents=True, exist_ok=True)
    build = subprocess.run(
        ['cargo', 'test', '-q', '-p', 'grund-core', '--lib', '--no-run',
         '--message-format=json'], cwd=checkout, capture_output=True, text=True)
    (log_dir / 'build.stderr').write_text(build.stderr)
    if build.returncode:
        raise RuntimeError(f'build failed ({build.returncode}); see {log_dir / "build.stderr"}')
    artifacts = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
    binaries = [item['executable'] for item in artifacts
                if item.get('reason') == 'compiler-artifact'
                and item.get('target', {}).get('name') == 'grund_core'
                and item['target']['kind'] == ['lib'] and item['profile']['test']
                and item.get('executable')]
    if len(binaries) != 1 or not Path(binaries[0]).is_file():
        raise RuntimeError(f'expected one executable core lib-test artifact, got {binaries}')
    print(f'core lib-test binary: {binaries[0]}', flush=True)
    counts = []
    with tempfile.TemporaryDirectory(prefix='grund-1362-suite-') as scratch:
        area = Path(scratch).resolve()
        for ancestor in [area, *area.parents]:
            for name in ['grund.toml', '.agents/grund.toml']:
                if (ancestor / name).exists():
                    raise RuntimeError(f'TMPDIR is config-covered: {ancestor / name}')
        environment = dict(os.environ, TMPDIR=str(area), TMP=str(area), TEMP=str(area))
        for number in [1, 2]:
            suite = subprocess.run([binaries[0], '-q'], cwd=checkout / 'crates/grund-core',
                                   env=environment, capture_output=True, text=True)
            output = suite.stdout + suite.stderr
            (log_dir / f'run{number}.log').write_text(output)
            summaries = [line for line in output.splitlines() if line.startswith('test result:')]
            if suite.returncode or len(summaries) != 1 or 'test result: ok.' not in summaries[0]:
                raise RuntimeError(f'run {number}: suite failed or missing summary '
                                   f'({suite.returncode}); see {log_dir / f"run{number}.log"}')
            leftover = entries(area)
            base = area / 'grund-lib-tests'
            roots = len(list(base.iterdir())) if base.is_dir() else 0
            counts.append(len(leftover))
            print(f'run {number}: {summaries[0]}', flush=True)
            print(f'run {number}: {len(leftover)} leftover entries; {roots} core roots', flush=True)
        if any(counts):
            print(f'DEFECT STANDS: passing suites left {counts} entries before harness cleanup')
            return 1
    print('both passing suites left no owned roots or contents')
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--checkout', type=Path, default=Path.cwd())
    parser.add_argument('--log-dir', type=Path, required=True)
    arguments = parser.parse_args()
    try:
        return run(arguments.checkout.resolve(), arguments.log_dir.resolve())
    except (OSError, ValueError, KeyError, RuntimeError) as error:
        print(f'HARNESS ERROR: {error}', flush=True)
        return 2


if __name__ == '__main__':
    raise SystemExit(main())

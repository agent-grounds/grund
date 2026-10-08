"""Portable inputs for the existing Python gate (§AR-ci.3.4).

Set GRUND_BUILT/GRUND_RELEASED to reuse local artifacts for
§FS-cochange-recipe.examples, and GRUND_BINDINGS_ORACLE to reuse a same-source
binding oracle for §FS-distribution.3.0.3; otherwise build this checkout, install
the exact released crate, and build the oracle from HEAD, without silently
skipping tests.
"""
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def inputs():
    """Fresh-checkout built, released and oracle inputs (§AR-ci.3.4)."""
    environment = dict(os.environ)
    suffix = '.exe' if os.name == 'nt' else ''
    scratch = Path.home() / 'ag/tmp'
    scratch.mkdir(parents=True, exist_ok=True)
    # Both builds share this checkout's target, so the oracle reuses grund-core's artifacts.
    target = Path(environment.get('CARGO_TARGET_DIR', str(ROOT / 'target'))).expanduser().resolve()
    if not environment.get('GRUND_BUILT'):
        subprocess.run(['cargo', 'build', '-p', 'grund', '--locked', '--target-dir', str(target)],
                       cwd=ROOT, check=True)
        environment['GRUND_BUILT'] = str(target / ('debug/grund' + suffix))
    if not environment.get('GRUND_RELEASED'):
        release = scratch / 'grund-cochange-release-0.16.1'
        executable = release / ('bin/grund' + suffix)
        if not executable.is_file():
            subprocess.run(['cargo', 'install', 'grund', '--version', '0.16.1', '--locked',
                            '--root', str(release), '--target-dir', str(scratch / 'grund-release-build')],
                           cwd=ROOT, check=True)
        environment['GRUND_RELEASED'] = str(executable)
    if not environment.get('GRUND_BINDINGS_ORACLE'):
        # §AR-ci.3.4: built from HEAD each run, as the workflow builds it; cargo is incremental.
        head = subprocess.run(['git', 'rev-parse', 'HEAD'], cwd=ROOT, capture_output=True,
                              text=True, check=True).stdout.strip()
        subprocess.run(['cargo', 'build', '--locked', '-p', 'grund-core', '--example',
                        'grund-binding-oracle', '--target-dir', str(target)], cwd=ROOT,
                       env=dict(environment, GRUND_BINDINGS_SOURCE_SHA=head), check=True)
        oracle = target / ('debug/examples/grund-binding-oracle' + suffix)
        if not oracle.is_file():
            raise FileNotFoundError(oracle)
        environment['GRUND_BINDINGS_ORACLE'] = str(oracle)
    for key in ('GRUND_BUILT', 'GRUND_RELEASED', 'GRUND_BINDINGS_ORACLE'):
        environment[key] = str(Path(environment[key]).expanduser().resolve())
    version = subprocess.run([environment['GRUND_RELEASED'], '--version'],
                             capture_output=True, text=True, check=True).stdout
    if version != 'grund 0.16.1\n':
        raise ValueError('GRUND_RELEASED must be released Grund 0.16.1')
    return environment


if __name__ == '__main__':
    sys.exit(subprocess.run([sys.executable, '-m', 'unittest', 'discover', '-s',
                            'tests/integration', '-p', 'test_*.py'], cwd=ROOT, env=inputs()).returncode)

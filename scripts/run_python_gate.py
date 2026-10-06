"""Portable inputs for §FS-cochange-recipe.examples in the existing Python gate.

Set GRUND_BUILT/GRUND_RELEASED to reuse local artifacts; otherwise build this
checkout and install the exact released crate, without silently skipping tests.
"""
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def inputs():
    """Fresh-checkout built/released inputs (§FS-cochange-recipe.examples)."""
    environment = dict(os.environ)
    suffix = '.exe' if os.name == 'nt' else ''
    scratch = Path.home() / 'ag/tmp'
    scratch.mkdir(parents=True, exist_ok=True)
    if not environment.get('GRUND_BUILT'):
        target = Path(environment.get('CARGO_TARGET_DIR', str(ROOT / 'target'))).expanduser().resolve()
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
    for key in ('GRUND_BUILT', 'GRUND_RELEASED'):
        environment[key] = str(Path(environment[key]).expanduser().resolve())
    version = subprocess.run([environment['GRUND_RELEASED'], '--version'],
                             capture_output=True, text=True, check=True).stdout
    if version != 'grund 0.16.1\n':
        raise ValueError('GRUND_RELEASED must be released Grund 0.16.1')
    return environment


if __name__ == '__main__':
    sys.exit(subprocess.run([sys.executable, '-m', 'unittest', 'discover', '-s',
                            'tests/integration', '-p', 'test_*.py'], cwd=ROOT, env=inputs()).returncode)

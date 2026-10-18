"""Raw Cargo extension setup (§FS-distribution.3.3.7.1, §AR-bindings.6.1).

The portable probe compiles the actual build.rs with the locked PyO3 build-config
library, then executes it for Darwin targets. It checks emitted Cargo directives,
not source text, and does not claim to cross-link or import a macOS extension.
The native test reuses command agreement's real preparation, as the #805 probe did.
The existing command-agreement acceptance remains the full actual-host proof.
"""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[2]
NATIVE_PREPARATION = '''
import sys
import tempfile
from pathlib import Path
from unittest.mock import patch
sys.path.insert(0, str(Path.cwd() / 'tests/bindings'))
import agreement_setup as setup
temporary_directory = tempfile.TemporaryDirectory
def isolated_temporary(*args, **kwargs):
    kwargs.update(dir=str(Path(tempfile.gettempdir()).resolve()), prefix='grund-805-host-')
    return temporary_directory(*args, **kwargs)
with patch.object(setup.tempfile, 'TemporaryDirectory', isolated_temporary):
    temporary, api, stage, receipt, runtime = setup.prepare()
try:
    assert Path(api.__file__).resolve().parent == Path(receipt['pythonPackage'])
    assert Path(receipt['pythonNative']).is_file()
    assert (stage / 'native/grund.node').is_file()
    assert runtime
    assert receipt['setupCommands']
    assert all(command['exit'] == 0 for command in receipt['setupCommands'])
finally:
    temporary.cleanup()
'''


def checked(argv, **kwargs):
    result = subprocess.run(argv, text=True, capture_output=True, **kwargs)
    if result.returncode:
        raise AssertionError(f"{argv}: exit {result.returncode}\n{result.stdout}\n{result.stderr}")
    return result.stdout


class PythonExtensionBuildTests(unittest.TestCase):
    def test_build_script_emits_darwin_cdylib_link_args(self):
        # A tiny executable uses the production build script verbatim. Its get()
        # needs resolve-config (normally unified through pyo3); the ABI matches
        # the binding. No grund-core compilation or target SDK is needed here.
        lock = tomllib.loads((ROOT / 'Cargo.lock').read_text())
        version = next(p['version'] for p in lock['package'] if p['name'] == 'pyo3-build-config')
        scratch = Path.home() / 'ag/tmp'
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='grund-805-build-script-', dir=scratch) as directory:
            home = Path(directory)
            script = ROOT / 'crates/grund-py/build.rs'
            (home / 'Cargo.toml').write_text(f'''
[package]
name = "grund-extension-build-probe"
version = "0.0.0"
edition = "2024"
[workspace]
[[bin]]
name = "build-script-probe"
path = {json.dumps(str(script))}
[dependencies]
pyo3-build-config = {{ version = "={version}", features = ["resolve-config", "abi3-py310"] }}
[features]
extension-module = []
''')
            environment = dict(os.environ, PYO3_PYTHON=sys.executable,
                               CARGO_TARGET_DIR=str(home / 'target'))
            checked(['cargo', 'build', '--quiet', '--offline', '--features', 'extension-module'],
                    cwd=home, env=environment)
            executable = home / 'target/debug' / ('build-script-probe.exe' if os.name == 'nt'
                                                 else 'build-script-probe')
            for target in ('aarch64-apple-darwin', 'x86_64-apple-darwin', 'x86_64-unknown-linux-gnu'):
                with self.subTest(target=target):
                    output = checked([str(executable)], cwd=home,
                                     env=dict(environment, TARGET=target,
                                              CARGO_FEATURE_EXTENSION_MODULE='1',
                                              CARGO_CFG_TARGET_OS=('macos' if target.endswith('darwin')
                                                                   else 'linux')))
                    # Accept both Cargo directive spellings, preserving order.
                    directives = [line.replace('cargo::', 'cargo:', 1)
                                  for line in output.splitlines()]
                    link_args = [line for line in directives if 'link-arg=' in line]
                    expected = (['cargo:rustc-cdylib-link-arg=-undefined',
                                 'cargo:rustc-cdylib-link-arg=dynamic_lookup']
                                if target.endswith('apple-darwin') else [])
                    self.assertEqual(link_args, expected,
                                     f'{target}: extension setup must target the cdylib; '
                                     f'build script stdout={output!r}')

    @unittest.skipUnless(sys.platform == 'darwin', 'native Darwin build/import requires macOS')
    def test_macos_same_source_preparation_builds_and_imports(self):
        # Port the triage probe without copying its build command or adding flags.
        # Reuse the real agreement target: the full matrix finds these natives
        # fresh later. Isolate imports so the staged grund cannot leak into other
        # tests after its temporary directory is removed.
        checked([sys.executable, '-c', NATIVE_PREPARATION], cwd=ROOT,
                env=dict(os.environ, PYTHONDONTWRITEBYTECODE='1'))


if __name__ == '__main__':
    unittest.main()

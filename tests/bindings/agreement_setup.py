"""Same-source, once-per-comparison host inputs (§REQ-one-answer.proof.matrix).

Builds run when the comparison runs, never during collection. No registry package
or Rust oracle supplies a host answer. Both native libraries come from one explicit
1.95.0 build into one target, and the ready receipt identifies the source and
loaded package paths.
"""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import sysconfig
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
COMMANDS = []


def node_runtime():
    """Use a supported installed runtime; hosted runners keep several in their cache.

    This is a tooling prerequisite, never a registry/native-package fallback
    (§FS-distribution.3.2.3.2).
    """
    candidates = [shutil.which('node')]
    cache = os.environ.get('RUNNER_TOOL_CACHE')
    if cache:
        for major in ('22', '24'):
            candidates.extend(sorted((Path(cache) / 'node').glob(f'{major}.*/x64/node.exe')))
            candidates.extend(sorted((Path(cache) / 'node').glob(f'{major}.*/x64/bin/node')))
    for candidate in candidates:
        if candidate and Path(candidate).is_file():
            version = checked([candidate, '--version'])
            if version.lstrip('v').split('.')[0] in ('22', '24'):
                return str(candidate), version
    raise RuntimeError('command agreement requires installed Node 22 or 24 (PATH or RUNNER_TOOL_CACHE)')


def checked(argv, **kwargs):
    start = time.monotonic()
    result = subprocess.run(list(map(str, argv)), cwd=ROOT, capture_output=True,
                            text=True, **kwargs)
    COMMANDS.append({'argv': list(map(str, argv)), 'seconds': time.monotonic() - start,
                     'exit': result.returncode})
    if result.returncode:
        raise RuntimeError(f'{argv}: {result.stdout}\n{result.stderr}')
    return result.stdout.strip()


# Installs the built cdylib with the Node builder's own file name, layout and metadata.
INSTALL = ("const [, builder, root, out, debug, target] = process.argv;"
           " const {install, library} = await import(builder);"
           " install(root, out, (await import('node:path')).join(debug, library(target)), target);")


def agreement_target():
    """This checkout's 1.95.0 target, nested in the workspace's cargo target.

    The `rust` jobs cache that target whole, so a warm job finds the natives'
    dependency tree compiled (§REQ-one-answer.proof.delivery.payment). It stays one
    target per checkout, keyed like the Node acceptance's (§AR-ci.3.4.1.2).
    """
    metadata = json.loads(checked(['cargo', 'metadata', '--no-deps', '--format-version', '1',
                                   '--locked']))
    key = hashlib.sha256(os.fsencode(ROOT.resolve())).hexdigest()[:16]
    return Path(metadata['target_directory']) / 'agreement' / key


def prepare():
    """Build the actual mixed Python package and Node package for one host.

    Every lane's host prepares its own packages; the first one's build compiles the
    natives, and a later one's finds them fresh. One invocation builds both natives,
    so the dependency tree and `grund-core` compile once rather than once per binding.
    """
    sha = checked(['git', 'rev-parse', 'HEAD'])
    node, node_version = node_runtime()
    toolchains = checked(['rustup', 'toolchain', 'list'])
    if not any(line.startswith('1.95.0-') for line in toolchains.splitlines()):
        checked(['rustup', 'toolchain', 'install', '1.95.0', '--profile', 'minimal'])
    host = next(line.split(': ', 1)[1] for line in checked(['rustc', '+1.95.0', '-vV']).splitlines()
                if line.startswith('host: '))
    scratch = Path.home() / 'ag/tmp'
    scratch.mkdir(parents=True, exist_ok=True)
    # Windows cannot delete the native module this process still has loaded.
    temporary = tempfile.TemporaryDirectory(prefix='grund-agreement-', dir=scratch,
                                            ignore_cleanup_errors=True)
    home = Path(temporary.name)
    target = agreement_target()
    environment = dict(os.environ, PYO3_PYTHON=sys.executable, CARGO_INCREMENTAL='0')
    checked(['cargo', '+1.95.0', 'build', '--quiet', '--locked', '-p', 'grund-py',
             '-p', 'grund-node', '--features', 'grund-py/extension-module',
             '--target-dir', target], env=environment)
    package = home / 'python/grund'
    shutil.copytree(ROOT / 'crates/grund-py/python/grund', package)
    library = '_native.dll' if os.name == 'nt' else (
        'lib_native.dylib' if sys.platform == 'darwin' else 'lib_native.so')
    native = package / ('_native' + sysconfig.get_config_var('EXT_SUFFIX'))
    shutil.copyfile(target / 'debug' / library, native)
    sys.path.insert(0, str(package.parent))
    import grund
    import grund._native
    assert Path(grund.__file__).resolve().parent == package
    assert Path(grund._native.__file__).resolve() == native
    stage = home / 'node'
    checked([node, ROOT / 'crates/grund-node/stage.mjs', '--out-dir', stage,
             '--target-dir', target])
    checked([node, '--input-type=module', '-e', INSTALL,
             (ROOT / 'crates/grund-node/build.mjs').as_uri(), ROOT, stage / 'native',
             target / 'debug', host])
    assert checked(['git', 'rev-parse', 'HEAD']) == sha, 'source changed during host build'
    receipt = {'sourceSha': sha, 'sourceRoot': str(ROOT.resolve()), 'setupCommands': COMMANDS,
               'nodeRuntime': node, 'nodeVersion': node_version,
               'pythonPackage': str(package), 'pythonNative': str(native),
               'pythonNativeSha256': hashlib.sha256(native.read_bytes()).hexdigest(),
               'nodePackage': str(stage), 'nodeNative': str(stage / 'native/grund.node'),
               'nodeNativeSha256': hashlib.sha256((stage / 'native/grund.node').read_bytes()).hexdigest()}
    return temporary, grund, stage, receipt, node

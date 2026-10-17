"""Persistent real host calls and fixture preparation (§REQ-one-answer.proof.evidence)."""

import dataclasses
import json
import os
from pathlib import Path
import subprocess
import shutil
import sys
import time
from collections.abc import Mapping

from agreement_fixtures import CASES, ROOT, materialize, manifest_request
from agreement_setup import prepare
from agreement_scope import isolated_fixtures, question_scopes

STARTED = time.monotonic()


def plain(value):
    if dataclasses.is_dataclass(value):
        return {f.name: plain(getattr(value, f.name)) for f in dataclasses.fields(value)}
    if isinstance(value, Mapping):
        return {k: plain(v) for k, v in value.items()}
    if isinstance(value, (tuple, list)):
        return [plain(v) for v in value]
    if value is None or type(value) in (str, bool, int, float):
        return value
    raise TypeError(f'non-contract host value: {type(value)}')


def python_call(api, request):
    os.chdir(request['cwd'])
    args = request['args']
    options = dict(request['options'])
    options['root'] = request['root'] if request['explicit'] else None
    try:
        data = plain(getattr(api, request['operation'])(*args, **options))
        envelope = {'failure': None, 'result': data, 'run_cautions': data.get('run_cautions', [])}
    except api.GrundError as error:
        failure = plain(error.failure)
        envelope = {'failure': failure, 'result': None, 'run_cautions': failure['run_cautions']}
    return {'request': request, 'envelope': envelope}


def gate_time(text):
    """One line the cargo suite passes on from a green run (§AR-ci.requirements.5.1)."""
    print(f'gate time: agreement host {text}', file=sys.stderr, flush=True)


def setup_phases(commands):
    """The preparation's compiles, which the host's start alone would not separate."""
    phases = {'toolchain': ['rustup', 'toolchain', 'install'],
              'natives': ['cargo', '+1.95.0', 'build']}
    seconds = {name: sum(c['seconds'] for c in commands if c['argv'][:len(argv)] == argv)
               for name, argv in phases.items()}
    return ', '.join(f'{name} {value:.1f}s' for name, value in seconds.items())


def main():
    temporary, api, stage, provenance, runtime = prepare()
    fixtures = isolated_fixtures()
    node = subprocess.Popen([runtime, str(ROOT / 'tests/bindings/agreement_node.mjs'), str(stage)],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True, encoding='utf-8')
    home = Path(fixtures.name)
    provenance['fixtureRoot'] = str(home)
    print(json.dumps({'ready': provenance}), flush=True)
    phases = setup_phases(provenance['setupCommands'])
    gate_time(f'ready {time.monotonic() - STARTED:.1f}s: {phases}')
    served, ready = 0, time.monotonic()
    try:
        for line in sys.stdin:
            served += 1
            request = json.loads(line)
            op = request['op']
            if op == 'discover':
                answer = sorted(p.name for p in CASES.iterdir() if p.is_dir())
            elif op == 'prepare':
                answer = materialize(home, request['case'], request['variant'])
                answer['manifestRequest'] = manifest_request(answer)
            elif op == 'scope':
                answer = question_scopes(request['context'], request['path'])
            elif op == 'clone':
                source = Path(request['source'])
                assert source.resolve().is_relative_to(home.resolve())
                destination = source.parent.parent / request['name'] / 'repo'
                assert Path(request['name']).name == request['name']
                if destination.exists():
                    shutil.rmtree(destination)
                shutil.copytree(source, destination, symlinks=True)
                answer = str(destination)
            elif op == 'release':
                os.chdir(ROOT)
                node.stdin.write(json.dumps({'control': 'cwd', 'cwd': str(ROOT)}) + '\n')
                node.stdin.flush()
                assert json.loads(node.stdout.readline()) == {'cwd': str(ROOT)}
                case_root = home / 'cases' / request['case']
                assert case_root.parent == home / 'cases'
                shutil.rmtree(case_root)
                answer = None
            elif op == 'python':
                start = time.monotonic()
                answer = python_call(api, request['request'])
                answer['seconds'] = time.monotonic() - start
            elif op == 'node':
                node.stdin.write(json.dumps(request['request']) + '\n')
                node.stdin.flush()
                response = node.stdout.readline()
                assert response, f'Node host exited: {node.poll()}'
                answer = json.loads(response)
                assert answer['request'] == request['request']
            else:
                raise ValueError(f'unknown agreement request: {op}')
            print(json.dumps(answer, ensure_ascii=False), flush=True)
    finally:
        node.stdin.close()
        if node.wait(timeout=30):
            raise RuntimeError('Node host failed')
        temporary.cleanup()
        fixtures.cleanup()
    gate_time(f'served {served} requests in {time.monotonic() - ready:.1f}s')


if __name__ == '__main__':
    main()

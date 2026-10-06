#!/usr/bin/env python3
"""Capture real recipe scenarios for §FS-cochange-recipe.examples through shared goldens."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def demonstrate(template, grund):
    """Present actual isolated recipe results (§FS-cochange-recipe.examples)."""
    scratch = Path.home() / 'ag/tmp'
    scratch.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='cochange-walkthrough-', dir=scratch) as directory:
        repo = Path(directory) / 'repo'
        shutil.copytree(template, repo)
        env = {**os.environ, 'GIT_CONFIG_GLOBAL': os.devnull, 'GIT_CONFIG_NOSYSTEM': '1',
               'GIT_AUTHOR_DATE': '2026-01-01T00:00:00Z', 'GIT_COMMITTER_DATE': '2026-01-01T00:00:00Z'}

        def git(*args):
            """Demonstration setup, confined to scratch (§FS-cochange-recipe.examples)."""
            return subprocess.run(['git', '-C', str(repo), *args], env=env,
                                  capture_output=True, text=True, check=True).stdout.strip()

        git('init', '-q', '--initial-branch=main')
        git('config', 'user.name', 'Walkthrough')
        git('config', 'user.email', 'walkthrough@example.invalid')
        git('config', 'core.autocrlf', 'false')
        git('add', '-A')
        git('commit', '-q', '-m', 'Baseline')
        base = git('rev-parse', 'HEAD')
        policy = Path(__file__).with_name('policy.json')
        cases = [
            ('missing grounding', False, False, (), None),
            ('missing evidence', False, False, ('FS-alpha.1',), None),
            ('spec only', True, False, ('FS-alpha.1',), None),
            ('test only', False, True, ('FS-alpha.1',), None),
            ('unrelated test', True, 'beta', ('FS-alpha.1',), None),
            ('split targets', True, 'beta', ('FS-alpha.1', 'FS-beta.1'), None),
            ('complete evidence', True, True, ('FS-alpha.1',), None),
            ('unchanged contract', False, True, ('FS-alpha.1',), ['spec']),
            ('refactor', False, False, ('FS-alpha.1',), ['spec', 'test']),
        ]
        for name, spec, test, targets, waiver in cases:
            git('reset', '--hard', base)
            (repo / 'src/lib.py').write_text(
                ''.join('# ' + chr(167) + t + '\n' for t in targets) + 'value = 2\n',
                encoding='utf-8', newline='\n')
            if spec:
                with (repo / 'docs/FS-alpha.md').open('a', encoding='utf-8', newline='\n') as stream:
                    stream.write('\nRelated contract edit.\n')
            if test:
                target = 'FS-beta.1' if test == 'beta' else 'FS-alpha.1'
                (repo / 'tests/test_lib.py').write_text('# ' + chr(167) + target + '\nvalue = 2\n',
                                                     encoding='utf-8', newline='\n')
            message = name
            if waiver:
                message += '\n\nGrund-Cochange: ' + json.dumps(dict(paths=['src/lib.py'],
                    missing=waiver, reason='Existing contract' if waiver == ['spec'] else 'Refactor only'))
            git('add', '-A')
            git('commit', '-q', '-m', message)
            result = subprocess.run([sys.executable, str(Path(__file__).with_name('cochange.py')),
                '--repo', str(repo), '--policy', str(policy), '--grund', grund,
                'ci', '--target', base, '--head', 'HEAD'], capture_output=True, text=True)
            report = json.loads(result.stdout)
            print(f'{name}: exit {result.returncode}')
            for row in report['sources']:
                print(f'  {row["path"]}: missing {", ".join(row["missing"]) or "none"}')
                for target in row['targets']:
                    print(f'  {target["id"]}: spec={json.dumps(target["spec_paths"])} '
                          f'test={json.dumps(target["test_paths"])}')
                for entry in row['waivers']:
                    print(f'  waiver {", ".join(entry["missing"])}: {entry["reason"]}; used={entry["used"]}')
            if result.returncode == 2:
                print(json.dumps(report['errors']))
        print(report['limitation'])


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--repo', type=Path, default=Path(__file__).with_name('repo'))
    parser.add_argument('--grund', required=True)
    args = parser.parse_args()
    demonstrate(args.repo.resolve(), str(Path(args.grund).resolve()))

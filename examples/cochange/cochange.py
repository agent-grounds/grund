#!/usr/bin/env python3
"""Opt-in entry point for §FS-cochange-recipe.inputs; no core or standing policy changes."""
import argparse
import json
import os
from pathlib import Path
import shutil
import sys
import tempfile

from common import LIMITATION, Refusal, error, parse_json, validate_policy
from configuration import configuration
from evidence import evaluate
from git_snapshot import Git
from queries import Facts
from waivers import messages, parse_waivers


def arguments():
    """Both entry modes share explicit policy and query inputs (§FS-cochange-recipe.inputs)."""
    parser = argparse.ArgumentParser(description='Opt-in file-level Git co-change evidence')
    parser.add_argument('--repo', required=True)
    parser.add_argument('--policy', required=True)
    parser.add_argument('--grund', required=True)
    modes = parser.add_subparsers(dest='mode', required=True)
    local = modes.add_parser('commit-msg')
    local.add_argument('--base', required=True)
    local.add_argument('--message', required=True)
    ci = modes.add_parser('ci')
    ci.add_argument('--target', required=True)
    ci.add_argument('--head', required=True)
    return parser.parse_args()


def run(args, report, scratch):
    """One evaluator uses exact snapshots and preserves truthful refusals (§FS-cochange-recipe.snapshots)."""
    try:
        report['policy'] = parse_json(Path(args.policy).resolve().read_text(encoding='utf-8'))
    except (OSError, ValueError) as exc:
        raise Refusal('config', f'Cannot read policy JSON: {exc}; provide a readable policy.')
    policy = report['policy']
    validate_policy(policy)
    executable = shutil.which(args.grund)
    if executable is None:
        raise Refusal('query', 'Grund executable unavailable; install supported Grund and supply --grund.')
    executable = str(Path(executable).resolve())
    git = Git(args.repo)
    if args.mode == 'commit-msg':
        base = git.empty() if args.base == 'empty' else git.identity(args.base)
        report['base'] = base
        candidate = git.index(scratch)
    else:
        target = git.identity(args.target)
        candidate = git.identity(args.head)
        report['candidate'] = candidate
        bases = git.text('merge-base', '--all', target['commit'], candidate['commit']).splitlines()
        if len(bases) != 1:
            raise Refusal('git-input', 'No unique merge base; supply connected, unambiguous target/head history.')
        base = git.identity(bases[0])
    report['base'], report['candidate'] = base, candidate
    changes = git.changes(base['tree'], candidate['tree'])
    scopes, waivers = parse_waivers(git, messages(git, args, base, candidate, changes), policy)
    facts, wiring, snapshots = {}, None, {}
    for name, identity in (('candidate', candidate), ('base', base)):
        snapshot = scratch / name
        tracked = git.materialize(identity['tree'], snapshot)
        if name == 'base' and base['commit'] is None:
            continue
        root, signature = configuration(snapshot, policy)
        if wiring is not None and wiring != signature:
            raise Refusal('unsupported-input', 'Project names, aliases or member wiring changed; '
                          'split namespace migrations from this recipe comparison.')
        wiring = signature
        snapshots[name] = (snapshot, root, tracked)
    # Validate namespace wiring before queries, so a removed member is reported
    # as an unsupported migration rather than apparently missing scan evidence.
    for name, (snapshot, root, tracked) in snapshots.items():
        facts[name] = Facts(snapshot, root, tracked, policy, executable,
                            report['errors'], report['check_findings'])
    for change in changes:
        if change.is_source(policy['source_paths']) and change.path not in facts[change.snapshot].cover:
            raise Refusal('scan-scope', f'{change.path}: source move has no cover row; '
                          'include its destination in snapshot scan configuration.', change.path, query='cover')
    report['sources'] = evaluate(changes, facts, policy, scopes, waivers, report['errors'])
    reported_paths = {source['path'] for source in report['sources']}
    report['unused_waivers'] = [waiver for path in sorted(waivers) if path not in reported_paths
                                for waiver in waivers[path]]


def main():
    """One deterministic JSON report and independent exits (§FS-cochange-recipe.output, §FS-cochange-recipe.exit)."""
    args = arguments()
    report = dict(mode=args.mode, policy=None, base=None, candidate=None,
                  limitation=LIMITATION, sources=[], errors=[], check_findings=[], unused_waivers=[])
    scratch_root = Path.home() / 'ag/tmp'
    try:
        scratch_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix='grund-cochange-', dir=scratch_root) as directory:
            try:
                run(args, report, Path(directory))
            except Refusal as exc:
                exc.error['message'] = exc.error['message'].replace(directory, '<snapshot>')
                report['errors'].append(exc.error)
    except (OSError, ValueError, UnicodeError) as exc:
        report['errors'].append(error('unsupported-input',
                                     f'Cannot evaluate input: {exc}; correct input or filesystem access.'))
    report['errors'].sort(key=lambda row: tuple(row.get(k) or '' for k in
                                               ('path', 'code', 'commit', 'query', 'message')))
    sys.stdout.write(json.dumps(report, ensure_ascii=True, separators=(',', ':')) + '\n')
    refusals = {'git-input', 'config', 'unsupported-input', 'check', 'query', 'scan-scope', 'resolution', 'waiver'}
    return 2 if any(e['code'] in refusals for e in report['errors']) else (1 if report['errors'] else 0)


if __name__ == '__main__':
    sys.exit(main())

"""Commit-local trailer scopes for §FS-cochange-recipe.waivers."""
from pathlib import Path

from common import CLASSES, Refusal, parse_json, valid_path


def messages(git, args, base, candidate, changes):
    """Index message or first-parent commit scopes (§FS-cochange-recipe.waivers)."""
    if args.mode == 'commit-msg':
        return [(None, Path(args.message).read_text(encoding='utf-8'), changes)]
    result = []
    for commit in sorted(git.text('rev-list', base['commit'] + '..' + candidate['commit']).splitlines()):
        parents = git.text('rev-list', '--parents', '-n', '1', commit).split()[1:]
        parent = git.identity(parents[0]) if parents else git.empty()
        identity = git.identity(commit)
        result.append((commit, git.text('show', '-s', '--format=%B', commit),
                       git.changes(parent['tree'], identity['tree'])))
    return result


def parse_waivers(git, records, policy):
    """Git trailer parsing plus bounded JSON scopes (§FS-cochange-recipe.waivers)."""
    scopes, waivers = {}, {}
    for commit, message, changes in records:
        paths = {c.path for c in changes if c.is_source(policy['source_paths'])}
        scopes[commit] = paths
        # --parse implies --unfold. Keep continuation lines visible instead so
        # a multiline JSON value cannot be silently accepted as a single line.
        parsed = git.run('-c', 'trailer.separators=:', 'interpret-trailers',
                         '--only-trailers', '--only-input', data=message.encode('utf-8')).decode('utf-8')
        lines = parsed.splitlines()
        entries = []
        for index, line in enumerate(lines):
            key, separator, value = line.partition(':')
            if key != 'Grund-Cochange' or not separator:
                continue
            try:
                if index + 1 < len(lines) and lines[index + 1].startswith((' ', '\t')):
                    raise ValueError('multiline JSON is unsupported')
                entry = parse_json(value)
                if not isinstance(entry, dict) or set(entry) != {'paths', 'missing', 'reason'}:
                    raise ValueError('expected exactly paths, missing, reason')
                for field in ('paths', 'missing'):
                    values = entry[field]
                    if (not isinstance(values, list) or not values or
                            not all(isinstance(v, str) for v in values) or len(set(values)) != len(values)):
                        raise ValueError(f'{field} must be a nonempty unique string array')
                if any(not valid_path(path) for path in entry['paths']):
                    raise ValueError('paths must be exact normalized Git-relative names')
                if any(value not in CLASSES for value in entry['missing']):
                    raise ValueError('missing must name spec and/or test')
                if not isinstance(entry['reason'], str) or not entry['reason'].strip():
                    raise ValueError('reason must be nonempty')
                entries.append(entry)
            except ValueError as exc:
                raise Refusal('waiver', f'Grund-Cochange: {exc}; correct the final JSON trailer.', commit=commit)
        occupied = set()
        for entry in entries:
            for path in sorted(entry['paths']):
                if path not in paths:
                    raise Refusal('waiver', f'{path}: trailer scope is not a source changed by this '
                                  'commit; remove or correct the scope.', path, commit)
                for missing in entry['missing']:
                    key = (path, missing)
                    if key in occupied:
                        raise Refusal('waiver', f'{path}: duplicate/conflicting {missing} trailer scope; '
                                      'keep each commit/path/obligation unique.', path, commit)
                    occupied.add(key)
                waivers.setdefault(path, []).append(dict(commit=commit, path=path,
                    missing=[c for c in CLASSES if c in entry['missing']], reason=entry['reason'],
                    used=False, used_missing=[]))
    return scopes, waivers

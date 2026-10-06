"""Shared-target file evidence and bounded exceptions (§FS-cochange-recipe.evidence)."""
from common import CLASSES, error, matches


def evaluate(changes, facts, policy, scopes, waivers, errors):
    """Both classes share one direct target (§FS-cochange-recipe.evidence)."""
    spec_edits, test_edits = {}, {}
    for change in changes:
        if not change.content:
            continue
        snapshot = facts[change.snapshot]
        for target in snapshot.catalog.values():
            if target['kind'] in policy['eligible_kinds'] and target['path'] == change.path:
                key = (target['project'], target['id'])
                spec_edits.setdefault(key, set()).add(change.path)
        if matches(change.path, policy['test_paths']):
            for key in snapshot.targets(change.path):
                test_edits.setdefault(key, set()).add(change.path)
    sources = []
    for change in changes:
        if not change.is_source(policy['source_paths']):
            continue
        path = change.path
        eligible = facts[change.snapshot].targets(path)
        targets = [dict(project=key[0], id=key[1], spec_paths=sorted(spec_edits.get(key, [])),
                        test_paths=sorted(test_edits.get(key, [])), snapshot=change.snapshot)
                   for key in sorted(eligible, key=lambda key: (key[0] or '', key[1]))]
        rows = waivers.get(path, [])
        missing = ['grounding']
        if targets:
            # Evaluate each target's residual obligations without consuming waivers
            # until the deterministic best target is selected.
            def obligations(target):
                """Bound each missing class to its commit (§FS-cochange-recipe.waivers)."""
                absent = [c for c in CLASSES if not target[c + '_paths']]
                uncovered = {}
                for kind in absent:
                    commits = [commit for commit, paths in scopes.items() if path in paths]
                    uncovered[kind] = [commit for commit in commits if not any(
                        w['commit'] == commit and kind in w['missing'] for w in rows)]
                return absent, uncovered
            def score(target):
                """Choose deterministic residual evidence (§FS-cochange-recipe.output)."""
                _, uncovered = obligations(target)
                return (sum(bool(v) for v in uncovered.values()), target['project'] or '', target['id'])
            selected = min(targets, key=score)
            absent, uncovered = obligations(selected)
            missing = [c for c in CLASSES if uncovered.get(c)]
            for waiver in rows:
                waiver['used_missing'] = [c for c in waiver['missing'] if c in absent]
                waiver['used'] = bool(waiver['used_missing'])
            if missing:
                identity = (selected['project'] + '/' if selected['project'] else '') + selected['id']
                errors.append(error('missing-evidence', f'{path}: {identity}: missing related '
                                    f'{", ".join(missing)} edit or complete commit-local waiver.', path))
                for kind in missing:
                    for commit in uncovered[kind]:
                        errors.append(error('missing-waiver', f'{path}: {identity}: missing '
                                            f'{kind} waiver for this source-changing commit.', path, commit))
        else:
            errors.append(error('missing-grounding', f'{path}: no eligible resolved direct target; '
                                'cite an FS declaration. Evidence waivers cannot supply grounding.', path))
        sources.append(dict(path=path, status=change.status, targets=targets,
                            missing=missing, waivers=sorted(rows, key=lambda w: (
                                w['commit'] or '', tuple(w['missing']), w['reason']))))
    return sources

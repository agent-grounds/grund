"""Policy, refusals and serialization for §FS-cochange-recipe.inputs and output."""
import json
from pathlib import PurePosixPath

LIMITATION = ("file-level related edits; no changed-line coverage, "
              "semantic correctness or test execution proof")
CLASSES = ("spec", "test")


class Refusal(Exception):
    """An input failure takes precedence over obligations (§FS-cochange-recipe.exit)."""
    def __init__(self, code, message, path=None, commit=None, query=None):
        super().__init__(message)
        self.error = error(code, message, path, commit, query)


def error(code, message, path=None, commit=None, query=None):
    """Stable error fields (§FS-cochange-recipe.output)."""
    return dict(code=code, message=message, path=path, commit=commit, query=query)


def json_object(pairs):
    """Reject duplicate JSON keys (§FS-cochange-recipe.waivers)."""
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key {key}")
        result[key] = value
    return result


def parse_json(text):
    """Strict transport JSON (§FS-cochange-recipe.snapshots)."""
    return json.loads(text, object_pairs_hook=json_object,
                      parse_constant=lambda value: (_ for _ in ()).throw(
                          ValueError(f"invalid JSON constant {value}")))


def valid_path(path, prefix=False):
    """Exact Git-relative paths, optionally directory prefixes (§FS-cochange-recipe.inputs)."""
    if not isinstance(path, str) or not path or '\x00' in path or '\\' in path:
        return False
    if any(c in path for c in '*?[]') or path.startswith('/'):
        return False
    value = path[:-1] if prefix and path.endswith('/') else path
    return (value not in ('', '.') and
            all(p not in ('', '.', '..') for p in value.split('/')) and
            str(PurePosixPath(value)) == value)


def matches(path, selections):
    """Classification is explicit, without globs (§FS-cochange-recipe.inputs)."""
    return any(path.startswith(s) if s.endswith('/') else path == s for s in selections)


def validate_policy(policy):
    """Fix both edit classes and reject ambiguous classifications (§FS-cochange-recipe.inputs)."""
    keys = {'config_root', 'source_paths', 'test_paths', 'eligible_kinds'}
    if not isinstance(policy, dict) or set(policy) != keys:
        raise Refusal('config', 'Policy must contain exactly config_root, source_paths, '
                      'test_paths and eligible_kinds; correct the policy JSON.')
    root = policy['config_root']
    if root != '.' and not valid_path(root):
        raise Refusal('config', 'config_root must be a normalized Git-relative directory.')
    for key in ('source_paths', 'test_paths'):
        selections = policy[key]
        if (not isinstance(selections, list) or not selections or
                not all(valid_path(s, prefix=True) for s in selections) or
                len(set(selections)) != len(selections)):
            raise Refusal('config', f'{key} must contain unique exact paths or directory prefixes.')
    if policy['eligible_kinds'] != ['FS']:
        raise Refusal('config', 'eligible_kinds currently supports exactly ["FS"].')
    for source in policy['source_paths']:
        for test in policy['test_paths']:
            if matches(source, [test]) or matches(test, [source]):
                raise Refusal('config', f'Source/test selections overlap: {source!r}, {test!r}.')

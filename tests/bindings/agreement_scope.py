"""Independent discovery/report bases from fixture inputs (§FS-config.1,
§FS-config.3.6.1, §FS-workspace.5.1, §REQ-one-answer.proof.matrix).

This reads configuration location/presentation only, never an engine answer or
expected declarations. Refused TOML remains input, with its parse error recorded.
"""

from pathlib import Path
import os
import tempfile
import tomllib


def config_file(directory):
    return next((p for p in (directory / 'grund.toml', directory / '.agents/grund.toml')
                 if p.is_file()), None)


def isolated_fixtures():
    """Build caches stay under ag/tmp; disposable inputs honor fixture TMPDIR."""
    parent = Path(os.environ.get('TMPDIR') or
                  ('/dev/shm' if Path('/dev/shm').is_dir() else tempfile.gettempdir())).resolve()
    for ancestor in (parent, *parent.parents):
        assert config_file(ancestor) is None, f'fixture ancestor configuration: {ancestor}'
    return tempfile.TemporaryDirectory(prefix='grund-agreement-inputs-', dir=parent)


def read_config(path):
    if path is None:
        return {}, None
    try:
        return tomllib.loads(path.read_text(encoding='utf-8')), None
    except (ValueError, OSError) as error:
        return {}, str(error)


def scope(root, cwd, path, explicit, surface):
    """Resolve one defined base, without trying alternative existing source paths."""
    root, cwd, path = Path(root).resolve(), Path(cwd).resolve(), Path(path).resolve()
    start = path.parent if path.is_file() else path
    config = next((config_file(p) for p in (start, *start.parents)
                   if config_file(p) is not None), None)
    if config:
        owner = config.parent.parent if config.parent.name == '.agents' else config.parent
        data, parse_error = read_config(config)
        # A member without its own config still owns a member-local invocation.
        members = data.get('workspace', {})
        members = members if isinstance(members, dict) else {}
        candidates = []
        member_names = members.get('members', [])
        optional_names = members.get('optional_members', [])
        member_names = member_names if isinstance(member_names, list) else []
        optional_names = optional_names if isinstance(optional_names, list) else []
        for member in member_names + optional_names:
            if not isinstance(member, str):
                continue
            if member.endswith('/*'):
                parent = (owner / member[:-2]).resolve()
                if start.is_relative_to(parent) and start != parent:
                    candidate = parent / start.relative_to(parent).parts[0]
                else:
                    continue
            else:
                candidate = (owner / member).resolve()
            if start.is_relative_to(candidate) and candidate != owner:
                candidates.append(candidate)
        if candidates:
            owner = max(candidates, key=lambda p: len(p.parts))
            config = config_file(owner)
            data, parse_error = read_config(config)
    else:
        parse_error = None
        data = {}
        owner = (start if surface in ('python', 'node') or
                 (surface == 'editor' and explicit and path.is_dir()) else cwd)
    assert owner.is_relative_to(root), f'{surface} loaded outside the copy: {owner}'
    if config:
        assert config.resolve().is_relative_to(root), f'config outside the copy: {config}'
    presentation = data.get('presentation', {})
    presentation = presentation if isinstance(presentation, dict) else {}
    output = data.get('output', {}) if data.get('grund_config_version') != 2 else (
        presentation.get('output', {}))
    output = output if isinstance(output, dict) else {}
    report = owner if output.get('relative_paths', True) else start
    return {'projectRoot': str(owner), 'configFile': str(config) if config else None,
            'invocationBase': str(start), 'reportBase': str(report), 'parseError': parse_error}


def question_scopes(context, path=None):
    path = path or context['path']
    scopes = {s: scope(context['saved'], context['cwd'], path, context['explicit'], s)
              for s in ('cli', 'python', 'node')}
    editor_path = Path(context['base']) / Path(path).relative_to(context['saved'])
    editor_cwd = Path(context['base']) / Path(context['cwd']).relative_to(context['saved'])
    scopes['editor'] = scope(context['base'], editor_cwd, editor_path,
                             context['explicit'], 'editor')
    return scopes

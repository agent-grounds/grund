"""Faithful isolated corpus materialization (§REQ-one-answer.proof.matrix).

Result goldens never enter discovery. Each returned context includes independently
read source bytes, manifest contents and verified symlink targets, so the Rust
acceptance can validate applicability instead of accepting a coverage label.
"""

import hashlib
import os
from pathlib import Path
import shutil
from agreement_scope import question_scopes

ROOT = Path(__file__).resolve().parents[2]
CASES = ROOT / 'tests/e2e/cases'
VARIANTS = ('whole-tree', 'narrowed', 'path-argument', 'editor-overlay')


def files(root):
    """Walk the physical fixture without traversing symlink cycles."""
    for directory, dirs, names in os.walk(root, followlinks=False):
        dirs[:] = sorted(d for d in dirs if not (Path(directory) / d).is_symlink())
        for name in sorted(names):
            path = Path(directory) / name
            if not path.is_symlink():
                yield path


def inventory(root):
    return {str(p.relative_to(root)).replace('\\', '/'): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in files(root)}


def links(case, root):
    manifest = case / 'symlinks'
    result = []
    if not manifest.exists():
        return result
    for number, line in enumerate(manifest.read_text(encoding='utf-8').splitlines(), 1):
        if not line.strip() or line.lstrip().startswith('#'):
            continue
        assert line.count('->') == 1, f'{case.name}: symlinks:{number}'
        link, target = (half.strip() for half in line.split('->'))
        assert link and target and '\\' not in link
        assert not Path(link).is_absolute() and '..' not in Path(link).parts
        destination = root / link
        assert destination.parent.resolve().is_relative_to(root.resolve())
        landing = destination.parent / target
        destination.symlink_to(target, target_is_directory=landing.is_dir())
        assert os.readlink(destination) == target
        result.append({'link': link, 'target': target,
                       'resolved': str(destination.resolve()), 'exists': destination.exists()})
    assert result, f'{case.name}: empty symlinks manifest'
    return result


def text(path, default=None):
    return path.read_bytes().decode('utf-8') if path.exists() else default


def materialize(home, name, variant):
    assert variant in VARIANTS
    case = CASES / name
    assert case.parent == CASES and case.is_dir()
    destination = home / 'cases' / name / variant
    if destination.exists():
        shutil.rmtree(destination)
    base = destination / 'base/repo'
    source = case / 'repo'
    manifests = {p.name: text(p) for p in case.iterdir() if p.is_file()}
    context = {'case': name, 'variant': variant, 'original': str(case),
               'manifests': manifests, 'inputExists': source.is_dir(),
               'base': str(base), 'saved': str(base), 'links': [], 'overlays': {},
               'sourceInventory': {}, 'savedInventory': {}, 'absent': None}
    if not source.is_dir():
        base.mkdir(parents=True)
        context['absent'] = 'no input repository: command grammar/process case'
    else:
        shutil.copytree(source, base, symlinks=True)
        context['sourceInventory'] = inventory(source)
        assert inventory(base) == context['sourceInventory']
        try:
            context['links'] = links(case, base)
        except OSError as error:
            # Only a Windows privilege refusal is a platform exemption. A malformed
            # fixture, bad link type or Unix failure remains a harness failure.
            if os.name != 'nt' or getattr(error, 'winerror', None) != 1314:
                raise
            context['absent'] = 'Windows refused symlink creation (winerror 1314)'
            context['symlinkError'] = {'winerror': error.winerror, 'message': str(error)}
    candidates = list(files(base))
    readable = []
    for path in candidates:
        if path.name in ('grund.toml', 'AGENTS.md') or any(p.startswith('.') for p in path.relative_to(base).parts):
            continue
        try:
            path.read_text(encoding='utf-8')
        except (UnicodeError, OSError):
            continue
        readable.append(path)
    readable.sort()
    subdirs = sorted({p.parent.relative_to(base) for p in readable if p.parent != base})
    all_directories = []
    for directory, dirs, _ in os.walk(base, followlinks=False):
        dirs[:] = sorted(d for d in dirs if not (Path(directory) / d).is_symlink())
        all_directories.extend(Path(directory, d).relative_to(base) for d in dirs)
    all_directories.sort()
    if not subdirs:
        subdirs = all_directories
    if variant == 'editor-overlay' and not context['absent']:
        editable = next((p for p in readable if p.suffix == '.md'),
                        readable[0] if readable else base / 'agreement-overlay.md')
        saved = destination / 'saved/repo'
        shutil.copytree(source, saved, symlinks=True)
        links(case, saved)
        relative = editable.relative_to(base)
        # §FS-scan.text.buffer-only: a tree with no editable source still gets
        # a held new file and its saved control, including configurations that
        # exclude it. No synthetic file enters the unedited input.
        held = editable.read_text(encoding='utf-8') + '\n' if editable.exists() else 'Agreement buffer control.\n'
        (saved / relative).write_text(held, encoding='utf-8', newline='')
        assert (saved / relative).read_bytes() == held.encode('utf-8')
        assert not editable.exists() or editable.read_bytes() != held.encode('utf-8')
        context.update(saved=str(saved), overlays={str(editable): held},
                       overlayRelative=str(relative), heldBytes=held, bufferOnly=not editable.exists())
    saved = Path(context['saved'])
    context['savedInventory'] = inventory(saved)
    context['textFiles'] = [str(p.relative_to(base)).replace('\\', '/') for p in readable]
    context['files'] = sorted(str(p.relative_to(base)).replace('\\', '/') for p in candidates)
    context['directories'] = sorted(str(p).replace('\\', '/') for p in all_directories)
    if variant == 'narrowed' and not subdirs and not context['absent']:
        context['absent'] = 'no proper directory below the input root'
    if variant == 'path-argument' and not candidates and not subdirs and not context['absent']:
        context['absent'] = 'no proper file or directory path below the input root'
    relative = str(subdirs[0]) if variant == 'narrowed' and subdirs else (
        (context['textFiles'][0] if readable else context['files'][0] if candidates else str(subdirs[0]))
        if variant == 'path-argument' and (candidates or subdirs) else '.')
    context['relative'] = relative
    context['path'] = str(saved / relative)
    context['editorPath'] = str(base / relative)
    context['cwd'] = str(saved / relative) if variant == 'narrowed' else str(saved)
    context['explicit'] = variant != 'narrowed'
    if not context['absent']:
        context['scopes'] = question_scopes(context)
    return context


def manifest_request(context):
    """Preserve literal argv/stdin/cwd, mapping both repo placeholders to copies."""
    manifests = context['manifests']
    root = context['base']
    cwd = manifests.get('command.cwd', '').strip()
    assert cwd in ('', '{repo}', '{repo_copy}')
    return {'cwd': root if cwd else str(ROOT),
            'stdin': manifests.get('command.stdin'),
            'external': 'command.external' in manifests}

"""Exact Git/index snapshots and comparisons for §FS-cochange-recipe.snapshots."""
from dataclasses import dataclass
import errno
import os
from pathlib import Path
import shutil
import subprocess

from common import Refusal, matches


@dataclass(frozen=True)
class Change:
    """A NUL-safe diff record with blob-based evidence (§FS-cochange-recipe.snapshots)."""
    path: str
    status: str
    snapshot: str
    content: bool
    old_path: str

    def is_source(self, selections):
        """Source moves retain their old or new role (§FS-cochange-recipe.snapshots)."""
        return matches(self.path, selections) or matches(self.old_path, selections)


class Git:
    """All comparisons use objects; no live-tree fallback (§FS-cochange-recipe.snapshots)."""
    def __init__(self, repo):
        """Local complete history only (§FS-cochange-recipe.snapshots)."""
        self.repo = Path(repo).resolve()
        # Git -C resolves a relative active index from the repository root.
        # Capture it before isolating Git environments (§FS-cochange-recipe.snapshots).
        active = os.environ.get('GIT_INDEX_FILE')
        self.active_index = self.repo / active if active is not None else None
        self.env = {k: v for k, v in os.environ.items() if not k.startswith('GIT_')}
        self.env.update(GIT_CONFIG_NOSYSTEM='1', GIT_CONFIG_GLOBAL=os.devnull,
                        GIT_OPTIONAL_LOCKS='0')
        if Path(self.text('rev-parse', '--show-toplevel')).resolve() != self.repo:
            raise Refusal('git-input', '--repo must name the Git root; supply its top-level path.')
        if self.text('rev-parse', '--is-shallow-repository') != 'false':
            raise Refusal('git-input', 'Shallow history is unsupported; fetch complete history first.')

    def run(self, *args, data=None, extra=None):
        """Git failures retain actionable diagnostics (§FS-cochange-recipe.snapshots)."""
        result = subprocess.run(['git', '-C', str(self.repo), *args], input=data,
                                capture_output=True, env={**self.env, **(extra or {})})
        if result.returncode:
            detail = result.stderr.decode('utf-8', errors='replace').strip()
            raise Refusal('git-input', f'Git {args[0]} failed: {detail}. '
                          'Verify arguments and make the required local objects available.')
        return result.stdout

    def text(self, *args, **kwargs):
        """Decode metadata only; path records remain NUL-separated (§FS-cochange-recipe.snapshots)."""
        return self.run(*args, **kwargs).decode('utf-8').strip()

    def empty(self):
        """Root comparisons use Git's own empty tree (§FS-cochange-recipe.inputs)."""
        return dict(commit=None, tree=self.text('hash-object', '-w', '-t', 'tree', '--stdin', data=b''))

    def identity(self, revision):
        """Resolve explicit full commit/tree identities (§FS-cochange-recipe.output)."""
        commit = self.text('rev-parse', '--verify', '--end-of-options', revision + '^{commit}')
        return dict(commit=commit, tree=self.text('rev-parse', commit + '^{tree}'))

    def index(self, scratch):
        """Copy the index before write-tree, refusing unsupported forms (§FS-cochange-recipe.snapshots)."""
        index = self.active_index
        if index is None:
            index = Path(self.text('rev-parse', '--git-path', 'index'))
        if not index.is_absolute():
            index = self.repo / index
        copied = scratch / 'index'
        if index.exists():
            shutil.copyfile(index, copied)
        env = {'GIT_INDEX_FILE': str(copied)}
        if self.text('rev-parse', '--shared-index-path', extra=env):
            raise Refusal('unsupported-input', 'Split index: use git update-index --no-split-index.')
        sparse = subprocess.run(['git', '-C', str(self.repo), 'config', '--bool',
                                 'core.sparseCheckout'], env=self.env, capture_output=True)
        if sparse.stdout.strip() == b'true':
            raise Refusal('unsupported-input', 'Sparse index/checkout: disable sparse checkout first.')
        staged = self.run('ls-files', '--stage', '-z', extra=env)
        for record in staged.split(b'\0'):
            if record and record.split(b'\t', 1)[0].split()[-1] != b'0':
                raise Refusal('unsupported-input', 'Unmerged index: resolve all conflicts first.')
        # --debug exposes CE_INTENT_TO_ADD; stage-0's empty blob is also a valid real file.
        debug = self.run('ls-files', '--debug', '-z', extra=env)
        import re
        if any(int(flag, 16) & 0x20000000 for flag in re.findall(rb'flags: ([0-9a-f]+)', debug)):
            raise Refusal('unsupported-input', 'Intent-to-add index: stage complete contents first.')
        return dict(commit=None, tree=self.text('write-tree', extra=env))

    def changes(self, base, candidate):
        """Fixed rename detection, exact names and content evidence (§FS-cochange-recipe.snapshots)."""
        records = self.run('diff-tree', '-r', '--no-commit-id', '--raw', '-z',
                           '--no-abbrev', '--find-renames=50%', base, candidate).split(b'\0')
        changes = []
        i = 0
        while i < len(records) and records[i]:
            meta = records[i].decode('ascii').split()
            i += 1
            old = decode_path(records[i]); i += 1
            status = meta[4][0]
            path = old
            if status == 'R':
                path = decode_path(records[i]); i += 1
            if status not in 'AMDRT':
                raise Refusal('unsupported-input', f'Unsupported Git change {status}; use ordinary blobs.')
            changes.append(Change(path, {'A': 'added', 'D': 'deleted', 'R': 'renamed'}.get(
                status, 'modified'), 'base' if status == 'D' else 'candidate', meta[2] != meta[3], old))
        return sorted(changes, key=lambda change: change.path)

    def materialize(self, tree, destination):
        """Read tracked blob bytes and modes without filters (§FS-cochange-recipe.snapshots)."""
        destination.mkdir()
        entries = []
        for record in self.run('ls-tree', '-r', '-z', tree).split(b'\0'):
            if not record:
                continue
            metadata, name = record.split(b'\t', 1)
            mode, kind, oid = metadata.decode('ascii').split()
            path = decode_path(name)
            if kind != 'blob' or mode not in ('100644', '100755', '120000'):
                raise Refusal('unsupported-input', f'{path}: submodules/unsupported modes; '
                              'supply a repository of tracked files.', path)
            if any(part in ('.git', '..', '.') for part in path.split('/')):
                raise Refusal('unsupported-input', f'{path}: unsupported tracked path.', path)
            entries.append((mode, oid, path))
        data = self.run('cat-file', '--batch', data=''.join(oid + '\n' for _, oid, _ in entries).encode())
        position = 0
        links = []
        for mode, oid, path in entries:
            end = data.index(b'\n', position)
            header = data[position:end].decode().split()
            if header[:2] != [oid, 'blob']:
                raise Refusal('git-input', f'{path}: missing blob; restore local objects.', path)
            size = int(header[2]); position = end + 1
            blob = data[position:position + size]; position += size + 1
            output = destination / path
            try:
                output.parent.mkdir(parents=True, exist_ok=True)
                if mode == '120000':
                    target = blob.decode('utf-8')
                    output.symlink_to(target)
                    links.append((output, path))
                else:
                    output.write_bytes(blob)
                    output.chmod(0o755 if mode == '100755' else 0o644)
            except (OSError, ValueError, UnicodeError) as exc:
                raise Refusal('unsupported-input', f'{path}: cannot materialize tracked path/mode '
                              f'({type(exc).__name__}); use a host that supports it or correct the path.', path)
        # Strict resolution catches cycles on Python 3.13+ too. All links must
        # exist before checking (§FS-cochange-recipe.snapshots).
        for output, path in links:
            try:
                try:
                    resolved = output.resolve(strict=True)
                except FileNotFoundError:
                    resolved = output.resolve()
            except (RuntimeError, OSError) as exc:
                if isinstance(exc, OSError) and exc.errno != errno.ELOOP:
                    raise
                raise Refusal('unsupported-input', f'{path}: symlink cycle; '
                              'replace it with a bounded non-cyclic link.', path)
            if not resolved.is_relative_to(destination):
                raise Refusal('unsupported-input', f'{path}: symlink leaves snapshot; '
                              'use a bounded symlink.', path)
        # A nested Git context prevents parent ignore rules from erasing the scan.
        isolated = {**self.env, 'GIT_CONFIG_COUNT': '1', 'GIT_CONFIG_KEY_0': 'init.defaultBranch',
                    'GIT_CONFIG_VALUE_0': 'snapshot'}
        for args in (['init', '-q'], ['add', '-f', '--all']):
            result = subprocess.run(['git', '-C', str(destination), *args],
                                    env=isolated, capture_output=True)
            if result.returncode:
                raise Refusal('git-input', 'Cannot initialize isolated snapshot Git context; check disk access.')
        return {path for _, _, path in entries}


def decode_path(value):
    """Refuse unsupported byte paths instead of lossy matching (§FS-cochange-recipe.snapshots)."""
    try:
        return value.decode('utf-8')
    except UnicodeError:
        raise Refusal('unsupported-input', 'Non-UTF-8 Git path: rename it to a UTF-8 path.')

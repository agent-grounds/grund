"""Snapshot-local config boundaries for §FS-cochange-recipe.snapshots."""
import glob
from pathlib import Path
import tomllib

from common import Refusal


def configuration(snapshot, policy):
    """Stable wiring with snapshot-local config (§FS-cochange-recipe.snapshots)."""
    root = snapshot / policy['config_root']
    signatures = {}
    visited = set()

    def bounded(home, value):
        """External fact/member roots refuse (§FS-cochange-recipe.snapshots)."""
        if not isinstance(value, str) or not (home / value).resolve().is_relative_to(snapshot):
            raise Refusal('unsupported-input', f'Config root {value!r} leaves the Git tree; '
                          'commit local roots instead.')

    def visit(home):
        """Visit each config root once (§FS-cochange-recipe.snapshots)."""
        home = home.resolve()
        if home in visited:
            return
        visited.add(home)
        config = home / 'grund.toml'
        if not config.is_file():
            config = home / '.agents/grund.toml'
        try:
            value = tomllib.loads(config.read_text(encoding='utf-8'))
        except (OSError, ValueError) as exc:
            raise Refusal('config', f'Cannot read snapshot Grund config: {exc}; correct tracked config.')
        workspace = value.get('workspace', {})
        kinds = value.get('kinds', [])
        scan = value.get('scan', {})
        if (not isinstance(workspace, dict) or not isinstance(scan, dict) or
                not isinstance(kinds, list) or any(not isinstance(kind, dict) for kind in kinds)):
            raise Refusal('config', 'Grund config tables have invalid types; correct tracked config.')
        members = workspace.get('members', [])
        optional = workspace.get('optional_members', [])
        includes = scan.get('include', [])
        if any(not isinstance(items, list) or any(not isinstance(item, str) for item in items)
               for items in (members, optional, includes)):
            raise Refusal('config', 'Grund member/include lists must contain strings; correct tracked config.')
        signatures[str(home.relative_to(snapshot))] = (value.get('project_name'), members, optional)
        for kind in kinds:
            for key in ('folder', 'file'):
                if key in kind:
                    bounded(home, kind[key])
        for include in includes:
            bounded(home, include)
        for member in members + optional:
            bounded(home, member)
            for directory in sorted(glob.glob(str(home / member))):
                visit(Path(directory))
    visit(root)
    return root, signatures

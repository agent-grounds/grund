#!/usr/bin/env bash
set -uo pipefail

# Manual live-network acceptance for #1485; run from the checkout root with bash.
# §FS-non-goals.1.1: a persistent authored 404 fails the production external-link sweep.
# §AR-ci.3.1.5: use the existing sweep and its retained failure report.
if (( $# != 0 )); then
    echo 'cannot run: this reproducer takes no arguments or flags' >&2
    exit 2
fi
if ! command -v python3 >/dev/null 2>&1; then
    echo 'cannot run: python3 is required' >&2
    exit 2
fi
PYTHONDONTWRITEBYTECODE=1 python3 - <<'PY'
from datetime import datetime, timezone
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile


def main():
    document = Path('docs/related-work/REL-traceability-tools.md')
    if not document.is_file() or not Path('scripts/check_links.py').is_file():
        raise ValueError('run from the grund checkout root')
    version = subprocess.check_output(['lychee', '--version'], text=True).strip()
    if version != 'lychee 0.23.0':
        raise ValueError(f'requires the sweep pin lychee 0.23.0; observed {version}')
    guides = re.findall(
        r'^- \*\*StrictDoc[^\n]*?\[user guide\]\((https://[^)]+)\)',
        document.read_text(), re.MULTILINE,
    )
    if len(guides) != 1:
        raise ValueError(f'expected one authored StrictDoc guide; observed {len(guides)}')
    url = guides[0]
    commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip()
    print(f'utc: {datetime.now(timezone.utc).isoformat()}\ncommit: {commit}')
    print(f'lychee: {version}\ndocument: {document}\nguide: {url}')
    print('expected: the authored StrictDoc guide passes the unchanged external-link sweep',
          flush=True)
    scratch_root = Path(os.environ.get('TMPDIR', '/var/tmp/grund-1485-tmp'))
    scratch_root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='grund-1485-', dir=scratch_root) as scratch:
        fixture = Path(scratch) / 'guide.md'
        report = Path(scratch) / 'report.md'
        fixture.write_text(f'[StrictDoc user guide]({url})\n')
        command = [sys.executable, 'scripts/check_links.py', '--all',
                   '--report', str(report), str(fixture)]
        print(f'command: {command!r}', flush=True)
        result = subprocess.run(
            command, env={**os.environ, 'TMPDIR': str(scratch_root)},
            text=True, capture_output=True, timeout=420,
        )
        print(result.stdout, end='')
        print(result.stderr, end='', file=sys.stderr)
        kept = report.read_text() if report.exists() else ''
        print(f'observed wrapper exit: {result.returncode}\nreport:\n{kept}', flush=True)
        if result.returncode == 0 and report.exists() and not kept.strip():
            print('observed: guide passes; defect is gone; exit 0')
            return 0
        lines = kept.strip().splitlines()
        if (result.returncode != 0 and len(lines) == 1
                and lines[0].startswith(f'- {url} - ')
                and ' - 404 | ' in lines[0]):
            print('observed: authored guide returns HTTP 404; defect stands; exit 1')
            return 1
        raise ValueError('sweep neither passed nor established the reported guide 404')


try:
    raise SystemExit(main())
except (OSError, ValueError, subprocess.SubprocessError) as exc:
    print(f'cannot conclude: {exc}; expected a successful sweep or its guide 404',
          file=sys.stderr)
    raise SystemExit(2)
PY

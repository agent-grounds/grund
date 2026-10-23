#!/usr/bin/env bash
# Demonstrate §FS-check.3.5.3.1 with an initialized member's stale mark.
set -eu

fixture_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/repo"
mkdir -p "$HOME/ag/tmp"
demo_dir="$(mktemp -d "$HOME/ag/tmp/grund-932-workspace.XXXXXX")"
trap 'rm -rf -- "$demo_dir"' EXIT
cp -R "$fixture_dir" "$demo_dir/repo"
git -C "$demo_dir/repo" init -q
cd "$demo_dir/repo"

# Settle all blocks before introducing drift, independently of init ordering.
for round in 1 2; do
    grund init >/dev/null
    grund init packages/api >/dev/null
    grund init packages/web >/dev/null
done
grund check --only agents-init

python3 - <<'PY'
from pathlib import Path

path = Path("AGENTS.md")
text = path.read_text()
initialized = "- [`api`](packages/api/AGENTS.md)"
stale = "- [`api`](packages/api/) *(not yet initialized)*"
if initialized not in text:
    raise SystemExit("the initialized api entry is missing from the baseline")
path.write_text(text.replace(initialized, stale, 1))
PY

status=0
grund check --only agents-init || status=$?
if [ "$status" -ne 1 ]; then
    echo "expected exit 1 for the deliberately stale entry, got $status" >&2
    exit 1
fi

grund init >/dev/null
grund check --only agents-init

#!/usr/bin/env bash
# Demonstrate §FS-check.3.1.5: a member hint repairs an unqualified citation.
set -eu

fixture_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/repo"
mkdir -p "$HOME/ag/tmp"
demo_dir="$(mktemp -d "$HOME/ag/tmp/grund-1489-missing-alias.XXXXXX")"
trap 'rm -rf -- "$demo_dir"' EXIT
cp -R "$fixture_dir" "$demo_dir/repo"
git -C "$demo_dir/repo" init -q
cd "$demo_dir/repo"

# The checked-in baseline remains valid; change only this disposable copy.
grund check
python3 - <<'PY'
from pathlib import Path

path = Path("docs/functional-spec/FS-platform.md")
text = path.read_text()
marker = chr(0xA7)
qualified = f"{marker}api/FS-session"
unqualified = f"{marker}FS-session"
assert text.count(qualified) == 1
path.write_text(text.replace(qualified, unqualified))
PY

status=0
output="$(grund check --only dangling)" || status=$?
expected="docs/functional-spec/FS-platform.md:3: error: unknown reference FS-session; did you mean api/FS-session or web/FS-session?"
if [ "$status" -ne 1 ] || [ "$output" != "$expected" ]; then
    echo "expected exit 1 and: $expected; got exit $status and: $output" >&2
    exit 1
fi
echo "$output"

# Choose the intended namespace; the hint never resolves the citation for us.
python3 - <<'PY'
from pathlib import Path

path = Path("docs/functional-spec/FS-platform.md")
text = path.read_text()
marker = chr(0xA7)
qualified = f"{marker}api/FS-session"
unqualified = f"{marker}FS-session"
assert text.count(unqualified) == 1
path.write_text(text.replace(unqualified, qualified))
PY
grund check
echo "check after repair: exit 0"

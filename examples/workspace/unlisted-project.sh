#!/usr/bin/env bash
# Demonstrate §FS-declarations.checks.misplaced-declaration.unlisted-project.
set -eu

mkdir -p "$HOME/ag/tmp"
demo_dir="$(mktemp -d "$HOME/ag/tmp/grund-1000-workspace.XXXXXX")"
trap 'rm -rf -- "$demo_dir"' EXIT
cd "$demo_dir"
git init -q
mkdir -p packages/billing packages/notify

cat >grund.toml <<'EOF'
grund_config_version = 2
project_name = "shop"
[schema.kinds.FS]
files = ["requirements.md"]
[workspace]
members = ["packages/billing"]
EOF
cp grund.toml unlisted.toml
for project in billing notify; do
    cat >"packages/$project/grund.toml" <<EOF
grund_config_version = 2
project_name = "$project"
[schema.kinds.FS]
files = ["requirements.md"]
EOF
done
printf '# FS-shop: Shop\n' >requirements.md
printf '# FS-billing: Billing\n' >packages/billing/requirements.md
printf '# Requirements\n\n## FS-email: Receipts are sent by email\n' \
    >packages/notify/requirements.md

status=0
grund check --only misplaced-declaration >unlisted.out || status=$?
cat unlisted.out
if [ "$status" -ne 1 ]; then
    echo "expected exit 1 for the unlisted project's placement error, got $status" >&2
    exit 1
fi
python3 - <<'PY'
from pathlib import Path

expected = (
    'packages/notify/requirements.md:3: error: '
    'FS-email must be declared in requirements.md (single-file kind)'
    ' — packages/notify/grund.toml (project notify) is not read: '
    'no [workspace] members entry lists packages/notify; '
    'kind homes from grund.toml (project shop) apply; '
    'add "packages/notify" to [workspace] members in grund.toml, '
    'or exclude "packages/notify/" with [schema.sources] exclude in grund.toml'
)
assert Path('unlisted.out').read_text().splitlines() == [expected]
path = Path('grund.toml')
path.write_text(path.read_text().replace(
    'members = ["packages/billing"]',
    'members = ["packages/billing", "packages/notify"]',
))
PY

echo 'Membership-only repair:'
grund check --only misplaced-declaration

cp unlisted.toml grund.toml
cat >>grund.toml <<'EOF'
[schema.sources]
exclude = ["packages/notify/"]
EOF
echo 'Exclusion alternative:'
grund check --only misplaced-declaration

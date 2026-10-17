#!/usr/bin/env bash
# §AR-ci.13.6: what a cargo cache holds before main saves it. Every run rebuilds the
# workspace crates from its own sources, so their artifacts and any incremental state
# are dropped from the target directory, and the entry keeps the registry and the
# dependencies' compiled artifacts.
#
# A cache is an optimization only (§AR-ci), so a prune that fails warns and leaves the
# target whole rather than failing the job.
#
#   bash scripts/prune_cargo_cache.sh [TARGET_DIR]    # default: target
set -uo pipefail

target="${1:-target}"
[ -d "$target" ] || exit 0

skip() {
  echo "::warning::prune_cargo_cache.sh: $1; the cache is saved as it stands"
  exit 0
}

# The workspace's own packages, read from the workspace rather than listed here. The bench
# job installs no Python of its own, and a Windows job may only have `python`.
python=$(command -v python || command -v python3) || skip "no python on PATH"
packages=$(cargo metadata --no-deps --format-version 1 --locked \
  | "$python" -c 'import json, sys; print(" ".join(p["name"] for p in json.load(sys.stdin)["packages"]), end="")') \
  || skip "cargo metadata failed"
specs=()
for package in $packages; do
  specs+=(-p "$package")
done
# With no package named, `cargo clean` would empty the whole target.
[ ${#specs[@]} -gt 0 ] || skip "no workspace package found"

# `dev` writes target/debug and `release` target/release; `test` and `bench` share them.
# The command agreement's 1.95.0 targets nest under agreement/, one per checkout
# (tests/bindings/agreement_setup.py), and are pruned the same way.
for dir in "$target" "$target"/agreement/*/; do
  [ -d "$dir" ] || continue
  for profile in dev release; do
    cargo clean --locked --target-dir "$dir" --profile "$profile" "${specs[@]}" \
      || skip "cargo clean --profile $profile failed in $dir"
  done
  rm -rf "$dir"/*/incremental
done

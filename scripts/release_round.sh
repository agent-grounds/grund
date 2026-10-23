#!/usr/bin/env bash
# One round's two candidates, built on the commit checked out (§FS-distribution.4.4.1.1):
# the release commit and the `-dev` advance over it, by the whole preparation, every
# round anew. `scripts/release_candidates.py` runs it and pushes what it commits.
#
#   scripts/release_round.sh <next> <dev>
#
# It commits and pushes nothing else: on success HEAD is the advance and HEAD^ the
# release commit. A refusal of any step stops the helper before a branch is pushed
# (§FS-distribution.4.4.1.4).

set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <next> <dev>" >&2
  exit 2
fi
next=$1 dev=$2

# §FS-distribution.4.2.6: refuse a release the tree's own messages contradict - a
# ramp a pull request merged since the last round landed, say.
python3 scripts/check_release_ramps.py "$next"

# §FS-distribution.4.5: write package versions and both checked CLI version inputs.
python3 scripts/distribution/candidate.py set-version "$next"
cargo update --workspace
# The release section lists the pull requests merged since the tag, read
# from the forge with GH_TOKEN (§FS-distribution.4.6.1); nothing is written first.
python3 scripts/prepare_changelog_release.py prepare "$next"

git config user.name 'github-actions[bot]'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'

# §FS-distribution.4.5: the release index includes the authored version input too.
git add Cargo.toml crates/grund-core/Cargo.toml crates/grund-cli/Cargo.toml crates/grund-lsp/Cargo.toml Cargo.lock crates/grund-py/pyproject.toml tests/e2e/cases/cli-version/expected.stdout tests/integration/command_agreement_facts/cli-version.json docs/changelog.md docs/changelog
git commit -q -m "Release v${next}"

# §FS-distribution.4.4.1: the -dev advance is built here, over the release commit, so
# CI runs on the commit main will hold rather than on one made after the release.

# §FS-distribution.4.5: write and stage both version inputs in this dev candidate.
python3 scripts/distribution/candidate.py set-version "$dev"
git add -A
git commit -q -m "Open ${dev} for development"

#!/bin/sh
# §FS-repository-maintenance.1.1: the clean verb lives under scripts/.
# §FS-repository-maintenance.1.2: ephor.json binds this script for checkout-owned discovery.
# §FS-repository-maintenance.1.3: ephor runs it with the checkout root as its cwd.

# §FS-repository-maintenance.1.4: cargo failure stops cleanup before cache deletion.
set -eu
cargo clean

# §FS-repository-maintenance.1.4: remove valid cache-tagged directories, pruning .git.
find . -name .git -prune -o -type f -name CACHEDIR.TAG -print |
  while IFS= read -r tag; do
    if head -c 43 "$tag" | grep -q '^Signature: 8a477f597d28d172789f06886806bc55'; then
      rm -rf -- "$(dirname -- "$tag")"
    fi
  done

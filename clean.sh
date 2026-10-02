#!/bin/sh
# The clean verb: `ephor clean` runs this at the root of a branch checkout no
# live run holds, to give back the disk its builds took. Exit 0 is done.
set -eu
cargo clean
# Build output left elsewhere in the tree, such as a plan's scratch build under
# panta/: every directory carrying a cache tag (bford.info/cachedir).
find . -name .git -prune -o -type f -name CACHEDIR.TAG -print |
  while IFS= read -r tag; do
    if head -c 43 "$tag" | grep -q '^Signature: 8a477f597d28d172789f06886806bc55'; then
      rm -rf -- "$(dirname -- "$tag")"
    fi
  done

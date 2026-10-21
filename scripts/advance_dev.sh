#!/usr/bin/env bash
# The `-dev` advance both release helpers run as their last act (§FS-distribution.4.1):
# push the tested advance onto `main` by the sha CI passed, never a commit rebuilt here.
#
#   scripts/advance_dev.sh <released> <dev> <candidate-branch> <dev-sha>
#
# §FS-distribution.4.4.1: a rejected push is not yet a refusal. While `main` is still
# the release commit the advance was built over (its parent), the same commit is
# pushed again, a few times with a short backoff (GRUND_ADVANCE_RETRY_DELAY seconds,
# growing per attempt). Only a `main` that has moved is refused as moved; a push that
# keeps failing onto an unmoved `main` names git's own error. Nothing is force-pushed.

set -euo pipefail

if [ "$#" -ne 4 ]; then
  echo "usage: $0 <released> <dev> <candidate-branch> <dev-sha>" >&2
  exit 2
fi
released=$1 dev=$2 branch=$3 sha=$4
attempts=${GRUND_ADVANCE_ATTEMPTS:-4}
delay=${GRUND_ADVANCE_RETRY_DELAY:-5}
release=$(git rev-parse "${sha}^")
log=$(mktemp)
trap 'rm -f "$log"' EXIT

attempt=1
while :; do
  if git push origin "${sha}:refs/heads/main" 2>"$log"; then
    cat "$log" >&2
    break
  fi
  cat "$log" >&2
  # Read main itself rather than guess why the push was refused.
  tip=$(git ls-remote origin refs/heads/main | cut -f1) || tip=""
  if [ "$tip" = "$sha" ]; then
    break # the refused push landed after all
  fi
  if [ -n "$tip" ] && [ "$tip" != "$release" ]; then
    echo "error: main moved past the v${released} release commit before ${dev} could follow it (main is now ${tip})" >&2
    echo "       $branch holds the tested advance; open ${dev} from the new main tip by hand" >&2
    exit 1
  fi
  if [ "$attempt" -ge "$attempts" ]; then
    echo "error: could not push ${dev} onto main after ${attempts} attempts; git's last error:" >&2
    sed 's/^/       /' "$log" >&2
    if [ -n "$tip" ]; then
      echo "       main is still the v${released} release commit ${release}, so pushing ${sha} to main by hand is safe" >&2
    else
      echo "       main could not be read either; if it is still ${release}, pushing ${sha} to main by hand is safe" >&2
    fi
    echo "       $branch holds the tested advance" >&2
    exit 1
  fi
  echo "main is still the v${released} release commit; pushing ${dev} again (attempt $((attempt + 1)) of ${attempts})" >&2
  sleep $((delay * attempt))
  attempt=$((attempt + 1))
done

git push origin --delete "$branch" || true
echo "main now opens ${dev}; release.yml was dispatched for v${released}."

#!/usr/bin/env bash
# Pre-release package-name guard for §FS-distribution.4.
# A claimed name passes when it is free, or when it is established as this
# project's. Only the package endpoint's own 404 makes a name free, and what
# makes an existing package this project's is registry-specific
# (§FS-distribution.1.1): crates.io answers from the registry's own owner
# record, npm and PyPI from the metadata the package declares about itself.
# The bare "grund" name on npm — a dormant low-use squat (§DA-rename-to-grund
# / §DA-pypi-uses-grund-as-the-package-name) — is reported as a notice so the
# release manager can revisit FS-distribution if it changes.

set -euo pipefail

ua="grund-release-name-check/0.1"

# npm and PyPI read ownership off metadata the last publish wrote, so it names
# the repository the package was published from; a move reaches them only with
# the next release, so the former owner passes too (§FS-distribution.1.1).
repo_pattern='github.com[/:](agent-grounds|vjovanov)/grund'

# This project's crates.io identity, held as a constant because it is a
# credential rather than a shape: it is compared literally against an owner
# record's `login` (§FS-distribution.1.1.1).
crates_io_owner="vjovanov"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

http_get() {
  local url="$1"
  local out="$2"
  curl -sS -L -A "$ua" -o "$out" -w '%{http_code}' "$url"
}

metadata_mentions_repo() {
  local file="$1"
  python3 - "$file" "$repo_pattern" <<'PY'
import json
import re
import sys

path, pattern = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as fh:
    data = json.load(fh)

haystack = json.dumps(data, sort_keys=True).lower()
sys.exit(0 if re.search(pattern, haystack) else 1)
PY
}

# One owner response read as `owned`, `untrusted` or `unreadable`, the three
# the diagnostics below are built from (§FS-distribution.1.1.2). Shape is
# validated first, so malformed evidence never reads as an absent owner.
crates_io_owner_verdict() {
  local file="$1"
  python3 - "$file" "$crates_io_owner" <<'PY'
import json
import sys

path, trusted = sys.argv[1], sys.argv[2]
try:
    with open(path, "r", encoding="utf-8") as fh:
        data = json.load(fh)
except (OSError, ValueError):
    print("unreadable")
    raise SystemExit(0)

users = data.get("users") if isinstance(data, dict) else None
if not isinstance(users, list):
    print("unreadable")
    raise SystemExit(0)
if any(not isinstance(r, dict) or "kind" not in r or "login" not in r for r in users):
    print("unreadable")
    raise SystemExit(0)

# Exactly a user record, exactly this login. Display name, profile URL, the
# numeric id and github_username_matches are publisher-supplied or opaque, and
# a team is a different subject (§FS-distribution.1.1.1).
owned = any(r["kind"] == "user" and r["login"] == trusted for r in users)
print("owned" if owned else "untrusted")
PY
}

# crates.io ownership is the registry's own owner record and never the
# package's declared metadata (§FS-distribution.1.1.1); ownership that cannot
# be established stops the release (§FS-distribution.1.1.2).
check_crates_io_claimed_name() {
  local registry="$1"
  local name="$2"
  local url="$3"
  local owners_url="$url/owners"
  local out="$tmpdir/${registry}-${name}.json"
  local owners="$tmpdir/${registry}-${name}-owners.json"
  local code
  local verdict

  if ! code="$(http_get "$url" "$out")"; then
    echo "error: could not reach $registry/$name" >&2
    echo "       $url" >&2
    return 1
  fi

  case "$code" in
    200) ;;
    404)
      echo "ok: $registry/$name is available"
      return 0
      ;;
    *)
      echo "error: could not query $registry/$name (HTTP $code)" >&2
      echo "       $url" >&2
      return 1
      ;;
  esac

  # The package endpoint has said the name exists, so nothing below it may turn
  # the name back into a free one (§FS-distribution.1.1).
  if ! code="$(http_get "$owners_url" "$owners")"; then
    echo "error: could not reach the crates.io owner record for $name" >&2
    echo "       $owners_url" >&2
    return 1
  fi

  if [ "$code" != "200" ]; then
    echo "error: could not determine who owns $registry/$name (HTTP $code)" >&2
    echo "       $owners_url" >&2
    return 1
  fi

  verdict="$(crates_io_owner_verdict "$owners")"
  case "$verdict" in
    owned)
      echo "ok: $registry/$name is owned by this project"
      ;;
    untrusted)
      echo "error: $registry/$name is already taken without trusted owner $crates_io_owner" >&2
      echo "       $owners_url" >&2
      return 1
      ;;
    *)
      echo "error: could not read the crates.io owner record for $name" >&2
      echo "       $owners_url" >&2
      return 1
      ;;
  esac
}

check_claimed_json_name() {
  local registry="$1"
  local name="$2"
  local url="$3"
  local out="$tmpdir/${registry}-${name}.json"
  local code

  code="$(http_get "$url" "$out")"
  case "$code" in
    200)
      if metadata_mentions_repo "$out"; then
        echo "ok: $registry/$name is owned by this project"
      else
        echo "error: $registry/$name is already taken by another project" >&2
        echo "       $url" >&2
        return 1
      fi
      ;;
    404)
      echo "ok: $registry/$name is available"
      ;;
    *)
      echo "error: could not query $registry/$name (HTTP $code)" >&2
      echo "       $url" >&2
      return 1
      ;;
  esac
}

notice_external_json_name() {
  local registry="$1"
  local name="$2"
  local url="$3"
  local out="$tmpdir/${registry}-${name}-external.json"
  local code

  code="$(http_get "$url" "$out")"
  case "$code" in
    200)
      if metadata_mentions_repo "$out"; then
        echo "notice: $registry/$name is owned by this project; docs may no longer need the alternate-name rationale"
      else
        echo "notice: $registry/$name is occupied by an external package as documented"
      fi
      ;;
    404)
      echo "notice: $registry/$name appears available; revisit the alternate-name rationale before publishing"
      ;;
    *)
      echo "warning: could not query documented external collision $registry/$name (HTTP $code)" >&2
      ;;
  esac
}

check_crates_io_claimed_name "crates.io" "grund-core" "https://crates.io/api/v1/crates/grund-core"
check_crates_io_claimed_name "crates.io" "grund" "https://crates.io/api/v1/crates/grund"
check_crates_io_claimed_name "crates.io" "grund-lsp" "https://crates.io/api/v1/crates/grund-lsp"
check_claimed_json_name "npm" "grund-cli" "https://registry.npmjs.org/grund-cli"
check_claimed_json_name "npm" "grund-lsp" "https://registry.npmjs.org/grund-lsp"
check_claimed_json_name "pypi" "grund" "https://pypi.org/pypi/grund/json"
check_claimed_json_name "pypi" "grund-lsp" "https://pypi.org/pypi/grund-lsp/json"

notice_external_json_name "npm" "grund" "https://registry.npmjs.org/grund"

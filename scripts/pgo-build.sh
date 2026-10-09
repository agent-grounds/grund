#!/usr/bin/env bash
# Profile-guided-optimization build of one distributed payload (§DA-pgo-release).
# This is a release/benchmarking tool, not part of the normal development build
# or push/PR CI loop.
#
#   bash scripts/pgo-build.sh [--product grund|grund-lsp|node-addon|python-extension]
#                             [--target-dir DIR] [--evidence FILE] [--profile FILE]
#                             [--container IMAGE] [--sha SHA]
#
# Three phases: build an instrumented payload (`-Cprofile-generate`), train it on
# its own product's workload to produce `.profraw` profiles, merge them, then
# rebuild the payload with `-Cprofile-use` (§FS-distribution-candidate.7.1). The
# `grund` executable trains on the hot command list `crates/grund-cli/benches/
# instructions.rs` benchmarks, against this repo's own conformant tree; the
# language server, the Node addon and the Python extension train through
# `scripts/distribution/train.py` (§AR-benchmarks.1.5). The generated
# `check_large_10k` benchmark is a CI budget input, not release-profile training data.
#
# A profile is keyed by compiler, target, source SHA, product, features and ABI
# (§FS-distribution-candidate.7.2): the key is written beside the merged profile,
# and `--profile` uses an existing one only when its key is this build's.
# `--evidence` writes the generate, train, merge and use record the candidate's
# manifest carries (§FS-distribution-candidate.7.3), its paths as the host's native
# programs read them. On aarch64-pc-windows-msvc, rustc crashing while it compiles
# the instrumented build exits 3 and says so; every other failure exits otherwise,
# a training run that writes no profile with 1, so a caller can tell the one
# failure §FS-distribution-candidate.7.4 lets the Windows arm64 row fall back on
# from all the rest.
#
# `--container IMAGE` runs the Cargo builds and the merge in that image (the
# pinned manylinux2014 image of §FS-distribution.4.8) with this repository and
# the target directory mounted at the same paths, and trains on the host.
#
# Output: <target-dir>/release/<payload> (<target-dir>/container/release/<payload>
# with `--container`), optimized against the merged profile.
# Requires: the `llvm-tools-preview` rustup component (`llvm-profdata`).
#
# `cargo install grund` from source does not run this — a plain `cargo build
# --release` has no profile to use. The release pipeline (§RM-distribution) runs
# this script to produce the distributed binaries; benchmarking can also run it
# when comparing the optimized release artifact.

set -euo pipefail

cd "$(dirname "$0")/.."
repo="$PWD"
product="grund"
target_dir="$repo/target"
evidence=""
profile=""
container=""
sha=""
while [ $# -gt 0 ]; do
  case "$1" in
    --product) product="$2"; shift 2 ;;
    --target-dir) target_dir="$2"; shift 2 ;;
    --evidence) evidence="$2"; shift 2 ;;
    --profile) profile="$2"; shift 2 ;;
    --container) container="$2"; shift 2 ;;
    --sha) sha="$2"; shift 2 ;;
    *) echo "error: unknown argument $1" >&2; exit 2 ;;
  esac
done
case "$product" in
  grund) crate="grund"; features=""; abi="null" ;;
  grund-lsp) crate="grund-lsp"; features=""; abi="null" ;;
  node-addon) crate="grund-node"; features=""; abi="null" ;;
  python-extension) crate="grund-py"; features="extension-module"; abi='"abi3-py310"' ;;
  *) echo "error: unknown product $product" >&2; exit 2 ;;
esac
mkdir -p "$target_dir"
target_dir="$(cd "$target_dir" && pwd)"
# A container's root does not own the checkout, which git otherwise refuses to read.
sha="${sha:-$(git -c safe.directory="$repo" -C "$repo" rev-parse HEAD 2>/dev/null || echo unknown)}"
pgo_dir="$target_dir/pgo-data/$product"
raw="$pgo_dir/raw"
toolchain="${RUSTUP_TOOLCHAIN:-${RUST_TOOLCHAIN:-1.95.0}}"
home="$target_dir/pgo-container-home"
# Host and container builds share a compiler version, so Cargo would take one's
# objects for the other's; the image's builds keep a target of their own.
cargo_target="$target_dir"
[ -z "$container" ] || cargo_target="$target_dir/container"

# Runs one command where the payload is built: here, or in the pinned image.
in_build() {
  if [ -z "$container" ]; then
    "$@"
    return
  fi
  local mounts=(-v "$repo:$repo" -v "$target_dir:$target_dir")
  case "$target_dir/" in "$repo"/*) mounts=(-v "$repo:$repo") ;; esac
  local user=()
  # A rootless daemon maps the container's root to the caller already.
  if ! docker info --format '{{.SecurityOptions}}' 2>/dev/null | grep -q rootless; then
    user=(--user "$(id -u):$(id -g)")
  fi
  docker run --rm "${user[@]}" "${mounts[@]}" -w "$repo" \
    -e HOME="$home" -e CARGO_HOME="$home/cargo" -e RUSTUP_HOME="$home/rustup" \
    -e RUSTUP_TOOLCHAIN="$toolchain" -e RUSTFLAGS="${RUSTFLAGS:-}" \
    -e PYO3_PYTHON=/opt/python/cp310-cp310/bin/python3 \
    "$container" bash -c 'export PATH="$CARGO_HOME/bin:$PATH"; "$@"' bash "$@"
}

if [ -n "$container" ]; then
  mkdir -p "$home"
  in_build bash -c 'set -e
    if [ ! -x "$CARGO_HOME/bin/rustup" ]; then
      curl --proto "=https" --tlsv1.2 -sSf https://sh.rustup.rs \
        | RUSTUP_INIT_SKIP_SUDO_CHECK=yes sh -s -- -y --profile minimal \
          --default-toolchain "$RUSTUP_TOOLCHAIN" --no-modify-path
    fi
    rustup toolchain install "$RUSTUP_TOOLCHAIN" --profile minimal \
      --component llvm-tools-preview >/dev/null'
  rustc_version="$(in_build rustc -vV)"
else
  rustc_version="$("${RUSTC:-rustc}" -vV)"
fi
host="$(printf '%s\n' "$rustc_version" | awk '/^host:/ { print $2 }')"
compiler="rustc $(printf '%s\n' "$rustc_version" | awk '/^release:/ { print $2 }')"

# A path as a native Windows program reads it — rustc's flags, and the evidence's
# readers (§FS-distribution-candidate.5.6) — rather than Git Bash's `/d/a/...`.
native_path() {
  local path="$1"
  if [[ "$host" == *windows* ]] && command -v cygpath >/dev/null 2>&1; then
    cygpath -m "$path"
  else
    printf '%s\n' "$path"
  fi
}

sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{ print $1 }'
  else
    shasum -a 256 "$1" | awk '{ print $1 }'
  fi
}

exe_suffix=""
case "$host" in
  *windows*) exe_suffix=".exe" ;;
esac
case "$product:$host" in
  grund:*) artifact="grund$exe_suffix" ;;
  grund-lsp:*) artifact="grund-lsp$exe_suffix" ;;
  node-addon:*windows*) artifact="grund_node.dll" ;;
  node-addon:*apple*) artifact="libgrund_node.dylib" ;;
  node-addon:*) artifact="libgrund_node.so" ;;
  python-extension:*windows*) artifact="_native.dll" ;;
  python-extension:*apple*) artifact="lib_native.dylib" ;;
  python-extension:*) artifact="lib_native.so" ;;
esac
payload="$cargo_target/release/$artifact"
grund="$payload"

# §FS-distribution-candidate.7.2: everything that changes the code a profile describes.
features_json="[]"
[ -z "$features" ] || features_json="[\"$features\"]"
key="{\"compiler\": \"$compiler\", \"target\": \"$host\", \"source_sha\": \"$sha\", \"product\": \"$product\", \"features\": $features_json, \"abi\": $abi}"

build() {
  local args=(build --release --locked -p "$crate" --target-dir "$cargo_target")
  [ -z "$features" ] || args+=(--features "$crate/$features")
  if [ "$product" = python-extension ] && [[ "$host" == *apple* ]]; then
    # PyO3 leaves the interpreter's symbols to the loading process on macOS.
    args=(rustc --lib "${args[@]:1}" -- -C link-arg=-undefined -C link-arg=dynamic_lookup)
  fi
  RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }$1" in_build cargo "${args[@]}"
}

# §FS-distribution-candidate.7.4: the failure the Windows arm64 row may fall back on.
# Every crate Cargo could not compile is a `-Cprofile-generate` rustc that crashed.
crash="rustc crashed compiling the instrumented build"
compiler_crashed() {
  local crashed failed uncompiled
  crashed="$(grep -c "process didn't exit successfully: .*rustc.*-Cprofile-generate.*(exit code: 0xc0000005, STATUS_ACCESS_VIOLATION)" "$1" || true)"
  failed="$(grep -c "process didn't exit successfully" "$1" || true)"
  uncompiled="$(grep -c "could not compile" "$1" || true)"
  [ "$crashed" -gt 0 ] && [ "$crashed" -eq "$failed" ] && [ "$crashed" -eq "$uncompiled" ]
}

# Builds the instrumented payload. Only on the Windows arm64 host is Cargo's output
# kept and read, and only that crash exits 3; elsewhere a failure keeps its own status.
build_instrumented() {
  if [ "$host" != aarch64-pc-windows-msvc ]; then
    build "$1"
    return
  fi
  local log="$pgo_dir/instrumented.log" status
  set +e
  { build "$1" 2>&1 1>&3 | tee "$log" >&2; status=${PIPESTATUS[0]}; } 3>&1
  set -e
  if [ "$status" -ne 0 ] && compiler_crashed "$log"; then
    echo "error: $product: $crash (exit code: 0xc0000005, STATUS_ACCESS_VIOLATION)" >&2
    exit 3
  fi
  return "$status"
}

find_python() {
  local candidate
  for candidate in "${GRUND_PGO_PYTHON:-}" python3 python /opt/python/cp312-cp312/bin/python3; do
    # A Windows store alias is on PATH yet runs nothing, so each one is asked.
    if [ -n "$candidate" ] && "$candidate" -c 'import sys; sys.exit(sys.version_info < (3, 6))' \
      >/dev/null 2>&1; then
      printf '%s\n' "$candidate"
      return
    fi
  done
  echo "error: no Python to run the $product training workload; set GRUND_PGO_PYTHON" >&2
  exit 1
}

if [ -n "$profile" ]; then
  if [ ! -f "$profile" ]; then
    echo "error: profile $profile does not exist" >&2
    exit 1
  fi
  recorded="$(sed -n 's/^key: //p' "$profile.key" 2>/dev/null || true)"
  if [ "$recorded" != "$key" ]; then
    echo "error: profile $profile is keyed ${recorded:-by nothing}, not $key;" \
      "a build uses only the profile of its own key (§FS-distribution-candidate.7.2)" >&2
    exit 1
  fi
  profdata="$profile"
else
  profdata="$pgo_dir/merged.profdata"
  rm -rf "$pgo_dir"
  mkdir -p "$raw"

  echo "==> 1/3  build instrumented $product (-Cprofile-generate)"
  build_instrumented "-Cprofile-generate=$(native_path "$raw")"
  # Build scripts are instrumented too; only the training run's profiles count.
  rm -rf "$raw"
  mkdir -p "$raw"
  mkdir -p "$pgo_dir/instrumented"
  cp "$payload" "$pgo_dir/instrumented/$artifact"
  generate_sha256="$(sha256 "$payload")"

  echo "==> 2/3  training run — the $product workload"
  if [ "$product" = grund ]; then
    training="$(sed -n 's/^ *"\$grund" \(.*\) "\$repo".*$/"grund \1"/p' "$repo/scripts/pgo-build.sh" | paste -sd, -)"
    training="[${training//,/, }]"
  else
    python="$(find_python)"
    training="$("$python" scripts/distribution/train.py describe "$product")"
  fi
fi

# Keep this self-repo hot command list in sync with crates/grund-cli/benches/instructions.rs.
# Exit codes are irrelevant here (a non-canonical tree makes `fmt --check` exit
# 1); we only want the code paths exercised.
set +e
if [ -n "$profile" ]; then
  :
elif [ "$product" = grund ]; then
  for _ in 1 2 3; do
    "$grund" check "$repo"                   >/dev/null 2>&1
    "$grund" list "$repo"                    >/dev/null 2>&1
    "$grund" show FS-check --brief "$repo"   >/dev/null 2>&1
    "$grund" show FS-check "$repo"           >/dev/null 2>&1
    "$grund" show FS-check --full "$repo"    >/dev/null 2>&1
    "$grund" refs GOAL-fast-feedback "$repo" >/dev/null 2>&1
    "$grund" cover "$repo"                   >/dev/null 2>&1
    "$grund" fmt --check "$repo"             >/dev/null 2>&1
  done
else
  "$python" scripts/distribution/train.py "$product" "$pgo_dir/instrumented/$artifact" \
    "$repo" "$pgo_dir/work" "$host" >/dev/null
fi
set -e

if [ -z "$profile" ]; then
  # Fail loudly if the training loop produced no profiles — every command above
  # ran under `set +e`, so a totally-broken instrumented payload would otherwise
  # be hidden until `llvm-profdata` errored on an empty input.
  shopt -s nullglob
  profraws=("$raw"/*.profraw)
  if [ ${#profraws[@]} -eq 0 ]; then
    echo "error: PGO training produced no .profraw files in $raw: training produced no profile" >&2
    echo "       (the instrumented $product did not run, or could not write its profile)" >&2
    exit 1
  fi

  # llvm-profdata ships in the build toolchain's llvm-tools-preview component.
  llvm_profdata="$(in_build bash -c 'find "$(rustc --print sysroot)" -type f -name "llvm-profdata*" | head -n1')"
  if [ -z "$llvm_profdata" ]; then
    echo "error: llvm-profdata not found — run: rustup component add llvm-tools-preview" >&2
    exit 1
  fi
  in_build "$llvm_profdata" merge -o "$profdata" "${profraws[@]}"
  printf '%s\n%s\n' "$training" "$sha" > "$pgo_dir/training"
  training_sha256="$(sha256 "$pgo_dir/training")"
  printf 'key: %s\ntraining: %s\ntraining_sha256: %s\ngenerate_sha256: %s\n' \
    "$key" "$training" "$training_sha256" "$generate_sha256" > "$profdata.key"
fi

echo "==> 3/3  rebuild optimized $product (-Cprofile-use)"
build "-Cprofile-use=$(native_path "$profdata") -Cllvm-args=-pgo-warn-missing-function"

if [ -n "$evidence" ]; then
  field() { sed -n "s/^$1: //p" "$profdata.key"; }
  mkdir -p "$(dirname "$evidence")"
  cat > "$evidence" <<EOF
{
  "product": "$product",
  "key": $key,
  "training": $(field training),
  "training_sha256": "$(field training_sha256)",
  "generate_sha256": "$(field generate_sha256)",
  "profile": "$(native_path "$profdata")",
  "profile_sha256": "$(sha256 "$profdata")",
  "payload": "$(native_path "$payload")",
  "build_sha256": "$(sha256 "$payload")"
}
EOF
fi

echo "==> done: $payload"
case "$product" in
  grund | grund-lsp) "$payload" --version ;;
esac

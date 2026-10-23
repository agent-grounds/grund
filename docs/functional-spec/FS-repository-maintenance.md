# FS-repository-maintenance: Checkout maintenance stays discoverable

The repository keeps its maintenance scripts under `scripts/` and carries the
bindings needed to invoke them in each checkout. This makes contributor
housekeeping easy to find and operate, serving
[§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).
These are repository maintenance hooks; they add no public `grund` command or
hook to `grund check`.

## terms: Terms

Leans on [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure)
(config root).

- **Checkout root**: the directory containing this checkout's `Cargo.toml` and
  `grund.toml`, independently of the maintenance script's directory.
- **Clean verb**: the executable ephor invokes to reclaim a checkout's build
  output when its own cleanup policy permits execution.
- **Cache-tagged directory**: a directory containing `CACHEDIR.TAG` whose first
  43 bytes are `Signature: 8a477f597d28d172789f06886806bc55`.
- **Node target root**: the external shared directory selected by
  `GRUND_NODE_TARGET`, defaulting to `~/ag/tmp/grund-node-target`, containing
  per-checkout Node acceptance build targets and shared tools.

## 1. Clean verb

### 1.1 Executable home

The tracked clean verb is a regular executable at `scripts/clean.sh`, with Git
mode `100755`. No `clean.sh` file or wrapper remains at the checkout root. The
script cites the cleanup responsibilities in [§FS-repository-maintenance.1.4](FS-repository-maintenance.md#14-cleanup-responsibilities).

### 1.2 Checkout-owned discovery

The checkout's root `ephor.json` declares `clean` as an object with
`"command": "./scripts/clean.sh"` and `"cwd": "root"`. This binding travels with
the branch, preserves any other manifest fields, and requires no machine-local
site override. In the absence of a site override, ephor's non-acting clean
preview must resolve this executable for the intended checkout, report
`would-clean` with gated output, and exit 0. Acceptance uses the supported
`ephor clean --workspace grund --json` preview without `--act`.

### 1.3 Checkout-root execution

The clean verb runs with the checkout root as its working directory, despite
living under `scripts/`. Both `cargo clean` and the cache-tag traversal operate
on that checkout, including cache-tagged build output outside `scripts/`.

### 1.4 Cleanup responsibilities

The verb first runs `cargo clean`; a failed cargo invocation stops cleanup and
returns a nonzero status. On success it removes directories beneath the checkout
that contain a valid cache tag, including scratch builds, and exits 0. It leaves
untagged directories and directories with invalid cache tags intact. Traversal
prunes `.git` entries, so Git metadata and cache-tagged content inside `.git`
remain intact. Relocation preserves these existing responsibilities.

#### 1.4.1 Disposable acceptance remains reliable under writable-descriptor pressure

Cleanup acceptance reads the actual copied `scripts/clean.sh` bytes in an
exclusively owned disposable root. A controlled cargo substitute records its
working directory and arguments and supplies the selected exit status; installed
cargo must never run. This proves checkout-root execution
[§FS-repository-maintenance.1.3](FS-repository-maintenance.md#13-checkout-root-execution)
without cleaning the working checkout.

On Linux, keeping writable descriptors open on the copied script and, separately,
any generated cargo executable must not prevent acceptance from running. Each
boundary is exercised with cargo status 0 and 7. Success records exactly
`cargo clean` at the fixture root, removes valid tagged directories, and preserves
invalid tags, untagged content, and Git metadata including tagged Git content.
Status 7 records the same invocation, returns 7, and leaves all cache payloads
and preserved content intact. A removed generated cargo executable needs no
writer, but still requires these invocation, status, and cleanup assertions.
Portable cleanup acceptance and the independent executable-mode and
checkout-manifest checks remain in place.

## 2. External Node acceptance target lifetime

### 2.1 Ownership and reuse

The Node acceptance harness must record each target's owning resolved checkout
root. Repeated calls, including separate processes and calls through a symlink
to the same checkout, must retain the same target and its build output. Two live
checkouts must retain separate targets. Revisiting an existing unowned target
for the current checkout must add ownership without discarding its artifacts.
The shared root, target paths and shared tools remain stable.

### 2.2 Automatic reclamation

Each call to the harness's existing `node_target()` must reclaim other targets
under the Node target root whose valid ownership positively identifies a
checkout that no longer exists, including nonempty build output. A later call
from a surviving checkout is sufficient; contributors need no separate prune
command. Live targets and their artifacts, including the caller's, and shared
tools must survive reclamation.

### 2.3 Conservative deletion boundary and legacy entries

Only a regular directory directly under the Node target root with a readable,
complete ownership record naming an absolute resolved checkout root whose
digest matches the target directory is eligible. A missing checkout must be
confirmed by a filesystem lookup; permission failures and other uncertainty
must preserve the target. Missing, malformed, incomplete or mismatched records,
symlink entries or ownership records, and unrelated directories must not
authorize deletion. Cleanup must never follow an entry outside the shared root.

Legacy entries without ownership remain untouched: their one-way names do not
identify their checkout. A returning live checkout can acquire ownership under
[§FS-repository-maintenance.2.1](FS-repository-maintenance.md#21-ownership-and-reuse) for later reclamation, but this policy cannot
automatically reclaim all old unrecorded entries. Identifying those entries
remains a manual operation.

### 2.4 Concurrent callers and source lifetime

Concurrent helper calls must preserve live targets and artifacts and publish
complete ownership records atomically, so another caller never consumes a
partially written record as permission to delete. Concurrent reclamation of
the same dead target must not fail an otherwise valid call. This is checkout
lifetime tracking, with no build lease: callers must keep a source checkout
present for the duration of builds using its target. Removing that source
while its build still runs permits a later helper call to reclaim the target.

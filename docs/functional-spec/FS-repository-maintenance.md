# FS-repository-maintenance: Checkout maintenance stays discoverable

The repository keeps its maintenance scripts under `scripts/`, carries the
bindings that invoke them in each checkout, ties its acceptance builds to the
checkout they exercise, and gives each temporary test root an owner, so
contributor housekeeping and verification stay reliable in every checkout,
serving [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).
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

The clean verb is the regular executable `scripts/clean.sh`, tracked with Git
mode `100755`; the checkout root holds no `clean.sh` or wrapper. The
checkout's root `ephor.json` declares `clean` as an object with
`"command": "./scripts/clean.sh"` and `"cwd": "root"`, so the binding travels
with the branch and needs no machine-local site override. Without one, ephor's
non-acting clean preview resolves this executable for the intended checkout,
reports `would-clean` with gated output, and exits `0`; acceptance uses the
supported `ephor clean --workspace grund --json` preview without `--act`.

The verb runs with the checkout root as its working directory although it
lives under `scripts/`, so `cargo clean` and the cache-tag traversal both run
on the checkout. It runs `cargo clean` first; if that fails, it removes no
cache-tagged directory and exits with cargo's status. Otherwise its traversal,
which prunes every entry named `.git` at any depth, removes each cache-tagged
directory it reaches and exits `0`: Git metadata, cache-tagged content inside
`.git`, untagged directories and directories with an invalid cache tag stay
intact. Its acceptance runs the script's actual bytes, copied into an
exclusively owned disposable root, against a controlled cargo substitute, so
testing it never cleans the working checkout or runs the installed cargo. On
Linux, a writable descriptor held open on the copied script or on a generated
cargo executable never stops that acceptance from running.

## 2. External Node acceptance target lifetime

The Node acceptance harness's `node_target()` gives each checkout, by its
resolved root, one target under the Node target root, records that root in
the target as its owner, and every call for that checkout returns that target
with its build output intact. Each call also reclaims every other target
whose recorded owner checkout is gone, so contributors need no prune command.
A target is reclaimed only when it is a regular directory directly under the
Node target root holding a readable, complete ownership record, itself no
symlink, that names an absolute resolved checkout root whose digest matches
the target directory, and a filesystem lookup confirms that checkout is
missing. A permission failure or any other uncertainty preserves the target,
cleanup never follows an entry outside the shared root, and the shared tools
are never reclaimed. An unowned target is never reclaimed, because its one-way
name does not identify its checkout: a call from its live checkout adopts it
with its build output, and one whose checkout is gone stays until someone
removes it by hand.

Concurrent calls publish ownership records atomically, so no caller takes a
partly written record as leave to delete, and concurrent reclamation of one
dead target fails no otherwise valid call. Removal waits out transient Windows
deletion contention for a bounded time, and only a success or a filesystem
lookup that confirms the path is gone completes it: a persistent permission
denial or any unrelated error fails the call. Ownership tracks the checkout,
not a build: removing a checkout while a build uses its target lets a later
call reclaim that target.

## 3. Checkout-owned terminal acceptance

With no build input supplied, the terminal watch acceptance exercises the CLI
and core of its own checkout, and two checkouts under one home never share its
default build output or executable. A caller may supply its own executable or
Cargo target and answers for that input. A build failure while preparing the
executable fails the acceptance rather than skipping it. The terminal behavior
itself is [§FS-check.6.2.2](FS-check.md#622-owned-terminal-screen).

## 4. Temporary test roots have an owner

The temporary-root helpers of the core, CLI and LSP tests own each root they
create from the creation of its leaf, through setup, to the caller's last use,
and remove it with its contents when that ends, on normal scope exit and while
a test unwinds from a panic alike, so a passing suite leaves at most an empty
shared base in its temporary area.

Each allocation exclusively claims an unoccupied leaf, so allocations with the
same name get independent roots. An occupied candidate is refused without
changing its contents, or a distinct candidate is chosen. Cleanup removes only
the claimed leaf, never a shared parent, a root an earlier run left, or a
refused candidate.

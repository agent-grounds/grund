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

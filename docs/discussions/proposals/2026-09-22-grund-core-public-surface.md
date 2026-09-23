# DISC-grund-core-public-surface: What grund-core's public root surface is, and what it should be

## 1. Status

Open, and an audit rather than a change. It is proposed under `agent-grounds/grund#250`, after [§DA-engine-renders-nothing](../../decisions/architectural/DA-engine-renders-nothing.md#da-engine-renders-nothing-the-engine-renders-nothing-so-the-deprecated-compat-frontend-retires) retired the deprecated process frontend and the parity seams that were kept for it — which is what makes the rest of the surface worth counting, because what is left is no longer waiting on a deletion already scheduled.

Nothing decided here changes a Rust symbol, a visibility, a `#[doc(hidden)]` attribute, a command, an output or an exit code. This discussion records what `grund_core` exports today, classifies each name against the much smaller embedding surface [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) documents, and proposes a target shape with the releases that would reach it. It proposes rather than cuts because every removal of a name an embedder could be calling owes the named notice-and-removal sequence of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) and [§GOAL-no-silent-breakage.2](../../goals.md#2-the-deprecation-path); accepting this discussion accepts the audit and the target, and a later change that actually moves the surface is separately specified, extends the specification points it would newly contradict, and ships under that sequence.

The evidence this discussion argues from is one row per public root name in `docs/discussions/proposals/2026-09-22-grund-core-public-surface-inventory.md`, and [§DISC-grund-core-public-surface.5](2026-09-22-grund-core-public-surface.md#5-what-is-checked) is what keeps those rows equal to the crate root rather than merely once-equal.

## 2. The audited baseline

A surface moves, so an audit of one is worth nothing without the commit it was taken on. This is that commit.

| | |
| --- | --- |
| Commit | `a6cb20e3ecf7cce7e3e99f900dbf1fec1edd64db` — "Format cover precedence test" |
| Workspace version | `0.14.2-dev` |
| Nearest release tag | `v0.14.1`, as `v0.14.1-7-ga6cb20e3ec` |
| Contains | `af4cd0cfe7` — "Remove the deprecated core renderer" |
| Public root names under [§DISC-grund-core-public-surface.3](2026-09-22-grund-core-public-surface.md#3-what-counts-as-a-public-root-name) | 181 |

`af4cd0cfe7` is a condition of the audit rather than a detail of it: it is the commit that removed `crates/grund-core/src/compat/` and `grund_core::main_entry()`, so a count taken before it is a count of a different crate and cannot be carried forward. Any earlier tally in the issue thread or in this repository's history is read that way — as evidence about the surface as it then was.

If the audit is re-taken on a later commit, this table and every inventory row move together; a table that names one commit while the rows were read on another is the one failure this baseline exists to prevent.

## 3. What counts as a public root name

`crates/grund-core/src/lib.rs` holds one `mod` line per component and the explicit `pub use` list that **is** the crate's public surface, which is why [§AR-core-module-layout.1.2](../../architecture/AR-core-module-layout.md#12-librs-is-the-one-file-outside-every-component) makes that list — and not what a component happens to leave `pub` — the boundary this audit counts. A component's own items reach another component through its `mod.rs` as `pub(crate)`, and reach an embedder only by appearing in that list ([§AR-system.4](../../architecture/README.md#4-dependency-direction), [§AR-bindings.2](../../architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives)).

The counting convention, stated once so that a row can be argued about without re-arguing what a row is:

- One row per distinct spelling reachable as `grund_core::<name>`.
- A grouped re-export is expanded: `pub use model::{A, B}` is two rows, not one.
- An `as` alias is counted by its exported name, and two aliases of one item are two rows, because two names is what an embedder can write and two names is what would have to be retired.
- A root `pub mod`, or a public item declared in `lib.rs` itself, is a row on the same footing as a re-export.
- An item a component leaves `pub` that this list does not re-export is not reachable at the root and is not a row.
- A `#[doc(hidden)]` name stays in the count. Hiding changes discoverability, not availability: the symbol still links, so it is still something a removal would take away.
- A glob re-export would make the set unenumerable and has no place in this list; [§AR-core-module-layout.1.2](../../architecture/AR-core-module-layout.md#12-librs-is-the-one-file-outside-every-component) already forbids one, and the check refuses it rather than guessing what it expanded to.

"Documented" throughout this discussion means Rustdoc-visible and nothing more. Whether the specification supports embedding a name is a separate column with a separate answer, and neither settles the other.

## 4. What every inventory row carries

Each row records, for one root name:

1. The name, and the component that owns the item behind it.
2. Rustdoc status: visible, or `#[doc(hidden)]`, read from the attribute at the definition.
3. Specification status: named or supported by [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) as part of the embedding surface, or not.
4. Repository consumers: `grund-cli`, `grund-lsp`, the test suites, or none found — each flag carrying the evidence location that justifies it.
5. Structural consumption: use a name-by-name import search cannot see, where a frontend reaches the type through a returned record or a field rather than by naming it.
6. A proposed disposition.

Two readings are forbidden of a finished row. `none found` is the result of a search of this repository and never a claim about embedders outside it; where a name has no known repository consumer, unknown external use and [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) decide what happens to it, not convenience. And a row's Rustdoc column, its specification column and its consumer columns answer three different questions: a name can be documented and unsupported, supported and unused here, or hidden and load-bearing, and the disposition is argued from all three.

## 5. What is checked

An inventory is worth exactly what its completeness is worth, and an audit that silently misses a name recommends a surface that does not exist. `tests/integration/test_public_surface_inventory.py` is what makes that failure loud: it parses the explicit `pub use` list and any root-declared public item out of `crates/grund-core/src/lib.rs`, parses the first column of the inventory's tables, and holds the two sets equal.

- A public root name with no row fails the check, which is how a partial or stale inventory is caught.
- A row naming something the crate root does not export fails it, which is how a row that outlived its symbol is caught.
- A name listed twice fails it, because a duplicated row is two dispositions for one name.
- Grouped re-exports, `as` aliases, single-name re-exports and hidden names are all parsed alike; a glob is an error rather than an expansion.

The check holds the count, not the judgement. Rustdoc visibility against the source attributes, the consumer flags against their evidence, the structural uses against the call sites they are inferred from, and the presence of a disposition on every row are the reviewer's, and [§DISC-grund-core-public-surface.4](2026-09-22-grund-core-public-surface.md#4-what-every-inventory-row-carries) is what they are held to. `cargo check --workspace` continues to hold the frontend edges the audit only describes.

## 6. What this discussion still has to settle

The sections above fix the baseline, the convention, the row and the check. What remains is the argument, and it is not written yet:

1. A target disposition for every row, and the target surface they add up to.
2. The coarse integrations entry point: one data-returning facade over the existing `writers` mechanisms in place of the fine-grained items the integrations command reads across the crate boundary — argued against keeping argv, rendering and exit policy in `grund-cli`, where [§FS-integrations.1.3](../../functional-spec/FS-integrations.md#13-which-side-owns-what) and [§AR-bindings.3](../../architecture/AR-bindings.md#3-grund-cli-the-cli-binary) put them, and against the engine's data-only boundary of [§AR-system.2.9](../../architecture/README.md#29-api) and [§AR-bindings.2](../../architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives).
3. Frontend-only seams left public but hidden from Rustdoc, each with the frontend that reads it named beside it — argued as what it is, a change of discoverability rather than of availability, and therefore neither privacy nor removal.
4. For every recommended removal, concrete releases counted from this baseline: release `N` ships the replacement beside the old name with a deprecation note naming the removal release, and release `N+1` or later removes it. The 0.15.0 deletion already taken is not permission for a fresh cut.

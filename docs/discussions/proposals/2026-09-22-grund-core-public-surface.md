# DISC-grund-core-public-surface: What grund-core's public root surface is, and what it should be

## 1. Status

Open, and an audit rather than a change. It is proposed under `agent-grounds/grund#250`, after [§DA-engine-renders-nothing](../../decisions/architectural/DA-engine-renders-nothing.md#da-engine-renders-nothing-the-engine-renders-nothing-so-the-deprecated-compat-frontend-retires) retired the deprecated process frontend and the parity seams that were kept for it — which is what makes the rest of the surface worth counting, because what is left is no longer waiting on a deletion already scheduled.

Nothing decided here changes a Rust symbol, a visibility, a `#[doc(hidden)]` attribute, a command, an output or an exit code. This discussion records what `grund_core` exports today, classifies each name against the much smaller embedding surface [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) documents, and proposes a target shape with the releases that would reach it. It proposes rather than cuts because every removal of a name an embedder could be calling owes the named notice-and-removal sequence of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) and [§GOAL-no-silent-breakage.2](../../goals.md#2-the-deprecation-path); accepting this discussion accepts the audit and the target, and a later change that actually moves the surface is separately specified, extends the specification points it would newly contradict, and ships under that sequence.

The evidence this discussion argues from is one row per public root name in `docs/discussions/proposals/2026-09-22-grund-core-public-surface-inventory.md`, and [§DISC-grund-core-public-surface.5](2026-09-22-grund-core-public-surface.md#5-what-is-checked) is what keeps those rows equal to the crate root rather than merely once-equal.

## 2. The audited baseline

A surface moves, so an audit of one is worth nothing without the commit it was taken on. This is that commit.

| | |
| --- | --- |
| Commit | `7283e23bb023e659a596c4bee43f318d43a109c7` — "Hold check's refusal set and fmt's protected set in one predicate" |
| Workspace version | `0.14.2-dev` |
| Nearest release tag | `v0.14.1`, as `v0.14.1-26-g7283e23bb0` |
| Contains | `af4cd0cfe7` — "Remove the deprecated core renderer" |
| Public root names under [§DISC-grund-core-public-surface.3](2026-09-22-grund-core-public-surface.md#3-what-counts-as-a-public-root-name) | 182 |

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

The sections above fix the baseline, the convention, the row and the check. What follows is the argument, and it is a position put to this discussion rather than a decision it records: [§DISC-grund-core-public-surface.1](2026-09-22-grund-core-public-surface.md#1-status) still holds, so accepting any of it authorizes a later change that is separately specified and separately shipped, and refusing any of it costs nothing already shipped.

### 6.1 The target disposition of every row

Six dispositions cover the 182 rows, and the inventory carries one on each.

| Disposition | Rows | What it does to the symbol |
| --- | --- | --- |
| keep | 106 | nothing |
| keep, hidden | 13 | nothing; an existing `#[doc(hidden)]` seam stays as it is |
| keep, name in the spec | 16 | nothing; [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) grows to reach it |
| hide | 12 | `#[doc(hidden)]`, with the frontend that reads it named beside it wherever the inventory found one |
| retire with the ramp | 1 | removed at 0.15.0 by a ramp already written into the tree, which this discussion records rather than proposes |
| facade, then retire | 34 | replaced by a coarse entry point, then removed |

They add up to a target of **147 public root names plus whatever the facade itself exports**, of which 122 are documented and 25 hidden — against 182 and 169 today. One group does the work: the 34 integrations items are a fifth of the whole surface, and every other proposal here moves documentation rather than symbols. The one further name that leaves the surface is the `retire with the ramp` row, and it leaves by a ramp [§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes) already scheduled ([§DISC-grund-core-public-surface.6.3](2026-09-22-grund-core-public-surface.md#63-frontend-only-seams-left-public-but-hidden)) — a fact this audit found rather than a cut it asks for.

Two of the six deserve saying out loud, because they are what the inventory found rather than what the ticket expected.

The **16 `keep, name in the spec`** rows are `init` and its records, `fetch_snapshot` and its failure, and `list_sizes` and its records: data-returning entry points that behave exactly like the embedding surface and that [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) does not reach, because it names `check`, `show`, `scan` "and related APIs" and these live outside the `api` component. That is a gap in the specification, not surface to cut. Hiding them would tell an embedder that a documented, data-returning function is not for them; the fix is the other direction.

The **38 rows with no repository consumer of either kind** are not a removal list, and the inventory says so on every one of them. Most of them are what `scan` returns: `Findings` and the model records under it are specification-supported ([§DISC-grund-core-public-surface.4](2026-09-22-grund-core-public-surface.md#4-what-every-inventory-row-carries)), and the frontends simply do not embed the engine — they call the seams instead. A surface built for embedders is expected to look unused from inside.

### 6.2 The coarse integrations entry point

Thirty-four names — `INTEGRATIONS_BLOCK_VERSION` and the 33 `writers` items beside it — are public for exactly one caller: the `grund integrations` command, which reads them across the crate boundary. None is reachable from any documented entry point, so [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) supports embedding none of them, and each is one client list, one detection, one managed write or one artifact payload.

For the facade: the command assembles its output from a fixed sequence of engine answers, and a surface that exposes every step of an assembly is a surface that pins the assembly. Two data-returning functions — one that reports what the integrations command would find and write, one that applies a requested write and returns its outcome — say the same thing in a shape an embedder could use, and shrink the surface by a fifth in one change.

Against it, and the reason it is argued rather than assumed: a facade over a command is how an engine acquires a rendering API by accident. [§FS-integrations.1.3](../../functional-spec/FS-integrations.md#13-which-side-owns-what) and [§AR-bindings.3](../../architecture/AR-bindings.md#3-cratesgrund-cli-the-cli-binary) put argv, rendering and exit policy in `grund-cli`, and [§AR-system.2.9](../../architecture/README.md#29-api) and [§AR-bindings.2](../../architecture/AR-bindings.md#2-grund-core-the-only-place-logic-lives) keep the engine returning data and writing to no stream. So the facade is proposed with the boundary as its acceptance test: every field it returns is a fact the CLI decides how to print, no field is a line of output, and the CLI still owns which of them reaches a terminal. An artifact the engine writes byte-for-byte — the resolver script, the VS Code payloads — is content, not rendering, and stays content on the far side of the facade.

If that test cannot be met, the right answer is to keep the 34 rather than to ship a facade that fails it.

### 6.3 Frontend-only seams left public but hidden

Twelve names serve one frontend and nothing else: `canonical_snapshot_path`, `citation_under_title`, `lsp_hover_with_kind_title`, `lsp_title_hover_body`, `on_type_line_edits`, `can_replace_trigger_at`, `DeclaredId`, `LineEdit` and `LspUsage` are `grund-lsp`'s editor mechanics; `names_member_id_candidate`, `AGENT_SETUP_INSTRUCTIONS` and `canonical_template_text` are `grund-cli`'s. The inventory names the reader of eleven of them. Of `can_replace_trigger_at` it names none — that row is `none found` in both consumer columns — and it is grouped here because it is an editor mechanism of the same module as `on_type_line_edits`, not because a frontend was found to read it, which is a reason to look at it rather than a reason to cut it.

Four of the twelve carry a claim against hiding them in their own source. `crates/grund-core/src/queries/editor_on_type.rs:20` is a module doc-comment over `can_replace_trigger_at`, `on_type_line_edits`, `DeclaredId` and `LineEdit`, and it says the module was "[s]plit out of the api's contract file", which [§AR-core-module-layout.2](../../architecture/AR-core-module-layout.md#2-refactor-boundary) "keeps as the published embedding surface: the public items here are part of it". Hiding the four is still right, and that sentence is something this audit disagrees with rather than something it missed: the point it cites makes `crates/grund-core/src/api/` the published surface, and the split it describes moved these four out of `api` into `queries`; [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate)'s closure reaches none of them, which is why all four read `Spec: no`. A file does not enrol itself in the embedding surface by asserting membership — [§DISC-grund-core-public-surface.4](2026-09-22-grund-core-public-surface.md#4-what-every-inventory-row-carries) keeps the three questions apart for exactly this reason — so the doc-comment's claim should be corrected in the change that lands the hiding, which is a `crates/` edit this audit records and does not make.

Hiding them is a change of discoverability and nothing else, and the distinction is the whole argument. `#[doc(hidden)]` removes a name from Rustdoc; the symbol still links, so an embedder already calling one keeps compiling and [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) is not engaged. It is neither privacy nor removal, and a proposal that treated it as either would be proposing something else.

What it buys: an embedder reading docs.rs today cannot tell `show` from `lsp_title_hover_body`, because both render identically, and the specification is the only place that says which is meant for them. What it costs: a dependency taken on a hidden name is harder to notice, both for the embedder and for us. The note naming the frontend is the mitigation, and it is why the disposition is "hide, with its reader named" rather than a bare "hide" — and why `can_replace_trigger_at`, the one name with no reader to name, is the one to settle before the hiding lands rather than the one to hide quietly.

**`REFS_QUERY_FAILURE_WARNING` is not the thirteenth.** It reads like one — a `grund-cli`-only constant the specification reaches nowhere, which is exactly what the twelve above are — but it cannot be hidden in the release that hides them, because that is the release its text must be gone in. The constant holds [§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes)'s ramp warning byte-for-byte at `crates/grund-core/src/api/refs.rs:102`, that point says the warning and the `error:` prefix retire together at 0.15.0, [§FS-distribution.4.2.4](../../functional-spec/FS-distribution.md#424-the-scalar-clause-is-the-refs-warnings) gives the landed phase a version-gated contract test that expects the warning's absence, and `scripts/check_release_ramps.py 0.15.0` refuses the cut while the line is in the tree, naming that very line. A constant whose whole value is a message the release retires has nothing left to hide, so its disposition is `retire with the ramp`: the removal is [§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes)'s, already scheduled, and this audit records it instead of proposing anything about the name.

**What that removal owes an embedder, and what it can no longer be given.** The 0.14.0 warning is notice to a *user* about an exit code; it tells an embedder nothing about a public Rust constant, so the ramp does not discharge [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) on the symbol's behalf. That path puts the notice in release `N` and the removal no earlier than `N+1`, which for a removal at 0.15.0 means the notice belonged in 0.14.0 — a release already cut, so the window has closed. The obligation is real and it stands. [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)'s covered list enumerates the user-visible surface and stops before the Rust exports, but that is where the letter stops rather than where the discipline does: [§FS-distribution.3.1.1](../../functional-spec/FS-distribution.md#311-main_entry-is-absent-from-the-embedding-api) says `main_entry()` — a root export of this same crate, and no part of that list — was kept "until 0.14.0 shipped a deprecation note naming its removal in 0.15.0, the sequence [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) requires". The shipped specification has already extended the path to this surface, which is why [§DISC-grund-core-public-surface.1](2026-09-22-grund-core-public-surface.md#1-status) gives that obligation as the reason this audit proposes rather than cuts, and why this section does not read the covered list against it. So the position is not that nothing further is owed but that what is owed can no longer be paid: `REFS_QUERY_FAILURE_WARNING` leaves at 0.15.0 without the notice release [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) requires, and an embedder who took the dependency loses the symbol in the release that should have warned them. This audit records that conflict and does not resolve it, because it has no authority to: the removal is [§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes)'s, fixed before this audit and encoded twice over — `scripts/check_release_ramps.py 0.15.0` refuses the release while the line stands, and [§FS-distribution.4.2.4](../../functional-spec/FS-distribution.md#424-the-scalar-clause-is-the-refs-warnings) gives the landed phase a version-gated contract test — and the only way to reopen the window is to move a ramp the specification fixes, which is a change to a specified schedule and belongs to whoever specifies it rather than to an inventory. What this discussion can do is make the release as loud as one release still can be: a changelog entry under **Removed** at 0.15.0 naming the constant and the ramp that decided it, so an embedder who took the dependency reads why rather than only that. That mitigates the break; it does not discharge the path. The contrast with the 34 is the point — nothing schedules those, so they get the full sequence in [§DISC-grund-core-public-surface.6.4](2026-09-22-grund-core-public-surface.md#64-the-releases), and the one name that cannot take it is the one whose schedule was fixed somewhere else.

**The same ramp's other half stays, and the reader should not have to find the pair.** `refs_query_failure_is_exit_one` at `crates/grund-core/src/api/refs.rs:106` is the predicate the constant is printed behind: it reads `CARGO_PKG_VERSION` and answers whether [§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes)'s mapping has reached its exit-`1` phase, so from 0.15.0 it answers `true` for every version, and the branch it guards at `crates/grund-cli/src/cli_refs.rs:113` — the branch that prints the warning — goes with the constant. What the ramp leaves behind is a function with one possible answer. Its row still reads `keep`, and that is right on the evidence: unlike the constant it is a Rustdoc-visible function of the `api` component, so it is `related API` and inside what [§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate) supports an embedder calling, and [§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes) retires the message without retiring the predicate. Collapsing a constant-valued predicate is worth doing later, but it would be a removal from the documented surface with no ramp behind it, so it owes [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s notice release in its own right and belongs to a change that specifies one. This discussion proposes keeping it and naming the pair.

### 6.4 The releases

The baseline's nearest tag is `v0.14.1` and the workspace is `0.14.2-dev`, so the next minor is 0.15.0 and that is where the notice goes.

- **0.15.0** ships the facade of [§DISC-grund-core-public-surface.6.2](2026-09-22-grund-core-public-surface.md#62-the-coarse-integrations-entry-point) beside all 34 names, each carrying a deprecation note naming 0.16.0 as the release that removes it. In the same release, the 12 hidings of [§DISC-grund-core-public-surface.6.3](2026-09-22-grund-core-public-surface.md#63-frontend-only-seams-left-public-but-hidden) land, with a changelog entry under **Changed** saying which names left the documentation and, for the eleven that have one, which frontend reads each, and the 16 specification additions of [§DISC-grund-core-public-surface.6.1](2026-09-22-grund-core-public-surface.md#61-the-target-disposition-of-every-row) land, which change no code.
- **0.16.0**, or later, removes the 34. Nothing else in this proposal removes anything, so this is the only release the proposal itself puts a removal in.

That is [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s path taken literally: release `N` ships the new form beside the old with a warning naming the release the old form stops working in, and the old form dies no earlier than `N+1`. The deletion already taken in this cycle is not permission to skip it — a caller who moved off `main_entry()` last release is exactly the caller who has not yet noticed the 34, and shortening the window for them is what [§GOAL-no-silent-breakage.2](../../goals.md#2-the-deprecation-path) exists to stop.

0.15.0 nevertheless removes one name, and the distinction is worth keeping straight rather than smoothing over: `REFS_QUERY_FAILURE_WARNING` goes in it because [§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes)'s ramp retires the message it holds, a schedule fixed before this audit and argued through in [§DISC-grund-core-public-surface.6.3](2026-09-22-grund-core-public-surface.md#63-frontend-only-seams-left-public-but-hidden). That release therefore carries a **Removed** entry for the constant and the ramp that decided it, beside the **Changed** entry for the hidings. Its notice window closed in 0.14.0, which is why that one symbol cannot take the path above and the 34 can.

# AR-core-module-layout: core implementation is split by category

The core implementation lives in `crates/grund-core/src/`, while `crates/grund-cli/src/main.rs` is the published `grund` CLI entrypoint described by [§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms). Inside `grund-core`, the source layout should match the same category boundaries the shipped LSP frontend and the later binding frontends need. A single large crate root hides ownership and makes spec-to-code citations harder to place.

## placement: Where the file layout sits

```text
components (AR-system.2) ─► [ file layout rule ] ─► one owner per file ─► reader, fissile
```

Not a component: the rule for how the engine's files are named, owned and sized, whichever component they implement. It takes the component list of [§AR-system.2](README.md#2-components) and gives each file exactly one owner among the categories below, which is what a reader and `fissile` need to find and bound a file ([§AR-system.5](README.md#5-what-holds-the-shape)). It says nothing about what a component may know; that is [§AR-system.4](README.md#4-dependency-direction).

## 1. Module categories

`crates/grund-core/src/lib.rs` stays the engine crate entrypoint and public Rust
API surface, and every implementation file under it belongs to one **module
directory**: one per component of [§AR-system.2](README.md#2-components), named
after it — `model/`, `grammar/`, `config/`, `workspace/`, `templates/`,
`scanner/`, `resolver/`, `rules/`, `checker/`, `queries/`, `writers/`, `api/`.
What each one implements, consumes and must not know is its component's
subsection, and what it may *read* is [§AR-system.4](README.md#4-dependency-direction);
this section says only how its files are arranged. There is no `compat/`
directory beside the api: the engine has no process frontend
([§AR-system.2.9.1](README.md#291-no-process-frontend-lives-in-the-engine)).

A file belongs to exactly one of those twelve module directories — for example,
`model/records.rs` belongs to **model** and `api/show.rs` belongs to **api** — and
it holds one thing, named for what that is: the invariant, rule or record a
reader would look for under that name. That is what a citation of this section
from a file's own doc comment means, and it is why a file grown past two subjects
is a split rather than an exception ([§AR-core-module-layout.3](AR-core-module-layout.md#3-file-size)).
The boundary, the one file outside it, where a test module sits, and the two tests that hold this against the tree are [§AR-core-module-layout.1.1](AR-core-module-layout.md#11-modrs-is-the-components-whole-boundary) to [§AR-core-module-layout.1.4](AR-core-module-layout.md#14-two-tests-hold-the-layout-against-the-tree).

### 1.1 `mod.rs` is the component's whole boundary

It declares the component's files, carries the doc comment citing the component's subsection, and re-exports with `pub(crate)` exactly the items another component reads; everything else is `pub(super)` or private to its file. Nothing outside the directory can name what `mod.rs` does not list, so the compiler holds the ownership a file-name prefix and a test used to hold.

### 1.2 `lib.rs` is the one file outside every component

As the crate entrypoint it holds one `mod` line per component, the
explicit `pub use <component>::{…}` list that **is** the crate's public surface
([§AR-core-module-layout.2](AR-core-module-layout.md#2-refactor-boundary)), and `#[cfg(test)] pub(crate) mod testing;` for the fixtures every test
module shares. It holds no implementation and re-exports nothing by glob, so a
name is public because this list says so rather than because a module happened
to leave it `pub`. Every test module sits in its component ([§AR-core-module-layout.1.3](AR-core-module-layout.md#13-a-test-module-sits-beside-the-code-it-pins)), so `lib.rs`
carries neither a `#[cfg(test)]` prelude of private globs nor an `include!` line:
there is no flat module left for either to serve.

### 1.3 A test module sits beside the code it pins

All 67 are `<component>/tests_<subject>.rs`, declared in that component's `mod.rs` as `#[cfg(test)] mod tests_<subject>;`, so `super` is the component. Which component that is comes from the entry point the cases exercise rather than from what they are named — a test of the walk is the scanner's, a test of a whole `check` run through the embedding surface is the api's. The module reads its own component through `use super::*`, anything from another component by `use crate::<other>::{…}`, and the shared fixtures as `use crate::testing::{…}`; no other glob. A private item a test needs from its own component it reaches through `super`, and one another component's tests read is a `#[cfg(test)] pub(crate) use` in that component's `mod.rs` — the same boundary as any other cross-component read, gated so it exists only in a test build. A test module may read any component, and the order of [§AR-system.4](README.md#4-dependency-direction) says nothing about one.

### 1.4 Two tests hold the layout against the tree

`tests/integration/test_module_layout.py` holds the layout: every `.rs` under
`crates/grund-core/src/` is `lib.rs`, `testing.rs`, or a file inside one of the
twelve component directories, each present directory has a `mod.rs`, and `lib.rs`
`include!`s nothing at all. Its pending-component set is empty; a named
component directory or its `mod.rs` going missing is an error.
`tests/integration/test_dependency_direction.py` holds the order of
[§AR-system.4](README.md#4-dependency-direction) across those directories,
skipping every `tests_*.rs` and `testing.rs` ([§AR-core-module-layout.1.3](AR-core-module-layout.md#13-a-test-module-sits-beside-the-code-it-pins)): every
`crate::<other>` reference runs downward, except the reads listed in it one by
one, each of which must still exist and still carry its
[§AR-system.4](README.md#4-dependency-direction) note at the import — so that
list can only shrink and a new upward read fails. The stronger internal rules
for parser, fact adapter, and evaluator imports are
[§AR-rules.6](AR-rules.md#6-boundary-tests).

### 1.5 Split ownership

When the named owners below are split, the destination is an ordinary sibling
module owned by the component directory in its path. Paths in this table are
relative to `crates/grund-core/src/`.

| Existing owner | Split destination | Subject owned by the destination |
| --- | --- | --- |
| `writers/fmt_rewrite.rs` | `writers/fmt_tree.rs` | `FmtRunOpts`, `FmtTreeOutcome`, `fmt_tree`, tree and scope orchestration, and automatic cross-reference enabling ([§FS-fmt.6.6](../functional-spec/FS-fmt.md#66-why-generated-configs-enable-cross-references)); per-file and per-line rewriting stays in `fmt_rewrite.rs`. |
| `checker/references.rs` | `checker/reference_scope.rs` | `ScanScope`, configured-scope narrowing, out-of-scope passes, and diagnostic tagging; citation resolution, alias hints, and their diagnostics stay in `references.rs`. |
| `api/tests_embedding.rs` | `api/tests_init_guidance.rs` | The three init-next and scaffold cases; the remaining embedding, list, check, and config cases stay in `tests_embedding.rs`. |
| `writers/tests_integrations.rs` | None | The already-compliant owner and its 18 cases stay intact and remain part of acceptance. |
| `writers/tests_init_agents.rs` | `writers/tests_agent_entrypoints.rs` | Companion-entrypoint discovery, selection, and validation cases; template and managed-block cases stay in `tests_init_agents.rs`. |
| `checker/tests_grounding_style.rs` | `config/tests_validation.rs` | Configuration-validation cases; grounding-floor and inline-style checker cases stay in `tests_grounding_style.rs`. |
| `checker/tests_grounding_style.rs` | `scanner/tests_qualified_citations.rs` | Qualified-citation recognition cases. |
| `checker/tests_grounding_style.rs` | `workspace/tests_scope.rs` | Root-scope classification cases. |

Each component's `mod.rs` declares these siblings and remains the component
boundary. Items shared only by siblings use private or `pub(super)` visibility;
only an item another component reads may be re-exported `pub(crate)` through
`mod.rs`. Moving an item does not widen the public Rust surface or change the
behavior protected by [§AR-core-module-layout.2](AR-core-module-layout.md#2-refactor-boundary).

Acceptance covers all six existing owners and all seven destinations, not just
the files that happen to exist at measurement time. After formatting, every
path must have its configured line measurement and remain at or below its
`fissile` soft limit: `core-source` for the production modules and `tests` for
the test modules ([§AR-ci.9](AR-ci.md#9-file-size-budget-gate)). An absent owner,
destination, or measurement is a failure; an exception is not an alternative
for this split. The 66 surviving test functions remain in their mapped owners
and destinations after the scheduled `main_entry()` availability case retires
with that symbol ([§FS-distribution.3.1.1](../functional-spec/FS-distribution.md#311-main_entry-is-absent-from-the-embedding-api)); additions are allowed, and the Rust
workspace suite must still execute them with their assertions unchanged.

## 2. Refactor boundary

Splitting the core and CLI crates is an architectural refactor only: it must not change CLI output, diagnostics, scan behavior, template bytes, or public entrypoints. The CLI package calls no compatibility command adapter any more — `integrations` was the last and it renders in `grund-cli` ([§AR-bindings.3](AR-bindings.md#3-cratesgrund-cli-the-cli-binary)) — and embedders use the public API in `crates/grund-core/src/api/`, whose contract files carry the published signatures and whose adapter files carry the conversions behind them, reaching it through the explicit `pub use` list in `lib.rs` ([§AR-core-module-layout.1.2](AR-core-module-layout.md#12-librs-is-the-one-file-outside-every-component)) — the one place a name becomes public.

## 3. File size

Each implementation file under `src/` stays below 500 lines of code. If a category grows past that limit, split it into smaller category subfiles, or into a category directory with submodules, rather than letting a new monolith form.

## 4. Citation placement

Code moved into a category file keeps the same behavior citations it carried before. When a whole category implements an architectural behavior, the file or module-level comment may cite this spec; narrower functional clauses remain cited on the specific function or branch that implements them.

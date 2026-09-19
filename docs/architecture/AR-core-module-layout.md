# AR-core-module-layout: core implementation is split by category

The core implementation lives in `crates/grund-core/src/`, while `crates/grund-cli/src/main.rs` is the published `grund` CLI entrypoint described by [§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms). Inside `grund-core`, the source layout should match the same category boundaries the shipped LSP frontend and the later binding frontends need. A single large crate root hides ownership and makes spec-to-code citations harder to place.

## placement: Where the file layout sits

```text
components (AR-system.2) ─► [ file layout rule ] ─► one owner per file ─► reader, fissile
```

Not a component: the rule for how the engine's files are named, owned and sized, whichever component they implement. It takes the component list of [§AR-system.2](README.md#2-components) and gives each file exactly one owner among the categories below, which is what a reader and `fissile` need to find and bound a file ([§AR-system.5](README.md#5-what-holds-the-shape)). It says nothing about what a component may know; that is [§AR-system.4](README.md#4-dependency-direction).

## 1. Module categories

`crates/grund-core/src/lib.rs` stays the engine crate entrypoint and public Rust API surface, and every implementation file under it belongs to one **module directory**: one per component of [§AR-system.2](README.md#2-components), named after it — `model/`, `grammar/`, `config/`, `workspace/`, `templates/`, `scanner/`, `resolver/`, `checker/`, `queries/`, `writers/`, `api/` — and `compat/`, a module directory but no component, for the deprecated path beside the api ([§AR-system.2.9](README.md#29-api)). What each one implements, consumes and must not know is its component's subsection, and what it may *read* is [§AR-system.4](README.md#4-dependency-direction); this section says only how its files are arranged.

A file belongs to exactly one of those twelve module directories — the one it sits under, `model/records.rs` to **model** and `compat/list.rs` to **compat** — and it holds one thing, named for what that is: the invariant, rule or record a reader would look for under that name. That is what a citation of this section from a file's own doc comment means, and it is why a file grown past two subjects is a split rather than an exception (§3).
The boundary, the one file outside it, where a test module sits, and the two tests that hold this against the tree are §1.1 to §1.4.

### 1.1 `mod.rs` is the component's whole boundary

It declares the component's files, carries the doc comment citing the component's subsection, and re-exports with `pub(crate)` exactly the items another component reads; everything else is `pub(super)` or private to its file. Nothing outside the directory can name what `mod.rs` does not list, so the compiler holds the ownership a file-name prefix and a test used to hold.

### 1.2 `lib.rs` is the one file outside every component

As the crate entrypoint it holds the twelve `mod` lines, the explicit `pub use <component>::{…}` list that **is** the crate's public surface (§2), and `#[cfg(test)] pub(crate) mod testing;` for the fixtures every test module shares. It holds no implementation and re-exports nothing by glob, so a name is public because this list says so rather than because a module happened to leave it `pub`. Every test module sits in its component (§1.3), so `lib.rs` carries neither a `#[cfg(test)]` prelude of private globs nor an `include!` line: there is no flat module left for either to serve.

### 1.3 A test module sits beside the code it pins

All 67 are `<component>/tests_<subject>.rs`, declared in that component's `mod.rs` as `#[cfg(test)] mod tests_<subject>;`, so `super` is the component. Which component that is comes from the entry point the cases exercise rather than from what they are named — a test of the walk is the scanner's, a test of a whole `check` run through the embedding surface is the api's. The module reads its own component through `use super::*`, anything from another component by `use crate::<other>::{…}`, and the shared fixtures as `use crate::testing::{…}`; no other glob. A private item a test needs from its own component it reaches through `super`, and one another component's tests read is a `#[cfg(test)] pub(crate) use` in that component's `mod.rs` — the same boundary as any other cross-component read, gated so it exists only in a test build. A test module may read any component, and the order of [§AR-system.4](README.md#4-dependency-direction) says nothing about one.

### 1.4 Two tests hold the layout against the tree

`tests/integration/test_module_layout.py` holds the layout: every `.rs` under `crates/grund-core/src/` is `lib.rs`, `testing.rs`, or a file inside one of the twelve directories, each of which exists and has a `mod.rs`, and `lib.rs` `include!`s nothing at all. `tests/integration/test_dependency_direction.py` holds the order of [§AR-system.4](README.md#4-dependency-direction) across those directories, skipping every `tests_*.rs` and `testing.rs` (§1.3): every `crate::<other>` reference runs downward, except the reads listed in it one by one, each of which must still exist and still carry its [§AR-system.4](README.md#4-dependency-direction) note at the import — so that list can only shrink and a new upward read fails.

## 2. Refactor boundary

Splitting the core and CLI crates is an architectural refactor only: it must not change CLI output, diagnostics, scan behavior, template bytes, or public entrypoints. The CLI package calls no compatibility command adapter any more — `integrations` was the last and it renders in `grund-cli` ([§AR-bindings.3](AR-bindings.md#3-grund-cli-the-cli-binary)) — and embedders use the public API in `crates/grund-core/src/api/`, whose contract files carry the published signatures and whose adapter files carry the conversions behind them, reaching it through the explicit `pub use` list in `lib.rs` (§1.2) — the one place a name becomes public.

## 3. File size

Each implementation file under `src/` stays below 500 lines of code. If a category grows past that limit, split it into smaller category subfiles, or into a category directory with submodules, rather than letting a new monolith form.

## 4. Citation placement

Code moved into a category file keeps the same behavior citations it carried before. When a whole category implements an architectural behavior, the file or module-level comment may cite this spec; narrower functional clauses remain cited on the specific function or branch that implements them.

# AR-core-module-layout: core implementation is split by category

The core implementation lives in `crates/grund-core/src/`, while `crates/grund-cli/src/main.rs` is the published `grund` CLI entrypoint described by [§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms). Inside `grund-core`, the source layout should match the same category boundaries the later LSP and binding frontends need. A single large crate root hides ownership and makes spec-to-code citations harder to place.

## placement: Where the file layout sits

```text
components (AR-system.2) ─► [ file layout rule ] ─► one owner per file ─► reader, fissile
```

Not a component: the rule for how the engine's files are named, owned and sized, whichever component they implement. It takes the component list of [§AR-system.2](README.md#2-components) and gives each file exactly one owner among the categories below, which is what a reader and `fissile` need to find and bound a file ([§AR-system.5](README.md#5-what-holds-the-shape)). It says nothing about what a component may know; that is [§AR-system.4](README.md#4-dependency-direction).

## 1. Module categories

`crates/grund-core/src/lib.rs` stays the engine crate entrypoint and public Rust API surface, and every implementation file under it belongs to one **component module**: one directory per component of [§AR-system.2](README.md#2-components), named after it — `model/`, `grammar/`, `config/`, `workspace/`, `scanner/`, `checker/`, `queries/`, `writers/`, `api/`, and `compat/` for the deprecated path beside the api ([§AR-system.2.9](README.md#29-api)). What each one implements, consumes and must not know is its component's subsection; this section says only how its files are arranged, and what a component may *read* is [§AR-system.4](README.md#4-dependency-direction).

A file belongs to exactly one component — the directory it sits under, `model/records.rs` to **model** and `compat/list.rs` to **compat** — and it holds one thing, named for what that is: the invariant, rule or record a reader would look for under that name. That is what a citation of this section from a file's own doc comment means, and it is why a file grown past two subjects is a split rather than an exception (§3).

`mod.rs` is the component's whole boundary. It declares the component's files, carries the doc comment citing the component's subsection, and re-exports with `pub(crate)` exactly the items another component reads; everything else is `pub(super)` or private to its file. Nothing outside the directory can name what `mod.rs` does not list, so the compiler holds the ownership a file-name prefix and a test used to hold.

`lib.rs` is the one file outside every component, as the crate entrypoint: the ten `mod` lines, the explicit `pub use <component>::{…}` list that **is** the crate's public surface (§2), the `#[cfg(test)]` prelude through which the crate's own test modules still read it flat, and the `include!` lines that splice those test modules in. It holds no implementation and re-exports nothing by glob, so a name is public because this list says so rather than because a module happened to leave it `pub`.

Two tests hold this against the tree. `tests/integration/test_module_layout.py` holds the layout: every `.rs` under `crates/grund-core/src/` is `lib.rs`, a `tests_*` module, or a file inside one of the ten directories, each of which exists and has a `mod.rs`, and `lib.rs` `include!`s nothing but its test modules. `tests/integration/test_dependency_direction.py` holds the order of [§AR-system.4](README.md#4-dependency-direction) across those directories: every `crate::<other>` reference runs downward, except the reads listed in it one by one, each of which must still exist and still carry its [§AR-system.4](README.md#4-dependency-direction) note at the import — so that list can only shrink and a new upward read fails.

## 2. Refactor boundary

Splitting the core and CLI crates is an architectural refactor only: it must not change CLI output, diagnostics, scan behavior, template bytes, or public entrypoints. The CLI package may keep calling compatibility command adapters while narrower data-returning APIs are introduced, but embedders use the public API in `crates/grund-core/src/api/`, whose contract files carry the published signatures and whose adapter files carry the conversions behind them, reaching it through the explicit `pub use` list in `lib.rs` (§1) — the one place a name becomes public.

## 3. File size

Each implementation file under `src/` stays below 500 lines of code. If a category grows past that limit, split it into smaller category subfiles, or into a category directory with submodules, rather than letting a new monolith form.

## 4. Citation placement

Code moved into a category file keeps the same behavior citations it carried before. When a whole category implements an architectural behavior, the file or module-level comment may cite this spec; narrower functional clauses remain cited on the specific function or branch that implements them.

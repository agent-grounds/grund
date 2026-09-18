# AR-core-module-layout: core implementation is split by category

The core implementation lives in `crates/grund-core/src/`, while `crates/grund-cli/src/main.rs` is the published `grund` CLI entrypoint described by [§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms). Inside `grund-core`, the source layout should match the same category boundaries the later LSP and binding frontends need. A single large crate root hides ownership and makes spec-to-code citations harder to place.

## placement: Where the file layout sits

```text
components (AR-system.2) ─► [ file layout rule ] ─► one owner per file ─► reader, fissile
```

Not a component: the rule for how the engine's files are named, owned and sized, whichever component they implement. It takes the component list of [§AR-system.2](README.md#2-components) and gives each file exactly one owner among the categories below, which is what a reader and `fissile` need to find and bound a file ([§AR-system.5](README.md#5-what-holds-the-shape)). It says nothing about what a component may know; that is [§AR-system.4](README.md#4-dependency-direction).

## 1. Module categories

`crates/grund-core/src/lib.rs` stays the engine crate entrypoint and public Rust API surface (`check`, `show`, `scan`, and the shared data types), while implementation code lives in smaller category files under `crates/grund-core/src/`.

What each category implements, consumes and must not know is its component's subsection in [§AR-system.2](README.md#2-components), and the table's third column says which. A category is named by the module directory that holds it; every one has a directory now, so the compiler holds the boundary that a file-name prefix and a test used to.

A file belongs to the category it sits under: `model/records.rs` to **model**, `compat/list.rs` to **compat**. A row may still name a file-name prefix beside its directory, which is how a file a move deliberately left flat would be recorded; none does today, because the last two — the deprecated renderers of [§AR-system.2.9](README.md#29-api) and the embedding surface beside them — became `compat/` and `api/`. `lib.rs` is the one file outside every category, as the crate entrypoint: module declarations, the public re-exports, and the crate's own test modules. What each category owns:

| Category | Module directory, or file-name prefixes | Component |
|---|---|---|
| **model** | `model/` | [§AR-system.2.2](README.md#22-model) |
| **config** | `config/` | [§AR-system.2.3](README.md#23-config) |
| **scanner** | `scanner/` | [§AR-system.2.5](README.md#25-scanner) |
| **checker** | `checker/` | [§AR-system.2.6](README.md#26-checker) |
| **queries** | `queries/` | [§AR-system.2.7](README.md#27-queries) |
| **writers** | `writers/` | [§AR-system.2.8](README.md#28-writers) |
| **api** | `api/` | [§AR-system.2.9](README.md#29-api) |
| **grammar** | `grammar/` | [§AR-system.2.1](README.md#21-grammar) |
| **workspace** | `workspace/` | [§AR-system.2.4](README.md#24-workspace) |
| **compat** | `compat/` | [§AR-system.2.9](README.md#29-api) |

`tests/integration/test_module_categories.py` holds this table against the tree: every top-level implementation file owned by exactly one row, every prefix owning a file, and every named module directory present with none of its former prefixes left at the top level but the ones its row still lists.

## 2. Refactor boundary

Splitting the core and CLI crates is an architectural refactor only: it must not change CLI output, diagnostics, scan behavior, template bytes, or public entrypoints. The CLI package may keep calling compatibility command adapters while narrower data-returning APIs are introduced, but embedders use the public API in `crates/grund-core/src/api/`, whose contract files carry the published signatures and whose adapter files carry the conversions behind them.

## 3. File size

Each implementation file under `src/` stays below 500 lines of code. If a category grows past that limit, split it into smaller category subfiles, or into a category directory with submodules, rather than letting a new monolith form.

## 4. Citation placement

Code moved into a category file keeps the same behavior citations it carried before. When a whole category implements an architectural behavior, the file or module-level comment may cite this spec; narrower functional clauses remain cited on the specific function or branch that implements them.

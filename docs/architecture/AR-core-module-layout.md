# AR-core-module-layout: core implementation is split by category

The core implementation lives in `crates/grund-core/src/`, while `crates/grund-cli/src/main.rs` is the published `grund` CLI entrypoint described by [§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms). Inside `grund-core`, the source layout should match the same category boundaries the later LSP and binding frontends need. A single large crate root hides ownership and makes spec-to-code citations harder to place.

## placement: Where the file layout sits

```text
components (AR-system.2) ─► [ file layout rule ] ─► one owner per file ─► reader, fissile
```

Not a component: the rule for how the engine's files are named, owned and sized, whichever component they implement. It takes the component list of [§AR-system.2](README.md#2-components) and gives each file exactly one owner among the categories below, which is what a reader and `fissile` need to find and bound a file ([§AR-system.5](README.md#5-what-holds-the-shape)). It says nothing about what a component may know; that is [§AR-system.4](README.md#4-dependency-direction).

## 1. Module categories

`crates/grund-core/src/lib.rs` stays the engine crate entrypoint and public Rust API surface (`check`, `show`, `scan`, and the shared data types), while implementation code lives in smaller category files under `crates/grund-core/src/`.

What each category implements, consumes and must not know is its component's subsection in [§AR-system.2](README.md#2-components), and the table's third column says which. A category is named by the module directory that holds it, and until it has one, by the file-name prefixes it owns.

A file belongs to the category it sits under where that category is a module directory — `model/records.rs` to **model** — and otherwise to the category whose prefix its name carries: `scanner_walk.rs` to **scanner**, `init_block.rs` to **init**. A category that has become a directory leaves no file of its former prefixes at the top level, except a prefix the row still lists beside the directory: that is how a file the move deliberately left flat is recorded, and every one today is a deprecated renderer waiting for `compat/` ([§AR-system.2.9](README.md#29-api)) — a whole command adapter, or the stream-writing half of a file whose data half moved. `lib.rs` is the one file outside every category, as the crate entrypoint. What each category owns:

| Category | Module directory, or file-name prefixes | Component |
|---|---|---|
| **model** | `model/` | [§AR-system.2.2](README.md#22-model) |
| **config** | `config/`, `config_cmd` | [§AR-system.2.3](README.md#23-config) |
| **scanner** | `scanner`, `value_json` | [§AR-system.2.5](README.md#25-scanner) |
| **checker** | `checker` | [§AR-system.2.6](README.md#26-checker) |
| **output** | `output` | [§AR-system.2.9](README.md#29-api) |
| **show** | `show` | [§AR-system.2.7](README.md#27-queries) |
| **refs** | `refs` | [§AR-system.2.7](README.md#27-queries) |
| **cover** | `cover` | [§AR-system.2.7](README.md#27-queries) |
| **list** | `list` | [§AR-system.2.7](README.md#27-queries) |
| **fmt** | `fmt` | [§AR-system.2.8](README.md#28-writers) |
| **id** | `id` | [§AR-system.2.8](README.md#28-writers) |
| **init** | `init` | [§AR-system.2.8](README.md#28-writers) |
| **completions** | `completions` | [§AR-system.2.7](README.md#27-queries) |
| **api** | `api` | [§AR-system.2.9](README.md#29-api) |
| **grammar** | `grammar/` | [§AR-system.2.1](README.md#21-grammar) |
| **workspace** | `workspace/`, `workspace_members_cmd` | [§AR-system.2.4](README.md#24-workspace) |
| **integrations** | `integrations`, `fetch`, `fetch_write` | [§AR-system.2.8](README.md#28-writers) |
| **lsp** | `lsp`, `on_type` | [§AR-system.2.7](README.md#27-queries) |
| **compat** | `compat` | [§AR-system.2.9](README.md#29-api) |

`tests/integration/test_module_categories.py` holds this table against the tree: every top-level implementation file owned by exactly one row, every prefix owning a file, and every named module directory present with none of its former prefixes left at the top level but the ones its row still lists.

## 2. Refactor boundary

Splitting the core and CLI crates is an architectural refactor only: it must not change CLI output, diagnostics, scan behavior, template bytes, or public entrypoints. The CLI package may keep calling compatibility command adapters while narrower data-returning APIs are introduced, but embedders use the public API in `api.rs`.

## 3. File size

Each implementation file under `src/` stays below 500 lines of code. If a category grows past that limit, split it into smaller category subfiles, or into a category directory with submodules, rather than letting a new monolith form.

## 4. Citation placement

Code moved into a category file keeps the same behavior citations it carried before. When a whole category implements an architectural behavior, the file or module-level comment may cite this spec; narrower functional clauses remain cited on the specific function or branch that implements them.

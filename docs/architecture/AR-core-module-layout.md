# AR-core-module-layout: core implementation is split by category

The core implementation lives in `crates/grund-core/src/`, while `crates/grund-cli/src/main.rs` is the published `grund` CLI entrypoint described by [§AR-bindings](AR-bindings.md#ar-bindings-target-shape-for-exposing-the-rust-engine-on-three-platforms). Inside `grund-core`, the source layout should match the same category boundaries the later LSP and binding frontends need. A single large crate root hides ownership and makes spec-to-code citations harder to place.

## placement: Where the file layout sits

Not a component: the rule for how the engine's files are named, owned and sized, whichever component they implement. It takes the component list of [§AR-system.2](README.md#2-components) and gives each file exactly one owner among the categories below, which is what a reader and `fissile` need to find and bound a file ([§AR-system.5](README.md#5-what-holds-the-shape)). It says nothing about what a component may know; that is [§AR-system.4](README.md#4-dependency-direction).

## 1. Module categories

`crates/grund-core/src/lib.rs` stays the engine crate entrypoint and public Rust API surface (`check`, `show`, `scan`, and the shared data types), while implementation code lives in smaller category files under `crates/grund-core/src/`.

What each category implements, consumes and must not know is its component's subsection in [§AR-system.2](README.md#2-components), and the table's third column says which. The categories are the file-name prefixes below.

A file belongs to the category whose prefix its name carries — `scanner_walk.rs` to **scanner**, `init_block.rs` to **init** — and `lib.rs` is the one file outside them, as the crate entrypoint. The prefixes each category owns:

| Category | File-name prefixes | Component |
|---|---|---|
| **model** | `model`, `values` | [§AR-system.2.2](README.md#22-model) |
| **config** | `config` | [§AR-system.2.3](README.md#23-config) |
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
| **grammar** | `grammar`, `markdown_fence`, `comment_line`, `comment_block`, `shorthand`, `inline_note_layout`, `never_rewrite` | [§AR-system.2.1](README.md#21-grammar) |
| **workspace** | `workspace` | [§AR-system.2.4](README.md#24-workspace) |
| **integrations** | `integrations`, `fetch`, `fetch_write` | [§AR-system.2.8](README.md#28-writers) |
| **lsp** | `lsp`, `on_type` | [§AR-system.2.7](README.md#27-queries) |
| **compat** | `compat` | [§AR-system.2.9](README.md#29-api) |

`tests/integration/test_module_categories.py` holds this table against the tree: every implementation file owned by exactly one row, every prefix owning a file.

## 2. Refactor boundary

Splitting the core and CLI crates is an architectural refactor only: it must not change CLI output, diagnostics, scan behavior, template bytes, or public entrypoints. The CLI package may keep calling compatibility command adapters while narrower data-returning APIs are introduced, but embedders use the public API in `api.rs`.

## 3. File size

Each implementation file under `src/` stays below 500 lines of code. If a category grows past that limit, split it into smaller category subfiles, or into a category directory with submodules, rather than letting a new monolith form.

## 4. Citation placement

Code moved into a category file keeps the same behavior citations it carried before. When a whole category implements an architectural behavior, the file or module-level comment may cite this spec; narrower functional clauses remain cited on the specific function or branch that implements them.

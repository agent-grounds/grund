# grund-core

`grund-core` is the shared Rust engine behind [`grund`](https://crates.io/crates/grund).
It contains the scanner, resolver, checker, formatter, config loader, and
structured APIs for the CLI surfaces (`check`, `show`, `refs`, `list`, `cover`,
`fmt`, `fetch`, `id`, `init`, `integrations`, dynamic completions, and config
inspection). The published CLI owns argument parsing, text/JSON rendering, and
exit-code mapping; `grund-core` returns data for embedders and frontends. This
package role follows
[§FS-distribution.1](../../docs/functional-spec/FS-distribution.md#1-targets).

Most users should install the CLI crate instead:

```bash
cargo install grund
```

## What 1.0 promises

Use `grund-core` directly when embedding the engine in Rust code. `1.0` freezes
exactly the names its documentation shows: the entry points `check`,
`check_with_opts`, `scan`, `show`, `list`, `list_sizes`, `refs`, `refs_outcome`,
`cover`, `cover_text`, `cover_lines`, `format_references`, `propose_id`,
`complete_ids`, `init`, `fetch_snapshot` and `integrations`, the typed refusals
`ShowQueryError` and `FmtScanAbort`, and every type their signatures reach
([§FS-distribution.3.1.2](../../docs/functional-spec/FS-distribution.md#312-what-10-freezes)).

A root name the documentation does not show is `#[doc(hidden)]`: a seam one of
grund's own frontends reads. It still links and still works, but it is outside
the promise and may change in any minor release.

Build an option record from `Default` and assign its fields, and read output
records and refusals rather than building them:

```rust
let mut opts = grund_core::CheckOpts::default();
opts.path = "docs".into();
let output = grund_core::check_with_opts(opts)?;
```

Since `0.21.0` the promised records and enums are `#[non_exhaustive]`, so a
struct literal of one does not compile, and a `match` on a promised enum ends in
a wildcard arm:

```rust
fn label(mode: grund_core::ShowMode) -> &'static str {
    match mode {
        grund_core::ShowMode::Toc => "toc",
        grund_core::ShowMode::Full => "full",
        _ => "another mode",
    }
}
```

The 30 integrations names deprecated in `0.19.0` were removed in the same
release; `integrations` answers what each of them did.

The [Rust API guide](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/rust-api.md)
has examples and the migration tables, and
[`examples/rust-api`](https://github.com/agent-grounds/grund/tree/main/examples/rust-api)
is a runnable embedder.

Project home: <https://github.com/agent-grounds/grund>

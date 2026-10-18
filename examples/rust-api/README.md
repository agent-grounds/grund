# Install an integration from Rust

This example embeds `grund-core` as a crate and installs one rendering-layer
integration through its one integrations entry point, `integrations`
([§FS-distribution.3.1.3](../../docs/functional-spec/FS-distribution.md#313-integrations-is-the-one-integrations-entry-point)).
It asks once and reads the answer: one event per target it touched, each an
outcome and a path ([§FS-integrations.1.3.2](../../docs/functional-spec/FS-integrations.md#132-the-output)).
From the repository root:

```sh
cargo run -p grund-rust-api-example -- kitty --write
```

It prints one line per target, the same verbs
`grund integrations kitty --write` reports:

```text
appended /home/you/.config/kitty/kitty.conf
wrote /home/you/.local/bin/grund-open
appended /home/you/.config/grund/config.toml
appended /home/you/.claude/CLAUDE.md
```

An event that carries a note gets a `note:` line on stderr. `IntegrationNote`
is `#[non_exhaustive]`, like every promised enum, so `describe` in
`src/main.rs` ends its `match` in a wildcard arm for a note a later `grund-core`
adds; without that arm the example does not compile.

The CLI and this example read the same answer, so they write the same files with
the same bytes; `tests/same_install_as_the_cli.rs` runs both in sandboxed homes
and compares them. Build an option record from `Default` and assign its fields,
and read the output rather than building it
([§FS-distribution.3.1.2](../../docs/functional-spec/FS-distribution.md#312-what-10-freezes)).

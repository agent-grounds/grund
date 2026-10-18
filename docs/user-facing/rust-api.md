# Embed grund from Rust

`grund-core` is the engine every `grund` frontend is built on: the CLI, the
language server, and the Python and Node packages. A Rust program can depend on
it directly and ask the same questions those frontends ask. Each call returns
data and prints nothing ([§FS-distribution.3.1](../functional-spec/FS-distribution.md#31-rust-grund-core-crate)).

```toml
[dependencies]
grund-core = "0.21"
```

[`examples/rust-api`](../../examples/rust-api/) is a runnable embedder. It
installs a terminal integration and checks that it wrote the same bytes as
`grund integrations kitty --write`.

## What 1.0 promises

`1.0` freezes exactly the names the crate's documentation shows
([§FS-distribution.3.1.2](../functional-spec/FS-distribution.md#312-what-10-freezes)):

- **The entry points.** `check`, `check_with_opts`, `scan`, `show`, `list`,
  `list_sizes`, `refs`, `refs_outcome`, `cover`, `cover_text`, `cover_lines`,
  `format_references`, `propose_id`, `complete_ids`, `init`, `fetch_snapshot`
  and `integrations`.
- **The typed refusals.** `ShowQueryError`, `FmtScanAbort` and
  `FmtConflictAbort`, which an error from `show` or `format_references`
  downcasts to, and `InitError`,
  `FetchFailure` and `IntegrationsError`, which `init`, `fetch_snapshot` and
  `integrations` return. An entry point with no typed refusal returns
  `anyhow::Result`.
- **Every type those signatures reach,** transitively. That makes 113 names.

A `1.x` release may add a name, a field or a variant, and may deprecate a name.
Only `2.0` removes or retypes one.

**Hidden names.** Every other root name is `#[doc(hidden)]`. It is a seam that
one of grund's own frontends reads: the configuration records and readers
(`Project`, `Run`, `effective_config`), the editor's snapshot API, or the host
packages' carriers. A hidden name still links and still works, but it is
outside the promise, and a minor release may change it. The `test-*` Cargo
features are outside the promise too.

## Build options from `Default`

Build an option record from `Default` and assign its fields. Read an output
record or a refusal; never build one.

```rust
use grund_core::{CheckOpts, check_with_opts};

let mut opts = CheckOpts::default();
opts.path = "docs".into();
opts.include_suggestions = true;
let output = check_with_opts(opts)?;
for finding in &output.report.errors {
    println!("{}:{}: {}", finding.path.as_deref().unwrap_or(""), finding.line.unwrap_or(0), finding.message);
}
```

That is what lets `1.x` add a field or a variant. Every promised record and
enum is `#[non_exhaustive]`, so outside the crate a struct literal of one does
not compile, and neither does a `match` on a promised enum without a wildcard
arm. Code written as above compiles.

## Match a promised enum with a wildcard arm

End a `match` on a promised enum in a `_` arm, for the variant a later release
adds. Name every variant you render, and decide what the arm does for one you
have not seen:

```rust
use grund_core::IntegrationOutcome;

fn changed(outcome: IntegrationOutcome) -> bool {
    match outcome {
        IntegrationOutcome::Wrote | IntegrationOutcome::Appended | IntegrationOutcome::Updated => true,
        IntegrationOutcome::Unchanged | IntegrationOutcome::Skipped => false,
        _ => true,
    }
}
```

A struct pattern on a promised record ends in `..` for the same reason.

## Examples

Read one declaration the way `grund <ID> --toc` does:

```rust
use grund_core::{ShowMode, ShowOpts, show};

let mut opts = ShowOpts::default();
opts.path = ".".into();
opts.mode = ShowMode::Toc;
let output = show("FS-distribution", opts)?;
print!("{}", output.body);
```

Walk a tree once and read what it declares:

```rust
let catalog = grund_core::scan(std::path::Path::new("."))?;
for declaration in catalog.declarations.values().flatten() {
    println!("{}:{}", declaration.file.display(), declaration.line);
}
```

Install an integration, as `grund integrations kitty --write` does
([§FS-distribution.3.1.3](../functional-spec/FS-distribution.md#313-integrations-is-the-one-integrations-entry-point)):

```rust
use grund_core::{IntegrationClient, IntegrationsOpts, integrations};

let mut opts = IntegrationsOpts::default();
opts.client = Some(IntegrationClient::Kitty);
opts.write = true;
match integrations(opts) {
    Ok(out) => {
        for event in &out.events {
            println!("{} {}", event.outcome.name(), event.path.display());
        }
    }
    Err(err) => eprintln!("error: {err}"),
}
```

Without `write`, the same call only describes: `detected`, one status per
client, and the named client's `artifact`. A refusal is an `IntegrationsError`.
Its `failure` says why, and its `output` keeps what the run wrote before it
stopped ([§FS-integrations.1.3.3](../functional-spec/FS-integrations.md#133-the-refusal)).

A `format_references` run that needs the whole declaration set and finds a merge
conflict left in a declaration writes nothing, and its error downcasts to
`FmtConflictAbort`. Its `conflicts` are the blocks' `conflict-marker` findings,
the ones `check` reports, so a caller can print them as `grund fmt` does
([§FS-fmt.7.4](../functional-spec/FS-fmt.md#74-no-write-without-a-complete-model)):

```rust
use grund_core::{FmtConflictAbort, FmtOpts, format_references};

let mut opts = FmtOpts::default();
opts.path = ".".into();
opts.write = true;
if let Err(err) = format_references(opts) {
    if let Some(abort) = err.downcast_ref::<FmtConflictAbort>() {
        for conflict in &abort.conflicts {
            let path = conflict.path.as_deref().unwrap_or_default();
            let line = conflict.line.unwrap_or_default();
            eprintln!("error: nothing was rewritten: {path}:{line}: {}", conflict.message);
        }
    }
}
```

## Migrating

### Removed in 0.19.0

These four root aliases were deprecated in `0.17.0`.

| Removed | Use instead |
|---|---|
| `Findings` | `Catalog`, the record it named |
| `Config` | `effective_project`, which returns the `Project` it was built from (hidden) |
| `KindConfig` | the `schema` rows of a `Project` |
| `CitationRules` | `rules.citations` of a `Project` |

### Non-exhaustive records and enums in 0.21.0

The crate documentation gave notice of this from `0.19.0`.

| What stopped compiling | Write instead |
|---|---|
| A struct literal of an option record, `ShowOpts { mode: ShowMode::Toc, ..Default::default() }` | `let mut opts = ShowOpts::default(); opts.mode = ShowMode::Toc;` |
| A struct literal of an output record or a refusal | Read the one an entry point returns |
| A `match` on a promised enum that names every variant | Add a `_` arm |
| A struct pattern on a promised record without `..` | End the pattern in `..` |

### Removed in 0.21.0

`grund integrations` used to be assembled from 34 root names. Four of them stay,
because `integrations` is spelled in them: `IntegrationClient`, `InstallKind`,
`ConversationRendering` and `ConversationTarget`. The other 30, and nine methods
of the kept types, were deprecated in `0.19.0` and removed in `0.21.0`. Each one
is answered by a field of `integrations`.

| Removed | Read instead |
|---|---|
| `detect_clients` | `IntegrationsOutput::detected` |
| `integration_is_current`, `vscode_integration_is_current` | `IntegrationClientStatus::installed` |
| `IntegrationClient::install_kind` | `IntegrationClientStatus::install_kind` |
| `IntegrationClient::config_target` | `IntegrationClientStatus::config_target` |
| `IntegrationClient::snippet` | `IntegrationArtifact::snippet` |
| `IntegrationClient::is_terminal`, `GRUND_OPEN_RESOLVER` | `IntegrationArtifact::resolver` |
| `RESOLVER_TARGET` | `IntegrationArtifact::resolver_target` |
| `VSCODE_PACKAGE_JSON`, `VSCODE_EXTENSION_JS` | `IntegrationArtifact::package_json`, `IntegrationArtifact::extension_js` |
| `BlockOutcome`, `block_outcome_verb`, `merge_outcomes` | `IntegrationEvent::outcome` and `IntegrationOutcome::name` |
| `WEZTERM_APPLY_CALL`, `needs_wezterm_wiring` | `IntegrationNote::UnwiredWezterm` |
| `GLOBAL_AGENT_INSTRUCTION_TARGETS`, `GlobalAgentTarget`, `LinkSupport`, `known_agent`, `known_agents_list`, `agent_override_table` | `IntegrationAgent::ALL`, `IntegrationAgent::from_name` and `IntegrationsOpts::agent`, with `IntegrationNote::Guidance` for what each agent got |
| `known_clients_line` | `IntegrationClient::ALL` and `name` |
| `ConversationTarget::accepted_list` | `ConversationTarget::ALL` and `name` |
| `IntegrationClient::install_command` | your own command line; the CLI spells it `grund integrations <client> --write` |
| `install_managed_block`, `install_agent_guidance_block`, `install_reference_key`, `write_resolver_script`, `expand_target`, `IntegrationClient::{comment_prefix, prepends_block, fresh_config_scaffold}`, `INTEGRATIONS_BLOCK_VERSION` | `integrations` with `write` set |
| `read_optional_text`, `user_grund_config_path`, `scan_user_config`, `UserConfigScan`, `USER_CONFIG_TARGET` | `integrations` reads the user configuration; its cautions are `IntegrationsOutput::warnings` |

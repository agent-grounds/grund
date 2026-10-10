# Repository options

## Workspaces and sub-projects

In a monorepo, keep each sub-project as its own local namespace and let the root
config orchestrate them:

```toml
project_name = "root"

[workspace]
members = ["apps/api", "packages/*"]
include_root = true
```

Local citations stay short:

```markdown
§FS-session
```

Cross-project citations add a stable alias before the ID:

```markdown
§api/FS-session
§root/GOAL-compatibility
```

`grund check` at the workspace root validates the root project and every member,
without letting root scans accidentally absorb member declarations, even if the
root `[scan] include` names a path inside a member. Members without
a `grund.toml` of their own use the canonical defaults, and a member that declares its
own `[workspace]` block is rejected in v1. Each project can also set a one-line
`project_description` next to `project_name`; `grund init` renders it beside
the alias in the generated workspace member list (see
[§FS-config](../functional-spec/FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up)). Cross-repository aliases — an
alias like `payments/FS-refunds` resolving to a neighboring repo — are not yet
supported.
See [§FS-workspace](../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace).

An independently checked project's canonical root also bounds directory
symlinks: outward directory targets are not scanned, including from inside a
workspace member, while in-root directory links, file links, and intentional
parent-relative `[scan] include` paths remain readable
([§FS-config.3.5.1](../functional-spec/FS-config.md#351-a-symlink-in-the-tree-is-followed)).

## Keep shared values consistent

A citable kind can opt its numbered fields into exact value checking:

An explicit `[[kinds]]` list replaces the implicit default kinds; copy the default rows from [`FS-config` section 3.4.4](../functional-spec/FS-config.md#344-the-default-kinds) first, or existing declarations may disappear from `list` and `check` remains green because those kinds no longer exist.

```toml
[[kinds]]
kind = "CONST"
folder = "values"
index = false
format = "{kind}-{slug}"
values = true
```

```markdown
# CONST-field-price: Reference field price
## 1. 1200

The offer uses `1200.0` (§CONST-field-price.1).
```

The backticks, one space, parentheses, marker, and positive numeric field are intentional syntax. `grund check` accepts exact decimal
equivalents such as `1200` and `1200.0`, and reports `value-mismatch` if the authored component drifts.
Leave the field off to bind the whole value at once: the literal is then every component joined by one ASCII space, such as `1200.0 USD` for `[1200, "USD"]` ([§FS-values.3.1.2](../functional-spec/FS-values.md#312-a-binding-aimed-at-the-root)).
A value can also live inside any ordinary scanned declaration without a
kind opt-in: end its numeric section heading with the exact marker, then give it
one contiguous level of numbered components ([§FS-values.2.4](../functional-spec/FS-values.md#24-embedded-section-value-roots)):

```markdown
# FS-pricing: Pricing rules
## 2. Regional floor <!-- grund:value -->
### 2.1. 1200

## 3. Use

The floor is `1200.0` (§FS-pricing.2.1).
```

The marked section and component keep their ordinary dotted identities for
`show`, `refs`, completion, formatting, and editor navigation. JSON arrays at
an opted-in kind home can provide a whole declaration instead, so application
code can read the source directly. See the complete
[first-class values guide](../user-facing/values.md) and the runnable
[`examples/values/`](../../examples/values/) repository ([§FS-values](../functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value)).

## Cite external facts without making checks depend on the network

External tickets and similar facts can use their own numeric grammar while the rest of the repository keeps slug IDs. Configure a committed snapshot home and
one repository-owned fetcher, then materialize a cited fact deliberately:

```toml
[[kinds]]
kind = "TICKET"
file = "docs/tickets.md"
format = "{kind}-{number}"
resolve = "should"
fetch = "scripts/fetch-ticket"
```

```sh
grund fetch TICKET-1234
```

Checks, queries, formatting, completion, and the LSP never run that program;
they resolve only the committed Markdown it produced. A missing `should`
snapshot is a warning with the fetch command, while `must` remains an error.
See the [external facts guide](../user-facing/external-facts.md) and runnable
[`examples/external-tickets/`](../../examples/external-tickets/) repository
([§FS-fetch](../functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)).

## Write the config in version 2

A `grund.toml` that writes `grund_config_version = 2` is spelled by concern:
`[schema]` says what exists, `[rules]` how it must relate, and `[presentation]`
which bytes `grund` writes. Every constraint takes one strength from `must`,
`warn`, `should` and `may`, so a rule can be adopted as a standing warning and
promoted to an error later without changing its spelling:

```toml
grund_config_version = 2
project_name = "example"

[schema.sources]
languages = ["markdown"]

[schema.kinds.FS]
files = ["requirements.md"]

[schema.kinds.GOAL]
files = ["docs/goals.md"]

[rules.citations.FS]
warn = ["GOAL"]

[rules.citations.grounding]
warn = "file"
```

`[rules.citations.<KIND>]` writes the same lists as v1's
[citation directions](citation-directions.md), with two more levels: `warn` is
the `must` obligation and `warn-not` the `must-not` prohibition, each reported
as a standing warning that leaves the exit code at 0. A v2 `default` may
forbid, as `should-not`, `warn-not` or `must-not`, but never obliges:
`default = "must"` is refused.

Each row is its own `[schema.kinds.<NAME>]` table, and a key is read only in
the table written above it. Its defaults are fixed by the version rather than
by the binary, and a version-1 file keeps its meaning unchanged.
`grund config show` prints a version-2 file back in version-2 spelling. What
this release reads, and the clauses it still refuses, are in
[§FS-config-v2](../functional-spec/FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults).

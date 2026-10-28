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
supported. A project whose citers live in a repository it cannot read can still
keep those points cited: see
[Citers in a repository you cannot read](#citers-in-a-repository-you-cannot-read).
See [§FS-workspace](../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace).

A check from a member directory, or `grund check <member>`, checks that member
locally and still rejects cross-project citations. When the refused alias names
the verified enclosing workspace's root project, the finding explains the
membership, gives the root's location from your current directory, and tells
you to run `grund check` there ([§FS-workspace.5.1.1](../functional-spec/FS-workspace.md#511-verified-enclosing-workspace-guidance)). Follow that location to
validate the cross-project target; the root check can still report a missing
declaration or section. A genuinely unknown alias needs correcting rather than
being treated as the root project's name.

Every project's managed block lists the workspace's members, so changing the
membership — adding or removing a member, or editing a `project_description` —
makes the other projects' blocks stale too. Initializing a project does not:
each member is listed by its directory, never by whether its `AGENTS.md` exists,
so a fresh workspace needs one `grund init` per project, in any order
([§FS-init.2.3.4.15.6](../functional-spec/FS-init.md#234156-the-link-target)). The
root's `grund check` reports each one and names the directory to run
`grund init` in, as in ``run `grund init` in packages/billing to refresh``;
running it in each named directory clears them
([§FS-errors.3.6.1](../functional-spec/FS-errors.md#361-the-final-templates)). Only
that run judges the two workspace sections: a check inside a member, at a nested
group's root, or in a checkout without one of the `optional_members` leaves them
alone, and in the last case the absent member's warning says so
([§FS-check.3.5.5](../functional-spec/FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)).
In such a checkout `grund init` draws the same line: it keeps each managed
block's `### Workspace members` section exactly as it is, refreshes the rest of
the block, and prints one note per absent member, so `grund init --check` there
passes when that list is the only difference. A block written there for the
first time lists only the members the checkout has, and the full checkout's root
`check` reports it until `grund init` runs there. The member list itself changes
only from a full checkout, whose root `check` names every block still waiting
([§FS-init.2.3.4.15.8](../functional-spec/FS-init.md#234158-while-an-optional-member-is-absent)).

An independently checked project's canonical root also bounds directory
symlinks: outward directory targets are not scanned, including from inside a
workspace member, while in-root directory links, file links, and intentional
parent-relative `[scan] include` paths remain readable
([§FS-scan.scope.links](../functional-spec/FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)).

### Citers in a repository you cannot read

An open project's gate never sees a closed repository that cites it. Say the
open `substratevm` declares `FS-sec`, `FS-rel` and `FS-old`, and the closed `ee`
cites `substratevm/FS-sec.1` and `substratevm/FS-rel.access.1`. Checked on
its own, `substratevm` warns `declared but never cited` for all three, and
nothing stops someone deleting the two that a shipped product depends on. The
recipe below makes the open repository's own `check` guard them, with nothing
from `ee` but the names of points the open tree already declares.

1. **List the citations, on the closed side.** `grund cover --format json` keeps
   a cross-project citation's alias even where the alias does not resolve
   ([§FS-cover.2.1](../functional-spec/FS-cover.md#21-a-cross-project-citation-counts)), so `ee` can list its citations into `substratevm` alone.
   Keep the ones into `substratevm` and drop the alias with `jq`:

   ```bash
   (cd ee && grund cover --format json) \
     | jq -r '.citations[] | select(.id | startswith("substratevm/"))
              | "- §\(.id | ltrimstr("substratevm/"))\(if .section then "." + .section else "" end)"' \
     | sort -u
   ```

2. **Commit the list upstream as a page,** such as
   `substratevm/docs/downstream-citers.md`: a heading, one sentence saying where
   the list came from and the command that regenerates it, then the lines the
   filter printed. Run `grund fmt --write` in `substratevm`, which links each
   line and records any cited section the section lock covers.

3. **Regenerate it whenever `ee` changes what it cites.** The page is a copy,
   so it goes stale: a point `ee` stopped citing stays "used" until the page is
   regenerated, and a point it started citing reads as unused until then. Rerun
   steps 1 and 2 on the closed side's schedule — on each `ee` release, or in its
   CI — and replace the list rather than editing it.

A citation counts wherever the run reads it ([§FS-check.4.1.4](../functional-spec/FS-check.md#414-a-citation-anywhere-in-the-resolution-scope-counts)), so the page is an
ordinary citing file. From then on `substratevm`'s `check` warns only where
nothing cites, deleting a listed point is an error at the page —
`docs/downstream-citers.md:9: error: unknown reference FS-sec; did you mean FS-rel?`
— and moving a listed section's text to another address is refused by the
section lock ([§FS-section-lock](../functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)). [`examples/downstream-citers`](../../examples/downstream-citers/)
runs the three steps.

The recipe has four limits, and each is part of using it:

- **The page only records that a point is cited,** not what `ee` relies on it
  for. Reviewing a change to a listed point still means asking the closed side.
- **It is stale until someone regenerates it.** That is why regeneration is a
  step of the recipe, not an afterthought.
- **Under a closed `[rules.citations] default = "must-not"`,** the page is in the
  homeless `code` row, which must `may` the kinds it lists — for example
  `[rules.citations.code]` with `may = ["FS"]`. Otherwise every line is refused:
  `docs/downstream-citers.md:8: error: code must not cite FS (citation direction) — re-point the citation or downgrade it to a plain Markdown link`.
- **The closed side's own gate still has to resolve its `substratevm/...`
  citations,** for example through a parent workspace that lists the open
  project as a member. Run alone, `ee`'s `check` reports
  `docs/tck.md:3: error: unknown project alias substratevm`; only `cover`
  works there without one.

Never write the page by hand, and never use it to silence a warning for a point
nobody cites. The unused warning is how a tree tells its maintainer that a point
has fallen out of use ([§DF-index-not-an-inbound-citation.3](../decisions/functional/DF-index-not-an-inbound-citation.md#3-alternatives-considered)); a page generated
from real citations keeps that signal, and a hand-written or never-refreshed one
empties it. The page is your file: grund neither keeps it nor refreshes it
([§FS-non-goals.6](../functional-spec/FS-non-goals.md#6-decision-database-audit-log-history-tracking)). Remote projects ([§FS-remote-projects](../functional-spec/FS-remote-projects.md#fs-remote-projects-a-project-cites-another-repositorys-declarations-from-a-committed-pinned-projection)) run the other way — a
consumer checks its citations against a committed copy of the cited repository —
and tell the cited repository nothing about who cites it, so this page stays the
way to do that.

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
kind = "GH"
file = "docs/tickets.md"
format = "{kind}-{number}"
resolve = "should"
fetch = "scripts/fetch-ticket"
```

```sh
grund fetch GH-1234
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

### Several places per row

A v2 row lists its places, as many as the repository needs, and they may nest:

```toml
[schema.kinds.FS]
folders = ["docs/spec/core", "docs/spec/plugins"]

[schema.kinds.page]
citable = false
folders = ["docs"]

[schema.kinds.code]
folders = ["crates", "scripts"]
```

A file listed in a row's `files` belongs to that row; otherwise the deepest
folder holding it decides, so `docs/spec/core/login.md` is `FS`'s, not
`page`'s; and a file in no place belongs to `code`. The row named `code` is the
complement: it declares no IDs, and listing places on it narrows the scan to
the places written. With no `code` row every file under the config root is
read. One place belongs to one row, a place stays inside the config root and
outside every workspace member, and `scan = false` covers every place of its
row. The runnable [`examples/places/`](../../examples/places/) walks through it
([§FS-config-v2.schema.places](../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)).

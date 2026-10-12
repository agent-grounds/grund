<p align="right"><a href="https://github.com/agent-grounds/grund/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/agent-grounds/grund/actions/workflows/ci.yml/badge.svg"></a> <a href="https://crates.io/crates/grund"><img alt="crates.io" src="https://img.shields.io/crates/v/grund.svg"></a> <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue.svg"></a></p>

# grund

> **Keep your agents grounded** — specs, docs, and code as one knowledge graph, checked on every commit.

<!-- grund:fmt off -->
<table>
<tbody>
<tr>
<td valign="top" width="400">
<b>1 · Declare</b><br>
Give a requirement an ID in <a href="examples/quickstart/repo/requirements.md"><code>requirements.md</code></a>
<pre lang="markdown">
&#35; FS-login: Login
&#35;&#35; format: Username format
&#35;&#35;&#35; format.1: Surrounding whitespace
Trim surrounding whitespace.
</pre>
</td>
<td valign="top" width="400">
<b>2 · Cite</b><br>
Cite it from the code in <a href="examples/quickstart/repo/src/login.py"><code>src/login.py</code></a>
<pre lang="python">
def clean_username(username):
    &#35; &#167;FS-login.format.1
    return username.strip()&#10;&#10;
</pre>
</td>
</tr>
</tbody>
<tbody>
<tr>
<td valign="top" width="400">
<b>3 · Read</b><br>
Look it up before you change the code
<pre lang="console">
$ grund FS-login.format.1
&#35;&#35;&#35; format.1: Surrounding whitespace
Trim surrounding whitespace.
</pre>
</td>
<td valign="top" width="400">
<b>4 · Check</b><br>
Misspell <code>format</code> in the citation, and CI fails
<pre lang="console">
$ grund check
… section not found: FS-login.fromat.1; …&#10;&#10;
</pre>
</td>
</tr>
</tbody>
</table>
<!-- grund:fmt on -->

## Default structure

`grund init` lays a project out in layers, each citing the one that explains it, under
rules it writes commented out in `grund.toml` until you adopt them:

|           | Kind        | Home                            | Cites                                   |
|-----------|-------------|---------------------------------|-----------------------------------------|
| Why       | GRUND       | `docs/grund.md`                 |                                         |
| Where     | GOAL        | `docs/goals.md`                 | should cite GRUND or GOAL               |
| What      | FS          | `docs/functional-spec/`         | should cite GOAL or FS; **never** AR    |
|           | REQ         | `docs/requirements/`            | should cite GRUND or GOAL; **never** AR |
| How       | AR          | `docs/architecture/`            | should cite FS or GOAL                  |
| Proof     | e2e         | `tests/e2e/`                    | **must** cite FS; avoid AR              |
|           | integration | `tests/integration/`            | should cite AR                          |
| Code      | code        | any other file                  | should cite FS or AR                    |
| Decisions | DF          | `docs/decisions/functional/`    | should cite FS or GOAL                  |
|           | DA          | `docs/decisions/architectural/` | should cite AR or FS                    |
| Plan      | RM          | `docs/roadmap.md`               |                                         |

## Agent instructions

`init` also writes the structure into your agent's instructions, a managed block in
`AGENTS.md` that `grund init --check` keeps current. From [ours](AGENTS.md), how an
agent reads a citation:

> A `§<ID>` is a pointer to a fact, not a file path. Resolve it with `grund` and climb only as far as needed:
>
> - `grund <ID>` — the lead (heading-less, cut at the first child section). The cheap first read for a bare `§<ID>` citation.
> - `grund <ID> --toc` — the lead plus the nested section map. Use to choose which subsection to fetch next.
> - `grund <ID> --full` — the entire body. Escalate to this when narrower reads aren't enough.

Where each kind lives in this repository:

> - [GRUND](docs/grund.md): Why: project motivation
> - [GOAL](docs/goals.md): Where: project direction and outcomes
> - [FS](docs/functional-spec): What: behavior, requirements, and constraints
> - [REQ](docs/requirements): Hard requirements: what grund must never break
> - [AR](docs/architecture): How: high-level implementation, structure, and design
> - …

How it writes:

> - **Cite as you write.** Place `§<ID>` at the point a claim or behavior is made — on the doc-comment for a whole behavior, inline beside the clause it enforces.
> - …
> - **Inline citation style.** Inline notes: ≤ 1 line preferred, hard cap 3 lines; ≤ 100 columns. A note is one comment block: a blank line splits it, an empty comment line does not. Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell or SQL comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there.
> - **Always cite the most-specific point.**

And the table's rules, as it reads them:

> `must`/`never` are `grund check` errors; `should`/`avoid` are suggestions (`grund check --suggestions`). …
>
> - Each **FS** declaration should cite GOAL or FS; never cite AR.
> - Each **REQ** declaration should cite GRUND or GOAL; never cite AR.
> - …
> - Each file in **tests/e2e/** must cite FS; avoid citing AR.

For an agent without the managed block, the loop fits in one rule for its system prompt:

> When you see `§<ID>` or `§<ID>.<section>` in any file you are reading, run `grund <ID>[.<section>]` and treat the output as the authoritative definition. Do not paraphrase or guess — quote what `grund` returned, or cite the ID and move on.

## Quick start

```bash
cargo install grund
```

Run the following **inside your repository** if you are starting with new specs:

```bash
grund init --docs     # writes config, agent instructions, and doc scaffolds
grund check
```

**Already have specs?** Hand the setup to your agent:

```bash
grund agent-setup-instructions   # the guided adoption workflow, for your agent to follow
```

It has your agent inspect the repository, recommend settings for the specs you have,
confirm every option with you, write `grund.toml`, run `grund init` and validate
([§FS-init.5](docs/functional-spec/FS-init.md#5-agent-setup-instructions)). To do it by hand, [map their existing
locations](docs/user-facing/setup.md) before running `grund init`; use `--docs` only
when you want scaffolds.
`init` writes your agent's instructions from `grund.toml` — ours is
[`AGENTS.md`](AGENTS.md). Add `grund check` to [pre-commit and CI](docs/user-facing/checking.md#in-pre-commit-and-ci).
[Installation](docs/user-facing/installation.md) covers prebuilt binaries, supported
platforms, the Python and Node APIs, and local candidate builds.

## Declaration format

A heading `# <ID>: <title>` in the kind's home declares an ID. The headings under it
are its sections, each at the depth its path names: numbered, or named once
`[id] named_sections = true` is set. This README's own requirement mixes both:

```markdown
# REQ-readme: the README is the grounded shop window
…
## 1. What it must say
…
### 1.1 The loop
…
## evidence: Positioning and performance claims have evidence
```

They are cited as [§REQ-readme.1.1](docs/requirements/REQ-readme.md#11-the-loop) and [§REQ-readme.evidence](docs/requirements/REQ-readme.md#evidence-positioning-and-performance-claims-have-evidence).

A declaration can also live in code: a doc-comment that opens with the ID declares it,
and a link from the kind's index enrolls it with no stub file
([§FS-check.3.18](docs/functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)). Grund's own `AR-checker` is declared only on the
function it describes, in [`crates/grund-core/src/checker/report.rs`](crates/grund-core/src/checker/report.rs):

```rust
/// AR-checker: how grund validates the scanner's findings
///
/// …
/// ## placement: Where the checker sits
/// …
pub(crate) fn check_on_disk(
```

`grund AR-checker` prints that prose without the `///` markers
([§FS-show.2.3](docs/functional-spec/FS-show.md#23-source-declarations-in-code-and-doc-comments)).

## Citation format

```plaintext
     ┌─────────────────── citation ───────────────────┐
            ┌───────────── ID ───────────────┐
  [§] [alias /] KIND - [number -] slug [.section]
   │     │        │       │         │       │
   │     │        │       │         │       └─ dotted path (.3, .3.1, .format, …)
   │     │        │       │         └───────── [a-z0-9][a-z0-9-]*  (default slug_pattern)
   │     │        │       └─────────────────── optional ordinal (e.g., 001)
   │     │        └─────────────────────────── GRUND│GOAL|FS│AR│DF│DA│RM│[custom]
   │     └──────────────────────────────────── project alias for subprojects or monorepo
   └────────────────────────────────────────── citation marker (writing only)
```

The [structure guide](docs/user-facing/structure.md) covers the three ID schemes and
the rest of the citation syntax.

## Workspaces

A monorepo of independently checked projects is a workspace: each member keeps its
own `grund.toml` and its own IDs, the root lists the members, and a citation that
crosses a project boundary names the target's alias. From the
[workspace example](examples/workspace/), the root's `grund.toml` and a member's spec:

```toml
project_name = "root"
# …
[workspace]
members = ["packages/*"]
include_root = true
```

```markdown
# FS-session: API session

The API session model lives in this project. The platform-level coordinator
is §root/FS-platform.
```

`grund check` at the root checks every member, each in its own namespace
([§FS-workspace](docs/functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace)).

## External facts

A ticket from GitHub, Jira, or Linear is cited like any spec once a kind names the
executable that fetches it. `grund fetch <ID>` saves it as a declaration in the
repository; every other command reads that committed snapshot offline
([§FS-fetch](docs/functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)). From the [external-tickets example](examples/external-tickets/):

```toml
[[kinds]]
kind = "TICKET"
file = "docs/tickets.md"
format = "{kind}-{number}"
resolve = "should"
fetch = "scripts/fetch-ticket"
```

```console
$ grund check
docs/guide.md:3: warning: no snapshot for TICKET-1234 in docs/tickets.md — run grund fetch TICKET-1234
$ grund fetch TICKET-1234
$ grund TICKET-1234
The issue requests committed snapshots that resolve offline.
```

The [external facts guide](docs/user-facing/external-facts.md) has the fetch contract
and a GitHub integration to start from.

## Values

A number a spec declares can be checked wherever it is used. With `values = true` on a
kind, a declaration's numbered headings are its components, and a backticked literal
beside a citation must equal them
([§FS-values](docs/functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value)). From the [values example](examples/values/):

```markdown
# CONST-field-price: Reference field price
## 1. 1200
## 2. USD
```

```markdown
The reference field price is `1200.0 USD` (§CONST-field-price).
```

A use that drifts from its declaration fails the check:

```console
$ grund check
docs/offer.md:7: error: value mismatch for CONST-discount.1: bound `0.30`, declared `0.25` at values/runtime.json:2
```

The [values guide](docs/user-facing/values.md) covers JSON declarations, values inside
prose specs, and bindings in code comments.

## Make citations clickable

Turn a `§<ID>` in your terminal into something you click, landing at the exact line it cites:

```bash
grund integrations                  # what applies in this environment
grund integrations wezterm          # read the snippet and the resolver first
grund integrations wezterm --write  # install it
```

Supported clients are `codium`, `iterm2`, `kitty`, `tmux`, `vscode`, and `wezterm`. `--write` is a one-time, idempotent user setup — the integration, the `grund-open` resolver, and a global instruction block for whichever agents you have installed. It changes no repository.

**`~/.local/bin` must be on your `PATH`** — that is where the resolver is installed, and it is not there by default on macOS, where a missing `PATH` entry makes every click silently do nothing.

**[Clickable citations](docs/user-facing/clickable-citations.md)** is the full setup guide: the per-client reload each one needs, the manual step WezTerm and iTerm2 require, how to check it works, what to do when a click does nothing, and how to control the citations agents write in conversations. See also [§FS-integrations](docs/functional-spec/FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations).

## Editor support via [LSP](https://microsoft.github.io/language-server-protocol/)

Install the optional language server separately when you want citation completion, editor diagnostics, hover previews, usage counts on declaration titles, definition jumps, document links, references, and live `$$` → `§` formatting:

```bash
cargo install grund-lsp
```

The server speaks LSP over stdio and has no daemon or socket. For IntelliJ
family IDEs, `grund-lsp integrations lsp4ij --write <directory>` generates the
LSP4IJ import template carried by the installed binary
([§FS-lsp.2.4](docs/functional-spec/FS-lsp.md#24-installed-editor-integrations)).

It also completes declared citations: in a Markdown note here,
after `See `, type `§` followed by `FS-ls`, request completion, and choose `FS-lsp`
(“grund ships an optional LSP server”, `docs/functional-spec/FS-lsp.md`).
Acceptance leaves `See §FS-lsp`; starting with
`$$F` works too ([§FS-lsp.1.6](docs/functional-spec/FS-lsp.md#16-declared-id-completion)).
Use your editor's acceptance key; see [authoring and client bindings](docs/user-facing/lsp.md#write-a-citation).
The [setup guide](docs/user-facing/lsp.md) has the complete import and
verification flow plus snippets for VSCode, Vim/Neovim, Emacs, Helix, Zed, and
Sublime Text. Put reusable client config in your editor's **user (global)
settings**, not a per-repo file, so `grund-lsp` works in every project rather
than only repos that ship an editor config.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/lsp/hover-preview.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/assets/lsp/hover-preview-light.png">
    <img src="docs/assets/lsp/hover-preview-light.png" width="96%" alt="Hover preview of [§FS-declarations.checks.misplaced-declaration](docs/functional-spec/FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home) shown directly above the grund code that implements the misplaced-declaration check">
  </picture>
  <br><sub><strong>Hover previews</strong> — the spec and the code that satisfies it, in one frame.</sub>
</p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/lsp/error-reporting.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/assets/lsp/error-reporting-light.png">
    <img src="docs/assets/lsp/error-reporting-light.png" width="96%" alt="Dangling citation diagnostic showing the nearest declared ID hint">
  </picture>
  <br><sub><strong>Error reporting</strong> — dangling citations are flagged inline with a "did you mean" hint.</sub>
</p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/lsp/definitions.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/assets/lsp/definitions-light.png">
    <img src="docs/assets/lsp/definitions-light.png" width="96%" alt="Go-to-definition results listing FS-check.3.7 declaration and citation targets">
  </picture>
  <br><sub><strong>Definitions &amp; references</strong> — jump between a declaration and every §citation of it.</sub>
</p>

## Go deeper

| Guide | For |
|---|---|
| [Checking a repository](docs/user-facing/checking.md) | what `grund check` enforces, its report, `--watch`, `--only`, pre-commit |
| [Structure and IDs](docs/user-facing/structure.md) | kinds, ID schemes, citation syntax, inline declarations |
| [Querying grund](docs/user-facing/querying.md) | selectors, batch reads, `refs`, `cover` |
| [Reviewing code](docs/user-facing/reviewing.md) | a citation's blast radius, diff recipes |
| [Chapter rules](docs/user-facing/rules.md) | checked rules in controlled English — [runnable example](examples/rules/) |
| [Values](docs/user-facing/values.md) | one value, declared once, checked wherever it is quoted |
| [External facts](docs/user-facing/external-facts.md) | cite tickets as committed offline snapshots |
| [Workspaces](docs/user-facing/repository-options.md#workspaces-and-sub-projects) | citations across sub-projects |
| [Related work](docs/related-work/REL-traceability-tools.md) | `grund` beside the requirements-traceability tools it descends from |
| [All guides](docs/user-facing/README.md) | each beside its runnable example |

`grund --help` is one screen; `grund <command> --help` is one page with flags, examples,
and exit codes.

The [benchmark report](docs/benchmarks.md) preserves a 2026-05-20 local timing run and a historical instruction-count snapshot, with methodology and build assumptions. [Current CI](docs/architecture/AR-ci.md#51-pull-requests-and-pushes) compares instruction counts on generated fixtures against the PR base branch. Counts proxy workload cost, not elapsed time; [regression limits are not enforced](docs/architecture/AR-ci.md#52-regression-limits).

`grund` follows its own scheme: [why](docs/grund.md), [goals](docs/goals.md),
[roadmap](docs/roadmap.md), [changelog](docs/changelog.md). This README is under spec
too — [§REQ-readme](docs/requirements/REQ-readme.md#req-readme-the-readme-is-the-grounded-shop-window): excerpts come from this repository or its runnable examples,
whose citations are checked in their own repositories.

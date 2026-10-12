# grund <a href="https://github.com/agent-grounds/grund/actions/workflows/ci.yml"><img align="absmiddle" alt="CI" src="https://github.com/agent-grounds/grund/actions/workflows/ci.yml/badge.svg"></a> <a href="https://crates.io/crates/grund"><img align="absmiddle" alt="crates.io" src="https://img.shields.io/crates/v/grund.svg"></a> <a href="LICENSE"><img align="absmiddle" alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue.svg"></a>

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
Trim surrounding whitespace.
</pre>
</td>
<td valign="top" width="400">
<b>2 · Cite</b><br>
Cite it from the code in <a href="examples/quickstart/repo/src/login.py"><code>src/login.py</code></a>
<pre lang="python">
def clean_username(username):
    """&#167;FS-login.format"""
    return username.strip()
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
$ grund FS-login.format
&#35;&#35; format: Username format
Trim surrounding whitespace.
</pre>
</td>
<td valign="top" width="400">
<b>4 · Check</b><br>
Misspell <code>format</code> in the citation, and CI fails
<pre lang="console">
$ grund check
… section not found: FS-login.fromat; …&#10;&#10;
</pre>
</td>
</tr>
</tbody>
</table>
<!-- grund:fmt on -->

`grund` gives specs stable IDs, lets code cite them, and checks those citations in CI.
Agents retrieve the section they need before editing ([§GRUND-grund](docs/grund.md#grund-grund-agents-stay-grounded-in-the-spec)).

- **Structure:** define your kinds of fact, their homes and their IDs in `grund.toml`; retrieve one section at a time ([§GRUND-schema](docs/grund.md#grund-schema-the-shape-of-a-projects-knowledge-is-hard-to-define-and-to-keep)).
- **Checks:** catch broken citations and violations of your declared rules in pre-commit and CI ([§GRUND-links](docs/grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)).

**A valid citation does not prove the code implements the requirement.** Returning
`username` unchanged would still pass `grund check`; tests and review verify behavior.
It does let an agent verify that quickly and cheaply — the requirement is one lookup
away from the code — and keep the repository consistent as both change.
[Lychee](https://lychee.cli.rs/) checks ordinary links; both belong in CI
([§GRUND-links.2](docs/grund.md#2-holding-every-edit-to-them)). The
[requirements-traceability comparison](docs/related-work/REL-traceability-tools.md#workmatrix-reader-tasks)
sets `grund` beside the tools it descends from.

## The default structure

`grund init` lays a project out in layers, each answering one question, and each
layer cites the one that explains it:

|           | Kind        | Home                            | Cites                                  |
|-----------|-------------|---------------------------------|----------------------------------------|
| Why       | GRUND       | `docs/grund.md`                 |                                        |
| Where     | GOAL        | `docs/goals.md`                 | should cite GRUND or GOAL              |
| What      | FS          | `requirements.md`               | should cite GOAL or FS; **never** AR   |
| How       | AR          | `docs/architecture/`            | should cite FS or GOAL                 |
| Proof     | e2e         | `tests/e2e/`                    | **must** cite FS; avoid AR             |
|           | integration | `tests/integration/`            | should cite AR                         |
| Code      | code        | any other file                  | should cite FS or AR                   |
| Decisions | DF          | `docs/decisions/functional/`    | should cite FS or GOAL                 |
|           | DA          | `docs/decisions/architectural/` | should cite AR or FS                   |
| Plan      | RM          | `docs/roadmap.md`               |                                        |

Kinds and homes are what `init` writes to `grund.toml`. The rules are the canonical
ruleset it writes commented out under `[citations]`; uncomment it to enforce them. A
**must** or **never** is a `grund check` error, and a should or avoid is a suggestion
(`grund check --suggestions`). Kinds, homes and rules are yours to change: the
[structure guide](docs/user-facing/structure.md) covers kinds, homes and the ID grammar,
and [citation directions](docs/user-facing/citation-directions.md) the `[citations]`
grammar.

## Quick start

```bash
cargo install grund
```

First, [try the login example](examples/quickstart/): retrieve its
requirement, get a passing check, then break and repair the citation in a temporary copy.

For your own project, run the following **inside its repository** if you are
starting with new specs:

```bash
grund init --docs     # writes config, agent instructions, and doc scaffolds
grund check
```

**Already have specs?** [Map their existing locations](docs/user-facing/setup.md)
before running `grund init`; use `--docs` only when you want scaffolds.
`init` writes your agent's instructions from `grund.toml` — ours is
[`AGENTS.md`](AGENTS.md). Add `grund check` to [pre-commit and CI](docs/user-facing/checking.md#in-pre-commit-and-ci).
[Installation](docs/user-facing/installation.md) covers prebuilt binaries, supported
platforms, the Python and Node APIs, and local candidate builds.

## What an agent reads

Step 3 above retrieves just the cited requirement. For a larger specification,
an agent can choose how much to read. These rounded word
counts are measured on Grund's own `FS-check` specification:

| An agent runs | and reads |
|---|---|
| `grund FS-check` | the lead — **~30 words** |
| `grund FS-check.3.2` | one section — **~140 words** |
| `grund FS-check --toc` | the lead and a map of ~250 sections — **~1,800 words** |
| `grund FS-check --full` | the whole specification — **~32,000 words** |

One rule in an agent's instructions keeps that loop: *when you see `§<ID>`, run
`grund <ID>` and treat what it prints as the authoritative definition.* Every agent
fetches the same bytes for the same ID. [Querying grund](docs/user-facing/querying.md)
has the recipes past this ladder; [coordinate sizes](docs/user-facing/coordinate-sizes.md)
shows how to find and split a lead that grew too heavy.

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
| [Clickable citations](docs/user-facing/clickable-citations.md) | `grund integrations` for your terminal and editor |
| [Editor support](docs/user-facing/lsp.md) | `grund-lsp`: completion, hover, diagnostics, `$$` → `§` |
| [All guides](docs/user-facing/README.md) | each beside its runnable example |

`grund --help` is one screen; `grund <command> --help` is one page with flags, examples,
and exit codes.

The [benchmark report](docs/benchmarks.md) preserves a 2026-05-20 local timing run and a historical instruction-count snapshot, with methodology and build assumptions. [Current CI](docs/architecture/AR-ci.md#51-pull-requests-and-pushes) compares instruction counts on generated fixtures against the PR base branch. Counts proxy workload cost, not elapsed time; [regression limits are not enforced](docs/architecture/AR-ci.md#52-regression-limits).

`grund` follows its own scheme: [why](docs/grund.md), [goals](docs/goals.md),
[roadmap](docs/roadmap.md), [changelog](docs/changelog.md). This README is under spec
too — [§REQ-readme](docs/requirements/REQ-readme.md#req-readme-the-readme-is-the-grounded-shop-window): excerpts come from this repository or its runnable examples,
whose citations are checked in their own repositories.

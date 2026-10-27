# grund

> **Keep your agents grounded** — docs, values, and code as one knowledge graph, checked on every commit.

<!-- grund:fmt off -->
<table>
<tbody>
<tr>
<td valign="top" width="50%">
<b>1 · Declare</b><br>
Write a spec in <a href="examples/quickstart/repo/docs/functional-spec/FS-login.md"><code>docs/functional-spec/</code></a>
<pre lang="markdown">
&#35; FS-login: Login
&#35;&#35; format: Username format
&#35;&#35;&#35; format.1: Surrounding whitespace
Trim surrounding whitespace.
</pre>
</td>
<td valign="top" width="50%">
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
<td valign="top" width="50%">
<b>3 · Read</b><br>
Look it up before you change the code
<pre lang="console">
$ grund FS-login.format.1
&#35;&#35;&#35; format.1: Surrounding whitespace
Trim surrounding whitespace.
</pre>
</td>
<td valign="top" width="50%">
<b>4 · Check</b><br>
Misspell <code>format</code>, and CI fails
<pre lang="console">
$ grund check
… not found: FS-login.fromat.1; …&#10;&#10;
</pre>
</td>
</tr>
</tbody>
</table>
<!-- grund:fmt on -->

<p><a href="https://github.com/agent-grounds/grund/actions/workflows/ci.yml"><img alt="CI" src="https://github.com/agent-grounds/grund/actions/workflows/ci.yml/badge.svg"></a> <a href="https://crates.io/crates/grund"><img alt="crates.io" src="https://img.shields.io/crates/v/grund.svg"></a> <a href="LICENSE"><img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue.svg"></a></p>

## Default structure

`grund init` lays a project out in layers, each citing the one that explains it, under
rules it writes commented out in `grund.toml` until you adopt them:

|           | Kind        | Home                                                    | Cites                                   |
|-----------|-------------|---------------------------------------------------------|-----------------------------------------|
| Why       | GRUND       | `docs/grund.md`                                         |                                         |
| Where     | GOAL        | `docs/goals.md`                                         | should cite GRUND or GOAL               |
| What      | FS          | `docs/functional-spec/`                                 | should cite GOAL or FS; **never** AR    |
|           | REQ         | `docs/requirements/`                                    | should cite GRUND or GOAL; **never** AR |
| How       | AR          | `docs/architecture/`                                    | should cite FS or GOAL                  |
| Proof     | e2e         | `tests/e2e/`                                            | **must** cite FS; avoid AR              |
|           | integration | `tests/integration/`                                    | should cite AR                          |
| Code      | code        | any other file                                          | should cite FS or AR                    |
| Decisions | DF          | `docs/decisions/functional/`                            | should cite FS or GOAL                  |
|           | DA          | `docs/decisions/architectural/`                         | should cite AR or FS                    |
| Plan      | RM          | `docs/roadmap.md`                                       |                                         |

Every rule `grund check` holds a project to, what each one means underneath, and the
finding it reports is listed in
[Schema and links](docs/user-facing/structure.md#schema-and-links-the-rules-a-repository-is-held-to).
`grund schema` prints the ones a repository is held to on one screen, read from its
`grund.toml` alone; [`examples/schema/`](examples/schema/) shows its table and Mermaid output.

## Agent instructions

`init` also writes the structure into your agent's instructions, a managed block in
`AGENTS.md` that `grund check` keeps current. The number in its heading is the
block's revision, not a grund release
([the setup guide](docs/user-facing/init.md#the-blocks-revision-is-not-grunds-release)
says how the two differ). For example, grund's own
[`AGENTS.md`](AGENTS.md) tells an agent how to read a citation:

> A `§<ID>` is a pointer to a fact, not a file path. Resolve it with `grund` and climb only as far as needed:
>
> - `grund <ID>` — the lead, cut at the first child section: heading-less for a whole declaration, while `grund <ID>.<section>` opens with that section's heading line. The cheap first read for a bare `§<ID>` citation.
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
> - **Inline citation style.** Inline notes: ≤ 1 line preferred, hard cap 3 lines; ≤ 100 columns. A note is one comment block: a blank line splits it, an empty comment line does not. Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell, SQL or Lisp comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there.
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
grund init --scaffold # config, agent instructions and starter files
grund check
```

**Already have specs?** Hand the setup to your agent:

```bash
grund guide setup   # the adoption guide, for your agent
```

Your agent inspects the repository, recommends settings for the specs you have,
confirms each one with you, then writes `grund.toml`, runs `grund init` and validates.
The command prints the core of the guide; the agent fetches each further section
it needs with `--section`.
To set up by hand instead, see [setup](docs/user-facing/setup.md).
The runnable [existing-structure adoption example](examples/init-template-none/)
starts with `grund init --template=none`, asks for the first kind's offline help,
then declares the repository's own kinds.

Then:

- add `grund check` to [pre-commit and CI](docs/user-facing/checking.md#in-pre-commit-and-ci);
- look up any key in the [`grund.toml` reference](docs/user-facing/configuration.md), and point your editor at its [JSON Schema](schemas/grund.schema.json);
- `grund.toml` has two versions: version 2 (`grund_config_version = 2`) is what `grund init` writes, holding only what differs from a default, and `grund config show` prints every effective setting; a file without a version is version 1 and keeps loading unchanged. Version 1 reads the extensions `[scan]` lists, and version 2 names the languages it reads, maps suffixes to them, and lets only doc comments declare — see [sources](docs/user-facing/sources.md) and its [runnable example](examples/sources/);
- for prebuilt binaries and the Python and Node APIs, see [installation](docs/user-facing/installation.md).

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

A section heading may write only the component it adds: under `## format: Formatting`,
`### .1: Username` declares `FS-login.format.1`, and every citation, `refs` line and
`--toc` shows that full path, as does the first line of a read of the section itself.
Moving the block under another parent changes its address, so after citing a relative
section run `grund fmt --write` with no path at the workspace root, or at the project
root outside a workspace, and commit the section lock it writes,
`.grund/sections/FS-login.lock`; `check` then says where a cited text went. See
[section headings](docs/user-facing/structure.md#section-headings) and
[`examples/relative-sections/`](examples/relative-sections/). For declaration renames,
follow the [maintenance workflow](docs/user-facing/structure.md#maintaining-declarations)
and try the copy-based exercise in
[`examples/relative-sections-lock/`](examples/relative-sections-lock/). `fmt`'s preview
names every kind of rewrite a line gets, its `--write` summary totals them, and a lock
file the rename deletes or creates says so; read both in
[`examples/fmt-reporting/`](examples/fmt-reporting/).

A declaration can also live in code: a doc-comment that opens with the ID declares it,
and a link from the kind's index enrolls it with no stub file. Grund's own `AR-checker` is declared only on the
function it describes, in [`crates/grund-core/src/checker/report.rs`](crates/grund-core/src/checker/report.rs):

```rust
/// AR-checker: how grund validates the scanner's findings
///
/// …
/// ## placement: Where the checker sits
/// …
pub(crate) fn check_on_disk(
```

`grund AR-checker` prints that prose without the `///` markers.

## Citation format

```plaintext
     ┌─────────────────── citation ───────────────────┐
            ┌───────────── ID ───────────────┐
  [§] [alias /] KIND - [number -] slug [.section]
   │     │        │       │         │       │
   │     │        │       │         │       └─ section: .3, .3.1, .format
   │     │        │       │         └───────── slug: [a-z0-9][a-z0-9-]*
   │     │        │       └─────────────────── optional ordinal (e.g., 001)
   │     │        └─────────────────────────── kind: FS, AR, … or your own
   │     └──────────────────────────────────── alias: root, api, …
   └────────────────────────────────────────── citation marker
```

`[id] format` picks one of three ID schemes, and a kind may override it:

| Scheme | Example | Trade-off |
|---|---|---|
| `{kind}-{number}-{slug}` *(default)* | `FS-014-user-login` | the number stays stable; allocate it with `grund id` |
| `{kind}-{number}` | `FS-014` | the most stable, but says nothing until resolved |
| `{kind}-{slug}` *(grund's own)* | `FS-user-login` | reads like English; renaming a slug rewrites its citations |

The [structure guide](docs/user-facing/structure.md) has the rest of the syntax, and a
runnable example of each scheme. A link you write around a citation, in any form
Markdown has, stays yours: `grund fmt` leaves it as written and `grund check` still
checks the citation in it, as
[`examples/authored-citations/`](examples/authored-citations/) shows. It also
explains why an index-only declaration warns while `refs` still lists its entry.

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

`grund check` at the root checks every member, each in its own namespace;
[repository options](docs/user-facing/repository-options.md#workspaces-and-sub-projects) has the details.
A member a checkout may lack, such as a private package CI does not get, goes
under `optional_members`; in a checkout without it, `grund check` does not judge
the managed blocks' member lists and `grund init` keeps them as they are, as the
[optional-member example](examples/workspace-optional-member/) shows.

## External facts

A GitHub issue, or a Jira or Linear ticket, is cited like any spec once a kind names the
executable that fetches it. `grund fetch <ID>` saves it as a declaration in the
repository; every other command reads that committed snapshot offline. From the [external-tickets example](examples/external-tickets/):

```toml
[[kinds]]
kind = "GH"
file = "docs/tickets.md"
format = "{kind}-{number}"
resolve = "should"
fetch = "scripts/fetch-ticket"
```

```console
$ grund check
… no snapshot for GH-1234 in docs/tickets.md — run grund fetch GH-1234
$ grund fetch GH-1234
$ grund GH-1234
The issue requests committed snapshots that resolve offline.
```

The [external facts guide](docs/user-facing/external-facts.md) has the fetch contract
and a GitHub integration to start from.

## Values

A number a spec declares can be checked wherever it is used. With `values = true` on a
kind, a declaration's numbered headings are its components, and a backticked literal
beside a citation must equal them. From the [values example](examples/values/), a physics simulation's time step:

```markdown
# CONST-time-step: Integration time step
## 1. 0.001
## 2. s
```

```markdown
Each solver step advances time by `1.0e-3 s` (§CONST-time-step).
```

A use that still quotes the step from before it was tuned fails the check, naming the
root the line cites and the literal it wrote:

```console
$ grund check
… whole binding: value mismatch for CONST-time-step (component 1): bound `0.002 s`, declared `0.001 s` …
```

The citation is right and the literal is stale, so the fix is `0.001 s`, cited as before.
Until grund 0.22.0 the line opens with the older component-only message, and from then
it is the root's alone.

The [values guide](docs/user-facing/values.md) covers JSON and YAML declarations, values inside
prose specs, and bindings in code comments.

## Chapter rules

A convention can be written as one sentence and checked. With `rules = true` on a kind,
each declaration's title is a rule in a small controlled English and its body says why.
From the [rules example](examples/rules/):

```markdown
# RULE-requirements: The requirements chapter of each FS must cite at least one REQ.

The requirement chapter carries the evidence, because §GOAL-rules makes conventions executable.
```

A rule can count chapters as well as citations: `RULE-security`, in the same example,
says `FS-demo must have exactly one security chapter.` A spec with no such chapter, which
cites its requirement from another chapter, fails both:

```console
$ grund check
…
docs/fs/FS-demo.md:1: error: FS-demo has 0 security chapters; RULE-security requires exactly one; …
…
docs/fs/FS-demo.md:6: error: FS-demo.requirements must cite REQ (RULE-requirements); add such a citation within it
```

A sentence that does not parse is refused rather than guessed at, offering the one that
would where it can tell, so a reader and `grund check` never give a rule two meanings.
Try one before you commit it:

```console
$ grund check --rule "Each FS may not cite any AR." --only-rule
error: modality "may not" is not accepted; accepted form: Each FS must not cite any AR.
```

The same example's `RULE-overview-ceiling` uses `cite each AR at most once`:
zero or one citation to a target passes; two physical sites to the same target fail.
Per-target `at most 1 times` is refused with an `at most once` rewrite.

`grund init` repeats every valid rule under `### Chapter rules` in the agent
instructions, so an agent reads the sentence `check` enforces. The
[chapter rules guide](docs/user-facing/rules.md) lists every accepted sentence, the
finding each one reports, and every refusal.

## Make citations clickable

Turn a `§<ID>` in your terminal into something you click, landing at the exact line it cites:

```bash
grund integrations                  # what applies in this environment
grund integrations wezterm          # review what it installs
grund integrations wezterm --write  # install it
```

Supported clients are `codium`, `iterm2`, `kitty`, `tmux`, `vscode`, and `wezterm`. `--write` is a one-time, idempotent user setup — the integration, the `grund-open` resolver, and a global instruction block for whichever agents you have installed. It changes no repository.

**`~/.local/bin` must be on your `PATH`** — that is where the resolver is installed, and it is not there by default on macOS, where a missing `PATH` entry makes every click silently do nothing.

**[Clickable citations](docs/user-facing/clickable-citations.md)** is the full setup guide: the per-client reload each one needs, the manual step WezTerm and iTerm2 require, how to check it works, what to do when a click does nothing, and how to control the citations agents write in conversations.

## Editor support via [LSP](https://microsoft.github.io/language-server-protocol/)

Install the optional language server separately when you want citation completion, editor diagnostics, hover previews, usage counts on declaration titles, definition jumps, document links, references, and live `$$` → `§` formatting:

```bash
cargo install grund-lsp
```

The server speaks LSP over stdio and has no daemon or socket. For IntelliJ
family IDEs, `grund-lsp integrations lsp4ij --write <directory>` generates the
LSP4IJ import template carried by the installed binary.

Type `§` or `$$` and it completes the declared IDs; see
[authoring and client bindings](docs/user-facing/lsp.md#write-a-citation).
The [setup guide](docs/user-facing/lsp.md) has the complete import and
verification flow plus snippets for VSCode, Vim/Neovim, Emacs, Helix, Zed, and
Sublime Text. Put reusable client config in your editor's **user (global)
settings**, so the configuration is available across projects rather than only
in repos that ship an editor config. In VSCode / VSCodium, user settings do not
grant Workspace Trust; see the [guide's trust prerequisite](docs/user-facing/lsp.md#vscode--vscodium)
for the affected clients.

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/lsp/hover-preview.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/assets/lsp/hover-preview-light.png">
    <img src="docs/assets/lsp/hover-preview-light.png" width="96%" alt="Hover preview of the misplaced-declaration spec point shown directly above the grund code that implements the misplaced-declaration check">
  </picture>
  <br><strong>Hover previews</strong> — the spec and the code that satisfies it, in one frame.
</p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/lsp/error-reporting.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/assets/lsp/error-reporting-light.png">
    <img src="docs/assets/lsp/error-reporting-light.png" width="96%" alt="Dangling citation diagnostic showing the nearest declared ID hint">
  </picture>
  <br><strong>Error reporting</strong> — dangling citations are flagged inline with a "did you mean" hint.
</p>

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/assets/lsp/definitions.png">
    <source media="(prefers-color-scheme: light)" srcset="docs/assets/lsp/definitions-light.png">
    <img src="docs/assets/lsp/definitions-light.png" width="96%" alt="Go-to-definition results listing FS-check.3.7 declaration and citation targets">
  </picture>
  <br><strong>Definitions &amp; references</strong> — jump between a declaration and every §citation of it.
</p>

## Go deeper

| Guide | For |
|---|---|
| [Checking a repository](docs/user-facing/checking.md) | what `grund check` enforces, its report, `--watch`, `--only`, pre-commit |
| [`grund.toml` reference](docs/user-facing/configuration.md) | every key of both config versions, and the JSON Schema for your editor |
| [Config explain](docs/user-facing/explain.md) | what a key or a check code means and what clears a finding, from the grund you run, in your config's version — [runnable example](examples/config-explain/) |
| [Migrating to version 2](docs/user-facing/migrating.md) | preview, repairs and preserving listed comments — [runnable example](examples/migrate/) |
| [Structure and IDs](docs/user-facing/structure.md) | kinds, ID schemes, citation syntax, inline declarations, front matter — [runnable example](examples/front-matter/) |
| [Querying grund](docs/user-facing/querying.md) | selectors, batch reads, `refs`, `cover` — [runnable example](examples/refs-unused/) of `refs` on what nothing cites |
| [Reviewing code](docs/user-facing/reviewing.md) | a citation's blast radius, diff recipes |
| [Chapter rules](docs/user-facing/rules.md) | checked rules in controlled English — [runnable example](examples/rules/) |
| [Sources](docs/user-facing/sources.md) | which files a version-2 project reads, and which comments declare — [runnable example](examples/sources/) |
| [Fields](docs/user-facing/fields.md) | a version-2 kind's chapters as fields, their order, titles and values checked, and front matter as a field — [runnable example](examples/fields/), [front matter](examples/front-matter/) |
| [Values](docs/user-facing/values.md) | one value, declared once, checked wherever it is quoted |
| [External facts](docs/user-facing/external-facts.md) | cite tickets as committed offline snapshots |
| [Workspaces](docs/user-facing/repository-options.md#workspaces-and-sub-projects) | citations across sub-projects |
| [Related work](docs/related-work/README.md) | where `grund` stands among academic work and open tools |
| [All guides](docs/user-facing/README.md) | each beside its runnable example |

`grund --help` is one screen; `grund <command> --help` is one page with flags, examples,
and exit codes.

`grund` follows its own scheme: [why](docs/grund.md), [goals](docs/goals.md),
[roadmap](docs/roadmap.md), [changelog](docs/changelog.md). This README is under spec
too — [§REQ-readme](docs/requirements/REQ-readme.md#req-readme-the-readme-is-the-grounded-shop-window): excerpts come from this repository or its runnable examples,
whose citations are checked in their own repositories.

# grund

[![CI](https://github.com/agent-grounds/grund/actions/workflows/ci.yml/badge.svg)](https://github.com/agent-grounds/grund/actions/workflows/ci.yml) [![crates.io](https://img.shields.io/crates/v/grund.svg)](https://crates.io/crates/grund) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> **Keep your agents grounded** — specs, docs, and code as one knowledge graph, always in sync.

`grund` exists so you always know *why* — why your agents did what they did, why a line is the way it is: all work stays grounded in the spec that called for it ([§GRUND-grund](docs/grund.md#grund-grund-agents-stay-grounded-in-the-spec)). Keeping the why means keeping a structure, and `grund` takes on the two parts of it that are hard:

- **The shape of what your project knows** — its kinds of fact, where each lives, and how it is sectioned — declared once in `grund.toml`, with every fact at a stable ID that is fetched on demand in minimal tokens instead of re-read from whole files ([§GRUND-schema](docs/grund.md#grund-schema-the-shape-of-a-projects-knowledge-is-hard-to-define-and-to-keep)).
- **The links between code and text, and the rules they follow** — every unit of code cites the spec point that says why it is the way it is, and `grund check` fails the build the moment a link dangles or breaks a rule the project declared ([§GRUND-links](docs/grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)).

`grund` is built around one workflow:

0. **Specify your intent.** Declare the goal, spec, or decision as a `# <ID>: …` heading before any code or doc cites it.
1. **Cite as you write.** Every code unit carries a `§<ID>` back to the spec section it implements (`§<KIND>-<slug>[.section]` — full grammar in [section 4](#4-the-structure-that-gets-cited)).
2. **Re-read before you edit.** `grund <ID>.<section>` pulls just that subsection into context — no full-file reads, no token bloat.
3. **No dangling pointers.** `grund check` validates that every cited ID resolves — in `.md`, Rust `///`, Java doc-comments, Python docstrings, Go `//`, JSDoc, every doc-comment form `grund` knows about.

[Lychee](https://lychee.cli.rs/) checks links in Markdown and HTML. A `§`-marked citation of `FS-check.3.2` in `crates/grund-core/src/checker/references.rs` needs an ID resolver rather than ordinary link validation. `grund` resolves those citations and checks declared constraints: Lychee is the link checker; `grund` is the intent checker. Both belong in CI; they guard different failure modes. [§GRUND-links.2](docs/grund.md#2-holding-every-edit-to-them)

See the [requirements-traceability comparison](docs/related-work/REL-traceability-tools.md#workmatrix-reader-tasks) for concrete retrieval and checking tasks and their limits.

The [benchmark report](docs/benchmarks.md) preserves a 2026-05-20 local timing run and a historical instruction-count snapshot, with methodology and build assumptions. [Current CI](docs/architecture/AR-ci.md#51-pull-requests-and-pushes) compares instruction counts on generated fixtures against the PR base branch. Counts proxy workload cost, not elapsed time; [regression limits are not enforced](docs/architecture/AR-ci.md#52-regression-limits).

## 0. Specify your intent

Before anything can be cited, the target has to exist. A declaration is a heading whose first token is the ID — `grund`'s own reason for being lives at [`docs/grund.md`](docs/grund.md):

```markdown
# GRUND-grund: agents stay grounded in the spec

Keep agents grounded in the spec — fewer bugs, cheaper LLM context,
faster onboarding. …
```

That heading lives in the configured home for its kind (`GRUND` → `docs/grund.md`, `FS` → `requirements.md`, `GOAL` → `docs/goals.md`, and so on — see [section 4](#4-the-structure-that-gets-cited)). Once it's declared, any code, doc, or test can cite `§GRUND-grund` and `grund check` will resolve it. A declaration can live in code too: drop the `#` in a doc-comment — `grund`'s own architecture spec [`AR-checker`](crates/grund-core/src/checker/report.rs) opens with `/// AR-checker: how grund validates the scanner's findings`, right on the code it describes ([section 4](#4-the-structure-that-gets-cited) shows the wiring).

## 1. Cite as you write

When code realizes a named behavior, it carries a `§<ID>` citation — on its doc-comment for a whole behavior, or inline beside the line that enforces one clause. From `grund`'s own source — the code implementing the missing-section check is grounded in [`FS-check.3.2`](docs/functional-spec/FS-check.md#32-missing-section), the spec section that defines that very check:

```rust
// crates/grund-core/src/checker/references.rs

/// The reference-resolution rule family — dangling citations (§FS-check.3.1),
/// missing sections (§FS-check.3.2), unknown project aliases (§FS-check.3.8), …

    // …
    // §FS-check.3.2: the ID resolves but no declaration has a heading at the
    // cited section path.
    if let Some(sec) = &cite.section {
        let any_match = decls.iter().any(|d| d.sections.contains_key(sec));
```

`grund` doesn't invent these citations — that's the contributor's call. What `grund` does is make sure the ones you wrote *resolve*. With `require_grounding = true` — in `[reference]` for every place at once, or on one `[[kinds]]` row for that place alone, at a `grounding_level` from the whole file down to every `##` of it ([§FS-config.3.4.8](docs/functional-spec/FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) — it also fails what carries no resolving citation; the stronger file-level "implementation changed with its spec and test, or bounded waivers" recipe is tracked separately in [§RM-cochange-gate](docs/roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits).

Store section citations with their full ID. For example, this input inside an
`FS-check` declaration body contains a live local section citation:

```text
See §2.1.
```

When section 2.1 exists, the citation remains navigable, but `grund check` reports:

```text
local section citation §2.1; write §FS-check.2.1 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`
```

Every form of the finding ends by naming the two releases its verdict moved between, and
an owned site the formatter would actually write also names the command that clears it
([§FS-check.3.24.1](docs/functional-spec/FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)) — so one line separates *this tree predates the binary running over
it* from *this citation is wrong*. `grund fmt --write` expands safe owned sites;
protected sites need manual replacement and are not offered the command.
Where the owner lacks the section, the finding names the absence and offers a full citation or the escape instead ([§FS-check.3.24.3](docs/functional-spec/FS-check.md#3243-an-absent-target-section-is-answered-with-the-escape)), the ordinary missing-section error
fires beside it, and the formatter leaves that site alone and does not offer the command ([§FS-fmt.2.4.6](docs/functional-spec/FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)). A site outside
a declaration is diagnosed without a guessed target and needs a full citation or an
escape: `<§>2.1` is an inert illustration, with no citation diagnostic or navigation.
This is intentionally newly loud compatibility behavior for a
form that older releases silently skipped ([§FS-check.3.24](docs/functional-spec/FS-check.md#324-declaration-local-section-citation), [§FS-fmt.2.4](docs/functional-spec/FS-fmt.md#24-shorthand-to-canonical)).

## 2. Re-read before you edit

A citation is a pointer to a fact, not a file path. Resolve it without opening files:

```bash
$ grund FS-check.3.2
### 3.2 Missing section

A citation with a section suffix (`§FS-<user-login>.3.1` or, in an opted-in repository, `§FS-<user-login>.goals`) where the declaration exists but the requested section heading does not. [… remaining lead output elided …]
```

`grund <ID>` returns *just* the useful slice — the lead prose for one section, cut at the first child section — so the agent pulls one fact into context instead of an entire file. Use `grund list --size=words` to measure how much prose a given slice contains. Wrappers flatten outside fenced examples. Its ladder:

- `grund <ID>` — the lead prose, cut at the first child section; the cheap default for a bare citation
- `grund <ID> --toc` — the lead plus the section map, for choosing the next subsection
- `grund <ID> --brief` — heading plus first paragraph only, for hover-sized previews
- `grund <ID> --full` — the full declaration body when the narrower reads are not enough
- `grund <ID> --format json` — for tooling; it carries the heading `anchor`, so a web link to the point is one read ([recipe](docs/user-facing/querying.md#link-a-citation-from-one-read))
- `--path-base=invocation` — on any read or check, spells the paths it reports relative to where you ran it rather than to the project root, without committing `[output] relative_paths = false` ([§FS-cli.3.4](docs/functional-spec/FS-cli.md#34---path-base--where-report-paths-are-spelled-from))

For many reads, the explicit batch form accepts ordered NDJSON and reuses one workspace scan; `--all` discovers every declaration and section from that same loaded catalog ([§FS-show.2.6](docs/functional-spec/FS-show.md#26-batch-resolution)):

```bash
printf '%s\n' '{"id":"FS-check"}' '{"id":"FS-check","section":"3.2"}' \
  | grund show --batch --format=json
grund show --batch --all --format=json
```

`grund refs <ID> --total` sizes the blast radius in one line — `cited at 140 sites across 50 files` — and `grund refs <ID> --summary` breaks that down one file per line before a full citation dump, while `grund list --kind FS,AR` keeps discovery scoped. When a specification feels heavy, `grund list --size=words --top 10` finds the largest leads before you read them in full. That's the "cheap grounding" half of the workflow: every agent fetches the same bytes for the same ID, every time.

Repositories can opt into a warning at their own measured boundary:

```toml
[reference]
lead_size_warning = { max = 600, unit = "words" }
```

An over-budget lead should keep its grounding: move detail into numbered child sections, or promote a child section to its own ID after checking its callers with `grund refs <ID> --summary`. See the [coordinate-size guide](docs/user-facing/coordinate-sizes.md) for counting rules, output fields, duplicate handling, and workspace scope ([§FS-list.3.4](docs/functional-spec/FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements), [§FS-declarations.checks.oversized-lead](docs/functional-spec/FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)).

For scripts, exit `0` is a completed `refs` answer even when it is empty. Exit
`1` means the selected repository grammar rejected the ID or its number-only
shorthand was ambiguous; route that status to ID repair, and reserve exit `2`
for setup, configuration, I/O, or incomplete-scan failure
([§FS-refs.4](docs/functional-spec/FS-refs.md#4-exit-codes)).

## 3. Check for dangling pointers

Renumber the heading `### 3.2 Missing section` in [`FS-check.md`](docs/functional-spec/FS-check.md) and `grund check` flags every site that leaned on it — code and decision docs alike, in one resolver:

```
$ grund check
crates/grund-cli/tests/index_entry_round_trip.rs:229: error: missing section FS-check.3.2
crates/grund-core/src/checker/index.rs:152: error: missing section FS-check.3.2
crates/grund-core/src/checker/index.rs:258: error: missing section FS-check.3.2
crates/grund-core/src/checker/references.rs:2: error: missing section FS-check.3.2
crates/grund-core/src/checker/references.rs:378: error: missing section FS-check.3.2
crates/grund-core/src/checker/report.rs:85: error: missing section FS-check.3.2
crates/grund-core/src/checker/report.rs:486: error: missing section FS-check.3.2
docs/decisions/functional/DF-duplicate-section-path.md:26: error: missing section FS-check.3.2
docs/decisions/functional/DF-require-grounding.md:8: error: missing section FS-check.3.2
docs/requirements/REQ-no-wrong-citation.md:7: error: missing section FS-check.3.2
```

`grund check <path>` reports on `<path>` while resolving citations against the whole project; with no path it scans the canonical layout (`requirements.md`, `docs/`, `e2e/`, `src/`). In the scanned tree it enforces:

1. Every cited ID resolves to a declaration. *(dangling references)*
2. Every section coordinate (`.3.1`) resolves to a heading inside the declaration. *(missing sections)*
3. No ID is declared in two places. *(duplicates)*
4. Every deeper ATX heading inside a scanned Markdown declaration body is another declaration or carries a numeric or enabled named section coordinate. Fences, file titles, body-closing headings, source doc-comments, setext text, and bold labels are exempt. *(unmarked headings)*
5. Every stub heading `# <ID>: [<text>](<path>)` points at a file containing the inline declaration. *(broken stubs)*
6. The `AGENTS.md` / `CLAUDE.md` entry-point block is up to date. *(stale init)*
7. Declared-but-uncited IDs are flagged. *(unused — warning, not error; a configured `E2E` kind's cases are exempt)*
8. *(opt-in)* With `require_grounding = true`: every source file — or every file of one configured place, down to every `##` section or doc-comment block of it — carries at least one citation. *(ungrounded source file)*
9. *(workspace)* Alias-qualified citations resolve across configured sub-projects. *(cross-project references — see [§FS-workspace](docs/functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace))*

`grund check` reads what `[scan] include` names, so a citation in a directory the config never mentioned is invisible rather than merely unchecked — it neither resolves nor dangles. `grund check --full` ([§FS-check.1.3](docs/functional-spec/FS-check.md#13-the-full-tree-scope---full)) walks the whole repository past that key and reports the references that resolve to nothing out there, and only those: a directory nobody configured is never judged against conventions it never adopted. It is purely additive, so it can only turn a green run red.

A passing text check prints `success` and exits 0. Findings go to stdout as
`<path>:<line>: error: <message>`, `warning:`, or opt-in `suggestion:` lines:
errors come first, then warnings and suggestions, while the location remains the
jump-friendly prefix. `grund check | …` / `grund check --format=json | jq` work
without redirection (the linter convention — only run-level `error:` lines, like
an unreadable path, go to stderr). JSON output remains diagnostics-only and in
global location order, so a clean `grund check --format=json` prints nothing.

`grund check --watch` checks immediately and updates the terminal after saves;
Ctrl-C restores an owned alternate screen and returns the last completed status.
Redirected output appends ordinary reports, and clean JSON runs have no visible
boundary. See the [watch guide](docs/user-facing/watch.md)
([§FS-check.6](docs/functional-spec/FS-check.md#6-watch-mode---watch)).
Exact-text consumers migrating from the former unmarked, global-location report
should use `--format=json`, whose bytes, object shape, and order are unchanged.

From this checkout, the initial report is:

```text
$ cargo run --quiet -- check --watch --format=text
success
```

Watch stays resident until Ctrl-C; this capture has empty stderr and exits 0 after that initial report.

When you need a narrower answer without weakening the repository's default check,
select its stable finding codes: `grund check --ignore agents-init` asks whether
the remaining content report has errors, while repeatable `--only <code>` and
`--ignore <code>` compose as sets, and `--only-rule` narrows the same report by
rule authority instead — only what a `--rule` trial sentence authored, which is
what lets you try a sentence out for the cost of its own findings
([§FS-check.1](docs/functional-spec/FS-check.md#1-inputs), [§FS-rules.8](docs/functional-spec/FS-rules.md#8-command-surfaces)).
A check narrowed to a code rules produce still says when a rule behind it was
skipped as invalid ([§FS-rules.7.6](docs/functional-spec/FS-rules.md#76-selection-json-ordering-and-exits)).
Selection happens only after the complete scan, and operational failures remain
visible; selected `success` describes only that view, not an all-findings
repository verdict ([§FS-check.2](docs/functional-spec/FS-check.md#2-outputs)).

`grund` does **not** check Markdown links, URLs, spelling, or grammar. Use [`lychee`](https://github.com/lycheeverse/lychee), `vale`, etc. for those.

- [Workspaces and sub-projects](docs/user-facing/repository-options.md#workspaces-and-sub-projects)
- [Keep shared values consistent](docs/user-facing/repository-options.md#keep-shared-values-consistent)
- [Cite external facts offline](docs/user-facing/repository-options.md#cite-external-facts-without-making-checks-depend-on-the-network)
- [Write the config in version 2](docs/user-facing/repository-options.md#write-the-config-in-version-2)

## 4. The structure that gets cited

See the [structure and ID grammar guide](docs/user-facing/structure.md) for kinds, schemes, citation syntax and inline declarations.

## 5. Reviewing code

See the [review guide](docs/user-facing/reviewing.md) for citation blast radius and diff recipes.

## Install

```bash
cargo install grund
```

That installs the `grund` binary from the [`grund` crate on crates.io](https://crates.io/crates/grund) onto your `PATH`. The [Python API](docs/user-facing/python-api.md) installs locally with `python -m pip install .`; [its example](examples/python-api/) shows `import grund`. The [Node Promise API](docs/user-facing/node-api.md) is locally buildable as an unpublished API-only rehearsal ([§FS-distribution.3.2.4](docs/functional-spec/FS-distribution.md#324-acceptance-evidence)). The npm and PyPI packages are built and rehearsed as a local candidate, and nothing is published to npm or PyPI ([§FS-distribution.4.13](docs/functional-spec/FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)). [Installation](docs/user-facing/installation.md) lists what installs today, the supported platforms, and how to build and install a local candidate.

This README is itself under spec: [§REQ-readme](docs/requirements/REQ-readme.md#req-readme-the-readme-is-the-grounded-shop-window) — every example above is captured from this repository, and the citations here are checked by `grund check` like any other scanned file's.

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

## 🧑‍💻 Editor Support via [LSP](https://microsoft.github.io/language-server-protocol/)

Install the optional language server separately when you want citation completion, editor diagnostics, hover previews, usage counts on declaration titles, definition jumps, document links, references, and live `$$` → `§` formatting:

```bash
cargo install grund-lsp
```

The server speaks LSP over stdio and has no daemon or socket. For IntelliJ
family IDEs, `grund-lsp integrations lsp4ij --write <directory>` generates the
LSP4IJ import template carried by the installed binary
([§FS-lsp.2.4](docs/functional-spec/FS-lsp.md#24-installed-editor-integrations)).
The `integrations` subcommand is not included in published `grund-lsp` 0.13.1;
until the next release, install the workspace crate from source as described in
the [LSP setup guide](docs/user-facing/lsp.md).

The source version also completes declared citations: in a Markdown note here,
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

See the [LSP screenshots](docs/user-facing/lsp.md#screenshots) for hover previews, diagnostics, and navigation.

## Set up a repo

```bash
grund init           # writes AGENTS.md and grund.toml in the cwd
grund init --docs    # also scaffolds docs/ and tests/ trees
grund init --check   # writes nothing; exits 1 if anything is still pending
```

`init` is non-interactive and idempotent: re-running never errors on existing files. With `--docs`, instructional ID shapes follow the repository `[id].format` and any illustrated kind's `[[kinds]].format` override ([§FS-config.3.2](docs/functional-spec/FS-config.md#32-id--id-grammar)). For an existing repo with specs, map those homes in `[[kinds]]` before `grund init`, or run `grund agent-setup-instructions` for the packaged adoption workflow and decision table ([§DF-skill-init-existing-specs](docs/decisions/functional/DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding)). It also checks *where* it was pointed before writing anything: a target no `.git`, `.hg`, `.jj`, or `.svn` marker covers is refused unless you pass `--no-vcs` — use it to scaffold a directory before `git init` — and the home directory and the machine-global agent instruction files are refused outright. `--check` is the `--dry-run` preview taken as a verdict — same report, nothing written, exit `1` when a file is still pending — so a hook can fail on a managed block that drifted in its text while its version heading stayed current, which `grund check` does not see. See [`FS-init`](docs/functional-spec/FS-init.md) for the full state table.

The generated project name comes from `--name` when supplied, then from the target's existing `project_name`, and otherwise from the target directory name. This keeps the canonical `AGENTS.md` heading stable when `grund init --force` regenerates it; pass `--name` only when that run should override the configured identity ([§FS-init.2.3.8](docs/functional-spec/FS-init.md#238-substituted-content)).

## Pre-commit

This repo ships a ready-to-install [.pre-commit-config.yaml](.pre-commit-config.yaml) — `grund check` for citations, `grund init --check` for a stale managed block, `lychee` for Markdown links. For optional staged commit-msg/whole-PR evidence of related spec and test edits with reason-bearing trailers, see the [co-change recipe](docs/user-facing/cochange.md); Grund runs its demonstrations without enabling that contribution gate ([§FS-cochange-recipe.examples](docs/functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance)):

```bash
pip install pre-commit && cargo install lychee && pre-commit install
```

## Commands

`grund --help` is one screen; `grund <command> --help` is one page with flags, examples, and exit codes. The full surface is in [`docs/functional-spec/`](docs/functional-spec/).

- **`grund check`** — validate every reference in the tree.
- **`grund <ID>[.<section>]`** — print one declaration body, for pulling spec content into agent prompts.
- **`grund list`** — the ID catalog.
- **`grund refs <ID>`** — list every citation of a declaration.
- **`grund cover`** — group the citation graph by file, for git-diff recipes; with `--lines`, say which declaration and section own a range of one file, so a diff hunk maps to the spec points it edits.
- **`grund fmt`** — normalize citation syntax (`$$` → `§`, optional Markdown link wrapping).
- **`grund fetch <ID>`** — explicitly materialize one configured external snapshot.
- **`grund id <KIND> "<title>"`** — emit the next conflict-free ID for a new declaration.
- **`grund init`** — scaffold `AGENTS.md` and `grund.toml`.
- **`grund config`** — validate or print the effective `grund.toml`.
- **`grund completions`** — print bash, zsh, or fish completion scripts.
- **`grund agent-setup-instructions`** — print the guided setup workflow for AI agents.

## Agent prompt pattern

The grounding loop, distilled to one rule for an AI agent's system prompt:

> When you see `§<ID>` or `§<ID>.<section>` in any file you are reading, run `grund <ID>[.<section>]` and treat the output as the authoritative definition. Do not paraphrase or guess — quote what `show` returned, or cite the ID and move on.

That rule plus a clean `grund check` is the whole contract: every reference resolves, except that a missing `should` snapshot is reported as a non-blocking warning, and every agent fetches the same bytes for the same ID.

## Project layout

`grund` follows its own scheme. Start at [`AGENTS.md`](AGENTS.md), then read down through [`docs/`](docs/):

- [`docs/user-facing/`](docs/user-facing/) — every user guide beside its runnable example; `grund --help` links it
- [`docs/user-facing/clickable-citations.md`](docs/user-facing/clickable-citations.md) — make citations clickable in your terminal
- [`docs/user-facing/external-facts.md`](docs/user-facing/external-facts.md) — materialize external tickets as committed offline snapshots
- [`docs/user-facing/coordinate-sizes.md`](docs/user-facing/coordinate-sizes.md) — measure coordinate leads and opt into oversized-lead warnings
- [`docs/user-facing/rules.md`](docs/user-facing/rules.md) — write checked chapter and citation rules in controlled English; [runnable example](examples/rules/)
- [`docs/user-facing/values.md`](docs/user-facing/values.md) — declare and check shared values in Markdown, JSON, prose, and code comments
- [`docs/grund.md`](docs/grund.md) — why this exists
- [`docs/goals.md`](docs/goals.md) — what we measure ourselves against
- [`docs/roadmap.md`](docs/roadmap.md) — what's next
- [`docs/changelog.md`](docs/changelog.md) — what changed
- [`docs/functional-spec/`](docs/functional-spec/) — external behavior
- [`docs/architecture/`](docs/architecture/) — internals: [§AR-scanner](docs/architecture/AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations) for discovery, [§AR-checker](crates/grund-core/src/checker/report.rs) for validation, and [§AR-core-module-layout](docs/architecture/AR-core-module-layout.md#ar-core-module-layout-core-implementation-is-split-by-category) for the core source layout
- [`docs/decisions/`](docs/decisions/) — how we got here
- [`tests/e2e/`](tests/e2e/) — executable proof that the spec holds; [`tests/integration/`](tests/integration/) — proof that the parts fit as designed

# Checking a repository

`grund check` is the gate: it scans the tree, resolves every citation against every
declaration, and exits non-zero when one dangles or breaks a rule the repository
declared ([§FS-check](../functional-spec/FS-check.md#fs-check-grund-validates-every-citation-in-a-repo)). Run it in pre-commit and in CI.

## What it enforces

Renumber the heading `### 3.2 Missing section` in [`FS-check.md`](../functional-spec/FS-check.md) to `3.99` and `grund check` flags every site that leaned on it — code, tests, decision docs and the README alike, in one resolver:

```text
$ grund check
README.md:34: error: missing section FS-check.3.2
crates/grund-cli/tests/index_entry_round_trip.rs:232: error: missing section FS-check.3.2
crates/grund-core/src/checker/index.rs:247: error: missing section FS-check.3.2
crates/grund-core/src/checker/index.rs:361: error: missing section FS-check.3.2
crates/grund-core/src/checker/judge.rs:69: error: missing section FS-check.3.2
crates/grund-core/src/checker/references.rs:2: error: missing section FS-check.3.2
…
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
9. *(workspace)* Alias-qualified citations resolve across configured sub-projects. *(cross-project references — see [§FS-workspace](../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace))*

`grund check` reads what `[scan] include` names, so a citation in a directory the config never mentioned is invisible rather than merely unchecked — it neither resolves nor dangles. `grund check --full` ([§FS-check.1.3](../functional-spec/FS-check.md#13-the-full-tree-scope---full)) walks the whole repository past that key and reports the references that resolve to nothing out there, and only those: a directory nobody configured is never judged against conventions it never adopted. It is purely additive, so it can only turn a green run red.

`grund` doesn't invent citations — that's the contributor's call. What `grund` does is make sure the ones you wrote *resolve*. With `require_grounding = true` — in `[reference]` for every place at once, or on one `[[kinds]]` row for that place alone, at a `grounding_level` from the whole file down to every `##` of it ([§FS-config.3.4.8](../functional-spec/FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) — it also fails what carries no resolving citation; the stronger file-level "implementation changed with its spec and test, or bounded waivers" recipe is tracked separately in [§RM-cochange-gate](../roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits).

## The report

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
boundary. See the [watch guide](watch.md)
([§FS-check.6](../functional-spec/FS-check.md#6-watch-mode---watch)).
Exact-text consumers migrating from the former unmarked, global-location report
should use `--format=json`, whose bytes, object shape, and order are unchanged.

From this checkout, the initial report is:

```text
$ cargo run --quiet -- check --watch --format=text
success
```

Watch stays resident until Ctrl-C; this capture has empty stderr and exits 0 after that initial report.

## Narrowing the report

When you need a narrower answer without weakening the repository's default check,
select its stable finding codes: `grund check --ignore agents-init` asks whether
the remaining content report has errors, while repeatable `--only <code>` and
`--ignore <code>` compose as sets, and `--only-rule` narrows the same report by
rule authority instead — only what a `--rule` trial sentence authored, which is
what lets you try a sentence out for the cost of its own findings
([§FS-check.1](../functional-spec/FS-check.md#1-inputs), [§FS-rules.8](../functional-spec/FS-rules.md#8-command-surfaces)).
A check narrowed to a code rules produce still says when a rule behind it was
skipped as invalid ([§FS-rules.7.6](../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits)).
Selection happens only after the complete scan, and operational failures remain
visible; selected `success` describes only that view, not an all-findings
repository verdict ([§FS-check.2](../functional-spec/FS-check.md#2-outputs)).

## What it does not check

`grund` does **not** check Markdown links, URLs, spelling, or grammar. Use [`lychee`](https://github.com/lycheeverse/lychee), `vale`, etc. for those.

What a repository can add to the check:

- [Workspaces and sub-projects](repository-options.md#workspaces-and-sub-projects)
- [Keep shared values consistent](repository-options.md#keep-shared-values-consistent)
- [Cite external facts offline](repository-options.md#cite-external-facts-without-making-checks-depend-on-the-network)

## In pre-commit and CI

This repo ships a ready-to-install [.pre-commit-config.yaml](../../.pre-commit-config.yaml) — `grund check` for citations, `grund init --check` for a stale managed block, `lychee` for Markdown links. For optional staged commit-msg/whole-PR evidence of related spec and test edits with reason-bearing trailers, see the [co-change recipe](cochange.md); Grund runs its demonstrations without enabling that contribution gate ([§FS-cochange-recipe.examples](../functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance)):

```bash
pip install pre-commit && cargo install lychee && pre-commit install
```

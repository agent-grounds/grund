# DF-refs-unused-note: an uncited declaration's empty `refs` answer points at `grund list --unused`

**Date:** 2026-10-23

Gives [§FS-refs.2.1](../../functional-spec/FS-refs.md#21-an-id-with-no-citations) a second note, for the one empty answer it kept silent: a declared ID nothing cites. Reported as [grund#936](https://github.com/agent-grounds/grund/issues/936), which carries the note [grund#925](https://github.com/agent-grounds/grund/pull/925) deferred when it closed [grund#905](https://github.com/agent-grounds/grund/issues/905).

## status: Accepted

## 1. Context

`grund refs <ID>` answers for one ID at a time. For a declared ID nothing cites, it printed nothing, exited `0` and wrote nothing to stderr; only an ID declared nowhere earned a `note:`. An agent asked which specs nothing cites therefore ran one `grund refs` per declared ID, hundreds of processes in a project this size, when `grund list --unused` answers for the whole set in one call ([§FS-list.1.3](../../functional-spec/FS-list.md#13---unused)). That is the loop grund#905 reported. Its fix named `--unused` on the top-level help page ([§FS-cli.2.2](../../functional-spec/FS-cli.md#22-the-top-level-help-page)), which helps a caller who reads the help before looping. A caller already in the loop reads only `refs` output, and that was the one place that said nothing.

[§FS-refs.2.1](../../functional-spec/FS-refs.md#21-an-id-with-no-citations) kept the case silent because an uncited declaration is normal and `check` already warns about it. That reason is about not treating the case as an error or a typo, and it still holds: the answer stays empty and the exit stays `0`. What it does not require is that the answer be a dead end on the escalation ladder [§GOAL-token-economy.1](../../goals.md#1-what-this-requires) asks to make explicit.

## 2. Decision

When a whole-ID query of a declared ID lists no site over a complete scan of the whole project, `refs` prints one stderr `note:` that names the listing answering the question for every declaration ([§FS-refs.2.1.1](../../functional-spec/FS-refs.md#211-a-declared-id-nothing-cites-points-at-list---unused)). Stdout and the exit are unchanged in every format and fold. The command named is one whose listing holds the queried declaration: it adds the project alias to a qualified query, and `--kind E2E` for a case, which the plain listing leaves out. The note is withheld wherever that listing answers another question: a section, a path, `--glob` or `--from`, an incomplete scan, or an answer with sites in it.

The engine decides the note once and returns it as the existing `note` of the result, so the CLI, the Rust API and both bindings carry one value, with no new field or signature.

## 3. Rejected alternatives

- **Teach `--unused` only in the documentation.** The help row, the managed block and the guides reach a caller before the loop starts. The note reaches the caller inside it, at the first empty answer.
- **Put it on stdout.** That would break every parser of the result and the `--total` sentence, and stdout is the answer, not advice about it ([§FS-refs.2](../../functional-spec/FS-refs.md#2-behaviour)).
- **Give it on every empty answer.** Under a section, a path, a pattern or a source, `list --unused` answers a different question, and a hint that names it there would be wrong.
- **Withhold it under `--descendants`.** With no section, `--descendants` changes nothing ([§FS-refs.4](../../functional-spec/FS-refs.md#4-exit-codes)), and the query asks about the whole ID, which is what `--unused` answers.
- **Give it to an ID cited only by its index entry.** `refs` lists that entry, so the answer is not empty. Teaching the citation query the index rule ([§FS-list.3.2.1](../../functional-spec/FS-list.md#321-refs-counts-citations)) to close the gap would cost more than the hint is worth. A missed hint gives no wrong answer.

## 4. Consequences

Stdout and the exit status are unchanged in every mode, so nothing that parses the result or reads the status moves. Stderr moves: [§FS-refs.2](../../functional-spec/FS-refs.md#2-behaviour) had promised it silent here, so a caller that treats any stderr line from a successful run as a failure now sees one, and the `note` of a Rust, Python or Node `refs` result is now text in a case where it was `null`.

The promise changes without a transition flag, under the pre-release licence [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) names. A switch that kept the silence would be a lasting flag for one advisory line, and the silence is the defect the line exists to end. No finding, count or exit status changes, so this is no verdict correction.

## release-note: Release note

- [§DF-refs-unused-note](DF-refs-unused-note.md#df-refs-unused-note-an-uncited-declarations-empty-refs-answer-points-at-grund-list---unused), [§FS-refs.2.1.1](../../functional-spec/FS-refs.md#211-a-declared-id-nothing-cites-points-at-list---unused): **`grund refs` points at `grund list --unused` when a declared ID has no citations.** A query such as `grund refs FS-login` for a declaration nothing cites still prints nothing on stdout, or `not cited` under `--total`, and still exits `0`, and now also writes ``note: FS-login is declared but nothing cites it; `grund list --unused` lists every such declaration`` to stderr. An alias-qualified query names `--project <alias>`, an `E2E` case adds `--kind E2E`, and the note is withheld for an explicit path, a section, `--glob`, `--from` and an incomplete scan. **Who this affects:** a script that treats any stderr output from a successful `refs` run as a failure, and a Rust, Python or Node caller that reads a non-null `note` on a `refs` result as a typo. Stdout and exit codes are unchanged.

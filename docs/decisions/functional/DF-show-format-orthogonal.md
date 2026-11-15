# DF-show-format-orthogonal: show's `--format` answers every query form, and a `text` or `md` batch heads each answer `==> <coordinate> <==`

**Date:** 2026-11-15

## status: Accepted

## 1. Context

`grund show --batch` answered only in JSON. Any other `--format`, including the `text` default, was refused with `error: --batch requires --format=json` and exit `2`, for an stdin query stream, for `--glob` and for `--all` alike. A person reading a chosen set of points at a terminal therefore piped the envelopes through `jq`, and the usual filter, `jq -r '.result.body'`, prints `null` in place of a failed query, drops which coordinate each body answers, and exits `0` unless the shell sets `pipefail`. In a workspace it loses the answer outright: two members' bodies that both open `## 1. One` are told apart only by a `query.id` the filter threw away. That is the abridged diagnostic [§GOAL-token-economy.2](../../goals.md#2-what-this-rules-out-1) rules out, and it is the friction [§GOAL-friendliness-first](../../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) is about.

Keeping the batch JSON-only was a ruling, not an accident: issue #407 offered a plain-text batch as its option 2, and on 2026-10-04 that option was declined. On 2026-11-13 the owner reversed it, on the principle that the output format does not depend on how the queries are selected, and on 2026-11-14 approved the design below on issue #1385.

## 2. Decision

### 2.1 Every format answers every query form

`--format` is one knob, independent of the query form: `text`, `md` and `json` each answer one coordinate, an stdin stream, `--glob` and `--all` ([§FS-show.1.8](../../functional-spec/FS-show.md#18---batch-an-explicit-query-stream), [§FS-show.3.3](../../functional-spec/FS-show.md#33-batch-query)). This reverses the 2026-10-04 ruling on #407. `--format json` prints the envelopes it printed before, byte for byte ([§FS-output-shapes.4.1](../../functional-spec/FS-output-shapes.md#41-show---batch---formatjson)), and the query input stays NDJSON: bare-coordinate stdin, #407's option 1, is not part of this.

### 2.2 The header is `==> <coordinate> <==`

Each answer of a `text` or `md` batch opens with a line of its own, `==> <coordinate> <==`, and every header but the first follows an empty line ([§FS-show.3.3.1](../../functional-spec/FS-show.md#331-the-header)). It is the convention `head` and `tail` use for several files, each named, so a reader recognises it, and the closing `<==` shows where the coordinate ends. A Markdown heading such as `# FS-show.1.7` lost: it reads as one of the author's headings, and pasted into a scanned file it looks like a declaration. A succeeded query's coordinate is spelled as `--all` generates it, alias and that project's separator included, so it can be pasted back into `grund show`; a failed one is the query as written.

No text separator can be unambiguous, because a body is its author's bytes and may contain this very line. The specification says so outright: the framing is for reading, and a program that has to split the answers reads `--format json`.

### 2.3 A failure stays in its place, on stdout only

Under a failed query's header come its message, the bytes the envelope's `error.message` carries, and the `hint:` lines the single read prints for it, on stdout and nowhere else ([§FS-show.3.3.2](../../functional-spec/FS-show.md#332-a-failure-stays-in-its-place)). That gives the exception one rule in every format: a batch keeps every query's outcome on stdout, in query order. Stderr-only lost: the failure loses its place among the answers, which was the point of the change. Both streams lost: a terminal would show every failure twice. Run-level failures do not move: they leave stdout empty, print `error:` on stderr and exit `2`. Every hint is read from the project the query names, out of the one load ([§FS-show.3.5](../../functional-spec/FS-show.md#35-the-hint-for-each-failure)).

### 2.4 `md` frames as `text` does

The header is framing, not body, so it is the same line in both formats and carries the alias the `md` declaration heading lacks. It renders as a paragraph, and each `md` body already opens with its own heading line. A Markdown-native frame would collide with the author's bytes: `---` reads as a thematic break or front matter, and an HTML comment disappears when rendered.

## 3. The bytes that move

Every invocation that succeeded before prints the same bytes on both streams with the same exit code, and no tree's verdict moves. What moves is output that [§GOAL-no-silent-breakage.1](../../goals.md#1-what-counts-as-user-visible) counts as user-visible and [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) holds to stable phrasing:

- `grund show --batch …` with no `--format`, or with `text` or `md`, no longer prints `error: --batch requires --format=json` and exits `2`; it prints the framed answers and exits `0` or `1`. A script that relied on that refusal now gets text.
- `--batch` with an unknown format is refused as the single read refuses it, ``error: unsupported show format `<f>` ``, still with exit `2`.
- The second-coordinate hint ([§FS-show.1.4.1](../../functional-spec/FS-show.md#141-a-second-coordinate-in-the-path-slot)) builds its command from the read's own format: it ends `grund show --batch` after a `text` read and `--format=md` after an `md` one, where it always ended `--format=json`. After a `json` read it is unchanged.
- A missing section in a project whose separator differs from the one at the run's path is hinted with that project's separator: `api/FS-api:9` in a member whose separator is `:` gets `grund api/FS-api --toc`, where it got `grund api/FS-api:9 --toc`, a command that fails the same way.

None of these has an old form worth carrying beside the new one: the refusal guarded nothing a caller could use, and a hint that names the wrong command is not worth a deprecation window. So they move under the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), as [§DF-show-default-token-cheap.4](DF-show-default-token-cheap.md#4-consequences) moved `show`'s flags without aliases. That licence expires at `1.0`, and the owner ruled that this change land before 0.23.0 is cut. It is not a verdict correction, because no verdict moves.

## 4. Consequences

- `grund show --help` shows `--format` in the batch usage line and a text batch among its examples.
- The query guide, the review-checklist example and the shipped `grund-init` skill teach the text batch for reading bodies; the managed block keeps teaching `--toc --format json`, which still works, so its version does not move.
- The bindings take no format and do not change: they project the CLI's JSON batch bytes ([§FS-distribution.3.2.1](../../functional-spec/FS-distribution.md#321-operations-options-and-refusals)).

## release-note: Release note

- [§DF-show-format-orthogonal](DF-show-format-orthogonal.md#df-show-format-orthogonal-shows---format-answers-every-query-form-and-a-text-or-md-batch-heads-each-answer--coordinate-), [§FS-show.3.3](../../functional-spec/FS-show.md#33-batch-query): **`grund show --batch` (stdin, `--all` or `--glob`) now prints in every `--format`.** `text`, the default, and `md` head each answer with a `==> <coordinate> <==` line and keep a failed query's message and hint in its place on stdout, where the command used to refuse with `--batch requires --format=json` and exit `2`; `--format json` prints the same envelopes as before. **Who this affects:** a script that ran `grund show --batch` without `--format json` and relied on its refusal now gets text; add `--format json` to keep the envelopes. An unknown format under `--batch` is now ``unsupported show format `<f>` ``, the second-coordinate hint keeps the read's own `--format`, and a missing section in a member whose separator differs is hinted with that member's separator. Part of [issue #1385](https://github.com/agent-grounds/grund/issues/1385).

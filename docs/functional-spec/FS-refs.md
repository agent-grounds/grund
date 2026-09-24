# FS-refs: grund lists every citation of an ID

The `refs` subcommand answers the reverse of `grund <ID>`: not "what does this ID say?" but "who points at it?". An agent about to change a declaration — or delete one — needs to know what leans on it; `grund refs FS-check` is that lookup, scheme-aware in the ways a `grep` cannot be, with a compact `--summary` for a quick blast-radius read. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible), [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), and the agent-grounding loop in [§GRUND-grund](../grund.md#grund-grund-agents-stay-grounded-in-the-spec) (an agent verifying a change reads the cited bodies *and* the back-references).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (doc-comment),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations)
(value, binding, snapshot).

## 1. Inputs

```
grund refs <ID> [<path>] [--section <s>] [--summary] [--format text|json]
```

- `<ID>` — the ID to look up, without the marker. It may carry an inline section in the configured `[id] section_separator` (`FS-check.3.1`, or `FS-plan.goals.performance` when named sections are enabled), or pass it as `--section 3.1` / `--section goals.performance`. Parsing uses the named kind's effective format and the selected project's shared catalog ([§FS-config.3.2](FS-config.md#32-id--id-grammar)): an exact off-grammar declaration is a valid read query and keeps its raw spelling, while an off-grammar argument with no exact declaration keeps the ordinary invalid-ID hint. The number-only shorthand is accepted where the effective format has one — `grund refs FS-042` lists the citations of `FS-042-user-login`, including those written in the shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) — and shorthand, duplicate, and section ambiguities are rejected with their candidates rather than resolved to a guess.
- `<path>` — directory or file whose tree is scanned, default `.`, discovered as by every other subcommand ([§FS-config.1](FS-config.md#1-file-location-and-discovery)).
- `--section <s>` — list only citations of exactly that numeric or named section path; without it, every citation of `<ID>` is listed, bare-ID ones included. Mutually exclusive with the dotted inline form. The filter reads the scanner's shared citation record, so it cannot disagree with `check`, `show`, completion, or the LSP about a named coordinate.
- `--summary` — one line per citing **file** instead of one per site ([§FS-refs.3.3](FS-refs.md#33---summary)).
- `--format text|json` — output shape ([§FS-refs.3](FS-refs.md#3-outputs)). Default `text`.

`refs` is a query, like `show` — non-interactive, no prompts ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).

## 2. Behaviour

`refs` runs the same scan as `check` ([AR-scanner](../architecture/AR-scanner.md#ar-scanner-how-grund-discovers-declarations-and-citations)) and lists, for the requested `<ID>`, every recognised citation site — the set `check` would validate, so it honours `[reference] strict` (bare tokens are listed only in non-strict mode), the source-string exclusions ([§FS-check.1.1.3](FS-check.md#113-string-literals-in-source-files)), and citations inside doc-comments. In particular, a real Python module, function, or method docstring after assigned triple-quoted data remains in the set, while the assigned-data span itself is absent ([§FS-check.1.1.3.1](FS-check.md#1131-assigned-python-triple-quoted-data)). It does **not** list the *declaration* of `<ID>` — that is `grund <ID> --format=json` (the README documents that one-liner). An ID with no declaration still lists its citations, exactly the ones `check` flags as dangling, so `refs` is also the "what would break if I never create this ID" tool.

An owned declaration-local numeric citation ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)) is in that same set under its resolved full ID and exact section. Whole-ID and `--section` queries include it, while text and JSON retain the authored local token. Ownerless and unsupported local candidates have no guessed target and therefore appear in no target's result.

Output is sorted by `(path, line, column)` ([§FS-errors.4](FS-errors.md#4-determinism)). The list is the command's *result*, so it goes to **stdout** — text lines and `--format json` NDJSON alike, as for `grund list` and `grund cover` ([§FS-errors.1](FS-errors.md#1-streams)). A text line has the `<path>:<line>: <message>` located-finding shape ([§FS-errors.2.1](FS-errors.md#21-located-finding)) so an editor can jump to it, but it is an *answer*, not a diagnostic; stderr is left for errors and the typo note of [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations).

### 2.1 An ID with no citations

An ID with zero citations produces empty output and exit `0`: an as-yet-uncited declaration is normal, and `check` already warns about it ([§FS-check.4.1](FS-check.md#41-unused-declaration)). If the ID is *also* declared nowhere in the scanned tree, the likeliest cause is a typo, so `refs` prints one `note:` line to **stderr** — `note: <ID> is neither declared nor cited — run \`grund list\` to see every declared ID` — and still exits `0`. The note is a hint, not part of the result: stdout stays empty, so machine consumers that read only stdout never see it. It mirrors the `ID not found` hint the ID query gives for the same mistake ([§FS-show.3](FS-show.md#3-outputs)) without that query's exit `1`. A resolver rejection is different: once the selected project's grammar rejects the operand there is no citation-list result, and from 0.15.0 that failed query exits `1` ([§FS-refs.4](FS-refs.md#4-exit-codes)).

### 2.2 Value bindings

A citation inside a recognized value binding, local or alias-qualified, is an ordinary citation in this index; JSON declaration sources themselves never contribute citations ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts)).

### 2.3 Fetch-backed snapshots

The undeclared-ID rule of [§FS-refs.2](FS-refs.md#2-behaviour) includes a missing fetch-backed snapshot: `refs` reports its citation sites without executing the configured integration. Once fetched, the result set is unchanged; only the target now resolves.

## 3. Outputs

### 3.1 `--format text` (default)

One line per citation site on **stdout**, in the located-finding shape ([§FS-errors.2.1](FS-errors.md#21-located-finding)):

```
$ grund refs FS-check.1
crates/grund-core/src/scanner/file_pass.rs:142: FS-check.1
docs/functional-spec/FS-show.md:11: §FS-check.1
```

`<message>` is the citation token exactly as it appears in the source — marker-prefixed or bare, with its section suffix — so the reader sees the form on disk. Exit `0` whenever the scan succeeds and the operand is not refused, regardless of how many citations were found; a refused operand exits `1` ([§FS-refs.4](FS-refs.md#4-exit-codes)).

### 3.2 `--format json`

NDJSON on stdout — one object per citation, matching the `Citation` shape ([AR-scanner.3](../architecture/AR-scanner.md#3-output)) plus the verbatim token and optional target-kind metadata. These examples select a kind without an effective title:

```json
{"path":"crates/grund-core/src/scanner/file_pass.rs","line":142,"column":12,"id":"FS-check","section":"1","marker":false,"text":"FS-check.1"}
{"path":"docs/functional-spec/FS-show.md","line":11,"column":42,"id":"FS-check","section":"1","marker":true,"text":"§FS-check.1"}
```

`section` is `null` for a bare-ID citation with no section coordinate.
`kind_title` is appended after `text` when the resolved queried kind has an
effective title ([§FS-config.3.4.3](FS-config.md#343-title)); absent titles omit
it, while a configured empty string is retained. A workspace record's existing
`project` still names the citing project, whereas `kind_title` belongs to the
selected target project, even when the queried ID is undeclared but valid.
All existing fields retain their values and relative order.

### 3.3 `--summary`

`grund refs <ID> --summary` emits one line per citing **file** instead of one per citation site, sorted by path:

```
$ grund refs FS-check --summary
crates/grund-core/src/scanner/file_pass.rs: 1 (line 142)
docs/functional-spec/FS-show.md: 3 (lines 11, 142, 200)
```

The shape is `<path>: <count> (lines <l1>, <l2>, …)`. The count is the number of sites from exactly the citation set [§FS-refs.3.1](FS-refs.md#31---format-text-default) lists, so `--summary` honours `[reference] strict`, the string-literal carve-out, and doc-comment citations the same way; the line list is the sorted, de-duplicated set of source lines holding them, so two citations on line 10 read `path: 2 (line 10)`. `grund refs <ID> --summary | wc -l` is then the number of files that lean on `<ID>`, while the line list still points an editor at every line with a site. With `--section`, the aggregate is over that section's citations only. No citations prints nothing, with exit `0` and the [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations) `note:` unaffected. With `--format json`: NDJSON, one object per file, `{"path":<path>,"count":<n>,"lines":[<unique l1>,<unique l2>,…]}`, in the same order; without `--summary` it is the per-citation form of [§FS-refs.3.2](FS-refs.md#32---format-json). Summary objects omit `kind_title`, including for a titled target kind. Exit codes ([§FS-refs.4](FS-refs.md#4-exit-codes)) are unchanged — `--summary` renders the same scan result, not a different query.

## 4. Exit codes

- `0` — scan succeeded; the listed citations (possibly none) are the result.
- `1` — from grund 0.15.0, the selected project's resolver rejected the ID
  operand: it does not match the effective `[id] format`, or it is a
  number-only shorthand naming more than one declaration
  ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)). Both text and
  JSON leave stdout empty. Text uses the bare query-failure shape
  ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)); only invalid format
  keeps the configured-format `hint:`. JSON emits exactly one failed-query
  diagnostic object on stderr, with `code` `invalid-id` or `ambiguous` and
  `sites:null`, and emits no hint ([§FS-errors.5](FS-errors.md#5-json-format)).
  It is also exit `1` when the ID has more than one home or its section path
  is claimed by more than one heading: `refs` refuses both as `show` does
  ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id),
  [§FS-show.2.2.2](FS-show.md#222-ambiguous-section)), and under JSON that
  object names its `sites` ([§FS-errors.5.2.1](FS-errors.md#521-sites-on-an-ambiguity-refusal)).
- `2` — scan / I/O error ([§FS-check.2](FS-check.md#2-outputs) partial-scan
  semantics apply: an incomplete scan exits `2` and the lookup is not
  trustworthy as complete), an unsupported `--format`, an unknown alias, or
  any other setup or CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)).

Grund 0.14.0 is the compatibility release. For the two resolver rejections it
keeps the former exit `2`, the existing `error:` diagnostic, and the same text
hint policy in text and JSON invocations, then appends exactly this raw stderr
line in both modes:

```text
warning: `grund refs` invalid IDs and ambiguous number-only shorthands currently exit 2; they will exit 1 (failed query) in grund 0.15.0
```

The warning and the `error:` prefix retire together at 0.15.0. `--summary` and
`--section` do not introduce another classification: after context and grammar
selection they inherit the same operand result. This staged boundary is the
decision in [§DF-refs-resolver-rejection](../decisions/functional/DF-refs-resolver-rejection.md#df-refs-resolver-rejection-an-id-rejected-by-a-selected-grammar-is-a-failed-query).

## 5. Why this exists

`grep -oE '§…'` gives a contributor a rough back-reference list but cannot tell a real citation from an ID-shaped substring in a string literal, respect `strict` mode, reach citations inside block doc-comments without language-specific regex, or produce a stable, machine-shaped result for an agent to program against. `refs` is the scheme's own answer, sharing the scanner with `check` so the two never disagree on what counts as a citation. `--summary` folds a wide back-reference set to one line per file, so the blast radius before changing a declaration is legible at a glance — token-cheap for an agent that needs the count and the file list, not every column. With `grund <ID>` it closes the loop: the ID query reads the body an ID promises, `refs` enumerates the code and docs that took the promise.

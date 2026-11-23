# FS-output-shapes: machine-readable output shapes

This page shows, on the wire, the bytes a program reads from `grund`: the JSON objects and text reports that [§FS-check](FS-check.md#fs-check-grund-validates-every-citation-in-a-repo), [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id), [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file) and [§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up) produce, the stream each goes to, and the exit beside it. [§FS-errors](FS-errors.md#fs-errors-grund-emits-messages-in-fixed-shapes) states the rules they follow; the examples here are normative for fields, stream split and ordering. Serves [§GOAL-no-silent-breakage.1](../goals.md#1-what-counts-as-user-visible).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, lead),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation, shorthand, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source
declaration, stub), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, workspace, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding,
severity, suggestion, caution), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, binding).

## 1. Finding object

Every finding `check` reports, and the one failure of a failed query, is an object of this shape, a suggestion carrying `"channel": "suggestion"` in place of `severity`, its keys in the order and with the meanings [§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output) gives:

```json
{"severity":"error","path":"docs/functional-spec/FS-001-alpha.md","line":3,"code":"dangling","message":"unknown reference FS-999-missing","sites":null,"authority":null}
{"severity":"error","path":"docs/offer.md","line":7,"code":"value-mismatch","message":"value mismatch for CONST-field-price.1: bound `1250`, declared `1200` at values/field-price.md:2; change the literal to the declared value, or the declaration to the bound one","sites":[{"path":"values/field-price.md","line":2}],"authority":null}
{"severity":"warning","path":null,"line":null,"code":"empty-scan","message":"nothing to scan — no files under `<path>` matched grund's extensions (<list>).","sites":null,"authority":null}
```

`code` is a stable kebab-case code, either one of [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)'s catalog or one of the failed-query codes [§FS-distribution.3.0.1](FS-distribution.md#301-report-and-finding) lists. `path` and `line` are each `null` where the finding has none: a failed query has neither, and a file-level finding has a `path` and no `line` ([§FS-distribution.3.0.1](FS-distribution.md#301-report-and-finding)). A value mismatch is located at its binding, and its `sites` is its one declaration site ([§FS-values.5.2](FS-values.md#52-fixed-value-errors)). Which stream each object goes to is [§FS-errors.5](FS-errors.md#5-json-format)'s.

## 3. Text report ordering

Text groups findings by channel and JSON keeps one global `(path, line, message)` order ([§FS-errors.4.1](FS-errors.md#41-ordering)), so the same findings can print in a different order in each. A failing `check` whose warning sorts first by message at one location prints, in text:

```text
docs/other.md:1: error: duplicate declaration of FS-second (also declared at source.rs:6); keep one declaration of the ID and rename or remove the others
docs/other.md:1: warning: declared but never cited: FS-second; cite it
```

and in JSON:

```json
{"severity":"warning","path":"docs/other.md","line":1,"code":"unused","message":"declared but never cited: FS-second; cite it","sites":null,"authority":null}
{"severity":"error","path":"docs/other.md","line":1,"code":"duplicate","message":"duplicate declaration of FS-second (also declared at source.rs:6); keep one declaration of the ID and rename or remove the others","sites":[{"path":"docs/other.md","line":1},{"path":"source.rs","line":6}],"authority":null}
```

## 4. `show --format=json`

A successful read prints the object [§FS-show.3.1.3](FS-show.md#313-json) defines, its keys in this order: `id`, `section`, `body`, `sections` (`--toc` only), `kind_title` (a kind with a title, an empty one included, [§FS-config.3.4.3](FS-config.md#343-title)), `anchor`, then `path` and `line`, where the declaration or the selected section starts. Every slice prints this object, `--brief` narrowing only `body`. Here a default read of a kind with no title, then `--toc` of a section of a titled kind:

```json
{"id":"FS-authored","section":null,"body":"Body stays authored.\n","anchor":"fs-authored-authored-contract","path":"docs/FS-authored.md","line":1}
{"id":"FS-001-alpha","section":"goals","body":"## goals: Goals, scope & limits\n\nGoals body.\n\n### goals.performance: Performance (p99)\n","sections":[{"path":"goals.performance","title":"Performance (p99)","depth":1,"anchor":"goalsperformance-performance-p99"}],"kind_title":"What: behavior, requirements, and constraints","anchor":"goals-goals-scope--limits","path":"docs/functional-spec/FS-001-alpha.md","line":5}
```

An E2E case ([§FS-declarations.e2e-case](FS-declarations.md#e2e-case-what-an-e2e-case-directory-declares)) prints the manifest object of [§FS-show.2.4.2](FS-show.md#242-the-manifest-as-json) instead, here a workspace member's case with no `command.args`:

```json
{"id":"E2E-login","kind":"E2E","path":"apps/api/e2e/cases/login","args":[],"expected_exit":0,"fixtures":["expected.exit"],"anchor":null,"kind_title":"Executable user scenarios"}
```

A value-source declaration or one of its components, JSON or YAML ([§FS-values.6.2](FS-values.md#62-value-source-declarations-in-the-catalog)), and a front-matter coordinate ([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)), whose `section` is `front-matter` or `front-matter.<key>` and whose `kind_title` is its declaration's, print this object in every read mode, with `body` the exact member, element or front-matter slice, `path` and `line` that slice's start, and `anchor` `null`.

### 4.1 `show --batch --format=json`

A batch prints one NDJSON envelope per query ([§FS-show.3.3](FS-show.md#33-batch-query), [§FS-errors.5.3](FS-errors.md#53-show---batch)), its keys in this order:

```json
{"query":{"id":"FS-alpha","section":"1"},"ok":true,"result":{"id":"FS-alpha","section":"1","body":"## 1. Numeric\n\nNumeric lead.\n","kind_title":"What: behavior, requirements, and constraints","anchor":"1-numeric","path":"docs/FS-alpha.md","line":7},"error":null}
{"query":{"id":"FS-missing","section":null},"ok":false,"result":null,"error":{"severity":"error","path":null,"line":null,"code":"not-found","message":"ID not found: FS-missing","sites":null,"authority":null}}
{"query":{"id":"missing/FS-alpha","section":null},"ok":false,"result":null,"error":{"severity":"error","path":null,"line":null,"code":"unknown-project","message":"unknown project alias `missing`\nknown aliases: root, member","sites":null,"authority":null}}
```

`query.id` is the caller's spelling for explicit input and the generated local-or-qualified spelling for `--all`, and `query.section` is the explicit or generated section string, or `null`. A success's `result` is byte for byte the object the single read of the same coordinate and mode prints, explicit and `--all` alike, and its `error` is `null`. A failure's `result` is `null` and its `error` is the finding object of [§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output). An unknown alias, which the single read refuses at launch, is code `unknown-project`, the code citation resolution uses, and its message is the single read's without the `error:` prefix, its `known aliases:` or standalone `note:` line included.

## 5. `list --format=json`

`list --format=json` prints one object per row, in [§FS-list.2.4](FS-list.md#24-order)'s order:

```json
{"id":"FS-login","kind":"FS","path":"docs/fs/FS-login.md","line":1,"title":"Users log in","stub":false,"defines":null,"refs":6,"duplicate":false}
{"id":"FS-login","section":"1","kind":"FS","path":"docs/fs/FS-login.md","line":5,"title":"First","stub":false,"defines":null,"refs":2,"duplicate":false}
```

Fields, in this order:

- `project`, in a workspace only ([§FS-workspace.8.3.5](FS-workspace.md#835---format-json)).
- `id`, the rendered ID, then `section`, the exact section path, on a section row only ([§FS-rules.8](FS-rules.md#8-command-surfaces)).
- `kind`, then `path` and `line` of the declaration or section heading.
- `title`, [§FS-list.3.1](FS-list.md#31---format-text-default)'s title; `null` where the text form's title column is empty or shows `→ <target>` ([§FS-list.3.1.1](FS-list.md#311-row-notes)).
- `stub`, true only for a broken stub, and `defines`, a stub's target path, otherwise `null` ([§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical)).
- `refs`, [§FS-list.3.2.1](FS-list.md#321-refs-counts-citations)'s count.
- `duplicate`, true when this ID has more than one independent declaration home.
- `value_roots`, only on a row that has them ([§FS-list.3.2.2](FS-list.md#322-value_roots)).

`list --summary --format=json` prints one object per summary line, in [§FS-list.3.3](FS-list.md#33---summary)'s order: `project` in a workspace only, `kind`, `title`, `home`, `count`. `title` is the kind's title, left out when the kind has none ([§FS-config.3.4.3](FS-config.md#343-title)); `home` and `count` are [§FS-list.3.3](FS-list.md#33---summary)'s columns.

```json
{"kind":"GOAL","title":"Where we are going","home":"docs/goal/goals.md","count":1}
{"kind":"FS","home":"docs/fs","count":1}
```

`list --total --format=json` prints exactly one `{"count":<n>}` object instead, at every count, `0` included: the number of rows the same filters list, with no other field ([§FS-list.total](FS-list.md#total---total-a-count-rung)).

### 5.1 `refs --format=json`

`refs --format=json` prints one citation object per line, its fields as [§FS-refs.3.2](FS-refs.md#32---format-json) specifies them, in this order: `path`, `line`, `column`, `id`, `section`, `marker`, `text`, the citing site's `enclosing_declaration` and `enclosing_section`, then the optional `kind_title` of the queried target kind, last. Under `--glob` and `--from` it keeps this order and drops only `kind_title`, and `id` keeps the alias the citation was written with, as `cover` writes it.

```json
{"path":"docs/functional-spec/FS-002-beta.md","line":3,"column":10,"id":"FS-001-alpha","section":null,"marker":false,"text":"FS-001-alpha","enclosing_declaration":"FS-002-beta","enclosing_section":null,"kind_title":"What: behavior, requirements, and constraints"}
```

With `--summary` it prints one object per file instead, its `count` and `lines` [§FS-refs.3.3](FS-refs.md#33---summary)'s, with no `kind_title` and no enclosing pair:

```json
{"path":"docs/functional-spec/FS-002-beta.md","count":3,"lines":[3,5]}
```

With `--total` it prints exactly one `{"sites":<n>,"files":<m>}` object at every count, [§FS-refs.3.4](FS-refs.md#34---total)'s two numbers, with no `project`, `kind_title` or enclosing pair.

### 5.2 `list --size --format=json`

`list --size --format=json` prints one object per declaration and citable section site, in the row order of [§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements):

```json
{"id":"ADR-cache","section":null,"kind":"ADR","path":"docs/adr/cache.md","line":7,"stub":false,"defines":null,"duplicate":false,"lead_lines":1,"full_lines":1,"lead_words":1,"full_words":1,"lead_bytes":6,"full_bytes":6}
{"id":"ADR-cache","section":"front-matter","kind":"ADR","path":"docs/adr/cache.md","line":1,"stub":false,"defines":null,"duplicate":false,"lead_lines":5,"full_lines":5,"lead_words":15,"full_words":15,"lead_bytes":96,"full_bytes":96}
```

Its fields, in this order: `project` in a workspace only, with `id` workspace-qualified ([§FS-workspace.8.3](FS-workspace.md#83-grund-list)); `id`, `section`, `kind`, `path`, `line`, `stub`, `defines` and `duplicate`; then a `lead_<unit>`, `full_<unit>` pair per selected unit, in the caller's order, so bare `--size` gives `lines`, `words`, then `bytes`. An unselected unit has no fields, a broken stub's selected measurements are `null`, and no row carries `title` or `refs`. A front-matter coordinate is a row like any section's, after the declaration's row and among its section paths in their order, and its lead and full both measure its slice ([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)).

### 5.3 `cover --lines --format=json`

`cover <file> --lines <range> --format=json` prints one line-ownership object per `--lines`, in the order given ([§FS-cover.6.3](FS-cover.md#63-output)):

```json
{"path":"src/lib.rs","start":1,"end":10,"owners":[{"declaration":"FS-003-gamma","start":1,"end":7,"sections":[{"section":null,"start":1,"end":4},{"section":"1","start":5,"end":7}]}]}
{"path":"src/lib.rs","start":8,"end":8,"owners":[]}
```

Its fields, in this order:

- `path`, then `start` and `end`, the requested range as numbers.
- `owners`, an array of owner runs in line order, empty when nothing in the range is owned. Each is `declaration` (a bare ID string), then `start` and `end` of the run, then `sections`.
- `sections`, a non-empty array of section runs in line order that partitions its owner run. Each is `section` (`string | null`, `null` for no section), then `start` and `end`.

No `--lines` record carries the `project` key a [§FS-cover.3.2](FS-cover.md#32---format-json) record can lead with: a `--lines` run names one file, which is always a narrowed scan with no workspace loaded ([§FS-workspace.8.6.1](FS-workspace.md#861-a-narrower-path-is-one-narrowed-scan)).

## 6. CLI and config failures

A failure that stops a command before it reaches its data is raw `error:` text on stderr, never JSON, even under `--format=json` ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)):

```text
error: grund.toml:2: unknown config key `strcit`
```

### 6.1 `refs` resolver rejections

A `refs` operand the selected resolver rejects, an invalid ID or an ambiguous number-only shorthand, is not a CLI failure but a failed query ([§FS-refs.4](FS-refs.md#4-exit-codes), [§DF-refs-resolver-rejection](../decisions/functional/DF-refs-resolver-rejection.md#df-refs-resolver-rejection-an-id-rejected-by-a-selected-grammar-is-a-failed-query)). Here an ID that misses the configured `[id] format`, in text and in JSON:

```text
invalid ID `FS-bar`
hint: this repo's [id] format is `{kind}-{number}-{slug}` (run `grund config show`); `grund list` shows the IDs that exist
```

```json
{"severity":"error","path":null,"line":null,"code":"invalid-id","message":"invalid ID `FS-bar`","sites":null,"authority":null}
```

## 7. Stream matrix

What reaches stdout and stderr in each case, and the exit:

| Case | stdout | stderr | Exit |
|------|--------|--------|------|
| clean `check` text | `success\n` | empty | `0` |
| clean `check --format=json` | empty | empty | `0` |
| empty scan JSON | empty | one warning finding object | `0` |
| graph findings text | located finding lines | empty unless run-level findings exist | `1` |
| graph findings JSON | finding NDJSON | line-less run findings only | `1` |
| successful `show --format=json` | one result object | empty | `0` |
| failed `show --format=json` query | empty | one finding object | `1` |
| `refs` resolver rejection text, `--summary` and `--total` included ([§FS-refs.4](FS-refs.md#4-exit-codes)) | empty | bare failure; invalid format alone adds a hint | `1` |
| `refs` resolver rejection JSON, `--summary` and `--total` included | empty | one `invalid-id` or `ambiguous` finding object; no hint | `1` |
| `refs --total` text, cited | one `cited at <n> site(s) across <m> file(s)` line | empty | `0` |
| `refs --total` text, uncited | one `not cited` line | empty, or one [§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations) `note:` | `0` |
| `refs --total --format=json`, any count | exactly one `{"sites":<n>,"files":<m>}` object | as the text rows | `0` |
| `list --total` text, any count | the bare count and a newline | empty | `0` |
| `list --total --format=json`, any count | exactly one `{"count":<n>}` object | empty | `0` |
| `list --total`, nothing selected | `0`, or `{"count":0}` in JSON | empty | `0` |
| refused coordinate glob ([§FS-coordinate-globs.errors.shape](FS-coordinate-globs.md#errorsshape-one-refusal-line)) | empty | one raw `error: invalid coordinate glob …` line | `2` |
| valid glob no kind in scope can start, on any `--glob` surface | that surface's ordinary result | one `note:` line per such pattern ([§FS-coordinate-globs.errors.note](FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)) | `0` |
| `refs --glob` or `refs --from`, any count, text or JSON | the sites, a `--summary` row per file, or the one `--total` line or object | empty, or the glob notes under `--glob` | `0` |
| `refs --from --total` text | one `cites at <n> site(s) across <m> file(s)` line, or `cites nothing` | empty | `0` |
| `refs --from` source that fails as a query, text | empty | show's failure text and its `hint:` ([§FS-refs.4](FS-refs.md#4-exit-codes)) | `1` |
| `refs --from` source that fails as a query, JSON | empty | one `not-found`, `missing-section`, `invalid-id`, `ambiguous` or `ambiguous-section` finding object | `1` |
| `cover --glob`, any count, matching nothing included | every scanned file's record, each with only the matched citations | empty, or the glob notes | `0` |
| `refs` or `cover` flag combination refused ([§FS-refs.1.2](FS-refs.md#12-refusals), [§FS-cover.6.4](FS-cover.md#64-errors)) | empty | one raw `error:` line | `2` |
| `refs --glob` or `--from` with a missing path that reads as an ID ([§FS-refs.1.3](FS-refs.md#13-an-id-typed-as-the-path-gets-a-hint)) | empty | the `error: path does not exist:` line and one `hint:` line | `2` |
| `cover --lines`, a scanned file, text or JSON | one block or object per range | empty | `0` |
| `cover --lines`, a file outside the scan | empty | empty | `0` |
| `cover --lines` usage error ([§FS-cover.6.4](FS-cover.md#64-errors)) | empty | one raw `error:` line | `2` |
| bad flag / malformed CLI | empty | raw `error:` text | `2` |
| invalid config during `config validate` ([§FS-errors.2.2.2](FS-errors.md#222-exit-codes)) | empty | raw `error: <path>:<line>:` text | `1` |
| invalid config blocking another command | empty | raw `error: <path>:<line>:` text | `2` |
| `schema`, any format, printed | the view | empty | `0` |
| `schema`, invalid config or usage error | empty | raw `error:` text, plus `known formats:` for an unknown format | `2` |
| semantic value finding, text | located finding lines | empty | `1` |
| semantic value finding, JSON | finding NDJSON with declaration `sites` | empty | `1` |
| a value source that [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete, whichever its format | partial findings if any | incomplete-scan finding | `2` |

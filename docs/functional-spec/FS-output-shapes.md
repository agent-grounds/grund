# FS-output-shapes: machine-readable output shapes

This file is the verbose output-shape companion to [§FS-errors](FS-errors.md#fs-errors-grund-emits-messages-in-fixed-shapes). It collects the JSON/text envelopes that are spread across [§FS-check](FS-check.md#fs-check-grund-validates-every-reference-in-a-repo), [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id), [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-id](FS-id.md#fs-id-grund-proposes-ids-for-new-declarations), and [§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up). The examples here are normative for fields, stream split, and ordering.

## 1. Diagnostic object

Diagnostics use this shape:

```json
{"severity":"error","path":"docs/functional-spec/FS-001-alpha.md","line":3,"code":"dangling","message":"unknown reference FS-999-missing","sites":null}
```

Fields:

- `severity` is `error` or `warning`. A suggestion, emitted only under `check --suggestions`, carries `"channel": "suggestion"` in place of a `severity` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)).
- `path` is a relative path string, or `null` when there is no single source location.
- `line` is 1-indexed, or `null` when `path` is `null`.
- `code` is a stable kebab-case diagnostic code.
- `message` is the same lowercase text used in text mode: no terminal period on a single-clause message, while a run-level caution of more than one clause keeps its sentences' periods ([§FS-errors.3](FS-errors.md#3-message-text)).
- `sites` is `null` for single-site diagnostics, or a sorted array of `{ "path": <path>, "line": <line> }` for multi-site diagnostics.

`check --format=json` splits these objects across streams as [§FS-errors.5](FS-errors.md#5-json-format) specifies: graph findings as NDJSON on stdout, run-level warnings such as empty scans and line-less mid-walk read failures on stderr. Launch-time CLI failures stay raw `error:` text on stderr even when `--format=json` was requested.

### 1.1 Value mismatch

A value mismatch uses `code: "value-mismatch"`, the binding as its primary `path`/`line`, and the declaration as the one `sites` entry. The text embedded in `message` includes that declaration's `path:line`; the object and text line therefore carry the same actionable content ([§FS-values.5.2](FS-values.md#52-fixed-value-errors)).

## 2. Empty JSON check

Command:

```text
grund check <empty-repo> --format json
```

If the scan reads no scannable files, exit `0`, stdout empty, stderr contains one JSON warning object:

```json
{"severity":"warning","path":null,"line":null,"code":"empty-scan","message":"nothing to scan — no files under `<path>` matched grund's extensions (md, rs, go, java, kt, ts, tsx, js, py, c, cpp, swift, scala, rb, php, cs, lisp, scm, clj, sql, hs, lhs, lua, ada, adb, ads).","sites":null}
```

A clean non-empty JSON check emits nothing on stdout and nothing on stderr. There is no success object.

## 3. Text report ordering

Text diagnostics are grouped by channel as
[§FS-errors.4](FS-errors.md#4-determinism) fixes: every error, then every
warning, then opt-in suggestions, each group sorted bytewise by `(path, line,
message)`. Each located line keeps its jump-friendly location prefix and places
the channel immediately after it. Example stdout for a failing `check` whose
warning sorts first by path:

```text
src/z-last.rs:1: error: unknown reference FS-999-missing
docs/functional-spec/FS-001-alpha.md:3: warning: declared but never cited: FS-001-alpha
```

stderr is empty for ordinary graph findings.

Value findings join their fixed severity group. Home JSON sources are read in normalized bytewise path order before their declaration and binding findings are sorted into the report ([§FS-values.2.2](FS-values.md#22-json-declarations-from-the-kind-home)).

JSON does not inherit the text grouping. The same two findings remain in the
existing global bytewise location order, and every object retains its existing
bytes and shape:

```json
{"severity":"warning","path":"docs/functional-spec/FS-001-alpha.md","line":3,"code":"unused-declaration","message":"declared but never cited: FS-001-alpha","sites":null}
{"severity":"error","path":"src/z-last.rs","line":1,"code":"dangling","message":"unknown reference FS-999-missing","sites":null}
```

## 4. `show --format=json`

A successful `show --format=json` emits exactly one JSON object on stdout and nothing on stderr:

```json
{"id":"FS-001-alpha","section":"1","body":"## 1. First\n\nFirst body.\n","path":"docs/functional-spec/FS-001-alpha.md","line":5}
```

Fields:

- `id` is the resolved declaration ID.
- `section` is the requested section path as a string, or `null` for a whole declaration.
- `body` is exactly the text-mode body, including trailing newline when text mode would print one.
- `path` and `line` point at the declaration or selected section start.
- `sections`, present for `show --toc --format=json`, is the ordered section-map slice as objects with `path`, `title`, and `depth`.

`show --toc --format=json` example:

```json
{"id":"FS-001-alpha","section":null,"body":"Alpha overview.\n\n## 1. First\n### 1.1 Child\n","path":"docs/functional-spec/FS-001-alpha.md","line":1,"sections":[{"path":"1","title":"First","depth":1},{"path":"1.1","title":"Child","depth":2}]}
```

`show --brief --format=json` keeps the normal `show` object shape and narrows only `body`:

```json
{"id":"FS-001-alpha","section":null,"body":"# FS-001-alpha: Alpha\n\nAlpha overview.\n","path":"docs/functional-spec/FS-001-alpha.md","line":1}
```

Failed queries emit one diagnostic object on stderr and leave stdout empty; launch-time errors stay raw `error:` text.

### 4.1 `show --batch --format=json`

Batch output is NDJSON with exactly one envelope per query. Object keys have the
fixed order shown here:

```json
{"query":{"id":"FS-login","section":"1"},"ok":true,"result":{"id":"FS-login","section":"1","body":"## 1. Login\n","path":"docs/functional-spec/FS-login.md","line":5},"error":null}
{"query":{"id":"FS-missing","section":null},"ok":false,"result":null,"error":{"severity":"error","path":null,"line":null,"code":"not-found","message":"ID not found: FS-missing","sites":null}}
```

`query.id` preserves the caller's spelling for explicit input and carries the
generated local-or-qualified spelling for `--all`; `query.section` is the
explicit or generated section string, or `null`. A success places the unchanged
current single-show JSON object in `result` and sets `error` to `null`. A failed
query sets `result` to `null` and places the unchanged current diagnostic object
in `error`. Every envelope is on stdout in explicit-input or exhaustive order;
stderr is empty for per-query failures. Run-level failures emit no envelopes
([§FS-errors.5](FS-errors.md#5-json-format)).

The batch-only diagnostic for an unknown alias uses code `unknown-project`, the
same stable code as citation resolution, and the single-show message without its
CLI `error:` prefix (including the `known aliases:` or standalone `note:` line).

### 4.2 E2E cases

For an E2E case, `show --format=json` uses the E2E manifest shape from [§FS-show.2.4](FS-show.md#24-e2e-cases), whose fields, `path` among them, [§FS-show.2.4.2](FS-show.md#242-the-manifest-as-json) defines.

```json
{"id":"E2E-login","kind":"E2E","path":"e2e/cases/login","args":[],"expected_exit":0,"fixtures":["expected.exit","expected.stdout","repo/docs/functional-spec/FS-001-login.md"]}
```

### 4.3 JSON value declarations

For a JSON value declaration, every read mode's `body` is the exact available member or element source slice and `path`/`line` is that slice's exact span start; no Markdown heading or title is synthesized ([§FS-values.6.2](FS-values.md#62-json-declarations-in-the-catalog)).

## 5. `list --format=json`

`list --format=json` emits one declaration object per line, sorted by `(id, path, line)`:

```json
{"id":"AR-001-auth","kind":"AR","path":"docs/architecture/AR-001-auth.md","line":1,"title":"The auth module","stub":false,"defines":null,"refs":0,"duplicate":false}
{"id":"FS-001-login","kind":"FS","path":"docs/functional-spec/FS-001-login.md","line":1,"title":"User can log in","stub":false,"defines":null,"refs":2,"duplicate":false}
```

Fields:

- `id`, `kind`, `path`, `line`, and `title` identify the declaration.
- `stub` is true only for a broken stub: a healthy docs stub collapses into its inline declaration and gets no row of its own ([§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical)).
- `defines` is the target path for a stub, otherwise `null`.
- `refs` is the number of citations that resolve to this ID.
- `duplicate` is true when this ID has more than one independent declaration home.

`list --summary --format=json` emits one kind summary object per line, in configured kind order after `--kind` / `--unused` filtering:

```json
{"kind":"FS","title":"What: behavior, requirements, and constraints","home":"requirements.md","count":2}
{"kind":"AR","title":"How: high-level implementation, structure, and design","home":"docs/architecture","count":1}
```

`list --size --format=json` emits point-site rows instead (§5.2).

### 5.1 `refs --format=json`

`refs --format=json` emits one citation object per line. With `--summary`, it emits one file summary object per line instead:

```json
{"path":"docs/functional-spec/FS-002-beta.md","count":3,"lines":[3,5]}
```

`count` is the number of citation sites in the file; `lines` is the sorted, de-duplicated set of 1-indexed source lines containing those sites.

### 5.2 `list --size --format=json`

`list --size --format=json` emits one object per declaration and citable section site, in the size-row order defined by [§FS-list.3.4](FS-list.md#34---size--per-point-lead-and-full-body-measurements):

```json
{"id":"FS-001-login","section":null,"kind":"FS","path":"docs/functional-spec/FS-001-login.md","line":1,"stub":false,"defines":null,"duplicate":false,"lead_lines":2,"full_lines":5,"lead_words":7,"full_words":18,"lead_bytes":41,"full_bytes":109}
{"id":"FS-001-login","section":"1","kind":"FS","path":"docs/functional-spec/FS-001-login.md","line":6,"stub":false,"defines":null,"duplicate":false,"lead_lines":3,"full_lines":3,"lead_words":9,"full_words":9,"lead_bytes":57,"full_bytes":57}
```

The fixed prefix fields are `id`, `section`, `kind`, `path`, `line`, `stub`, `defines`, and `duplicate`. A workspace row begins with `project`; its `id` remains workspace-qualified ([§FS-workspace.8.3](FS-workspace.md#83-grund-list)). Selected unit pairs follow in caller order as `lead_<unit>`, `full_<unit>`; bare `--size` therefore emits `lines`, `words`, then `bytes`. Unselected pairs are absent. A broken stub uses `null` for each selected measurement. These are point-site records rather than declaration-summary records, so `title` and `refs` are absent.

## 6. CLI and config failures

CLI-level failures use raw text on stderr, not JSON, because the command did not reach its data-producing phase:

```text
error: grund.toml:2: unknown config key `strcit`
```

This config validation example exits `1` for `grund config validate` and `2`
when the same invalid config blocks another subcommand.

### 6.1 `refs` resolver rejections across 0.14.0 and 0.15.0

During grund 0.14.0, `refs` preserves the former CLI-level classification for resolver-rejected operands and warns about the 0.15.0 change ([§FS-refs.4](FS-refs.md#4-exit-codes)); at 0.15.0 the rejection becomes a failed query with exit `1`. §6.1.1 and §6.1.2 give an invalid ID's exact output at each stage; an ambiguous number-only shorthand follows the same stages, without a hint.

#### 6.1.1 In 0.14.0

The invalid-ID text and JSON invocations both write exactly:

```text
error: invalid ID `FS-bar`
hint: this repo's [id] format is `{kind}-{number}-{slug}` (run `grund config show`); `grund list` shows the IDs that exist
warning: `grund refs` invalid IDs and ambiguous number-only shorthands currently exit 2; they will exit 1 (failed query) in grund 0.15.0
```

The `refs` example exits `2` and leaves stdout empty in 0.14.0.

#### 6.1.2 From 0.15.0

The same example becomes the failed-query text shape below, exit `1`; JSON emits the one object shown and no hint, warning, or stdout:

```text
invalid ID `FS-bar`
hint: this repo's [id] format is `{kind}-{number}-{slug}` (run `grund config show`); `grund list` shows the IDs that exist
```

```json
{"severity":"error","path":null,"line":null,"code":"invalid-id","message":"invalid ID `FS-bar`","sites":null}
```

An ambiguous number-only shorthand's 0.15.0 JSON code is `ambiguous` and `sites` is `null`.

## 7. Stream matrix

For each case above, §7.1 lists what reaches stdout and stderr and the exit code.

### 7.1 The matrix

| Case | stdout | stderr | Exit |
|------|--------|--------|------|
| clean `check` text | `success\n` | empty | `0` |
| clean `check --format=json` | empty | empty | `0` |
| empty scan JSON | empty | one warning diagnostic object | `0` |
| graph findings text | located finding lines | empty unless run-level diagnostics exist | `1` |
| graph findings JSON | diagnostic NDJSON | line-less run diagnostics only | `1` |
| successful `show --format=json` | one result object | empty | `0` |
| failed `show --format=json` query | empty | one diagnostic object | `1` |
| `refs` resolver rejection, 0.14.0 text or JSON | empty | raw `error:`, optional invalid-format hint, then exact migration warning | `2` |
| `refs` resolver rejection, 0.15.0 text | empty | bare failure; invalid format alone adds a hint | `1` |
| `refs` resolver rejection, 0.15.0 JSON | empty | one `invalid-id` or `ambiguous` diagnostic object; no hint | `1` |
| bad flag / malformed CLI | empty | raw `error:` text | `2` |
| invalid config during `config validate` | empty | raw `error: <path>:<line>:` text | `1` |
| invalid config blocking another command | empty | raw `error: <path>:<line>:` text | `2` |
| semantic value finding, text | located finding lines | empty | `1` |
| semantic value finding, JSON | diagnostic NDJSON with declaration `sites` | empty | `1` |
| home JSON that [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete | partial findings if any | incomplete-scan diagnostic | `2` |

# FS-show: grund reads a single declaration body by ID

The `show` subcommand prints slices of declaration bodies by ID. Its established
single-coordinate form is the cheap way to pull one grounded fact into context;
its opt-in batch forms reuse one loaded workspace for an ordered query stream or
for every resolvable coordinate. Serves
[§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible),
[§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file),
and [§GOAL-fast-feedback.1](../goals.md#1-performance-targets).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand, canonical form),
[§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope,
workspace, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, fetcher,
snapshot).

- **slice** — How much of a body a read returns — `--brief`, the default, `--toc`, `--full`, and
  the section forms. The slices are strictly nested, so escalating is one more flag.
- **brief** — The cheapest slice: the heading line and the first paragraph, the hover preview.
- **section map** — What `--toc` returns: the nested map of the coordinates a body records,
  without their prose.
- **batch query** — One `--batch` invocation answering an ordered stream of coordinates, or
  every coordinate in scope, from a single load.

## 1. Inputs

```
grund [show] <ID> [<path>] [--section <s>] [--brief | --toc | --full] [--format <text|md|json>]
grund show --batch [<path>] --format=json [--brief | --toc | --full] [--path <path>]
grund show --batch --all [<path>] --format=json [--brief | --toc | --full] [--path <path>]
```

The first form reads one coordinate: `<ID>` names it ([§FS-show.1.1](FS-show.md#11-id), [§FS-show.1.2](FS-show.md#12-the-number-only-shorthand)), `show` itself
may be omitted ([§FS-show.1.3](FS-show.md#13-show-is-optional)), `<path>` picks the tree ([§FS-show.1.4](FS-show.md#14-path-and---path)), `--section` or the dotted
form picks a section ([§FS-show.1.5](FS-show.md#15---section-s)), one slice flag picks how much ([§FS-show.1.6](FS-show.md#16-how-much---brief-the-default---toc---full)), and `--format`
picks the shape ([§FS-show.1.7](FS-show.md#17---format)). The two `--batch` forms answer many coordinates from one
load: an explicit query stream ([§FS-show.1.8](FS-show.md#18---batch-an-explicit-query-stream)) or every coordinate in scope ([§FS-show.1.9](FS-show.md#19---batch---all-every-coordinate-in-scope)).

### 1.1 `<ID>`

`<ID>` is the full ID without the marker (e.g. `FS-check`). It may include an inline section (`FS-check.3.1`); the dotted form uses the configured `[id] section_separator`. Beyond the kind's grammar it accepts an exact off-grammar ID the catalog retains ([§FS-show.1.1.1](FS-show.md#111-exact-off-grammar-ids)); the catalog-prefix ambiguity fails rather than guesses ([§FS-show.1.1.2](FS-show.md#112-where-the-id-ends-and-the-section-begins)); and a missing fetch-backed snapshot is a failed offline query ([§FS-show.1.1.3](FS-show.md#113-a-missing-fetched-snapshot)).

#### 1.1.1 Exact off-grammar IDs

Parsing first accepts the named kind's effective grammar, then exact written IDs retained in that project's shared catalog for read compatibility ([§FS-config.3.2](FS-config.md#32-id--id-grammar)); an off-grammar string with no exact declaration remains invalid. All four whole-declaration slices and both section forms apply unchanged to an exact off-grammar declaration.

#### 1.1.2 Where the ID ends and the section begins

The one inline ambiguity a loaded config leaves is [§FS-config.3.2](FS-config.md#32-id--id-grammar)'s catalog-prefix case — competing valid readings of where the ID ends and the section begins — which fails rather than guesses.

#### 1.1.3 A missing fetched snapshot

A missing fetch-backed snapshot is still a failed offline query; `show` never invokes its fetcher ([§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service)).

### 1.2 The number-only shorthand

In a kind whose effective format carries both `{number}` and `{slug}`, the number-only shorthand is also accepted — `grund FS-042` and `grund FS-042.1` read `FS-042-user-login` ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)). Nothing is persisted by a query, so the shorthand is convenience here rather than the error it is in a file under the default `[reference] shorthand = "canonical"` ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)); shorthand, duplicate, exact-ID, and section interpretations obey [§FS-config.3.2](FS-config.md#32-id--id-grammar)'s fail-rather-than-guess precedence.

### 1.3 `show` is optional

`show` is optional when the first non-flag word is the ID: `grund FS-check`, `grund FS-check --toc`, and `grund --toc FS-check` are byte-for-byte equivalent to their explicit `grund show …` forms ([§FS-cli.1](FS-cli.md#1-the-default-subcommand)).

### 1.4 `<path>` and `--path`

`<path>` is the directory or file whose tree is scanned to resolve the ID. Defaults to `.`. Discovery is the same as every other subcommand (walk up to a `grund.toml`, else defaults — [§FS-config.1](FS-config.md#1-file-location-and-discovery)). `--path <path>` is an accepted alias for scripts that prefer to pass it as a flag; the two forms are equivalent.

### 1.5 `--section <s>`

`--section <s>` is an alternative way to specify a section path (`3.1`). Mutually exclusive with the dotted form. Composes with each `--brief` / `--toc` / `--full` slice exactly as the dotted form does ([§FS-show.2.2](FS-show.md#22-section)).

### 1.6 How much: `--brief`, the default, `--toc`, `--full`

`--brief`, `--toc`, and `--full` are mutually exclusive — each picks one rung on the "how much" ladder: title + 1 paragraph → lead prose → lead + section map → full body. The rungs' bodies are strictly nested (each contains the previous), so escalating is always one more flag; in `text` only `--brief` keeps the whole-declaration H1 ([§FS-show.2.1.1](FS-show.md#211-brief---brief), [§FS-show.3.1](FS-show.md#31-format-variants)), so for a one-paragraph lead it is longer than the default and not contained in it.

- `--brief` — the cheapest "what is this about" view, a hover-preview slice ([§FS-show.2.1.1](FS-show.md#211-brief---brief)).
- (no flag, the default) — the lead, cut at the first *citable* point, so an agent landing on a bare `§<ID>` reads enough to know whether to fetch a deeper section ([§FS-show.2.1](FS-show.md#21-whole-declaration-default)).
- `--toc` — the move when the next step is `grund <ID>.<sec>` and the section number needs to be chosen ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)).
- `--full` — print the entire body: heading down to the end of its body span ([§FS-show.2.1.2.1](FS-show.md#2121-the-map-is-the-declarations-body-span)), all subsections recursively included. The escalation when narrower slices are not enough ([§FS-show.2.1.3](FS-show.md#213-full-body---full)).

### 1.7 `--format`

`--format` — output shape ([§FS-show.3.1](FS-show.md#31-format-variants)); defaults to `text`.

### 1.8 `--batch`: an explicit query stream

`--batch` is available only on the explicit `show` subcommand and requires
`--format=json`. Without `--all`, it reads one query object per non-empty stdin
line: `{"id":"api/FS-login","section":"3.1"}`. `id` is required and must be a
string; `section` is optional and must be a string or `null`; unknown fields,
malformed JSON, and any other shape are batch-input errors. The `id` field
accepts every local, qualified, shorthand, and inline-section spelling the
single-coordinate form accepts. An explicit non-null `section` and an inline
section in `id` form a valid record whose query fails rather than malformed
input ([§FS-show.2.6.3](FS-show.md#263-query-failures-and-run-level-failures)). The whole input is validated before configuration discovery or
scanning.

### 1.9 `--batch --all`: every coordinate in scope

`--batch --all` reads no stdin and discovers its query set from the selected
scope ([§FS-show.2.6.2](FS-show.md#262-exhaustive-generation)). `--all` requires `--batch`; stdin supplied with `--all` is not a
query source. Both batch forms use one invocation-level slice mode. `--section`
is rejected in batch mode because each explicit record owns its section and the
exhaustive form generates sections. `<path>` and `--path` retain their existing
equivalence and mutual exclusion.

## 2. Behavior

A fetched snapshot is an ordinary declaration. Every slice below returns its
committed body verbatim through the same scanner spans as an authored Markdown
declaration; no query checks freshness or contacts its integration.

Opted-in JSON value declarations are members of the same catalog. For one, `show` returns the exact source member slice for the ID or exact array-element slice for a numbered section; `--brief`, default, `--toc`, and `--full` all collapse to that available slice and never synthesize Markdown ([§FS-values.6](FS-values.md#6-shared-catalog-consumers)).

### 2.1 Whole declaration (default)

`grund FS-check` prints the *lead* — the prose between the declaration heading and the first child citable section heading (`## 1. ...`, or `## goals: ...` when named sections are enabled). The opening heading is omitted in `text` format and included in `md`. A named heading is a citable point and cuts its parent's lead exactly as a numbered heading does; a plain heading remains prose and does not cut it. This is the new default: a 1–2 paragraph slice that names what the declaration is about, without paying for the whole body. Decided in [§DF-show-default-token-cheap](../decisions/functional/DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in).

A declaration with no lead prints nothing ([§FS-show.2.1.4](FS-show.md#214-a-declaration-with-no-lead-prints-nothing)); a selected section gets the same cut one level down ([§FS-show.2.1.5](FS-show.md#215-a-sections-lead)).

#### 2.1.1 Brief (`--brief`)

`grund --brief FS-check` prints the declaration heading and only the first blank-line-separated paragraph below it: a hover-preview slice, the narrowest body slice of all — though because it alone keeps the H1 in `text` (below), for a one-paragraph lead it is longer than the default. "First paragraph" means the first non-blank run of lines after the heading, terminated by the first blank line, the first child heading, or the end of the body — whichever comes first.

`--brief` always includes the heading line so the slice is self-labeled, regardless of `text` vs `md`. This is the one mode where the `text` rule of "omit the H1" yields ([§FS-show.3.1](FS-show.md#31-format-variants)): a single paragraph with no title is unreadable for the hover-preview use case. In `text` the heading is rendered as written, with the leading `#` prefixes preserved.

With no lead prose, `--brief` prints the heading line alone and exits `0`; with `--section` or the dotted form it prints the section heading and its first paragraph, or the heading alone when a sub-subsection opens the section ([§FS-show.2.1.1.1](FS-show.md#2111-with-no-lead-the-heading-alone)).

##### 2.1.1.1 With no lead, the heading alone

If the declaration has no lead prose (opens directly with `## 1. ...`), `--brief` prints just the heading line and exits `0`. With `--section` / the dotted form, `--brief` prints the section heading and the first paragraph under it; if the section opens directly with a sub-subsection, just the section heading is printed.

#### 2.1.2 Section map (`--toc`)

`grund --toc FS-check` prints the default lead ([§FS-show.2.1](FS-show.md#21-whole-declaration-default)), then a blank line, then every citable section heading in the declaration body, one per line, in document order, each at the depth and in the complete form it was written (`## 1. Inputs`, `## goals: Goals`, `### goals.performance: Performance`, …). No section bodies. The heading lines are emitted verbatim — the same bytes `--full` would show for those lines — so the coordinate the reader needs is right there to feed back into `grund FS-check.<path>`. No generated summary, ever: `--toc` is a structural slice, as deterministic as the default ([§FS-errors.4](FS-errors.md#4-determinism)). A whole-declaration TOC still lists every claimant of a duplicate named coordinate; selecting that coordinate refuses as ambiguous ([§FS-show.2.2.2.3](FS-show.md#2223-the-whole-declaration-map-still-lists-both)).

Which headings the map holds is [§FS-show.2.1.2.1](FS-show.md#2121-the-map-is-the-declarations-body-span); what it prints when the lead or the map is empty is [§FS-show.2.1.2.2](FS-show.md#2122-an-empty-lead-an-empty-map); a selected section's map is [§FS-show.2.1.2.3](FS-show.md#2123-a-selected-sections-map).

##### 2.1.2.1 The map is the declaration's body span

A coordinate enters the section map only when its heading lies inside that
declaration's existing body span. The span, not the most recently scanned
declaration, owns the coordinate: a same-or-higher Markdown heading, the end of
an inline source doc-comment or docstring, the next declaration in a shared
comment block, and a stub's single declaration line each end ownership. A
deeper numeric or enabled named heading before that boundary remains a section;
one after it is absent from the map and is reported by
[§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration). Fenced Markdown
pseudo-headings remain content under [§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example). This boundary does not make an
otherwise legal plain heading inside a body an error; that separate policy is
outside this contract.

##### 2.1.2.2 An empty lead, an empty map

If the lead is empty (`## 1.` or `## goals:` opens the body), the leading blank line is omitted — the output is the section map only. If the body has no citable headings (a short declaration that is all lead prose, an E2E manifest), `--toc` prints the default and nothing else. If both are empty, `--toc` prints **nothing** and exits `0`.

##### 2.1.2.3 A selected section's map

`grund --toc FS-check.3.1` restricts the map to headings **nested under** the selected section: it prints `### 3.1`'s lead, then a blank line, then `#### 3.1.1 …`, `#### 3.1.2 …`, and so on, stopping at the next sibling-or-shallower heading. A selected section with no nested headings is just its lead — i.e. behaves like the default. A section that does not exist is still a `section not found` error.

#### 2.1.3 Full body (`--full`)

`grund --full FS-check` prints from the heading of `FS-check` to the end of its body span ([§FS-show.2.1.2.1](FS-show.md#2121-the-map-is-the-declarations-body-span)): in Markdown, the start of the next same-or-higher heading, whether or not it declares an ID (or end of file). Every subsection and sub-subsection body is included. With `--section` / the dotted form, `--full` prints the selected section's heading and full body — the same slice [§FS-show.2.2](FS-show.md#22-section) defines. The opening heading is omitted in `text` and included in `md`, as in the default.

`--full` is the escalation path when `--brief`, the default, and `--toc` are not enough. It is also the way to recover today's pre-[§DF-show-default-token-cheap](../decisions/functional/DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in) behavior: use `grund <ID> --full`.

#### 2.1.4 A declaration with no lead prints nothing

If a declaration has no lead paragraph (its body opens directly with `## 1. ...`), the default prints **nothing** and exits `0`. This is not an error: the declaration simply has no lead. Callers (IDE hovers, agents) can detect this case by the empty output and escalate to `--toc` or `--full`. We do not auto-fall-back; the caller knows what it wants.

#### 2.1.5 A section's lead

`grund FS-check.3.1` applies the same cut one level down. It prints the selected section heading (`### 3.1 ...`), kept in `text` as in `md` ([§FS-show.2.2](FS-show.md#22-section)), and the prose between that heading and the first *child* heading (`#### 3.1.1 ...`). If the section opens directly with a sub-subsection, the output is just the section heading line. A section that does not exist is still a `section not found` error.

### 2.2 Section

`grund FS-check.3.1` selects a section; in an opted-in repository, `grund FS-plan.goals.performance` and `grund FS-plan --section goals.performance` select the same named section. The flag determines how much of it is printed:

- `--brief`: section heading + first paragraph.
- (default): section heading + prose down to the first child heading.
- `--toc`: section heading + lead + nested heading map.
- `--full`: section heading + full body (everything down to the next sibling-or-shallower heading; nested deeper headings included).

The selected section heading is printed verbatim in all four modes — `text` strips only the whole-declaration H1, not section headings ([§FS-show.3.1](FS-show.md#31-format-variants)). For `--brief`, the section heading is the slice's self-label. Named paths, including `name.number`, compose with default, brief, TOC, full, and `--section` exactly as numeric paths do. Arbitrary nesting depth is supported per [§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth). Every surface that names a section reads the same map ([§FS-show.2.2.3](FS-show.md#223-every-surface-reads-the-same-section-map)).

#### 2.2.1 Ambiguous ID

If an ID has more than one home — the duplicate-declaration error from [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) — `show` does not pick one. A stub paired with the inline declaration it points at is *one* home, not two; ambiguity means two or more independent declarations remain after that pairing collapses. When ambiguous, `show` exits 1 with a single bare stderr line — no `<path>:<line>:` prefix ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)):

```
ambiguous ID: <ID> (declared at <path>:<line>, <path>:<line>[, ...])
```

Sites are listed in lexicographic `path:line` order so the message is stable across runs. The repo must be fixed (run `grund check` first) before `show` will return a body. With `--format=json`, those same sites travel in the diagnostic's `sites` field, `[{ path, line }]` in the same order ([§FS-errors.5](FS-errors.md#5-json-format)).

This shape matches the bare-message form used for `ID not found` and `section not found` ([§FS-show.3](FS-show.md#3-outputs)): all three are queries that found something other than exactly one body. An ambiguous number-only shorthand fails the same way but names candidates rather than sites ([§FS-show.2.2.1.1](FS-show.md#2211-an-ambiguous-shorthand-names-its-candidates)).

##### 2.2.1.1 An ambiguous shorthand names its candidates

A number-only shorthand argument ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) fails the same way for a different reason — not one ID with two homes, but one abbreviation naming two IDs — so it names the **candidates** rather than the sites:

```
ambiguous ID: FS-042 (matches FS-042-user-login, FS-042-user-logout)
```

Candidates are listed in ID order, and the repo needs no fixing: the caller does, by passing one of the full IDs. Nothing is guessed at ([§DF-number-only-citation-shorthand.2.7](../decisions/functional/DF-number-only-citation-shorthand.md#27-ambiguity-is-reported-never-guessed), [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)).

#### 2.2.2 Ambiguous section

The same refusal one level down. If two citable headings inside the selected declaration claim the requested dotted path — numeric or named, under the duplicate-section error of [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path) — `show` does not pick one either:

```
ambiguous section: FS-001-login.1 (declared at docs/functional-spec/FS-001-login.md:5, docs/functional-spec/FS-001-login.md:9)
```

Sites are in `path:line` order, as in [§FS-show.2.2.1](FS-show.md#221-ambiguous-id), and the exit is `1` with the bare stderr line of [§FS-errors.2.3](FS-errors.md#23-bare-query-failure). The repo must be fixed before `show` will return a body. With `--format=json`, the same sites travel in the diagnostic's `sites` field too ([§FS-errors.5](FS-errors.md#5-json-format)).

What this replaces is worse than a pick: the reader used to get *both* headings and both bodies concatenated into one slice, a body no heading in the file spans ([§DF-duplicate-section-path.1](../decisions/functional/DF-duplicate-section-path.md#1-context)).

The failure has its own code ([§FS-show.2.2.2.1](FS-show.md#2221-its-own-code-ambiguous-section)) and refuses exactly what `check` reports ([§FS-show.2.2.2.2](FS-show.md#2222-the-headings-check-counts)); the whole-declaration map still lists both headings ([§FS-show.2.2.2.3](FS-show.md#2223-the-whole-declaration-map-still-lists-both)), and only the requested path can collide ([§FS-show.2.2.2.4](FS-show.md#2224-only-the-requested-path-can-collide)).

##### 2.2.2.1 Its own code, `ambiguous-section`

The code is `ambiguous-section`, not [§FS-show.2.2.1](FS-show.md#221-ambiguous-id)'s `ambiguous` ([§FS-distribution.3.0](FS-distribution.md#30-language-neutral-data-shapes)). The two failures need different edits — one ID with two homes is fixed in whichever file should not have declared it, one declaration with two `1.` headings is fixed by renumbering inside it — and the check side already spells that difference `duplicate` versus `duplicate-section` ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)). Reusing one code would leave a JSON consumer parsing the message prose to tell them apart, the cost [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only) refused to pay for its own four rules. Nothing regresses by adding it: before this rule the query returned a body and exit `0`, so no consumer ever saw `ambiguous` here to filter on.

##### 2.2.2.2 The headings `check` counts

Which headings count is [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)'s question, answered once: `show` refuses exactly the coordinates that rule reports, from the same recorded section set, so no coordinate is clean in `check` and unresolvable in `show`. For a stub ([§FS-show.2.3.4](FS-show.md#234-broken-stub)) that set is the **inline home's** — the file the query reads — never the stub's own prose.

##### 2.2.2.3 The whole-declaration map still lists both

`--toc` over the **whole declaration** ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)) is the exception and still lists both heading lines — it is a map of what is written, and seeing the collision is the point. `grund FS-001-login.1 --toc` is not that map: it selects the ambiguous coordinate, so it refuses like every other slice. The exemption is for the query that asks what the declaration contains, not for the one that asks which of two headings section `1` is.

##### 2.2.2.4 Only the requested path can collide

A duplicate elsewhere in the declaration is not this error: only a collision on the **requested** path can make the query ambiguous, so `grund FS-001-login.2` answers normally while `grund FS-001-login.1` refuses. `grund check` reports the file either way.

#### 2.2.3 Every surface reads the same section map

Every coordinate-bearing surface reads the same body-local map ([§FS-show.2.1.2.1](FS-show.md#2121-the-map-is-the-declarations-body-span)):
direct and batch `show`, exhaustive batch generation, citation and value
resolution, `refs`, completion, list/size output, duplicate detection, and LSP
navigation, references, highlights, and hover counts. A heading rejected by
[§FS-show.2.1.2.1](FS-show.md#2121-the-map-is-the-declarations-body-span) can therefore neither resolve nor be suggested, listed, measured, navigated to,
validated as an embedded-value root, or treated as a duplicate claimant. A
query for its coordinate has the ordinary `section not found` result and hint
from [§FS-show.3](FS-show.md#3-outputs), emits none of the outside heading's body, and never substitutes the
located `check` finding for query semantics.

### 2.3 Inline declarations in code and doc-comments

When the ID's home is in code — whether discovered directly, enrolled by its kind's canonical index link ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)), or paired with a [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) stub — `show` extracts the comment block surrounding the inline declaration, strips comment markers, and prints the resulting prose. Index enrollment creates no declaration and therefore changes no query resolution. The same section logic applies — and so do the `--brief` / (default) / `--toc` / `--full` slices, computed over the stripped block exactly as over a `.md` body (the lead is what precedes the first citable heading inside the comment; the section map is the citable headings recorded within it, per [§FS-show.2.3.3](FS-show.md#233-section-selection-inside-a-doc-comment)).

A code-resident declaration is written as `<comment-marker> <ID>: <title>` (or bare `<ID>: <title>` inside a Python docstring), with no markdown `#` prefix. Decided in [§DF-code-declarations-drop-hash](../decisions/functional/DF-code-declarations-drop-hash.md#df-code-declarations-drop-hash-code-resident-declarations-may-drop-the--prefix).

The doc-comment forms are [§FS-show.2.3.5](FS-show.md#235-the-doc-comment-forms), and one doc-comment may hold several declarations ([§FS-show.2.3.6](FS-show.md#236-several-declarations-in-one-doc-comment)).

#### 2.3.1 What counts as the "comment block"

Extraction is precisely defined so that the implementation has no freedom and the same input produces the same output across editor, CLI, and binding callers.

A declaration is found on a "declaration line" — a line that matches the declaration regex from [AR-scanner.2.1](../architecture/AR-scanner.md#21-declaration-detection) *and* sits inside a comment or docstring. The block surrounding it runs from an open boundary ([§FS-show.2.3.1.1](FS-show.md#2311-find-the-open-boundary)) to a close boundary ([§FS-show.2.3.1.2](FS-show.md#2312-find-the-close-boundary)), and ends early at any other declaration line ([§FS-show.2.3.1.3](FS-show.md#2313-terminate-early-on-another-declaration)).

##### 2.3.1.1 Find the open boundary

Walk **backwards** from the declaration line over consecutive lines that are part of the same comment construct:

- For line-style comments (`//`, `///`, `//!`, `#`, `;`, `--`): consecutive lines whose first non-whitespace character matches the same comment prefix family. A blank line ends the block. A line whose first non-whitespace character is not a comment marker ends the block.
- For block-style comments (`/* … */`, `/** … */`): walk backward until the opener is found (`/*` or `/**`). The opener line itself is part of the block.
- For Python triple-quoted docstrings: walk backward until the opening `"""` (or `'''`). The opener line is part of the block.

##### 2.3.1.2 Find the close boundary

Walk **forwards** from the declaration line by the symmetric rules:

- Line-style: until a blank line or a non-comment line.
- Block-style: until the closing `*/`. The closer line is part of the block.
- Python docstring: until the matching `"""` or `'''`. The closer line is part of the block.

##### 2.3.1.3 Terminate early on another declaration

Walking in **either direction**, if another declaration line of any ID is encountered, the block ends at the line before it. This is what allows two adjacent inline declarations to live in the same comment without bleeding into each other — backward termination keeps a later declaration's block from absorbing the previous declaration's tail; forward termination keeps the previous declaration's block from absorbing the next declaration's head.

#### 2.3.2 Stripping comment markers

After the block is selected, comment markers are removed line-by-line so the output is plain prose:

- Leading whitespace is preserved up to the comment marker, then the marker is dropped, then a single space following the marker is dropped if present. The remainder of the line is kept verbatim.
- For block-style continuation lines, a leading ` * ` (with surrounding spaces) is removed if present. Lines that do not have it are kept as-is.
- For Python docstrings, no marker is stripped — docstring content is plain text already; only the surrounding `"""` lines are skipped.
- Trailing comment-close markers (`*/`) on their own line are dropped entirely.
- Blank lines inside the block are preserved.

The result is the markdown that the declaration's author wrote, identical to what would have lived in a `.md` file had the spec been doc-resident instead of inline. This is the property that makes [§FS-show.2.3](FS-show.md#23-inline-declarations-in-code-and-doc-comments) round-trip-stable across the in-docs and in-code homes.

#### 2.3.3 Section selection inside a doc-comment

Section selection (`AR-<event-bus>.2`) works the same way inside a doc-comment as inside a markdown file: the scanner records the numbered subsection headings declared within the doc-comment block and `show` slices to the requested section. Section depth is measured relative to the declaration's heading level exactly as in markdown ([AR-scanner.2.2](../architecture/AR-scanner.md#22-section-detection)) — an `AR-<event-bus>:` declaration inside a `///` block is "level 1", so `## 1.` is a depth-1 section. The comment-stripping pass leaves these headings intact.

#### 2.3.4 Broken stub

If the ID's only home is a stub (`# <ID>: [<text>](<path>)`) whose link is broken — the `<path>` does not exist, or the file at `<path>` contains no inline declaration of `<ID>` (the [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) error) — `show` has no body to extract. It exits `1` with a bare query-failure line ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)), not a `path:line:` finding:

```
broken stub: <ID> (stub at <path>:<line> points at <target>, which does not exist)
broken stub: <ID> (stub at <path>:<line> points at <target>, which contains no inline declaration of <ID>)
```

This is the same "found something other than exactly one body" family as `ID not found` and `ambiguous ID` ([§FS-show.3](FS-show.md#3-outputs)). Run `grund check` to see the error in located form; fix the stub or the target before `show` will return a body.

#### 2.3.5 The doc-comment forms

The scanner recognizes the same doc-comment forms enumerated in [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments) — Javadoc, JSDoc/TSDoc, Doxygen, KDoc, Scaladoc, PHPDoc, Rustdoc (`///`, `//!`, `/** … */`), C# XML doc comments, Go's `// …` doc blocks, Ruby `#` comments, and Python `""" … """` docstrings. This means an architectural spec can live directly in the class-level Javadoc, and `grund AR-<event-bus>` returns the comment-stripped Javadoc lead ([§FS-show.2.3.2](FS-show.md#232-stripping-comment-markers)); the optional LSP server's hover shows the `--toc` slice of the same prose, the lead plus its section map ([§FS-lsp.1.2.1](FS-lsp.md#121-citation-preview)). The stub at `docs/architecture/AR-<event-bus>.md` is a single-line H1 — `# AR-<event-bus>: [<path>](<path>)` — pointing at the file.

#### 2.3.6 Several declarations in one doc-comment

A single doc-comment may declare **multiple** IDs — most usefully an `AR-` and an `FS-` co-located on the same class — and each gets its own body. The scanner ends each declaration's block at the next declaration line in either direction ([§FS-show.2.3.1.3](FS-show.md#2313-terminate-early-on-another-declaration)):

```rust
/// AR-router: In-process event router
///
/// Implements the publish-subscribe contract from §FS-events.
///
/// FS-router-priority: Routes are matched in declared priority order
///
/// Ties broken by registration order; see §DF-router-tiebreak.
pub struct Router { ... }
```

`grund AR-router` returns the first body; `grund FS-router-priority` returns the second. Multi-declaration comments compose with every slice flag (`--brief`, default, `--toc`, `--full`) because each is just a normal declaration the scanner happens to have found in the same doc-comment.

### 2.4 E2E cases

The JSON manifest appends the optional final `kind_title` field from the selected
E2E kind, with the same absent/empty semantics as every successful show object
([§FS-config.3.4.3](FS-config.md#343-title)). Its existing `id`, `kind`, `path`
prefix stays in that order because the installed resolver's E2E fallback reads that
prefix; metadata does not move ahead of `path`. All existing fields and Markdown
rendering retain their meanings.

`grund E2E-<name>` returns the case's manifest ([AR-scanner.6](../architecture/AR-scanner.md#6-e2e-case-declarations)) in three parts:

```
grund <args…>
expected exit: <code>
fixtures:
- <relative path>
- <relative path>
…
```

The first line is the invocation (`grund check` when the case has no `command.args`); then an `expected exit: <code>` line; then a `fixtures:` line followed by one `- <path>` line per file in the case directory, paths relative to that directory, sorted lexicographically — deterministic for a given tree. How each slice prints it is [§FS-show.2.4.1](FS-show.md#241-the-slices-over-a-manifest); its JSON object is [§FS-show.2.4.2](FS-show.md#242-the-manifest-as-json).

#### 2.4.1 The slices over a manifest

`--full` produces this output. The default and `--toc` produce the same output (an E2E manifest has no heading tree, so the default's "lead" *is* the manifest). `--brief` prints only the first line (the invocation). Section paths are not defined for E2E cases (the manifest is not a numbered-heading tree); `grund E2E-<name>.1` is a section-not-found error.

#### 2.4.2 The manifest as JSON

`--format=json` emits a single object `{"id":"E2E-<name>","kind":"E2E","path":"e2e/cases/<name>","args":[…],"expected_exit":<code>,"fixtures":[…]}` — `path` is the case directory under the configured `E2E` home; `e2e/cases/<name>` is the example produced by the conventional configuration that selects that folder. `args` is the parsed `command.args` (empty when there is none), `fixtures` the same sorted relative-path list; `--brief` / `--toc` / default over a case do not change this object (the manifest has no headings or lead prose to slice further).

### 2.5 A heading inside a fenced code block is an example

In a Markdown body, a line inside a fenced block (```` ``` ````, `~~~`) is content, never structure. It does not end a lead (§FS-show.2.1), does not bound a section (§FS-show.2.2), does not appear in a `--toc` map (§FS-show.2.1.2), and does not open or close a declaration — the fence delimiters and everything between them are printed verbatim as part of whatever slice contains them. Verbatim includes the final text/JSON cross-reference pass: a complete citation wrapper inside the fence stays byte-for-byte as authored rather than being flattened (§FS-show.3.2), while wrapper flattening resumes after a valid closer.

This is the carve-out [§FS-check.1.1.5](FS-check.md#115-contexts-read-as-neither-prose-nor-code) already makes for citations, applied to headings and post-slice flattening for the same reason and for one more: the scan bounds a declaration's sections by exactly this rule, so a slice that disagreed would cut a body where the recorded section map says no section starts. The shared grammar recognizes both backtick and tilde fences, their close/resume rules, and an unclosed fence through end of body; a short or wrong-character would-be closer leaves the fence open. A spec whose [§FS-show.1](FS-show.md#1-inputs) opens with a fenced `# FS-001-login: …` example — the shape these documents are written in — would otherwise print three lines and stop.

The rule is Markdown's. Inside a code or docstring comment block ([§FS-show.2.3](FS-show.md#23-inline-declarations-in-code-and-doc-comments)) a fence is not tracked, on either side: the scan does not track it there either, so the two still agree.

### 2.6 Batch resolution

A non-empty explicit batch and an exhaustive batch each load exactly one
workspace context and answer every query from that context. The loader is not
called once per coordinate. An empty explicit stream exits successfully without
loading configuration or scanning. An exhaustive run always performs its one
load; an empty catalog then succeeds with no records.

The operation is additive: the existing one-query core API and every
single-coordinate CLI spelling keep their signatures and behavior. How explicit
queries are answered is [§FS-show.2.6.1](FS-show.md#261-explicit-queries), what the exhaustive form generates is [§FS-show.2.6.2](FS-show.md#262-exhaustive-generation),
and which failures stay inside one query is [§FS-show.2.6.3](FS-show.md#263-query-failures-and-run-level-failures).

#### 2.6.1 Explicit queries

Explicit queries retain input order and duplicates. Every well-formed query is
attempted, even after an earlier query fails. It uses the same project selection,
ID and section resolution, body extraction, cross-reference flattening, and
default/`--brief`/`--toc`/`--full` renderer as single-coordinate `show`; the one
invocation-level mode applies to every record.

#### 2.6.2 Exhaustive generation

The exhaustive form generates one query for every unique declaration and every
recorded legal numeric or named section of that declaration. Generated queries
sort bytewise by their workspace-qualified ID, with the whole declaration first
and its section paths byte-sorted after it. IDs in the current project are
unqualified; IDs in every other loaded project use that project's stable alias.
When a workspace excludes its root and therefore has no current project, every
generated ID is qualified. Duplicate declarations or section claimants generate
one coordinate and let normal resolution report its ambiguity.

#### 2.6.3 Query failures and run-level failures

A well-formed query that names an invalid, missing, or ambiguous ID; a missing or
ambiguous section; a broken stub; an unknown project alias; or both an inline and
explicit section produces that query's failed envelope ([§FS-show.3](FS-show.md#3-outputs)) and does not stop
later records. Malformed input or invocation, configuration failure, and any scan
failure are run-level errors: stdout stays empty, stderr carries the error, and no
query is attempted. In particular, malformed explicit input is diagnosed as
`error: batch input line <N>: <reason>` before the workspace loader is called.

## 3. Outputs

- `0` — printed successfully.
- `1` — ID not found, ambiguous ID (multiple homes — [§FS-show.2.2.1](FS-show.md#221-ambiguous-id)), ambiguous section (two headings claiming the requested path — [§FS-show.2.2.2](FS-show.md#222-ambiguous-section)), broken stub ([§FS-show.2.3.4](FS-show.md#234-broken-stub)), or section not found in declaration.
- `2` — I/O error, or a CLI-level failure that stops the query before it runs: the commonest is a qualified ID naming a project this run does not hold, which exits `2` with `error: unknown project alias` however many segments the path has ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)). An ID the grammar rejects is *not* one of these — `invalid ID` is a failed query, `1` ([§FS-show.3.5.1](FS-show.md#351-an-id-the-grammar-rejects)).

Stdout carries the body (or, with `--format=json`, the result object — one JSON object, never NDJSON, per [§FS-errors.5.1.1](FS-errors.md#511-query-results)). Stderr carries errors. Stdout is empty on error.

Format variants are [§FS-show.3.1](FS-show.md#31-format-variants), link flattening [§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json), batch mode [§FS-show.3.3](FS-show.md#33-batch-mode), what a failed query prints [§FS-show.3.4](FS-show.md#34-what-a-failed-query-prints), and each failure's hint [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure).

### 3.1 Format variants

`show` prints in one of three formats: `text`, the default ([§FS-show.3.1.1](FS-show.md#311-text)), `md` ([§FS-show.3.1.2](FS-show.md#312-md)), and `json` ([§FS-show.3.1.3](FS-show.md#313-json)). Verbose `show --format=json` examples, including failed-query stream behavior, live in [§FS-output-shapes](FS-output-shapes.md#fs-output-shapes-machine-readable-output-shapes).

#### 3.1.1 `text`

The body only. The whole-declaration H1 (`# FS-<x>: …`) is omitted; section headings inside the slice are kept verbatim, including explicit named handles. Mode-by-mode: the default prints the lead prose ([§FS-show.2.1](FS-show.md#21-whole-declaration-default)); `--brief` prints the heading line and the first paragraph ([§FS-show.2.1.1](FS-show.md#211-brief---brief)) — the one mode that includes the H1 in `text`, since the slice would otherwise be unlabeled; `--toc` prints the lead plus the citable heading lines ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)); `--full` prints the full body ([§FS-show.2.1.3](FS-show.md#213-full-body---full)); a selected section is printed with its own section heading in every mode ([§FS-show.2.2](FS-show.md#22-section)). For an inline-source declaration the body is the comment-stripped prose ([§FS-show.2.3.2](FS-show.md#232-stripping-comment-markers)); for an E2E case it is the manifest ([§FS-show.2.4](FS-show.md#24-e2e-cases)). A `grund fmt --cross-refs` link wrapper around a citation (`[§FS-<x>.goals](FS-<x>.md#goals-scope)`) is flattened back to the bare citation — [§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json).

#### 3.1.2 `md`

Same as `text` but the opening declaration heading line is **included** verbatim, and `--cross-refs` link wrappers are kept as written — that is the renderable form ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)). For the default and `--toc`, the heading is prefixed; for `--brief` it is already included in `text` and stays as written in `md`; for `--full`, the heading is prefixed. The kind's `[[kinds]] title` ([§FS-config.3.4.3](FS-config.md#343-title)) is *not* injected — it is metadata that no `show` format carries, exposed in JSON only by `grund list --summary --format json` ([§FS-list.3.3](FS-list.md#33---summary)). For an inline-source declaration the included heading is the one written in the doc-comment (`AR-<event-bus>: In-process event broadcaster`), comment-markers stripped.

#### 3.1.3 `json`

A single object on stdout: `{"id":<ID>,"section":<section-path or null>,"body":<string>,"kind_title":<optional string>,"path":<declaring file>,"line":<1-indexed>}`. `body` is the same text `text` prints — `--cross-refs` wrappers flattened ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)). `section` is `null` when the whole declaration was requested and otherwise carries the exact numeric or named path string. With `--toc` the object additionally carries `sections` — one `{"path":<section path>,"title":<heading title after its coordinate>,"depth":<integer>}` per citable heading in the selected outline slice, in document order and before `kind_title`. `kind_title` is omitted when the selected kind has no effective title. The `path`, `line` pair always closes a declaration or section object: installed `grund-open` copies anchor on that trusted tail so arbitrary body prose cannot be mistaken for the location. For E2E cases the object is the distinct [§FS-show.2.4.2](FS-show.md#242-the-manifest-as-json) shape instead. The wire form is stable per [§GOAL-no-silent-breakage.1](../goals.md#1-what-counts-as-user-visible).

### 3.2 Cross-reference links are flattened in `text` and `json`

A repo that has run `grund fmt --cross-refs` ([§FS-fmt.6](FS-fmt.md#6-cross-reference-emission)) carries each citation in its `.md` files as a Markdown link *wrapping* the citation — `[§FS-check.1](FS-check.md#1-inputs)` instead of `§FS-check.1`. That wrapper is a rendered-view convenience ([§DF-md-link-emission](../decisions/functional/DF-md-link-emission.md#df-md-link-emission-grund-fmt-may-emit-clickable-markdown-links-alongside--prefixed-citations)), not the canonical form; for an agent pulling a fact into context it is noise, and the relative path inside it is the wrong pointer — the consumer should resolve the citation with `grund <ID>`, not open the file.

So when `show` prints a body in `text` or in the `json` `body` field, it **flattens** every such wrapper back to the bare citation; the exact wrap shape it collapses is [§FS-show.3.2.1](FS-show.md#321-the-wrap-shape-it-collapses); what it leaves as written, and that it resolves nothing, is [§FS-show.3.2.2](FS-show.md#322-what-is-left-as-written). Decided in [§DF-show-cross-ref-flattening](../decisions/functional/DF-show-cross-ref-flattening.md#df-show-cross-ref-flattening-grund-show-flattens-cross-reference-link-wrappers).

#### 3.2.1 The wrap shape it collapses

A `[` immediately before a marker-prefixed citation token and `](…)` immediately after it — exactly the wrap shape `grund fmt --cross-refs` emits and re-derives ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)) — collapses to just the `§[<alias>/]<ID>[.<section>]` text when it occurs in ordinary Markdown prose. This includes qualified workspace citations such as `[<§>api/FS-login](...)`, because they are the same presentation wrapper over the same canonical citation syntax ([§FS-workspace.8.5](FS-workspace.md#85-grund-fmt---cross-refs)). It does not collapse inside a fenced code block recognized by [§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example); fenced-content preservation takes precedence until that fence closes.

#### 3.2.2 What is left as written

Nothing else changes: an ordinary Markdown link in the prose, a citation that is not wrapped, a complete wrapper on a Markdown fence delimiter or within its fenced contents, and a `grund <ID> --format md` body (the self-contained markdown fragment, [§FS-show.3.1.2](FS-show.md#312-md)) are all left exactly as written. Fence-looking lines in a source doc-comment, a JSON value slice, or an E2E manifest do not acquire Markdown fence semantics: those source forms keep their existing interpretation. In particular, a manually authored complete wrapper in a source doc-comment is still flattened even between fence-looking lines, although `fmt --cross-refs` never writes that shape to source ([§FS-fmt.6.1](FS-fmt.md#61-scope)). The flattening is purely textual — it does not resolve the citation, so a dangling wrapper in ordinary prose is flattened just the same and `grund check` still reports it.

### 3.3 Batch mode

Batch mode is the explicit exception to the single-coordinate stream shape. It
emits one NDJSON envelope on stdout for every well-formed query, in query order;
query failures move inside their envelope so they cannot hide later outcomes.
Stderr is empty for all per-query successes and failures. The aggregate exits `0`
when every query succeeds (including an empty explicit stream or empty exhaustive
catalog), `1` after emitting all records when any query fails, and `2` with empty
stdout for a malformed invocation/input or configuration/scan failure. These
batch rules do not change any single-coordinate byte, stream, or exit behavior.

### 3.4 What a failed query prints

A failed query (`1`) prints the bare result line and, where the next step is obvious, one extra `hint:` line on stderr below it — never on stdout. With `--format=json`, stderr instead carries one diagnostic JSON object per [§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output), with `path` and `line` set to `null` because the failure has no single source location. The hint each failure gets is [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure).

`ambiguous ID`, `ambiguous section` and `broken stub` get no hint: the fix (run `grund check`, then edit the duplicate, renumber one of the two headings, or repair the stub) is already stated in [§FS-show.2.2.1](FS-show.md#221-ambiguous-id) / [§FS-show.2.2.2](FS-show.md#222-ambiguous-section) / [§FS-show.2.3.4](FS-show.md#234-broken-stub) and the message names the sites.

### 3.5 The hint for each failure

- `ID not found: <ID>` → `hint: run \`grund list\` to see every declared ID, or \`grund id <KIND> "<title>"\` to propose a new one` — withheld in the one case where the result line already names the answer, a workspace run whose refusal carries a `did you mean <alias>/<ID>?` clause ([§FS-workspace.8.1.1](FS-workspace.md#811-an-unqualified-id-another-project-declares))
- a missing snapshot for an ID whose parsed kind carries `fetch` in the loaded config → `hint: run grund fetch <qualified-ID>` — this remains offline and is the prescribed materialization hint ([§FS-check.4.12](FS-check.md#412-missing-snapshot), [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)). This branch is specified but not implemented today: `show` currently emits the generic `ID not found` hint.
- `section not found: <ID>.<s>` → `hint: run \`grund <ID> --toc\` to print the lead with the section map`
- a `<ID>` argument that does not match its kind's effective format → `invalid ID` and a format hint, [§FS-show.3.5.1](FS-show.md#351-an-id-the-grammar-rejects)

#### 3.5.1 An ID the grammar rejects

A `<ID>` argument that does not match its kind's effective format ([§FS-config.3.2](FS-config.md#32-id--id-grammar)), once the scan has found no exact off-grammar declaration of that spelling, fails with `invalid ID \`<arg>\``, followed by `hint: this repo's [id] format is \`<format>\` (run \`grund config show\`); \`grund list\` shows the IDs that exist` — naming the kind's effective format instead where a `[[kinds]] format` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)) governs it; this is the common surprise in a repo whose format differs from the `{kind}-{slug}` `grund` itself uses.

## 4. Why this matters

Without `show`, an agent retrieving a spec section either loads the whole file (token-expensive) or reimplements the parser. With `show`, the canonical way to pull `§FS-check.3.1` into a prompt is exactly:

```
grund FS-check.3.1
```

When the citation names no section, which slice to start from and when to widen or narrow it is [§FS-show.4.1](FS-show.md#41-a-citation-with-no-section).

This is the agent-grounding loop: declarations live in one place, and any agent — at any time — can fetch one, or just its lead, or just its map, with a single command.

### 4.1 A citation with no section

When the citation is a bare `§FS-check` with no section, the cheap first move is just `grund FS-check` — the new default prints the lead paragraph, enough to know whether this is the right declaration. If the section needs to be chosen, `grund FS-check --toc` adds the section map; `grund FS-check --full` holds the full body in reserve for when even that is not enough. An agent that grounds itself this way pays for the fact it needs, not the file it lives in. `grund FS-check --brief` is the narrowest body slice of all — heading plus one paragraph — for hover previews and "is this the right ID?" checks before committing to a deeper read, though in `text` it alone keeps the H1, so on a one-paragraph lead like `FS-check`'s it prints more than the default ([§FS-show.3.1.1](FS-show.md#311-text)).

# Querying grund

A grounded tree is a graph of declarations, their sections, and the citations
between them, and `grund` answers questions about that structure directly. Read
it with the queries below rather than by parsing heading lines or filtering the
whole citation graph by hand: they resolve named and numbered sections exactly
the way `grund check` does, and their JSON shapes are made to feed one another.
What each command does is specified at the point every recipe cites; this page
only puts the recipes side by side ([§FS-init.2.3.4.3.1](../functional-spec/FS-init.md#23431-structural-queries)).

Every example below was run in the `grund` repository itself.

## Select units: `list --glob`, `list --selector`, `list --kind`

`grund list --glob` takes a coordinate pattern and prints one row per declaration
or section it matches, sections at any depth included ([§FS-coordinate-globs](../functional-spec/FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)). `*` matches
inside one component, `**` any number of whole components, and `{a,b}` each of its
alternatives; the flag repeats, and its patterns union:

```console
$ grund list --glob 'FS-config.requirements.*'
FS-config.requirements.1  docs/functional-spec/FS-config.md:96  Every default is overridable where it has a meaning — directional
FS-config.requirements.2  docs/functional-spec/FS-config.md:100  Zero config works — realized
…
FS-config.requirements.8  docs/functional-spec/FS-config.md:124  A project's meaning is self-contained — realized
$ grund list --glob 'FS-c{heck,over}.1'
FS-check.1  docs/functional-spec/FS-check.md:46  Inputs
FS-cover.1  docs/functional-spec/FS-cover.md:12  Inputs
```

A pattern is always the `--glob` flag's value: typed as the positional, it stays a scan
path, and the failure says so
([§FS-coordinate-globs.errors.hint](../functional-spec/FS-coordinate-globs.md#errorshint-a-glob-typed-as-path-gets-a-hint)):

```console
$ grund list 'FS-config.*'
error: path does not exist: FS-config.*
hint: `list` reads `FS-config.*` as a scan path; for a coordinate pattern, use `grund list --glob 'FS-config.*'`
```

`grund list --selector` takes a rule subject — a sentence such as `The
requirements chapter of each FS`, or the dotted form `FS.requirements` — and
prints one row per unit it denotes ([§FS-list](../functional-spec/FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-rules.8](../functional-spec/FS-rules.md#8-command-surfaces)). `grund list --kind`
narrows the plain listing to some kinds:

```console
$ grund list --selector FS.requirements
FS-config.requirements  docs/functional-spec/FS-config.md:85  What the config contract holds to
$ grund list --selector "The requirements chapter of each FS" --format json
{"id":"FS-config","section":"requirements","kind":"FS","path":"docs/functional-spec/FS-config.md","line":85,"title":"What the config contract holds to","stub":false,"defines":null,"refs":0,"duplicate":false}
$ grund list --kind FS --format json | head -1
{"id":"FS-check","kind":"FS","path":"docs/functional-spec/FS-check.md","line":1,"title":"grund validates every citation in a repo","stub":false,"defines":null,"refs":2814,"duplicate":false}
```

With `--format json` each row is one NDJSON object. `id` and `section` name the
unit — `section` is present only for a selected section — and the rest say
where it is declared, its title, and how often it is cited.

A subject may end in a whose clause that keeps only the declarations whose
scalar field holds one value ([§FS-rules.2.2](../functional-spec/FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)), so the listing shows what a
filtered rule would reach before you write it:

```console
$ grund list --selector "Each SWR whose verification is Test"
SWR-brake-latency  req/SWR-brake-latency.md:1  The brake command reaches the actuator within 10 ms
```

A refused clause offers no selector, because a field or a value is the author's
to choose; the reason names the field or the value that failed.

A selector names chapters only, so a numbered section is refused, and every
refusal offers a selector to paste back, or lists the configured kinds where it
names none of them
([§FS-rule-refusals.selectors](../functional-spec/FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector)):

```console
$ grund list --selector FS.requirements.1
error: numbered chapter subjects can detach when headings move; accepted selector: FS.requirements
hint: grund show --batch --toc expands each selected unit into its sections
```

In a workspace, the projects a selector or `--kind` reads each answer for their
own kinds ([§FS-scan.projects.kinds](../functional-spec/FS-scan.md#projectskinds-whose-kinds-a-run-reads)). A `KIND.NAME` selector follows the
`[id] named_sections` setting of the project that configures `KIND`, not the
root's. So when member `api` turns named sections on, `list --selector API.terms`
lists the same `terms` chapters that `grund api/API-a.terms` shows. A kind
configured with `citable = false` is refused as a kind that declares no IDs,
naming its home, rather than as an unknown kind.

To reach `requirements.1`, match it with a glob, `--glob 'FS-*.requirements.*'`, or
expand the named chapter, as the section after next does. Every filter intersects
the others, so `--selector` and `--glob` together list what both select
([§FS-coordinate-globs.scope.filters](../functional-spec/FS-coordinate-globs.md#scopefilters-filters-intersect-and-never-widen)).

## Count them: `list --total`

`--total` prints the number of rows the same filters would list, and nothing else: a
bare number in text, `{"count":N}` in JSON, and `0` for an empty selection
([§FS-list.total](../functional-spec/FS-list.md#total---total-a-count-rung)). It is the cheap rung before any body is read:

```console
$ grund list --glob 'FS-config.requirements.*' --total
8
$ grund list --glob 'FS-*.terms' --total --format json
{"count":40}
```

A pattern no configured kind can start still exits `0` with its empty result, and
stderr gets a note naming the kind it most likely meant
([§FS-coordinate-globs.errors.note](../functional-spec/FS-coordinate-globs.md#errorsnote-a-pattern-no-kind-can-start-gets-a-note)):

```console
$ grund list --glob 'fs-config.*' --total
note: no configured kind in scope starts with "fs-config"; did you mean "FS-config.*"?
0
```

## What nothing cites yet: `list --unused`

`grund list --unused` lists the declarations nothing cites yet, in one call and one
scan, rather than one `grund refs` per declared ID ([§FS-list.1.3](../functional-spec/FS-list.md#13---unused)). The `grund`
repository cites every declaration it has, so this section runs in
[`examples/authored-citations/repo`](../../examples/authored-citations/repo/), where one decision is still uncited:

```console
$ grund list examples/authored-citations/repo --unused
ADR-consider-memory  docs/adr/consider-memory.md:1  Consider an in-process cache
$ grund list examples/authored-citations/repo --unused --format json
{"id":"ADR-consider-memory","kind":"ADR","path":"docs/adr/consider-memory.md","line":1,"title":"Consider an in-process cache","stub":false,"defines":null,"refs":1,"duplicate":false}
```

With `--format json` each uncited declaration is one NDJSON object of the shape
`--kind` lists. `--unused` leaves out the cases of a configured `E2E` kind unless `--kind`
selects them, since nothing is expected to cite a test. The kinds `grund init` writes have
no such kind: their e2e folder declares no IDs, so there is no case to leave out, and
`--kind E2E` is refused there as an unknown kind.

An entry in a kind's index does not count as a use, which is why the row above still
carries `refs: 1`: read the rows `--unused` lists rather than thresholding on `refs`.
[Checking a repository](checking.md#when-only-an-index-cites-a-declaration) says why the
index entry is discounted; `grund check --only unused` reports the same set as warnings.

A point listed here may still be cited from a repository this run cannot read, such as
a closed downstream. Have that repository generate a page of its citations into yours
rather than ignoring the entry: [Citers in a repository you cannot read](repository-options.md#citers-in-a-repository-you-cannot-read)
gives the recipe and its limits.

## Expand a unit's subtree: `show --batch --glob`

`grund show --batch --glob` generates the queries `--all` would and keeps the ones a
pattern matches, in one scan and with no stdin ([§FS-show.1.9](../functional-spec/FS-show.md#19---batch---all-every-coordinate-in-scope)). With `--toc` each
answer carries the unit's section map ([§FS-show.2.1.2](../functional-spec/FS-show.md#212-section-map---toc)), so expanding a chapter's
subpoints is one command:

```console
$ grund show --batch --glob 'FS-config.requirements' --toc --format json \
    | jq -c '{query, sections: [.result.sections[] | {path,title,depth}]}'
{"query":{"id":"FS-config","section":"requirements"},"sections":[{"path":"requirements.1","title":"Every default is overridable where it has a meaning — directional","depth":1},{"path":"requirements.2","title":"Zero config works — realized","depth":1},…,{"path":"requirements.8","title":"A project's meaning is self-contained — realized","depth":1}]}
```

`<ID>{,.**}` is a point and its whole subtree, so `--glob 'FS-config.requirements{,.**}'`
answers the chapter and each of its points, one envelope each. The explicit form,
`grund show --batch --toc --format json`, reads `{"id", "section"}` queries as NDJSON on
stdin instead ([§FS-show.1.8](../functional-spec/FS-show.md#18---batch-an-explicit-query-stream)), for a set no pattern names.

Each output line answers one query, in input order:

```text
{"query": {"id", "section"},
 "ok": true,
 "result": {"id", "section", "body", "sections": [{"path", "title", "depth", "anchor"}, …],
            "kind_title", "anchor", "path", "line"},
 "error": null}
```

Read `.result.sections[]`: `path` is the section path under `.result.id`
(`requirements.1`), so cite or fetch it as `<id>.<path>`
(`FS-config.requirements.1`); `title` is its heading text, `depth` how far
below the queried unit it sits, and `anchor` the fragment a web link to that
heading takes ([Link a citation from one read](#link-a-citation-from-one-read)). A query that does not resolve answers
`"ok":false` with an `error` object instead of a `result`, and the batch exits 1:

```console
$ echo '{"id":"FS-nope"}' | grund show --batch --brief --format json
{"query":{"id":"FS-nope","section":null},"ok":false,"result":null,"error":{"severity":"error","path":null,"line":null,"code":"not-found","message":"ID not found: FS-nope","sites":null,"authority":null}}
```

## Bodies in bulk: `show --batch --brief|--full`, `--all`

The same query stream fetches bodies instead of maps. `--brief` returns the
heading and first paragraph of each unit, `--full` the whole body, and no slice
flag the lead; the answer is the `result` object above without `sections`:

```console
$ echo '{"id":"FS-show","section":"1.8"}' | grund show --batch --brief --format json | jq -c '{query, ok, result: (.result | keys)}'
{"query":{"id":"FS-show","section":"1.8"},"ok":true,"result":["anchor","body","id","kind_title","line","path","section"]}
```

`grund show --batch --full --format json` reads the same queries and returns
every subsection's text in `body`. `--glob` reads the bodies a pattern selects, and to
read every declaration and section in scope without writing the queries, pass
`--all`: both read no stdin and generate one query per coordinate, in one scan
([§FS-show.1.9](../functional-spec/FS-show.md#19---batch---all-every-coordinate-in-scope)):

```console
$ grund show --batch --all --brief --format json | head -1 | cut -c1-120
{"query":{"id":"AR-benchmarks","section":null},"ok":true,"result":{"id":"AR-benchmarks","section":null,"body":"# AR-benc
```

Prefer `--glob` to `--all` filtered with `jq`: it prints only the units you asked for.

Each body names the unit it holds, so bodies read in bulk stay apart when they are
concatenated. A selected section's body opens with its own heading at the full path,
even where the file writes it relative: `### .2: Ceiling` under `## retry:` reads as
`### retry.2: Ceiling`, and the heading lines below it stay as written
([§FS-declarations.section.relative.shown.selected](../functional-spec/FS-declarations.md#sectionrelativeshownselected-a-selected-sections-own-heading-names-its-path)).

## Review a checklist: count, read, then map the diff

A checklist is a set of points, and a review reads it in three rungs
([§FS-coordinate-globs](../functional-spec/FS-coordinate-globs.md#fs-coordinate-globs-a-coordinate-glob-selects-catalog-coordinates-by-pattern)): count the points, read their bodies in one scan, then name the points a
changed line range belongs to with `grund cover FILE --lines N-M`
([§FS-cover.6](../functional-spec/FS-cover.md#6-line-ownership)):

```console
$ grund list --glob 'FS-config.requirements.*' --total
8
$ grund show --batch --glob 'FS-config.requirements.*' --brief --format json | head -1 | cut -c1-120
{"query":{"id":"FS-config","section":"requirements.1"},"ok":true,"result":{"id":"FS-config","section":"requirements.1","
$ grund cover docs/functional-spec/FS-config.md --lines 96-101
docs/functional-spec/FS-config.md:96-101
  96-99  FS-config.requirements.1
  100-101  FS-config.requirements.2
```

The [review-checklist example](../../examples/review-checklist/) runs these three steps on a small tree,
then two more from [the next sections](#who-cites-a-set-and-what-a-point-cites-refs---glob-refs---from):
who cites the points, and what one of them cites.

## The pattern grammar

A pattern is `[<qualifier>/]<body>` ([§FS-coordinate-globs.grammar](../functional-spec/FS-coordinate-globs.md#grammar-grammar)). Its first component, the ID
atom, matches the whole ID, and each later component one section path component;
matching is anchored and case-sensitive. The operators are:

- `*` — any run of characters inside one component, the empty run included;
- `?` — exactly one character;
- `[…]`, `[!…]`, `[^…]` — one character that is, or is not, a member of the class;
- `**` — written as a whole component, zero or more whole components (one or more at
  the end), so `X.*` is the children of `X`, `X.**` everything beneath it, and
  `X{,.**}` the point and its subtree;
- `{a,b}` — each comma-separated alternative, which may be empty and may cross `.`;
- `\` — the next character, literally; escape a `.` inside an ID.

In a workspace, an unqualified pattern reads the current project, `*/` every project,
and `<alias>/` that member, each split with its own section separator
([§FS-coordinate-globs.scope](../functional-spec/FS-coordinate-globs.md#scope-scope)). A refused pattern is one line, its column, and the pattern to
paste back, and the run exits `2`:

```console
$ grund list --glob 'FS-**'
error: invalid coordinate glob "FS-**" at column 4: "**" must be a whole component; accepted globs: "FS-*" or "FS-*{,.**}"
```

The grammar reads like the gitignore dialect of `exclude`, and departs from it
deliberately ([§FS-coordinate-globs.grammar.departures](../functional-spec/FS-coordinate-globs.md#grammardepartures-where-it-departs-from-the-exclude-dialect)): `x**` is refused, braces
do not nest, `{,…}` keeps its empty alternative, a pattern is always anchored at the ID
with no basename matching, there is no `!` negation and no `#` comment, and components
are separated by `.`, with `/` only ending a qualifier.

## Link a citation from one read

Every `--format json` read of a declaration or section carries `anchor`: the
fragment, without its `#`, that `grund fmt --cross-refs` would write for the
same coordinate under the project's `anchor_format`
([§FS-show.3.1.3.1](../functional-spec/FS-show.md#3131-the-heading-anchor)). So a web link is `path` plus `anchor`, and
nothing has to slug a heading by hand:

```console
$ grund FS-show.3.1.3 --format json | jq -c '{path, anchor, line}'
{"path":"docs/functional-spec/FS-show.md","anchor":"313-json","line":458}
$ grund FS-show.3.1.3 --format json \
    | jq -r '"https://github.com/agent-grounds/grund/blob/main/\(.path)" + (if .anchor then "#\(.anchor)" else "#L\(.line)" end)'
https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-show.md#313-json
```

`anchor` is `null` where the site has no heading anchor: a declaration whose
home is a source file, a JSON value, an E2E case, and every read under
`anchor_format = "none"`. Link those by `#L<line>`, as the recipe does, except
an E2E case: its object has no `line`, so link its directory `path` with no
fragment. The
base and the ref are yours to choose; `grund` hands over the data and renders
no URL itself ([§DF-show-anchor-data](../decisions/functional/DF-show-anchor-data.md#df-show-anchor-data-show-json-carries-the-heading-anchor-grund-already-derives)).

Every JSON `path` is spelled from the project root by default, which is
what a repository URL wants. `--path-base=invocation` spells it from the path
you passed, or the current directory if you passed none, instead — for an
editor or a shell that joins it to where it stands
([§FS-cli.3.4](../functional-spec/FS-cli.md#34---path-base--where-report-paths-are-spelled-from)).

## Who cites a point: `refs`, `--descendants`

`grund refs <ID>` lists every site that cites the point ([§FS-refs.1](../functional-spec/FS-refs.md#1-inputs)); `--summary`
folds the sites to one line per file, and `--descendants` widens the question
from the section to the section and everything beneath it:

```console
$ grund refs FS-show.1.8 --summary
crates/grund-cli/src/cli_show.rs: 1 (line 20)
crates/grund-cli/src/cli_show_batch.rs: 1 (line 11)
…
$ grund refs FS-show.1 --descendants --summary | head -2
crates/grund-cli/src/cli_help_show.rs: 1 (line 1)
crates/grund-cli/src/cli_show.rs: 3 (lines 20, 211, 216)
$ grund refs FS-show.1.8 --format json | head -1
{"path":"crates/grund-cli/src/cli_show.rs","line":20,"column":6,"id":"FS-show","section":"1.8","marker":true,"text":"§FS-show.1.8","enclosing_declaration":null,"enclosing_section":null,"kind_title":"What: behavior, requirements, and constraints"}
```

`--total` sizes the blast radius in one line, before any listing:

```console
$ grund refs FS-show.1.8 --total
cited at 8 sites across 8 files
```

For scripts, exit `0` is a completed `refs` answer even when it is empty. Exit
`1` means the selected repository grammar rejected the ID or its number-only
shorthand was ambiguous; route that status to ID repair, and reserve exit `2`
for setup, configuration, I/O, or incomplete-scan failure
([§FS-refs.4](../functional-spec/FS-refs.md#4-exit-codes)).

An explicit path limits the scan ([§FS-scan.path.refs](../functional-spec/FS-scan.md#pathrefs-refs-scans-the-paths-tree)). Suppose `FS-api-audit` is declared in `docs/` and cited nowhere. From the project root:

```console
$ grund refs FS-api-audit src --total
not cited
note: FS-api-audit is neither declared nor cited in scan path `src` — run `grund list` to see project declarations, including outside this scan
```

Here `not cited` is stdout and the note is stderr. The note describes only the
selected scan; `grund list` can still list the declaration in `docs/`.
From `src/`, `grund refs FS-api-audit .` names `.` in that same note.

With no path the question is about the whole project, where `FS-api-audit` is
declared, so the note points at the one call that lists every declaration
nothing cites ([§FS-refs.2.1.1](../functional-spec/FS-refs.md#211-a-declared-id-nothing-cites-points-at-list---unused)):

```console
$ grund refs FS-api-audit
note: FS-api-audit is declared but nothing cites it; `grund list --unused` lists every such declaration
```

stdout stays empty, `--total` still prints `not cited`, and the exit is `0`, so a
script that reads the answer sees no change. If you are checking IDs one at a
time, the first empty answer tells you that
[`grund list --unused`](#what-nothing-cites-yet-list---unused) answers all of
them at once. The note names the declaration as the catalog spells it, whatever
shorthand or digits you typed. An alias-qualified workspace query names its
project, `grund list --project <alias> --unused`, and a case of a configured `E2E`
kind adds `--kind E2E`, because `--unused` leaves those cases out unless that kind
is selected ([§FS-list.1.3.1](../functional-spec/FS-list.md#131-a-configured-e2e-kind-is-left-out-unless-selected)).

The note is given only where that listing answers the question you asked. It is
withheld under an explicit path, `.` included, which asks about part of the
tree; for a section, inline or `--section`, since `--unused` judges whole
declarations (a bare `--descendants` changes nothing and keeps the note); for an
ID whose only citation is its kind's index entry, which `refs` lists as
`(index entry)` while `--unused` lists the ID; and when the scan is incomplete,
which exits `2`. [`examples/refs-unused`](../../examples/refs-unused/) runs both
calls.

[Reviewing code](reviewing.md) walks through using these before a move,
rename or delete.

## Who cites a set, and what a point cites: `refs --glob`, `refs --from`

`grund refs --glob <pattern>` asks ID mode's question of every coordinate a pattern
matches ([§FS-refs.globs](../functional-spec/FS-refs.md#globs---glob-lists-who-cites-a-set)). Repeated, the patterns union and each site is listed once, and a
site whose cited point is no longer declared is listed too, so a glob finds what a rename
left dangling:

```console
$ grund refs --glob 'FS-show.1.{8,9}' --summary | head -2
crates/grund-cli/src/cli_help_show.rs: 1 (line 31)
crates/grund-cli/src/cli_show.rs: 2 (lines 20, 163)
```

A pattern's bare ID is not ID mode's. `--glob 'FS-refs'` selects only the bare citations
of `FS-refs`, unlike `grund refs FS-refs`, which is `--glob 'FS-refs{,.**}'`: the point
and every section beneath it ([§FS-refs.globs.descendants](../functional-spec/FS-refs.md#globsdescendants-one-id-asked-both-ways-gives-one-answer)):

```console
$ grund refs --glob 'FS-refs' --total
cited at 26 sites across 21 files
$ grund refs FS-refs --total
cited at 356 sites across 99 files
$ grund refs --glob 'FS-refs{,.**}' --total
cited at 356 sites across 99 files
```

`grund refs --from <coordinate>` turns the question round: it lists the sites in that
coordinate's own body, what it cites ([§FS-refs.from](../functional-spec/FS-refs.md#from---from-lists-what-one-point-cites)). A section inherits what its
declaration's lead cites and what each ancestor cites above its first child, but never
what a sibling or a descendant cites, so `--from FS-show.1.8` lists `FS-show`'s lead and
`1`'s opening as well as `1.8` itself ([§FS-refs.from.owned](../functional-spec/FS-refs.md#fromowned-a-point-owns-its-lead-its-ancestors-and-itself)). The source is
resolved as `grund FS-show.1.8` resolves it, and one that query refuses is refused with its
bytes:

```console
$ grund refs --from FS-show.1.8 | head -3
docs/functional-spec/FS-show.md:7: §GOAL-friendliness-first
docs/functional-spec/FS-show.md:8: §GOAL-token-economy
docs/functional-spec/FS-show.md:9: §GOAL-fast-feedback.1
$ grund refs --from FS-show.1.8 --total
cites at 13 sites across 1 file
```

A path bounds the citation sites listed, while the source still resolves against
the project the path discovers
([§FS-scan.path.refs](../functional-spec/FS-scan.md#pathrefs-refs-scans-the-paths-tree)).
Since this source owns sites only in its declaration in `docs/`, selecting `src/`
prints nothing and succeeds; `--total` reports `cites nothing`:

```console
$ grund refs --from FS-show.1.8 src
$ grund refs --from FS-show.1.8 src --total
cites nothing
```

`--summary`, `--total` and `--format` fold and print both modes as they do ID mode's, and
a JSON record carries no `kind_title`, since no one queried target names it
([§FS-refs.3.2](../functional-spec/FS-refs.md#32---format-json)). Under either flag every positional is the scan path, so an ID
typed there is read as a path, and the error says how to ask for it
([§FS-refs.1.3](../functional-spec/FS-refs.md#13-an-id-typed-as-the-path-gets-a-hint)):

```console
$ grund refs FS-refs --glob 'FS-refs.from'
error: path does not exist: FS-refs
hint: with `--glob`, `refs` reads `FS-refs` as a scan path; to add its citations, use `--glob 'FS-refs{,.**}'`
```

`--section` and `--descendants` belong to ID mode. Beside one `--glob` the refusal offers
the pattern that asks for them, and `--glob` beside `--from`, a second `--from` or a second
path is refused outright, all before any config is read
([§FS-refs.1.2](../functional-spec/FS-refs.md#12-refusals)):

```console
$ grund refs --glob 'FS-refs.from' --descendants
error: --descendants cannot be combined with --glob; accepted glob: "FS-refs.from{,.**}"
```

## The whole citation graph: `cover --format json`

`grund cover --format json` emits one record per scanned file, holding every
citation in it ([§FS-cover.3.2](../functional-spec/FS-cover.md#32---format-json)):

```text
{"path", "citations": [{"path", "line", "column", "id", "section", "marker", "text",
                        "enclosing_declaration", "enclosing_section"}, …]}
```

`id` and `section` name what a site cites; `enclosing_declaration` and
`enclosing_section` name the unit the site sits in, so each citation is one
edge of the graph with both its ends. They are `null` in a file that declares
nothing:

```console
$ grund cover docs/functional-spec/FS-show.md --format json \
    | jq -c '.citations[] | select(.enclosing_section != null) | {line,id,section,enclosing_declaration,enclosing_section}' \
    | head -1
{"line":13,"id":"FS-terms","section":"terms.1","enclosing_declaration":"FS-show","enclosing_section":"terms"}
```

`cover --glob <pattern>` keeps, in each file's record, only the citations whose
cited coordinate a pattern matches, and keeps every scanned file's record, so a file
that cites none of them still reads `(no citations)`, or `"citations":[]` in JSON
([§FS-cover.globs](../functional-spec/FS-cover.md#globs---glob-keeps-the-citations-a-pattern-matches)):

```console
$ grund cover crates/grund-cli/src/ --glob 'FS-refs.from{,.**}' | head -2
crates/grund-cli/src/cli.rs:
  (no citations)
```

Reach for `cover` when the question is about the whole graph — which specs a
diff's files touch, which units cite which. When the question is about named
units, select them with a glob and expand them with `show --batch --glob` above.

# Querying grund

A grounded tree is a graph of declarations, their sections, and the citations
between them, and `grund` answers questions about that structure directly. Read
it with the queries below rather than by parsing heading lines or filtering the
whole citation graph by hand: they resolve named and numbered sections exactly
the way `grund check` does, and their JSON shapes are made to feed one another.
What each command does is specified at the point every recipe cites; this page
only puts the recipes side by side ([§FS-init.2.3.4.3.1](../functional-spec/FS-init.md#23431-structural-queries)).

Every example below was run in the `grund` repository itself.

## Select units: `list --selector`, `list --kind`

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

## Expand a unit's subtree: `show --batch --toc --format json`

The `{id, section}` pair of a `list` row is exactly one query of
`grund show --batch`, which reads such queries as NDJSON on stdin
([§FS-show.1.8](../functional-spec/FS-show.md#18---batch-an-explicit-query-stream)). With `--toc` each answer carries the unit's section map
([§FS-show.2.1.2](../functional-spec/FS-show.md#212-section-map---toc)), so selecting chapters and expanding their subpoints is two
commands:

```console
$ grund list --selector "The requirements chapter of each FS" --format json \
    | jq -c '{id,section}' \
    | grund show --batch --toc --format json \
    | jq -c '{query, sections: [.result.sections[] | {path,title,depth}]}'
{"query":{"id":"FS-config","section":"requirements"},"sections":[{"path":"requirements.1","title":"Every default is overridable where it has a meaning — directional","depth":1},{"path":"requirements.2","title":"Zero config works — realized","depth":1},…,{"path":"requirements.8","title":"A project's meaning is self-contained — realized","depth":1}]}
```

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
every subsection's text in `body`. To read every declaration and section in
scope without writing the queries, pass `--all`: it reads no stdin and
generates one query per coordinate, in one scan ([§FS-show.1.9](../functional-spec/FS-show.md#19---batch---all-every-coordinate-in-scope)):

```console
$ grund show --batch --all --brief --format json | head -1 | cut -c1-120
{"query":{"id":"AR-benchmarks","section":null},"ok":true,"result":{"id":"AR-benchmarks","section":null,"body":"# AR-benc
```

Prefer a selector-fed batch to `--all` filtered with `jq`: it reads only the
units you asked for.

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

[Reviewing code](reviewing.md) walks through using these before a move,
rename or delete.

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

Reach for `cover` when the question is about the whole graph — which specs a
diff's files touch, which units cite which. When the question is about named
units, select them and expand them with the two commands above.

# Coordinate sizes

Configuration reference: [`[schema.leads.words]`](configuration.md#schemaleadswords) covers the warning threshold configured here.

`grund list --size` measures the text an agent can fetch through each declaration
and section coordinate. It reports both the
[lead returned by a default read](querying.md#read-one-point-with-grund-id) and
the full body returned by `--full`, using the same body slicing as `grund show`
([§FS-list.3.4](../functional-spec/FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements)).

```console
$ grund list --size=words --top 3
FS-large.2  docs/functional-spec/FS-large.md:44  words=721/944
FS-large    docs/functional-spec/FS-large.md:1   words=510/1730
AR-wide.1   docs/architecture/AR-wide.md:19      words=405/405
```

Bare `--size` selects `lines,words,bytes`. An attached comma list selects and
orders columns explicitly; `--top N` sorts on the first selected unit after the
ordinary project, kind, and unused filters. JSON output is NDJSON with stable
metadata fields followed by each selected `lead_<unit>` / `full_<unit>` pair.

The counts are byte-defined and deterministic:

- `lines` counts LF-delimited segments containing at least one byte other than
  ASCII whitespace;
- `words` counts non-empty byte runs separated by ASCII whitespace;
- `bytes` is the UTF-8 byte length.

Section headings are part of section bodies; declaration headings are not part
of declaration bodies. Comment markers are stripped from inline/doc-comment
declarations. Duplicate declarations and duplicate section claimants retain one
site-local row each. A broken stub remains visible with `-/-` in text or `null`
measurements in JSON rather than borrowing another site's body.

Measurements include complete citation-wrapper bytes preserved inside Markdown
fences, because those bytes are part of the exact text body `grund show`
returns. They can raise byte and word counts (and an opted-in warning near its
limit), but they do not add lines; wrappers in ordinary prose remain flattened.

## Opt into a lead-size check

In a version-2 config, add this table to the project whose coordinate leads you
want checked ([§FS-config.3.1](../functional-spec/FS-config.md#31-reference--citation-form)):

```toml
[schema.leads.words]
warn = 600
```

The warning does not change the exit status. Version 2 measures words and is
off by default. A lead strictly
over the maximum is reported at the severity set by its strength. Equality passes
([§FS-declarations.checks.oversized-lead](../functional-spec/FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)).
Without the key the check is off, so `grund check --only oversized-lead` says
`--only oversized-lead judged nothing` and names the key, rather than printing a
`success` that judged no lead
([§FS-check.1.4.1](../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)).

The remedy is structural, not deletion: move detailed prose into citable child
sections, which keeps the parent coordinate stable, or promote a child section
to its own declaration. Before splitting, run `grund refs <ID>.<section>` and
change each citer to the new ID; then move the section, renumber the remaining
sections, and run `grund fmt --write`. Run `grund refs <ID> --total` for the
declaration's size, or `--summary` for the files behind it. Each workspace
member uses its own config. Explicit-path checks judge only sites in that path,
and `--full` does not widen this project policy beyond its configured scan scope.

# External facts

External facts are ordinary declarations saved in the repository. Grund reads
those committed bytes offline; only an explicit `grund fetch <ID>` runs the
integration configured by the repository ([§FS-fetch](../functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot)).

Configure a kind with its provider's ID shape, one snapshot home, and one direct
executable:

```toml
[[kinds]]
kind = "GH"
file = "docs/tickets.md"
title = "External tickets (generated snapshots)"
format = "{kind}-{number}"
resolve = "should"
fetch = "scripts/fetch-ticket"
```

The executable receives exactly one argument, the local ID, and prints one
complete Markdown declaration to stdout. Save the following as `scripts/fetch-ticket`
and run `chmod +x scripts/fetch-ticket` before the first fetch:

````sh
#!/bin/sh
title=$(gh issue view "${1#GH-}" --json title --jq .title) || exit
body=$(gh issue view "${1#GH-}" --json body --jq .body) || exit
fence=$(printf '%s\n```\n' "$body" | grep -o '`\{3,\}' | sort | tail -n 1)
printf '## %s: %s\n\n%s`markdown\n%s\n%s`\n' "$1" "$title" "$fence" "$body" "$fence"
````

For a `file` home the declaration is H2; for a `folder` home it is H1, and body
subsections must be exactly one level deeper than that declaration. The example
computes an outer backtick fence one character longer than any run in the
provider body, so its own Markdown headings remain content but are not citable
child headings; raw issue Markdown is not a declaration body.
A heading anywhere below the declaration that would itself declare an ID, of
any kind and at any depth, is refused, so a fetcher that copies provider
headings must fence them as the example does
([§FS-fetch.3.1](../functional-spec/FS-fetch.md#31-a-nested-declaration-is-refused)).
A heading one level below the declaration must also carry a section path the
project's configuration admits, such as `### 1. Notes`, or `### notes: Notes`
with named sections on. Any other is refused rather than left for the next
`grund check` to reject, so number it, fence it, or make it a bold label
([§FS-fetch.3.2](../functional-spec/FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)).
Grund validates the complete output before atomically replacing only that ID's
snapshot. It preserves accepted bytes verbatim and inserts a new file-home
declaration in ID order ([§FS-fetch.3](../functional-spec/FS-fetch.md#3-accepted-declaration),
[§FS-fetch.4](../functional-spec/FS-fetch.md#4-file-home-write)).

A snapshot is Markdown, so fetch refuses a `file` home the scan reads as source,
such as `fs.py`, or `fs.txt` under `txt = "python"`. It exits 2 before running the
integration and leaves every file unchanged, whatever the file holds and whether
or not it exists. Move the kind and its declarations to a Markdown file or a
folder, or, for a kind that is not a value kind, remove its `fetch` and keep the
declarations in the source file
([§FS-fetch.4.2](../functional-spec/FS-fetch.md#42-a-home-read-as-source-is-refused)).

For a folder home, fetch replaces an existing Markdown snapshot even when its
filename differs from the ID. If a recognized value source in that home already
declares the requested ID, fetch instead exits 2 and names the source, leaving
all files unchanged. Keep that fact in the value source, or move it out of the
value home before fetching. A table holding only other IDs does not block fetching
([§FS-fetch.5.1](../functional-spec/FS-fetch.md#51-existing-value-source-declarations-prevent-installation)).

In a `folder` home, a file the scan reads under a name other than `.md` that
already declares the ID stops the fetch: grund exits 2, names that file and
changes nothing, because a new `<ID>.md` beside it would be a duplicate and
replacing it would overwrite what it holds around the declaration. Keep the fact
in that file, or take the declaration out of it and fetch again
([§FS-fetch.5.2](../functional-spec/FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation)).
Where the scan does not read `<ID>.md` as Markdown, taking the declaration out leaves
the ID declared nowhere until it does, so change both together.

With `resolve = "must"`, a missing snapshot is the ordinary `dangling` error.
With `resolve = "should"`, it is the fixed `missing-snapshot` warning and a
warning-only check exits 0. Choose `must` when CI should enforce that every
external fact is materialized before merge; choose `should` when the repository
may carry a citation ahead of its snapshot and should only report the gap. These
mechanics are defined by [§FS-config.3.4.10](../functional-spec/FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds).
Omitting `resolve` from a fetch-enabled kind means `must`; `resolve` without
`fetch` and `resolve = "may"` are invalid. No check or editor action fetches
implicitly ([§FS-check.4.12](../functional-spec/FS-check.md#412-missing-snapshot)).

A version-2 config spells the same kind as a row and its resolution as a rule
([§FS-config-v2.rules.resolution](../functional-spec/FS-config-v2.md#rulesresolution-rulesresolution)).
A fetched kind is `must` unless `[rules.resolution]` says `warn`, which is the
version-1 `should`:

```toml
[schema.kinds.GH]
files = ["docs/tickets.md"]
id_format = "{kind}-{number}"
fetch = "scripts/fetch-ticket"

[rules.resolution]
GH = "warn"

[presentation.kinds.GH]
title = "External tickets (generated snapshots)"
```

In a workspace, qualify the ID to select the owning project:

```sh
grund fetch api/GH-1234
```

The member's executable receives only `GH-1234`, and the declaration is
written under that member's configured home ([§FS-fetch.1](../functional-spec/FS-fetch.md#1-input-and-project-selection)).

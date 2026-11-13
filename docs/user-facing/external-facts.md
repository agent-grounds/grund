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

For a `file` home the declaration is H2; for a `folder` home it is H1. Below it,
a subsection is any heading the next `grund check` reads as a section of it, at the
level `check` holds sections to, so `### 1. Notes` and `#### 1.1. Detail` under it
are both kept. The example
computes an outer backtick fence one character longer than any run in the
provider body, so its own Markdown headings remain content but are not citable
child headings; raw issue Markdown is not a declaration body.
A heading anywhere below the declaration that would itself declare an ID, of
any kind and at any depth, is refused, so a fetcher that copies provider
headings must fence them as the example does
([§FS-fetch.3.1](../functional-spec/FS-fetch.md#31-a-nested-declaration-is-refused)).
A heading the next scan would not read as a section, such as `### Other`, is
refused rather than left for the next `grund check` to reject, so number it,
fence it, or make it a bold label
([§FS-fetch.3.2](../functional-spec/FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)).

Grund does not install output on its own reading of it. It builds the file it
would write, has your project's own scan read it, and installs it only when that
reading is exactly what was asked for: the requested declaration once, every
other declaration in the home reading as it did, no finding `check` would report
on the home that it did not already report, every fence and raw-text block the
output opens closed, and a second fetch of the same output changing nothing. The
first of these that fails refuses the fetch with exit 2 and leaves the tree
unchanged, with the line `check` would print where `check` has one
([§FS-fetch.admission](../functional-spec/FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)).
Accepted bytes are kept verbatim, replacing only the range the scan reads as that
ID's declaration, and a new file-home declaration goes in ID order
([§FS-fetch.3](../functional-spec/FS-fetch.md#3-accepted-declaration),
[§FS-fetch.4](../functional-spec/FS-fetch.md#4-file-home-write)).

The scan runs once, before the integration does, and the write is made against
what it read; nothing else is read again, so a file the integration writes beside
the home plays no part in what fetch admits. If the home is edited or created
while the integration runs, fetch
exits 2 with `<path> changed while the integration ran; fetch again` and leaves
the change in place
([§FS-fetch.6.1](../functional-spec/FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)).

A snapshot is Markdown, so fetch refuses a `file` home the scan reads as source,
such as `fs.py`, or `fs.txt` under `txt = "python"`. It exits 2 before running the
integration and leaves every file unchanged, whatever the file holds and whether
or not it exists. Move the kind and its declarations to a Markdown file or a
folder, or, for a kind that is not a value kind, remove its `fetch` and keep the
declarations in the source file
([§FS-fetch.4.2](../functional-spec/FS-fetch.md#42-a-home-read-as-source-is-refused)).
A home the scan does not read at all, such as `fs.dat` or a hidden `.fs.py`, is
refused the same way, with the reason `check`'s unread warning gives for that
name; in a `folder` home the same holds for the `<ID>.md` fetch would create
([§FS-fetch.4.2.1](../functional-spec/FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)).

A home, or a folder home's `<ID>.md`, that is a symbolic link is written through
when its target stays inside the config root, and the link stays. A link that
leaves the root, whose target does not exist, or that the scan does not read as
a file, such as one an ignore file skips, is refused with the tree unchanged;
replace the link with the file it points to, create the target, or remove it
([§FS-fetch.links](../functional-spec/FS-fetch.md#links-a-link-at-the-target-is-written-through-when-it-stays-inside-the-config-root)).

The `fetch` key must name a program: an empty or whitespace-only value names none. From grund 0.23.0, version 2 refuses it when the config loads, at the key's line, on every command that loads the config, and no selector suppresses that. Version 1 still loads the key and meets the rule at `grund fetch`, before running anything, and at `grund config migrate`. Either way the repair is the same: name the program, or, if the kind is not fetched, remove the key and any `[rules.resolution] <KIND>` (version 1: `resolve`). A setting of `warn` needs the key, so removing the key alone is a load error, and with both gone the kind's missing IDs are errors
([§FS-fetch.2.1](../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable),
[§FS-config-v2.checks.config-fetch-names-nothing](../functional-spec/FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)).
Under version 1, `grund check` gives the same repair at each citation of a missing ID rather than sending you to a command with nothing to run:

```text
docs/arch.md:7: warning: no snapshot for AR-002-other in docs/arch.md — [[kinds]] `AR` fetch names no executable; name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors
```

([§FS-check.3.1.3.2](../functional-spec/FS-check.md#3132-a-key-fetch-has-nothing-to-run-for)).

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

A folder-home fetch also refuses when the selected project's scan finds the ID
declared outside that home, in Markdown or a source comment. It exits 2, names
the declaring file and leaves the entire tree unchanged. A stub pointing there
stays intact, even when the scan reaches the target only through that stub.
Keep the existing fact in its file, or remove its declaration and fetch again
to install the snapshot. Moving a value source out of its home therefore only
permits fetching if the scan no longer reads it as declaring that ID
([§FS-fetch.5.3](../functional-spec/FS-fetch.md#53-a-declaration-outside-the-home-prevents-installation)).

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

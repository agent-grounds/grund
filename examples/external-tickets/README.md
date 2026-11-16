# External ticket snapshots

This example teaches the fetch-backed external-fact workflow specified by
[§FS-fetch](../../docs/functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot).
The repository uses ordinary slug IDs by default and a numeric `GR` override
beside `GRUND`. `repo/docs/guide.md` declares `GRUND-why` and cites a committed
ticket snapshot; the fetched ticket cites `GRUND-why` back. Their `GR-` and
`GRUND-` heads distinguish the IDs in one project
([§FS-config.3.4.5.1](../../docs/functional-spec/FS-config.md#3451-citable-names-and-formats-must-distinguish-their-tokens)).
A `GR-X` kind beside `GR` still needs a rename because punctuation follows the
shorter name, even with distinct separators. `repo/scripts/fetch-ticket` is a
deterministic local stand-in for a GitHub, Jira, or Linear integration.

From inside `repo/`, the complete workflow is:

```sh
grund check
grund fetch GR-80923
grund check
grund show GR-80923
grund refs GR-80923
```

The first check exits 0. Fetch is the
only command that executes the repository integration; it is silent on success
and replaces the complete declaration in `docs/tickets/local.txt`. The remaining
commands resolve and query only that committed Markdown snapshot, so they keep
working offline and never execute the integration.

Fetch writes only what the next scan reads back as asked: the requested
declaration once, every other declaration reading as it did, and nothing new
for `check` to report there. Output that would read any other way is refused with
exit 2 and the file left as it was
([§FS-fetch.admission](../../docs/functional-spec/FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)).

This version-2 project maps `txt = "markdown"`, so the existing `local.txt` is
read as Markdown and replaced in place, even though its name differs from the ID.
Fetch does not create a second `GR-80923.md` beside it. The folder can also contain
a nested snapshot under another name; the same reading decides which file to
replace. Two files declaring the ID, or content beside it in the same file, are
refused without a write
([§FS-fetch.5.5](../../docs/functional-spec/FS-fetch.md#55-which-file-declares-and-what-idmd-may-hold)).
A home nothing reads as Markdown is refused before the integration runs, naming
the cause. A data mapping is named as data, rather than as an unlisted language
([§FS-fetch.4.2.1](../../docs/functional-spec/FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)).

The e2e runner performs the mutating fetch from a copied repository and compares
the entire result with `expected.repo`. No network or credentials are involved.
The practical cost is deliberate: snapshot freshness is an explicit repository
diff rather than an implicit property of every check.

The `fetch` key has to name a program; an empty or whitespace-only value names
none ([§FS-fetch.2.1](../../docs/functional-spec/FS-fetch.md#21-a-value-that-names-no-executable)).
Where that is refused depends on the config version. Version 2 refuses it when
the config loads, so this config stops every command that reads it:

```toml
grund_config_version = 2
project_name = "tickets"

[schema.kinds.GR]
files = ["docs/tickets.md"]
fetch = ""
```

```text
$ grund check
error: grund.toml:6: `[schema.kinds.GR] fetch` names no executable; name the program, or if this kind is not fetched remove the key and any `[rules.resolution] GR`, and its missing IDs are then errors; this became an error in grund 0.23.0
```

Version 1 still loads a blank `fetch` key. In a version-1 project, `grund check`
keeps working and `grund fetch` is what refuses, before running anything:

```text
$ grund fetch GR-80923
error: cannot run fetch integration `` for GR-80923: grund.toml:15: [[kinds]] `GR` fetch names no executable; name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors
```

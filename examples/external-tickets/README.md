# External ticket snapshots

This example teaches the fetch-backed external-fact workflow specified by
[§FS-fetch](../../docs/functional-spec/FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot). The repository uses ordinary slug IDs by default and a numeric
`GH` override. `repo/docs/guide.md` cites a ticket before its snapshot
exists; `repo/scripts/fetch-ticket` is a deterministic local stand-in for a
GitHub, Jira, or Linear integration.

From inside `repo/`, the complete workflow is:

```sh
grund check
grund fetch GH-1234
grund check
grund show GH-1234
grund refs GH-1234
```

The first check exits 0 with the fixed `missing-snapshot` warning. Fetch is the
only command that executes the repository integration; it is silent on success
and writes the complete declaration to `docs/tickets.md`. The remaining
commands resolve and query only that committed Markdown snapshot, so they keep
working offline and never execute the integration.

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

[schema.kinds.GH]
files = ["docs/tickets.md"]
fetch = ""
```

```text
$ grund check
error: grund.toml:6: `[schema.kinds.GH] fetch` names no executable; name the program, or if this kind is not fetched remove the key and any `[rules.resolution] GH`, and its missing IDs are then errors; this became an error in grund 0.23.0
```

This example's own config is version 1, which still loads the key. With
`fetch = ""` in `repo/grund.toml`, `grund check` keeps working and `grund fetch`
is what refuses, before running anything:

```text
$ grund fetch GH-1234
error: cannot run fetch integration `` for GH-1234: grund.toml:15: [[kinds]] `GH` fetch names no executable; name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors
```

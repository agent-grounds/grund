# Places: several per row, and the one row that owns each file

This runnable mini-repository demonstrates version-2 places ([§FS-config-v2.schema.places](../../docs/functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)). It is for a repository whose kinds do not each fit one folder: a spec split across two directories, a goals file that sits among prose pages, and source code in more than one top-level directory. Open `repo/grund.toml` first:

```toml
grund_config_version = 2
project_name = "places-example"

[schema.sources]
languages = ["markdown", "rust", "shell"]

# One kind, two folders: each folder keeps its own index.
[schema.kinds.FS]
folders = ["docs/spec/core", "docs/spec/plugins"]

# A file listed inside the `page` folder belongs to this row, not to `page`.
[schema.kinds.GOAL]
files = ["docs/guide/goals.md"]

# Every file under docs/ that no deeper place and no listed file claims.
[schema.kinds.page]
citable = false
folders = ["docs"]

# The complement: with places, it narrows the scan to them.
[schema.kinds.code]
folders = ["crates", "scripts"]
```

The citation rules that follow ask `FS` to cite `GOAL`, every `page` to cite `GOAL`, and code to cite `FS`.

Run it from the repository root:

```bash
grund schema examples/places/repo
echo $?    # 0
```

The view on stdout is the golden below; stderr is empty:

```text
places-example · grund.toml · v2

kinds and places
  FS    kind        docs/spec/core/, docs/spec/plugins/
  GOAL  kind        docs/guide/goals.md
  page  place       docs/
  code  complement  crates/, scripts/

citations
  FS    should  GOAL
  GOAL  —
  page  must    GOAL
  code  must    FS

must = check error · warn = check warning · should = suggestion (check --suggestions)
may = allowed · must not / warn not / should not = forbidden at that strength
A|B = one obligation, met by citing either · — = no direction rule names it
must/warn/should skip an uncited file of a place or the complement · [rules.citations.<ROW>.grounding] makes it a finding
```

**The ownership rule, in one line:** a file listed in a row's `files` belongs to that row; otherwise the deepest folder holding it decides; and a file in no place belongs to `code` ([§FS-config-v2.schema.places.ownership](../../docs/functional-spec/FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)).

Each row of the view is one part of that rule at work:

- **Two folders, one kind.** `FS` lists `docs/spec/core` and `docs/spec/plugins`, and the view names both. Each folder keeps its own index, `README.md`, which lists only the declarations that folder owns: `FS-login` in `core/`, `FS-plugin-load` in `plugins/`.
- **The deepest folder wins.** Both `FS` folders sit inside `page`'s `docs/`. A file in `docs/spec/core/` is in two places, and the deeper one, `FS`'s, owns it. So `FS-login` is in its kind's home, and the `page` rule does not ask it to cite `GOAL`. Under version 1, a file in two nested homes falls to the complement instead ([§FS-config.3.9.2](../../docs/functional-spec/FS-config.md#392-the-homeless-kind)).
- **A listed file beats its folder.** `docs/guide/goals.md` is inside `docs/`, but `GOAL` lists it, so it is `GOAL`'s and may declare `GOAL-ship`. `docs/guide/intro.md`, listed by no one, is a `page`, and must cite `GOAL`.
- **`code` with places is the complement.** The `code` row lists `crates` and `scripts`, so the view draws it as the complement with those places rather than as `every scanned file no other row claims`, and draws no second complement. The scan is the places: `notes/scratch.md` cites an ID nobody declares, and no place reads it.

`grund check examples/places/repo` prints `success`. Delete the `files` line from the `GOAL` row and the goals file falls back to `page`, which declares nothing:

```text
docs/guide/goals.md:1: error: GOAL-ship must not be declared in docs/ (not a citable home)
```

The trade-offs:

- One place belongs to one row. Writing `docs` in two rows, or `docs` and `./docs/`, or two names that link to one directory, is refused at the later entry ([§FS-config-v2.schema.places.boundaries](../../docs/functional-spec/FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)).
- Listing places on `code` narrows what is read. Without a `code` row, or with one that lists no place, the complement is the whole config root and every file is read. `grund check --full` still reads past the places.
- `scan = false` covers every place of its row. Places that differ in it must be two rows.

The [structure guide](../../docs/user-facing/structure.md) shows when to split a kind across folders, and the [configuration reference](../../docs/user-facing/configuration.md) lists every row key.

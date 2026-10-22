# Migrating a config to version 2

`grund config migrate --to 2` shows the version-2 `grund.toml` that means what
your version-1 config means, and writes it only when you ask and only when it
does ([§FS-config-migrate](../functional-spec/FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)).
It never changes what `grund check` reports: if version 2 cannot say the same
thing, it refuses and names the version-1 edit that would let it. The
[`examples/migrate`](../../examples/migrate/) walkthrough runs every step below
on a small repository, and its recorded output is what `grund` prints.

```bash
grund config migrate --to 2                          # preview: nothing is written
grund config migrate --to 2 --write                  # write, past both gates
grund config migrate --to 2 --write --full           # write with candidate and complete report
grund config migrate --to 2 --write --drop-comments  # and drop the listed comments
```

`PATH` (default `.`) finds the config the way `grund config validate` does
([§FS-config-migrate.1.1](../functional-spec/FS-config-migrate.md#11-which-file)).
`--to 2` is required; nothing else is a target version.

## 1. Preview first

Run it without `--write`. Stdout is the candidate, exactly the bytes `--write`
would write, so `grund config migrate --to 2 > candidate.toml` keeps it to read
([§FS-config-migrate.2.1](../functional-spec/FS-config-migrate.md#21-stdout-is-the-candidate)).
Stderr is the report: what was refused, the comments with nowhere to go, what
was dropped or changed, carried comments to check after a value-vocabulary
translation, where each version-1 key went, the defaults written out, and
whether the two gates passed
([§FS-config-migrate.2.2](../functional-spec/FS-config-migrate.md#22-stderr-is-the-report)).
The exit is `0` when `--write` would succeed, `1` when it would refuse, and `2`
when there is no config or the command was used wrongly
([§FS-config-migrate.2.3](../functional-spec/FS-config-migrate.md#23-exits)).

In [`examples/migrate/migrate-1-refused`](../../examples/migrate/migrate-1-refused/)
the preview refuses twice, `named_sections = false` and an `include` entry that
does not exist, and stdout is empty because nothing would be written.

## 2. Repair under version 1

A refusal is never resolved by migration. Each one names a repair, and every
repair is an edit to the version-1 file
([§FS-config-migrate.3.1](../functional-spec/FS-config-migrate.md#31-a-refusal-names-a-v1-repair)).
Make it, run `grund check` while the file is still version 1 to see what it
changes, and commit it on its own. That commit is the change of meaning, and it
is yours to review; the migration that follows changes none.

[`examples/migrate/migrate-2-repaired`](../../examples/migrate/migrate-2-repaired/)
is the same repository after both repairs. Its preview prints the candidate.

## 3. Check carried comments and read the listed comments

A comment above a key that maps to exactly one version-2 key moves with it, and
the header block at the top of the file stays at the top. A comment with no
single key to follow, such as one above `[scan] include`, which becomes several
version-2 keys, is listed by its lines
([§FS-config-migrate.4](../functional-spec/FS-config-migrate.md#4-comments)).

When a carried comment follows a setting whose value vocabulary changes, the
report names its original location and destination and asks you to check it
([§FS-config-migrate.4.1.1](../functional-spec/FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes)).
For example, `section_heading_levels = "strict" # strict | warn | loose`
becomes `heading_depth = "must" # strict | warn | loose`: the comment stays,
but its listed values need updating to `must | warn | may`. The warning also
appears for neutral prose, since migration does not judge what comments mean.
Check these comments in the candidate, in preview or after `--write`;
`--drop-comments` retains them and does not silence their warnings.

`--write` refuses while any comment is listed. Read them; move what still
matters into the version-1 file next to a key that maps one-to-one, or accept
that they go. Only then add `--drop-comments`, which drops exactly the comments
the preview listed
([§FS-config-migrate.4.4](../functional-spec/FS-config-migrate.md#44-listed-comments-and---drop-comments)).

## 4. The two gates

Before anything is written, the candidate is held to two comparisons
([§FS-config-migrate.5](../functional-spec/FS-config-migrate.md#5-equivalence-two-gates)):

- **the records**: the candidate, loaded as any version-2 file is, means the
  same project as the version-1 file;
- **the report**: `grund check --suggestions` on this tree, run once with each
  file, gives the same findings and the same exit.

When both pass the preview or full report says `compared:`; compact successful
writes run the same gates without repeating that line. When one differs it
prints the differing lines, `- ` for version 1 and `+ ` for the candidate, and refuses:
the candidate would change a meaning, so nothing is written
([§FS-config-migrate.5.4](../functional-spec/FS-config-migrate.md#54-the-result-and-a-difference)).

The `AGENTS.md` block `grund init` would write is compared too, whether or not
the file exists yet. A difference `grund check` would hold the tree to is
refused: most often a `values = true` row, which version 2 gives a declaration
shape that adds a `Declaration shapes` section
([§FS-config-migrate.5.3](../functional-spec/FS-config-migrate.md#53-guidance)).

## 5. Write

Add `--write` (and `--drop-comments` if the preview listed comments). The file is
replaced atomically, keeps its permissions, and nothing else is written; if the
file changed since it was read, nothing is written and the run says so
([§FS-config-migrate.6](../functional-spec/FS-config-migrate.md#6-the-write)).

A successful write leaves stdout empty. Its compact stderr report starts with
what was written and the exact original ranges of any dropped comments, then
retains every notice and carried-comment warning, member migration commands
and the distinct Project map note when applicable. It explains how to undo the
write and preview again, then ends with
([§FS-config-migrate.6.5](../functional-spec/FS-config-migrate.md#65-what-it-says-after-the-write)):

```text
note: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them
```

Run `grund init` in the migrated project to refresh its managed instructions;
migration leaves them unchanged
([§FS-config-migrate.6.5](../functional-spec/FS-config-migrate.md#65-what-it-says-after-the-write)).
The qualified reminder appears after every successful write, even when no
`AGENTS.md` exists. Previews, refused or failed writes, and already-version-2
runs omit it. A second migration run says the config is already version 2.

To capture the candidate and complete mapping/default/gate report while
applying, use `grund config migrate --to 2 --write --full`. Stdout then contains
exactly the bytes written, and stderr retains the preview's complete report
before the write outcome and advice. `--full` changes no preview, refusal,
failure or already-version-2 output, and can be combined with PATH,
`--drop-comments` and either `--path-base`
([§FS-config-migrate.1.3](../functional-spec/FS-config-migrate.md#13-the-other-flags)).

## Workspaces

Migration touches one config. A workspace root lists each member still on
version 1 with the command that migrates it, and the members are migrated one at
a time
([§FS-config-migrate.8](../functional-spec/FS-config-migrate.md#8-workspaces)).

## Why grund tells you about version 1

A `grund check`, `grund config validate` or `grund config show` on a config read
as version 1 — one that writes `grund_config_version = 1`, or no version at all —
says so on stderr, once per such config, where a clean `check` printed `success`:

```text
warning: grund.toml is version 1; migrate it with grund config migrate --to 2
```

In a workspace each member still on version 1 gets its own line, ending with the
directory to pass, and a config on version 2 or a project with no config file
gets none
([§FS-check.checks.config-version-1](../functional-spec/FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)).
The line is a caution and never becomes an error: the exit code does not move,
no release is named in which version 1 stops loading, and the file keeps meaning
what it means
([§FS-config.5.3](../functional-spec/FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired)).
Migrating is what ends it. Until then, `grund check --ignore config-version-1`
leaves it out and restores `success`; under `--format json` it is the stderr
finding with code `config-version-1`.

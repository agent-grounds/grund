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
was dropped or changed, where each version-1 key went, the defaults written
out, and whether the two gates passed
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

## 3. Read the listed comments

A comment above a key that maps to exactly one version-2 key moves with it, and
the header block at the top of the file stays at the top. A comment with no
single key to follow, such as one above `[scan] include`, which becomes several
version-2 keys, is listed by its lines
([§FS-config-migrate.4](../functional-spec/FS-config-migrate.md#4-comments)).
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

When both pass the report says `compared:`. When one differs it prints the
differing lines, `- ` for version 1 and `+ ` for the candidate, and refuses:
the candidate would change a meaning, so nothing is written
([§FS-config-migrate.5.4](../functional-spec/FS-config-migrate.md#54-the-result-and-a-difference)).

## 5. Write

Add `--write` (and `--drop-comments` if the preview listed comments). The file is
replaced atomically, keeps its permissions, and nothing else is written; if the
file changed since it was read, nothing is written and the run says so
([§FS-config-migrate.6](../functional-spec/FS-config-migrate.md#6-the-write)). The
report ends with how to get version 1 back, from version control. A second run
says the config is already version 2.

## Workspaces

Migration touches one config. A workspace root lists each member still on
version 1 with the command that migrates it, and the members are migrated one at
a time
([§FS-config-migrate.8](../functional-spec/FS-config-migrate.md#8-workspaces)).

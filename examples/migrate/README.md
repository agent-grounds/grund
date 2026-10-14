# Migrate: a version-1 config to version 2, previewed first

This runnable walkthrough moves a version-1 `grund.toml` to version 2 with `grund config migrate --to 2` ([§FS-config-migrate](../../docs/functional-spec/FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)). It is for a repository `grund init` set up before version 2 existed. Its config is the one `grund init` writes, trimmed to two kinds and two languages, with three comments: a header block, one above the `FS` row, and one above `[scan] include`.

The example pins two runs, one per step ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). Neither writes anything.

## Step 1: the preview refuses, and names the repairs

`migrate-1-refused/repo/grund.toml` is the config as `grund init` left it: `named_sections = false`, and `include` still names `e2e`, a directory this repository never made. Run:

```bash
grund config migrate --to 2 examples/migrate/migrate-1-refused/repo
echo $?    # 1
```

Stdout is empty, because there is no file migration would write. Stderr says why, one refusal at a time, each with the version-1 edit that repairs it ([§FS-config-migrate.3.1](../../docs/functional-spec/FS-config-migrate.md#31-a-refusal-names-a-v1-repair)):

```text
grund.toml: version 1 → 2, preview, nothing written
refused: grund.toml:13: [id] named_sections = false turns named sections off, and version 2 always reads them
  repair: set [id] named_sections = true under version 1; grund check shows its effect
refused: grund.toml:29: [scan] include names `e2e`, which does not exist, and version 2 must know whether it is a folder or a file
  repair: create e2e, or remove it from include; grund check under version 1 shows the effect
comments with no single key to follow (--write refuses; --drop-comments drops exactly these):
  grund.toml:28  above [scan] include
```

Version 2 always reads named sections, and it puts a folder and a file under different keys, so it cannot carry either setting with the same meaning. Migration does not guess. Each repair is a version-1 edit: make it, run `grund check` to see its effect while the file is still version 1, and commit it. That commit is the change of meaning, made and reviewed by you.

The comment above `include` is listed because `include` splits into several version-2 keys, so there is no one key for the comment to follow ([§FS-config-migrate.4.3](../../docs/functional-spec/FS-config-migrate.md#43-everything-else-is-listed)). The header block and the comment above the `FS` row are not listed: they are carried.

## Step 2: after the repairs, the candidate

`migrate-2-repaired/repo/grund.toml` is the same file after both edits: `named_sections = true`, and `e2e` removed from `include`. Run the same preview:

```bash
grund config migrate --to 2 examples/migrate/migrate-2-repaired/repo
echo $?    # 1
```

Stdout is now the version-2 file, exactly the bytes `--write` would write ([§FS-config-migrate.2.1](../../docs/functional-spec/FS-config-migrate.md#21-stdout-is-the-candidate)); `migrate-2-repaired/expected.stdout` holds it. The header block is still at the top, and the comment above the `FS` row is above `[schema.kinds.FS]`. Every version-1 default that differs from version 2's is written out, so the file keeps its meaning after later edits.

Stderr names the comment that would be lost, then maps each version-1 key to where it went and lists the defaults written out:

```text
grund.toml: version 1 → 2, preview, nothing written
comments with no single key to follow (--write refuses; --drop-comments drops exactly these):
  grund.toml:28  above [scan] include
…
compared: the records are equal, and grund check --suggestions on this tree gives the same findings and the same exit before and after
note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check does not compare that table
```

The `compared:` line is the two checks every write must pass: the records both files lower into are equal, and `grund check` on this tree gives the same findings with either ([§FS-config-migrate.5](../../docs/functional-spec/FS-config-migrate.md#5-equivalence-two-gates)). It still exits 1, because `--write` would refuse while the listed comment would be lost.

## Writing it

Read the listed comment, then write:

```bash
grund config migrate --to 2 --write --drop-comments
```

The write is atomic, writes no other file, and says how to get version 1 back: from version control, or by previewing again ([§FS-config-migrate.6](../../docs/functional-spec/FS-config-migrate.md#6-the-write)). Without `--drop-comments` it refuses, and the file stays as it is. Run it a second time and it reports that the config is already version 2.

## Trade-offs

- Most configs `grund init` wrote are refused on the first preview, for `named_sections` and for default `include` entries that never existed. Each repair is one line, and `grund check` shows its effect before anything is migrated.
- The trimmed extension list keeps the candidate short. A config with `grund init`'s full list migrates the same way, with one definition per way version 1 read those files.

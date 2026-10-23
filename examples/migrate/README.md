# Migrate: a version-1 config to version 2, previewed first

This runnable walkthrough moves a version-1 `grund.toml` to version 2 with `grund config migrate --to 2` ([§FS-config-migrate](../../docs/functional-spec/FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)). It is for a repository `grund init` set up before version 2 existed. Its config is the one `grund init` writes, trimmed to two kinds and two languages, with four comments: a header block, one above the `FS` row, one above `[scan] include`, and one trailing `[scan] exclude`.

The example pins four runs, one per step ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). The first two preview; the third writes with compact output and the fourth with complete output, each changing only the config in a disposable copy.

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
changed: grund.toml:30: [scan] exclude: value vocabulary translated to [schema.sources] exclude; check the carried comment
```

Version 2 always reads named sections, and it puts a folder and a file under different keys, so it cannot carry either setting with the same meaning. Migration does not guess. Each repair is a version-1 edit: make it, run `grund check` to see its effect while the file is still version 1, and commit it. That commit is the change of meaning, made and reviewed by you.

The comment above `include` is listed because `include` splits into several version-2 keys, so there is no one key for the comment to follow ([§FS-config-migrate.4.3](../../docs/functional-spec/FS-config-migrate.md#43-everything-else-is-listed)). The header block and the comment above the `FS` row are not listed: they are carried.

Adding `--write` here still exits `1`, prints no stdout, and leaves the config unchanged. Its report keeps the repairs and comment listing above and ends with:

```text
not written: 2 refusals (follow the repairs above); 1 comment has no single key to follow (move it, or re-run with --drop-comments); grund.toml is unchanged
```

The closing line names every blocking cause, so `--drop-comments` alone cannot clear the two setting refusals. A refused write prints no `written:` defaults or next-init Project map note, even after repairing the settings so only the comment blocks it. `--full` keeps the same refused-write report ([§FS-config-migrate.2.2](../../docs/functional-spec/FS-config-migrate.md#22-stderr-is-the-report)). The CLI regression also checks this run against [`migrate-1-refused/expected.write.stderr`](migrate-1-refused/expected.write.stderr); the preview expectation stays unchanged.

## Step 2: after the repairs, the candidate

`migrate-2-repaired/repo/grund.toml` is the same file after both edits: `named_sections = true`, and `e2e` removed from `include`. Run the same preview:

```bash
grund config migrate --to 2 examples/migrate/migrate-2-repaired/repo
echo $?    # 1
```

Stdout is now the version-2 file, exactly the bytes `--write` would write ([§FS-config-migrate.2.1](../../docs/functional-spec/FS-config-migrate.md#21-stdout-is-the-candidate)); `migrate-2-repaired/expected.stdout` holds it. The header block is still at the top, and the comment above the `FS` row is above `[schema.kinds.FS]`. Every version-1 default that differs from version 2's is written out, so the file keeps its meaning after later edits.

Stderr names the comment that would be lost, warns about the carried comment on the translated exclusion values, then maps each version-1 key to where it went and lists the defaults written out:

```text
grund.toml: version 1 → 2, preview, nothing written
comments with no single key to follow (--write refuses; --drop-comments drops exactly these):
  grund.toml:28  above [scan] include
changed: grund.toml:30: [scan] exclude: value vocabulary translated to [schema.sources] exclude; check the carried comment
…
compared: the records are equal, and grund check --suggestions on this tree gives the same findings and the same exit before and after
note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check reports the block stale until then
```

The `compared:` line is the two checks every write must pass: the records both files lower into are equal, and `grund check` on this tree gives the same findings with either ([§FS-config-migrate.5](../../docs/functional-spec/FS-config-migrate.md#5-equivalence-two-gates)). It still exits 1, because `--write` would refuse while the listed comment would be lost.

The exclusion comment is retained: the candidate contains `exclude = ["target/", "node_modules/", ".git/", "dist/", "build/", ".venv/"] # directory names`. Version 1's directory names became glob patterns. Check that the comment still explains the setting as intended; the warning identifies the translation without judging its prose ([§FS-config-migrate.4.1.1](../../docs/functional-spec/FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes)). It appears in preview and write, including with `--drop-comments`, which drops only the listed `include` comment.

## Step 3: write, then refresh managed instructions

`migrate-3-write/repo/` starts with the repaired version-1 repository. Read the listed comment, then write:

```bash
grund config migrate --to 2 --write --drop-comments examples/migrate/migrate-3-write/repo
echo $?    # 0
```

The write is atomic, writes no other file, and says how to get version 1 back: from version control, or by previewing again ([§FS-config-migrate.6](../../docs/functional-spec/FS-config-migrate.md#6-the-write)). Without `--drop-comments` it refuses, and the file stays as it is. Run it a second time and it reports that the config is already version 2.

Stdout is empty. The complete compact report on stderr is:

```text
wrote grund.toml: version 1 (35 lines) replaced by version 2
dropped the comments at grund.toml:28 (version 1 lines)
changed: grund.toml:30: [scan] exclude: value vocabulary translated to [schema.sources] exclude; check the carried comment
note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check reports the block stale until then
to get version 1 back: it is in version control, e.g. git restore grund.toml
to see this candidate again: restore version 1 and run grund config migrate --to 2
note: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them
```

Run `grund init` in the migrated project to refresh managed instructions that may still describe version 1 ([§FS-config-migrate.6.5](../../docs/functional-spec/FS-config-migrate.md#65-what-it-says-after-the-write)). The reminder appears even though this fixture has no `AGENTS.md`; migration does not create one. `migrate-3-write/expected.repo/` records the complete final tree: only `grund.toml` changes, and it equals the candidate in `migrate-2-repaired/expected.stdout`. The harness uses a disposable copy for the write and checks that tree; the follow-up `grund init` is a separate action.

## Step 4: request complete output when writing

`migrate-4-full/repo/` starts from the same repaired version-1 fixture, so this
is a separate write rather than a second migration of step 3's version-2 file:

```bash
grund config migrate --to 2 --write --drop-comments --full examples/migrate/migrate-4-full/repo
echo $?    # 0
```

`--full` restores the candidate on stdout and the complete report on stderr
([§FS-config-migrate.1.3](../../docs/functional-spec/FS-config-migrate.md#13-the-other-flags)).
`migrate-4-full/expected.stdout` is byte-identical to the preview candidate and
the written `expected.repo/grund.toml`. Stderr preserves the preview's comments,
notices, key mappings, defaults, gate result and Project map note in their
original order, then adds the write outcome, dropped ranges and advice. The
refresh reminder still appears once and last. `--full` leaves preview and
refusal output unchanged, and both write modes make the same config-only change.

## Trade-offs

- Most configs `grund init` wrote are refused on the first preview, for `named_sections` and for default `include` entries that never existed. Each repair is one line, and `grund check` shows its effect before anything is migrated.
- The trimmed extension list keeps the candidate short. A config with `grund init`'s full list migrates the same way, with one definition per way version 1 read those files.

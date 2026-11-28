# Migrate: a version-1 config to version 2, previewed first

This runnable walkthrough moves a version-1 `grund.toml` to version 2 with `grund config migrate --to 2` ([§FS-config-migrate](../../docs/functional-spec/FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does)). It is for a repository `grund init` set up before version 2 existed. Its config is the one `grund init` writes, trimmed to two kinds and two languages, with four comments: a header block, one above the `FS` row, one above `[scan] include`, and one trailing `[scan] exclude`.

The example pins seven runs, one per step ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). The first two preview; the third writes with compact output and the fourth with complete output, each changing only the config in a disposable copy. The fifth migrates a different tree, a workspace block with `include_root = false`. The last two take a migrated file one step further, onto version 2's built-in languages: the sixth previews that and the seventh writes it.

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
  grund.toml:28  above [scan] include: destinations are unavailable; retain the prose, repair the settings or candidate loading problem above and preview again before choosing its placement
changed: grund.toml:30: [scan] exclude: value vocabulary translated to [schema.sources] exclude; check the carried comment
would not write: 2 refusals (follow the repairs above); 1 comment has no single key to follow (follow the comment guidance above)
```

The last line is what would block the write, in the words step 1's `--write` uses below ([§FS-config-migrate.2.2](../../docs/functional-spec/FS-config-migrate.md#22-stderr-is-the-report)). Version 2 always reads named sections, and it puts a folder and a file under different keys, so it cannot carry either setting with the same meaning. Migration does not guess. Each repair is a version-1 edit: make it, run `grund check` to see its effect while the file is still version 1, and commit it. That commit is the change of meaning, made and reviewed by you.

The comment above `include` is listed because `include` splits into several version-2 keys, so there is no one key for the comment to follow ([§FS-config-migrate.4.3](../../docs/functional-spec/FS-config-migrate.md#43-everything-else-is-listed)). Here its destinations are unavailable until the missing path is repaired; save the prose and preview again after the repairs. The header block and the comment above the `FS` row are not listed: they are carried.

Adding `--write` here still exits `1`, prints no stdout, and leaves the config unchanged. Its report keeps the repairs and comment listing above and ends with the same causes:

```text
not written: 2 refusals (follow the repairs above); 1 comment has no single key to follow (follow the comment guidance above); grund.toml is unchanged
```

The closing line names every blocking cause, so `--drop-comments` alone cannot clear the two setting refusals. A refused write prints no `written:` defaults or next-init Project map note, even after repairing the settings so only the comment blocks it. `--full` keeps the same refused-write report ([§FS-config-migrate.2.2](../../docs/functional-spec/FS-config-migrate.md#22-stderr-is-the-report)). The CLI regression also checks this run against [`migrate-1-refused/expected.write.stderr`](migrate-1-refused/expected.write.stderr).

## Step 2: after the repairs, only the comment refuses

`migrate-2-repaired/repo/grund.toml` is the same file after both edits: `named_sections = true`, and `e2e` removed from `include`. Run the same preview:

```bash
grund config migrate --to 2 examples/migrate/migrate-2-repaired/repo
echo $?    # 1
```

The settings no longer refuse, so stderr maps each version-1 key to where it went and runs the two gates. The comment above `include` still has no single key to follow, and `--write` would refuse while it would be lost, so the preview still exits `1`, prints no candidate, and ends with that one cause ([§FS-config-migrate.2.1](../../docs/functional-spec/FS-config-migrate.md#21-stdout-is-the-candidate)):

```text
grund.toml: version 1 → 2, preview, nothing written
comments with no single key to follow (--write refuses; --drop-comments drops exactly these):
  grund.toml:28  above [scan] include: setting splits or merges into version-2 settings: [schema.kinds.code] folders, docs and src; requirements.md is a place of [schema.kinds.FS] already; retain the prose, authorize the listed drops with --write --drop-comments, then re-add it at one of these named destinations after a successful write (subject to all other refusals and gates)
changed: grund.toml:30: [scan] exclude: value vocabulary translated to [schema.sources] exclude; check the carried comment
…
compared: the records are equal, and grund check --suggestions on this tree gives the same findings and the same exit before and after
would not write: 1 comment has no single key to follow (follow the comment guidance above)
```

The `compared:` line is the two checks every write must pass: the records both files lower into are equal, and `grund check` on this tree gives the same findings with either ([§FS-config-migrate.5](../../docs/functional-spec/FS-config-migrate.md#5-equivalence-two-gates)).

The candidate exists all the same; only the comment refuses. Add `--drop-comments` to preview it with exit `0`, or `--full` to print it with the complete report, the `written:` defaults and the next-init note, before the same closing line ([§FS-config-migrate.1.3](../../docs/functional-spec/FS-config-migrate.md#13-the-other-flags)). It is the version-2 file, exactly the bytes `--write` would write; step 4 pins it as [`migrate-4-full/expected.stdout`](migrate-4-full/expected.stdout). The header block is still at the top, and the comment above the `FS` row is above `[schema.kinds.FS]`. Every version-1 default that differs from version 2's is written out, so the file keeps its meaning after later edits.

The exclusion comment is retained: the candidate contains `exclude = ["target/", "node_modules/", ".git/", "dist/", "build/", ".venv/"] # directory names`. Version 1's directory names became glob patterns. Check that the comment still explains the setting as intended; the warning identifies the translation without judging its prose ([§FS-config-migrate.4.1.1](../../docs/functional-spec/FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes)). It appears in preview and write, including with `--drop-comments`, which drops only the listed `include` comment.

The listed entry itself names the destinations: `docs` and `src` become
`[schema.kinds.code]` folders, while `requirements.md` is already a place of
`[schema.kinds.FS]`. Moving the comment one line up above `[scan]` still leaves
it on a split header. To preserve it, save `# What the walk reads besides the
homes.`, authorize the listed drops in step 3, then re-add the saved prose above
`[schema.kinds.code]` after the write succeeds
([§FS-config-migrate.2.2](../../docs/functional-spec/FS-config-migrate.md#22-stderr-is-the-report)).

## Step 3: write, then refresh managed instructions

`migrate-3-write/repo/` starts with the repaired version-1 repository. Save the listed comment's prose somewhere outside the config, then write:

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
to get version 1 back: it is in version control, e.g. git restore examples/migrate/migrate-3-write/repo/grund.toml
to see this candidate again: restore version 1 and run grund config migrate --to 2 examples/migrate/migrate-3-write/repo --drop-comments
note: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them
```

The `wrote` line and the notices are report paths, named from the migrated project's root. The undo and the re-preview are commands to paste where you ran `grund config migrate`, so they name the config from there and carry the path you gave ([§FS-config-migrate.6.5.1](../../docs/functional-spec/FS-config-migrate.md#651-the-commands-a-migrate-report-prints-are-spelled-from-the-working-directory)).

Run `grund init` in the migrated project to refresh managed instructions that may still describe version 1 ([§FS-config-migrate.6.5](../../docs/functional-spec/FS-config-migrate.md#65-what-it-says-after-the-write)). The reminder appears even though this fixture has no `AGENTS.md`; migration does not create one. `migrate-3-write/expected.repo/` records the complete final tree: only `grund.toml` changes, and it equals the candidate step 4 prints in `migrate-4-full/expected.stdout`. The harness uses a disposable copy for the write and checks that tree; the follow-up `grund init` is a separate action.

To try the optional manual recovery after the successful write above, re-add
the saved comment with this command from the repository root:

```bash
python3 - <<'PY'
from pathlib import Path

config = Path("examples/migrate/migrate-3-write/repo/grund.toml")
text = config.read_text()
assert "grund_config_version = 2" in text
config.write_text(text.replace(
    "[schema.kinds.code]",
    "# What the walk reads besides the homes.\n[schema.kinds.code]",
    1,
))
PY
```

This follow-up edit preserves the comment at the named destination. It is
separate from the migration's recorded final tree. Drop consent still requires
every setting refusal to be repaired and both comparison gates to pass
([§FS-config-migrate.4.4](../../docs/functional-spec/FS-config-migrate.md#44-listed-comments-and---drop-comments)).

## Step 4: request complete output when writing

`migrate-4-full/repo/` starts from the same repaired version-1 fixture, so this
is a separate write rather than a second migration of step 3's version-2 file:

```bash
grund config migrate --to 2 --write --drop-comments --full examples/migrate/migrate-4-full/repo
echo $?    # 0
```

`--full` restores the candidate on stdout and the complete report on stderr
([§FS-config-migrate.1.3](../../docs/functional-spec/FS-config-migrate.md#13-the-other-flags)).
`migrate-4-full/expected.stdout` is byte-identical to the candidate step 2's
`--drop-comments` preview prints and to the written `expected.repo/grund.toml`. Stderr preserves the preview's comments,
notices, key mappings, defaults, gate result and Project map note in their
original order, then adds the write outcome, dropped ranges and advice. The
refresh reminder still appears once and last. `--full` changes no refused
write, and no preview except one only listed comments refuse, and both write
modes make the same config-only change.

## Step 5: a block with `include_root = false` migrates to its workspace table

`migrate-5-block/repo/` is a version-2 workspace root that lists `group`. `group/grund.toml` is a version-1 block that only lists its member, `group/a`, and sets `include_root = false`, so no project reads its own files:

```toml
[workspace]
include_root = false
members = ["a"]
```

From the workspace root, migrate the block:

```bash
cd examples/migrate/migrate-5-block/repo
grund config migrate --to 2 --write group
echo $?    # 0
```

Neither the `named_sections` refusal nor the default-`include` refusal of step 1 applies here: nothing reads the block, and none of the default include entries or kind homes exist outside its member ([§FS-config-migrate.8.1](../../docs/functional-spec/FS-config-migrate.md#81-a-block-with-include_root--false)). The written file is the version line and the `[workspace]` table, and the report says the block takes version 2's defaults rather than writing version 1's out ([§FS-config-migrate.8.1](../../docs/functional-spec/FS-config-migrate.md#81-a-block-with-include_root--false)):

```text
wrote grund.toml: version 1 (3 lines) replaced by version 2
dropped: grund.toml: version 1's defaults: include_root = false, so no project scans this block's own files; it takes version 2's defaults
```

`migrate-5-block/expected.repo/group/grund.toml` is the written file:

```toml
grund_config_version = 2

[workspace]
members = ["a"]
include_root = false
```

Both gates still ran. Put a `README.md` or an `AGENTS.md` beside the block's config and the same command exits `1` at the report gate, because version 2 would read that file and warn that nobody scans the block; the block stays on version 1 until the file moves into a member ([§FS-config-migrate.8.1](../../docs/functional-spec/FS-config-migrate.md#81-a-block-with-include_root--false)). The report names the file `grund.toml`, from the block's own directory, as every migration report does by default; pass `--path-base=invocation` to see `group/grund.toml`. The undo and the re-preview it closes with already name the block from the workspace root, because they are commands to run there: `git restore group/grund.toml` and `grund config migrate --to 2 group` ([§FS-config-migrate.6.5.1](../../docs/functional-spec/FS-config-migrate.md#651-the-commands-a-migrate-report-prints-are-spelled-from-the-working-directory)).

## Step 6: preview the built-in languages for what migration kept

Migration writes a built-in language only where it reads a suffix exactly as version 1 did, as `markdown` does `.md` under version 1's default comment prefixes ([§FS-config-v2.schema.sources.migration](../../docs/functional-spec/FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes)). Every other suffix keeps a `v1_*` definition, so nothing `grund check` reports moves. `grund config adopt-builtins` is the step after that: it reads each of those suffixes through the built-in language that claims it, and says every reading that changes ([§FS-config-adopt-builtins](../../docs/functional-spec/FS-config-adopt-builtins.md#fs-config-adopt-builtins-grund-config-adopt-builtins-reads-a-version-2-files-suffixes-through-the-built-in-languages-that-claim-them-says-every-reading-that-changes-and-writes-only-what-it-planned-only-when-asked)).

`migrate-6-adopt/repo/grund.toml` is what `grund config migrate --to 2 --write` writes for a version-1 project that read `md`, `java`, `gradle` and `sh` through `comment_prefixes = ["//", "#", "*", "/*"]`. No built-in reads a suffix through exactly those hosts, so every suffix kept its definition:

```toml
[schema.sources]
languages = ["v1_markdown", "v1_c_family", "v1_plain", "v1_shell"]
…
[schema.sources.extensions]
md = "v1_markdown"
java = "v1_c_family"
gradle = "v1_plain"
sh = "v1_shell"
```

Preview the adoption:

```bash
grund config adopt-builtins examples/migrate/migrate-6-adopt/repo
echo $?    # 0
```

Stdout is the candidate, exactly the bytes `--write` would write ([§FS-config-adopt-builtins.5.1](../../docs/functional-spec/FS-config-adopt-builtins.md#51-stdout-is-the-candidate)). Stderr is the plan, suffix by suffix, and what `grund check --suggestions` reports with the file and with the candidate ([§FS-config-adopt-builtins.5.2](../../docs/functional-spec/FS-config-adopt-builtins.md#52-stderr-is-the-report)):

```text
grund.toml: adopting version 2's built-in languages, preview, nothing written
adopts: md: definition v1_markdown → built-in markdown
  reads differently: comments behind `;` and `--` are now read
adopts: java: definition v1_c_family → built-in java
  reads differently: `#` is no longer a comment host
adopts: sh: definition v1_shell → built-in shell, opt-in, listed in languages
  reads differently: `//`, `/*` and `*` are no longer comment hosts
added: bash, zsh: admitted with shell
kept: gradle: definition v1_plain and gradle = "v1_plain"; version 2 has no built-in language for `.gradle`
compared: the candidate reads as planned; grund check --suggestions: exit 1 → 1, no finding changes
```

Each `reads differently:` line is a change you are agreeing to: after the write, a `#` in a Java file no longer starts a comment, and `.sh` reads only `#` ([§FS-config-adopt-builtins.5.3](../../docs/functional-spec/FS-config-adopt-builtins.md#53-reading-differences)). `shell` reads `.bash` and `.zsh` as well, so `src/extra.bash`, which nothing read before, is now scanned. `.gradle` has no built-in language and keeps `v1_plain`. A kept suffix never refuses. The `compared:` line shows that none of this moves a finding on this tree: the one error before and after is the reference to `FS-typo` in `src/build.gradle`, which `v1_plain` still reads.

Adoption plans before it writes the candidate, and refuses any candidate that reads a suffix otherwise than planned, so a mistake in the edit cannot change a reading the report did not name ([§FS-config-adopt-builtins.6.2](../../docs/functional-spec/FS-config-adopt-builtins.md#62-the-readings-are-the-plan)).

## Step 7: write, and check what it read

`migrate-7-adopt-write/repo/` starts from the same migrated file. Write the adoption:

```bash
grund config adopt-builtins --write examples/migrate/migrate-7-adopt-write/repo
echo $?    # 0
```

Stdout is empty. Stderr repeats step 6's report and ends with the write ([§FS-config-adopt-builtins.8.2](../../docs/functional-spec/FS-config-adopt-builtins.md#82-what-it-says-after-the-write)):

```text
wrote grund.toml: adopted built-in markdown, java and shell; kept v1_plain for gradle
to get the previous file back: it is in version control, e.g. git restore grund.toml
to see this candidate again: restore it and run grund config adopt-builtins
```

`migrate-7-adopt-write/expected.repo/grund.toml` is the written file. `languages` names the built-ins in place of the definitions they replace, and only the `.gradle` entry and its definition remain:

```toml
[schema.sources]
languages = ["markdown", "java", "v1_plain", "shell"]
…
[schema.sources.extensions]
gradle = "v1_plain"

[schema.sources.definitions.v1_plain]
class = "source"
comment_prefixes = ["//", "#", "*", "/*"]
doc_comments = "none"
docstrings = "off"
```

The write replaces only the config, atomically, and only after the whole gate passed in the same run ([§FS-config-adopt-builtins.8.1](../../docs/functional-spec/FS-config-adopt-builtins.md#81-bounds)). Run `grund check` in the written tree. It reports what step 6's comparison showed on both sides, the typo in the file `v1_plain` still reads:

```text
src/build.gradle:2: error: unknown reference FS-typo
```

A second `grund config adopt-builtins` says there is nothing to adopt, and names `.gradle` as kept again ([§FS-config-adopt-builtins.9](../../docs/functional-spec/FS-config-adopt-builtins.md#9-a-second-run-is-a-no-op)).

## Trade-offs

- Most configs `grund init` wrote are refused on the first preview, for `named_sections` and for default `include` entries that never existed. Each repair is one line, and `grund check` shows its effect before anything is migrated.
- The trimmed extension list keeps the candidate short. A config with `grund init`'s full list migrates the same way: a built-in language for each suffix version 2 reads exactly as version 1 did, and one definition per other way version 1 read those files. Step 6 moves the rest onto the built-ins where you accept the readings it names; a suffix no built-in claims, such as `.gradle`, stays on its definition.
- Adoption changes readings on purpose, so it is a separate, reviewed step rather than part of migration: migration never changes what `grund check` reports, and adoption names every reading it changes before it writes.

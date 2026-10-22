# FS-config-migrate: grund config migrate --to 2 shows the version-2 file that means what a version-1 config means, and writes it only when asked and only when it does

`grund config migrate --to 2` reads a version-1 `grund.toml` and shows the version-2 file that means the same thing, without touching anything. It says, line by line, where each setting went, what it drops and why, and every comment it cannot carry. Where a v1 setting has no v2 spelling it refuses and names the v1 edit that gets to one, so every change of meaning is one the user made, checked with `grund check`, and committed. It writes only with `--write`, only after two comparisons show that the new file means what the old one did, and only by an atomic replace. That is the one exception to "the config is never overwritten" ([§REQ-no-data-loss.4](../requirements/REQ-no-data-loss.md#4-one-migration-may-replace-the-config)). The translation is held to [§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart) and [§FS-config-v2.schema.sources.migration](FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes); the full table of every v1 key is [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default). It serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path): the way off version 1 is a reviewed path, not a hand rewrite nothing checks. Why it refuses rather than offers a flag, why there are two gates, and why `--to` is required are [§DF-config-migrate](../decisions/functional/DF-config-migrate.md#df-config-migrate-migration-refuses-what-version-2-cannot-spell-names-a-version-1-repair-and-writes-only-past-two-gates).

## terms: Terms

Leans on [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (config root, workspace, member), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding) and [§FS-config-v2.terms](FS-config-v2.md#terms-terms) (reader, place, complement). Defines:

- **Candidate**: the version-2 file migration would write for the config as it stands, with the same flags. It exists only when nothing is refused.
- **Preview**: a run without `--write`. It prints the candidate and the report and writes nothing.
- **Refusal**: a v1 setting that version 2 cannot spell with the same meaning. It is never approximated.
- **Repair**: the v1 edit a refusal names. The user makes it, checks it under version 1, and commits it before migrating.
- **Notice**: a v1 setting that is dropped, or written differently, because version 1 gave it no force. It is reported and does not refuse.
- **Key map**: each authored v1 key with its line and the v2 key it became.
- **Attached comment**: a run of comment lines directly above a key or table header, with no blank line between, or a trailing comment on that line.
- **Header block**: the comment lines from line 1 to the first blank or non-comment line.
- **Listed comment**: a comment migration cannot carry, named in the report with its lines.
- **Gate**: one of the two comparisons a write must pass, the records gate and the report gate ([§FS-config-migrate.5](FS-config-migrate.md#5-equivalence-two-gates)).

## 1. The command

```text
grund config migrate --to 2 [PATH] [--write] [--full] [--drop-comments] [--path-base project|invocation]
```

### 1.1 Which file

`PATH` works as it does for `config validate` and `config show` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path)). Discovery walks up from it, `.` when it is omitted, and migrates the config it finds where it is: the root `grund.toml`, or the deprecated `.agents/grund.toml` ([§FS-config.1](FS-config.md#1-file-location-and-discovery)). Where one directory carries both, it migrates the one that governs ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)). A path inside a workspace member migrates that member's config ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces)). Exactly one file is migrated per run.

### 1.2 `--to` is required

`--to 2` is the only accepted target. A run without it is a usage error, exit `2`: `` error: config migrate needs --to 2 (the version to migrate to) ``. `--to` with any other value, `1` included, is exit `2`: `` error: config migrate converts to version 2 only, not `<value>` ``. Both are answered before anything is loaded ([§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read--realized-directional-for-the-next-flag)).

### 1.3 The other flags

- `--write` replaces the config with the candidate, on the terms of [§FS-config-migrate.6](FS-config-migrate.md#6-the-write).
- `--full` is migration-only. After a successful write it restores the candidate on stdout and the complete report on stderr; without it successful writes use the compact report of [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write). On previews it changes nothing. It accepts `PATH`, `--write`, `--drop-comments` and either `--path-base`; it does not relax `--to 2` or permit `--format`.
- `--drop-comments` lets the write drop exactly the listed comments ([§FS-config-migrate.4.4](FS-config-migrate.md#44-listed-comments-and---drop-comments)). It is accepted without `--write`, so a preview can predict the write.
- `--path-base` is accepted and spells the paths of the report ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)).
- `--format` is refused as it is for every `config` command ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)), with `` error: unknown flag `--format` ``, exit `2`, as `config show` answers it.

## 2. Streams and exits

### 2.1 Stdout is the candidate

On a preview, stdout carries exactly the bytes `--write` would write with the same flags, and nothing else ([§FS-errors.1](FS-errors.md#1-streams)). A successful write has empty stdout by default; with `--full` it prints exactly the candidate it wrote. The output choice never changes candidate bytes. When there is no candidate, because something is refused or the config is already version 2, stdout is empty, and so it is for a refused or failed write, with or without `--full`. So `grund config migrate --to 2 > new.toml` captures only the candidate. The candidate is a version-2 file, so it prints its tables keyed by a row name in the order a version-2 file prints in ([§FS-config.4.2.4](FS-config.md#424-the-order-show-prints-in)): the rows' order, which is the order of the version-1 `[[kinds]]` entries. A preview that exits `1` only because a listed comment would be lost still prints the candidate, without those comments, as `--drop-comments` would write it: the comments are what refuse, not the file.

### 2.2 Stderr is the report

Every other line goes to stderr. Only an actual successful write without `--full` uses the compact report defined in [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write). Previews and `--write --full` use the complete report below; refused writes retain that complete diagnostic report regardless of `--full`. Failures and already-version-2 runs retain their error or no-op messages and exits, without a refresh reminder. `<config>` is the config's path spelled from the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), `grund.toml` by default.

1. The heading: `<config>: version 1 → 2, preview, nothing written` for a preview, or `<config>: version 1 → 2, --write` with `--write`.
2. One block per refusal, in file order: `refused: <config>:<line>: <what version 2 cannot spell>`, then any indented detail lines, then `  repair: <the v1 edit>`. A refusal for a key the file omits has no line: `refused: <config>: …`. Every refusal is listed in one run.
3. The listed comments, when there are any: `comments with no single key to follow (--write refuses; --drop-comments drops exactly these):`, then one line each, `  <config>:<first>-<last>  <where>`, or `  <config>:<line>  <where>` for one line. `<where>` is `above <v1 key>`, `inside <v1 key>` or `standing alone`.
4. One line per notice, in file order: `dropped: <config>:<line>: <v1 key>: <why>`, or `changed: <config>:<line>: <v1 key>: <why>`. A carried comment on a translated value also gets the explanatory warning of [§FS-config-migrate.4.1.1](FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes), identifying the original config location/key and its destination and asking the reader to check the comment.
5. Only when nothing is refused: the key map, one `mapped: <config>:<line>: <v1 key> → <v2 spelling>` per authored key in file order, where a key version 2 needs no spelling for reads `→ nothing, version 2 always means it` and an `include` entry that is already a place reads `→ nothing, <entry> is a place of [schema.kinds.<ROW>] already`, then one `written: <v2 key> = <value>: version 1's default, written out` per default written out ([§FS-config-migrate.3.3](FS-config-migrate.md#33-defaults-are-written-out)), then member commands ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces)), then the gates' result ([§FS-config-migrate.5.4](FS-config-migrate.md#54-the-result-and-a-difference)) and any `note:` line.
6. After a write, the lines of [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write). A `--write` that refuses ends with `not written: <config> is unchanged`.

A line is never wrapped. Paths inside either report follow `--path-base`. The output policy changes no gates, comment authorization, discovery, atomic config-only writing, or exit codes. The compatibility decision for the new default is [§DF-config-migrate-output](../decisions/functional/DF-config-migrate-output.md#df-config-migrate-output-successful-migration-writes-report-the-outcome-the-candidate-and-mapping-report-are-opt-in).

### 2.3 Exits

| Exit | When |
|---|---|
| `0` | The preview's `--write` would write; or the write was made; or the config is already version 2. |
| `1` | Refused: a v1 setting with no v2 spelling, a difference at either gate, a comment that would be lost without `--drop-comments`, or a config that is a symbolic link. A preview with the same flags exits `1` for the same cause. |
| `2` | The command could not run: a usage error, no config, a config that does not load, an unsupported version, or an I/O failure or concurrent edit during the write. The original is untouched in every case. |

## 3. Refusals, notices and defaults

### 3.1 A refusal names a v1 repair

A setting version 2 cannot spell is never approximated. The refusal names the v1 key and its line and gives a repair: a v1 edit the user makes, checks with `grund check` under version 1, and commits. That commit is the visible authorization, and `grund check` shows its effect before any v2 file exists. The full set is in [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default); the ones most v1 files meet are below, each with its exact lines.

#### 3.1.1 Named sections off

`grund init` writes `named_sections = false`, and an omitted key means the same, so most v1 files carry it. Version 2 always reads named sections ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)).

```text
refused: grund.toml:7: [id] named_sections = false turns named sections off, and version 2 always reads them
  repair: set [id] named_sections = true under version 1; grund check shows its effect
```

When the key is omitted the block reads ``refused: grund.toml: [id] named_sections is not set, so version 1 reads no named sections, and version 2 always reads them`` and ``  repair: add [id] named_sections = true under version 1; grund check shows its effect``.

#### 3.1.2 Bare citations

```text
refused: grund.toml:4: [reference] strict = false recognizes bare citations, and version 2 recognizes only marked ones
  repair: run grund fmt --marker --write, then set [reference] strict = true under version 1
```

The repair is [§FS-fmt.2.2](FS-fmt.md#22-bare-to-marker-with---marker).

#### 3.1.3 An `include` entry that does not exist

Version 2 puts a folder and a file under different keys, so migration reports a missing path instead of guessing its type. `grund init` writes `include = ["requirements.md", "docs", "e2e", "src"]`, so many repositories need this edit once. An entry that is itself a row's home is not refused, because that row already says whether it is a file or a folder.

```text
refused: grund.toml:12: [scan] include names `src`, which does not exist, and version 2 must know whether it is a folder or a file
  repair: create src, or remove it from include; grund check under version 1 shows the effect
```

A file that omits `[scan] include` reads that same default list, and a missing default entry is refused too, never dropped: a `src` created later would be scanned under version 1 and not under version 2. One block names every missing default entry, and its repair authors the list without them:

```text
refused: grund.toml: [scan] include is not set, so version 1 reads its default, which names `e2e` and `src`, which do not exist, and version 2 must know whether each is a folder or a file
  repair: add [scan] include = ["requirements.md", "docs"] under version 1; grund check shows its effect
```

#### 3.1.4 A path-shaped `[scan] exclude` entry

The v1 reader refuses an entry containing `/`. Migration reads the file tolerantly, only to diagnose it ([§DF-scan-exclude-component-names.2.4](../decisions/functional/DF-scan-exclude-component-names.md#24-what-this-leaves-to-the-v2-migration)), and names each entry with the files on this tree the glob it resembles would stop reading. Removing the entry keeps what is scanned exactly as it was; a project that wants the glob adopts it in version 2 by hand.

```text
refused: grund.toml:10: [scan] exclude entry `docs/drafts` contains `/`, so version 1 never matched it; as a version 2 glob it would stop reading 1 file
  would stop reading: docs/drafts/plan.md
  repair: remove `docs/drafts` from exclude under version 1, which keeps what is scanned exactly as it is
```

#### 3.1.5 Nested homes

A v1 file inside two nested homes belongs to the complement; in version 2 the deepest place owns it ([§DF-v2-place-ownership](../decisions/functional/DF-v2-place-ownership.md#df-v2-place-ownership-in-v2-the-row-named-code-is-the-complement-and-a-file-belongs-to-its-listed-row-or-else-its-deepest-folder)). Migration refuses and names every file whose owner would change, even when no finding on this tree would differ.

```text
refused: grund.toml:15: [[kinds]] `AR` home `docs/arch` is inside `FS` home `docs`; version 1 gives the files in both to the complement, version 2 to the deepest place
  would change owner: docs/arch/store.md
  repair: un-nest the two homes under version 1
```

#### 3.1.6 A renamed complement

```text
refused: grund.toml:9: [[kinds]] `source` is a non-citable row with no home, which renames the complement; version 2 calls the complement `code`
  repair: rename the row, and its [citations.source] table, to `code` under version 1
```

#### 3.1.7 Settings this grund has no v2 spelling for yet

A `value_chapter`, a lead budget counted in lines or bytes, and a non-default `[fmt.cross_refs]` each refuse with the line of the key:

```text
refused: grund.toml:11: [[kinds]] `FS` value_chapter would become a field, which adds a "Declaration shapes" section grund check compares
  repair: none keeps the generated guidance; remove value_chapter under version 1 only if that section may change
refused: grund.toml:5: [reference] lead_size_warning counts lines, and version 2 counts a lead in words
  repair: restate it with unit = "words" under version 1
refused: grund.toml:8: [fmt.cross_refs] enabled = false has no version 2 spelling in this grund
  repair: none yet; keep version 1, or set enabled = true under version 1 if fmt may write links
```

The same `lead_size_warning` line reads `counts bytes` for a byte budget, and `anchor_format` other than `"github"` reads `` [fmt.cross_refs] anchor_format = "<value>" has no version 2 spelling in this grund ``.

### 3.2 Notices

Some settings are written with a notice rather than refused, because version 1 gave them no force ([§DISC-core-concerns.7.5](../discussions/proposals/2026-09-30-core-concerns.md#75-what-has-no-v2-spelling-and-how-migrate-reports-it)):

- A positive `[citations] default` creates no obligation in v1, and is written as `may` ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)): `` changed: grund.toml:20: [citations.FS] default = "must": version 1 gives a positive default no force, so it is written as may ``.
- `[output] format` and `relative_paths` are dropped. At their defaults: `` dropped: grund.toml:22: [output] format = "text": the default of --format, nothing to add `` and `` dropped: grund.toml:24: [output] relative_paths = true: the default of --path-base, nothing to add ``. Otherwise the notice names the flag that keeps the behaviour: `` dropped: grund.toml:22: [output] format = "json": pass --format json to keep it `` and `` dropped: grund.toml:24: [output] relative_paths = false: pass --path-base=invocation to keep it ``.
- `color` is dropped: `` dropped: grund.toml:23: [output] color: nothing reads it ``.

### 3.3 Defaults are written out

Every v1 default that differs from version 2's is written out explicitly, so the file keeps v1's meaning after later edits ([§FS-config-v2.schema.sources.migration](FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes)): the slug pattern, the ID format, the default exclusions as globs, `declarations = "any-comment"`, the explicit language list with its definitions, a suggested note length as the `may` rung of `[schema.notes.lines]`, and `[presentation] rules = "root"` ([§DF-scoped-guidance](../decisions/functional/DF-scoped-guidance.md#df-scoped-guidance-a-version-2-rows-guidance-is-written-above-its-places-with-a-root-pointer-for-agents-that-do-not-load-nested-files)). A file with no `[[kinds]]` gets every built-in row written out. Each one is a `written:` line of the complete report, in the candidate's order; compact success omits that narration, never the defaults in the candidate.

## 4. Comments

The owner's ruling on agent-grounds/grund#461: a comment attached to a key that maps one-to-one moves with that key, a leading header block stays at the top of the file, and every other comment is listed with its line.

### 4.1 Attached comments move with a one-to-one key

A key maps one-to-one when exactly one v2 key or header comes from it: `[[kinds]]` becomes `[schema.kinds.<NAME>]`, `[citations.<KIND>]` becomes `[rules.citations.<KIND>]`, `[id] format` becomes `id_format`, a row's `title` becomes `[presentation.kinds.<NAME>] title`, and `exclude` keeps its name, as does every key the mapping table moves without splitting. Its attached comment is written directly above the v2 key or header it became, and a trailing comment stays on that line. `include`, `extensions`, `comment_prefixes` and the `[scan]`, `[reference]` and `[id]` headers split or merge, so their comments are listed.

#### 4.1.1 Check carried comments when the value vocabulary changes

When an attached comment is carried onto a setting whose value vocabulary is
translated, the stderr report warns that the value vocabulary changed and asks
the reader to check the carried comment. It names the original config path,
the v1 key's line and key, and the destination v2 key. This applies to comments
directly above a key and to trailing comments, in preview and with `--write`,
with or without `--drop-comments`. It does not judge the comment's prose: even
a comment that does not list values needs the same warning.

Vocabulary translation means changing the setting's accepted terms, type or
value language, rather than merely renaming its key or changing TOML layout.
The carryable cases are:

- `[reference] inline_style`: style names become `[schema.notes.text] must`'s boolean.
- `[id] section_heading_levels`: depth modes become `[schema] heading_depth`'s strengths.
- `[[kinds]] resolve`: `should` becomes `[rules.resolution] <KIND>`'s `warn`.
- `[[kinds]] rules` or `values`: a boolean becomes `[schema.kinds.<KIND>] form`'s enum,
  when the destination and a carried comment exist.
- `[fmt.cross_refs] enabled`: a boolean becomes `[presentation.fmt] links`'s enum.
- `[scan] exclude`: directory names become `[schema.sources] exclude`'s escaped globs.
- `[reference] lead_size_warning`: the inline table with a `max` and `unit`
  becomes `[schema.leads.words] warn`'s integer word count.

Classify the mapping's vocabulary, not raw byte inequality: a depth mode of
`warn` still warrants the warning, since other modes in its old vocabulary are
translated. Renames such as `anchor_format` to `anchors`, scalar paths wrapped
in `files` or `folders` lists, and the config version marker do not warrant it.
Citation defaults normalized to `may` keep their strength vocabulary and their
existing changed notice ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices)).

Warn only when a comment is actually carried to the destination. A setting
without a carried comment needs no such warning. The warning changes neither
candidate bytes nor successful exits, gates, refusal or drop policy: the
comment remains carried, including with `--drop-comments`, and is not added
to the comments with no single key to follow ([§FS-config-migrate.4.4](FS-config-migrate.md#44-listed-comments-and---drop-comments)).

### 4.2 The header block stays at the top

The header block is written as the first lines of the candidate, above `grund_config_version`, followed by one blank line.

### 4.3 Everything else is listed

A comment inside a multi-line value, a comment standing on its own, and a comment attached to a key that is dropped, split or merged are listed ([§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report)). The lines come from the v1 reader's own line pass, which records each key's line and its attached comments; migration does not parse the file a second time.

Migration explicitly requests these owned raw-line, key, value, header and
comment records during that same read. An ordinary version-1 config load does
not construct or retain them. This holds for every caller of the ordinary
reader, including discovery, workspace members, `config show`, `config validate`
and pending config text. It serves [§GOAL-fast-feedback.1](../goals.md#1-performance-targets): commands that do not
migrate must not pay for migration's record of the file.

Choosing whether to capture does not change the lowered project, accepted
spellings, first error, error location or rendered output. Migration alone
keeps its tolerant reading of path-shaped exclusions ([§FS-config-migrate.3.1.4](FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry));
ordinary loads still refuse them ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)).

### 4.4 Listed comments, and `--drop-comments`

`--write` refuses while any listed comment would be lost: exit `1`, and the file is not written. `--drop-comments` lets it go ahead, drops exactly the listed comments, and names them again after the write ([§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write)). It is the only way migration loses a comment ([§REQ-no-data-loss.4](../requirements/REQ-no-data-loss.md#4-one-migration-may-replace-the-config)).

## 5. Equivalence: two gates

The mapping table "is what a migration audits against, and tests on the records hold it, not a comparison of one tree's reports" ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)). So the records come first and the report on this tree second. Neither gate is skipped and neither is relaxed by a flag.

### 5.1 The records gate

The candidate is loaded through the v2 reader, the path a file on disk goes through, and its project records are compared with the v1 file's by one comparison that ignores only:

- the version, and where the config was found;
- the dropped `[output]` preferences ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices));
- the source model, compared suffix by suffix: each suffix's reader class, comment prefixes, docstring reading, note-measurement rule and declaration eligibility, rather than the language names that carry them;
- grounding, compared as the effective pair each row resolves to, rather than as v1's inherited keys or v2's whole ladders;
- `include`, compared as the set of derived scan roots;
- the nesting rule, when no two v1 homes nest, because the rule then decides nothing;
- a positive citation default, compared as `may`.

### 5.2 The report gate

`grund check --suggestions` runs twice in the same process, on the same root, members and invocation: once with the file as read, once with the candidate. It compares every finding's code, channel, path, line, column and message, the run warnings, the projects checked, and the exit status. It ignores only:

- a finding located in the config itself, whose line is mapped through the key map before comparing; a finding with no mapped line is a difference;
- the config-table names inside messages, such as `[scan] exclude` against `[schema.sources] exclude`: both runs are spelled as v1;
- the `config-version-1` caution ([§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)), which the run on the file as read carries and the candidate's run never does, so it is dropped from both before comparing.

### 5.3 Guidance

Migration also renders in memory what `grund init` would write into each `AGENTS.md`, for the file as read and for the candidate, whether or not an `AGENTS.md` exists yet. A difference `grund check` compares, the block's version or one of its config-derived sections, is a refusal, because the next `grund init` would write it and `check` would then hold the tree to it. A value row is the common case: version 2 gives it a declaration shape, which adds a section version 1 never wrote:

```text
refused: grund.toml:11: [[kinds]] `TERM` values = true adds a "Declaration shapes" section to the AGENTS.md block in version 2, which grund check compares
  would write: - **TERM**
  would write:   - field <number> may · values
  repair: none keeps the generated guidance; remove values under version 1 only if that section may change
```

Any other compared difference is one block per section, its lines prefixed `- ` for version 1 and `+ ` for the candidate, or a version alone when no section differs:

```text
refused: grund.toml: the AGENTS.md block grund init writes would change its citation directions section, which grund check compares
  + <line>
  repair: none keeps the generated guidance; keep version 1 until that section may change
refused: grund.toml: the AGENTS.md block grund init writes would go from v19 to v21, which grund check compares
  repair: none keeps the generated guidance; keep version 1 until that block may change
```

The Project map row the complement gains once its places are listed is a `note:` line, because only `init` writes that table and `check` does not compare it ([§FS-init.2.3.7.2](FS-init.md#2372-what-check-compares)): `` note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check does not compare that table ``.

### 5.4 The result, and a difference

When both gates pass the complete report says `compared: the records are equal, and grund check --suggestions on this tree gives the same findings and the same exit before and after`. Compact success omits that narration, but runs the same gates. A difference always prints `differs: the records` or `differs: grund check --suggestions on this tree`, then each differing line prefixed `- ` for version 1 and `+ ` for the candidate, and refuses with exit `1` and empty stdout, with or without `--full`.

## 6. The write

### 6.1 Only with `--write`, only past both gates

It writes only with `--write`, and only after both gates pass in that same run and no listed comment stands without `--drop-comments`.

### 6.2 Atomic

It writes a temporary file in the config's own directory, keeps the original's permissions, syncs it to disk, and renames it over the original. Any failure leaves the original byte for byte, and no temporary file behind. A failure is exit `2`: `` error: <config>: could not write the migrated config (<reason>); the original is unchanged ``.

### 6.3 A concurrent edit

Just before the rename it reads the config's bytes again. If they changed since the read that built the candidate it removes the temporary file and exits `2`: `` error: <config> changed while it was being migrated; the original is unchanged, run the migration again ``.

### 6.4 A symbolic link, and nothing else written

A config that is a symbolic link is refused, exit `1` and empty stdout, because renaming over it would replace the link with a regular file:

```text
refused: grund.toml is a symbolic link, and replacing it would replace the link with a regular file
  repair: replace the link with the file it points to, or migrate that file where it is the config
```

Nothing but the config is ever written: no `AGENTS.md`, no lock, no backup.

### 6.5 What it says after the write

After an actual successful write without `--full`, stderr contains only these items, in order, omitting inapplicable items:

1. The `wrote` outcome below.
2. The exact dropped-comment ranges below, once, in original file order.
3. Every notice of [§FS-config-migrate.3.2](FS-config-migrate.md#32-notices) and every carried-comment warning promised by [§FS-config-migrate.4.1.1](FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes), in file order. That warning still appears in compact output.
4. Workspace member commands ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces)).
5. The distinct Project map note ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)), when applicable.
6. The undo and re-preview advice below.
7. The qualified managed-instructions refresh reminder below, exactly once and last.

Compact success omits the heading, key mappings, defaults, passing-gate narration and pre-write comment listing. With `--full`, the complete report of [§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report) precedes the same outcome, dropped ranges, undo/re-preview advice and final reminder; notices, warnings, member commands and the Project map note stay in their complete-report positions and are not repeated.

```text
wrote grund.toml: version 1 (24 lines) replaced by version 2
dropped the comments at grund.toml:6-7 and grund.toml:12 (version 1 lines)
to get version 1 back: it is in version control, e.g. git restore grund.toml
to see this candidate again: restore version 1 and run grund config migrate --to 2
note: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them
```

The `dropped the comments` line appears only when `--drop-comments` dropped any, and names every range in file order, joined by `, ` and a final ` and `.

The final `note:` line appears on stderr after every successful version-1-to-version-2 write, including when no `AGENTS.md` exists. It is a qualified reminder about existing managed instructions, not a claim that a file was inspected or found stale. Run `grund init` in the migrated project to refresh those instructions; migration itself still writes only the config ([§FS-config-migrate.6.4](FS-config-migrate.md#64-a-symbolic-link-and-nothing-else-written)). Migration neither inspects managed blocks for drift nor runs init. The reminder is absent from previews, refused or failed writes, and already-version-2 runs. It does not change exits or replace the distinct Project map note ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)).

### 6.6 A second run is a no-op

A config that is already version 2 is reported as `` <config> is already version 2; nothing to migrate ``, exit `0`, empty stdout, with or without `--write`. The file, its modification time included, is left alone.

## 7. Outcomes

| Situation | Outcome |
|---|---|
| No config found | Exit `2`: `` error: no grund.toml at <path> or above it; grund init writes one ``. Creating a config is not migrating one. |
| Unreadable, malformed, or failing v1 validation | Exit `2`, with the error `config validate` gives ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). |
| A path-shaped `[scan] exclude` | Read tolerantly for diagnosis only; exit `1` with each entry's reach ([§FS-config-migrate.3.1.4](FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry)). |
| `grund_config_version` above 2 | Exit `2`, with today's unsupported-version error ([§FS-config.5](FS-config.md#5-schema-versioning)). |
| Already version 2 | Exit `0`, nothing to migrate ([§FS-config-migrate.6.6](FS-config-migrate.md#66-a-second-run-is-a-no-op)). |
| A symbolic link | Exit `1` ([§FS-config-migrate.6.4](FS-config-migrate.md#64-a-symbolic-link-and-nothing-else-written)). |

## 8. Workspaces

Only the file at `PATH` is migrated; mixed versions are legal ([§FS-config.5](FS-config.md#5-schema-versioning)). At a workspace root the report names each member still on version 1 and the command that migrates it, one line each: `` member: <member>/grund.toml is version 1; migrate it with grund config migrate --to 2 <member> ``. The complete report places these after the `written:` lines and before the gates' result; compact success retains them at the position defined in [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write), without requiring default narration before them. The report gate runs over the whole workspace as `check` would, each member read by its own reader. A member's config is migrated from the member's root, and the report gate runs from there. No member is written; there is no `--recursive`, because several files cannot be replaced atomically together.

## mapping: Every version-1 key and default

**Outcome:** *mapped* is written in v2; *written* is a v1 default written out; *notice* is dropped or changed and reported ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices)); *refused* names its repair ([§FS-config-migrate.3.1](FS-config-migrate.md#31-a-refusal-names-a-v1-repair)). Each row has a mapping case, and a test fails when a row of the v1 lowering table has none. The ground of a row is [§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart) unless it names another point of FS-config-v2.

| v1 key | v2 spelling | Outcome |
|---|---|---|
| `grund_config_version` | `grund_config_version = 2` | mapped |
| `project_name` | `project_name` | mapped |
| `project_description` | `[presentation] description` | mapped |
| `kinds = []`, the empty kind set | no `[schema.kinds]` row | mapped |
| `[reference] conversation` | `[presentation] conversation` | mapped |
| `[reference] grounding_level` | a rung of `[rules.citations.grounding]` | mapped; refused when only `--require-grounding` would switch it on, repair: set `require_grounding` or remove the level |
| `[reference] inline_note_layout` | `[schema.notes.layout] <strength> = …` | mapped |
| `[reference] inline_note_layout_check` | that key's strength: error→`must`, warn→`warn`, suggest→`should`, off→`may` | mapped |
| `[reference] inline_note_max_columns` | `[schema.notes.columns] must` | mapped |
| `[reference] inline_note_max_lines` | `[schema.notes.lines] must` | mapped |
| `[reference] inline_note_suggested_lines` | `[schema.notes.lines] warn` with `warn_on_suggested`, else `may` | mapped |
| `[reference] inline_style` | `[schema.notes.text] must = true` or `false` | mapped |
| `[reference] lead_size_warning` | `[schema.leads.words] warn` | mapped in words; refused in lines or bytes ([§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet)) |
| `[reference] marker` | `[schema] marker` | mapped |
| `[reference] require_grounding` | the `must` rung of the project ladder | mapped |
| `[reference] shorthand` | `[schema] shorthand` | mapped |
| `[reference] strict` | nothing; v2 is always marked | `true` mapped; `false` refused ([§FS-config-migrate.3.1.2](FS-config-migrate.md#312-bare-citations)) |
| `[reference] trigger` | `[presentation] trigger` | mapped |
| `[reference] warn_on_suggested` | chooses `warn` or `may`, as above | mapped |
| `[id] format` | `[schema] id_format` | mapped, and written even at v1's default |
| `[id] named_sections` | nothing; v2 always reads them | `true` mapped; `false` refused ([§FS-config-migrate.3.1.1](FS-config-migrate.md#311-named-sections-off)) |
| `[id] number_pattern` | `[schema] number_pattern` | mapped |
| `[id] section_heading_levels` | `[schema] heading_depth`: strict→`must`, warn→`warn`, suggest→`should`, loose→`may` | mapped |
| `[id] section_separator` | `[schema] section_separator` | mapped |
| `[id] slug_pattern` | `[schema] slug_pattern` | mapped, and written even at v1's default |
| `[[kinds]] citable` | `citable = false` | mapped |
| `[[kinds]] fetch` | `fetch` | mapped |
| `[[kinds]] file` | `files = […]` | mapped |
| `[[kinds]] folder` | `folders = […]` | mapped |
| `[[kinds]] format` | the row's `id_format` | mapped |
| `[[kinds]] grounding_level` | `[rules.citations.<ROW>.grounding]`, the whole effective ladder | mapped |
| `[[kinds]] index` | `index` | mapped |
| `[[kinds]] kind` | `[schema.kinds.<NAME>]` | mapped; a non-citable homeless row that renames the complement is refused ([§FS-config-migrate.3.1.6](FS-config-migrate.md#316-a-renamed-complement)) |
| `[[kinds]] require_grounding` | the row ladder's `must` rung | mapped |
| `[[kinds]] resolve` | `[rules.resolution] <KIND>`: should→`warn` | mapped |
| `[[kinds]] rules` | `form = "rule"` | mapped |
| `[[kinds]] scan` | `scan = false` | mapped |
| `[[kinds]] title` | `[presentation.kinds.<NAME>] title` | mapped |
| `[[kinds]] value_chapter` | a field, which would add a section `check` compares | refused ([§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet)) |
| `[[kinds]] values` | `form = "value"` | refused by the guidance comparison ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)): version 2 gives every value row a declaration shape |
| `[scan] comment_prefixes` | each definition's `comment_prefixes`, the whole list | mapped |
| `[scan] docstring_python` | `.py`'s definition, `docstrings` python or off | mapped |
| `[scan] exclude` | `[schema.sources] exclude`, each name as the escaped glob `<name>/` | mapped; a path-shaped entry refused ([§FS-config-migrate.3.1.4](FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry)) |
| `[scan] extensions` | `languages`, `[schema.sources.definitions.v1_<profile>]` and `[schema.sources.extensions]` | mapped; JSON mixed with value authority, unreachable profiles and out-of-domain suffixes refused as [§FS-config-v2.schema.sources.migration](FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes) says |
| `[scan] include` | `[schema.kinds.code] files` and `folders`, by what each entry is on disk | mapped; an entry at or inside a scanned home dropped as a duplicate; refused when missing, the default list's entries included ([§FS-config-migrate.3.1.3](FS-config-migrate.md#313-an-include-entry-that-does-not-exist)), outside the root, inside a member, or inside an unscanned home |
| `[scan] respect_gitignore` | `[schema.sources] respect_gitignore` | mapped |
| `[output] color` | none | notice: nothing reads it |
| `[output] format` | none | notice; `"json"` names `--format json` |
| `[output] relative_paths` | none | notice; `false` names `--path-base=invocation` |
| `[fmt.cross_refs] anchor_format` | `[presentation.fmt] anchors = "github"` | `"github"` mapped; others refused |
| `[fmt.cross_refs] enabled` | `[presentation.fmt] links = "index"` | `true` mapped; `false` refused |
| `[fmt] exclude` | `[presentation.fmt] exclude` | mapped |
| `[workspace] include_root` | `[workspace] include_root` | mapped |
| `[workspace] members` | `[workspace] members` | mapped |
| `[workspace] optional_members` | `[workspace] optional_members` | mapped |
| `[citations] default` | `[rules.citations] default` | mapped; a positive default is a notice and written `may` |
| `[citations.<KIND>] default` | `[rules.citations.<KIND>] default` | as above |
| `[citations.<KIND>] may`, `must`, `must-not`, `should`, `should-not` | the same key of `[rules.citations.<KIND>]` | mapped |
| *built-in kinds* | every built-in row, written out | written |
| *name-keyed index* | each row's `index`, the E2E `index = false` included | written |
| *fetch resolves must* | nothing; v2's default for a fetched kind is `must` | mapped |
| *kind grounding inherits* | the effective pair per row, as whole ladders | written |
| *complement name* | `code`, with a row only when it has places | mapped |
| *nesting* | nothing when no two v1 homes nest | mapped; refused otherwise ([§FS-config-migrate.3.1.5](FS-config-migrate.md#315-nested-homes)) |
| *strict* | nothing | mapped |
| *named sections, off* | nothing | refused, as `[id] named_sections` |
| *scan exclusions* | the six names as `<name>/` globs | written |
| *version* | `grund_config_version = 2` | written |
| *any-comment declarations* | `[schema.sources] declarations = "any-comment"` | written |
| *guidance at the root* | `[presentation] rules = "root"` | written |

# FS-config-migrate: grund config migrate --to 2 shows the version-2 file that means what a version-1 config means, and writes it only when asked and only when it does

`grund config migrate --to 2` reads a version-1 `grund.toml` and shows the version-2 file that means the same thing. A v1 setting version 2 cannot spell is refused with the v1 edit that gets to one, so every change of meaning is one the user made. It writes only with `--write`, only past two comparisons that show the new file means what the old one did, and only by an atomic replace: the one exception to "the config is never overwritten" ([§REQ-no-data-loss.4](../requirements/REQ-no-data-loss.md#4-one-migration-may-replace-the-config)). The translation is held to [§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart) and [§FS-config-v2.schema.sources.migration](FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes). Serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path); decided in [§DF-config-migrate](../decisions/functional/DF-config-migrate.md#df-config-migrate-migration-refuses-what-version-2-cannot-spell-names-a-version-1-repair-and-writes-only-past-two-gates).

## terms: Terms

Leans on [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (config root, workspace, member), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding) and [§FS-config-v2.terms](FS-config-v2.md#terms-terms) (reader, place, complement). Defines:

- **Candidate**: the version-2 file migration would write for the config as it stands, with the same flags; it exists only when nothing is refused.
- **Preview**: a run without `--write`; it writes nothing.
- **Refusal**: a v1 setting version 2 cannot spell with the same meaning.
- **Repair**: the v1 edit a refusal names, which the user makes, checks under version 1 and commits before migrating.
- **Notice**: a v1 setting dropped, or written differently, because version 1 gave it no force; it is reported and refuses nothing.
- **Key map**: each authored v1 key with its line and the v2 key it became.
- **Attached comment**: a run of comment lines directly above a key or table header, with no blank line between, or a trailing comment on that line.
- **Header block**: the comment lines from line 1 to the first blank or non-comment line.
- **Listed comment**: a comment migration cannot carry, named in the report with its lines.
- **Gate**: one of the two comparisons a write must pass ([§FS-config-migrate.5](FS-config-migrate.md#5-equivalence-two-gates)).

## 1. The command

```text
grund config migrate --to 2 [PATH] [--write] [--full] [--drop-comments] [--path-base project|invocation]
```

### 1.1 Which file

`PATH` works as it does for `config validate` and `config show` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path)): discovery walks up from it, `.` when omitted, to the config that governs there, the root `grund.toml` or the deprecated `.agents/grund.toml` ([§FS-config.1](FS-config.md#1-file-location-and-discovery), [§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)), and migrates that one file where it is. Inside a workspace member that is the member's config ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces)).

### 1.2 `--to` is required

`--to 2` is the only accepted target ([§DF-config-migrate.2.3](../decisions/functional/DF-config-migrate.md#23---to-is-required)). A run without `--to` prints the first line below, and one with any other value, `1` included, the second, naming the value given. Both are usage errors, exit `2`, answered before anything is loaded ([§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read--realized-directional-for-the-next-flag)):

```text
error: config migrate needs --to 2 (the version to migrate to)
error: config migrate converts to version 2 only, not `1`
```

### 1.3 The other flags

- `--write` replaces the config with the candidate ([§FS-config-migrate.6](FS-config-migrate.md#6-the-write)).
- `--full`, which no other `config` command accepts, prints the candidate on stdout and the complete report on stderr in the two runs that otherwise leave them out: a successful write ([§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write)) and a preview that only listed comments refuse, which still closes with its refusal line ([§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report)). Elsewhere it changes nothing. It combines with `PATH` and every other migrate flag, and neither relaxes `--to 2` nor admits `--format`.
- `--drop-comments` lets the write drop exactly the listed comments ([§FS-config-migrate.4.4](FS-config-migrate.md#44-listed-comments-and---drop-comments)). A preview accepts it, to predict the write.
- `--path-base` spells the report's paths ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)).
- `--format` is refused as for every `config` command ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)): `` error: unknown flag `--format` ``, exit `2`.

## 2. Streams and exits

### 2.1 Stdout is the candidate

Stdout carries the candidate and nothing else ([§FS-errors.1](FS-errors.md#1-streams)), in three runs: a preview whose `--write` would write prints exactly the bytes `--write` would write with the same flags, `--drop-comments` included, with or without `--full`; `--write --full` prints the candidate it wrote; and a `--full` preview that only listed comments refuse prints the candidate without those comments, as `--drop-comments` would write it, and still exits `1`. Every other run leaves stdout empty, so `grund config migrate --to 2 > new.toml` captures only a candidate, and one only when it exits `0`. The output choice never changes the candidate's bytes. Its tables keyed by a row name print in version 2's order ([§FS-config.4.2.4](FS-config.md#424-the-order-show-prints-in)), which is the order of the version-1 `[[kinds]]` entries.

### 2.2 Stderr is the report

Every other line goes to stderr, unwrapped. A successful write without `--full` prints the compact report of [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write), and a failure or a config already on version 2 prints its error or no-op line alone ([§FS-config-migrate.6.6](FS-config-migrate.md#66-a-second-run-is-a-no-op), [§FS-config-migrate.7](FS-config-migrate.md#7-outcomes)). Every other run prints the complete report, in this order. `<config>` is the config's path from the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), `grund.toml` by default; every path follows `--path-base` except the four commands to paste ([§FS-config-migrate.6.5.1](FS-config-migrate.md#651-the-commands-a-migrate-report-prints-are-spelled-from-the-working-directory)).

1. The heading: `<config>: version 1 → 2, preview, nothing written`, or `<config>: version 1 → 2, --write`.
2. One block per refusal, every refusal in one run, in file order: `refused: <config>:<line>: <what version 2 cannot spell>`, any indented detail lines, then `  repair: <the v1 edit>`. A refusal for a key the file omits has no line: `refused: <config>: …`.
3. The listed comments: `comments with no single key to follow (--write refuses; --drop-comments drops exactly these):`, then one line each, `  <config>:<first>-<last>  <where>: <guidance>`, or `  <config>:<line>  <where>: <guidance>` for one line, where `<where>` is `above <v1 key>`, `inside <v1 key>` or `standing alone`. The guidance says why the comment cannot follow one key and where its prose can go. For a split or merged key or header it names the v2 destinations the entry's authored settings map to, from the key map's facts, and asks the reader to keep the prose, authorize the drops with `--write --drop-comments`, and re-add it at one of them after a successful write, subject to every other refusal and gate. Where no single destination can carry it, for a dropped setting, a standalone comment, an omitted destination key or a trailing-comment collision, whose shared destination it names, it asks that the prose be placed by hand. Where refusals leave no candidate, it names the destinations as unavailable until the repairs and a new preview, never promising that `--drop-comments` clears a refusal.
4. The notices, in file order: `dropped: <config>:<line>: <v1 key>: <why>` or `changed: <config>:<line>: <v1 key>: <why>` ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices)), and the warning of [§FS-config-migrate.4.1.1](FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes) for a comment carried onto a translated value.
5. When no setting or file is refused: the key map, one `mapped: <config>:<line>: <v1 key> → <v2 spelling>` per authored key in file order, `→ nothing, version 2 always means it` for a key version 2 needs no spelling for and `→ nothing, <entry> is a place of [schema.kinds.<ROW>] already` for an `include` entry that is already a place; one `written: <v2 key> = <value>: version 1's default, written out` per default ([§FS-config-migrate.3.3](FS-config-migrate.md#33-defaults-are-written-out)); the member lines ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces)); the gates' result ([§FS-config-migrate.5.4](FS-config-migrate.md#54-the-result-and-a-difference)); and any `note:` line. A refused run leaves out the `written:` lines and the Project map note unless its candidate is on stdout ([§FS-config-migrate.2.1](FS-config-migrate.md#21-stdout-is-the-candidate)), and keeps the rest, every comparison it made with its differing lines included.
6. After a write, the lines of [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write). A refused `--write` ends with `not written: <causes>; <config> is unchanged`, and a refused preview, with or without `--full`, with `would not write: <causes>`. Both read one list, so the same tree and flags name the same causes in the same words: every blocking cause, in this order, joined by `; `:
   - `1 refusal (follow the repairs above)` or `<n> refusals (follow the repairs above)`, counting refusal blocks;
   - without `--drop-comments`, `1 comment has no single key to follow (follow the comment guidance above)` or `<n> comments have no single key to follow (follow the comment guidance above)`, counting listed ranges, not lines;
   - `the records differ (inspect the comparison above)` or `grund check --suggestions differs (inspect the comparison above)`, for the gate that differed. A gate not run because of an earlier refusal adds none.

Decided in [§DF-config-migrate-output](../decisions/functional/DF-config-migrate-output.md#df-config-migrate-output-successful-migration-writes-report-the-outcome-the-candidate-and-mapping-report-are-opt-in) and [§DF-config-migrate-refused-preview](../decisions/functional/DF-config-migrate-refused-preview.md#df-config-migrate-refused-preview-a-refused-migration-preview-closes-with-its-causes-and-keeps-its-candidate-behind---full).

Item 3 lists an `include` comment whose entries go to places of two rows as:

```text
comments with no single key to follow (--write refuses; --drop-comments drops exactly these):
  grund.toml:14  above [scan] include: setting splits or merges into version-2 settings: [schema.kinds.code] folders, src; docs is a place of [schema.kinds.FS] already; retain the prose, authorize the listed drops with --write --drop-comments, then re-add it at one of these named destinations after a successful write (subject to all other refusals and gates)
```

### 2.3 Exits

| Exit | When |
|---|---|
| `0` | The preview's `--write` would write; or the write was made; or the config is already version 2. |
| `1` | Refused: a v1 setting with no v2 spelling, a difference at either gate, a comment that would be lost without `--drop-comments`, or a config that is a symbolic link. A preview with the same flags exits `1` for the same cause. |
| `2` | The command could not run: a usage error, no config, a config that does not load, an unsupported version, a version 1 scan that cannot walk the tree where a refusal asks what it reads ([§FS-config-migrate.3.1.4](FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry)), or an I/O failure or concurrent edit during the write. The original is untouched in every case. |

## 3. Refusals, notices and defaults

### 3.1 A refusal names a v1 repair

A setting version 2 cannot spell is never approximated: its refusal names the v1 key, its line and a repair, which the user makes, checks with `grund check` and commits under version 1 as the visible authorization ([§DF-config-migrate.2.1](../decisions/functional/DF-config-migrate.md#21-a-refusal-names-a-v1-repair-not-a-flag)). [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default) lists every refusal; the sections below give the lines most v1 files meet.

#### 3.1.1 Named sections off

Version 2 always reads named sections ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)), so `[id] named_sections = false` is refused, and so is the key omitted, except on a block with `include_root = false` ([§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false)):

```text
refused: grund.toml:6: [id] named_sections = false turns named sections off, and version 2 always reads them
  repair: set [id] named_sections = true under version 1; grund check shows its effect
refused: grund.toml: [id] named_sections is not set, so version 1 reads no named sections, and version 2 always reads them
  repair: add [id] named_sections = true under version 1; grund check shows its effect
```

#### 3.1.2 Bare citations

```text
refused: grund.toml:5: [reference] strict = false recognizes bare citations, and version 2 recognizes only marked ones
  repair: run grund fmt --marker --write, then set [reference] strict = true under version 1
```

The repair is [§FS-fmt.2.2](FS-fmt.md#22-bare-to-marker-with---marker). Since `fmt` narrows to a path it is given ([§FS-fmt.1](FS-fmt.md#1-inputs)), the repair names the refused project's root, not `PATH`, so it reaches every bare citation the project holds; [§FS-config-migrate.6.5.1](FS-config-migrate.md#651-the-commands-a-migrate-report-prints-are-spelled-from-the-working-directory) spells that root.

#### 3.1.3 An `include` entry that does not exist

Version 2 puts a folder and a file under different keys, so a missing path is refused rather than guessed. An entry that is a row's home is not, since the row already says which it is.

```text
refused: grund.toml:14: [scan] include names `src`, which does not exist, and version 2 must know whether it is a folder or a file
  repair: create src, or remove it from include; grund check under version 1 shows the effect
```

A file that omits `[scan] include` reads version 1's default list, and a missing default entry is refused too, never dropped, since a `src` created later would be scanned under version 1 and not under version 2. One block names every missing default entry, and its repair authors the list without them, except on a block with `include_root = false` that migrates compactly ([§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false)):

```text
refused: grund.toml: [scan] include is not set, so version 1 reads its default, which names `requirements.md`, `e2e` and `src`, which do not exist, and version 2 must know whether each is a folder or a file
  repair: add [scan] include = ["docs"] under version 1; grund check shows its effect
```

#### 3.1.4 A path-shaped `[scan] exclude` entry

The v1 reader refuses an entry containing `/` ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)). Migration reads it tolerantly, only to diagnose it ([§DF-scan-exclude-component-names.2.4](../decisions/functional/DF-scan-exclude-component-names.md#24-what-this-leaves-to-the-v2-migration)), and refuses it with the files the glob it resembles would stop reading. Removing it keeps what is scanned as it is; a project that wants the glob adopts it in version 2 by hand.

```text
refused: grund.toml:15: [scan] exclude entry `docs/drafts` contains `/`, so version 1 never matched it; as a version 2 glob it would stop reading 1 file
  would stop reading: docs/drafts/plan.md
  repair: remove `docs/drafts` from exclude under version 1, which keeps what is scanned exactly as it is
```

A file is named and counted when the version 1 scan of this tree reads it ([§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads)) and the entry, read alone as one version 2 `[schema.sources] exclude` glob ([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)), would stop it from the file's scan root: the deepest place version 1's homes and `include` entries become ([§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default)), and otherwise the config root. A scan root is read even where the entry names it ([§FS-config-v2.schema.sources.exclude.roots](FS-config-v2.md#schemasourcesexcluderoots-how-exclude-composes-with-the-other-filters)), so an entry naming a place stops nothing there, while one below it, such as `docs/fs/*` under a home `docs/fs`, stops what lies below. Paths are compared and printed lexically, `.` and `..` resolved and symbolic links not, while the glob keeps its authored spelling. With no file named the refusal stands, counting `0 files` without the `would stop reading:` line, as it does for an entry that stops nothing as a glob: a `#` comment, a negation, which rescues nothing read alone, or a pattern the version 2 reader refuses, such as `docs/[a`, which fails nothing here.

Only a refusal that needs what the scan reads walks the tree. An entry that stops nothing as a glob, or one that can match only the path it spells, anchored by a leading or inner `/`, holding none of `*`, `?`, `[`, `{` or `\`, and spelled without trailing spaces or a `/` at either end, asks the scan only when its spelled path is on disk, and otherwise counts `0 files`; every other entry asks. Where an entry asks and the version 1 scan cannot walk this tree at all, as under a `[workspace]` member glob that names no directory, the run fails with exit `2` and that glob's error, not the path-shaped entry's error an ordinary load of the file gives ([§FS-config-migrate.7](FS-config-migrate.md#7-outcomes)); a run with no such refusal goes on. A path the scan cannot read inside a tree it walks is not named, and its error is not printed.

#### 3.1.5 Nested homes

A v1 file inside two nested homes belongs to the complement, and in version 2 to the deepest place ([§DF-v2-place-ownership](../decisions/functional/DF-v2-place-ownership.md#df-v2-place-ownership-in-v2-the-row-named-code-is-the-complement-and-a-file-belongs-to-its-listed-row-or-else-its-deepest-folder)), so nesting is refused, naming every file whose owner would change, even when no finding on this tree would differ:

```text
refused: grund.toml:16: [[kinds]] `AR` home `docs/arch` is inside `FS` home `docs`; version 1 gives the files in both to the complement, version 2 to the deepest place
  would change owner: docs/arch/store.md
  repair: un-nest the two homes under version 1
```

The files named are those the version 1 scan reads in the inner home ([§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads)), which is a scan root ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)), compared, printed and asked of the scan as [§FS-config-migrate.3.1.4](FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry) says. With none named, as for an inner home not on disk, the refusal stands without the `would change owner:` line.

#### 3.1.6 A renamed complement

```text
refused: grund.toml:18: [[kinds]] `source` is a non-citable row with no home, which renames the complement; version 2 calls the complement `code`
  repair: rename the row, and its [citations.source] table, to `code` under version 1
```

#### 3.1.7 Settings this grund has no v2 spelling for yet

A `value_chapter`, a lead budget counted in lines or bytes, and a non-default `[fmt.cross_refs]` are refused at the key's line:

```text
refused: grund.toml:12: [[kinds]] `FS` value_chapter would become a field, which adds a "Declaration shapes" section grund check compares
  repair: removing value_chapter alone ends the comparison of every binding that quoted the chapter; to keep them compared, under version 1 rewrite its roots as marked sections, remove value_chapter and cite the new paths
refused: grund.toml:5: [reference] lead_size_warning counts lines, and version 2 counts a lead in words
  repair: restate it with unit = "words" under version 1
refused: grund.toml:18: [fmt.cross_refs] enabled = false has no version 2 spelling in this grund
  repair: none yet; keep version 1, or set enabled = true under version 1 if fmt may write links
```

A byte budget reads `counts bytes`, and an `anchor_format` other than `"github"` reads `` [fmt.cross_refs] anchor_format = "<value>" has no version 2 spelling in this grund ``. A `value_chapter` on a row that sets `fetch` is refused in words of its own, since a fetched kind takes no fields ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)):

```text
refused: grund.toml:9: [[kinds]] `AR` value_chapter has no version 2 spelling on a fetched kind, because a fetched kind takes no fields
  repair: none yet; keep version 1, or under version 1 rewrite its roots as marked sections, remove value_chapter and cite the new paths
```

Removed alone, a `value_chapter` or a `values = true` ends the value comparison of every binding that quoted its values, whatever the row's `fetch`, so a wrong literal goes from an error to exit `0`, and the gates, comparing the tree with the key already gone, see no difference. A root rewritten as a marked section ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)) is judged the same under both versions, whatever the row's `fetch`: a wrong literal bound to it fails `grund check` before the migration and after, and once the key is gone nothing here refuses the row. This holds of the tree `grund check` and migration read; what `grund fetch` writes into a fetched row's home is [§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)'s to say.

#### 3.1.8 A string that closes early

Version 1 reads a string from its opening quote to the last quote of its value or element, every quote between kept, for good ([§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)); version 2 ends it at its first unescaped quote ([§FS-config-v2.reader.6](FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote)). Migration refuses such a string at its key's line ([§DF-v2-string-closes-at-its-quote.2.5](../decisions/functional/DF-v2-string-closes-at-its-quote.md#25-version-1-keeps-its-reading-for-good)), one block per key, naming the first string in its value that closes early as version 1 reads it, with a repair under version 1; a list element's repair also offers a `,` between two strings:

```text
refused: grund.toml:8: [[kinds]] folder reads the one string `docs" "x`, and version 2 ends a string at its first unescaped quote
  repair: end the string at its closing quote under version 1: write `\"` for a quote inside it; grund check shows its effect
refused: grund.toml:13: [scan] exclude reads the one string `a"b`, and version 2 ends a string at its first unescaped quote
  repair: end the string at its closing quote under version 1: write `\"` for a quote inside it, or a `,` between two strings; grund check shows its effect
```

#### 3.1.9 An empty string in `[scan] extensions`

Version 1 reads each file with no extension through an empty string in `[scan] extensions`, for good ([§FS-scan.scope.extensions.empty](FS-scan.md#scopeextensionsempty-in-version-1-an-empty-string-in-the-list-reads-a-file-with-no-extension)); version 2 has no such entry ([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)). The refusal says what the entry reads and what removing it costs ([§DF-v1-empty-extension-reading.2.4](../decisions/functional/DF-v1-empty-extension-reading.md#24-migration-says-what-the-entry-reads-and-what-removing-it-costs)):

```text
refused: grund.toml:4: [scan] extensions names ``, which is not a final extension version 2 can map; version 1 reads every file with no extension through it, and version 2 reads none
  repair: remove `` from extensions under version 1, and give any such file you want read an extension the list holds; grund check shows its effect
```

Any other entry no version 2 mapping can hold, such as `tar.gz` or `*.md`, is refused without those clauses:

```text
refused: grund.toml:4: [scan] extensions names `tar.gz`, which is not a final extension version 2 can map
  repair: remove `tar.gz` from extensions under version 1
```

#### 3.1.10 A `fetch` that names no executable

A version 1 `fetch` that names no executable loads for good ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable), [§FS-config.5.2](FS-config.md#52-a-versions-meaning-moves-only-as-backwards-compatibility-allows)), and version 2 refuses it at load ([§FS-config-v2.checks.config-fetch-names-nothing](FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)). Migration refuses it at the key's line with the repair [§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable) prints, made under version 1 ([§DF-fetch-names-an-executable.2.3](../decisions/functional/DF-fetch-names-an-executable.md#23-version-1-keeps-loading-the-key)), judging the string alone, so a whitespace-only value is refused whether or not a file of that name exists:

```text
refused: grund.toml:6: [[kinds]] `AR` fetch names no executable
  repair: under version 1, name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors; grund check shows its effect
```

### 3.2 Notices

Settings version 1 gave no force are written with a notice rather than refused ([§DISC-core-concerns.7.5](../discussions/proposals/2026-09-30-core-concerns.md#75-what-has-no-v2-spelling-and-how-migrate-reports-it)): a positive `[citations] default` is written as `may` ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)); `[output] format` and `relative_paths` are dropped, naming the flag that keeps a value other than the default; `[output] color` is dropped, since nothing reads it; and on a block with `include_root = false` each other key [§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false) lists as without force is dropped, a compact block adding one line with no line number, last, for the defaults it does not write.

```text
changed: grund.toml:20: [citations.FS] default = "must": version 1 gives a positive default no force, so it is written as may
dropped: grund.toml:22: [output] format = "text": the default of --format, nothing to add
dropped: grund.toml:18: [output] format = "json": pass --format json to keep it
dropped: grund.toml:24: [output] relative_paths = true: the default of --path-base, nothing to add
dropped: grund.toml:24: [output] relative_paths = false: pass --path-base=invocation to keep it
dropped: grund.toml:23: [output] color: nothing reads it
dropped: grund.toml:5: [id] named_sections: include_root = false, so no project scans this block's own files
dropped: grund.toml: version 1's defaults: include_root = false, so no project scans this block's own files; it takes version 2's defaults
```

### 3.3 Defaults are written out

Every v1 default that differs from version 2's is written out ([§FS-config-v2.schema.sources.migration](FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes)): the slug pattern, the ID format, the default exclusions as globs, `declarations = "any-comment"`, the explicit language list with its definitions, a suggested note length as the `may` rung of `[schema.notes.lines]`, and `[presentation] rules = "root"` ([§DF-scoped-guidance](../decisions/functional/DF-scoped-guidance.md#df-scoped-guidance-a-version-2-rows-guidance-is-written-above-its-places-with-a-root-pointer-for-agents-that-do-not-load-nested-files)); a file with no `[[kinds]]` also gets every built-in row. Each is a `written:` line of the complete report, in the candidate's order. A compact block writes none ([§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false)).

## 4. Comments

A comment follows a key only where it has one key to follow, and the header block stays at the top; every other comment is named, never moved by guess ([§DF-config-migrate.2.6](../decisions/functional/DF-config-migrate.md#26-no-text-rewrite)).

### 4.1 Attached comments move with a one-to-one key

A key maps one-to-one when exactly one v2 key or header comes from it: `[[kinds]]` to `[schema.kinds.<NAME>]`, `[citations.<KIND>]` to `[rules.citations.<KIND>]`, `[id] format` to `id_format`, a row's `title` to `[presentation.kinds.<NAME>] title`, `exclude` to `exclude`, and every key the mapping table moves without splitting. Its attached comment is written directly above the v2 key or header it became, and a trailing comment stays on that line. `include`, `extensions`, `comment_prefixes` and the `[scan]`, `[reference]` and `[id]` headers split or merge, so their comments are listed.

#### 4.1.1 Check carried comments when the value vocabulary changes

A comment carried onto a setting whose value vocabulary is translated gets a warning to check it, naming the original config path, the v1 key's line and key, and the destination v2 key: a comment above the key or trailing it, in a preview and with `--write`, with or without `--drop-comments`, whatever its prose says. A vocabulary is translated where the accepted terms, type or value language change, not where only the key's name or TOML layout does:

- `[reference] inline_style`: style names become `[schema.notes.text] must`'s boolean.
- `[id] section_heading_levels`: depth modes become `[schema] heading_depth`'s strengths.
- `[[kinds]] resolve`: `should` becomes `[rules.resolution] <KIND>`'s `warn`.
- `[[kinds]] rules` or `values`: a boolean becomes `[schema.kinds.<KIND>] form`'s enum,
  when the destination and a carried comment exist.
- `[fmt.cross_refs] enabled`: a boolean becomes `[presentation.fmt] links`'s enum.
- `[scan] exclude`: directory names become `[schema.sources] exclude`'s escaped globs.
- `[reference] lead_size_warning`: the inline table with a `max` and `unit`
  becomes `[schema.leads.words] warn`'s integer word count.

The mapping's vocabulary decides, not byte inequality: a depth mode of `warn` still warns. Renames such as `anchor_format` to `anchors`, scalar paths wrapped in `files` or `folders` lists, and the config version marker do not warn, and a citation default normalized to `may` keeps its strength vocabulary and its `changed:` notice ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices)). A warned comment stays carried, `--drop-comments` included, and is never listed.

### 4.2 The header block stays at the top

The header block is written as the first lines of the candidate, above `grund_config_version`, followed by one blank line.

### 4.3 Everything else is listed

A comment inside a multi-line value, a comment standing on its own, and a comment attached to a key that is dropped, split or merged are listed, with the guidance of [§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report).

### 4.4 Listed comments, and `--drop-comments`

`--write` refuses while any listed comment would be lost: exit `1`, nothing written. `--drop-comments` lets it go ahead, drops exactly the listed comments, and names them after the write ([§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write)). It is the only way migration loses a comment ([§REQ-no-data-loss.4](../requirements/REQ-no-data-loss.md#4-one-migration-may-replace-the-config)).

## 5. Equivalence: two gates

A write must pass both gates in the same run, the records first and the report on this tree second ([§DF-config-migrate.2.2](../decisions/functional/DF-config-migrate.md#22-two-gates-records-first)). No flag skips or relaxes either.

### 5.1 The records gate

The candidate is loaded through the v2 reader, the path a file on disk goes through, and its project records are compared with the v1 file's, ignoring only the following, or for a block with `include_root = false` what [§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false) says:

- the version, and where the config was found;
- the dropped `[output]` preferences;
- the source model, compared suffix by suffix: each suffix's reader class, comment prefixes, docstring reading, multi-line string and quote reading as a file of that suffix gets it ([§FS-languages.multi-line-string.suffix](FS-languages.md#multi-line-stringsuffix-which-row-and-which-quote-reading-a-file-reads)), note-measurement rule and declaration eligibility, rather than the language names that carry them, so `.cc` read under version 1 and `.cc` mapped to the built-in `cpp` compare equal, while `.h` read under version 1 and `.h` mapped to the built-in `c` do not ([§DF-defined-language-string-forms.2.2](../decisions/functional/DF-defined-language-string-forms.md#22-the-records-gate-compares-the-reading)); a suffix that differs shows on the `sources` line;
- grounding, compared as the effective pair each row resolves to, rather than as v1's inherited keys or v2's whole ladders;
- `include`, compared as the set of derived scan roots;
- the nesting rule, when no two v1 homes nest;
- a positive citation default, compared as `may`.

### 5.2 The report gate

`grund check --suggestions` runs twice in one process, on the same root, members and invocation: with the file as read and with the candidate. It compares each finding's code, channel, path, line, column and message, the run warnings, the projects checked and the exit status, each line as many times as `check` would print it, so a run warning printed once is compared once. It ignores only:

- the config's own lines: a finding located in the config, and every `<config>:<line>` its message names, is mapped through the key map first;
- the config-table names inside messages, such as `[scan] exclude` against `[schema.sources] exclude`: both runs are spelled as v1;
- the `config-version-1` caution ([§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)), dropped from both runs, since only the first carries it;
- an `agents-init` finding about a managed block's text outside the block version and the five generated sections, which [§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance) passes ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison), [§DF-check-compares-the-whole-block.2.4](../decisions/functional/DF-check-compares-the-whole-block.md#24-migrate-refuses-what-it-refused-before));
- an `agents-init` finding that the root block's citation directions section is stale, where [§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance) found that section equal once the grounding sentence's setting is spelled in one grammar.

Every other finding on a generated section is compared, a scoped or nested block's included, which the root-only guidance gate never renders. A candidate line the key map does not hold, such as a default written out ([§FS-config-migrate.3.3](FS-config-migrate.md#33-defaults-are-written-out)) or a built-in row, has no version-1 line: it prints as the candidate's own, the line `grund check` names once the candidate is written, marked `the candidate's <config>:<line>`, or `the candidate's <config>:<line>:<column>` as a finding's location. The mark equals no version-1 line, even one with the same number, so such a finding is a difference. A config finding with no line keeps `-`.

### 5.3 Guidance

Migration renders in memory what `grund init` would write into each `AGENTS.md`, for the file as read and for the candidate, whether or not an `AGENTS.md` exists. A difference in the block's version or in one of its five generated sections, `### Citation directions`, `### Clickable citations`, `### Declaration shapes`, `### Chapter rules` and `### Scoped guidance`, is a refusal, because those carry the rules the tree is held to and a migration must not move them. Every other difference passes ([§DF-check-compares-the-whole-block.2.4](../decisions/functional/DF-check-compares-the-whole-block.md#24-migrate-refuses-what-it-refused-before)), and `check` reports the block stale until `grund init` runs ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)).

In a grounding sentence for a home whose row grounds nothing ([§FS-init.2.3.5.2.2](FS-init.md#23522-a-home-whose-row-grounds-nothing-says-so)), the comparison spells the named setting in the candidate's grammar on both sides, though `grund init` and `grund check` spell it in the live config's version. What the sentence says is still compared: a row that grounds on one side only, or a different row or home, is refused with `repair: none`.

A value row is the common refusal, since version 2 gives it a declaration shape. For a `TERM` row whose `file` is `terms.md`:

```text
refused: grund.toml:16: [[kinds]] `TERM` values = true adds a "Declaration shapes" section to the AGENTS.md block in version 2, which grund check compares
  would write: - **TERM**
  would write:   - `# <ID>: <title>` with only `## 1. <number or word>`, `## 2. …` under it in `terms.md`: part 1 is `§<ID>.1`; a unit is a part of its own
  repair: removing values alone ends the comparison of every binding that quoted its values; to keep them compared, under version 1 rewrite its roots as marked sections, remove values and cite the new paths
```

A fetched value row is refused the same way, with the same repair, whose cost [§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet) states, because `form = "value"` is not a field ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)); its bullet names the file `grund fetch` writes ([§FS-schema.4.1.4](FS-schema.md#414-a-value-kinds-line-names-the-file-its-values-take)).

Any other compared difference is one block per differing section, its lines prefixed `- ` for version 1 and `+ ` for the candidate, or the versions alone when no section differs:

```text
refused: grund.toml: the AGENTS.md block grund init writes would change its citation directions section, which grund check compares
  + <line>
  repair: none keeps the generated guidance; keep version 1 until that section may change
refused: grund.toml: the AGENTS.md block grund init writes would go from v19 to v21, which grund check compares
  repair: none keeps the generated guidance; keep version 1 until that block may change
```

The Project map row the complement gains once its places are listed is not a generated section ([§FS-init.2.3.7.2](FS-init.md#2372-what-check-compares)), so it is a note, not a refusal: `` note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check reports the block stale until then ``.

### 5.4 The result, and a difference

When both gates pass, the complete report says `compared: the records are equal, and grund check --suggestions on this tree gives the same findings and the same exit before and after`; compact success leaves the line out but runs the same gates. A difference prints `differs: the records` or `differs: grund check --suggestions on this tree`, then each differing line prefixed `- ` for version 1 and `+ ` for the candidate, config lines mapped as in [§FS-config-migrate.5.2](FS-config-migrate.md#52-the-report-gate), and refuses with exit `1` and empty stdout, with or without `--full`.

## 6. The write

### 6.1 Only with `--write`, only past both gates

It writes only with `--write`, and only after both gates pass in that same run and no listed comment stands without `--drop-comments`.

### 6.2 Atomic

It writes a temporary file in the config's own directory with the original's permissions, syncs it to disk, and renames it over the original. A failure leaves the original byte for byte and no temporary file, and exits `2`: `` error: <config>: could not write the migrated config (<reason>); the original is unchanged ``.

### 6.3 A concurrent edit

Just before the rename it reads the config again. If its bytes changed since the read that built the candidate, it removes the temporary file and exits `2`: `` error: <config> changed while it was being migrated; the original is unchanged, run the migration again ``.

### 6.4 A symbolic link, and nothing else written

A config that is a symbolic link is refused:

```text
refused: grund.toml is a symbolic link, and replacing it would replace the link with a regular file
  repair: replace the link with the file it points to, or migrate that file where it is the config
```

Nothing but the config is written: no `AGENTS.md`, no lock, no backup ([§DF-config-migrate.2.4](../decisions/functional/DF-config-migrate.md#24-no-backup-file)).

### 6.5 What it says after the write

After a successful write without `--full`, stderr holds only these, in order, each where it applies:

1. The `wrote` line.
2. The dropped-comment ranges, once, in file order.
3. Every notice of [§FS-config-migrate.3.2](FS-config-migrate.md#32-notices) and every carried-comment warning of [§FS-config-migrate.4.1.1](FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes), in file order.
4. The member lines ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces)).
5. The Project map note ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)).
6. The undo and the re-preview, spelled from the working directory ([§FS-config-migrate.6.5.1](FS-config-migrate.md#651-the-commands-a-migrate-report-prints-are-spelled-from-the-working-directory)).
7. The reminder to refresh managed instructions, exactly once and last.

With `--full`, the complete report of [§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report) comes first, then the same `wrote` line, dropped ranges, undo, re-preview and reminder; notices, warnings, member lines and the Project map note keep their complete-report places and are not repeated.

```text
wrote grund.toml: version 1 (24 lines) replaced by version 2
dropped the comments at grund.toml:6-7, grund.toml:12 and grund.toml:22-23 (version 1 lines)
to get version 1 back: it is in version control, e.g. git restore grund.toml
to see this candidate again: restore version 1 and run grund config migrate --to 2 --drop-comments
note: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them
```

The `dropped the comments` line appears only when `--drop-comments` dropped any, naming every range in file order, joined by `, ` and a final ` and `. The re-preview ends in `--drop-comments` exactly when that line appears, and `--full` and `--path-base` never follow it. The reminder follows every successful write, whether or not an `AGENTS.md` exists, and no other run: migration neither inspects managed blocks for drift nor runs `grund init`.

#### 6.5.1 The commands a migrate report prints are spelled from the working directory

A migrate report prints four commands to paste, none of them a report path ([§FS-errors.terms](FS-errors.md#terms-terms)): the undo and the re-preview after a write, the `[reference] strict = false` refusal's repair ([§FS-config-migrate.3.1.2](FS-config-migrate.md#312-bare-citations)), and the command on a `member:` line ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces)). Each is spelled for the directory `grund config migrate` ran in, whatever the report base, and `--path-base` moves none of them ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so pasted there each acts on what the report names: the undo and the re-preview on the config the run replaced, the repair on the refused project, the `member:` command on that member.

1. The undo is `to get version 1 back: it is in version control, e.g. git restore <file>`. `<file>` is the replaced config as git names it: spelled with `/` from the working directory, with `..` where it climbs, and each side's directory links resolved as canonical identity resolves them once discovery has selected the config ([§FS-config.1.5](FS-config.md#15-the-ancestor-search-keeps-the-path-through-directory-links)), so a `PATH` through a link names the file where git tracks it. A command is not a report path, so the bound [§FS-errors.4](FS-errors.md#4-determinism) puts on `..` does not hold it. With a working directory but no relative spelling, as for a config on another drive on Windows, it is absolute.
2. The re-preview is `to see this candidate again: restore version 1 and run grund config migrate --to 2`, then a space and `PATH` as it was given, when one was, then ` --drop-comments` where [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write) says, so the same discovery runs from the same directory.
3. The repair names the refused project's root after `--write`, and the `member:` command the member's root after `--to 2`, each spelled as `<file>` is; for an `.agents/grund.toml` that root is the project root, not `.agents/`. Each is left out, with the space before it, when it is the working directory. From `docs` below a workspace root, the member `apps/api` is migrated with `grund config migrate --to 2 ../apps/api`.
4. Each path is one word to a POSIX shell, on every platform. A path made only of ASCII letters, digits and `_ - . / , : @ % + =` is written as it is; any other, the empty one included, is written between single quotes, each `'` in it written `'\''`. An undo path that would start with `-` or `:` gets `./` before it, since `git` reads a leading `-` as an option and a leading `:` as pathspec magic; a repair or `member:` directory gets it only before a `-`, and `PATH` keeps the spelling it was given.

Without a working directory, as when the run's directory was deleted, `<file>` is the config as discovery reached it from `PATH`, the repair's directory is that project's root, and the `member:` command's is that root followed by the member's path below it, each written with `/`: absolute for an absolute `PATH`, and possibly relative for a relative one. Git cannot restore from the deleted directory, and a relative path pasted elsewhere can name another file. Run migration from a directory that exists, or pass an absolute `PATH`.

### 6.6 A second run is a no-op

A config already on version 2 prints `` <config> is already version 2; nothing to migrate ``, exit `0`, empty stdout, with or without `--write`, and is left alone, its modification time included. At a workspace root no `member:` line follows; `grund check` there names the configs below still on version 1 ([§FS-check.4.11.3](FS-check.md#4113-once-per-config-not-once-per-scope)).

## 7. Outcomes

| Situation | Outcome |
|---|---|
| No config found | Exit `2`: `` error: no grund.toml at <path> or above it; grund init writes one ``. |
| Unreadable, malformed, or failing v1 validation | Exit `2`, with the error `config validate` gives ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). |
| A path-shaped `[scan] exclude` | Exit `1` with each entry's reach, or exit `2` with the scan's error where an entry asks the scan and the version 1 scan cannot walk the tree ([§FS-config-migrate.3.1.4](FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry)). |
| `grund_config_version` above 2 | Exit `2`, with the unsupported-version error ([§FS-config.5](FS-config.md#5-schema-versioning)). |
| A symbolic link | Exit `1` ([§FS-config-migrate.6.4](FS-config-migrate.md#64-a-symbolic-link-and-nothing-else-written)). |

## 8. Workspaces

Only the file at `PATH` is migrated; mixed versions are legal ([§FS-config.5](FS-config.md#5-schema-versioning)). At a workspace root whose own config is on version 1, the report names, one line each, every member its own `[workspace]` lists that is still on version 1: `` member: <file> is version 1; migrate it with grund config migrate --to 2 <dir> ``, `<file>` the member's config as a report path, following `--path-base`, and `<dir>` the member's root spelled from the working directory ([§FS-config-migrate.6.5.1](FS-config-migrate.md#651-the-commands-a-migrate-report-prints-are-spelled-from-the-working-directory)), so the command migrates that member and no other config. At the workspace root it is the sentence `grund check` prints there ([§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)):

```text
member: api/grund.toml is version 1; migrate it with grund config migrate --to 2 api
```

The report gate runs over the whole workspace as `check` would, each member read by its own reader. A member's config is migrated, and its report gate run, from the member's root. No member is written, and there is no `--recursive` ([§DF-config-migrate.2.5](../decisions/functional/DF-config-migrate.md#25-no---recursive)). A config further down, a nested member's or an `include_root = false` block's below a listed one, earns no `member:` line; `grund check` at the root names it ([§FS-check.4.11.3](FS-check.md#4113-once-per-config-not-once-per-scope)).

### 8.1 A block with `include_root = false`

A version-1 file whose `[workspace]` sets `include_root = false` is not a project: nobody reads its own files, and its project settings have one reader, the probe that asks whether the block would have read a file had it been a project ([§FS-check.4.10.2](FS-check.md#4102-what-counts)). This section is the whole rule for such a block; a project, or a block with `include_root = true`, migrates as the rest of this page says. Decided in [§DF-config-migrate-opted-out-block](../decisions/functional/DF-config-migrate-opted-out-block.md#df-config-migrate-opted-out-block-a-version-1-block-with-include_root--false-migrates-to-its-envelope-and-workspace-table-held-by-the-report-gate).

- `[id] named_sections` is never refused on such a block, whatever its value ([§FS-config-migrate.3.1.1](FS-config-migrate.md#311-named-sections-off)): an authored key is dropped with a notice ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices)), and an omitted one adds nothing.
- A refusal that names the files a scan reads ([§FS-config-migrate.3.1.4](FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry), [§FS-config-migrate.3.1.5](FS-config-migrate.md#315-nested-homes)) reads the block's settings as a project's, as the probe does.
- The block migrates **compactly** when its default scope is empty, no root of its version-1 default scope existing on disk outside its expanded member roots, and it authors nothing a project would read. That scope is the probe's: the `[scan] include` entries and the scanned `[[kinds]]` homes, or the built-in rows' homes when it writes no `[[kinds]]`. Each authored key must then be `grund_config_version`, `project_name`, `project_description`, or a key of the `[workspace]` table or its imports, which have force on any block and map as [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default) says; `[id] named_sections`, either value; `[scan] include` that is `[]` or exactly version 1's default list, `["requirements.md", "docs", "e2e", "src"]`; an `[output]` key; or any other key at version 1's default, which a `[[kinds]]` entry and a `[citations]` table never are, having none.
- A block that fails either condition migrates as a project does, with every refusal of [§FS-config-migrate.3.1](FS-config-migrate.md#31-a-refusal-names-a-v1-repair) but the named-sections one, its missing-`include` refusal in [§FS-config-migrate.3.1.3](FS-config-migrate.md#313-an-include-entry-that-does-not-exist)'s bytes, and every default of [§FS-config-migrate.3.3](FS-config-migrate.md#33-defaults-are-written-out).
- A compact block's candidate is the version line and what the keys with force on any block map to, nothing else: no `[schema]`, `[rules]` or `[presentation]` default, no built-in row, no `written:` line, so the block takes version 2's defaults. Every other authored key is dropped with a notice, an `[output]` key with its own, and one more notice names the defaults left unwritten ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices)).
- Both gates run. The records gate compares only what the candidate keeps, the workspace records (members, optional members, `include_root` and imports), the project name and the description; the report gate runs unchanged. So a block whose root holds a file version 1's default scope never read but version 2's whole-root scope does, usually a `README.md` or the `AGENTS.md` `grund init` writes, is refused in one run, its candidate's run reporting that nobody reads the block ([§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)), and stays on version 1 until the file moves into a member or the block becomes a project.

A block with `[workspace] include_root = false` and `members = ["a"]` on lines 2 and 3, and nothing else, becomes:

```toml
grund_config_version = 2

[workspace]
members = ["a"]
include_root = false
```

## mapping: Every version-1 key and default

**Outcome:** *mapped* is written in v2; *written* is a v1 default written out; *notice* is dropped or changed and reported ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices)); *refused* names its repair ([§FS-config-migrate.3.1](FS-config-migrate.md#31-a-refusal-names-a-v1-repair)). Each row has a mapping case, and a test fails when a row of the v1 lowering table has none. The ground of a row is [§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart) unless it names another point of FS-config-v2. On a block with `include_root = false`, [§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false) decides which rows apply.

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
| `[[kinds]] fetch` | `fetch`; a value that names no executable has none ([§FS-config-migrate.3.1.10](FS-config-migrate.md#3110-a-fetch-that-names-no-executable)) | mapped, or refused for that value |
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
| `[[kinds]] value_chapter` | a field, which would add a section `check` compares; on a row that sets `fetch`, none | refused ([§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet)) |
| `[[kinds]] values` | `form = "value"` | refused by the guidance comparison ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)): version 2 gives every value row a declaration shape |
| `[scan] comment_prefixes` | each definition's `comment_prefixes`, the whole list | mapped |
| `[scan] docstring_python` | `.py`'s definition, `docstrings` python or off | mapped |
| `[scan] exclude` | `[schema.sources] exclude`, each name as the escaped glob `<name>/` | mapped; a path-shaped entry refused ([§FS-config-migrate.3.1.4](FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry)) |
| `[scan] extensions` | `languages`, `[schema.sources.definitions.v1_<profile>]` and `[schema.sources.extensions]` | mapped; JSON mixed with value authority, unreachable profiles and out-of-domain suffixes refused as [§FS-config-v2.schema.sources.migration](FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes) says, and the empty string with [§FS-config-migrate.3.1.9](FS-config-migrate.md#319-an-empty-string-in-scan-extensions)'s lines |
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

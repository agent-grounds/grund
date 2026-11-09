# FS-config-migrate: grund config migrate --to 2 shows the version-2 file that means what a version-1 config means, and writes it only when asked and only when it does

`grund config migrate --to 2` reads a version-1 `grund.toml` and shows the version-2 file that means the same thing, without touching anything. It says, line by line, where each setting went, what it drops and why, and every comment it cannot carry. Where a v1 setting has no v2 spelling it refuses and names the v1 edit that gets to one, so every change of meaning is one the user made, checked with `grund check`, and committed. It writes only with `--write`, only after two comparisons show that the new file means what the old one did, and only by an atomic replace. That is the one exception to "the config is never overwritten" ([§REQ-no-data-loss.4](../requirements/REQ-no-data-loss.md#4-one-migration-may-replace-the-config)). The translation is held to [§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart) and [§FS-config-v2.schema.sources.migration](FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes); the full table of every v1 key is [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default). It serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path): the way off version 1 is a reviewed path, not a hand rewrite nothing checks. Why it refuses rather than offers a flag, why there are two gates, and why `--to` is required are [§DF-config-migrate](../decisions/functional/DF-config-migrate.md#df-config-migrate-migration-refuses-what-version-2-cannot-spell-names-a-version-1-repair-and-writes-only-past-two-gates).

## terms: Terms

Leans on [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (config root, workspace, member), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding) and [§FS-config-v2.terms](FS-config-v2.md#terms-terms) (reader, place, complement). Defines:

- **Candidate**: the version-2 file migration would write for the config as it stands, with the same flags. It exists only when nothing is refused.
- **Preview**: a run without `--write`. It writes nothing. It prints the candidate and the complete report when its `--write` would write, and with `--full` when only listed comments block that write; otherwise it closes with what would block the write ([§FS-config-migrate.2.1](FS-config-migrate.md#21-stdout-is-the-candidate), [§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report)).
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
- `--full` is migration-only. After a successful write it restores the candidate on stdout and the complete report on stderr; without it successful writes use the compact report of [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write). On a preview that only listed comments refuse it restores the candidate and the complete report, followed by the preview's closing line ([§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report)). On every other preview, and on a refused write, it changes nothing. It accepts `PATH`, `--write`, `--drop-comments` and either `--path-base`; it does not relax `--to 2` or permit `--format`.
- `--drop-comments` lets the write drop exactly the listed comments ([§FS-config-migrate.4.4](FS-config-migrate.md#44-listed-comments-and---drop-comments)). It is accepted without `--write`, so a preview can predict the write.
- `--path-base` is accepted and spells the paths of the report ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)).
- `--format` is refused as it is for every `config` command ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)), with `` error: unknown flag `--format` ``, exit `2`, as `config show` answers it.

## 2. Streams and exits

### 2.1 Stdout is the candidate

On a preview whose `--write` would write, stdout carries exactly the bytes `--write` would write with the same flags, and nothing else ([§FS-errors.1](FS-errors.md#1-streams)); that preview, `--drop-comments` included, prints the same bytes on both streams whether or not `--full` is given. A refused preview has empty stdout by default. A successful write has empty stdout by default; with `--full` it prints exactly the candidate it wrote. The output choice never changes candidate bytes. When there is no candidate, because something is refused or the config is already version 2, stdout is empty, and so it is for a refused or failed write, with or without `--full`. So `grund config migrate --to 2 > new.toml` captures only the candidate, and captures one only when it exits `0`. The candidate is a version-2 file, so it prints its tables keyed by a row name in the order a version-2 file prints in ([§FS-config.4.2.4](FS-config.md#424-the-order-show-prints-in)): the rows' order, which is the order of the version-1 `[[kinds]]` entries. A preview that exits `1` only because a listed comment would be lost still has a candidate, without those comments, as `--drop-comments` would write it: the comments are what refuse, not the file. It prints that candidate only with `--full`, so `--full` and `--drop-comments` show the same candidate, and the `--full` run still exits `1`.

### 2.2 Stderr is the report

Every other line goes to stderr. Only an actual successful write without `--full` uses the compact report defined in [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write). Passing previews and `--write --full` use the complete report below; refused writes and refused previews retain that complete diagnostic report, without the lines item 5 leaves out, and close with item 6's line. Failures and already-version-2 runs retain their error or no-op messages and exits, without a refresh reminder. `<config>` is the config's path spelled from the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), `grund.toml` by default.

1. The heading: `<config>: version 1 → 2, preview, nothing written` for a preview, or `<config>: version 1 → 2, --write` with `--write`.
2. One block per refusal, in file order: `refused: <config>:<line>: <what version 2 cannot spell>`, then any indented detail lines, then `  repair: <the v1 edit>`. A refusal for a key the file omits has no line: `refused: <config>: …`. Every refusal is listed in one run.
3. The listed comments, when there are any: `comments with no single key to follow (--write refuses; --drop-comments drops exactly these):`, then one line each, `  <config>:<first>-<last>  <where>: <guidance>`, or `  <config>:<line>  <where>: <guidance>` for one line. `<where>` is `above <v1 key>`, `inside <v1 key>` or `standing alone`. Each entry explains why the comment cannot follow a single key. For a split or merged key or header, it names the actual v2 destinations of the authored settings in that entry, using the same mapping facts as the key map; a header does not suggest tables absent from that translation. For example, `[scan] include` containing `requirements.md`, `docs` and `src`, where `requirements.md` is already an FS place, names `[schema.kinds.code] folders` for `docs` and `src` and the existing `[schema.kinds.FS]` place. The entry tells the reader to retain the comment's prose, explicitly authorize the listed drops with `--write --drop-comments`, then re-add the comment at a named destination after a successful write. This advice remains subject to all other refusals and gates. When no unique destination can carry the comment (a dropped setting, a standalone comment, an omitted destination key or a trailing-comment collision), explain that reason and ask the reader to retain the prose and choose its placement manually; name a known shared destination for a collision. When refusals prevent a candidate, identify unresolved destinations as unavailable and direct the reader to repair and preview again, rather than claim a destination exists or promise that `--drop-comments` clears the refusal.
4. One line per notice, in file order: `dropped: <config>:<line>: <v1 key>: <why>`, or `changed: <config>:<line>: <v1 key>: <why>`. A carried comment on a translated value also gets the explanatory warning of [§FS-config-migrate.4.1.1](FS-config-migrate.md#411-check-carried-comments-when-the-value-vocabulary-changes), identifying the original config location/key and its destination and asking the reader to check the comment.
5. When there are no setting/file refusals: the key map, one `mapped: <config>:<line>: <v1 key> → <v2 spelling>` per authored key in file order, where a key version 2 needs no spelling for reads `→ nothing, version 2 always means it` and an `include` entry that is already a place reads `→ nothing, <entry> is a place of [schema.kinds.<ROW>] already`, then one `written: <v2 key> = <value>: version 1's default, written out` per default written out ([§FS-config-migrate.3.3](FS-config-migrate.md#33-defaults-are-written-out)), then member commands ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces)), then the gates' result ([§FS-config-migrate.5.4](FS-config-migrate.md#54-the-result-and-a-difference)) and any `note:` line. A refused run, preview or write, omits the `written:` defaults and the next-init Project map note unless its candidate is on stdout, which only a `--full` preview that only listed comments refuse prints; retain eligible mappings, members, notices and actual comparison diagnostics, including differing lines. A passing preview's output is unchanged.
6. After a write, the lines of [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write). A `--write` that refuses ends with `not written: <causes>; <config> is unchanged`, and a preview that refuses, with or without `--full`, ends with `would not write: <causes>`. `<causes>` is one list, defined once and read by both lines, so the same tree and flags name the same causes in the same words. It lists every blocking cause, in this order, separated by `; `:
   - Setting/file refusals: `1 refusal (follow the repairs above)` or `<n> refusals (follow the repairs above)`, counting the refusal blocks.
   - Listed comments without `--drop-comments`: `1 comment has no single key to follow (follow the comment guidance above)` or `<n> comments have no single key to follow (follow the comment guidance above)`, counting listed ranges, not their lines. Do not invite an unqualified move: the nearest v1 header may itself split. Authorized comments are not a cause. With other causes, the comment remedy applies only to the comments; their authorization does not clear the other causes.
   - A differing gate: `the records differ (inspect the comparison above)` or `grund check --suggestions differs (inspect the comparison above)`, naming the gate that differed. Gates not run because of an earlier refusal add no cause.

A line is never wrapped. Paths inside either report follow `--path-base`. The output policy changes no gates, comment authorization, discovery, atomic config-only writing, or exit codes. The compatibility decision for the successful-write default is [§DF-config-migrate-output.3](../decisions/functional/DF-config-migrate-output.md#3-compatibility-and-alternatives); for refused-write wording it is [§DF-config-migrate-output.4](../decisions/functional/DF-config-migrate-output.md#4-refused-writes-close-with-their-causes-and-remedies); for refused previews it is [§DF-config-migrate-refused-preview](../decisions/functional/DF-config-migrate-refused-preview.md#df-config-migrate-refused-preview-a-refused-migration-preview-closes-with-its-causes-and-keeps-its-candidate-behind---full).

### 2.3 Exits

| Exit | When |
|---|---|
| `0` | The preview's `--write` would write; or the write was made; or the config is already version 2. |
| `1` | Refused: a v1 setting with no v2 spelling, a difference at either gate, a comment that would be lost without `--drop-comments`, or a config that is a symbolic link. A preview with the same flags exits `1` for the same cause. |
| `2` | The command could not run: a usage error, no config, a config that does not load, an unsupported version, a version 1 scan that cannot walk the tree where a refusal asks what it reads ([§FS-config-migrate.3.1.4.1](FS-config-migrate.md#3141-the-files-named-are-the-ones-version-1-reads)), or an I/O failure or concurrent edit during the write. The original is untouched in every case. |

## 3. Refusals, notices and defaults

### 3.1 A refusal names a v1 repair

A setting version 2 cannot spell is never approximated. The refusal names the v1 key and its line and gives a repair: a v1 edit the user makes, checks with `grund check` under version 1, and commits. That commit is the visible authorization, and `grund check` shows its effect before any v2 file exists. The full set is in [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default); the ones most v1 files meet are below, each with its exact lines.

#### 3.1.1 Named sections off

`grund init` writes `named_sections = false`, and an omitted key means the same, so most v1 files carry it. Version 2 always reads named sections ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)).

```text
refused: grund.toml:7: [id] named_sections = false turns named sections off, and version 2 always reads them
  repair: set [id] named_sections = true under version 1; grund check shows its effect
```

When the key is omitted the block reads ``refused: grund.toml: [id] named_sections is not set, so version 1 reads no named sections, and version 2 always reads them`` and ``  repair: add [id] named_sections = true under version 1; grund check shows its effect``. A block with `include_root = false` is not refused here ([§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false)).

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

A file that omits `[scan] include` reads that same default list, and a missing default entry is refused too, never dropped: a `src` created later would be scanned under version 1 and not under version 2. One block names every missing default entry, and its repair authors the list without them, unless the file is a block with `include_root = false` that migrates compactly ([§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false)):

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

##### 3.1.4.1 The files named are the ones version 1 reads

The files named and counted are the ones the version 1 scan of this tree reads ([§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads)) and the entry would stop, read as one version 2 `[schema.sources] exclude` glob ([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)). Version 1 never matched the entry, so that scan is the one its removal keeps. A file it does not read has no reading for a glob to stop, so it is neither named nor counted. Such a file can lie outside `[scan] include` and every home, or have a hidden name or a hidden directory above it ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)). It can be a file the ignore files name while `respect_gitignore` holds ([§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)), or have an extension the list does not hold ([§FS-scan.scope.extensions](FS-scan.md#scopeextensions-an-extension-makes-a-file-readable-not-declarable)). An entry is not a scan root, so the same holds when it names one file. The refusal stands when no file is named, because version 2 would still read the entry as a glob. The count is then `0 files` and the `would stop reading:` line is absent:

```text
refused: grund.toml:16: [scan] exclude entry `docs/data.txt` contains `/`, so version 1 never matched it; as a version 2 glob it would stop reading 0 files
  repair: remove `docs/data.txt` from exclude under version 1, which keeps what is scanned exactly as it is
```

The entry is read alone and as version 2 reads it, in the order the walk meets a path: down from the file's scan root, each directory before what lies under it and the file last. So `*` does not cross a `/`, and a directory the entry matches takes what lies below it: over `docs/a.md` and `docs/sub/b.md`, `docs/*.md` stops `docs/a.md` alone and `docs/*` stops both. A `/` only at the end anchors nothing, so `drafts/` stops `lib/drafts/x.md` with no `drafts` at the root. Glob syntax in a name on disk still reads as glob syntax, so `docs/[x]` stops `docs/x/b.md` and not `docs/[x]/a.md`. Nothing normalizes the entry's spelling, so `docs//x` and `./docs/x` stop nothing, and trailing spaces drop as they do from a `.gitignore` line, so `docs/x ` stops what `docs/x` stops.

A file's scan root is the deepest place holding it among those version 1's homes and `[scan] include` entries become ([§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default)), and otherwise the config root. A scan root is read even where a pattern names it ([§FS-config-v2.schema.sources.exclude.roots](FS-config-v2.md#schemasourcesexcluderoots-how-exclude-composes-with-the-other-filters)), so an entry naming a place stops nothing at that place: `docs/fs` over a home `docs/fs` counts `0 files`, as `docs/f*` does, while `docs/fs/*` stops what lies below it. An `include` entry inside a home becomes no place, so an entry naming it stops what it holds.

For paths that stay inside the config root, spelling an include or home with `.` or `..` changes neither which scan root holds a read file nor which files the glob stops. These path comparisons and the printed config-relative names use lexical paths with those components resolved, without resolving symlinks; the authored glob keeps its spelling and semantics above. Over `docs/b.md` and `docs/x/a.md`, with FS home `docs` and exclude `docs/x`, include `docs/x/..` or `docs/../docs` therefore reports `would stop reading 1 file` and exactly `would stop reading: docs/x/a.md`. The same holds with include `other` and the FS home spelled `docs/x/..` or `docs/../docs`. A home spelled `docs/x/../x` or `docs/../docs/x` still protects `docs/x` itself as a scan root, so exclude `docs/x` counts `0 files` there.

Some entries stop nothing as a glob: a `#` comment, a negation, because `!` only rescues what an earlier line matched and this entry is read alone, and a pattern the version 2 reader refuses ([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)), such as `docs/[a`. Migration reads the version 1 file tolerantly, so such a pattern fails nothing here. Each counts `0 files`, and its refusal stands.

On a block with `include_root = false`, nobody reads the block's own files, and the scan meant here and in [§FS-config-migrate.3.1.5.1](FS-config-migrate.md#3151-the-files-named-are-the-ones-version-1-reads) is the one [§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false) names: the block's settings read as a project's, which is the probe's reading.

An entry that stops nothing, or whose glob can match only the path it spells, needs the scan only for a path on disk. That is an entry anchored by a leading or inner `/` and holding none of `*`, `?`, `[`, `{` or `\`, its path spelled without trailing spaces or a `/` at either end. A path that is not on disk is read by nobody ([§FS-check.4.10.2](FS-check.md#4102-what-counts)), so such an entry names nothing and counts `0 files` without the scan being asked. Any other entry can stop a file it does not spell, whatever stands at its spelling, so it always asks the scan: `docs/*.md` and `drafts/` are two. Where an entry asks the scan and the version 1 scan cannot walk this tree at all, nothing can say what it reads there. A `[workspace]` member glob that names no directory is one such tree, and `grund list` fails on it with exit `2`: with a path-shaped entry's error while the file holds one, and with that glob's error where it holds none. A migration that asks the scan what that entry reads fails with that glob's error and exit `2`, rather than print a count it could not take ([§FS-config-migrate.7](FS-config-migrate.md#7-outcomes)). Migration's tolerant read loads the config, and only a refusal that needs what the scan reads asks for the walk, so on that tree a run with no such refusal goes on as on any other. The same holds for the detail of [§FS-config-migrate.3.1.5.1](FS-config-migrate.md#3151-the-files-named-are-the-ones-version-1-reads): an inner home that is not on disk names nothing, and one that is fails the run where the scan cannot walk the tree. A path the scan cannot read inside a tree it walks is not read, so it is not named, and its error is not printed with the refusal.

#### 3.1.5 Nested homes

A v1 file inside two nested homes belongs to the complement; in version 2 the deepest place owns it ([§DF-v2-place-ownership](../decisions/functional/DF-v2-place-ownership.md#df-v2-place-ownership-in-v2-the-row-named-code-is-the-complement-and-a-file-belongs-to-its-listed-row-or-else-its-deepest-folder)). Migration refuses and names every file whose owner would change, even when no finding on this tree would differ.

```text
refused: grund.toml:15: [[kinds]] `AR` home `docs/arch` is inside `FS` home `docs`; version 1 gives the files in both to the complement, version 2 to the deepest place
  would change owner: docs/arch/store.md
  repair: un-nest the two homes under version 1
```

##### 3.1.5.1 The files named are the ones version 1 reads

The files named are the ones the version 1 scan of this tree reads in the inner home ([§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads)). A file it does not read has no owner to change, so it is not named. The home is a scan root, which outruns every rule about descent ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)). A folder home is walked even when it is hidden, lies below a hidden directory or is named by the ignore files, and a `file` home the ignore files name is read. Below a folder home the scan's rules apply as usual. A file there is not named when it is hidden or below a hidden directory ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), named by the ignore files while `respect_gitignore` holds ([§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)), or of an extension the list does not hold. A `file` home whose name is hidden, or whose extension the list does not hold, is not read either, so it names nothing ([§FS-scan.scope.extensions.root](FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so)). The refusal stands when no file is named, because the two homes still nest. The `would change owner:` line is then absent.

The path comparisons and printed names follow the same lexical path rule as [§FS-config-migrate.3.1.4.1](FS-config-migrate.md#3141-the-files-named-are-the-ones-version-1-reads). With FS home `docs`, REQ home `docs/x`, and read files `docs/b.md` and `docs/x/a.md`, include `docs/x/..` or `docs/../docs` therefore reports exactly `would change owner: docs/x/a.md`: neither spelling puts `docs/b.md` in the inner home or leaves `docs/x/a.md` out of it.

#### 3.1.6 A renamed complement

```text
refused: grund.toml:9: [[kinds]] `source` is a non-citable row with no home, which renames the complement; version 2 calls the complement `code`
  repair: rename the row, and its [citations.source] table, to `code` under version 1
```

#### 3.1.7 Settings this grund has no v2 spelling for yet

A `value_chapter`, a lead budget counted in lines or bytes, and a non-default `[fmt.cross_refs]` each refuse with the line of the key:

```text
refused: grund.toml:11: [[kinds]] `FS` value_chapter would become a field, which adds a "Declaration shapes" section grund check compares
  repair: removing value_chapter alone ends the comparison of every binding that quoted the chapter; to keep them compared, under version 1 rewrite its roots as marked sections, remove value_chapter and cite the new paths
refused: grund.toml:5: [reference] lead_size_warning counts lines, and version 2 counts a lead in words
  repair: restate it with unit = "words" under version 1
refused: grund.toml:8: [fmt.cross_refs] enabled = false has no version 2 spelling in this grund
  repair: none yet; keep version 1, or set enabled = true under version 1 if fmt may write links
```

The same `lead_size_warning` line reads `counts bytes` for a byte budget, and `anchor_format` other than `"github"` reads `` [fmt.cross_refs] anchor_format = "<value>" has no version 2 spelling in this grund ``. The `value_chapter` repair says what removal costs, for the reason [§FS-config-migrate.3.1.7.2](FS-config-migrate.md#3172-removing-a-value-key-alone-ends-the-comparison) gives.

##### 3.1.7.1 A `value_chapter` on a fetched kind

A row that sets `fetch` as well as `value_chapter` is refused at the same line, in words of its own:

```text
refused: grund.toml:9: [[kinds]] `AR` value_chapter has no version 2 spelling on a fetched kind, because a fetched kind takes no fields
  repair: none yet; keep version 1, or under version 1 rewrite its roots as marked sections, remove value_chapter and cite the new paths
```

The reason line of [§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet) stays for a row without `fetch`, and is true of it: version 2 loads that field. On this row it does not hold, so the row has words of its own.

- **The obstacle is another one.** [§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts) gives a fetched kind no fields, so the key cannot become one and adds no "Declaration shapes" section. The refusal names what actually stops the row.
- **Removing the key is not the way through**, for the reason [§FS-config-migrate.3.1.7.2](FS-config-migrate.md#3172-removing-a-value-key-alone-ends-the-comparison) gives of every such row. Here there is no field to keep the values compared either, so the repair offers the rewrite and version 1 alone.

The rewrite holds on this row as [§FS-config-migrate.3.1.7.2](FS-config-migrate.md#3172-removing-a-value-key-alone-ends-the-comparison) says it holds on any.

##### 3.1.7.2 Removing a value key alone ends the comparison

Removed alone, a `value_chapter` or a `values = true` ends the value comparison for every binding that quoted its values, so a wrong literal goes from an error to exit `0`, whatever the row's `fetch`. The gates of [§FS-config-migrate.5](FS-config-migrate.md#5-equivalence-two-gates) cannot say so: they compare the tree with the key already gone, and truthfully report no difference. The repair is the only place left to say it, so the repair of a `value_chapter` row without `fetch` ([§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet)) and of every `values = true` row ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)) says what removal costs and names the rewrite that keeps the values compared. It weighs no generated section, because removal leaves that section out: the section comes with keeping the comparison under version 2, through a field for `value_chapter` and through `form = "value"` for `values = true`, which is what the reason line says.

A root rewritten as a marked section ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)) is judged the same under both versions, whatever the row's `fetch`: a wrong literal bound to it fails `grund check` before the migration and after it, with the key gone nothing here refuses the row, and `grund init` writes no "Declaration shapes" section for it. The repair speaks of the tree `grund check` and this command read; what `grund fetch` may write into a fetched row's home is [§FS-fetch.3.2](FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)'s to say.

#### 3.1.8 A string that closes early

Version 1 reads a string from its opening quote to the last quote of its value or its element, every quote between kept, and keeps that reading for good ([§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)); version 2 ends a string at its first unescaped quote ([§FS-config-v2.reader.6](FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote)). Writing the value out would carry the same string into version 2 escaped, `"docs\" \"x"`, so a forgotten comma or a stray quote would become valid TOML that means what the typo meant. Migration refuses instead, at the key's line, naming the string as version 1 reads it, with the repair version 2's warning gave through grund 0.21.x ([§FS-config-v2.reader.6.window](FS-config-v2.md#reader6window-through-grund-021x-a-string-that-closes-early-is-read-as-before-and-warns)), made under version 1: the first half alone for a scalar, and both halves for a list element.

```text
refused: grund.toml:8: [[kinds]] folder reads the one string `docs" "x`, and version 2 ends a string at its first unescaped quote
  repair: end the string at its closing quote under version 1: write `\"` for a quote inside it; grund check shows its effect
refused: grund.toml:12: [scan] include reads the one string `a"b`, and version 2 ends a string at its first unescaped quote
  repair: end the string at its closing quote under version 1: write `\"` for a quote inside it, or a `,` between two strings; grund check shows its effect
```

A key earns one block, naming the first string in its value that closes early, and the text in backticks is that string as written from its opening quote to the last quote, as version 1 reads it. The refusal stays after grund 0.22.0 refuses such a string in version 2 ([§FS-config-v2.reader.6.refusal](FS-config-v2.md#reader6refusal-from-grund-0220-a-string-that-closes-early-is-expected-string)): a version-1 file reads it for good, and only its migration meets the rule.

#### 3.1.9 An empty string in `[scan] extensions`

Version 1 reads each file with no extension through an empty string in `[scan] extensions`, for good ([§FS-scan.scope.extensions.empty](FS-scan.md#scopeextensionsempty-in-version-1-an-empty-string-in-the-list-reads-a-file-with-no-extension)), and version 2 has no such entry ([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)). Removing the entry is the repair, and it costs those reads, so this refusal says what the entry reads and what the repair costs.

```text
refused: grund.toml:4: [scan] extensions names ``, which is not a final extension version 2 can map; version 1 reads every file with no extension through it, and version 2 reads none
  repair: remove `` from extensions under version 1, and give any such file you want read an extension the list holds; grund check shows its effect
```

The longer lines are for this entry alone, because their added clauses are true of no other. Every other entry a version 2 mapping cannot hold, `tar.gz` or `*.md` for example, keeps its two lines as they stand:

```text
refused: grund.toml:4: [scan] extensions names `tar.gz`, which is not a final extension version 2 can map
  repair: remove `tar.gz` from extensions under version 1
```

Under the entry, the files [§FS-config-migrate.3.1.4.1](FS-config-migrate.md#3141-the-files-named-are-the-ones-version-1-reads) and [§FS-config-migrate.3.1.5.1](FS-config-migrate.md#3151-the-files-named-are-the-ones-version-1-reads) name are still the ones version 1 reads. So they include each file with no extension that the scan reads through it ([§FS-scan.scope.extensions.empty](FS-scan.md#scopeextensionsempty-in-version-1-an-empty-string-in-the-list-reads-a-file-with-no-extension)), `Makefile` for one, and leave out one it does not read, such as a hidden `.gitkeep`. Those details ask the scan itself, so a file whose extension is not UTF-8 is named exactly when the scan reads it. Under a list naming an extension spelled with U+FFFD it is not read, because the scan asks the path and not the printed name, so it is neither named nor counted.

#### 3.1.10 A `fetch` that names no executable

A version 1 row whose `fetch` names no executable loads for good ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable), [§FS-config.5.2](FS-config.md#52-every-older-version-keeps-its-meaning)), and version 2 refuses that value at load ([§FS-config-v2.checks.config-fetch-names-nothing](FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)). Writing the value out would carry the mistake across, and no command can know whether the author meant a program or no fetching at all. Migration refuses at the key's line, with the repair [§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable) prints, made under version 1:

```text
refused: grund.toml:6: [[kinds]] `AR` fetch names no executable
  repair: under version 1, name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors; grund check shows its effect
```

The test is of the string alone ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)), so a whitespace-only value is refused whether or not a file of that name exists. A value with any other character is mapped as written.

A row that also sets `value_chapter` earns two blocks, this one at the `fetch` line and that of [§FS-config-migrate.3.1.7.1](FS-config-migrate.md#3171-a-value_chapter-on-a-fetched-kind) at the `value_chapter` line, one each, ordered by line as every refusal of [§FS-config-migrate.3.1](FS-config-migrate.md#31-a-refusal-names-a-v1-repair) is. A person who removes the `fetch` key under the first block meets the words of [§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet) on the next run, because the row no longer sets `fetch`.

### 3.2 Notices

Some settings are written with a notice rather than refused, because version 1 gave them no force ([§DISC-core-concerns.7.5](../discussions/proposals/2026-09-30-core-concerns.md#75-what-has-no-v2-spelling-and-how-migrate-reports-it)):

- A positive `[citations] default` creates no obligation in v1, and is written as `may` ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)): `` changed: grund.toml:20: [citations.FS] default = "must": version 1 gives a positive default no force, so it is written as may ``.
- `[output] format` and `relative_paths` are dropped. At their defaults: `` dropped: grund.toml:22: [output] format = "text": the default of --format, nothing to add `` and `` dropped: grund.toml:24: [output] relative_paths = true: the default of --path-base, nothing to add ``. Otherwise the notice names the flag that keeps the behaviour: `` dropped: grund.toml:22: [output] format = "json": pass --format json to keep it `` and `` dropped: grund.toml:24: [output] relative_paths = false: pass --path-base=invocation to keep it ``.
- `color` is dropped: `` dropped: grund.toml:23: [output] color: nothing reads it ``.
- On a block with `include_root = false`, the keys [§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false) lists as without force are dropped, each with one line: `` dropped: grund.toml:2: [id] named_sections: include_root = false, so no project scans this block's own files ``. A block that migrates compactly also gets one line with no line number, after the others, for the defaults it does not write: `` dropped: grund.toml: version 1's defaults: include_root = false, so no project scans this block's own files; it takes version 2's defaults ``.

### 3.3 Defaults are written out

Every v1 default that differs from version 2's is written out explicitly, so the file keeps v1's meaning after later edits ([§FS-config-v2.schema.sources.migration](FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes)): the slug pattern, the ID format, the default exclusions as globs, `declarations = "any-comment"`, the explicit language list with its definitions, a suggested note length as the `may` rung of `[schema.notes.lines]`, and `[presentation] rules = "root"` ([§DF-scoped-guidance](../decisions/functional/DF-scoped-guidance.md#df-scoped-guidance-a-version-2-rows-guidance-is-written-above-its-places-with-a-root-pointer-for-agents-that-do-not-load-nested-files)). A file with no `[[kinds]]` gets every built-in row written out. A block with `include_root = false` that migrates compactly writes none of them ([§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false)). Each one is a `written:` line of the complete report, in the candidate's order; compact success omits that narration, never the defaults in the candidate, and a refused run lists the defaults only beside its candidate on stdout ([§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report)).

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

Listing guidance follows [§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report), including for split headers and comments whose setting has no destination. It does not change which comments are carried under [§FS-config-migrate.4.1](FS-config-migrate.md#41-attached-comments-move-with-a-one-to-one-key).

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

The preservation advice in [§FS-config-migrate.2.2](FS-config-migrate.md#22-stderr-is-the-report) describes a manual recovery after that explicit consent and a successful write. It does not carry a listed comment automatically, authorize its loss, or relax any other write condition.

## 5. Equivalence: two gates

The mapping table "is what a migration audits against, and tests on the records hold it, not a comparison of one tree's reports" ([§FS-config-v2.mapping](FS-config-v2.md#mapping-what-the-records-keep-apart)). So the records come first and the report on this tree second. Neither gate is skipped and neither is relaxed by a flag.

### 5.1 The records gate

The candidate is loaded through the v2 reader, the path a file on disk goes through, and its project records are compared with the v1 file's by one comparison that ignores only the following, or, for a block with `include_root = false`, what [§FS-config-migrate.8.1](FS-config-migrate.md#81-a-block-with-include_root--false) says it ignores:

- the version, and where the config was found;
- the dropped `[output]` preferences ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices));
- the source model, compared suffix by suffix: each suffix's reader class, comment prefixes, docstring reading, note-measurement rule and declaration eligibility, rather than the language names that carry them;
- grounding, compared as the effective pair each row resolves to, rather than as v1's inherited keys or v2's whole ladders;
- `include`, compared as the set of derived scan roots;
- the nesting rule, when no two v1 homes nest, because the rule then decides nothing;
- a positive citation default, compared as `may`.

### 5.2 The report gate

`grund check --suggestions` runs twice in the same process, on the same root, members and invocation: once with the file as read, once with the candidate. It compares every finding's code, channel, path, line, column and message, the run warnings, the projects checked, and the exit status. Each finding is one line in its run, as many times as `check` would print it: a run warning `check` prints once is compared once, however many places the run carries it, so a candidate that adds one finding adds one `+ ` line. It ignores only:

- a finding located in the config itself, whose line is mapped through the key map before comparing; a finding with no mapped line is a difference;
- the config-table names inside messages, such as `[scan] exclude` against `[schema.sources] exclude`: both runs are spelled as v1;
- the `config-version-1` caution ([§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)), which the run on the file as read carries and the candidate's run never does, so it is dropped from both before comparing;
- an `agents-init` finding about a managed block's text outside the block version and the five generated sections: [§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance) passes that difference on purpose, and the candidate's run reports it because `check` compares the whole block ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)). Kept, it would refuse every version-1 migration, whose citation scheme paragraph always changes.

### 5.3 Guidance

Migration also renders in memory what `grund init` would write into each `AGENTS.md`, for the file as read and for the candidate, whether or not an `AGENTS.md` exists yet. A difference in the block's version or in one of its five generated sections — `### Citation directions`, `### Clickable citations`, `### Declaration shapes`, `### Chapter rules` and `### Scoped guidance` — is a refusal, because those carry the rules the tree is held to, and a migration must not move them. Every other difference passes: the rest of the block describes the config, the citation scheme, the project map and the declaration forms among it, so it is expected to change under a migration, and `check` reports the block stale until `grund init` runs ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)). A value row is the common case: version 2 gives it a declaration shape, which adds a section version 1 never wrote:

```text
refused: grund.toml:11: [[kinds]] `TERM` values = true adds a "Declaration shapes" section to the AGENTS.md block in version 2, which grund check compares
  would write: - **TERM**
  would write:   - field <number> may · values
  repair: removing values alone ends the comparison of every binding that quoted its values; to keep them compared, under version 1 rewrite its roots as marked sections, remove values and cite the new paths
```

The repair says what removal costs, for the reason [§FS-config-migrate.3.1.7.2](FS-config-migrate.md#3172-removing-a-value-key-alone-ends-the-comparison) gives, on a row that sets `fetch` as on one that does not. The reason line holds on both rows too. `values = true` becomes `form = "value"` whatever the row's `fetch`, and that a fetched kind takes no fields ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)) does not stop it: a form is not a field, and `field <number>` is how [§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node) prints a value kind's numbered components. So version 2 writes this section for a fetched value kind as for any other, and `grund check` compares it.

Any other compared difference is one block per section, its lines prefixed `- ` for version 1 and `+ ` for the candidate, or a version alone when no section differs:

```text
refused: grund.toml: the AGENTS.md block grund init writes would change its citation directions section, which grund check compares
  + <line>
  repair: none keeps the generated guidance; keep version 1 until that section may change
refused: grund.toml: the AGENTS.md block grund init writes would go from v19 to v21, which grund check compares
  repair: none keeps the generated guidance; keep version 1 until that block may change
```

The Project map row the complement gains once its places are listed is a `note:` line rather than a refusal, because the Project map is not one of the generated sections ([§FS-init.2.3.7.2](FS-init.md#2372-what-check-compares)): `` note: the next grund init adds a `code` row to the AGENTS.md Project map, because the complement's places are now listed; grund check reports the block stale until then ``.

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

A config that is already version 2 is reported as `` <config> is already version 2; nothing to migrate ``, exit `0`, empty stdout, with or without `--write`. The file, its modification time included, is left alone. The line speaks of that one file: at a workspace root no `member:` line follows it ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces)), and the configs below still on version 1 are named by `grund check` there ([§FS-check.4.11.3](FS-check.md#4113-once-per-config-not-once-per-scope)).

## 7. Outcomes

| Situation | Outcome |
|---|---|
| No config found | Exit `2`: `` error: no grund.toml at <path> or above it; grund init writes one ``. Creating a config is not migrating one. |
| Unreadable, malformed, or failing v1 validation | Exit `2`, with the error `config validate` gives ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). |
| A path-shaped `[scan] exclude` | Read tolerantly for diagnosis only; exit `1` with each entry's reach ([§FS-config-migrate.3.1.4](FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry)). Where an entry asks the scan what it reads and the version 1 scan cannot walk the tree, exit `2` with the scan's error instead ([§FS-config-migrate.3.1.4.1](FS-config-migrate.md#3141-the-files-named-are-the-ones-version-1-reads)). |
| `grund_config_version` above 2 | Exit `2`, with today's unsupported-version error ([§FS-config.5](FS-config.md#5-schema-versioning)). |
| Already version 2 | Exit `0`, nothing to migrate ([§FS-config-migrate.6.6](FS-config-migrate.md#66-a-second-run-is-a-no-op)). |
| A symbolic link | Exit `1` ([§FS-config-migrate.6.4](FS-config-migrate.md#64-a-symbolic-link-and-nothing-else-written)). |

## 8. Workspaces

Only the file at `PATH` is migrated; mixed versions are legal ([§FS-config.5](FS-config.md#5-schema-versioning)). At a workspace root whose own config is on version 1, the report names each member the root's own `[workspace]` lists that is still on version 1, and the command that migrates it, one line each: `` member: <member>/grund.toml is version 1; migrate it with grund config migrate --to 2 <member> ``. The complete report places these after the `written:` lines and before the gates' result; compact success retains them at the position defined in [§FS-config-migrate.6.5](FS-config-migrate.md#65-what-it-says-after-the-write), without requiring default narration before them. The report gate runs over the whole workspace as `check` would, each member read by its own reader. A member's config is migrated from the member's root, and the report gate runs from there. No member is written; there is no `--recursive`, because several files cannot be replaced atomically together. A config further down, a nested member's or an `include_root = false` block's below a listed one, earns no `member:` line; `grund check` at the root names it ([§FS-check.4.11.3](FS-check.md#4113-once-per-config-not-once-per-scope)).

### 8.1 A block with `include_root = false`

A version-1 file whose `[workspace]` sets `include_root = false` is not a project: nobody reads its own files ([§FS-check.4.10.1](FS-check.md#4101-nobody-reads-the-blocks-own-files)). The settings version 1 gives a project have one reader there, the probe that asks whether the block would have read a file had it been a project ([§FS-check.4.10.2](FS-check.md#4102-what-counts)). This point is the whole rule for such a block. The points that refer here do not restate it, and an ordinary project, or a block with `include_root = true`, migrates exactly as the rest of this specification says. Why the block may be held by the report gate alone is [§DF-config-migrate-opted-out-block](../decisions/functional/DF-config-migrate-opted-out-block.md#df-config-migrate-opted-out-block-a-version-1-block-with-include_root--false-migrates-to-its-envelope-and-workspace-table-held-by-the-report-gate).

#### 8.1.1 Named sections

`[id] named_sections` has no reader on such a block, whatever its value, so it is never refused there ([§FS-config-migrate.3.1.1](FS-config-migrate.md#311-named-sections-off)). An authored key is dropped with a notice ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices)), and an omitted one adds nothing to the report.

#### 8.1.2 When the block migrates compactly

The block migrates **compactly** when both of these hold:

1. **Its default scope is empty.** No root of its version-1 default scope exists on disk outside its expanded member roots. That scope is the one [§FS-check.4.10.2](FS-check.md#4102-what-counts) takes: the `[scan] include` entries and the scanned `[[kinds]]` homes, or the built-in rows' homes when it writes no `[[kinds]]`.
2. **It authors nothing a project would read.** Each authored key is one of these:
   - `grund_config_version`, `project_name`, `project_description`, or a key of the `[workspace]` table or its imports. These have force on any block, and they map as [§FS-config-migrate.mapping](FS-config-migrate.md#mapping-every-version-1-key-and-default) says.
   - `[id] named_sections`, either value.
   - `[scan] include`, when it counts as omitted: it is `[]`, or exactly version 1's default list `["requirements.md", "docs", "e2e", "src"]`, as `grund init` writes it. Item 1 has already found each of those entries missing.
   - An `[output]` key, a notice on every file ([§FS-config-migrate.3.2](FS-config-migrate.md#32-notices)).
   - Any other key whose value is version 1's default for that key. Its omission would mean the same. A `[[kinds]]` entry and a `[citations]` table have no default, so either one takes the block off the compact path.

A block for which either item fails migrates as a project does, with every other refusal of [§FS-config-migrate.3.1](FS-config-migrate.md#31-a-refusal-names-a-v1-repair) and every default of [§FS-config-migrate.3.3](FS-config-migrate.md#33-defaults-are-written-out). Its missing-`include` refusal keeps the bytes [§FS-config-migrate.3.1.3](FS-config-migrate.md#313-an-include-entry-that-does-not-exist) gives it.

#### 8.1.3 What a compact block's candidate holds

The candidate is the version line, then what the keys of item 2's first bullet map to, and nothing else. No `[schema]`, `[rules]` or `[presentation]` default is written, no built-in row, and no `written:` line. The block takes version 2's defaults. Every other authored key is dropped with a notice, an `[output]` key with its own, and one more notice names the defaults left unwritten. Their lines are in [§FS-config-migrate.3.2](FS-config-migrate.md#32-notices). The reported block, `[workspace] include_root = false` and `members = ["a"]` on lines 2 and 3, becomes:

```toml
grund_config_version = 2

[workspace]
members = ["a"]
include_root = false
```

#### 8.1.4 Both gates still run

The records gate compares what the candidate keeps: the workspace records (members, optional members, `include_root` and imports), the project name and the description. Everything else that tells this version-1 file from its candidate is ignored there and left to the report gate ([§FS-config-migrate.5.1](FS-config-migrate.md#51-the-records-gate)). The report gate runs unchanged ([§FS-config-migrate.5.2](FS-config-migrate.md#52-the-report-gate)). A block whose root holds a file version 1's default scope never read, but version 2's whole-root scope does, therefore stops there, in one run. That file is usually a `README.md`, or the `AGENTS.md` `grund init` writes. Its candidate's run reports that nobody reads the block ([§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)). Such a block stays on version 1 until the file moves into a member or the block becomes a project.

#### 8.1.5 The reports for the reported tree

The reported tree is a version-2 root listing `group`, the block above at `group/grund.toml`, and a version-2 member at `group/a/grund.toml`. `grund config migrate --to 2 group` prints the candidate above on stdout, exits `0`, and reports:

```text
grund.toml: version 1 → 2, preview, nothing written
dropped: grund.toml: version 1's defaults: include_root = false, so no project scans this block's own files; it takes version 2's defaults
mapped: grund.toml:2: [workspace] include_root → [workspace] include_root
mapped: grund.toml:3: [workspace] members → [workspace] members
compared: the records are equal, and grund check --suggestions on this tree gives the same findings and the same exit before and after
```

With `--write` it writes those bytes, exits `0`, and reports:

```text
wrote grund.toml: version 1 (3 lines) replaced by version 2
dropped: grund.toml: version 1's defaults: include_root = false, so no project scans this block's own files; it takes version 2's defaults
to get version 1 back: it is in version control, e.g. git restore grund.toml
to see this candidate again: restore version 1 and run grund config migrate --to 2
note: existing AGENTS.md managed instructions may still describe version 1; run grund init in the migrated project to refresh them
```

The block that the two old repairs produced also adds `[id] named_sections = true` on line 2 and `[scan] include = []` on line 5. It writes the same candidate, with two more lines after `wrote`:

```text
dropped: grund.toml:2: [id] named_sections: include_root = false, so no project scans this block's own files
dropped: grund.toml:5: [scan] include: include_root = false, so no project scans this block's own files
```

The same reported block with a `README.md` beside its config is refused by `--write` at the report gate, exit `1`. The `differs:` lines are the unread-block warning as this grund prints it:

```text
grund.toml: version 1 → 2, --write
dropped: grund.toml: version 1's defaults: include_root = false, so no project scans this block's own files; it takes version 2's defaults
mapped: grund.toml:2: [workspace] include_root → [workspace] include_root
mapped: grund.toml:3: [workspace] members → [workspace] members
differs: grund check --suggestions on this tree
+ run-warning warning unread-workspace-block grund.toml:2:- grund.toml:2: no project scans `.`, so its citations are never checked. Set include_root = true, or configure another project's source scope to read it.
not written: grund check --suggestions differs (inspect the comparison above); grund.toml is unchanged
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
| `[[kinds]] value_chapter` | a field, which would add a section `check` compares; on a row that sets `fetch`, none ([§FS-config-migrate.3.1.7.1](FS-config-migrate.md#3171-a-value_chapter-on-a-fetched-kind)) | refused ([§FS-config-migrate.3.1.7](FS-config-migrate.md#317-settings-this-grund-has-no-v2-spelling-for-yet)) |
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

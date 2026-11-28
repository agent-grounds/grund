# FS-config-adopt-builtins: grund config adopt-builtins reads a version-2 file's suffixes through the built-in languages that claim them, says every reading that changes, and writes only what it planned, only when asked

`grund config adopt-builtins` reads a version-2 `grund.toml` and, without touching anything, shows the file in which each suffix now read through a definition is read through the built-in language that claims it instead. The `v1_*` definitions migration writes are the common case ([§FS-config-v2.schema.sources.migration](FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes)). The command names every reading that changes, every suffix it keeps and why, and every suffix the adopted languages start to read. It also runs `grund check --suggestions` before and after. It writes only with `--write`, only when the candidate reads exactly as it planned, and only by an atomic replace. That is the second command allowed to replace the config ([§REQ-no-data-loss.4](../requirements/REQ-no-data-loss.md#4-two-commands-may-replace-the-config)). It serves [§GOAL-no-silent-breakage](../goals.md#goal-no-silent-breakage-changes-ship-through-a-deprecation-path): a project leaves its `v1_*` readings through a reviewed step that names what changes. The alternative is a hand edit, which can stop reading a file type without a word. Why it is a command of its own, why it keeps a suffix rather than refuse, and why it edits text are [§DF-config-adopt-builtins](../decisions/functional/DF-config-adopt-builtins.md#df-config-adopt-builtins-a-version-2-file-moves-its-definitions-to-the-built-in-languages-through-a-command-of-its-own-gated-on-the-readings-it-planned-and-migration-names-a-built-in-only-where-it-reads-a-suffix-exactly-as-version-1-did).

## terms: Terms

Leans on [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (workspace, member, language), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, suggestion), [§FS-config-v2.terms](FS-config-v2.md#terms-terms) (reader, place) and [§FS-config-migrate.terms](FS-config-migrate.md#terms-terms) (candidate, preview, refusal, attached comment, gate). Defines:

- **Built-in row**: a row of version 2's language epoch ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)). The row that **claims** a suffix is the one whose suffixes list it: `shell` claims `sh`, `bash` and `zsh`, and no row claims `gradle` or `h`.
- **In play**: a suffix the file reads through a definition ([§FS-config-adopt-builtins.2](FS-config-adopt-builtins.md#2-which-suffixes-are-in-play)).
- **Reading**: how a suffix's files are read, compared as the records gate compares it ([§FS-config-migrate.5.1](FS-config-migrate.md#51-the-records-gate)). That covers the class, the comment hosts as a set, the docstring reading, the multi-line string and quote reading, the note-measurement rule and the declaration eligibility.
- **Plan**: each suffix's state, adopted, kept, added or unchanged, with the reading it is expected to have ([§FS-config-adopt-builtins.3](FS-config-adopt-builtins.md#3-the-plan)). The plan's readings are the **expected model**. They are built from the file as read and version 2's epoch alone, never from the candidate.

## 1. The command

```text
grund config adopt-builtins [PATH] [--write] [--drop-comments] [--path-base project|invocation]
```

### 1.1 Which file

`PATH` works as it does for `config migrate` ([§FS-config-migrate.1.1](FS-config-migrate.md#11-which-file)). Discovery walks up from it, `.` when it is omitted, and the command adopts in the config it finds: the root `grund.toml`, or the deprecated `.agents/grund.toml`. A path inside a workspace member adopts in that member's config ([§FS-config-adopt-builtins.11](FS-config-adopt-builtins.md#11-workspaces)). Exactly one file is adopted in per run, and it stays at its own version.

### 1.2 The flags

- `--write` replaces the config with the candidate, on the terms of [§FS-config-adopt-builtins.8](FS-config-adopt-builtins.md#8-the-write).
- `--drop-comments` lets the write drop exactly the listed comments ([§FS-config-adopt-builtins.7](FS-config-adopt-builtins.md#7-comments)). It is accepted without `--write`, so a preview can predict the write.
- `--path-base` is accepted, and spells every path the report prints: the config's and those of the findings it compares ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)).
- `--format` is refused as it is for every `config` command ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)): `` error: unknown flag `--format` ``, exit `2`. `--to` is refused the same way, `` error: unknown flag `--to` ``, because the command never changes the file's version. Both are answered before anything is loaded ([§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read)).

## 2. Which suffixes are in play

A suffix is in play when the file reads it through a definition: its `[schema.sources.extensions]` entry names a language defined under `[schema.sources.definitions]`, whatever that definition is called. A `v1_` name is neither required nor enough. A definition reads only the suffixes an entry maps to it ([§FS-config-v2.schema.sources.definitions](FS-config-v2.md#schemasourcesdefinitions-schemasourcesdefinitionsname-defines-a-language)), so a definition no entry names puts nothing in play and is left as it is.

A suffix the file already maps to a built-in, such as `kt = "java"` or `tsx = "javascript"`, is a choice the file made. It is not in play, its entry stays, and no report line names it. A suffix read through its own epoch row is not in play either.

A file that omits `languages` admits no definition ([§FS-config-v2.schema.sources.languages](FS-config-v2.md#schemasourceslanguages-languages-admits-languages-and-only-it-does)), so nothing is in play and it has nothing to adopt ([§FS-config-adopt-builtins.9](FS-config-adopt-builtins.md#9-a-second-run-is-a-no-op)). The command never writes out the default set to admit a row.

## 3. The plan

For each suffix in play the plan asks the row that claims it, and gives every suffix the file reads, or would read, one of four states and an expected reading.

### 3.1 Adopted

A suffix in play whose claiming row reads its files with the class its definition reads them with is adopted. Its entry goes, and its expected reading is the row's reading of that suffix. That reading may differ from the definition's in any other part, comment hosts or doc profile for one, and changing it is what adoption is for. Each difference is printed ([§FS-config-adopt-builtins.5.3](FS-config-adopt-builtins.md#53-reading-differences)), and none refuses.

### 3.2 Kept

A suffix in play is kept when no row claims it, as with `.gradle` and `.h`, or when its row reads it with another class. A `.json` mapped to a `source` definition is kept, because built-in `json` reads `.json` as `data`. A kept suffix's entry and definition stay byte for byte, and its expected reading is the one it has now. The other suffixes are still adopted: keeping a suffix never refuses.

### 3.3 Added

A suffix of an adopted row is added when no entry of `[schema.sources.extensions]` maps it and the file does not read it now. Admitting the row admits it too ([§FS-config-v2.schema.sources.epoch](FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)): `sh` adopted into `shell` adds `bash` and `zsh`. Its expected reading is the row's.

### 3.4 Unchanged

Every other suffix is unchanged: one not in play, a kept suffix's neighbours under its definition, and one the file does not read and no adopted row claims. Its expected reading is the one it has now, or none.

## 4. The candidate

Adoption edits the file's text, and it edits only three things: the `languages` list, the entries of adopted suffixes, and the definitions no remaining entry names. Every other byte stays. `declarations`, `exclude`, `respect_gitignore`, `fenced`, every other table, and every comment on a line it keeps are written back as they were.

### 4.1 `languages` is renamed in place, never widened

A definition whose every entry is adopted leaves `languages`. The rows its suffixes were adopted into take its place, in the order of each row's first adopted entry. A definition that keeps an entry stays listed, and the rows adopted from it follow it directly. A row already listed, or listed earlier by the same edit, is not listed twice. Nothing else enters the list: never the default set, and never a row no suffix was adopted into. A value on one line is written back as a one-line array, and the key and any comment after the value stay. A value written across lines keeps its lines and the comments on them: each name is rewritten where it stands, and a line goes only when the list drops every name on it and nothing else stands on it, so a comment on that line is listed ([§FS-config-adopt-builtins.7](FS-config-adopt-builtins.md#7-comments)). On the example of [§FS-config-adopt-builtins.5.2](FS-config-adopt-builtins.md#52-stderr-is-the-report), `languages = ["v1_markdown", "v1_c_family", "v1_plain", "v1_shell"]` becomes `languages = ["markdown", "java", "v1_plain", "shell"]`.

### 4.2 Entries and definitions

Each adopted suffix's entry is removed with its line. A `[schema.sources.extensions]` table left with no entry is removed. A definition is removed when adoption removed every entry that named it. A definition with an entry left, or one no entry ever named, stays. A removed table goes with its header, every line up to the next header, and the blank lines between.

## 5. Streams and the report

### 5.1 Stdout is the candidate

A preview whose `--write` would write prints, on stdout, exactly the bytes `--write` would write with the same flags, and nothing else ([§FS-errors.1](FS-errors.md#1-streams)). Every other run leaves stdout empty: a refused preview, any `--write`, a run with nothing to adopt, and a failure. So `grund config adopt-builtins > new.toml` captures only a candidate, and only when it exits `0`.

### 5.2 Stderr is the report

Every other line goes to stderr, in this order. `<config>` is the config's path spelled from the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), `grund.toml` by default. A line is never wrapped.

1. The heading: `<config>: adopting version 2's built-in languages, preview, nothing written`, or `<config>: adopting version 2's built-in languages, --write` with `--write`.
2. One line per adopted suffix, in the order of its entry: `adopts: <suffix>: definition <definition> → built-in <row>`, followed by `, opt-in, listed in languages` when the row is opt-in. Its `reads differently:` lines follow it ([§FS-config-adopt-builtins.5.3](FS-config-adopt-builtins.md#53-reading-differences)).
3. One line per adopted row that adds suffixes, in the candidate's `languages` order: `added: <suffixes>: admitted with <row>`, with the suffixes in the row's order and joined by `, `.
4. One line per kept suffix, in the order of its entry. With no row: ``kept: <suffix>: definition <definition> and <suffix> = "<definition>"; version 2 has no built-in language for `.<suffix>` ``. With another class: `kept: <suffix>: built-in <row> reads it as <class>`.
5. The listed comments, when there are any ([§FS-config-adopt-builtins.7](FS-config-adopt-builtins.md#7-comments)).
6. The gate's lines ([§FS-config-adopt-builtins.6](FS-config-adopt-builtins.md#6-the-gate)).
7. After a write, the lines of [§FS-config-adopt-builtins.8.2](FS-config-adopt-builtins.md#82-what-it-says-after-the-write). A `--write` that refuses ends with `not written: <causes>; <config> is unchanged`, and a preview that refuses ends with `would not write: <causes>`. `<causes>` lists every blocking cause, in this order, separated by `; `:
   - `the candidate does not load (see the error above)`;
   - `the readings differ from the plan (inspect the comparison above)`;
   - `the records differ (inspect the comparison above)`;
   - `1 comment would be lost (pass --drop-comments to drop exactly the comments listed above)`, or `<n> comments would be lost (…)`, counting listed ranges.

A symbolic link is refused before the plan, so its report is the heading, the refusal of [§FS-config-adopt-builtins.8.1](FS-config-adopt-builtins.md#81-bounds) and `not written: the config is a symbolic link; <config> is unchanged`, or `would not write: the config is a symbolic link`.

The example is a project as `grund config migrate --to 2 --write` writes it from a version-1 file. The file maps `md`, `java`, `gradle` and `sh` to four `v1_*` definitions, each with the prefixes `//`, `#`, `*` and `/*`. The tree adds `src/extra.bash` and a `.py` file. The preview prints the candidate and exits `0`:

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

The `.py` file is read in neither file: the list never admits `python`.

### 5.3 Reading differences

An `adopts:` line is followed by one `  reads differently: <what>` line for each part of the reading that differs from the suffix's reading now, in this order, and by none when the two are equal:

1. Hosts the row does not read: ``reads differently: `#` is no longer a comment host``, or ``reads differently: `//`, `/*` and `*` are no longer comment hosts``.
2. Hosts only the row reads: ``reads differently: comments behind `;` and `--` are now read``.
3. The docstring reading: `reads differently: triple-quoted docstrings are now read`, or `… are no longer read`.
4. The multi-line string and quote reading: `reads differently: multi-line strings and quotes are now read as built-in <row> reads them`.
5. The note-measurement rule or the declaration eligibility, one line for either: ``reads differently: doc comments are now recognized by the `<new>` profile, not `<old>` ``, naming each reading's doc profile.

Hosts are named in the order `//`, `/*`, `*`, `#`, `;`, `--`, then any other host in byte order. A list is joined by `, ` with a final ` and `.

## 6. The gate

A write needs every part of the gate to pass in the same run, and a preview reports what they say. None is skipped, and no flag relaxes one.

### 6.1 The candidate loads

The candidate is loaded through the version-2 reader, the path a file on disk goes through, as migration's candidate is ([§FS-config-migrate.5.1](FS-config-migrate.md#51-the-records-gate)). A candidate that does not load is refused with exit `1`, and nothing after it runs. That includes anything this grund refuses in a version-2 file. The report gives the reason as `refused: the candidate does not load: <the error config validate gives for it>`.

### 6.2 The readings are the plan

The candidate's reading of every suffix, over the union of the suffixes the file reads and the ones the candidate reads, must equal the expected reading. A candidate that reads any suffix otherwise is refused with exit `1`, whatever part of the reading differs, class included. So a writer that wrote `java = "shell"` is refused, although the set of suffixes that change looks right. The difference prints as:

```text
differs: the readings adoption planned
- java: <the planned reading>
+ java: <the candidate's reading>
```

A reading prints as the records gate prints it, and a suffix not read prints `none`.

### 6.3 Nothing else moves

Every other record of the candidate must equal the file's, compared as the records gate compares a version-2 file ([§FS-config-migrate.5.1](FS-config-migrate.md#51-the-records-gate)), and the version line must be the file's. A difference is refused with exit `1`, as `differs: the records` and then the section's `- `/`+ ` lines ([§FS-config-migrate.5.4](FS-config-migrate.md#54-the-result-and-a-difference)).

### 6.4 What check reports, before and after

When the candidate reads as planned, `grund check --suggestions` runs twice in the same process, once with the file as read and once with the candidate. Both runs use the same root, members and invocation, as migration's report gate does ([§FS-config-migrate.5.2](FS-config-migrate.md#52-the-report-gate)). A changed finding never refuses: adoption changes readings on purpose, and what that does to this tree is what the comparison shows. The line is `compared: the candidate reads as planned; grund check --suggestions: exit <before> → <after>, no finding changes`. With changes it is `…, 1 finding changes` or `…, <n> findings change`, followed by each changed finding as the line `grund check` prints for it. First come `  - <line>` for each one only the file reports, then `  + <line>` for each one only the candidate reports, each in the order `check` prints them.

After both runs the config's bytes are read again. If they changed since the read that built the plan, the run exits `2` with the message of [§FS-config-adopt-builtins.8.1](FS-config-adopt-builtins.md#81-bounds). A comparison that cannot be computed, such as a run that cannot scan the tree, exits `2` with the run's error. Neither writes.

## 7. Comments

A comment on a line adoption removes cannot be kept, and is listed. That is an attached comment ([§FS-config-migrate.terms](FS-config-migrate.md#terms-terms)) above a removed entry or header, a trailing comment on a removed line, or a comment line inside a removed definition table. A comment on `languages` stays where it is, and so does every comment on a line adoption keeps.

```text
comments on lines adoption removes (--write refuses; --drop-comments drops exactly these):
  grund.toml:21  above [schema.sources.extensions] java
```

Each entry is `  <config>:<first>-<last>  <where>`, or `  <config>:<line>  <where>` for one line, in file order. `<where>` is `above <key or header>`, `after <key>` for a trailing comment, or `inside [schema.sources.definitions.<name>]`. Without `--drop-comments` a write with a listed comment is refused with exit `1`, and a preview exits `1` with empty stdout, both naming the cause. With it, exactly those comments are dropped and nothing else, and the write names them again ([§FS-config-adopt-builtins.8.2](FS-config-adopt-builtins.md#82-what-it-says-after-the-write)).

## 8. The write

### 8.1 Bounds

It writes only with `--write`, and only after the gate passes in that same run with no listed comment left standing without `--drop-comments`. It writes as migration does. It writes a temporary file in the config's directory, keeps the original's permissions, syncs it and renames it over the original ([§FS-config-migrate.6.2](FS-config-migrate.md#62-atomic)). It reads the config's bytes again just before the rename ([§FS-config-migrate.6.3](FS-config-migrate.md#63-a-concurrent-edit)). It refuses a symbolic link and writes nothing but the config ([§FS-config-migrate.6.4](FS-config-migrate.md#64-a-symbolic-link-and-nothing-else-written)). Its messages are its own:

- An I/O failure is exit `2`: `` error: <config>: could not write the adopted config (<reason>); <config> is unchanged, run grund config adopt-builtins again ``.
- A config whose bytes changed after the plan was built is exit `2`. The bytes are checked after the comparison and again before the rename: `` error: <config> changed while its built-in languages were being adopted; nothing written, run grund config adopt-builtins again ``.
- A config that is a symbolic link is refused with exit `1` and empty stdout:

```text
refused: grund.toml is a symbolic link, and replacing it would replace the link with a regular file
  repair: replace the link with the file it points to, or adopt the built-in languages in that file where it is the config
```

### 8.2 What it says after the write

A write prints the complete report of [§FS-config-adopt-builtins.5.2](FS-config-adopt-builtins.md#52-stderr-is-the-report), then:

```text
wrote grund.toml: adopted built-in markdown, java and shell; kept v1_plain for gradle
to get the previous file back: it is in version control, e.g. git restore grund.toml
to see this candidate again: restore it and run grund config adopt-builtins
```

The `wrote` line names the adopted rows in the candidate's `languages` order. It then adds `; kept <definition> for <suffixes>` for each definition a kept suffix still maps to, in the same order. When `--drop-comments` dropped any comment, `dropped the comments at <config>:<range>, … and <config>:<range> (lines before adoption)` follows it, naming every range in file order.

## 9. A second run is a no-op

When no suffix is adopted, because nothing is in play or every suffix in play is kept, the run prints its `kept:` lines, then `<config>: nothing to adopt; the file is unchanged`. It exits `0` with empty stdout, with or without `--write`, and leaves the file alone, its modification time included. No heading is printed and no comparison runs, because nothing would change. On that example after the write, the second run prints:

```text
kept: gradle: definition v1_plain and gradle = "v1_plain"; version 2 has no built-in language for `.gradle`
grund.toml: nothing to adopt; the file is unchanged
```

## 10. Exits

| Exit | When |
|---|---|
| `0` | The preview's `--write` would write; or the write was made; or there is nothing to adopt. A `kept:` suffix never refuses. |
| `1` | Refused: the candidate does not load; a suffix's reading differs from the plan, a reclassified suffix included; a record outside `languages`, the entries and the definitions differs, or the version line does; the config is a symbolic link; or a comment on a removed line would be lost without `--drop-comments`. A preview with the same flags exits `1` for the same cause. |
| `2` | The command could not run: a usage error or an unknown flag; no config; a config that does not load; a version this grund does not read; a version-1 file, with `` error: <config> is version 1; migrate it first with grund config migrate --to 2 --write ``; a comparison that cannot be computed; a config that changed while the checks ran or before the rename; or an I/O failure during the write. The file is unchanged in every case. |

Exit `1` means grund judged the file and the report says what to change; exit `2` means it never reached a verdict ([§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed)).

## 11. Workspaces

At a workspace root the command adopts in that one file, and the checks of [§FS-config-adopt-builtins.6.4](FS-config-adopt-builtins.md#64-what-check-reports-before-and-after) run on what `grund check` at that root covers, members included. A member's config is adopted in only by a run whose path is inside the member. A run at the root names no member and touches none.

# DF-config-adopt-builtins: a version-2 file moves its definitions to the built-in languages through a command of its own, gated on the readings it planned, and migration names a built-in only where it reads a suffix exactly as version 1 did

**Date:** 2026-11-17

## status: Accepted

## 1. Context

`grund config migrate --to 2` gives every version-1 reading a `v1_<profile>` definition, which keeps a project's meaning ([§FS-config-v2.schema.sources.migration](../../functional-spec/FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes)). A project that then wants version 2's built-in languages had only a hand edit. The one agent-grounds/grund#1384 reports deleted the definitions and their mappings, so `.gradle` silently stopped being read, and `grund check` printed `success` over a dangling citation. The owner ruled out a flag on `config migrate` on 2026-11-15, and approved this plan on 2026-11-17 with rulings V-D02 and V-D04.

## 2. Decision

### 2.1 A command of its own, not the flag migration ruled out

[§DF-config-migrate.2.1](DF-config-migrate.md#21-a-refusal-names-a-v1-repair-not-a-flag) rejected a flag that makes a change of meaning one keystroke away and records nothing. It stands, and this record does not supersede it. `grund config adopt-builtins` is a separate command. It previews by default, prints every reading that changes, and writes only on `--write`, and its write is a reviewed commit ([§FS-config-adopt-builtins](../../functional-spec/FS-config-adopt-builtins.md#fs-config-adopt-builtins-grund-config-adopt-builtins-reads-a-version-2-files-suffixes-through-the-built-in-languages-that-claim-them-says-every-reading-that-changes-and-writes-only-what-it-planned-only-when-asked)). That is the visible, recorded change [§DF-config-migrate.2.1](DF-config-migrate.md#21-a-refusal-names-a-v1-repair-not-a-flag) asked for, not a flag. Migration keeps its promise that what it writes means what the old file meant. Adoption is the step that changes a meaning, under a gate that names each change. The name says what it does: it adopts the built-in languages and nothing else.

### 2.2 The gate is the plan, not the candidate

The expected model is built from the file as read and version 2's epoch alone, before any candidate exists, and the candidate must read exactly as it says ([§FS-config-adopt-builtins.6.2](../../functional-spec/FS-config-adopt-builtins.md#62-the-readings-are-the-plan)). A gate that compared the candidate with itself, or only counted which suffixes changed, would pass a writer bug that maps `java` to `shell`. `grund check --suggestions` before and after is printed, never a refusal ([§FS-config-adopt-builtins.6.4](../../functional-spec/FS-config-adopt-builtins.md#64-what-check-reports-before-and-after)), because adoption changes readings on purpose. What a changed reading does to the tree is the report's to show and the user's to judge.

### 2.3 Keep a suffix rather than refuse the file

A suffix whose built-in row would read it with another class, or that no row claims, is kept with a `kept:` line, and the rest are adopted ([§FS-config-adopt-builtins.3.2](../../functional-spec/FS-config-adopt-builtins.md#32-kept)), as ruling V-D02 says. Refusing the whole write over one suffix would send the user back to the hand edit this command exists to replace. Class preservation is a rule of how the plan is built, so a candidate that reclassifies a suffix still fails the gate with exit `1`.

### 2.4 Migration's exit codes

Exit `1` means grund judged the file and the report says what to change, and exit `2` means it never reached a verdict ([§FS-cli.5](../../functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed)). The write bounds carried over from migration keep the exit codes they have there, as ruling V-D04 says. A symbolic link and a comment lost without `--drop-comments` are `1`. A concurrent edit, a change while the checks run, and an I/O failure during the write are `2`, and the message says to run again ([§FS-config-adopt-builtins.10](../../functional-spec/FS-config-adopt-builtins.md#10-exits)).

### 2.5 A text edit

[§DF-config-migrate.2.6](DF-config-migrate.md#26-no-text-rewrite) rejects a text rewrite because version 1 to version 2 regroups every key. A version-2 to version-2 run regroups nothing. It touches three keys of one table and the tables they name, so an edit keeps every other byte and every comment on a line it keeps ([§FS-config-adopt-builtins.4](../../functional-spec/FS-config-adopt-builtins.md#4-the-candidate)). Rendering the whole file through `config show`'s renderer would list every comment in the file as lost.

### 2.6 An explicit list is never widened

A `languages` list replaces the default set whole ([§FS-config-v2.schema.sources.languages](../../functional-spec/FS-config-v2.md#schemasourceslanguages-languages-admits-languages-and-only-it-does)). The command renames a definition's entry to the rows adopted from it, and adds nothing else ([§FS-config-adopt-builtins.4.1](../../functional-spec/FS-config-adopt-builtins.md#41-languages-is-renamed-in-place-never-widened)). Taking the default set would start reading every file type the list left out, a second change of meaning that nobody asked for. A file that omits `languages` admits no definition, so it has nothing to adopt ([§FS-config-adopt-builtins.2](../../functional-spec/FS-config-adopt-builtins.md#2-which-suffixes-are-in-play)).

### 2.7 Migration names a built-in where it reads a suffix exactly as version 1 did

`config migrate --to 2` names a built-in row in `languages`, with no mapping and no definition, where every suffix the row claims is read by version 1 and each reads the same as the row reads it ([§FS-config-v2.schema.sources.migration](../../functional-spec/FS-config-v2.md#schemasourcesmigration-what-a-v1-source-configuration-becomes)). A version-1 file with the default prefixes gets `markdown`, not `v1_markdown`. This changes the bytes migration writes for the same version-1 file, and nothing else. The records gate still holds every write to the old file's meaning ([§FS-config-migrate.5.1](../../functional-spec/FS-config-migrate.md#51-the-records-gate)), so no verdict and no finding text moves. That keeps it inside [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered), which covers verdicts and finding text, not the bytes of a preview. A row is named only when it claims nothing version 1 does not read: built-in `shell` would also start reading `.bash` and `.zsh`, so a lone `.sh` stays on `v1_shell`.

## 3. Consequences

A version-2 project leaves its `v1_*` readings in one previewed step that names each reading that changes and each suffix it keeps. Shell and Gradle needing no list or mapping wait for a version 3, tracked separately. No verdict of an existing command moves, so this record carries no release note. Migration's preview bytes change where a built-in is equal, which a script that greps them for `v1_markdown` would see.

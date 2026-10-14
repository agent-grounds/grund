# DF-config-migrate: migration refuses what version 2 cannot spell, names a version-1 repair, and writes only past two gates

**Status:** Accepted
**Date:** 2026-10-14

## 1. Context

Version 2 regroups the configuration by concern and fixes its defaults at the version ([§FS-config-v2](../../functional-spec/FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults)), and a v1 file keeps its meaning forever ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)). Moving from one to the other by hand leaves nothing to say whether the hand-written file means what the old one did. agent-grounds/grund#461 asked for `grund config migrate`, with explicit losses and a guarded write. The owner ruled on 2026-10-14 that `--write` is the one bounded exception to "the config is never overwritten" ([§REQ-no-data-loss.4](../../requirements/REQ-no-data-loss.md#4-one-migration-may-replace-the-config)) and that comments carry or are listed, with `--drop-comments` the only way one is lost. The proposal that settles the rest was approved on #461 the same day (comment 6178962696). The command is [§FS-config-migrate](../../functional-spec/FS-config-migrate.md#fs-config-migrate-grund-config-migrate---to-2-shows-the-version-2-file-that-means-what-a-version-1-config-means-and-writes-it-only-when-asked-and-only-when-it-does).

## 2. Decision

### 2.1 A refusal names a v1 repair, not a flag

The issue asks that "policy-changing repairs must be separately visible and authorized". One `--accept-…` flag per meaning that cannot be carried would make a change of meaning one keystroke away, and nothing would record it. A v1 edit the user makes, checks with `grund check` under version 1, and commits is visible in review and in history, and its effect is shown before any v2 file exists ([§FS-config-migrate.3.1](../../functional-spec/FS-config-migrate.md#31-a-refusal-names-a-v1-repair)). So migration itself never changes a meaning: what it writes means what the committed v1 file means.

### 2.2 Two gates, records first

[§FS-config-v2.mapping](../../functional-spec/FS-config-v2.md#mapping-what-the-records-keep-apart) says a report that matches on one tree is no proof of equivalence. The records are the proof, so the first gate compares them, independent of the tree. The report on this tree is the second gate, because the records comparison ignores stated differences and a mistake in what it ignores should still be caught where the user would see it ([§FS-config-migrate.5](../../functional-spec/FS-config-migrate.md#5-equivalence-two-gates)). Comparing reports alone was rejected for the reason the mapping gives.

### 2.3 `--to` is required

A bare `migrate` that defaulted to version 2 would change meaning on the day a version 3 exists, and a script written today would follow it there ([§FS-config-migrate.1.2](../../functional-spec/FS-config-migrate.md#12---to-is-required)).

### 2.4 No backup file

A `grund.toml.v1` would be a file grund then owns, and [§REQ-no-data-loss.4](../../requirements/REQ-no-data-loss.md#4-one-migration-may-replace-the-config) says the migration writes no other file. Version control already has the old version, and the command says how to get it back ([§FS-config-migrate.6.5](../../functional-spec/FS-config-migrate.md#65-what-it-says-after-the-write)).

### 2.5 No `--recursive`

Several files cannot be replaced atomically together, so one run migrates one file and names the members still on version 1 ([§FS-config-migrate.8](../../functional-spec/FS-config-migrate.md#8-workspaces)).

### 2.6 No text rewrite

Rewriting the v1 text key by key cannot produce version 2, which regroups the keys by concern, and comments would move by accident rather than by rule. Migration translates the records and hands them to the one renderer of v2 spelling that `config show` uses, so the two cannot disagree on it ([§FS-config.4.2](../../functional-spec/FS-config.md#42-grund-config-show-path)).

## 3. Consequences

Most files `grund init` wrote are refused at first, for `named_sections = false` and for missing default `include` entries; each is a one-line v1 edit whose effect `grund check` shows. No verdict of an existing command moves, so this record carries no release note.

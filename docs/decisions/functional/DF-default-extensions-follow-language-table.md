# DF-default-extensions-follow-language-table: the `[scan] extensions` default reads every language grund classifies

**Date:** 2026-10-13

## status: Accepted

## 1. Context

Two lists answered one question, "which extensions does grund know?". [§FS-inline-citation-style.1.1.2](../../functional-spec/FS-inline-citation-style.md#112-languages) gives a doc-comment recognizer to C and C++ headers and the other C++ spellings (`h cc cxx hpp hh hxx`), Objective-C (`m mm`), Kotlin script (`kts`), the JavaScript module and JSX spellings (`jsx mjs cjs`), Dart (`dart`), R (`r R`) and shell (`sh bash zsh`). The `[scan] extensions` default of [§FS-config.3.5](../../functional-spec/FS-config.md#35-scan--what-gets-scanned) named none of those 18, so a project that wrote no `extensions` never opened such a file. A C header with a dangling citation in its API comment passed `check` with `success`, and its declarations were missing from `list`, `refs` and `show` ([issue #705](https://github.com/agent-grounds/grund/issues/705)).

[§REQ-no-missed-citation.3](../../requirements/REQ-no-missed-citation.md#3-proven-per-host-language) already says that a form the default configuration cannot reach is not supported. The table claimed doc forms for 18 extensions the default could not reach.

## 2. Decision

### 2.1 The default is derived from the language table

The `extensions` default is `md` followed by every extension grund's built-in language table names, in table order ([§FS-scan.scope.extensions.default](../../functional-spec/FS-scan.md#scopeextensionsdefault-the-extensions-default-follows-the-language-table)). The table holds each extension [§FS-inline-citation-style.1.1.2](../../functional-spec/FS-inline-citation-style.md#112-languages) gives a recognizer and each language read through a comment prefix alone. One table answers both "how is a doc comment recognized here" and "is this file read by default", so the two cannot drift again: a row added for its recognizer is read by default in the same release ([§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation)).

The 18 extensions are appended after the 26 the default already named, so the old default stays a prefix of the new one. No comment prefix is added: every new language is read through `//`, `#`, `*` or `/*`, which the default already has.

### 2.2 Widen, rather than warn

A warning for an unread file of a classified extension would leave the file unread and say so on every run. The file is one grund knows how to read, and nobody chose to leave it out: the project wrote no `extensions` at all. Reading it is what the zero-configuration default is for ([§GOAL-zero-config](../../goals.md#goal-zero-config-works-on-any-conformant-tree)).

### 2.3 An explicit list is untouched

An explicit `[scan] extensions` still replaces the default whole. Every project `grund init` scaffolded carries one, so those projects see no change. Only a project with no `extensions` key reads more.

## 3. Rejected alternatives

### 3.1 Keep two literal lists and test that they agree

That keeps two answers to one question, held together by a test rather than by construction. The test would be the third place a new language has to be named.

### 3.2 Leave the default and narrow the table's claim

That would make [§REQ-no-missed-citation.3](../../requirements/REQ-no-missed-citation.md#3-proven-per-host-language) true by withdrawing support grund already has, and leave C, Objective-C, Dart, R and shell projects to discover their list by trial.

## 4. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the move in one release, and each of its five conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.3](../../requirements/REQ-no-missed-citation.md#3-proven-per-host-language), which said a form the default cannot reach is not supported from 0.10.1 on, while the table claimed the 18 extensions' doc forms from 0.12.3 on, so the prohibition already applied when the shorter default shipped. A `success` over a dangling citation in such a file is also the silent skip [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips) forbids.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form to warn about: the affected projects wrote nothing. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has nothing to rewrite.
3. *Named release.* The release note below names the verdict move and who sees it.
4. *Actionable findings.* Every finding that replaces the old `success` is one the file already earns when its extension is listed: an `unknown reference` at its file and line, or a `duplicate declaration` at both sites. Each names its location and the edit to make.
5. *No new licence.* No rule, finding or prohibition is new. The default `include`, `exclude` and `comment_prefixes` do not move. This is the bounded correction route and not a licence for ordinary policy tightening, feature removal, or a newly invented prohibition.

## release-note: Release note

- [§FS-scan.scope.extensions.default](../../functional-spec/FS-scan.md#scopeextensionsdefault-the-extensions-default-follows-the-language-table), [§FS-inline-citation-style.1.1.2](../../functional-spec/FS-inline-citation-style.md#112-languages), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.3](../../requirements/REQ-no-missed-citation.md#3-proven-per-host-language): **with no `[scan] extensions`, grund now reads every language it classifies.** The default gains `h cc cxx hpp hh hxx m mm kts jsx mjs cjs dart r R sh bash zsh` after the 26 it already named, and from now on it is `md` plus every extension grund's language table names. Declarations in those files appear in `list`, `refs` and `show`, and their citations are checked. **Who this breaks:** a project with **no** `extensions` key that keeps a stale or misspelled `§` citation in a comment in one of those files under a scanned root. Its `check` moves from `success` to `unknown reference` at that file and line, and newly read declarations can surface `duplicate declaration` errors. **The fix:** correct the citation at the reported line. **To keep the old scan:** write the old 26-entry list as your `extensions`. A project with an explicit `extensions`, which every `grund init` scaffold has, sees no change. The verdict changes by the correction route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), because the old default broke [§REQ-no-missed-citation.3](../../requirements/REQ-no-missed-citation.md#3-proven-per-host-language), and every finding that replaces it names its location and the fix. Why the default widens rather than warns is [§DF-default-extensions-follow-language-table](DF-default-extensions-follow-language-table.md#df-default-extensions-follow-language-table-the-scan-extensions-default-reads-every-language-grund-classifies). Closes [issue #705](https://github.com/agent-grounds/grund/issues/705).

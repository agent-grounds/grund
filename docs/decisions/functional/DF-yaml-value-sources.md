# DF-yaml-value-sources: a value home reads YAML beside JSON, and a YAML source's findings are warnings for one release

**Date:** 2026-10-27

## status: Accepted

## 1. Context

A value kind's home read JSON alone ([§FS-values.2.2](../../functional-spec/FS-values.md#22-declarations-from-value-sources)). A `.yaml` file beside the JSON was ignored without a word, and a citation of a value it declared was an unknown reference with nothing to say the file was why ([issue #1049](https://github.com/agent-grounds/grund/issues/1049)). JSON has no comments, and teams keep tables of limits, prices and versions in YAML, with comments saying where each number came from; to be held by grund such a table had to be converted, and the converted copy was the checked one while the commented one was the one people edited.

## 2. Decision

### 2.1 YAML is a second format of a value source

A value source is the `.json`, `.yaml` or `.yml` file at a value home that the scan does not claim ([§FS-values.terms](../../functional-spec/FS-values.md#terms-terms), [§FS-values.2.2](../../functional-spec/FS-values.md#22-declarations-from-value-sources)). A YAML source is read through a subset that keeps every component the bytes at its own site, and takes a component's kind from its quotes rather than from YAML's type resolution ([§FS-values.2.2.3](../../functional-spec/FS-values.md#223-how-a-yaml-source-is-read)). Nothing about binding, equality or findings depends on the format ([§FS-values.requirements.origin-blind](../../functional-spec/FS-values.md#requirementsorigin-blind-a-root-is-used-the-same-way-whichever-authority-made-it--realized-directional-for-the-next-authority)). No configuration key is added.

### 2.2 The scan's claim wins

In version 1, a YAML file whose extension the effective `[scan] extensions` lists is scan input and not a value source, so a repository that already scans YAML for the citations in its comments keeps them. In version 2 a mapping wins over the extension fallback: a `.yaml` mapped to a `source` or `document` language is not a value source, and one mapped to a `data` language is read as JSON, as it is today ([§FS-config-v2.schema.sources.values](../../functional-spec/FS-config-v2.md#schemasourcesvalues-a-value-home-reads-a-file-by-its-language)).

### 2.3 One release of warnings

Until grund 0.22.0 every error located at a YAML value source is a `yaml-value-source-deferred` warning, a source the finding keeps from being enrolled declares nothing, and a fetched kind's YAML source is skipped with a warning of its own ([§FS-values.2.2.4](../../functional-spec/FS-values.md#224-yaml-findings-are-warnings-until-grund-0220)). From grund 0.22.0 the findings carry their own codes and severities and the exit classes of [§FS-values.5.3](../../functional-spec/FS-values.md#53-incomplete-input-and-deterministic-output).

## 3. Why the window, and why it is enough

### 3.1 The verdicts that can move

Reading a file that was never read can move a verdict, which [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) covers. Four paths, and how each is held in the release that adds the reader:

- **A YAML file in a value folder that is not a value table** — a stray `ci.yml`, a config — draws findings about its shape. Each is the window warning, so the run's exit stays what it was.
- **A citation that dangles today**, whose ID a clean YAML source now declares, becomes a pass or a mismatch. Its exit was already `1`, and stays `1` or falls to `0`.
- **A `.yaml` `file` home on a value row** loses the `unread-scan-root` warning it drew, because it is now read.
- **A fetched kind's home** keeps its answer: its YAML source is skipped, so `missing-snapshot` or `dangling` is still what a citation of it gets.

So no tree goes from passing to failing in that release, and every finding that will fail one from grund 0.22.0 is named in it, with its fix and the release ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)).

### 3.2 A version-2 config reads files it did not read

[§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) says a binary keeps interpreting every version under the semantics it shipped with. The value pass is not a meaning of the config file: it is shared by both versions, and what it reads is the home a row names, which this change does not move. What widens is the pass, announced by a coded warning and binding only in a named release, which is the precedent [§DF-default-extensions-follow-language-table](DF-default-extensions-follow-language-table.md#df-default-extensions-follow-language-table-the-scan-extensions-default-reads-every-language-grund-classifies) set for a widened read.

### 3.3 The epoch does not move

No language is added, `yaml` stays unadmitted, and the language set an omitted `languages` admits is unchanged ([§FS-config-v2.schema.sources.epoch](../../functional-spec/FS-config-v2.md#schemasourcesepoch-the-v2-language-epoch)). What widens is the extension fallback of a value home, which applies only to a file no language claims and is not a language.

## 4. Rejected alternatives

### 4.1 A built-in `yaml` language

Under the epoch it would be opt-in, so the issue's example would need a full `languages` list before its YAML could declare anything.

### 4.2 A YAML reader inside the `data` class

It would change what an existing `yaml = "json"` mapping reads, which is a v2 config's meaning moving under it.

### 4.3 Errors from the first release

A stray YAML file in a value folder would fail a tree that passed the day before, with no release to move it in.

## 5. What it does not cover

A `§` citation in a YAML value source's comment is read by no pass: it neither resolves nor dangles. That blind spot is named ([§FS-values.2.2](../../functional-spec/FS-values.md#22-declarations-from-value-sources), [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)), and reading those comments is a follow-up. Front matter is not touched ([issue #1048](https://github.com/agent-grounds/grund/issues/1048)).

## release-note: Release note

- [§FS-values.2.2](../../functional-spec/FS-values.md#22-declarations-from-value-sources), [§FS-values.2.2.3](../../functional-spec/FS-values.md#223-how-a-yaml-source-is-read), [§FS-values.2.2.4](../../functional-spec/FS-values.md#224-yaml-findings-are-warnings-until-grund-0220), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a value kind's home now reads YAML beside JSON.** A `.yaml` or `.yml` file in a `form = "value"` / `values = true` home declares values as a `.json` one does, comments allowed, and its IDs appear in `list`, `show`, `refs` and `cover`. **Who sees a change:** a repository with a YAML file in a value home. A file that is not a value table — a stray `ci.yml` — now draws a `yaml-value-source-deferred` warning naming the file and the fix; a citation that dangled because its ID was in a YAML file now resolves; a `.yaml` `file` home on a value row loses its `unread-scan-root` warning. **Until grund 0.22.0** every error at a YAML value source is that warning and the run's exit does not move; from grund 0.22.0 they are errors, and an unparsable YAML source is an incomplete scan at exit `2`, as a JSON one is. **The fix:** correct what the warning names, or move the file out of the value home. **To keep YAML scanned as text** in version 1, list `yaml` or `yml` in `[scan] extensions`; in version 2, map the extension to a source language. A `§` citation in a YAML value source's comment is not checked yet: cite provenance from Markdown. Why: [§DF-yaml-value-sources](DF-yaml-value-sources.md#df-yaml-value-sources-a-value-home-reads-yaml-beside-json-and-a-yaml-sources-findings-are-warnings-for-one-release). Closes [issue #1049](https://github.com/agent-grounds/grund/issues/1049).

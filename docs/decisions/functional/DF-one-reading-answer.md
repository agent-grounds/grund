# DF-one-reading-answer: whether a file is read as Markdown, or has its docstrings read, is the configuration's one answer, never its name, and every home of a row is judged by it

**Date:** 2026-11-12

## status: Accepted

## 1. Context

"Is this file read as Markdown?" already had an answer, the reading the scan takes from the language a name resolves to ([§FS-config-v2.schema.sources.extensions](../../functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)). Several places asked the question again from the file's name: the version-2 `index` check, the index rules in the checker, the index filter for link targets, the first-entry name test on a rule kind's `files`, and the snapshot hint on an absent value home. The specification asked it from the name as well, in the sentences on docstrings ([§FS-fmt.2.3.1.1](../../functional-spec/FS-fmt.md#2311-a-python-docstring-is-scanned-for-its-content)), on qualified citations in Markdown ([§AR-scanner.2.3.2](../../architecture/AR-scanner.md#232-a-qualified-citation-in-source-code)) and on what a source file is ([§FS-check.3.6.1](../../functional-spec/FS-check.md#361-which-files-a-row-governs)). So the load step, the checker and the scan could disagree about one file:

- `index = "index.txt"` under `txt = "markdown"` was refused as not Markdown, though the scan reads that file as Markdown (issue #1184);
- `index.md` under `md = "rust"` was judged by the index rules as Markdown, which drew an error whose repair, `grund fmt --write`, does nothing to a file it reads as Rust (issue #1203);
- a rule kind's `files` entry read as Rust loaded or was refused by the name of the row's first entry, so the order of two entries decided whether the config loaded (issue #1187);
- a value row's second `files` entry that does not exist loaded green, and a second `folders` entry failed the scan with no config location, while the same entry first was refused at its line (issue #1152);
- three spec sentences named `.md` or `.py` where the code follows the reading (issues #1243, #1244, #1245).

The owner ruled on 2026-11-05 that the same behaviour lives in one place, under one name, and every caller uses it.

## 2. Decision

### 2.1 One answer

[§FS-fmt.2.3.1](../../functional-spec/FS-fmt.md#231-string-literal-exclusion-rule) defines the two readings a rule about which files it covers asks: a file read as Markdown, and a file whose docstrings are read. In version 2 each is the language's; in version 1 each is the name's, `.md` and `.py`, which is what version 1 reads as those. A version-2 name no admitted language claims is neither, whatever its suffix. Every sentence that says which files a rule covers names the reading and cites that definition, never a suffix.

### 2.2 A name written in the config is asked the same question

An `index`, a `files` entry and a `folders` entry are judged by the same predicate as a file in the tree ([§AR-config.1.6](../../architecture/AR-config.md#16-a-files-reading-is-asked-through-one-rule)). No load site tests `.md` itself.

### 2.3 Every home of a row is judged

What a form asks of a home is asked of every `files` and `folders` entry, not only the first, and the order of the entries moves no verdict ([§FS-config-v2.schema.sources.rules](../../functional-spec/FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading), [§FS-config.3.4.9](../../functional-spec/FS-config.md#349-values--first-class-value-declarations)). A value row with one file and one folder stays refused, in version 1's words. A later value entry that fails a check the first is refused for, by its shape, by resolving outside the root or by a lookup that fails, was never asked; it now loads with `config-value-home-invalid` and is read as grund 0.22.0 read it, a scan error it drew included, and is refused from 1.0.0 ([§FS-config-v2.checks.config-value-home-invalid](../../functional-spec/FS-config-v2.md#checksconfig-value-home-invalid-a-value-rows-later-home-is-the-wrong-shape-outside-the-root-or-not-looked-up)), so judging it moves no exit.

### 2.4 The verdicts that move

Taking the name test away moves four verdicts, each from `2` to whatever the run gives, and none from a pass to a failure:

1. **A named index another name the configuration reads as Markdown** (`index = "index.txt"` under `txt = "markdown"`) loads, and its index rules judge it.
2. **A named index not read as Markdown that is not named `.md`** (`index = "x.rs"`) was refused with `` `[schema.kinds.FS] index` must name a Markdown file ``. Telling it apart from `index.md` under `md = "rust"`, which loaded, needs the name test [§DF-one-reading-answer.2.1](DF-one-reading-answer.md#21-one-answer) removes, and no new load refusal ships before grund 1.0.0 ([§DF-markdown-homes-warn-before-1-0](DF-markdown-homes-warn-before-1-0.md#df-markdown-homes-warn-before-1-0-an-md-mapped-away-from-markdown-a-named-index-or-rule-home-not-read-as-markdown-and-an-absent-later-value-home-warn-from-0230-and-are-refused-from-100)). So both load with `config-index-not-markdown`, whose message opens with those same words, and both are refused from 1.0.0 ([§FS-config-v2.checks.config-index-not-markdown](../../functional-spec/FS-config-v2.md#checksconfig-index-not-markdown-a-named-index-is-not-read-as-markdown)). For one release window a config that was refused loads, with a warning that names the release that refuses it again.
3. **A rule kind's `files` entry read by a `source` language, where the row's first entry is not named `.md`** (`rules/rules.rs` alone, or `rules/more.rs` before `rules/rules.md`) was refused, and the same entries in the other order loaded. Both orders now load with `config-rule-file-not-markdown`, refused from 1.0.0 ([§FS-config-v2.checks.config-rule-file-not-markdown](../../functional-spec/FS-config-v2.md#checksconfig-rule-file-not-markdown-a-rule-kinds-files-entry-is-read-as-source)). This is the move the author approved for that row.
4. **A value row's later `folders` entry that does not exist** failed the scan, exit `2`, with `read value home: entity not found` and no config location. A row's homes get one verdict whichever key lists them, and a later absent `files` entry already loaded, so the walk no longer visits an absent entry: both shapes load with `config-value-home-absent` at the key that lists the entry, and the scan error goes. Both are refused from 1.0.0, as an absent first entry already is ([§FS-config-v2.checks.config-value-home-absent](../../functional-spec/FS-config-v2.md#checksconfig-value-home-absent-a-value-rows-later-home-does-not-exist)).

### 2.5 Version 1 does not move

Version 1's readings are its names, so routing its questions through the one predicate changes no answer: its index must still be named `.md`, its rule homes still pass the name test, and its value row still has exactly one home ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)).

## 3. Rejected alternatives

- **Keep `index = "x.rs"` refused while `index.md` under `md = "rust"` warns.** The two differ only in their names, so keeping the refusal keeps the name test this record removes.
- **Keep the scan's error beside `config-value-home-absent` for a `folders` entry.** It would give one fact two findings, one of them unlocated, and give a row's homes two verdicts by the key that lists them.
- **Leave the spec sentences naming `.md` and `.py` and treat them as shorthand.** The reproducers for issues #1243 to #1245 read those sentences and found them false of the shipped behaviour; a sentence that is false of a mapped project is not shorthand.

## release-note: Release note

- [§DF-one-reading-answer](DF-one-reading-answer.md#df-one-reading-answer-whether-a-file-is-read-as-markdown-or-has-its-docstrings-read-is-the-configurations-one-answer-never-its-name-and-every-home-of-a-row-is-judged-by-it), [§FS-fmt.2.3.1](../../functional-spec/FS-fmt.md#231-string-literal-exclusion-rule), [§FS-config.3.4.2.3](../../functional-spec/FS-config.md#3423-a-named-index-is-a-file-inside-folder-that-is-read-as-markdown), [§FS-config-v2.schema.sources.rules](../../functional-spec/FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading), [§FS-config.3.4.9](../../functional-spec/FS-config.md#349-values--first-class-value-declarations): **whether a file is read as Markdown is how the configuration reads it, never its name, and every home of a row is judged that way.** **What loads that was refused:** a version-2 `index` the configuration reads as Markdown under another name, such as `index = "index.txt"` under `txt = "markdown"`, now loads and its index rules apply, so the run reports what they find. A version-2 `index` that is not read as Markdown and not named `.md`, such as `index = "x.rs"`, was refused with `` `[schema.kinds.FS] index` must name a Markdown file `` and now loads with `config-index-not-markdown`, whose message opens with those words and names grund 1.0.0, the release that refuses it again. A version-2 rule kind whose `files` entry is read by a `source` language, listed before the row's Markdown entry or alone, such as `files = ["rules/rules.rs"]`, was refused and now loads with `config-rule-file-not-markdown`, naming 1.0.0. **What no longer fails the scan:** a version-2 `form = "value"` row whose later `folders` entry does not exist exited `2` with `read value home: entity not found`; it now loads with `config-value-home-absent` at the `folders` key, naming 1.0.0, and the run's exit is what the rest of it finds. **What now warns:** a later value entry in the wrong shape, outside the root, or that cannot be looked up loads with `config-value-home-invalid`, naming 1.0.0, and is read as before, so its exit does not move. No run that passed fails because of this record, and version 1 is unchanged. Part of [issue #1206](https://github.com/agent-grounds/grund/issues/1206).

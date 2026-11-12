# DF-markdown-homes-warn-before-1-0: an `md` mapped away from Markdown, a named index or rule home not read as Markdown, and an absent later value home warn from 0.23.0 and are refused from 1.0.0

**Date:** 2026-11-12
**Supersedes:** [§DF-v2-rule-home-by-language.2.2](DF-v2-rule-home-by-language.md#22-a-folder-home-is-left-as-it-is), that a rule kind's `folder` home is accepted on its presence and a folder the scan reads nothing from may go unnamed, and [§DF-v2-rule-home-by-language.2.3](DF-v2-rule-home-by-language.md#23-a-home-a-source-language-reads-keeps-the-answer-it-had), that a rule kind's `files` entry a `source` language reads keeps the name test on the row's first entry.

## status: Accepted

## 1. Context

The owner ruled on 2026-11-05 that `.md` is always Markdown, and that whether a file is read as Markdown has one answer, the configuration's reading ([§DF-one-reading-answer](DF-one-reading-answer.md#df-one-reading-answer-whether-a-file-is-read-as-markdown-or-has-its-docstrings-read-is-the-configurations-one-answer-never-its-name-and-every-home-of-a-row-is-judged-by-it)). Under that answer five configurations name a home that the configuration does not read as Markdown, or does not have, where the row needs one it does:

- an `md` entry of `[schema.sources.extensions]` that maps `.md` to a `source` or `data` language, so every `.md` in the project is read as something other than Markdown;
- a version-2 `index` that names a file not read as Markdown, whose index rules judge a file `fmt` cannot repair (issue #1203);
- a rule kind's `files` entry read by a `source` language, where a Markdown rule declares nothing and drops on a green run (issue #1187);
- a rule kind's `folders` entry in a project where no suffix resolves to an admitted `document` language, so nothing the folder holds is read as Markdown, an empty folder and one holding only `.gitkeep` included;
- a value row's later home that does not exist (issue #1152).

Each loaded through grund 0.22.x, most of them in silence. [§DF-v2-rule-home-by-language.2.2](DF-v2-rule-home-by-language.md#22-a-folder-home-is-left-as-it-is) and .2.3 left the third and fourth standing on purpose, naming the gap and leaving it open. Refusing all five at once would take projects that load today to exit `2` in the release that tells them, and the approved plan for issue #1206 ships no new load refusal in 0.23.0.

## 2. Decision

### 2.1 Five load warnings, each naming 1.0.0

Each of the five is a version-2 load warning from grund 0.23.0, at the config line that has to change, in the deprecation path's shape ([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)): the refusal it becomes, what is read until then, the remedy, and `this becomes an error in grund 1.0.0`. They are `config-md-not-markdown`, `config-index-not-markdown`, `config-rule-file-not-markdown`, `config-rule-folder-not-markdown` and `config-value-home-absent` ([§FS-config-v2.checks](../../functional-spec/FS-config-v2.md#checks-checks)). From grund 1.0.0 each is refused at load in its words up to the remedy, and its code is spent, as `config-fetch-names-nothing` was in 0.23.0 ([§DF-fetch-names-an-executable-lands-in-0-23](DF-fetch-names-an-executable-lands-in-0-23.md#df-fetch-names-an-executable-lands-in-0-23-a-version-2-fetch-key-that-names-no-executable-is-refused-at-load-from-0230)). The release guard holds 1.0.0 while any message promises a change in it ([§FS-distribution.4.2.2](../../functional-spec/FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)), so the refusals land before the 1.0 freeze or 1.0.0 does not ship.

`config-index-not-markdown` is for a named `index` only. A row that writes no `index` named nothing, so where the default `README.md` is not read as Markdown its folders are judged as under `index = false` with no warning, and 1.0.0 refuses nothing about it.

`config-rule-folder-not-markdown` is a load warning rather than `unread-scan-root`, because the configuration alone decides it and `unread-scan-root` names no release ([§FS-scan.checks.unread-scan-root.channel](../../functional-spec/FS-scan.md#checksunread-scan-rootchannel-a-located-finding-on-stdout-and-the-exit-code-does-not-move)); a folder the walk reads nothing from could not otherwise be told before 1.0.0 refused it.

### 2.2 A file in a rule folder read as source stays a warning

A file the walk reads from a rule kind's folder with a `source` language draws `rule-folder-file-not-markdown` from `check`, at that file, and stays a warning past 1.0.0. Only the walk sees what a folder holds, so the configuration cannot be refused for it, and a rule in a block the language makes eligible to declare is in force there. Refusing an `md` mapped away from Markdown in 1.0.0 closes the route by which a Markdown rules file is read as source, which is what issue #1200 reported. It is a code of its own because one code cannot be both spent at 1.0.0 and live after it.

### 2.3 What this supersedes

[§DF-v2-rule-home-by-language.2.2](DF-v2-rule-home-by-language.md#22-a-folder-home-is-left-as-it-is) accepted a rule kind's `folder` home on its presence and left unnamed a folder the scan reads nothing from; [§DF-markdown-homes-warn-before-1-0.2.1](DF-markdown-homes-warn-before-1-0.md#21-five-load-warnings-each-naming-100) and [§DF-markdown-homes-warn-before-1-0.2.2](DF-markdown-homes-warn-before-1-0.md#22-a-file-in-a-rule-folder-read-as-source-stays-a-warning) above name both cases. [§DF-v2-rule-home-by-language.2.3](DF-v2-rule-home-by-language.md#23-a-home-a-source-language-reads-keeps-the-answer-it-had) kept the name test for a `files` entry a `source` language reads; [§DF-one-reading-answer](DF-one-reading-answer.md#df-one-reading-answer-whether-a-file-is-read-as-markdown-or-has-its-docstrings-read-is-the-configurations-one-answer-never-its-name-and-every-home-of-a-row-is-judged-by-it) removes it, and [§DF-markdown-homes-warn-before-1-0.2.1](DF-markdown-homes-warn-before-1-0.md#21-five-load-warnings-each-naming-100) warns about every such entry wherever it sits in the list. [§DF-v2-rule-home-by-language.2.1](DF-v2-rule-home-by-language.md#21-the-check-asks-the-readers-question), which refuses an entry read by a `data` language or by no admitted language, stands.

## 3. Consequences

| Home, and how it is read | Through 0.22.x | 0.23.0 | 1.0.0 |
|---|---|---|---|
| `md` mapped to a language not of class `document` | loads, silent | `config-md-not-markdown` | refused at load |
| `md` mapped to `markdown`, `v1_markdown` or a `document` definition | loads | unchanged | unchanged |
| a named `index` not read as Markdown, named `.md` (#1203) | loads, its rules judge it | `config-index-not-markdown`, judged as `index = false` | refused at load |
| a named `index` not read as Markdown, not named `.md` (`index = "x.rs"`) | refused | `config-index-not-markdown`, its message opening with the refusal's words | refused at load |
| the default `README.md`, not read as Markdown | its rules judge it | judged as `index = false`, silent | unchanged |
| a rule `files` entry read as source, the row's first entry named `.md` (#1187) | loads, silent | `config-rule-file-not-markdown` | refused at load |
| the same, the first entry not named `.md` | refused | `config-rule-file-not-markdown`, loads | refused at load |
| a rule `folders` entry where no suffix resolves to an admitted `document` language | loads, silent | `config-rule-folder-not-markdown` | refused at load |
| a file read as source in any other rule `folders` entry (#1200) | silent | `rule-folder-file-not-markdown` | unchanged |
| a value row's later entry absent (#1152) | `files`: loads green; `folders`: an unlocated scan error, exit `2` | `config-value-home-absent`, the entry not read | refused at load |
| a value row with one file and one folder | refused | unchanged | unchanged |

Every warning leaves the exit code alone. No config that loaded through 0.22.x is refused in 0.23.0, and none of this applies to version 1.

**Why the deprecation path, and why these releases.** [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) is the default route, and each row above has an old form and a new one to move to: map `md` back, name a file read as Markdown, create the home or drop the entry. The window is the rest of 0.x rather than one minor release, so that a project meets each refusal only after at least one release of its warning, and the refusals are one follow-up change that must land before the 1.0 freeze run, because the release guard holds 1.0.0 until it does.

## release-note: Release note

- [§DF-markdown-homes-warn-before-1-0](DF-markdown-homes-warn-before-1-0.md#df-markdown-homes-warn-before-1-0-an-md-mapped-away-from-markdown-a-named-index-or-rule-home-not-read-as-markdown-and-an-absent-later-value-home-warn-from-0230-and-are-refused-from-100), [§FS-config-v2.checks](../../functional-spec/FS-config-v2.md#checks-checks), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **five version-2 configurations that name a home not read as Markdown, or a home that does not exist, now warn, and each warning names grund 1.0.0, the release that refuses it.** `config-md-not-markdown` warns at an `md` entry of `[schema.sources.extensions]` that names a language not of class `document`, such as `md = "rust"` or `md = "json"`; `md = "v1_markdown"`, which `config migrate` writes, and `md = "markdown"` stay silent. `config-index-not-markdown` warns at a named `index` the configuration does not read as Markdown, and its folders are judged as under `index = false`. `config-rule-file-not-markdown` warns at a rule kind's `files` entry a `source` language reads, in any position. `config-rule-folder-not-markdown` warns at a rule kind's `folders` key where no admitted language reads Markdown, even for an empty folder. `config-value-home-absent` warns at a value row's key whose later entry does not exist. A file a `source` language reads inside a rule kind's folder draws `rule-folder-file-not-markdown` from `check`, which stays a warning. **Who this affects:** a version-2 `grund.toml` that draws any of the five load warnings: it loads exactly as before with the same exit code, and `--ignore <code>` drops the line; make the change the warning names before 1.0.0. A project whose default `README.md` index is not read as Markdown, as under `languages = ["rust"]`, is no longer told its declarations are missing from it. Version 1 is unchanged. Part of [issue #1206](https://github.com/agent-grounds/grund/issues/1206).

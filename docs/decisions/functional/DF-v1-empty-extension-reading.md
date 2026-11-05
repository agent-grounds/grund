# DF-v1-empty-extension-reading: version 1 keeps reading a file with no extension through an empty string in `[scan] extensions`, and the specification is corrected to say so

**Date:** 2026-11-05

## status: Accepted

## 1. Context

A version 1 `[scan] extensions` may hold the empty string. The extension test read a missing, empty or non-UTF-8 extension as the empty string and compared that with each entry, so `extensions = ["md", ""]` reads `NOTES`, `FACTS.` and a name whose extension is not UTF-8. [Issue #1237](https://github.com/agent-grounds/grund/issues/1237) reports that against [§FS-scan.scope.extensions.root](../../functional-spec/FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so), which said a file with no extension "has none the list can hold". Nothing documented the entry, and nothing in the repository used it.

The two disagree, and the code is the older of them: the reading is in grund 0.1.0, and the sentence first shipped in 0.18.0. The same release wrote [§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning), which pulls the other way: a binary keeps interpreting every version under the semantics that version shipped with, and the v1 reader keeps v1's validation, so that "v2's stricter rules never refuse a file v1 accepts". The entry is not inert either. On a tree that holds it, a `FACTS` home cited from `docs/a.md` is green because of the read, and `docs/NOTES` fails on its dangling citation because of the read.

## 2. Decision

### 2.1 The read stays, and the specification says so

Version 1 keeps this reading for good ([§FS-scan.scope.extensions.empty](../../functional-spec/FS-scan.md#scopeextensionsempty-in-version-1-an-empty-string-in-the-list-reads-a-file-with-no-extension)), and [§FS-scan.scope.extensions.root](../../functional-spec/FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so) is corrected to say so. No result moves on upgrade, so [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) is served and no route is owed. [§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) and [§FS-config.5.3](../../functional-spec/FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired) do not change: this rests on them. It is the ruling the author gave for the same shape [on #1088](https://github.com/agent-grounds/grund/issues/1088#issuecomment-6484930620): "Version 1 keeps its reading for good; it is not deferred." [§FS-config.requirements.5](../../functional-spec/FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly--realized-one-case-deferred) says of this entry what it says of a string that closes early: version 1 keeps the reading and meets the rule at `config migrate`.

### 2.2 No warning is added

A new warning is itself a verdict change, and it is not needed to reach the person who holds the entry. The version 1 caution of [§FS-config.5.3](../../functional-spec/FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired) already sends every version 1 file to `config migrate`, and that command names the entry.

### 2.3 One test answers, wherever the list is asked

What is read does not change, so the change is who answers. The rule is one fact and gets one implementation ([§AR-system.principle](../../architecture/README.md#principle-one-fact-one-point-one-implementation)): the walk, a file root, a buffer, a stub's target and a link the walker cannot resolve already ask one test, and migration's walk kept a second copy that disagreed with it, leaving out `NOTES` though the walk reads it. It now asks the same test.

### 2.4 Migration says what the entry reads and what removing it costs

A person who follows the repair loses those reads, and today's lines do not say so. [§FS-config-migrate.3.1.9](../../functional-spec/FS-config-migrate.md#319-an-empty-string-in-scan-extensions) lengthens them for this entry alone: the added clauses are true of no other entry a version 2 mapping cannot hold, so `tar.gz` and `*.md` keep their lines.

### 2.5 What does change, in a run that already refuses

`config explain scan.extensions` gains the sentence that says what the entry reads. Beyond that wording, nothing here changes a run that is not already refused. In a `config migrate` run that is:

- The refusal of the entry prints the two longer lines. Each keeps its old text as a verbatim prefix, which [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) allows in one release: only a script matching a whole line sees the difference.
- Under the entry, the `would change owner` and `would stop reading` details of [§FS-config-migrate.3.1.5](../../functional-spec/FS-config-migrate.md#315-nested-homes) and [§FS-config-migrate.3.1.4](../../functional-spec/FS-config-migrate.md#314-a-path-shaped-scan-exclude-entry) come to name each file with no extension that migration's own walk finds, and the file count in the `would stop reading` refusal's own sentence counts it. That walk asks the scan's extension test and applies neither the hidden rule nor the ignore files, so one among them can be a file the scan does not read, as `.gitkeep` is ([§DF-v1-empty-extension-reading.4](DF-v1-empty-extension-reading.md#4-what-this-leaves-as-it-is)).

Neither moves a verdict. The refusal count and the exit stay as they were, because both refusals are raised whatever files they find. A tree whose list does not hold the entry gets the same bytes, with one exception: a list naming an extension spelled with U+FFFD, beside a file whose extension is not UTF-8. The old copy matched that file through the replaced character in its printed name, which the scan does not do, so the detail and its count now drop it.

## 3. Alternatives considered

**Make the extension test treat these names as unlisted.** This is the ticket's first repair, and it moves verdicts silently in both directions on files that load today: a `FACTS` home cited from `docs/a.md` goes from green to `unknown reference`, `docs/NOTES` goes from failing to green with nothing said, and the entry stays in the list doing nothing. It contradicts [§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning).

**Refuse the entry at load, with or without a release of warnings.** This is the ticket's second repair, and the cleanest end state: one located error, no inert entry, every surface closed. But it refuses a file version 1 accepts, which [§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) and [§FS-config.5.3](../../functional-spec/FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired) rule out, and the caution of [§FS-config.5.3](../../functional-spec/FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired) names no release in which a version 1 file stops loading. Taking it through the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) means amending both points first. The correction route of <§>REQ-backwards-compatibility.5 is not open: no hard requirement forbids this read, and a specification sentence does not carry that route ([§DF-v2-string-closes-on-its-line.2.2](DF-v2-string-closes-on-its-line.md#22-the-route-is-the-deprecation-window)). [§FS-config.3.5.16](../../functional-spec/FS-config.md#3516-an-exclude-entry-containing--is-refused) is no precedent, because its entry matched nothing on any tree and it predates the sentence in [§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning).

**Warn on the entry with no deadline.** Rejected by [§DF-v1-empty-extension-reading.2.2](DF-v1-empty-extension-reading.md#22-no-warning-is-added).

## 4. What this leaves as it is

A stray entry still reads `LICENSE` and `Makefile` with no warning of its own. Migration does not list those files in the entry's own refusal, because the only walk it has ignores `.gitignore`, so the list would be wrong. `config show` prints the entry as written, because what it prints has to load back the same ([§FS-config.requirements.6](../../functional-spec/FS-config.md#requirements6-the-effective-configuration-is-inspectable--realized-provenance-deferred)), and the empty-scan caution prints the list as written, because correcting that would change bytes and correct no verdict.

The `would change owner` and `would stop reading` details do come from that walk, and they can be wrong in the same way, because it applies neither the hidden rule nor the ignore files. Before this change they could already name a file the scan does not read when the list holds its extension: one that is hidden, in a hidden directory or ignored. Under the entry they can also name such a file with no extension, `.gitkeep` for one. They can also be right: a hidden or ignored directory that is itself a scan root is walked ([§FS-scan.scope.roots](../../functional-spec/FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)), so with `.spec` and `.spec/inner` as homes, `.spec/inner/b.md` is named and read. Leaving out the files the scan does not read changes the `would change owner` detail on trees that hold no entry at all, so it is a separate change with its own release line, and is no part of this one.

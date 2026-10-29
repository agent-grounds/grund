# DF-v2-array-one-trailing-comma: a version-2 array takes one trailing comma, after an element, warned for in 0.21.0 and refused from 0.22.0

**Date:** 2026-10-29

## status: Accepted

## 1. Context

[§FS-config-v2.reader.5](../../functional-spec/FS-config-v2.md#reader5-an-array-may-span-lines) has said since version 2 shipped that an array may end in the trailing comma TOML allows before its `]`. TOML allows one, after an element. The readers before grund 0.21.0 took off the whole run of adjacent commas before the `]` instead, so a version-2 config such as

```toml
[schema.sources]
exclude = ["ab",,]
```

loaded as `exclude = ["ab"]`, and `exclude = [,]` as the empty list, with `check` green ([issue #1087](https://github.com/agent-grounds/grund/issues/1087)). TOML refuses both lines, so every other tool that reads the file as TOML, an editor, a linter or Python's `tomllib`, refuses the file grund calls valid, and a typo goes unnoticed in grund and breaks everywhere else.

Here, unlike [§DF-v2-string-closes-at-its-quote](DF-v2-string-closes-at-its-quote.md#df-v2-string-closes-at-its-quote-a-version-2-string-ends-at-its-first-unescaped-quote-warned-for-in-0210-and-refused-from-0220-and-version-1-keeps-its-reading), the value grund read is the one its author meant: `["ab",,]` is `["ab"]` with a comma doubled. What is wrong is only that grund loads a file nobody else does. Every release with the version-2 reader, 0.18.0 through 0.20.x, loads these files, and [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) allows a release to turn a file that loads into one that is refused only by one of its routes.

## 2. Decision

### 2.1 An array takes one trailing comma, after an element

A comma before a version-2 array's `]` comes off only where the text before it, whitespace and comments skipped, is neither `[` nor `,`. Any other comma has no element before it, and stays for the element reader to refuse as the empty element it is, `expected string` at the key's line ([§FS-config-v2.reader.5.commas](../../functional-spec/FS-config-v2.md#reader5commas-an-array-takes-one-trailing-comma-after-an-element)). That is TOML's rule, and the one the specification already stated; only the code that took the comma off was lax.

Exactly the shapes that loaded move: two or more adjacent commas after the last element, and an array with no element and one comma or one run of commas. A shape with whitespace between two commas, such as `["ab", ,]` or `[ , , ]`, already left an empty element for the element reader, was refused as `expected string`, and is refused as before.

### 2.2 The route is the deprecation window

The route is the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): grund 0.21.0 reads these files as before and warns at the key's line, naming the line of the first comma with no element before it and grund 0.22.0, and grund 0.22.0 refuses them as `expected string` ([§FS-config-v2.reader.5.commas](../../functional-spec/FS-config-v2.md#reader5commas-an-array-takes-one-trailing-comma-after-an-element), [§FS-config-v2.reader.5.commas.refusal](../../functional-spec/FS-config-v2.md#reader5commasrefusal-from-grund-0220-a-comma-with-no-element-before-it-is-expected-string)).

The correction route, section 5 of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly), would refuse them in this release, and it needs a hard requirement, cited by number, that the old reading already broke. None does: the reading broke a specification point, and a specification point is not a hard requirement, which is the finding [§DF-v2-string-closes-on-its-line.2.2](DF-v2-string-closes-on-its-line.md#22-the-route-is-the-deprecation-window) made for the string left open at a line's end. The mechanical migration of section 3 does not apply either: it needs one documented command the tool ships that makes the repair, and no command grund ships rewrites a version-2 `grund.toml`.

### 2.3 One code, with a remedy that names the line to edit

The warning's code is `config-not-toml` ([§FS-config-v2.checks.config-not-toml](../../functional-spec/FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses)), which was named for the class, text TOML refuses that a version-2 reader still loads, so that this construct could join it ([§DF-v2-string-closes-on-its-line.2.4](DF-v2-string-closes-on-its-line.md#24-two-codes-because-a--string-is-toml)). It joins as a third construct with a remedy case of its own, `delete each comma on line {line} that has no element before it`. The line is the one the first such comma stands on, which in an array written across lines is not the key's line, so that the line named is the line the author edits; the warning is still located at the key's line, as every value is. One array earns one warning however many such commas it holds. An array that also has an open line or a string that closes early earns each construct's warning; `config validate` and `config show` print them in the order the reader meets them, and `check` sorts them as it sorts every finding.

The refusal reuses the element reader's `expected string`. A new message would say the same thing about the same place, and the empty element it names is exactly what the comma leaves.

### 2.4 Version 1 is untouched

Version 1's list reader takes no trailing comma at all: `["a",]` is `expected string` there, and so is every shape this decision moves ([§FS-config.3.11](../../functional-spec/FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string)). It keeps that reading for good ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)), and `config migrate` meets none of these shapes, because version 1 refuses them before there is anything to migrate.

### 2.5 The ancestor climb records nothing

The ancestor climb reads `[workspace] members` through the version-2 reader's own join, and has no run to report into. Through 0.21.x it reads what the full load reads, `members = ["a",,]` as the one member `a`, and from 0.22.0 it reads such a value as not a list, as it reads any value the full load refuses ([§FS-workspace.6.1.7.5](../../functional-spec/FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)).

## 3. Alternatives considered

**Refuse in this release.** Rejected on [§DF-v2-array-one-trailing-comma.2.2](DF-v2-array-one-trailing-comma.md#22-the-route-is-the-deprecation-window): it is the correction route with no hard requirement behind it, and it would fail every command on a repository that loads today, with no release to fix it in, for a typo whose value was what its author meant.

**Keep reading every comma off, and change the specification to say so.** Rejected on [§DF-v2-array-one-trailing-comma.2.1](DF-v2-array-one-trailing-comma.md#21-an-array-takes-one-trailing-comma-after-an-element): a version-2 `grund.toml` is meant to be TOML, and a leniency nobody else shares is the compatibility trap this window exists to close while it is still rare.

**A new refusal message.** Rejected on [§DF-v2-array-one-trailing-comma.2.3](DF-v2-array-one-trailing-comma.md#23-one-code-with-a-remedy-that-names-the-line-to-edit): the empty element the comma leaves is what `expected string` already names, at the same line.

## 4. Consequences

A repository whose version-2 `grund.toml` holds such a comma keeps its exit code and its values in 0.21.0, and `check`, `config validate` and `config show` print the warning where `check` printed `success`; JSON output carries one more warning. From 0.22.0 every command that loads the config exits `2` until the extra comma is deleted. A version-1 config never breaks. `config explain config-not-toml` gives the remedy for each construct. The issue that closes the window in 0.22.0 is opened when this decision ships.

## release-note: Release note

- [§FS-config-v2.reader.5.commas](../../functional-spec/FS-config-v2.md#reader5commas-an-array-takes-one-trailing-comma-after-an-element), [§FS-config-v2.checks.config-not-toml](../../functional-spec/FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a comma before a version-2 array's `]` with no element before it now warns, and becomes an error in grund 0.22.0.** `exclude = ["ab",,]` loaded silently as `["ab"]`, and `exclude = [,]` as the empty list, though TOML allows one trailing comma, after an element; they still load, and `check`, `config validate` and `config show` print `warning: grund.toml:9: expected string, read for now with every comma before the `]` taken off; delete each comma on line 9 that has no element before it; this warning becomes an error in grund 0.22.0`, code `config-not-toml`, naming the line the comma stands on in an array written across lines. One trailing comma after the last element, `["ab",]`, loads as before with no warning, and a shape already refused, such as `["ab", ,]`, keeps its bare `expected string`. Version-1 configs load exactly as before. **Who this affects:** a repository whose version-2 `grund.toml` doubles a trailing comma or writes a comma in an empty array, which gets a warning now and a load error in 0.22.0. Closes [issue #1087](https://github.com/agent-grounds/grund/issues/1087).

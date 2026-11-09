# DF-fetch-names-an-executable: a `fetch` key must name an executable, warned for in version 2 through grund 0.22.x and refused from 0.23.0, and version 1 meets the rule at `grund fetch` and `config migrate`

**Date:** 2026-11-09

## status: Accepted

## 1. Context

`fetch = ""` loads under both config versions as a fetched kind, and `grund check` says nothing. `grund fetch` then joins the empty path to the config root, tries to execute the directory, and reports `Permission denied`, which sends a reader to file modes ([issue #1233](https://github.com/agent-grounds/grund/issues/1233)). `fetch = " "` loads the same way and fails as `No such file or directory`. [§FS-fetch.2](../../functional-spec/FS-fetch.md#2-integration-invocation) says the key names one executable and no clause said what a value that names none does, so [§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable) now does.

Moving `grund check` from exit `0` to exit `2` on such a tree is a verdict change, so [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) allows it only by one of its routes.

## 2. Decision

### 2.1 Refused rather than read as absent

Reading the value as the key's absence would silently turn a kind its author meant to fetch into one that is not fetched. A value that names nothing is reported, and testing only `""` would leave `" "` standing, so one test of the string covers both ([§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable)).

### 2.2 The route is the deprecation window

The route is [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). Version 2 warns at the key's line through grund 0.22.x, naming 0.23.0, and refuses from 0.23.0 ([§FS-config-v2.checks.config-fetch-names-nothing](../../functional-spec/FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)).

The correction route, section 5 of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly), is not open: it needs a hard requirement, at a numbered section, that the old verdict already broke, and "names one executable" and "a mistake in the config fails loudly" ([§FS-config.requirements.5](../../functional-spec/FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly--realized-one-case-deferred)) are specification sentences, as [§DF-v2-string-closes-on-its-line.2.2](DF-v2-string-closes-on-its-line.md#22-the-route-is-the-deprecation-window) ruled for the sibling case. The mechanical migration of [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit, because no command can know whether the author meant a program or no fetching at all. [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), what was never a promise, does not hold either, because the key has effects: the kind's missing snapshots are reported, a field on it is refused, and it is what lets a resolution setting load.

### 2.3 Version 1 keeps loading the key

[§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) keeps version 1's validation and [§FS-config.5.3](../../functional-spec/FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired) never retires a version 1 file, so its reader does not refuse this key. It meets the rule at `grund fetch`, where the refusal moves no verdict, and at `config migrate` ([§FS-config-migrate.3.1.10](../../functional-spec/FS-config-migrate.md#3110-a-fetch-that-names-no-executable)). No version 1 warning is added, as [§DF-v1-empty-extension-reading.2.2](DF-v1-empty-extension-reading.md#22-no-warning-is-added) reasons: a new warning is itself a verdict change, and every version 1 run already names `migrate`.

### 2.4 `grund fetch` changes in this release

It already exits `2` on this key, so no verdict moves. The new text keeps `cannot run fetch integration` and the quoted value as a verbatim prefix ([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text)) and says what is wrong where it quoted the operating system's reason ([§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable)).

### 2.5 One repair, true of every tree

A blank key is what lets `[rules.resolution] <KIND>` (version 1: `resolve`) load, so "remove the key" alone would stop that file loading, and a repair branching on whether the kind carries the setting would be a second fact to keep right. The repair names the setting for every tree and says the kind's missing IDs are then errors, in the same words at all four sites ([§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable)).

## 3. Alternatives considered

**Refuse now, under both versions**, as the report asks. Rejected on [§DF-fetch-names-an-executable.2.2](DF-fetch-names-an-executable.md#22-the-route-is-the-deprecation-window) and [§DF-fetch-names-an-executable.2.3](DF-fetch-names-an-executable.md#23-version-1-keeps-loading-the-key).

**Read the value as absent.** Rejected on [§DF-fetch-names-an-executable.2.1](DF-fetch-names-an-executable.md#21-refused-rather-than-read-as-absent).

**Test only the empty string.** Rejected on [§DF-fetch-names-an-executable.2.1](DF-fetch-names-an-executable.md#21-refused-rather-than-read-as-absent).

## 4. Who breaks

- A version 2 repository with a blank `fetch` gets a warning, with exit `0` and no `success` line, through grund 0.22.x, and exit `2` from 0.23.0.
- A tree with a blank `fetch` and `[rules.resolution] <KIND> = "warn"` (version 1: `resolve = "should"`) depends on the key: the key is what lets the setting load, and the kind's missing IDs are warnings because of it. Removing the key alone is a load error; removing both makes those IDs errors.
- A script matching a whole `cannot run fetch integration` line for this key sees longer text; the prefix stays.
- `config migrate` on a blank key goes from success to a refusal, with the file left alone.

A blank key could never fetch anything, so no working fetch is lost. The issue that makes the 0.23.0 refusal land is opened when this decision ships, and the release guard will not cut 0.23.0 over the pending message ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).

## release-note: Release note

- [§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable), [§FS-config-v2.checks.config-fetch-names-nothing](../../functional-spec/FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable), [§FS-config-migrate.3.1.10](../../functional-spec/FS-config-migrate.md#3110-a-fetch-that-names-no-executable), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a `fetch` key that is empty or only whitespace now warns under version 2, and becomes an error in grund 0.23.0.** `grund check`, `config validate` and `config show` print the warning at the key's line, and `grund fetch` and `config migrate` refuse the key, under both config versions, with the repair. If a kind's missing IDs are warnings only because of `[rules.resolution] <KIND> = "warn"` (version 1: `resolve = "should"`), the blank key is what lets that setting load: removing the key means removing the setting too, and those IDs are then errors.

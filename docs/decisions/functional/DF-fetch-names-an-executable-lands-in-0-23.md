# DF-fetch-names-an-executable-lands-in-0-23: a version 2 `fetch` key that names no executable is refused at load from 0.23.0

**Date:** 2026-11-09

## status: Accepted

## 1. Context

[§DF-fetch-names-an-executable](DF-fetch-names-an-executable.md#df-fetch-names-an-executable-a-fetch-key-must-name-an-executable-warned-for-in-version-2-through-grund-022x-and-refused-from-0230-and-version-1-meets-the-rule-at-grund-fetch-and-config-migrate) took the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): through grund 0.22.x a version 2 row whose `fetch` names no executable loads as a fetched kind and warns `config-fetch-names-nothing` at the key's line, naming 0.23.0. That record's notice ships with the warning, and a notice already released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that closes the window needs a notice of its own, and this record carries it.

## 2. Decision

In grund 0.23.0 (agent-grounds/grund#1321), version 2 refuses what 0.22.x warned for ([§FS-config-v2.checks.config-fetch-names-nothing](../../functional-spec/FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)):

- the key is refused at its line, ending `; this became an error in grund 0.23.0`, exit `2` on every command that loads the config and `1` on `config validate` ([§FS-config.4.3](../../functional-spec/FS-config.md#43-invalid-config-behavior));
- the test stays the string alone ([§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable)), so a real file named by whitespace does not rescue a version 2 key;
- a row an earlier release already refused, for lacking a citable row or one place, keeps that error, because a suffix on it would blame a release that changed nothing about it;
- `config-fetch-names-nothing` is spent and stays, so the `--ignore` the window recommended keeps working and the installed binary still answers for the code ([§REQ-discoverable-offline](../../requirements/REQ-discoverable-offline.md#req-discoverable-offline-the-installed-grund-answers-for-every-setting-and-every-check-code), [§REQ-spec-section-names.permanence](../../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent)).

Version 1 is unchanged: it keeps loading the key ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)) and meets the rule at `grund fetch` ([§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable)) and `config migrate` ([§FS-config-migrate.3.1.10](../../functional-spec/FS-config-migrate.md#3110-a-fetch-that-names-no-executable)).

## 3. Rejected alternatives

- **Read a blank value as the key's absence.** It would silently change whether the kind is fetched, as [§DF-fetch-names-an-executable.2.1](DF-fetch-names-an-executable.md#21-refused-rather-than-read-as-absent) rejected at the start of the window.
- **Remove the code with the warning.** A selector that named it would become an unknown-code error, so closing the window would break the flag the window offered.

## release-note: Release note

- [§DF-fetch-names-an-executable-lands-in-0-23](DF-fetch-names-an-executable-lands-in-0-23.md#df-fetch-names-an-executable-lands-in-0-23-a-version-2-fetch-key-that-names-no-executable-is-refused-at-load-from-0230), [§FS-config-v2.checks.config-fetch-names-nothing](../../functional-spec/FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable), [§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a version 2 `fetch` key that is empty or only whitespace is now an error.** What grund 0.22.x announced with its `config-fetch-names-nothing` warning lands: every command that loads the config prints ``error: grund.toml:6: `[schema.kinds.AR] fetch` names no executable; name the program, or if this kind is not fetched remove the key and any `[rules.resolution] AR`, and its missing IDs are then errors; this became an error in grund 0.23.0`` and exits `2`, and `config validate` exits `1`. `--ignore config-fetch-names-nothing` is still accepted and selects nothing. Version 1 configs load exactly as before. **Who this affects:** a version 2 `grund.toml` that drew that warning: name the program, or remove the key. If a kind's missing IDs are warnings only because of `[rules.resolution] <KIND> = "warn"`, the blank key is what lets that setting load, so remove the setting with the key, and those IDs are then errors. Closes [issue #1321](https://github.com/agent-grounds/grund/issues/1321).

# DF-v2-later-place-window: a version-2 row holds every entry to what its form asks of a home, warned for through 0.22.x and refused from 0.23.0

**Date:** 2026-11-04

## status: Accepted

## 1. Context

Version 1 gives a row one `file` and one `folder`, so "exactly one existing `file` or `folder`" ([§FS-config.3.4.9](../../functional-spec/FS-config.md#349-values--first-class-value-declarations)) could only count shapes. Version 2 writes places as lists ([§FS-config-v2.schema.places](../../functional-spec/FS-config-v2.md#schemaplaces-rows-and-their-places)), and its reader projects a row's lists onto version 1's record to ask the prerequisites of [§FS-fields.forms](../../functional-spec/FS-fields.md#forms-form-lowers-to-the-version-1-contracts), filling each slot with the first entry of that shape. Every later entry was never asked. So `files = ["values/absent.md", "values/a.md"]` on a `form = "value"` row is refused at load, and `files = ["values/a.md", "values/absent.md"]` is green, with nothing said about the absent home ([issue #1152](https://github.com/agent-grounds/grund/issues/1152)).

The gap runs through all three forms that read the record. A `content = "values"` field takes the same green, and a `form = "rule"` row takes it for a later file not named `.md`. A later `folders` entry on a `form = "value"` row is not green, but it fails only at the scan, with an error that names no config line, while `config validate` exits `0`.

An ordinary row's absent place is quiet by design ([§FS-config-v2.schema.places.boundaries.4](../../functional-spec/FS-config-v2.md#schemaplacesboundaries4-a-missing-place-is-quiet)), so nothing else reports a value source that was mistyped, moved or deleted, as long as it is not listed first. The first citer gets `unknown reference`, which sends the reader to the citation and not to the path in `grund.toml`.

Every release with the version-2 reader, 0.18.0 through 0.21.x, loads the green rows, and [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) allows a release to turn a tree that passes into one that is refused only by one of its routes.

## 2. Decision

### 2.1 Every entry is held, and several present entries keep loading

What a form or a values field asks of a home it asks of every entry, in the words the first entry of that shape is refused in ([§FS-fields.forms.places](../../functional-spec/FS-fields.md#formsplaces-every-entry-of-a-row-is-held-to-what-its-form-asks-of-a-home)). A row of several entries that all meet it keeps loading ([§FS-fields.forms.places.several](../../functional-spec/FS-fields.md#formsplacesseveral-several-homes-of-one-shape-load-and-each-is-read)). "Exactly one" keeps its meaning across the two shapes, so one `files` entry beside one `folders` entry stays refused with its existing bytes ([§FS-fields.forms.places.shapes](../../functional-spec/FS-fields.md#formsplacesshapes-exactly-one-counts-the-shapes-of-a-row), [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text)).

### 2.2 The route is the deprecation window

The route is the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): a later entry that loads and passes today warns through 0.22.x, naming the release that refuses it, and grund 0.23.0 refuses it ([§FS-fields.forms.places.window](../../functional-spec/FS-fields.md#formsplaceswindow-through-grund-022x-any-other-later-entry-loads-and-warns)).

The correction route, section 5 of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly), would refuse them in this release, and it needs a hard requirement, cited by number, that the old verdict already broke. None does. [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)'s surfaces do not list `config validate`, and a list written the other way round is a different configuration, not one tree answered two ways. [§REQ-deterministic-output.2](../../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation) counts the config as input, so a verdict that follows the order of a list is still the same bytes for the same input. [§REQ-no-missed-citation](../../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked) concerns citations in text the run reads, an absent home holds none, and a citer of a value in that home is already loud. The rules this does break are specification points, [§FS-config.3.4.9](../../functional-spec/FS-config.md#349-values--first-class-value-declarations) and [§FS-fields.forms](../../functional-spec/FS-fields.md#forms-form-lowers-to-the-version-1-contracts), and [§DF-v2-string-closes-on-its-line.2.2](DF-v2-string-closes-on-its-line.md#22-the-route-is-the-deprecation-window) already ruled that an FS point does not carry section 5.

The mechanical migration of section 3 does not apply either: no command can make the fix, because only the author knows whether the entry is a mistyped path, a home that was moved, or a line to delete.

The price is that the verdict on these entries still depends on the order of the list until the window closes.

### 2.3 What the scan already fails on is refused now

A window buys a passing tree time to move, and a tree holding a later `form = "value"` `folders` entry that is absent or not a folder, or a later `.json` `files` entry that is absent or not a file, does not pass: every command that scans exits `2` on it, with no config line in the error. Warning there would add a line to a run that already fails and leave the error unlocated, so it is refused at load with the first entry's message and no release clause ([§FS-fields.forms.places.unreadable](../../functional-spec/FS-fields.md#formsplacesunreadable-a-later-home-the-scan-already-fails-on-is-refused-at-load)). `check`'s verdict does not move.

Three commands read the config and not the scan, and they do move. `config validate` exits `1` and `config show` and `schema` exit `2` on such a row, where each exited `0`. That is what a first entry already does, and it is why `config validate` now says what every `check` on the row was going to say. No tree that passes `check` is on this row, so [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly)'s guarantee to a passing repository is not what moves; the release note names it because the exit of those commands is user-visible ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)).

A values field is not on this path. Its home holds no value source, so the scan never fails on its later entry, and each shape of it passes today and takes the window. A later `.yaml` entry on a value row is not on it either: an absent one is a warning today ([§DF-yaml-value-sources](DF-yaml-value-sources.md#df-yaml-value-sources-a-value-home-reads-yaml-beside-json-and-a-yaml-sources-findings-are-warnings-for-one-release)), so it takes the window beside that warning.

### 2.4 One new code, and one sentence

The warning's code is `config-later-place` ([§FS-config-v2.checks.config-later-place](../../functional-spec/FS-config-v2.md#checksconfig-later-place-a-later-entry-of-a-row-fails-what-its-form-asks-of-a-home)), by the channel `config-not-toml` uses. It could not be `config-not-toml`: such a file is TOML, and the reader reads it. Its message is the refusal the entry becomes, why it still loads, the one remedy and the release, so the first two clauses are what a person greps for today and in 0.23.0.

A rule kind's refusal names no entry, and its bytes do not change ([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text)), so the warning adds the entry after those bytes, where a value home's message already has it. How a rule home is judged Markdown stays what it is: its name, and that question belongs to [issue #1149](https://github.com/agent-grounds/grund/issues/1149).

## 3. Alternatives considered

**Refuse every row with more than one entry.** This is what "exactly one" reads as if it counts entries, and it would match the `fetch` and mixed-shape refusals. Rejected: two present files load and resolve on every release since 0.18.0, so it turns working rows red for a defect they do not have, and it crosses the owner's ruling that a kind may name several places ([§DISC-core-concerns.6.4](../../discussions/proposals/2026-09-30-core-concerns.md#64-amended-after-the-verdict-v-a08-and-v-a14)).

**Refuse every failing later entry in this release.** Rejected on [§DF-v2-later-place-window.2.2](DF-v2-later-place-window.md#22-the-route-is-the-deprecation-window): it is the correction route with no hard requirement behind it.

**Warn on every failing later entry, the unreadable ones too.** Rejected on [§DF-v2-later-place-window.2.3](DF-v2-later-place-window.md#23-what-the-scan-already-fails-on-is-refused-now): it would leave `config validate` passing and the scan's error unlocated on rows that already fail, which is the half of the defect a refusal here costs no passing tree to close.

**Accept one file beside one folder on a value row.** It would make the shapes consistent, but it widens what the tool accepts beyond this report and moves the bytes of an existing refusal. A question about it is a follow-up.

## 4. Consequences

A repository with a later entry that passes today keeps its exit code in 0.22.x, and `check`, `config validate` and `config show` print the warning, `check` where it printed `success`; JSON output carries one more warning. From 0.23.0 every command that loads the config exits `2` until the entry is corrected. Fixing a row may leave the same entry refused at another position, which is the point: its verdict no longer depends on where it is written. The issue that closes the window in 0.23.0 is opened when this decision ships.

## release-note: Release note

- [§FS-fields.forms.places](../../functional-spec/FS-fields.md#formsplaces-every-entry-of-a-row-is-held-to-what-its-form-asks-of-a-home), [§FS-config-v2.checks.config-later-place](../../functional-spec/FS-config-v2.md#checksconfig-later-place-a-later-entry-of-a-row-fails-what-its-form-asks-of-a-home), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a version-2 `form = "value"`, `form = "rule"` or `content = "values"` row now holds every `files` and `folders` entry to what it asks of a home, not only the first of each shape.** A later entry that fails and that passed `check` before still loads and now warns, `warning: grund.toml:6: value home for kind `CONST` does not exist: values/absent.md, loaded for now because it is not the row's first `files` entry; correct it or take it off the row; this warning becomes an error in grund 0.23.0`, code `config-later-place`, on `check`, `config validate` and `config show`. A later `folders` entry on a `form = "value"` row that is absent or not a folder, and a later `.json` `files` entry on one that is absent or not a file, already failed every scanning command with no config line; they are now refused at load with the first entry's message, so `config validate` exits `1` and `config show` and `schema` exit `2` on such a row where each exited `0`. A row of several present entries, one `files` entry beside one `folders` entry, a `fetch` row and version 1 do not move. **Who this affects:** a repository with such a later entry on a value, rule or values-field row, which gets a warning now and a load error in 0.23.0, or a load error now where every scanning command already failed. Closes [issue #1152](https://github.com/agent-grounds/grund/issues/1152).

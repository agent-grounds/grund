# DF-blank-fetch-check-tail: `check` offers the repair, not a command that can only refuse, for a `fetch` key with nothing to run

**Date:** 2026-11-09

Replaces the `run grund fetch <ID>` tail where `grund fetch` would have nothing to run for the kind, as [§FS-check.3.1.3.2](../../functional-spec/FS-check.md#3132-a-key-fetch-has-nothing-to-run-for) lays out, and joins the third exception of [§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change). Serves [§GOAL-friendliness-first.1](../../goals.md#1-hard-requirements): the next step a finding recommends has to be one that can work. Reported as [grund#1322](https://github.com/agent-grounds/grund/issues/1322).

## status: Accepted

## 1. Context

Version 1 keeps loading a `fetch` key that names no executable, for good ([§FS-config.3.4.10.2](../../functional-spec/FS-config.md#34102-fetch-and-resolve)), and such a key counts as `fetch` being present. So a citation of an ID the kind does not hold was told to `run grund fetch <ID>`, and that command can only print the refusal of [§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable). A reader, or an agent working through warnings, spent a turn per citation on a command that could not have worked, to learn the repair `check` could have given.

[§FS-check.3.1.3.1](../../functional-spec/FS-check.md#3131-a-home-fetch-refuses) already says that `check` does not send a reader to a command that refuses a home. It did not cover a key.

## 2. Decision

- **The question is `grund fetch`'s.** The test of the string alone would be wrong: a whitespace-only value naming a file literally called `" "` is still run. So `check` asks what `grund fetch` asks, of the project the ID is in, and that one path lookup runs nothing ([§REQ-runs-offline.1](../../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).
- **The words are the refusal's.** The tail is the key's name and the repair of [§FS-fetch.2.1](../../functional-spec/FS-fetch.md#21-a-value-that-names-no-executable), worded once in `dangling`'s catalog row with a `{setting}` slot ([§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog)), and spelled by the version of the project the ID is in.
- **The base, the code, the severity and the exit do not move,** so no verdict moves and section 5 of `REQ-backwards-compatibility` does not apply. Hints and refused homes keep their precedence.

The tail moves in place, with no window, under the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), as [§DF-stale-section-differs-in-0-20](DF-stale-section-differs-in-0-20.md#df-stale-section-differs-in-0-20-the-stale-block-finding-names-its-section-as-one-subject-corrected-in-place-in-0200) and [§DF-scan-failure-code-and-fix](DF-scan-failure-code-and-fix.md#df-scan-failure-code-and-fix-a-scan-failure-carries-its-own-code-and-a-fix-that-works-at-its-path) moved theirs. A window would print a command that can only refuse, on purpose, to exactly the projects this decision is about. The value-source and source-file tails of 0.22.0 were replaced the same way ([§DF-fetch-source-home.3.4](DF-fetch-source-home.md#34-what-each-finding-names)), so [§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change) now names the three of them as one entry: `check`'s fetch tail where `grund fetch` cannot run what it would name. The base survives as a verbatim prefix, and `code` is the contract for a program ([§FS-errors.requirements.fields](../../functional-spec/FS-errors.md#requirementsfields-what-a-program-needs-from-a-finding-is-a-field--realized-directional-for-the-next-finding)).

## 3. Consequences

- A consumer matching the words after the em dash, or the exact line, on a project whose `fetch` key has nothing to run stops matching, and moves to the JSON `code`.
- `grund fetch` of an imported ID keeps its read-only refusal, which in this one case names a command the target refuses with the repair ([§FS-imports.writes.answer](../../functional-spec/FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer)).
- `grund config explain` lists the case under `dangling` and `missing-snapshot`.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Keep `run grund fetch <ID>` whole for a release and append the repair | It recommends a command that can only refuse, which is the defect, and consumers key on `code`. |
| Test only that the value is blank | A file literally named `" "` is still run, so the tail would hide a command that works. |
| Add a new code or warning | The finding is the same finding with a remedy that works. A new code would move `--only` and `--ignore` for no gain. |

## release-note: Release note

- [§FS-check.3.1.3.2](../../functional-spec/FS-check.md#3132-a-key-fetch-has-nothing-to-run-for): **`check` no longer recommends `grund fetch` for a key that has nothing to run.** Where a kind's `fetch` is empty or only whitespace and no file of that name exists, a citation of an ID the kind does not hold now ends in the repair `grund fetch` itself gives, for example ``[[kinds]] `AR` fetch names no executable; name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors``, in place of `run grund fetch <ID>`. The code, severity and exit do not change. **Who this affects:** a script matching the words after the em dash, or the exact line, on such a project. Filter on the JSON `code` (`missing-snapshot` or `dangling`) instead.

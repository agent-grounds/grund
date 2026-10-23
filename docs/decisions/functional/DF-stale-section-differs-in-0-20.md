# DF-stale-section-differs-in-0-20: the stale-block finding names its section as one subject, corrected in place in 0.20.0

**Date:** 2026-10-23

Rewords the two stale templates of [§FS-errors.3.6.1](../../functional-spec/FS-errors.md#361-the-final-templates) and names the exception in [§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change). Serves [§GOAL-friendliness-first.1](../../goals.md#1-hard-requirements): the stale-block error is the first line `grund check` prints after a migration to config version 2, and it should read as a sentence. Reported as [grund#933](https://github.com/agent-grounds/grund/issues/933).

## status: Accepted

## 1. Context

The stale template read `stale grund init block: <section> differ from grund.toml`. It was written when every noun in the slot was plural, such as `chapter rules`. Since `check` compares the whole block ([§DF-check-compares-the-whole-block](DF-check-compares-the-whole-block.md#df-check-compares-the-whole-block-grund-check-holds-the-whole-managed-block-the-workspace-sections-at-the-outermost-root-of-a-full-checkout)), the slot takes any noun of [§FS-check.3.5.3](../../functional-spec/FS-check.md#353-code-and-message), and four of them are singular: `citation scheme`, `grounding from a citation`, `project map` and `scoped guidance`. A project migrated to version 2 is shown `citation scheme differ from grund.toml` and `project map differ from grund.toml` first. The noun can also be a heading someone wrote into the block, so the set of nouns is open.

[§FS-errors.3.6.1](../../functional-spec/FS-errors.md#361-the-final-templates) holds the first five templates stable, and [§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change) says a wording change keeps the old message whole through a window. So changing the text is a decision rather than a fix.

## 2. Decision

Both stale templates name the section as one singular subject:

```text
repo maintenance: stale grund init block: the <section> section differs from grund.toml — run `grund init` to refresh (does not affect citation validity)
repo maintenance: stale grund init block: the <section> section differs from the workspace — run `grund init` in <dir> to refresh (does not affect citation validity)
```

The `; run it in <dir>` tail of the first and the ` in <dir>` of the second stay where they were. The code `agents-init`, the error severity, the line, the exit status, the JSON shape and the leading `repo maintenance: stale grund init block: ` do not move.

The new text replaces the old in place in 0.20.0, with no window, under the pre-1.0 licence of [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise):

- A window would print the ungrammatical sentence on purpose, for one more release, to every project that migrates to version 2 in 0.20.0, which is the case this decision is about. Three of the four singular nouns have never shipped at all.
- Exact-line consumers of the `agents-init` messages were moved to `code == "agents-init"` by the migration that ended in 0.16.0 ([§FS-errors.3.6](../../functional-spec/FS-errors.md#36-the-agents-init-messages)), and no consumer acts on `message` ([§FS-errors.requirements.fields](../../functional-spec/FS-errors.md#requirementsfields-what-a-program-needs-from-a-finding-is-a-field--realized-directional-for-the-next-finding)). The prefix a consumer may match survives.
- [§DF-rule-refusal-reasons](DF-rule-refusal-reasons.md#df-rule-refusal-reasons-a-chapter-path-is-refused-for-the-component-that-failed-corrected-in-place) corrected a refusal's reason in place under the same licence.

No verdict moves: no exit status, code or severity changes. So this is not a verdict correction, and the five-condition route of section 5 of `REQ-backwards-compatibility` does not apply.

The sixth template, `differ from the workspace`, has never been released. It changes with the fourth so that the family keeps one shape, and so that one reader can take the noun back out of either.

## 3. Consequences

- The released bytes that move are `scoped guidance differ from grund.toml`, shipped in 0.18.0 and 0.19.0, and the plural `citation directions`, `clickable citations`, `chapter rules` and `declaration shapes` lines, shipped since 0.16.0. A script that matched the words after the prefix stops matching.
- A hand-written heading reads as a sentence too: `### My notes` reports `the my notes section differs from grund.toml`.
- `--only agents-init` and `--ignore agents-init` select what they selected before. `grund config migrate`'s report gate reads the noun back through the same text the finding is written with, so it sets aside and keeps the same findings.
- The block's text and version do not change, so nobody needs to run `grund init` because of this.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Make the verb agree with each noun (`project map differs`, `chapter rules differ`) | It moves fewer shipped bytes, but it needs every noun's number written down, and the set is open: a heading the user wrote has no number grund could know. The next section added to the template would need its number declared, and forgetting is how this defect would come back. |
| Add a `section` field that the migrate gate reads instead of the message | [§FS-errors.requirements.fields](../../functional-spec/FS-errors.md#requirementsfields-what-a-program-needs-from-a-finding-is-a-field--realized-directional-for-the-next-finding) would prefer a field, but `Finding` is the published embedder struct and the shape of the JSON object, so a new key is a surface change of its own. The only reader is grund's own gate in the same crate. |
| Keep each old line whole for one release and append a tail naming 0.21.0 | It shows the defect on purpose to the users it is about, and buys nothing: consumers already key on the code, and the prefix survives. |

## release-note: Release note

- [§FS-errors.3.6.1](../../functional-spec/FS-errors.md#361-the-final-templates): **the stale managed-block finding names its section as one subject.** `grund check`'s `agents-init` error for a block section that no longer matches read `<section> differ from grund.toml`, which is wrong for a singular section such as `scoped guidance`. It now reads `the <section> section differs from grund.toml`, for example ``repo maintenance: stale grund init block: the scoped guidance section differs from grund.toml — run `grund init` to refresh (does not affect citation validity)``. **Who this affects:** a script matching the words after `repo maintenance: stale grund init block: ` in the text line, the `message` in `--format=json`, or the `message` of a Python or Node finding. Match `code == "agents-init"` instead. The code, the severity, the line, the exit status, the JSON shape and that leading prefix are unchanged. Closes [issue #933](https://github.com/agent-grounds/grund/issues/933).

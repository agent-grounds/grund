# DF-index-carve-out-one-entry: the index carve-out writes one entry per owed ID, and check names no fix fmt declines

## status: Accepted

## 1. Decision

Where the ordinary link pass is off in a kind's index, in an excluded index file or inside a `grund:fmt off` region, `fmt` writes one entry for each ID the index owes and leaves every other citation as it stands ([§FS-fmt.2.5.3](../../functional-spec/FS-fmt.md#253-a-kinds-index-is-still-linkified)). Before, it wrapped every bare mention of an owed ID there, so `check` was green on a suppressed index while `fmt --check` asked to rewrite it. One link per ID is what an index owes ([§FS-check.3.18.6](../../functional-spec/FS-check.md#3186-and-nothing-more)), so a second mention was never the carve-out's to write.

A bare value binding, or a refused attempt at one, in a kind's index is no entry `check` asks `grund fmt --write` to link, because `fmt` leaves its citation unwrapped ([§FS-check.3.17.4](../../functional-spec/FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule), [§FS-values.8](../../functional-spec/FS-values.md#8-formatting-stability)). An index that mentions its ID only that way is reported as missing its entry, not as holding an unlinked one. Like a recognized binding's, the attempt's citation counts as use ([§FS-check.4.1.2.1](../../functional-spec/FS-check.md#4121-a-section-citation-or-a-value-binding-in-the-index-is-not-the-entry)).

## 2. Compatibility

Two verdicts move. `fmt --check` can exit `0` on an index whose only proposed edits were second mentions inside a suppressed scope, and `--write` writes fewer links there. The `check` exit does not move, but at a value form in a kind's index `unlinked-index-entry`, whose fix `fmt` declined, becomes `missing-index-entry`, whose fix is to write the entry. This corrects a formatter that disagreed with `check` ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)); no previously passing tree gains a finding.

## release-note: Release note

- [§DF-index-carve-out-one-entry](DF-index-carve-out-one-entry.md#df-index-carve-out-one-entry-the-index-carve-out-writes-one-entry-per-owed-id-and-check-names-no-fix-fmt-declines), [§FS-fmt.2.5.3](../../functional-spec/FS-fmt.md#253-a-kinds-index-is-still-linkified): **The index carve-out writes one entry per ID.** In an excluded kind's index, or a `grund:fmt off` region in one, `fmt` now links only each owed ID's entry and leaves its other mentions bare, so `fmt --check` agrees with `check` on a suppressed index. A value binding or refused attempt in a kind's index is reported as `missing-index-entry` rather than an `unlinked-index-entry` that `grund fmt --write` would not fix.

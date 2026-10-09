# DF-local-section-absent-target-escape: an owned local section citation whose section is absent is answered with the escape, in place

**Status:** Accepted
**Date:** 2026-10-09

## 1. Context

A bare numeric section token inside a declaration, such as `<§>5.1`, is owned by
that declaration and reported as `local-section-citation` with the remedy `write
<§>OWNER.5.1` (§FS-check.3.24). Where the owner has no section 5.1, that remedy
is a citation the same run reports on the same line as `missing section
OWNER.5.1` (§FS-check.3.2). An author who follows it trades one error for the
other, and one who does not has to prove the hint wrong before ignoring it.
In practice this shape is almost always prose naming a section of another
file, `argus.spec.md` and a marked `5.1` after it, which is exactly what the
`<§>` escape is for; and the escape is the one remedy the finding left out,
though the ownerless and unsupported shapes of the same rule already offer it
(agent-grounds/grund#515). On the reporting tree, thirteen such references in
one file each got the pair, and each had to be checked by hand against the
document it named before the escape turned out to be the fix every time.

`grund` already knew. The finding withholds `` ; run `grund fmt --write` `` at
these sites because the formatter refuses the rewrite ([§FS-check.3.24.1](../../functional-spec/FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld),
[§FS-fmt.2.4.6](../../functional-spec/FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)), and it decides that from the same lookup the `missing section`
error is reported from. It used that answer to drop the command and kept the
remedy.

The message is text a tool may match ([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text),
[§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)), and [§FS-check.3.24.2](../../functional-spec/FS-check.md#3242-an-append-not-a-wording-change) said its wording was
final. So changing it is a decision rather than a fix.

## 2. Decision

Where the owner has no section at the cited path, the owned finding names the
absence and gives the full-citation-or-escape guidance the other two shapes
give, with the escape echoing the token as written, the release attribution
last, and no command clause ([§FS-check.3.24.3](../../functional-spec/FS-check.md#3243-an-absent-target-section-is-answered-with-the-escape)). Where the owner has the
section, the finding keeps every byte. The new text replaces the old in place,
in one release, under the pre-release licence of
[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), as [§DF-rule-after-enabling-rewrites](DF-rule-after-enabling-rewrites.md#df-rule-after-enabling-rewrites-a-rule-subject-that-needs-named-sections-is-answered-with-one-they-make-valid-in-place) did for the
same question on another finding.

- Every byte that changes is advice that fails when followed: `write
  <§>OWNER.N` at these sites always yields `missing section` on the next run, so
  nothing that worked can have depended on it. Where the advice does work,
  because the section exists, it is kept.
- The finding already has a stable code, `local-section-citation`, and
  [§FS-check.3.24.2](../../functional-spec/FS-check.md#3242-an-append-not-a-wording-change) already sends an exact-line consumer to it. A migration
  window would give that consumer nothing new and keep the failing advice in
  front of every reader for another release.
- A consumer keeps the code, severity, location, `sites`, count, ordering, exit
  code, selection by `--only` and `--ignore`, the text through `local section
  citation <token>; `, and the attribution at the end. Only the words between
  those change, and only at these sites.
- No verdict moves. Nothing that passed fails, nothing that failed passes, and
  no finding appears or disappears, so this changes wording, not a verdict, and
  no verdict correction is needed.

This narrows one sentence of [§DF-declaration-local-section-shorthand.2.5](DF-declaration-local-section-shorthand.md#25-stored-citations-carry-their-full-id), that
the diagnostic still names the manual full replacement for an owned protected
site, to sites whose section exists. That record is not edited. Its
consequences already say that cross-document prose and broken section paths
require review rather than guessed rewriting
([§DF-declaration-local-section-shorthand.4](DF-declaration-local-section-shorthand.md#4-consequences)); this decision makes the finding
say the same.

## 3. Consequences

- A consumer matching the exact `message` of a `local-section-citation`
  finding at an owned site whose section is absent reads a new line. One
  matching through `local section citation <token>; ` keeps matching, and so
  does one matching the attribution at the end.
- A tool that extracted `write <citation>` from the message to apply it finds
  nothing to apply at these sites. What it applied there failed anyway.
- `missing section` does not change and still fires beside each site, and the
  two lines now agree. `--ignore missing-section` no longer hides the absence,
  because the first line names it.
- `grund fmt` does not change: it still leaves these sites alone, and the
  finding still never offers it there.
- [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) gains no entry. Its entries record migrations, and this is not
  one.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Append the absence and the escape after the attribution, the [§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text) route | No contract moves, but the advice that fails still comes first, permanently, which is what agent-grounds/grund#515 reports. |
| Migrate over three releases, as [§FS-errors.3.7](../../functional-spec/FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) did | The finding's code already exists and does not move, so a window gives exact-line consumers nothing new, and it prints the failing advice first for another release. |
| Put the escape on `missing section` instead | That finding names the canonical coordinate and also fires on full citations, where the usual cause is a mistyped number and the escape is the wrong advice. The advice for one token would be split across two lines with the wrong half first, and [§FS-check.3.2.1](../../functional-spec/FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not) holds that finding's wording fixed. |
| Keep `write <§>OWNER.N` and append the escape as a second option | The first option still fails whenever it is taken, and it is the one a reader or an agent applies first. |

## release-note: Release note

- [§FS-check.3.24.3](../../functional-spec/FS-check.md#3243-an-absent-target-section-is-answered-with-the-escape): **where a declaration-local section citation names a section
  its owner does not have, the finding says so and offers the escape.** Inside a
  declaration whose sections stop at 2, a marked `5.1` used to be told to write
  the owner's full citation of section 5.1, which the same run reports as
  `missing section` on the same line. Its `local-section-citation` finding now
  reads `local section citation <token>; <owner> has no section 5.1, so write a
  full citation or <§>5.1 to show the shape without citing it`, followed by the
  same release attribution and never by the `grund fmt --write` clause. A site
  whose section exists keeps every byte, the command clause included. **Who
  this breaks:** a script matching the exact `message` of a
  `local-section-citation` finding, in text or JSON, from `check` or the
  language server, at an owned site whose section is absent, including one that
  extracts `write <citation>` from it to apply. The code, severity, location,
  count, exit code, the text through `local section citation <token>; ` and the
  attribution at the end do not move, and `missing section` does not change.
  Closes [issue #515](https://github.com/agent-grounds/grund/issues/515).

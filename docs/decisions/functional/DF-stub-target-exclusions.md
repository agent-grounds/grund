# DF-stub-target-exclusions: Excluded and ignored stub targets are broken from grund 0.19.0

## status: Accepted

## context: The warning release has shipped

D5 of [§DF-one-answer-rulings-run.decision](DF-one-answer-rulings-run.md#decision-decision) schedules one eligibility answer:
a stub target honors exclude and enabled ignore rules with only the include gate
removed. grund 0.18.0 shipped the warning under
[§FS-declarations.checks.excluded-stub-target](../../functional-spec/FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-broken-from-grund-0190). This is the subsequent activation,
using the deprecation route [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

## decision: Activate the recorded refusal

[§FS-declarations.stubs.target.3](../../functional-spec/FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing) defines the active eligibility, precedence,
reason and remedy. A refused target makes its stub broken before pairing or
lending content ([§FS-declarations.stubs.target.5](../../functional-spec/FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)). Every surface reads the same
recorded verdict ([§FS-declarations.checks.broken-stub.4](../../functional-spec/FS-declarations.md#checksbroken-stub4-every-command-reads-the-one-verdict)).
Outside include alone remains allowed. No additional boundary refusal is introduced.
The old warning code stays accepted as a spent selector; its permanent explanation
is [§FS-declarations.checks.excluded-stub-target](../../functional-spec/FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-broken-from-grund-0190).

## release-note: Release note

From grund 0.19.0, a stub whose target is excluded or ignored by an enabled
ignore rule is an error coded `broken-stub`, instead of the
`excluded-stub-target` warning shipped in 0.18.0. Check exits 1 and show refuses.
Rejected targets no longer lend bodies, sections, chapters or citation edges.
Move the target or change the refusing rule named in the error.
An old `--ignore excluded-stub-target` no longer suppresses this finding;
select or ignore `broken-stub` instead. `--only excluded-stub-target` still
parses but selects no findings ([§FS-declarations.checks.excluded-stub-target](../../functional-spec/FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-broken-from-grund-0190)).
Targets outside include alone and explicit scan-root precedence keep their behavior.
The shared 0.19.0 release floor ([§FS-distribution.release-floor](../../functional-spec/FS-distribution.md#release-floor-one-shared-minimum-publishable-release)) prevents publishing
this activation in a 0.18.x patch; the diagnostic's landed clause is not a separate
release-floor source.

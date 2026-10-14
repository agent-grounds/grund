# DF-direction-absorbs-findings-not-rules: a `[citations]` direction absorbs a duplicate rule's finding for a unit it reports, not the rule

**Status:** Accepted
**Date:** 2026-10-14

## 1. Context

A rule that expresses the same kind-wide citation constraint as a `[citations]`
direction was deduplicated against it before evaluation, and the direction
won: the rule never ran ([§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication)). That is sound only if the two always
reach the same verdict, and they do not. A direction counts a citation by the
kind its token names whether or not it resolves, while a rule counts only a
citation that resolves
([§FS-rules.5.1](../../functional-spec/FS-rules.md#51-facts-and-identity)); [§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication) now
says so. Over `[citations.FS] must = ["REQ"]` and an `FS-a` whose only
requirement citation is the undeclared `<§>REQ-missing`:

```console
$ grund check . --ignore unused --rule 'Each FS must cite at least one REQ.'
docs/fs/FS-a.md:3: error: unknown reference REQ-missing
$ grund check . --ignore unused --rule 'Each FS must cite at least one REQ.' --only-rule
success
$ echo $?
0
```

Without the `[citations]` entry the same `--only-rule` run reports
`docs/fs/FS-a.md:1: error: FS-a must cite REQ (--rule)` and exits `1`: adding
the entry removed a finding. The same holds for a section citation its
declaration does not have (`<§>REQ-b.9`), for a dangling citation into a
workspace member (`<§>mem/AR-missing` against `must = ["mem/AR"]`), and at
`should` level, where the rule's `suggested-citation` vanished from
`--suggestions`. Reported as
[issue #727](https://github.com/agent-grounds/grund/issues/727).

## 2. Decision

Config-to-rule deduplication is decided per unit on the findings, after
evaluation, as field-to-rule pairing already is
([§DF-kind-fields](DF-kind-fields.md#df-kind-fields-a-kind-states-its-fields-by-section-name-and-a-present-optional-field-is-still-checked)). The rule runs. Where the direction reports a unit or site with the
same code on the same channel, the direction's finding stands byte-for-byte
and the rule's is dropped, exactly as before. Where the direction reports
nothing and the rule reports a finding, the rule's finding stands with the
rule's `authority`.

Making the two evaluators count the same evidence was rejected. Either way it
moves a verdict for every repository that uses the evaluator that changes,
including ones that declare no rule at all, and the question of what a dangling
token should satisfy belongs to its own change. Keeping the absorption and
adding a warning was rejected too: it leaves `--only-rule` exiting `0` on a
violated rule, which is the harm.

## 3. Consequences

- **A verdict moves.** A rule that duplicates a `[citations]` direction now
  reports `missing-citation` or `suggested-citation` at a declaration whose only
  citation of the target kind does not resolve, beside the existing
  `unknown reference` or `missing section`. A run scoped with `--only-rule`
  moves from exit `0` to exit `1`, and so does a run that ignores
  `unknown-reference` or `missing-section`, or reads them at a lower level.
  An unscoped run in the default configuration already exits `1` on the
  unresolved token, so its exit does not move; it gains one line.

  **Which clause licenses the move.** [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)
  does, and each of its conditions is met:

  1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), which already applied when the absorbing deduplication shipped: an incomplete run "never passes as if it were" complete. A rule the repository declared was never evaluated, and nothing said so, so the run passed as if it had been.
  2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form beside a new one to warn about: a release that printed the rule's finding as a warning while keeping exit `0` would keep the very verdict the prohibition forbids for a further release. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because the fix is to cite a declaration that exists, which no tool can choose for the author.
  3. *Named release.* The release note below names the move.
  4. *Actionable findings.* The finding that replaces the old verdict is the
     rule's own `missing-citation` or `suggested-citation`, located at the
     declaration and naming the kind it must cite, beside the existing finding
     at the unresolved token, which is where the edit goes.
  5. *No new licence.* The route is not a licence for anything else. Nothing is tightened, removed or newly prohibited: the rule already reports this finding on the same tree with the `[citations]` entry removed.

- **What stays.** A unit the direction reports is reported once, as the
  direction's finding, with empty `authority` and no rule ID appended. A trial
  sentence duplicating it therefore still authors nothing there, and
  `--only-rule` still prints `success` for that unit; that case stays deferred
  ([§FS-rules.requirements.unseen](../../functional-spec/FS-rules.md#requirementsunseen-nothing-a-rule-governs-passes-unseen--realized-one-case-deferred)).
- A repository that declares no rule is unaffected: `[citations]` is evaluated
  and rendered as before.

## release-note: Release note

- [§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication), [§FS-rules.8](../../functional-spec/FS-rules.md#8-command-surfaces), [§DF-direction-absorbs-findings-not-rules](DF-direction-absorbs-findings-not-rules.md#df-direction-absorbs-findings-not-rules-a-citations-direction-absorbs-a-duplicate-rules-finding-for-a-unit-it-reports-not-the-rule), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): **a `[citations]` direction no longer hides a duplicate rule's finding it does not report itself.** A rule such as `Each FS must cite at least one REQ.` beside `[citations.FS] must = ["REQ"]` now reports `missing-citation` (or, at `should`, `suggested-citation`) at a declaration whose only `REQ` citation does not resolve — a dangling ID, a missing section, or a dangling citation into a workspace member — which the direction counts and the rule does not. **Who this breaks:** the verdict moves from exit `0` to exit `1` for `grund check --rule … --only-rule` over such a declaration, which printed `success` and now prints `docs/fs/FS-a.md:1: error: FS-a must cite REQ (--rule)`, and for a run that ignores `unknown-reference` or `missing-section`; an unscoped default run gains that line beside the `unknown reference` it already fails on. Every finding that replaces the old verdict names the declaration's location and the kind it must cite, and the fix is to cite a declaration that exists. A unit the direction itself reports is unchanged: its finding wins byte-for-byte. Closes [issue #727](https://github.com/agent-grounds/grund/issues/727).

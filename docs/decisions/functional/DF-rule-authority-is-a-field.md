# DF-rule-authority-is-a-field: a finding's rule authority is a record field, and the trial-sentence selector is a query over it

**Status:** Accepted
**Date:** 2026-09-30

## 1. Context

`check --rule "<sentence>"` exists to try a sentence out before declaring it
([§FS-rules.8](../../functional-spec/FS-rules.md#8-command-surfaces)), and it
adds that sentence to every configured rule rather than replacing them. So the
answer to *what does this sentence find?* arrived mixed into the whole tree's
report, and nothing could separate the two.

Nothing in the selection surface could, either, and not by oversight. `--only`
and `--ignore` select by exact code
([§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore)),
and a trial sentence emits the same codes the declared rules emit — by
construction, since the code names the kind of failure and not who found it. An
authority is not a code, so no composition of that vocabulary could ever narrow
a report to one rule. The machine-readable surface was no escape: the record
([§FS-output-shapes.1](../../functional-spec/FS-output-shapes.md#1-finding-object))
carried no authority either, so a script had no better answer than a human's.

The only thing telling a trial finding from a declared rule's was the authority
name in the message tail, which
[§FS-rules.7.6](../../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits)
is free to reword. Filtering on it meant `grep -- "--rule"`: undiscoverable from
`--help`, silently wrong for a rule named `RULE-only`, and with no JSON
equivalent.

The datum was never missing. The engine already collects a semantic group's
contributing origins into a sorted set and joins it for the tail
([§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication));
[§AR-rules.2](../../architecture/AR-rules.md#2-parsedrule) already gives that
origin as stable, including the `--rule` origin. It reached a human-readable
string and stopped.

## 2. Decision

The authority stops being message text only. Every `check` finding record
carries `authority`: the bytewise-sorted set of rule origins that authored it,
`null` where no rule did
([§FS-errors.5.1](../../functional-spec/FS-errors.md#51-on-stdout--the-commands-output)).
`check --only-rule` is then defined *in terms of that field* — retain a finding
whose `authority` contains the `--rule` origin — so the flag and the key are one
mechanism rather than two that can disagree
([§FS-rules.8](../../functional-spec/FS-rules.md#8-command-surfaces)).

Four calls inside that, each of which could have gone the other way.

**A list, not the joined string.** The tail is `origins.join(", ")`; the honest
datum underneath is the set. A string makes a caller split on `", "` to ask "did
my sentence author this?", which is message-text matching wearing a different
hat. `["--rule","RULE-security"]` says *both* authored it where
`"--rule, RULE-security"` says only that there is a string. Order is the
bytewise order the tail already uses, which is what keeps
[§REQ-deterministic-output](../../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)
true of the new key.

**Present and `null`, never absent.** The key goes on the record shape
[§FS-distribution.3.0.1](../../functional-spec/FS-distribution.md#301-report-and-finding)
fixes, which is normative for every binding, so it appears — always `null` — on
a failed ID query and on a run-level finding too
([§FS-errors.5.2](../../functional-spec/FS-errors.md#52-on-stderr--what-is-not-output),
[§FS-errors.5.2.3](../../functional-spec/FS-errors.md#523-run-level-findings-in-checks-report)).
One key set is what makes the cross-binding equivalence test of
[§GOAL-multi-language](../../goals.md#goal-multi-language-same-engine-three-platforms)
mean anything, it is how `sites` already behaves on those same two shapes, and a
conditional key forces every consumer to branch.

**A rule-derived diagnostic carries its one rule's origin.** An `invalid-rule`
([§FS-rules.7.1](../../functional-spec/FS-rules.md#71-invalid-rule)) is a
diagnostic *about* a rule rather than an evaluation *by* one, so it never passes
through the group join. Setting its authority from that rule's origin anyway is
what keeps a syntactically valid sentence whose literal subject does not resolve
inside a scoped report. Defining the selector as "keep what the evaluator
attributed to `--rule`" would have dropped it and rendered a typo as `success` —
silently reporting nothing, which is the failure the whole feature exists to
remove.

**The selector is a boolean, and refuses to stand alone.** `--only-rule` means
"only what `--rule` found" and takes no value; a general
`--rule-authority <name>` is a larger contract than the ask and is deferred, not
foreclosed — `authority` is exactly what such a selector would query. Given
without `--rule` it is an invocation error with exit `2` rather than an empty
report, because a run scoped to no sentence would print `success` and exit `0`,
which reads as a verdict instead of as the mistake it is.

## 3. Alternatives rejected

- **Filter on the message tail, in the CLI alone.** A dozen lines, no record
  change, no public surface moved — and it is `grep -- "--rule"` with a flag on
  it: defeated by a rule named `RULE-only`, unusable by any later authority
  selector, and correct only while wording nobody promised stays put.
- **Make bare `--rule` imply scoping.** It changes a documented default, so a
  caller who wrote nothing gets something else, against
  [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly).
  Adding a rule and asking only about it are two different questions.
- **Teach `--ignore` to accept a rule identity.** It widens a documented value
  grammar and makes one flag select on two kinds of thing, so
  `--ignore RULE-count` and `--ignore io` would no longer mean the same shape of
  request.
- **A general `--rule-authority <name>` now.** More surface than the report
  asked for, and undecided in its own right: what an unknown name should do, and
  whether a project alias qualifies an origin, are questions this change does not
  need answered.
- **The field on `check` report findings only.** Cheaper diff, but it breaks the
  one-key-set property the multi-language equivalence test rests on and forces a
  consumer to branch per shape.
- **Exclude a jointly authored finding from a scoped report.** The trial sentence
  did author it. Hiding it would make a scoped report claim the sentence found
  nothing where it found something a declared rule happened to find too.

## 4. Consequences

A sentence can be tried for the cost of its own findings rather than the tree's,
which is what [§GOAL-token-economy](../../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)
asks of the affordance that created the problem, and any caller can ask which
rule authored a finding from a script.

Bare `--rule` is byte-identical, no exit code moves, and no finding appears or
disappears, so nobody who wrote nothing is affected on the text surface. The JSON
surface gains one key: a consumer that asserts a record's exact key set must
update, a consumer that reads the keys it wants is unaffected, and an embedder
that constructs or exhaustively matches a finding sees a new public field. That
is the one noticeable change, and it ships under **Changed**.

Two behaviors are now written down rather than merely true, so they cannot drift.
A jointly authored finding is retained with its authority and bytes unchanged. A
trial sentence duplicating a `[citations]` direction yields an empty scoped
report, because the config finding wins byte-for-byte and the rule authored
nothing
([§FS-rules.6](../../functional-spec/FS-rules.md#6-semantic-deduplication));
making such a sentence visible is a separate question, deliberately left open.

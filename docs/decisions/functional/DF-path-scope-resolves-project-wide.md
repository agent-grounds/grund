# DF-path-scope-resolves-project-wide: a path-scoped `check` resolves against the whole project and reports only the path

**Status:** Accepted
**Date:** 2026-10-01

## 1. Context

`grund check <path>` read exactly `<path>` and resolved exactly what it read, so a citation whose
declaration lived elsewhere in the same repository was reported `unknown reference <ID>` — the same message
and the same `dangling` code as an ID declared nowhere at all, with nothing in the output saying the
declaration had been outside the scanned path. On grund's own tree, clean under `grund check .`, the narrow
runs produced **6,836 findings, every one of them false**: 6,491 under `crates/`, 112 under `tests/e2e/`,
86 under `scripts/`, 47 on `README.md`. `grund check crates/grund-core/src/api/check.rs` printed
twenty-three `unknown reference` lines naming IDs that all resolve, and called a file holding twenty-three
citations an `ungrounded source file` ([§FS-check.3.6](../../functional-spec/FS-check.md#36-ungrounded-unit-opt-in)) on top of them — one cause reaching a second rule.

The declaring side was wrong in the mirror image: `grund check docs/` warned `declared but never cited`
about a declaration whose only citation sits in `src/`, which the run had not read.

That is a false alarm of the class [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) forbids outright — *a citation that resolves and
is written in canonical form is never reported as broken* — and the escape that point allows does not apply.
It permits a finding the spec knowingly scopes, but only on the condition that it "stay legible as such in
its message", and neither `unknown reference FS-widget` nor `declared but never cited: FS-widget` is legible
as a statement about a scope. [§GOAL-no-dangling-refs](../../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) is served by the same argument from the other end: a
checker that cries wolf 6,836 times teaches its reader to skip the finding that mattered.

The practical consequence was that the question "is this file clean?" had **no command**. The only correct
form was `grund check .` from the repository root piped through `grep`, which is what this repository's own
agent instructions had to document. A reviewer could not check the files a commit touched, a hook could not
check staged paths, and an agent working inside one module could not check its own work.

## 2. Decision

A run given an explicit path **reads what its project's ordinary run reads, and reports only the path**
([§FS-check.1.3.6.1](../../functional-spec/FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)).

### 2.1 Two scopes, named, and the whole change is telling them apart

The **resolution scope** is the set of files the run reads so that citations resolve and citation edges
count. The **report scope** is the set of files a finding may be reported about. For a run given an explicit
path these were one set — the path — and that was the defect. The report scope stays exactly the path, as
[§FS-check.1.3.6](../../functional-spec/FS-check.md#136-an-explicit-path-still-narrows) has always specified; the resolution scope becomes the enclosing project's ordinary scope,
`[scan] include` plus every walked kind home ([§FS-config.3.5](../../functional-spec/FS-config.md#35-scan--what-gets-scanned), [§FS-config.3.5.8](../../functional-spec/FS-config.md#358-every-configured-kind-home-is-scanned)), **union the path itself**,
because a path may lie outside `include` and must still be read.

The run then walks the resolution scope, runs every rule over the whole of it, and drops from the report
every diagnostic anchored at a file outside the report scope. Narrowing at the **diagnostic** stage rather
than the **scan** stage is what makes both halves of the defect fall out of one change: the dangling rule
sees the declarations, and the unused-declaration rule sees the citations.

This is not a new argument. [§FS-check.3.14.3](../../functional-spec/FS-check.md#3143-resolution-sees-the-whole-scan) already separates what resolution sees from what is reported,
one scope over — *"This check reports citations that point at nothing, not citations that point outside the
default scope."* This record is that sentence applied to the one scope that never had it.

### 2.2 The declaration set is the one `grund check .` reads, not the kind homes alone

The cheaper-looking reading — resolve from the configured `[[kinds]]` homes and nothing else — is wrong,
not merely incomplete. A declaration may live inline in a source doc-comment and enroll through its
canonical link in a kind index ([§FS-check.3.18.3](../../functional-spec/FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link), [§AR-scanner.4](../../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)): [§AR-checker](../../../crates/grund-core/src/checker/report.rs) is declared in
`crates/grund-core/src/checker/report.rs` and has no file in `docs/architecture/` at all. A homes-only read
misses that shape, and it is wrong in thirty-one places in this repository today. "The declaration side"
therefore **is** every scanned file, and there is no cheaper subset to walk.

### 2.3 The verdict moves in both directions, and the added finding is correct

Three verdicts retire: the dangling citation whose declaration is outside the path, the `ungrounded source
file` error derived from it, and the `declared but never cited` warning for a declaration cited from
outside the path. `grund check <path>` goes **1 → 0** where nothing is wrong.

One verdict is added, and it is the one direction a green path-scoped run can turn red: a **duplicate
declaration** whose twin lies outside the path was invisible and is now reported, exiting 1
([§FS-declarations.checks.duplicate](../../functional-spec/FS-declarations.md#checksduplicate-duplicate-declaration)). Before this change, `grund check docs/functional-spec/FS-widget.md`
over a doubly-declared `FS-widget` printed a warning that was false and missed an error that was real.
That is the general shape of the correction: the path-scoped run and the whole-tree run stop disagreeing.

### 2.4 No new finding message, and no new code

The dangling message and the `dangling` code are unchanged ([§FS-check.3.1](../../functional-spec/FS-check.md#31-dangling-citation)); so is the unused-declaration
warning ([§FS-check.4.1](../../functional-spec/FS-check.md#41-unused-declaration)). Nothing is reported with a scope qualifier, because a run that resolves
project-wide has nothing to qualify. So `--only` and `--ignore` gain no vocabulary ([§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore)), and the
`out-of-scope-dangling` code that [§FS-check.3.14.5](../../functional-spec/FS-check.md#3145-a-compound-code-per-rule) already assigns to a different scope is not reused for a
second meaning.

### 2.5 The unused-declaration warning is counted here, and `--full`'s is still not

[§FS-check.1.3.5](../../functional-spec/FS-check.md#135-the-unused-declaration-warning-is-unchanged-out-there) leaves the warning standing for a declaration cited only from outside `include` under
`--full`, and that stays true. The two cases differ by a contract. `--full` owes additivity
([§FS-check.1.3.4](../../functional-spec/FS-check.md#134-purely-additive)): it must report a superset of the plain run, so counting the wider walk's citations would
*retire* an in-scope finding, the one direction that flag may never move. A path scope owes nothing of the
kind — it is a subset run by construction, and it counts an edge **exactly where `grund check .` counts
it**, never anywhere else, so it cannot disagree with the run additivity protects ([§FS-check.4.1.4](../../functional-spec/FS-check.md#414-a-citation-anywhere-in-the-resolution-scope-counts)).

### 2.6 [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) is the route, and all five conditions hold

A shipped verdict flips here, which only [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) permits, and only on five conditions. Each is proved on its own line, because each is a separate gate.

**Prior prohibition:** the old verdict violated [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), a separately declared hard requirement whose prohibition already applied when the old verdict shipped — a citation that resolves and is written in canonical form was reported as broken, and neither `unknown reference FS-widget` nor `declared but never cited: FS-widget` is legible as the knowingly-scoped finding that point permits.

**Accepted proof:** this record, which cites that section and proves the conflict above. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path does not fit, because there is no old form to keep beside a new one — only a wrong answer, and nothing a user could write selects it. [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s mechanical migration does not fit either, because nothing a user wrote is at fault and there is no edit to make on anyone's behalf.

**Named release:** the release notes name the verdict change, in both of the directions it moves.

**Actionable findings:** every finding that replaces the old verdict names its location and the action to take — the duplicate declaration error names both declaration sites, which is the pair a maintainer has to choose between.

**No new licence:** this route cannot justify ordinary policy tightening, feature removal, or a prohibition invented by this change. Nothing is tightened, nothing is removed, and no new prohibition is created; one wrong answer becomes right.

## 3. Consequences

- `grund check <path>` costs what `grund check .` costs. Measured on this repository (631 scanned files,
  168 declarations, release build, warm, median of 15): `grund check <one file>` goes **4 ms → ~140 ms**,
  and on a 10,000-file fixture ~212 ms. [§GOAL-fast-feedback.1](../../goals.md#1-performance-targets)'s large-repo target (under 1 s on a
  10k-file repo) is met with room; its small-repo target (under 100 ms here) is **not**, and was already
  not met — `grund check .` on this repository is 140 ms today. The 4 ms bought a wrong answer, and the
  documented `grep` workaround already cost the full walk, so nobody loses a fast correct answer. The cost
  was put to the maintainer with the offer to re-plan instead, and accepted
  ([issue #370](https://github.com/agent-grounds/grund/issues/370)).
- No cheaper build was found. `grund list` — the closest proxy for "harvest declarations and check nothing"
  — is 125 ms of that 140 ms: the walk is the cost and the rules are 11% of it here. A declaration-only
  harvest would be 11% faster, could not fix the `declared but never cited` half at all (which needs the
  **citations** outside the path), and [§DF-path-scope-resolves-project-wide.2.2](DF-path-scope-resolves-project-wide.md#22-the-declaration-set-is-the-one-grund-check--reads-not-the-kind-homes-alone) says there is no subset of
  the tree to harvest from.
- A script that read `grund check <path>` exiting 1 as meaningful now gets 0 on a clean tree, and a script
  that greps its stdout gets fewer lines and `success` where there were findings.
- `--full` with an explicit path is still a no-op with its caution; only the caution's stated *reason*
  moves, because "already bypasses `[scan] include`" became false ([§FS-check.1.3.7](../../functional-spec/FS-check.md#137-a-path-the-flag-cannot-widen-earns-a-caution-not-a-refusal)).
- `--format json` keeps its object shape, codes and ordering, and emits fewer objects.
- Resolution never crosses a workspace member boundary, and `unknown project alias` on a narrowed member
  run is unchanged ([§FS-workspace.5.1](../../functional-spec/FS-workspace.md#51-a-member-run)) — a message that names its own scope, which is why a narrowed
  workspace run was never a second instance of this defect.
- `grund cover` is unaffected: it reports citation **sites**, never a resolution verdict, so its narrow
  answer was narrow but true, and it does not share `check`'s loader.
- The language server is the precedent rather than a caller to change. It already runs the check at the
  project root and distributes diagnostics per file, so project-wide resolution with per-file reporting is
  what a grund user has always met in an editor; `grund check <file>` was the one surface that disagreed.
- [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)'s blind-spot list gains the path scope, as a reporting boundary rather than a
  reading one — the cheaper kind to live with, because nothing goes unresolved for being outside it.

## 4. Alternatives considered

| Option | Why rejected |
|---|---|
| Keep the narrow read and qualify the message — `declared outside the scanned path: <ID>` | A new finding tier needs a new code, and `out-of-scope-dangling` is taken by a different scope ([§FS-check.3.14.5](../../functional-spec/FS-check.md#3145-a-compound-code-per-rule)). It also leaves the user with a report they must read past rather than an answer: the question "is this file clean?" still has no command. |
| Resolve from the configured kind homes alone | Misses a declaration living inline in a source doc-comment and enrolled from its index — thirty-one places in this repository ([§DF-path-scope-resolves-project-wide.2.2](DF-path-scope-resolves-project-wide.md#22-the-declaration-set-is-the-one-grund-check--reads-not-the-kind-homes-alone)). |
| Harvest only the declaration side outside the path | 11% of the cost for half the fix; the walk is the cost, and the unused-declaration half needs the citations, not the declarations ([§DF-path-scope-resolves-project-wide.3](DF-path-scope-resolves-project-wide.md#3-consequences)). |
| Let `--full` widen the resolution scope further under a path | Would give a path scope an out-of-scope tier, which [§FS-check.1.3.6](../../functional-spec/FS-check.md#136-an-explicit-path-still-narrows) says it has not got, and a new tier means a new code ([§DF-path-scope-resolves-project-wide.2.4](DF-path-scope-resolves-project-wide.md#24-no-new-finding-message-and-no-new-code)). |
| A flag to opt out of the widening and keep the 4 ms | Two knobs describing one scope, which is how two correctly-configured installs come to disagree ([§FS-non-goals.13](../../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)) — and the fast answer it preserves is the wrong one. |
| Split the `declared but never cited` half into its own change | Declined in the plan: a path scope counts a citation edge exactly where `grund check .` counts it, so it has no additivity contract to break and nothing to stage ([§FS-check.4.1.4](../../functional-spec/FS-check.md#414-a-citation-anywhere-in-the-resolution-scope-counts)). |

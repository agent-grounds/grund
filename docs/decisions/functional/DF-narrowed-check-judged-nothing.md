# DF-narrowed-check-judged-nothing: a narrowed check says when what it selected judged nothing

**Date:** 2026-10-20

## status: Accepted

## 1. Context

A narrowed `grund check` is how a person, a CI step or an agent asks one
question: `--only <code>` for one check, `--rule "<sentence>" --only-rule` for
one trial sentence. Two such questions got the answer a full pass gets when
nothing they asked about was judged at all:

```console
$ grund check --rule "Each e2e must cite at least one FS." --only-rule   # tests/e2e/ declares no e2e ID
success
$ grund check --only oversized-lead          # no [schema.leads.words] table
success
$ grund check --only oversized-lead --format json   # prints nothing on either stream, exit 0
```

The first sentence had no unit to apply to, and the second check was off. In
one run an agent took `success` as proof although three of five tests cited no
FS; in another, empty JSON sent it to inspect the binary with `strings` instead
of adding the missing table. Reported as
[issue #829](https://github.com/agent-grounds/grund/issues/829).

[§FS-check.requirements.success-earned](../../functional-spec/FS-check.md#requirementssuccess-earned-success-is-earned-never-a-default--realized)
already refused `success` for "an obligation with no unit to judge" and for "a
rule that could not run under a narrowing `--only`", but realized the first
only for a `[citations]` table, and dropped even that line under `--only`.

## 2. Decision

### 2.1 A rule joins `empty-citation-obligation`

A `must` or `should` rule, declared or given with `--rule`, whose subject kind
yields no unit earns the warning a `[citations]` obligation already earns, under
the same conditions and from the same function, with the rule named in the
table's place and the line ending at the fact
([§FS-check.2.2.1.4](../../functional-spec/FS-check.md#2214-a-rules-obligation),
[§FS-check.2.2.1.2](../../functional-spec/FS-check.md#2212-the-messages)). A
rule's subject is a citable kind, so the `citable = false` question the table's
line asks would get the trial sentence refused and turn a declared rule into an
`invalid-rule` error; the rule's line does not ask it. The line carries its
rule's authority, so `--only-rule` keeps the trial sentence's, and selecting a
code rules produce carries every `empty-citation-obligation` row as it carries
every `invalid-rule` row
([§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore)).
It is the same code, not a new one, because it is the same fact about the same
folder, and a second code would let `--ignore empty-citation-obligation` silence
only half of it. Since rules now produce it, it joins the rule-produced codes of
[§FS-rules.7.6](../../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits), so
`--only empty-citation-obligation` keeps every `invalid-rule` row as well: the
rule that could not run may be the one that would have said its subject was
empty.

### 2.2 A selected check that is off says so

Each gate that decides whether a check runs records the codes it armed, and a
code `--only` selects that no project armed earns one `selected-check-off`
warning naming what turns it on, in the config version the run read
([§FS-check.1.4.1](../../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)).
Recording at the gate keeps "is this check on" one fact; a predicate per code
read back from the config would restate a dozen gates and drift from them. The
code-to-advice table stays internal, and no command lists it.

### 2.3 A warning, not an error

Both lines are warnings: `success` is withheld and the exit does not move.
[§DF-narrowed-check-keeps-invalid-rule](DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run)
chose an error because the full run over the same tree already failed. Here the
full run passes, so failing the narrowed one would be new policy rather than a
correction.

## 3. Consequences

- Bytes change in three places, and one exit code moves. An `--only`
  of a code whose switch is off prints the off line on stderr in place of
  `success`, and gains a JSON warning object on stderr. A run narrowed to a code
  rules produce, over a tree with an empty obligation, now shows the table's
  line. A run with a `must` or `should` rule over a folder kind that holds files
  but no units gains the rule's line on stderr and loses `success`; that is the
  one change a run without `--only` or `--rule` can see.
- One verdict moves from `0` to `1`. `--only empty-citation-obligation` over a
  tree holding an invalid rule printed `success`; it now keeps the
  `invalid-rule` error the unnarrowed run already prints. That is the move
  [§DF-narrowed-check-keeps-invalid-rule](DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run)
  made for the other codes rules produce, for its reason: the full run over the
  same tree already exits `1`. `--ignore invalid-rule` restores the old view.
- A script that greps a narrowed run for `success`, or treats any stderr line as
  a failure, notices; so does a CI step that used `--only <code>` as a no-op.
- [§REQ-deterministic-output](../../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)
  holds: the off lines are written in code order.

**Which clause licenses the move.** New warning lines change bytes a caller
parses, which [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)
governs. [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)
licenses it, as it licensed
[§DF-unread-hidden-root](DF-unread-hidden-root.md#df-unread-hidden-root-a-configured-file-root-with-a-hidden-name-is-named-not-read)'s
warning whose exit did not change:

1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), which already applied when that verdict shipped: a run that read less never passes as if it were complete. It also broke [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), under which every blind spot is declared and answerable on demand. A narrowed run that judged nothing and printed what a full pass prints is that run.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form to carry beside a new one: the missing line is the change. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to run, because the remedy is the author's, among the ones the line names.
3. *Named release.* The release note below names the change and the opt-outs.
4. *Actionable findings.* Each line names the switch that turns the check on,
   the replacement of a spent code, or the folder that holds no unit.
5. *No new licence.* The route is not a licence for anything else: nothing is
   tightened or newly prohibited, and the one exit that moves is that of a run
   the unnarrowed check over the same tree already fails, as under
   [§DF-narrowed-check-keeps-invalid-rule](DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run).

**The weak point.** For the empty rule subject the prohibition is plain: no
section said a rule over an empty kind may pass silently, so "a skip that no
section names" in [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)
covers it. For `oversized-lead` and the spent stub codes it does not. Their
silence was written into the spec: `--only oversized-lead` "never activates" the
check and the absent key left the output "byte-identical"
([§FS-declarations.checks.oversized-lead.3](../../functional-spec/FS-declarations.md#checksoversized-lead3-only-the-key-activates-it),
[§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore)),
and an old `--only excluded-stub-target` kept "normal selected-report exit
semantics"
([§FS-declarations.checks.excluded-stub-target](../../functional-spec/FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-broken-from-grund-0190)).
An objector can say those skips were named. They were named in the spec, not to
the caller: the run that asked about them could not tell an off check from a
pass, and a blind spot is declared only where it is answerable on demand. That
is the clause these cases rest on, and it is narrower than the first. Were the
route refused for them, the fallback is the
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)
deprecation path for those codes alone: for one release the off line is printed
beside `success`, and a note names the release in which it takes its place.

## release-note: Release note

- [§FS-check.1.4.1](../../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so), [§FS-check.2.2.1.4](../../functional-spec/FS-check.md#2214-a-rules-obligation), [§DF-narrowed-check-judged-nothing](DF-narrowed-check-judged-nothing.md#df-narrowed-check-judged-nothing-a-narrowed-check-says-when-what-it-selected-judged-nothing), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids): **a narrowed `check` says when what it selected judged nothing.** `--only <code>` of a check the run leaves off prints `warning: --only <code> judged nothing — …`, naming the switch that turns it on, in place of `success`, with the new code `selected-check-off`. A `must` or `should` rule, declared or `--rule`, whose subject kind holds files but no unit prints `warning: <rule> applies to nothing — …` under `empty-citation-obligation`, and a run narrowed to a code rules produce now keeps every `empty-citation-obligation` row. **Who this breaks:** a script that greps a narrowed run for `success` or treats a stderr line as failure, and a CI step that used `--only <code>` as a no-op; with `--format json` stderr gains a warning object. **One exit moves:** `empty-citation-obligation` joins the rule-produced codes, so `--only empty-citation-obligation` keeps every `invalid-rule` row, and over a tree holding an invalid rule it exits `1` with the error the unnarrowed `grund check` already prints, where it printed `success`. **The opt-outs:** `--ignore selected-check-off`, `--ignore empty-citation-obligation`, and `--ignore invalid-rule`. The verdict moves by the correction route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), because the old `success` broke [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips) and [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded); every finding that replaces it is about the run rather than a file location, or is the `invalid-rule` row the unnarrowed run already prints, and names the fix: the switch to set, the code to select, the folder that holds no unit, or the rule to repair. Closes [issue #829](https://github.com/agent-grounds/grund/issues/829).

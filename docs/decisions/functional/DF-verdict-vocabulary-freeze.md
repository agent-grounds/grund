# DF-verdict-vocabulary-freeze: the freeze is on the verdict vocabulary, not on which rules a project holds in force

**Status:** Accepted
**Date:** 2026-09-26

## 1. Context

Two standing points disagreed about who the freeze on severity, exit codes and report ordering binds. [§GOAL-configurable.2](../../goals.md#2-what-is-not-configurable) stated it under an install-local subject — *severity, exit codes, report ordering … have to agree for two installs reading the same project and the same configuration* — while [§FS-config.6](../../functional-spec/FS-config.md#6-what-is-not-configured-here), [§GOAL-friendliness-first.2](../../goals.md#2-what-this-rules-out) and [§FS-non-goals.9](../../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization) state the same list unqualified, for every project. The scope contradiction was recorded as `C-hubs-198`, deferred out of issue #288 as needing a ruling rather than an edit, and ruled on issue #294.

It was not a wording dispute. Four settings ship today that a project commits and that move `grund check`'s verdict with no machine-local state anywhere, so the unqualified reading was already false about the tool. The install-local reading, meanwhile, said nothing at all about them — a `grund.toml` is shared input, so two installs reading it still agree, and the reader is left without an answer to the question they actually asked.

That reader is not hypothetical. [§DF-unmarked-markdown-headings](DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph) refused a `allow | warn | error` selector for one check and cited the goal's install-local sentence as its authority — while `section_heading_levels`, in the same specification chapter, is exactly such a selector for a different check. The refusal was right; the authority was not.

Serves [§GOAL-configurable](../../goals.md#goal-configurable-every-default-is-overridable): *may my `grund.toml` do this?* becomes one read of the goal rather than an induction over four points that are about other questions.

## 2. Decision

### 2.1 The doctrine

For every project and every install alike, the **set** of severity levels is exactly `{error, warning}`, the **mapping** from a report to an exit code is the one [§FS-cli.5](../../functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed) fixes, and the **ordering of the report** is the grouped, bytewise one of [§FS-errors.4.1](../../functional-spec/FS-errors.md#41-ordering). That trio is the verdict *vocabulary*, and nothing configures it.

What a project's committed configuration may choose is which of the specified rules are in force over its own tree, and — where a rule's own specification fixes the complete set of channels its finding may speak through — which of those channels it speaks through there. Neither choice touches the trio: a project that holds one rule at advisory standing has configured that rule, not the severity set.

Separately, and about a different axis, no install-local value may change any verdict at all ([§FS-non-goals.13](../../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). The procedure for staying on the right side of that line when authoring a key is [§FS-config.principle.install-local](../../functional-spec/FS-config.md#principleinstall-local-the-relation-governs-committed-repository-state-only)'s — classify first, verify by reach — and is not restated here.

### 2.2 The admissibility test

A proposed setting is **admissible** only if all four limbs hold. They are limbs of one test: a key that fails any one of them is refused.

1. **The set.** No legal value of the key adds, removes or renames a severity. A suggestion is not a third severity; it rides the separate advisory channel of [§FS-check.2.3](../../functional-spec/FS-check.md#23-suggestions-channel-opt-in), which is why the set stays exactly two ([§FS-config.6.1](../../functional-spec/FS-config.md#61-suggestions-are-not-a-third-severity)).
2. **The mapping and the order.** No legal value changes the report→exit function or the order findings are reported in. A key may change *what is in* the report; the function from a report to an exit code, and the order, are not its to move.
3. **Locality.** The key is declared at the one rule it governs, reaches no other rule, takes no rule or finding as its argument from outside that rule's point, and has its complete value→channel ladder fixed in that rule's own specification. A closed ladder written at the rule keeps the blind spot it opens declared and bounded — the same discipline [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) applies to the regions the scanner does not read.
4. **Scope.** A verdict-affecting key is committed repository state. An install-local key is classified as presentation and then verified to change no verdict, per [§FS-config.principle.install-local](../../functional-spec/FS-config.md#principleinstall-local-the-relation-governs-committed-repository-state-only).

Whether a given rule *should* have such a ladder is a specification judgement made at that rule. It is not an output of this test, and no gate answers it.

### 2.3 The shipped settings the test decides

Four committed settings move a verdict today. All four are admissible, and always were.

| Setting | What it moves | The limb that decides it |
| --- | --- | --- |
| `[citations.FS]` and its siblings, `must` against `should` ([§FS-config.3.9](../../functional-spec/FS-config.md#39-citations--citation-direction-rules)) | a direction's standing: exit `0` → `1` on an unchanged tree | 1, 2 — the obligation is in force or it is not; the level→surface mapping is itself fixed ([§FS-config.3.9.1.3](../../functional-spec/FS-config.md#3913-the-levelsurface-mapping-is-fixed)) |
| `[reference] shorthand = "accepted"` ([§FS-config.3.1.1](../../functional-spec/FS-config.md#311-shorthand--persisted-number-only-citations)) | whether the persisted-shorthand rule fires at all: exit `1` → `0` | 1, 2 — the rule fires or it does not; neither value touches the set or the report→exit function |
| `[reference] inline_note_layout_check`, `off / warn / error` ([§FS-config.3.1.9](../../functional-spec/FS-config.md#319-inline_note_layout-and-inline_note_layout_check)) | that rule's channel, and with it the exit code | 3 — a closed ladder fixed at the rule |
| `[id] section_heading_levels`, `strict / warn / loose` ([§FS-config.3.3.2](../../functional-spec/FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)) | that rule's severity `error` → `warning`, exit `1` → `0`, identical code and byte-identical message | 3 — a closed ladder fixed at the rule |

`[reference] lead_size_warning` ([§FS-config.3.1.2](../../functional-spec/FS-config.md#312-lead_size_warning--the-oversized-lead-opt-in)) is the control on the other side: it conjures warnings out of a clean tree and the exit code stays `0`, because it has no severity field to move. It holds all four limbs, as the other four do; limb 1 is the one at issue, and the severity set its warnings draw from is the frozen one.

### 2.4 What the test refuses

A key that looks like a channel selector but reaches past its own rule fails limb 3, not the doctrine: `[severity] downgrade = [...]`, which takes other rules as its argument, and `[check] level = "warn"`, which sets a standing for rules it does not name. Both would leave the vocabulary intact and are refused all the same. So is a rule-local selector whose ladder is open-ended, because what it may report is then not stated anywhere.

[§DF-unmarked-markdown-headings](DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph)'s refusal survives this record and is re-grounded by it: an unmarked-heading selector would pass all four limbs, and is still refused, because `grund` ships no command that can choose an author's intended hierarchy — there is no standing an advisory mode could report from. That is the rule-specific judgement [§DF-verdict-vocabulary-freeze.2.2](DF-verdict-vocabulary-freeze.md#22-the-admissibility-test) says the test does not make.

## 3. Alternatives rejected

- **The unqualified reading: severity, exit codes and ordering are not configurable, full stop.** It is false about the shipped tool. Four committed settings move a verdict, and under this reading each is a specification violation the spec never noticed — including two that a project sets precisely so CI can stay green through a migration.
- **The install-local reading: the trio is frozen only against machine-local variation.** It answers a question nobody asks. A `grund.toml` is shared input, so two installs agree under it by construction; a reader holding this reading learns nothing about what their own committed file may do, which is what sent [§DF-unmarked-markdown-headings](DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph) to the wrong authority.
- **Putting the classify-first-then-verify-by-reach procedure in the goal as well.** Ruled against: a goal that teaches key authorship reaches machine-local behaviour the question never asked about. The goal cites [§FS-config.principle.install-local](../../functional-spec/FS-config.md#principleinstall-local-the-relation-governs-committed-repository-state-only) instead, which keeps the counterexample — inert `[output] color` changes no verdict and is committed repository state all the same — in the one place that already carries it.
- **Amending [§FS-config.6](../../functional-spec/FS-config.md#6-what-is-not-configured-here), [§GOAL-friendliness-first.2](../../goals.md#2-what-this-rules-out) or [§FS-non-goals.9](../../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization).** All three read true under the doctrine: each names the *set*, the *mapping* and the *ordering* rather than a rule's standing. Sharpening them is a different review with a different risk, and [§FS-config.6](../../functional-spec/FS-config.md#6-what-is-not-configured-here) is the point the ruling ratified.
- **Leaving the doctrine in the goal paragraph with no record.** The goal can carry the bright line; it cannot carry a four-limb test, its worked cases, or what the ruling costs. This ticket exists because a previous ruling's operative note was left unwritten and then reported as spent.

## 4. Consequences and cost

No behaviour changes. No command, flag, configuration key, output byte or exit code moves; no repository that passes today fails tomorrow or the reverse; `grund_config_version` stays 1. The four settings above behave exactly as they did and are now documented as conforming rather than tolerated.

What changes is the review question. *Does this key configure severity?* stops being what a proposed mode key is asked, and *should this rule be one a project may hold at advisory standing?* replaces it — a judgement about that rule, made at that rule, which [§DF-verdict-vocabulary-freeze.2.2](DF-verdict-vocabulary-freeze.md#22-the-admissibility-test)'s limbs bound but do not answer.

And one sentence in common use is now false as stated. "Severity is not configurable" survives only as "the *set* of severity levels is not configurable": a project may already choose which rung a rule sits on wherever that rule's specification fixes the ladder. One decision record in this tree refused a feature on the false version and happened to be right for other reasons; the next such refusal might not be, which is the cost of having left this unstated until now.

# DF-closed-row-must-not-cite-anything: A row closed to every citation renders one unnarrowed "must not cite anything"

**Date:** 2026-11-19

Settles how the `### Citation directions` section states a row closed to every citation ([§FS-config.3.9.4.1](../../functional-spec/FS-config.md#3941-a-row-closed-to-every-citation)), as [§FS-init.2.3.5](../../functional-spec/FS-init.md#235-citation-directions) specifies. Refines the narrowing of [§DF-directions-render.6.1](DF-directions-render.md#61-decision), which was written for a row with an obligation. Serves [§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work): the rule an agent reads is the rule `grund check` enforces.

## status: Accepted

## 1. Context

A project closes its non-citable `ce/` home with `[citations.ce] default = "must-not"` and leaves its grounding off. `grund check` enforces that: a citation in `ce/A.java` is reported `ce/ must not cite FS (citation direction)`. But the block told agents something else:

```text
- Each file in **ce/** that cites anything must not cite anything. A file there that cites nothing is not checked; `require_grounding = true` on its `[[kinds]]` row makes it a finding.
```

The first sentence contradicts itself. The second names the one setting under which no file in `ce/` can pass, because with grounding on an uncited file is `ungrounded` and a cited one is a forbidden citation. The config with that setting loads in silence. The homeless kind read `Each source file outside the Project map (**code**) that cites anything must not cite anything.` in the same contradiction. Both are the narrowing and the hint of [§DF-directions-render.6.1](DF-directions-render.md#61-decision), applied to every non-citable row that grounds nothing. That narrowing exists for agent-grounds/grund#908's row, `must = ["FS"]`, where a file that cites nothing really does slip past the rule and grounding really closes the hole. A closed row has no such hole: a file that cites nothing already keeps it (agent-grounds/grund#1456).

## 2. Decision

A row is closed to every citation when its effective fallback is `must-not` and every list it has is `must-not` ([§FS-config.3.9.4.1](../../functional-spec/FS-config.md#3941-a-row-closed-to-every-citation)). The Citation directions bullet reads that one predicate:

- **The subject.** A closed row takes its unit's unnarrowed subject and no second sentence, whatever its grounding, the homeless kind's included ([§FS-init.2.3.5](../../functional-spec/FS-init.md#235-citation-directions)).
- **The clause.** A closed row renders the one clause `must not cite anything`. Its `must-not` lists fold into it with its own default's clause, because every target they name is one that clause forbids already ([§FS-init.2.3.5](../../functional-spec/FS-init.md#235-citation-directions)). The fold is the row's, so a citable row closed that way folds too: `- Each **FS** declaration must not cite anything.`
- **What stays.** Whether a row has a bullet does not change: a row closed by the global default alone still has none. Open and discouraged rows, a closed default with a `may`, `should-not` or `warn-not` exception, a `should-not` or `warn-not` default, and agent-grounds/grund#908's obligation row keep their bullets byte for byte, the obligation row its narrowing and its hint. `grund check` does not change, and no verdict moves.

So the closed `ce/` home renders `- Each file in **ce/** must not cite anything.` under either version, grounding off or on, and the homeless kind `- Each source file outside the Project map (**code**) must not cite anything.` The text an agent reads moves under one block-version bump, v51–v54 ([§FS-init.2.3.7](../../functional-spec/FS-init.md#237-the-block-version)).

Grounding on a closed home still loads in silence. The warning that names both settings is a change of its own, agent-grounds/grund#1460, which reads the same predicate.

## 3. Alternatives considered

| Option | Why rejected |
|---|---|
| `Files in **ce/** cite nothing.`, the report's own wording | It drops the verb the legend defines, so it no longer says the rule is a `grund check` error, and a bullet in a second sentence shape needs a clause grammar [§FS-init.2.3.5](../../functional-spec/FS-init.md#235-citation-directions) does not have. |
| Keep the narrowed subject and drop only the hint | `that cites anything must not cite anything` still contradicts itself. |
| Keep `must not cite FS; never cite anything else` for a closed row with its own list | It names a target the next clause already forbids, so a reader looks for a difference between FS and anything else that `grund check` does not make. |
| Fold only a non-citable row's lists | The clause belongs to the row, not to its place. A citable row's subject is never narrowed, so the only citable bullets that change are closed rows that list their own `must-not` targets. |
| Refuse or warn on grounding a closed row in the same change | The author split the warning out, so this change ships the bullet alone and the warning lands after it (agent-grounds/grund#1460). |

## release-note: Release note

- A row closed to every citation, one whose effective citation default is `must-not` and whose every list is `must-not`, now renders its Citation directions bullet as one sentence: `- Each file in **ce/** must not cite anything.` A non-citable home closed that way that grounded nothing read `Each file in **ce/** that cites anything must not cite anything.`, followed by a sentence naming the grounding setting under which no file there can pass, and the homeless kind's closed bullet loses its `that cites anything` too. A closed home whose only rule is its `must-not` default keeps its bullet when its grounding is on, because it already rendered `must not cite anything` unnarrowed. Two shapes change whatever the grounding: a closed row with its own `must-not` list folds it, so `must not cite FS; never cite anything else` and, under a `must-not` global default, `must not cite FS` both become `must not cite anything`; and a closed homeless kind loses its narrowing. The managed block's versions advance from v47/v48 to v51/v52 and from v49/v50 to v53/v54. Existing blocks are reported outdated by `grund check` and `grund init --check`; run `grund init` to refresh them, preserving unmanaged text. No verdict changes and no config is refused.

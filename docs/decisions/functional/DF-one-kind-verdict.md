# DF-one-kind-verdict: every command reads one kind verdict in its own scope, and refuses a rule sentence for the first stage it fails

**Status:** Accepted
**Date:** 2026-10-12

## context: Context

Step 6 of [issue #576](https://github.com/agent-grounds/grund/issues/576), [issue #599](https://github.com/agent-grounds/grund/issues/599), carries out the rulings D10, D11, D18 and D20 of [§DF-one-answer-rulings-grammar](DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary). It does not decide them again; this record holds the routes the step took and the one notice it publishes. Three rows of [§REQ-one-answer.differences](../../requirements/REQ-one-answer.md#differences-the-differences-known-when-this-requirement-landed) stood against it:

- **G03.** A `[citations]` target `api/API` was refused on load as an unknown kind, while the rule `Each FS must cite at least one api/API.` parsed in the same workspace.
- **G04.** One non-citable kind got three different refusal reasons from `list --kind`, `list --selector` and a rule subject.
- **G05.** With named sections off at the root and on in member `api`, `grund api/API-a.terms` answered and an unfiltered `list --selector API.terms` refused.

All three are one fact asked in several places: what a kind name is, in a given scope. About ten readers built their own kind set to answer it. The rule-sentence refusals had a second, smaller copy of the same kind: the documented order of [§FS-rule-refusals.sentences.conjunction](../../functional-spec/FS-rule-refusals.md#sentencesconjunction-any-clauses-joined-by-and-are-refused-as-a-conjunction) put the phase-1 subject refusals before the conjunction, and the code checked a section-component wildcard after it.

The coordinate lexer the issue first named is cut out into [issue #626](https://github.com/agent-grounds/grund/issues/626): #510, #513 and #568 have shipped, so it is a refactor that moves no verdict. A rule in a nested member that pins its own alias misses that member's declarations; that is a different fact, which declarations a target matches, and is [issue #627](https://github.com/agent-grounds/grund/issues/627).

## decision: Decision

### decision.verdict: One reading of every project's kinds

A run reads the kinds of every project its configuration declares, once, from the configs ([§FS-scan.projects.kinds](../../functional-spec/FS-scan.md#projectskinds-whose-kinds-a-run-reads)), and every kind-name question is answered from that reading within the asking command's documented scope. The scope stays each command's own, as D18 requires; the verdict within it is one. A kind's named sections are read under the project that configures it. A non-citable kind is refused for that reason on every surface ([§FS-errors.2.5.1](../../functional-spec/FS-errors.md#251-a-kind-that-declares-no-ids)), in the quoting the surface already uses.

### decision.order: One refusal order

A rule sentence is refused for the first stage of [§FS-rule-refusals.sentences.order](../../functional-spec/FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails) it fails. The order is the documented one, as D20 requires; the one move is that a section-component wildcard is decided with the other phase-1 subject refusals, ahead of the conjunction.

### decision.routes: Which route each move takes

Every move takes [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). None is a correction: no requirement forbade the old answers, and [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) is never the requirement a correction rests on ([§DF-one-answer-rulings-grammar](DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary)).

| Move | Next release | `0.19.0` |
|---|---|---|
| A pinned or `*/` target naming a citable kind of its project | loads and is enforced | same |
| `list --selector KIND.NAME` where `KIND`'s own project enables named sections | answers | same |
| A pinned or `*/` target its project does not have citable, or an alias the workspace does not load | loads, with a warning naming `0.19.0` ([§FS-config.3.9.5.2](../../functional-spec/FS-config.md#3952-a-pinned-target-is-judged-against-its-projects-kinds)) | an error, exit `2` |
| A pinned target in a run that holds no workspace | loads, with a warning in [§FS-errors.3.7.1](../../functional-spec/FS-errors.md#371-the-final-templates)'s wording naming `0.19.0` | an error, exit `2` |
| A non-citable kind on a selector or in a rule | the legacy reason, then the new one appended ([§FS-errors.3.8](../../functional-spec/FS-errors.md#38-the-one-grammar-wording-migration)) | the new reason alone |
| A wildcard subject joined by `and` | the conjunction refusal, then the wildcard reason appended ([§FS-errors.3.8](../../functional-spec/FS-errors.md#38-the-one-grammar-wording-migration)) | the wildcard reason first |

The first two only turn a refusal into an answer, so they need no window. The rest keep what they printed and add the notice, and [§RM-one-grammar-flip](../../roadmap.md#rm-one-grammar-flip-end-the-one-grammar-window-in-0190) spends the window. The documented refusal rows of [§FS-rule-refusals.sentences](../../functional-spec/FS-rule-refusals.md#sentences-strict-refusals) and [§FS-rule-refusals.selectors](../../functional-spec/FS-rule-refusals.md#selectors-a-refused-selector-is-answered-with-a-selector) keep their bytes (D10): `Each POLICY …` still prints `unknown kind "POLICY"`.

## alternatives: Alternatives refused

- **One accepted kind set for every command.** D18 refused it: at a workspace root `list --kind` accepts every loaded project's kinds and a root rule's subject only the root's.
- **A named-sections flag for `list` alone.** It would close G05 and leave the next reader to diverge again.
- **Judging a pinned target a second time at check time.** Load and check would then judge one target twice, against [§AR-workspace.5.2](../../architecture/AR-workspace.md#52-validation-runs-once-at-the-right-layer).
- **Moving the refusal order outright.** A different first refusal is a change of bytes users match (D20), so it takes the window.

## consequences: Consequences

Rows G03, G04 and G05 leave [§REQ-one-answer.differences](../../requirements/REQ-one-answer.md#differences-the-differences-known-when-this-requirement-landed), and [§REQ-one-answer.identity.kinds](../../requirements/REQ-one-answer.md#identitykinds-every-command-reads-one-kind-verdict) and [§REQ-one-answer.sections.named](../../requirements/REQ-one-answer.md#sectionsnamed-every-command-reads-a-kinds-named-sections-under-its-own-project) hold what they promised, proved by `crates/grund-cli/tests/kind_verdict_agreement.rs`. A workspace can now state in config what it could only state as a rule. Two config shapes that can never match a citation start warning, and become load errors in `0.19.0`.

## release-note: Release note

- [§FS-scan.projects.kinds](../../functional-spec/FS-scan.md#projectskinds-whose-kinds-a-run-reads), [§FS-config.3.9.5.2](../../functional-spec/FS-config.md#3952-a-pinned-target-is-judged-against-its-projects-kinds), [§FS-errors.2.5.1](../../functional-spec/FS-errors.md#251-a-kind-that-declares-no-ids), [§FS-errors.3.8](../../functional-spec/FS-errors.md#38-the-one-grammar-wording-migration), [§FS-rule-refusals.sentences.order](../../functional-spec/FS-rule-refusals.md#sentencesorder-a-sentence-is-refused-for-the-first-stage-it-fails): **every command now reads one kind verdict in its own scope, and a rule sentence is refused for the first stage it fails.** A `[citations]` target pinned to a member, `api/API`, or to any project, `*/API`, is judged against that project's kinds: it loads and is enforced where it was refused as `an unknown target kind`. `list --selector API.terms` at a workspace root reads named sections under the project that configures `API`, so it answers whatever `grund api/API-a.terms` answers. **What warns until 0.19.0:** a pinned or `*/` target its project does not have citable, or one naming an alias the workspace does not load, loads with ``warning: grund.toml: [citations.FS] must names an unknown target kind `FS` in namespace `api`; this becomes an error in grund 0.19.0`` and its siblings, and a member's own run warns in the same way on a target pinned to another project; from 0.19.0 each is a load error, exit `2`. **What is appended until 0.19.0:** a non-citable kind on `list --selector` or in a rule keeps its reason, `unknown kind "e2e"` or ``literal subject "e2e.terms" does not match the configured ID grammar``, and gains ` — kind "e2e" declares no IDs — tests/e2e/ is not a citable home; this wording changes in grund 0.19.0`; a section-component wildcard subject joined by `and`, `FS-x.* must cite at least one FS and must not cite any FS.`, keeps its conjunction refusal and gains ` — section-component wildcards are not accepted in phase 1; this wording changes in grund 0.19.0`. **What does not move:** exit codes on every refusal, the `invalid-rule` code, `list --kind` and `grund id` text, every documented refusal row, and `--project` narrowing. **Who this breaks:** a script that matches the exact text of a refusal above, or that expected exit `2` from loading `api/API` or from `list --selector API.terms`. Closes [issue #599](https://github.com/agent-grounds/grund/issues/599).

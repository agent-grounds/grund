# DF-directions-only-from-rules: The Citation directions section renders only the rules a config declares

**Date:** 2026-10-28

Settles what the managed block renders for a config that declares no citation rule, as [§FS-init.2.3.4.10](../../functional-spec/FS-init.md#23410-citation-direction) specifies. Refines [§DF-directions-render](DF-directions-render.md#df-directions-render-the-citation-directions-wording-is-chosen-once-against-a-canonical-config), whose wording a config with rules keeps byte for byte. Serves [§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work): every line under the heading is a rule `grund check` enforces.

## status: Accepted

## 1. Context

A config that declared kinds and no `[citations]` got a `### Citation directions` section holding one fixed sentence: specs cite goals, architecture cites specs, code and executable tests cite the specs they realize, and "a citation rule array" reads a certain way, with a link to the citation-directions guide. [§FS-init.2.3.4.10](../../functional-spec/FS-init.md#23410-citation-direction) kept it on purpose, so that a block stayed stable for configs that predated `[citations]`. In a workspace with one `GRUND` kind, the user asked why the rules section had no rules, and it took three scratch configs to show that nothing was missing from theirs (agent-grounds/grund#1075). The sentence named kinds the project did not have and taught a convention its config did not hold and `check` did not enforce.

`kinds = []` already replaced the sentence with "No kind is declared yet, so no citation direction applies …" ([§FS-init.2.3.4.4.2](../../functional-spec/FS-init.md#23442-no-kinds-yet)), so the spec never treated it as fixed text. A version-2 scoped block with a grounding ladder and no rules rendered the legend, its grounding sentence and `Anything not listed above is allowed.`, a closing line with no list above it. Version 1 and version 2 rendered the same, because both read one "rules declared" flag that a grounding ladder alone does not set.

## 2. Decision

The section states the rules a config declares and no others ([§FS-init.2.3.4.10](../../functional-spec/FS-init.md#23410-citation-direction)). A config that declares none renders no section, at the root and in every scoped block, whatever kinds it declares. Where a row's effective grounding lands at a destination, that destination's section is the heading and its grounding sentence alone ([§FS-init.2.3.5.7](../../functional-spec/FS-init.md#2357-the-grounding-sentence)), because whether a file must ground at all is not a direction rule. The root block is rendered from the root's own config and never from a member's ([§FS-init.2.3.8](../../functional-spec/FS-init.md#238-substituted-content)), so a workspace root without rules renders no section even when its members declare some. A config with rules keeps the render of [§DF-directions-render](DF-directions-render.md#df-directions-render-the-citation-directions-wording-is-chosen-once-against-a-canonical-config), legend and closing line included. No `grund check` verdict over a repository's files moves: only the text an agent reads, under the block-version pairs it shares ([§FS-init.2.3.7.1](../../functional-spec/FS-init.md#2371-the-current-version)).

The climbing convention the sentence carried moves to the citation-directions guide as a recommended starting ruleset, which `grund guide citations` prints from the installed binary ([§FS-init.5](../../functional-spec/FS-init.md#5-the-guide)). A block for a config with rules never linked the guide either.

### 2.1 The grounding-only section is the sentence alone

The legend exists because a bullet's verb is unreadable without it ([§FS-init.2.3.5.1](../../functional-spec/FS-init.md#2351-layout)). This section has no bullets. The grounding sentence renders only for a `must` rung, a `grund check` error, so its "must" means what it says in plain English. A legend about `should`, `avoid` and `never`, none of which appear, would be the same noise this record removes. The closing line reports the global default for rules listed above it ([§FS-init.2.3.5.6](../../functional-spec/FS-init.md#2356-the-closing-line)), and none is.

### 2.2 `kinds = []` loses its paragraph

It declares no rule, so it renders what every config without rules renders. Keeping "no direction applies" there, while a config with one kind and no rules says nothing, would give two configs with equally no rules two different answers. The Project map paragraph still teaches the first kind ([§FS-init.2.3.4.4.2](../../functional-spec/FS-init.md#23442-no-kinds-yet)).

## 3. Alternatives considered

| Option | Why rejected |
|---|---|
| Keep the static sentence for configs without rules | The defect itself: a section that reads as a statement of the rules, holding a convention the config does not hold. A stable block is not worth a wrong one, and the version pairs this shares already report every block outdated once. |
| Open the grounding-only section with the legend, as a scoped block did | Explains levels no line in the section uses, the noise this record removes ([§DF-directions-only-from-rules.2.1](DF-directions-only-from-rules.md#21-the-grounding-only-section-is-the-sentence-alone)). |
| Keep `kinds = []`'s "No kind is declared yet, so no citation direction applies" paragraph | Two configs with equally no rules would get two different answers ([§DF-directions-only-from-rules.2.2](DF-directions-only-from-rules.md#22-kinds---loses-its-paragraph)). |
| A one-line pointer such as "No citation rules are declared; see the guide" | That pointer is the empty rules section this record removes. The guide is one command away as `grund guide citations`, offline. |

## release-note: Release note

- A config that declares kinds but no citation rules (`[citations]` in version 1, `[rules.citations]` in version 2) no longer gets a `### Citation directions` section with the fixed "Specs cite goals, architecture cites specs, …" sentence: the section renders only the rules a config declares, or, where grounding is on, the grounding sentence alone. `kinds = []` drops its "No kind is declared yet" paragraph the same way, and a scoped block with grounding but no rules drops its legend and closing line. The guide stays available as `grund guide citations`. This shares the v43/v44 and v45/v46 pairs this release already moves to. Existing blocks are reported outdated by `grund check` and `grund init --check`; run `grund init` to refresh them, preserving unmanaged text. No verdict over the repository's own files changes.

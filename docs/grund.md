# GRUND-grund: agents stay grounded in the spec

**Keep agents grounded in the spec — fewer bugs, cheaper LLM context, faster onboarding.** On a long-lived codebase with many humans and AIs, the spec drifts: citations rot, decisions get forgotten, e2e tests prove the wrong things, and "the why" lives in someone's head until that someone moves on. Each agent has limited context, and without a shared, mechanically-checked reference frame, knowledge silently fragments.

The remedy is to write the why down as a structure and hold every change to it. Projects that want one rarely keep one, because structure is hard twice over: the shape of what a project knows is hard to define and to keep ([§GRUND-schema](grund.md#grund-schema-the-shape-of-a-projects-knowledge-is-hard-to-define-and-to-keep)), and so are the rules that link it — above all the links between code and the text that says why ([§GRUND-links](grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)). `grund` makes both cheap: `grund.toml` declares the schema and the rules as data, and `grund check` holds every edit to them in prose and source alike. A Markdown link checker sees no kinds, no rules, and no `§FS-<events>.4` cited from `src/bus.rs`.

## 1. What grund does about it

`grund` owns the scheme end to end: the IDs and citation grammar, the config in `grund.toml`, and a scan of every `.md` file and every source file under the configured scan roots ([§FS-config.3.5](functional-spec/FS-config.md#35-scan--what-gets-scanned); [§AR-scanner.4](architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)). It serves [§GOAL-agent-grounding](goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) — the headline goal that every other goal exists in service of — and the mechanisms that make it viable: [§GOAL-no-dangling-refs](goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration), [§GOAL-fast-feedback](goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible), [§GOAL-friendliness-first](goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible), and [§GOAL-polyglot-citation](goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful).

## 2. Who it is for

- **Codebases that adopt the specification-driven design** — to verify the spec stays intact across changes, *and across the docs/code boundary*. That design has a current wave of agent tools ([§REL-spec-driven-development](related-work/REL-spec-driven-development.md#rel-spec-driven-development-spec-driven-development-the-current-wave-of-agent-tools)) and an older line of requirements-traceability tools beside `grund` ([§REL-traceability-tools](related-work/REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund)).
- **Polyglot projects** whose specs are cited from source as well as docs — the case off-the-shelf link checkers cannot serve.
- **Agents (human and AI) working in those codebases** — to retrieve grounded spec content cheaply.
- **CI systems** — as a fast pre-merge check.

If a project does not use a grund-style ID scheme, `grund` has nothing to offer it. We deliberately do not generalize.

# GRUND-schema: the shape of a project's knowledge is hard to define and to keep

A project's memory holds facts of different kinds — why, goals, behavior, design, decisions, proofs — each wanting its own home, addresses, and sections. No one shape fits every project, none knows its shape up front, and the shape keeps changing: a kind splits, a home moves, a heading is reworded. Kept by hand, a schema is a README convention every new file may ignore, and every reshaping breaks the links into the old shape.

## 1. Declaring it

`grund` makes the schema data: a `[[kinds]]` row names a kind, its home, and its title ([§FS-config.3.4](functional-spec/FS-config.md#34-kinds--recognized-kinds)), a declaration gives a fact a stable ID, numbered headings address its sections ([§FS-config.3.3.2](functional-spec/FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)), and a rule declaration states a convention about the shape as one sentence the check applies ([§FS-rules](functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules)). Nothing has to be declared up front: an unconfigured tree gets the default kinds, and the prose around the declarations stays free, formalized one spot at a time where it pays ([§REL-incremental-formalization](related-work/REL-incremental-formalization.md#rel-incremental-formalization-incremental-formalization-the-idea-grund-descends-from), [§REL-flexiformality](related-work/REL-flexiformality.md#rel-flexiformality-flexiformality-documents-that-are-partly-formal), [§REL-schema-later](related-work/REL-schema-later.md#rel-schema-later-pay-as-you-go-structure-schema-added-where-it-pays)).

## 2. Keeping it

An address does not depend on where its fact lives: `§FS-<user-login>.3.1` survives a moved file and a reworded heading, where a Markdown anchor breaks, and `grund FS-<user-login>.3.1` returns that one section instead of its file ([§GOAL-friendliness-first.1](goals.md#1-hard-requirements)). A fact outside its kind's home, a duplicate ID, or a heading at the wrong depth fails the check, and when the shape itself changes, `grund refs` names every site the change touches.

# GRUND-links: the cross-linking rules are hard to define and to hold

**The links that matter most cross between code and text.** Code says what happens and prose says why, and the two live apart — in different files and languages, edited by different hands at different times. A compiler does not read the prose and a link checker does not read the code, so nothing on either side sees the edge between them. So everything in the project — code, docs, decisions, tests — cites the point that says why it is the way it is: a behavior on its doc-comment, a clause inline beside the line that enforces it. The why holds for the work as it stands, readable in place, never reconstructed from git history or someone's memory.

Which other edges a project needs is its own call: here a spec should cite a goal and a black-box test should not read the design. Kept by hand, these rules live in a reviewer's head, and the edges rot silently.

## 1. Declaring the rules

`grund` makes the rules data. Citation directions say which kind must, should, should not, or must not cite which ([§DF-citation-directions](decisions/functional/DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels), [§FS-config.3.9](functional-spec/FS-config.md#39-citations--citation-direction-rules)), and `require_grounding` on the source tree makes the edge from code to text an obligation rather than a habit: every source file, or every unit in it, must carry a citation that resolves ([§FS-config.3.9.2.3](functional-spec/FS-config.md#3923-grounding-the-source-tree-and-nothing-else)).

## 2. Holding every edit to them

One citation grammar is read through one resolver in Markdown, Javadoc, Rustdoc, Python docstrings, Go blocks, and JSDoc ([§GOAL-polyglot-citation](goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)). A dangling reference, a broken section coordinate, an edge a `must-not` rule forbids, or one a `must` rule requires fails the build ([§FS-check.3](functional-spec/FS-check.md#3-errors-detected)), fast enough to run on every save ([§GOAL-fast-feedback](goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).

# DF-chapter-rules: chapter rules are grounded controlled-English declarations over producer-neutral facts

**Status:** Accepted
**Date:** 2026-09-19

## 1. Context

A repository can already constrain citation directions per kind, but it cannot
require a chapter, constrain citations inside one named chapter, or require one
chapter to cite every declaration of a kind exactly once. Those conventions
therefore live as unchecked prose even though the declaration, chapter, and
citation graph already contains the facts needed to verify them. This leaves
the agent-grounding loop incomplete ([§GOAL-agent-grounding](../../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work)) and makes a project's
more precise conventions less configurable than its kind-wide ones
([§GOAL-configurable](../../goals.md#goal-configurable-every-default-is-overridable)).

The feature also needs one authored form that a person can read, an agent can
follow, and `grund check` can evaluate. If those are separate representations,
their translations can drift. If the evaluator depends directly on Markdown or
scanner records, a second fact producer or evaluation strategy requires a
redesign rather than a replacement.

## 2. Decision

Rules are declarations of any citable kind whose `[[kinds]]` row opts in with
`rules = true`. The declaration title is one sentence in grund's strict,
closed controlled-English grammar; the declaration body is its ordinary,
non-empty rationale. The same exact sentence is parsed, rendered into managed
agent guidance, documented, and tried through the CLI. SBVR is informative
heritage only, while the relational clauses in
[§FS-rules.5](../../functional-spec/FS-rules.md#5-relational-meaning) are the
normative meaning.

Phase 1 admits only the five sentence families and local declaration or named-
chapter subjects in [§FS-rules.3](../../functional-spec/FS-rules.md#3-the-five-sentence-families). The surface is closed: one semantic verb per
sentence, one canonical spelling per meaning, a terminal period, and
case-sensitive fixed words. Broader selectors, settings, exceptions,
definitions, program facts, and a Datalog runtime are deferred rather than
reserved as partially implemented syntax.

`[citations]` remains supported and unmigrated. It deduplicates with a rule only
where both express the same existing kind-wide citation constraint; the
existing config finding wins byte-for-byte. Rules-only duplicates instead name
their sorted rule IDs. Thus opting in adds the feature without perturbing an
existing repository's public report bytes
([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)).

The boundary is producer-neutral. A sentence front end emits only a
`ParsedRule`; fact producers emit only a complete, immutable, versioned
`RuleFacts`; and the logic engine evaluates those two representations. The
parser knows no facts or findings, and the engine knows no sentences, Markdown,
scanner records, or file layout beyond stable fact anchors. The next
architecture step records the component placement and dependency tests; this
decision fixes what must cross the boundary, not which files implement it.

## 3. Alternatives rejected

- **Rules as TOML rows.** A table is not a declaration title or an instruction,
  so the checked form and the rendered form would differ.
- **Datalog as the authoring language.** It states the semantics precisely but
  is not the friendly instruction surface
  [§GOAL-friendliness-first](../../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) requires.
- **A general SBVR parser.** It imports an external standard and synonyms that
  the release does not promise; grund's own five productions are the contract.
- **Rules in config.** One malformed title should be a located finding at its
  declaration rather than an invalid config that prevents every command from
  scanning the repository.
- **Evaluator access to scanner structures.** That would make the Markdown
  producer the logical model and defeat replacement by a second producer.
- **Path subjects, settings, and exceptions in phase 1.** None is needed for
  the three chapter-rule asks, and accepting their syntax would freeze behavior
  before their independently tracked decisions are made.

## 4. Consequences

Repositories opt in explicitly and keep schema version 1. A bad configured
rule is a located `invalid-rule`; a bad `check --rule` sentence is a pre-scan
invocation error. Closed-world absence and count conclusions are made only from
a complete snapshot. Documentation, examples, shipped skills, managed agent
guidance, CLI text/JSON, and LSP diagnostics are held to one grammar and one
shared report by [§FS-rules](../../functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules).

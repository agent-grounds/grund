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

A chapter subject's `NAME` is the chapter's whole path from its declaration
([§FS-rules.2.1](../../functional-spec/FS-rules.md#21-a-chapters-name-is-its-whole-path)). Through grund 0.16.1 the selector compared only a path's last
component with it, so a dotted `NAME` such as `requirements.terms` selected
nothing on any surface: `list --selector` and `--size` printed no row, a
positive citation rule reported every declaration of its kind
`unreached-declaration`, and a prohibition passed without reading a chapter.
A one-component `NAME` reached nested chapters in `list` that `check` never
selected. Matching the whole path moves verdicts in one direction that can
fail a run: a dotted prohibition, or a dotted positive rule run with
`unreached-declaration` ignored, now evaluates the chapters it names. That is
not a break under [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise). The dotted subject had no
defined meaning, because [§FS-rules.5.2](../../functional-spec/FS-rules.md#52-family-clauses) joined `N` as one section component, and
on those runs it produced no output. A positive rule whose findings were
reported already failed every tree holding a declaration of its kind, so that
run can only move toward passing, and no verdict over a one-component `NAME`
moves.

## release-note: Release note

- [§FS-rules.2.1](../../functional-spec/FS-rules.md#21-a-chapters-name-is-its-whole-path), [§FS-rules.5.2](../../functional-spec/FS-rules.md#52-family-clauses), [§FS-rules.8](../../functional-spec/FS-rules.md#8-command-surfaces): **a chapter subject's `NAME` is the
  chapter's whole path.** `grund list --selector FS.requirements.terms`, the
  sentence `The requirements.terms chapter of each FS`, `--size`, and
  `FS:requirements.terms` under a `:` separator list the nested
  `requirements.terms` chapter where they listed nothing, and `check` evaluates
  a rule over that subject on the chapters that exist where it reported every
  FS `has no requirements.terms chapter`. A one-component `NAME` selects only a
  chapter directly under its declaration in `list`, as it already did in
  `check`: `list --selector FS.terms` no longer lists a nested
  `requirements.terms`. A nested chapter's row is titled by its display name,
  `Terms`, where an exact `FS-login.requirements.terms` row repeated the
  heading's `requirements.terms:` coordinate
  ([§FS-rules.5.1.1](../../functional-spec/FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)). **Who this breaks:** a script that read nested chapters
  through a one-component `list --selector` or `--size` `NAME`, or matched a
  nested chapter row's title; a dotted
  prohibition such as `The requirements.terms chapter of each FS must not cite
  any AR.`, which passed without reading a chapter and now reports the sites it
  forbids; and a run that ignored `unreached-declaration` over a dotted positive
  rule, which now reports the chapters that miss their citation. No deprecation
  window is owed
  ([§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)):
  the dotted subject had no defined meaning, and the only `must` runs it let
  pass are those two, which read no chapter at all. Closes [issue #511](https://github.com/agent-grounds/grund/issues/511).

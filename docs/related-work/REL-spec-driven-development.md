# REL-spec-driven-development: spec-driven development, the current wave of agent tools

Spec-driven development has a coding agent write a specification before the code and work
from it. It is the current wave of the design whose codebases grund is for
([§GRUND-grund.2](../grund.md#2-who-it-is-for)).

## work: What the work is

- GitHub Spec Kit ([github/spec-kit](https://github.com/github/spec-kit)), v1.1.3
  (2026-10-09), checked 2026-10-13, is a toolkit for spec-driven development. Its
  [specification template](https://github.com/github/spec-kit/blob/main/templates/spec-template.md)
  numbers requirements `FR-001` on and success criteria `SC-001` on, afresh in each spec, and
  marks what the spec leaves open with `[NEEDS CLARIFICATION: …]`; its
  [tasks template](https://github.com/github/spec-kit/blob/main/templates/tasks-template.md)
  numbers tasks `T001` on. Its
  [analysis command](https://github.com/github/spec-kit/blob/main/templates/commands/analyze.md)
  is a prompt the user's coding agent runs, and its report is advisory. Its experimental
  [MCP server](https://github.github.com/spec-kit/reference/mcp.html)
  "supports only the dotted command `version`".
- Kiro ([kiro.dev](https://kiro.dev/)), AWS's agentic development environment, turns
  prompts into requirements, architectural designs and sequenced tasks, kept as
  `requirements.md`, `design.md` and `tasks.md` ([specs](https://kiro.dev/docs/specs/)). It
  is proprietary: the IDE and CLI are licensed as ["AWS Content"](https://kiro.dev/license/),
  and [kirodotdev/Kiro](https://github.com/kirodotdev/Kiro) is an issue tracker, not its
  source. IDE 1.2.56 (2026-10-08), CLI 2.30.0 (2026-10-12) ([changelog](https://kiro.dev/changelog/)),
  checked 2026-10-13. [Steering](https://kiro.dev/docs/steering/) files reference files by
  relative path and line range, and [headless mode](https://kiro.dev/docs/cli/headless/)
  needs an account's API key.
- OpenSpec ([Fission-AI/OpenSpec](https://github.com/Fission-AI/OpenSpec)), v1.14.1
  (2026-10-06), checked 2026-10-13, has the repository description "Spec-driven development
  (SDD) for AI coding assistants". Its requirements are named by heading,
  `### Requirement: …` with `#### Scenario:` blocks beneath, not by ID, and the header is
  the name a change matches, where
  ["archive matching is exact"](https://openspec.dev/docs/cli#openspec-show), so a rename
  needs a `RENAMED` delta. Its
  [CLI](https://github.com/Fission-AI/OpenSpec/blob/main/docs/cli.md) `validate` checks the
  Markdown under `openspec/` "for structural issues" and exits 1 on a failure, and
  `show -r <n>` returns one requirement by position. Outside CI, the CLI reports command
  names and its version unless the user
  [opts out](https://github.com/Fission-AI/OpenSpec/blob/main/docs/faq.md).
- specre ([yoshiakist/specre](https://github.com/yoshiakist/specre)) is "a minimal
  specification format and toolkit for Spec-Driven Development (SDD)": each specre is one
  Markdown file "describing exactly one behavior", whose front matter carries a ULID `id`.
- SpecMine (Shyam Agarwal, Anmol Singhal, Travis Breaux and Bogdan Vasilescu, *SpecMine: A
  Large-Scale Corpus of Spec-Driven Development Artifacts*, 2026,
  [arXiv:2608.25202](https://arxiv.org/abs/2608.25202), not peer-reviewed) collects "470,795
  files across 73,030 repositories, attributed to 17 named tools".
- Jessica Diaz, Joaquin Gayoso, Andrea Cimminio and Jorge Perez, *Spec-Driven Development
  for Agentic Software Engineering: Harnessing Human-Agent Teamwork*, 2026
  ([arXiv:2609.00252](https://arxiv.org/abs/2609.00252), not peer-reviewed), is conceptual:
  it states that "peer-reviewed evidence ... not yet established" and draws mainly on
  gray literature.

## agrees: Where grund agrees

- The specification comes first and the code is held to it; grund exists for codebases
  that work this way ([§GRUND-grund.2](../grund.md#2-who-it-is-for)).
- A requirement gets a stable identifier code can point back at, as Spec Kit's numbers,
  within one spec, and specre's ULIDs do ([§GRUND-schema.2](../grund.md#2-keeping-it)).

## departs: Where grund departs

- grund writes no specification and drives no agent's workflow. It checks and retrieves,
  and leaves the work to other tools ([§FS-non-goals.7](../functional-spec/FS-non-goals.md#7-inter-agent-messaging-or-workflow)).
- Partial formality is grund's subject. As grund reads these tools, none of them gives an
  account of what the unmarked prose of a specification means to a checker; that reading
  is grund's position, not something the tools state. In grund, prose stays free, only
  what is marked is checked ([§DF-reference-marker.2.4](../decisions/functional/DF-reference-marker.md#24-strict-vs-optional)), and the lineage of that idea is
  incremental formalization ([§REL-incremental-formalization](REL-incremental-formalization.md#rel-incremental-formalization-incremental-formalization-the-idea-grund-descends-from)).
- Spec Kit's analysis is a prompt an LLM runs, so its verdict is not a function of the
  files; grund's is, and two installs of one version must agree on it ([§FS-non-goals.13](../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).

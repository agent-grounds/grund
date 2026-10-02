# REL-spec-driven-development: spec-driven development, the current wave of agent tools

Spec-driven development has a coding agent write a specification before the code and work
from it. It is the current wave of the design whose codebases grund is for
([§GRUND-grund.2](../grund.md#2-who-it-is-for)).

## work: What the work is

- GitHub Spec Kit ([github/spec-kit](https://github.com/github/spec-kit)), from 2025, is a
  toolkit for spec-driven development. Its specification template numbers requirements
  per spec, `FR-001` and on, and marks what the spec leaves open with
  `[NEEDS CLARIFICATION: …]`.
- Kiro ([kiro.dev](https://kiro.dev/)), AWS's agentic development environment, turns
  prompts into requirements, architectural designs and sequenced tasks, kept as
  `requirements.md`, `design.md` and `tasks.md`.
- OpenSpec ([Fission-AI/OpenSpec](https://github.com/Fission-AI/OpenSpec)), from 2025, is
  "Spec-driven development (SDD) for AI coding assistants". Its requirements are named by
  heading, `### Requirement: …` with `#### Scenario:` blocks beneath, not by ID.
- specre ([yoshiakist/specre](https://github.com/yoshiakist/specre)), from 2026, is "a
  minimal specification format and toolkit for Spec-Driven Development (SDD)": each specre
  is one Markdown file "describing exactly one behavior", whose front matter carries a
  ULID `id`.
- SpecMine (Shyam Agarwal, Anmol Singhal, Travis Breaux and Bogdan Vasilescu, *SpecMine: A
  Large-Scale Corpus of Spec-Driven Development Artifacts*, 2026,
  [arXiv:2608.25202](https://arxiv.org/abs/2608.25202)) collects "470,795 files across
  73,030 repositories, attributed to 17 named tools".

## agrees: Where grund agrees

- The specification comes first and the code is held to it; grund exists for codebases
  that work this way ([§GRUND-grund.2](../grund.md#2-who-it-is-for)).
- A requirement gets a stable identifier code can point back at, as Spec Kit's numbers
  and specre's ULIDs do ([§GRUND-structure](../grund.md#grund-structure-the-projects-long-term-memory-stays-organized)).

## departs: Where grund departs

- grund writes no specification and drives no agent's workflow. It checks and retrieves,
  and leaves the work to other tools ([§FS-non-goals.7](../functional-spec/FS-non-goals.md#7-inter-agent-messaging-or-workflow)).
- Partial formality is grund's subject. As grund reads these tools, none of them gives an
  account of what the unmarked prose of a specification means to a checker; that reading
  is grund's position, not something the tools state. In grund, prose stays free, only
  what is marked is checked ([§DF-reference-marker.2.4](../decisions/functional/DF-reference-marker.md#24-strict-vs-optional)), and the lineage of that idea is
  incremental formalization ([§REL-incremental-formalization](REL-incremental-formalization.md#rel-incremental-formalization-incremental-formalization-the-idea-grund-descends-from)).

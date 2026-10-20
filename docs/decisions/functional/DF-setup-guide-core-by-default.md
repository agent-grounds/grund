# DF-setup-guide-core-by-default: grund agent-setup-instructions prints a core that opens with the fresh path, and prints each reference section on request

**Date:** 2026-10-20

## status: Accepted

## 1. Context

`grund agent-setup-instructions` printed the whole setup skill in one response: 58,566 bytes and 738 lines at 9d78aaaf38, against 3,221 bytes for `grund init -h`. It held the adoption workflow, every config question, the language examples, the editor and integrations setup, and verbatim copies of two reference pages, the citation-directions page (5.7 KB) and the chapter-rules writing guide (12.3 KB). `grund init --scaffold`, the one command a fresh repository needs, first appeared some 75 lines in. An agent that followed the top-level help to set up an empty repository read all of it, and a guide that size overflowed the output destination it was printed into ([grund#830](https://github.com/agent-grounds/grund/issues/830)).

The output satisfied the specification as written: the command took no arguments, printed the skill byte for byte, and carried both copies inline. The defect was against [§GOAL-token-economy](../../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), which asks for the smallest slice that answers the question and an explicit way to ask for more. Every remedy that cut the default output materially moved a contract, and the one that moved none, rewording the help, left the output at 58 KB.

## 2. Decision

The skill becomes an Agent Skills directory, `skills/grund-init/`, and the command maps onto it one file per call ([§FS-init.5.3](../../functional-spec/FS-init.md#53-one-directory-two-surfaces)):

- With no flag it prints the core, `SKILL.md`, which is at most 16,000 bytes ([§FS-init.5.5](../../functional-spec/FS-init.md#55-the-core-and-its-sections)).
- `--section NAME` prints `references/NAME.md`, one of a closed set of ten sections, and an unknown name is refused with the list of known ones ([§FS-init.5](../../functional-spec/FS-init.md#5-agent-setup-instructions)).
- `--full` prints the core and then every section.

The core opens with *Fresh repository*: the three commands a fresh repository runs, in order, and what success looks like. *Contents* follows, naming every section and how to print it, and then the adoption *Workflow* in its existing order ([§FS-init.5.1](../../functional-spec/FS-init.md#51-what-the-instructions-tell-the-agent)). Each reference section is one of the old guide's `##` sections moved whole, except `config` and the marked copies. `config` holds two of them, *Recommendation Heuristics* and then *Config Questions* without `[citations]`. The two marked copies become sections of their own, so neither copy is in the default output.

The flag vocabulary is `grund show`'s: the cheap read by default, `--section` for one part, `--full` for all of it ([§DF-show-default-token-cheap](DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in)). The unknown-name refusal takes the shape `grund integrations` gives an unknown client, a `known …:` line after the error ([§FS-integrations](../../functional-spec/FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations)).

### 2.1 Leading with the fresh path does not amend the adoption decision

[§DF-skill-init-existing-specs](DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding) already decides this case. Its rule table gives a fresh repository with no meaningful docs or specs `grund init --scaffold` ([§DF-skill-init-existing-specs.4](DF-skill-init-existing-specs.md#4-decision-rule)). The fresh path still begins with the inventory that record requires: "no `grund.toml`, nothing to adopt" is what the inspection concludes. It still asks the user once before it writes. What it skips is the per-option interview, which exists to map existing artifacts and to choose settings other than the defaults, and a fresh repository has nothing to map and takes every default.

### 2.2 `--full` is the whole guide, not the old bytes

`--full` prints the same text the single file held, in a new order: the core, which now opens with *Fresh repository* and *Contents*, then the sections in Contents order, one blank line between files. Sections the old file interleaved with the adoption guidance, the config questions among them, now follow the core instead. So `--full` restores the content and the one-response shape, but it is not an alias for the old default, and a caller that compared bytes against the old output sees a difference.

### 2.3 The bindings return the whole guide

Python `agent_setup_instructions()` and Node `agentSetupInstructions()` keep their signatures and return the `--full` text ([§FS-distribution.3.2.1](../../functional-spec/FS-distribution.md#321-operations-options-and-refusals), [§FS-distribution.3.3.5](../../functional-spec/FS-distribution.md#335-complete-initial-inventory)). A program that receives the guide as data chooses what of it reaches a model's context, so no binding caller loses content. A `section` argument for the bindings is a separate request.

## 3. Rejected alternatives

- **Link the reference pages instead of embedding them.** An agent that has only the installed binary could not follow the links, and the command exists for exactly that agent ([§FS-init.5](../../functional-spec/FS-init.md#5-agent-setup-instructions)).
- **A positional topic, `grund agent-setup-instructions rules`.** grund already spells "one part of a document" as `--section` on `show`, and a second spelling is one more thing to learn. A positional is refused with a message of its own, so a caller who guessed this form is told it is not one.
- **Move the adoption workflow into a section too.** The default would then fit in about 3 KB, but the command's main audience is the agent adopting a repository that already has specs, the reader the README sends here, and it would need two calls to start.
- **Reword the help and keep the output.** It moves no contract and fixes nothing: the output stays at 58 KB.

## 4. Compatibility

The default output changes, and no verdict moves. No tree passes or fails differently, no finding is added or removed, and the exit codes keep their meaning: `0` printed, `2` a CLI-level error ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)).

- **The default prints the core.** An agent or script that expected the whole guide now gets the core. It loses nothing it cannot ask for: Contents names every section, and `--full` restores the whole guide with one flag, in the order of [§DF-setup-guide-core-by-default.2.2](DF-setup-guide-core-by-default.md#22---full-is-the-whole-guide-not-the-old-bytes). No alias keeps the old bytes. Before `1.0`, a surface may change without one where carrying it would cost more than the change ([§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)), and an alias here would keep the 58 KB response as the thing an agent gets by default, the cost this decision removes.
- **A rejected flag reads `unknown flag`.** A caller that matched `agent-setup-instructions takes no arguments` on stderr sees ``unknown flag `<flag>` `` for a flag, or `takes no positional arguments` for a positional, with the same exit `2` ([§FS-cli.3](../../functional-spec/FS-cli.md#3-cross-subcommand-flags)).
- **`--section` and `--full` now succeed.** They were exit-`2` errors, and a refusal that becomes an answer moves no verdict.

No release note is required for a change that moves no verdict ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)). This record carries one all the same, so the changed default is announced in the release rather than discovered.

## release-note: Release note

- [§DF-setup-guide-core-by-default](DF-setup-guide-core-by-default.md#df-setup-guide-core-by-default-grund-agent-setup-instructions-prints-a-core-that-opens-with-the-fresh-path-and-prints-each-reference-section-on-request), [§FS-init.5](../../functional-spec/FS-init.md#5-agent-setup-instructions), [§FS-init.5.5](../../functional-spec/FS-init.md#55-the-core-and-its-sections): **`grund agent-setup-instructions` prints a core of at most 16,000 bytes, and each reference section on request.** The core opens with the three commands a fresh repository runs, then a table of contents, then the adoption workflow. `--section NAME` prints one of ten reference sections (`config`, `citations`, `languages`, `members`, `scoped-guidance`, `schema`, `queries`, `editor`, `integrations`, `rules`), and `--full` prints the whole guide in one response, in a new order. **Who this affects:** a caller that read the whole guide from the default output gets the core, and passes `--full` for the rest. A caller that matched `takes no arguments` on stderr sees `unknown flag` for a flag or `takes no positional arguments` for a positional, still with exit `2`. No verdict moves.

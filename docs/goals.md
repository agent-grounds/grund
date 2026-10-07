# Goals

Goals state direction. Measurement details live in specs, e2e, CI, and benchmark pages; the map is [AR-goal-measurement](architecture/AR-goal-measurement.md#ar-goal-measurement-goal-and-requirement-meters-live-outside-goals).

## GOAL-agent-grounding: agents stay cited as they work

Keep specs, decisions, tests, and code cited while work happens — in the shape the project declared ([§GRUND-schema](grund.md#grund-schema-the-shape-of-a-projects-knowledge-is-hard-to-define-and-to-keep)), along links the check holds to the project's rules, above all the links between code and the text that says why ([§GRUND-links](grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)). This is the headline outcome from [§GRUND-grund](grund.md#grund-grund-agents-stay-grounded-in-the-spec); every other goal exists to keep that loop correct, cheap, and easy.

### 1. The three layers

Instruction (`init` writes agent entrypoints), verification at rest (`check`), and diff-gated co-change enforcement (`cover` plus recipe).

### 2. What "grounded" requires of a diff

New declarations, code, decisions, and e2e cases cite the most-specific ID they realize or prove.

### 3. What this rules out

No guessed citations, separate lint surface, or hard-coded code-unit heuristic.

### 4. Composition with other goals

Grounding depends on resolving citations, fast checks, readable output, polyglot scan coverage, and zero-config defaults.

### 5. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

## GOAL-no-dangling-refs: every cited ID resolves to a declaration

A passing repo has zero dangling references and zero broken section coordinates in the text the run read — the declared, bounded blind spots of [§REQ-no-missed-citation.2](requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) included, which [§FS-check.1.3](functional-spec/FS-check.md#13-the-full-tree-scope---full)'s `--full` is the switch for. False negatives are bugs. This is the correctness floor under [§GRUND-links.2](grund.md#2-holding-every-edit-to-them): a citation an agent cannot trust grounds nothing.

### 1. What "resolves" means

A citation resolves when exactly one declaration of its ID exists, its section path exists, and any stub points at an inline declaration of the same ID. Two are an error, not a ranking ([§FS-declarations.checks.duplicate](functional-spec/FS-declarations.md#checksduplicate-duplicate-declaration)).

### 2. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

## GOAL-polyglot-citation: IDs cite cleanly from anywhere they are useful

One citation grammar works in Markdown and source comments, across docs/code boundaries, through one resolver. This is the reason `grund` is more than a Markdown link checker: the link between code and the text that says why ([§GRUND-links](grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)) has to hold in every language a line is written in.

### 1. What "cleanly" means

Same marker, same section grammar, same resolver, same line-located errors in every supported host language.

### 2. Why this is a goal, not a side effect

Markdown links degrade outside rendered Markdown; `§<ID>` citations must stay useful wherever implementation intent lives.

### 3. Composition with other goals

This is coverage; [§GOAL-no-dangling-refs](goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) is correctness.

### 4. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

## GOAL-fast-feedback: grund must be as fast as possible

Speed is an ordering principle. `grund` runs in editors, save loops, commits, and CI; anything slower than the loop gets routed around. A consistency check ([§GRUND-links.2](grund.md#2-holding-every-edit-to-them)) that is too slow to run on every save is one that stops being run.

### 1. Performance targets

- Under 100 ms on this repo.
- Under 1 s on a 10k-file repo.
- At most one allocation per file; zero on hot regex paths where possible.

### 2. How we get there

Linear scans, streaming reads, shared compiled regexes, skipped dead directories, and parallelism only when the simple path stops winning.

### 3. Measurable

See [AR-benchmarks](architecture/AR-benchmarks.md#ar-benchmarks-instruction-counting-benchmarks-for-the-hot-cli-commands), [AR-ci.5](architecture/AR-ci.md#5-benchmark-job), and [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

## GOAL-zero-config: works on any conformant tree

A canonical repo works with no config and no flags. Divergent layouts configure the difference; empty scans fail loud.

### 1. What "canonical layout" means

Root agent entrypoint, `grund.toml` when needed, `docs/`, `e2e/`, `src/`, configured kinds, and the canonical citation grammar.

### 2. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

### 3. Composition with [§GOAL-configurable](goals.md#goal-configurable-every-default-is-overridable)

Zero-config owns the default; configurability owns deliberate divergence.

## GOAL-multi-language: same engine, three platforms

Cargo, npm, and PyPI ship the same engine with idiomatic host bindings and byte-identical behavior. The grounding loop ([§GRUND-grund](grund.md#grund-grund-agents-stay-grounded-in-the-spec)) must hold wherever an agent works, so every host platform runs the same engine.

### 1. Identical behavior

Same tree plus same config produces the same report across bindings and supported operating systems.

### 2. Idiomatic surfaces

Rust returns `Result`, Node returns promises, Python raises exceptions. Behavior stays identical; APIs fit the host.

### 3. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

## GOAL-friendliness-first: as user- and agent-friendly as possible

Friendliness is an ordering principle beside speed: prefer output and workflows humans and agents can act on directly. Grounding ([§GRUND-links](grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)) only sticks if staying cited is the path of least resistance.

### 1. Hard requirements

- A finding at a site in the repository points at `path:line`; a message about the run itself names the run ([§FS-errors.2.2](functional-spec/FS-errors.md#22-cli-level-message)).
- JSON output has stable shapes.
- `grund <ID>` returns the smallest useful grounded read.
- Top-level help fits one screen.
- Frequent commands are one token after `grund`.
- Same input produces byte-identical output.
- Passing text `check` prints exactly `success`.

### 2. What this rules out

No configurable set of severity levels, report ordering, exit-code mapping, hidden prompts, or extra verbs in frequent workflows ([§FS-config.6](functional-spec/FS-config.md#6-what-is-not-configured-here)).

### 3. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

## GOAL-token-economy: give an agent the right amount of spec, not the whole file

Return the smallest deterministic slice that answers the grounding question; make escalation explicit. Cheap reads keep the organized memory ([§GRUND-schema.2](grund.md#2-keeping-it)) affordable enough that an agent grounds every change, not just the cheap ones.

### 1. What this requires

Bare `grund <ID>` is the cheap lead read; `--brief`, `--toc`, section reads, `--full`, `refs --total`, `refs --summary`, and narrowed `list` form the escalation ladder. `refs --total` is the ladder's cheapest rung on the back-reference side — the blast radius as two numbers, before the per-file rows are worth reading ([§FS-refs.3.4](functional-spec/FS-refs.md#34---total)).

### 2. What this rules out

No forced full-body reads, generated summaries, abridged diagnostics, or token saving that changes facts.

### 3. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

### 4. Research notes

Evidence for the cheap default lives in [DF-show-default-token-cheap](decisions/functional/DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in). The unit grund's slices return, one labelled fact per declaration, has its precedent in information typing ([§REL-information-typing](related-work/REL-information-typing.md#rel-information-typing-information-typing-units-of-one-kind-under-one-label)).

## GOAL-configurable: every default is overridable

Defaults fit canonical `grund`; config makes different project conventions first-class. A project can only keep its work grounded ([§GRUND-grund](grund.md#grund-grund-agents-stay-grounded-in-the-spec)) if the scheme bends to its layout instead of the reverse.

### 1. What is configurable

Every key [§FS-config.3](functional-spec/FS-config.md#3-keys) declares. That chapter is the list and this goal does not repeat it: a second copy falls behind the first the next time a key is added, and a goal that names keys is wrong rather than merely stale when it does. What puts a key on this side of the goal is not which key it is but what it governs — a project's own convention for how it names, files and cites, which `grund` reads instead of imposing canonical `grund`'s.

### 2. What is NOT configurable

Three things are frozen for every project and every install alike: the **set**
of severity levels (`error` and `warning`, and no third), the **mapping** from a
report to an exit code ([§FS-cli.5](functional-spec/FS-cli.md#5-exit-code-mapping-is-fixed)),
and the **ordering of the report**. No `grund.toml` and no machine may move any
of the three ([§FS-config.6](functional-spec/FS-config.md#6-what-is-not-configured-here)).

What a project's committed configuration does choose is which of the specified
rules are in force over its own tree — including whether a uniquely resolving
number-only shorthand may persist ([§FS-config.3.1](functional-spec/FS-config.md#31-reference--citation-form)) —
and, where a rule's own specification fixes the complete set of channels its
finding may speak through, which of those channels it speaks through there.
Neither is a choice about the three frozen things: a project that holds one rule
at advisory standing has configured that rule, not the severity set
([§DF-verdict-vocabulary-freeze](decisions/functional/DF-verdict-vocabulary-freeze.md#df-verdict-vocabulary-freeze-the-freeze-is-on-the-verdict-vocabulary-not-on-which-rules-a-project-holds-in-force)).

Separately, and about a different axis: install-local state may change no
verdict — that is the bright line
([§FS-non-goals.13](functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).
The procedure an author of a new key follows to stay on the right side of it is
[§FS-config.principle.install-local](functional-spec/FS-config.md#principleinstall-local-the-relation-governs-committed-repository-state-only)'s.

### 3. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

## GOAL-no-silent-breakage: changes ship through a deprecation path

Anything user-visible stays backward-compatible or crosses a named deprecation window. Silent semantic change is a release blocker. A grounding contract ([§GRUND-links](grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)) that shifts under a repo without warning is one nobody can rely on.

### 1. What counts as user-visible

CLI surface, output bytes, JSON schema, config schema/version, citation grammar, and managed agent-entrypoint block content.

### 2. The deprecation path

Release N adds the new form while the old form warns; release N+1 or later may remove it after the named horizon ([§REQ-backwards-compatibility.2](requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)). Two bounded routes join it, each argued rather than assumed: a loud mechanical migration whose fix is one command the tool ships ([§REQ-backwards-compatibility.3](requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)), and the correction of a verdict another hard requirement already forbade, on an accepted decision record ([§REQ-backwards-compatibility.5](requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)). What is never available is the fourth way — moving a verdict with neither a window nor a record.

### 3. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

## GOAL-small-and-large: start small, configure for big

One binary serves small repos and large monorepos; scale changes layout depth, not the citation contract.

### 1. Small-repo promise

A tiny flat repo works without ceremony.

### 2. Large-repo promise

A large repo can organize specs by component without changing citation syntax or resolver invariants.

### 3. Layout knobs live in config

Scale features are opt-in `grund.toml` settings, not implicit mode switches. Structure is paid for where it pays, as in pay-as-you-go data management ([§REL-schema-later](related-work/REL-schema-later.md#rel-schema-later-pay-as-you-go-structure-schema-added-where-it-pays)).

### 4. Composition with [§GOAL-zero-config](goals.md#goal-zero-config-works-on-any-conformant-tree) and [§GOAL-configurable](goals.md#goal-configurable-every-default-is-overridable)

Flat defaults keep small repos zero-config; config carries large layouts.

### 5. Measurable

See [AR-goal-measurement.2](architecture/AR-goal-measurement.md#2-goal-meters).

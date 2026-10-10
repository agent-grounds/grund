# REQ-readme: the README is the grounded shop window

The root `README.md` is the first surface a human or an agent reads, and it sells a discipline — so it must practice that discipline on itself. This serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) — the common path is legible before anyone opens the spec tree — and [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) — the first thing a reader learns is the loop they are expected to keep.

## 1. What it must say

The README opens with what `grund` is: the two grounds, one bullet each, every bullet citing the GRUND declaration that states it ([§GRUND-schema](../grund.md#grund-schema-the-shape-of-a-projects-knowledge-is-hard-to-define-and-to-keep), [§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)). It then shows the workflow rather than narrating it. The managed `AGENTS.md` block is what an agent is told about a grounded project ([§REQ-agents-md](REQ-agents-md.md#req-agents-md-the-agent-entrypoint-stays-managed-and-grounded)), and the README draws that same block for a human as three figures, in this order — the loop, the read ladder, the map — and then says how to install and set up a repository, which is what writes the block. The README is read whole, so depth lives in `docs/` behind links, not inline ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)); it stays measured against the repository's entrypoint line budget, and clearing an overflow of the 250-line soft tier means moving a section into `docs/` ([AR-ci section 9](../architecture/AR-ci.md#9-file-size-budget-gate)).

The README gives concise links to the existing traceability comparison and to performance
evidence and its methodology. It reuses the introduction and section-retrieval examples
rather than adding another slogan block or copying the detailed comparison.

### 1.1 The loop

Declare, cite and check, as one row of cards. Each card names its step, says in one line
what the step does, and carries one excerpt from this repository ([§REQ-readme.2](REQ-readme.md#2-every-example-is-real)): the
declared section, the code that cites it, and `grund check` failing once that section is
renumbered. The row is an HTML table, so it renders in either theme and its text stays
under the scan: a citation a card shows is checked like any other ([§REQ-readme.3](REQ-readme.md#3-the-readmes-citations-are-checked)). A
picture whose text the scan cannot read carries no excerpt, because nothing would notice
it drift.

### 1.2 The read ladder

What an agent reads in place of the file: the lead, one section, the section map and the
full body of one real declaration, each beside its measured size, so what the ladder saves
is a number rather than a claim ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)). Sizes are rounded, so that an
ordinary edit to that declaration does not stale them.

### 1.3 The map

The repository's kinds and the directions their citations take, drawn as one diagram from
its `[citations]`, with `must` and `should` told apart. The diagram summarizes; the rendered
directions in `AGENTS.md` stay the complete statement.

## 2. Every example is real

Code excerpts are verbatim from this repository, with elisions marked. Command output is captured from actually running the command against this tree — no invented IDs, no invented paths, no invented output. A change that invalidates a captured excerpt or output updates the README in the same change, per the co-change contract in [§FS-examples.4](../functional-spec/FS-examples.md#4-maintenance-contract).

## 3. The README's citations are checked

`README.md` is named in `[scan] include` ([§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-scanned)), so every `§`-marked citation in its prose resolves under `grund check` like any other scanned file's. Illustrations that must not resolve stay inside fenced code blocks, which the scanner ignores.

## evidence: Positioning and performance claims have evidence

The README and its supporting comparison, benchmark report and positioning roadmap copy
describe shipped facts, serving [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).
The undated throughput badge is retired. The existing related-work document explains
reader tasks in prose without a ranking matrix, blanket atomic-clause or coverage-only
claims, installation rankings, creation dates, or tests of competitors by Grund flag
spelling. Each retained competitor capability claim has a defined scope and current
primary-source support with the checked date and source version (or an explicit
unversioned designation); unsupported assertions are removed or qualified.

Positioning distinguishes structural section retrieval and citation validation from proof
of semantic implementation correctness. It describes `cover` as citations grouped by
scanned file, including empty files, without judging sufficient implementation coverage.
Supported declarative chapter and citation constraints are distinct from fixed severity
and absent arbitrary engine scripting. Supporting copy makes no future gap-report or
coverage-parity promise and derives no new product policy from existing non-goals.
Published section coordinates remain available.

Performance copy distinguishes the 2026-05-20 local wall-clock run, the historical
committed instruction-count snapshot, and current generated-fixture comparisons against
the pull request base branch. It preserves measured tables, raw samples and machine,
tool, commit and workload provenance. Instruction count is a repeatable workload-cost
proxy, not elapsed time or a universal latency guarantee: binary, input, build and PGO
assumptions govern comparability. Unlike workloads and their timing ratio do not establish
general relative speed. Current instruction-regression limits are reported as unenforced.
The report is archival; future measurement instructions write elsewhere rather than
overwrite it. This documentation correction requires no refreshed measurements or
benchmark-generator change.

Text regression checks pin known misleading passages, required paths and evidence
structure. Acceptance also requires factual review of every retained capability claim
against its primary source, agreement with the most-specific shipped specification, and
citation/link validation; deleting a detected sentence alone is insufficient.

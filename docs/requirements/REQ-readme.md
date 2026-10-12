# REQ-readme: the README is the grounded shop window

The root `README.md` is the first surface a human or an agent reads, and it sells a discipline — so it must practice that discipline on itself. This serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) — the common path is legible before anyone opens the spec tree — and [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) — the first thing a reader learns is the loop they are expected to keep.

## 1. What it must say

The README opens on the workflow, shown rather than narrated, and lets what follows say what `grund` is rather than a sentence about it. The managed `AGENTS.md` block is what an agent is told about a grounded project ([§REQ-agents-md](REQ-agents-md.md#req-agents-md-the-agent-entrypoint-stays-managed-and-grounded)), and the README shows that same block to a human: the loop, then the default structure, then excerpts of the block itself, then how to install and set up a repository, which is what writes the block. After setup come three reference sections: the citation format, drawn as the grammar of a citation; making citations clickable in a terminal; and editor support through `grund-lsp`, shown in screenshots of hover previews, diagnostics and navigation, each in both themes. The README is read whole, so depth lives in `docs/` behind links, not inline ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)); it stays measured against the repository's entrypoint line budget, and clearing an overflow of the 250-line soft tier means moving a section into `docs/` ([AR-ci section 9](../architecture/AR-ci.md#9-file-size-budget-gate)).

The README gives concise links to the existing traceability comparison and to performance
evidence and its methodology. It reuses the introduction and section-retrieval examples
rather than adding another slogan block or copying the detailed comparison.
The loop's cards link into a runnable example before a reader needs to learn the
whole schema. Setup distinguishes adopting existing specs
from scaffolding new ones, and offers adoption to an agent through
`grund agent-setup-instructions`.

### 1.1 The loop

Four cards form a two-by-two grid: declare and cite above, read and check below.
Every card has the same shape: a numbered verb, one line naming the file it shows or
the change it makes, and one excerpt from a committed runnable example
([§REQ-readme.2](REQ-readme.md#2-every-example-is-real)): a short requirement, the code that cites it, the section an agent
reads before changing that code, and `grund check` failing on a citation that
misspells that section. The cited section is a named section's numbered child
(`format.1`), so the README shows that a section path can mix names and numbers. Columns are equally wide and top-aligned, and the excerpts in a row are
equally tall. Each row is its own table body, so the renderer's alternating row shading
never falls on a card. The excerpts stay highlighted HTML text, readable in either theme.
The example has its own configuration and checked citations; its IDs are escaped in
the root README, and each card's file name links to its example file.

### 1.2 The default structure

The structure `grund init` writes, as one table read top to bottom from why to code: a
row for every kind `init` writes, with the question it answers, its home, and what it
cites under the canonical ruleset `init` writes commented out under `[citations]`,
every alternative named and the error levels told apart from the suggestions. It is a
table rather than a drawing because a table renders the same on every surface that
shows the README: GitHub, crates.io, either theme, and the raw text an agent reads. The
table stands alone, with no caption: the one sentence that introduces it says the ruleset
is commented out until a project adopts it.

### 1.3 The agent instructions

Straight after the default structure, the README quotes this repository's own managed
`AGENTS.md` block, verbatim with elisions marked ([§REQ-readme.2](REQ-readme.md#2-every-example-is-real)): how an agent reads
a citation, the project map that says where each kind lives, how it writes and cites,
and the citation directions as it reads them, so a reader sees the table turned into
instructions. The excerpts are quoted Markdown rather
than a code block, so their long lines wrap rather than scroll. A managed-block upgrade
that changes a quoted line updates the README in the same change. A short note closes
the section: the loop as one rule for the system prompt of an agent without the block.

## 2. Every example is real

Code excerpts are verbatim from this repository or its committed runnable examples, with elisions marked. Command output is captured from actually running the command against the named repository or example — no invented IDs, no invented paths, no invented output. A change that invalidates a captured excerpt or output updates the README in the same change, per the co-change contract in [§FS-examples.4](../functional-spec/FS-examples.md#4-maintenance-contract).

## 3. The README's citations are checked

`README.md` is named in `[scan] include` ([§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-scanned)), so every `§`-marked citation in its prose resolves under `grund check` like any other scanned file's. Illustrations that must not resolve stay inside fenced code blocks, which the scanner ignores.
HTML excerpts from a separate example repository encode the marker as `&#167;`;
the example's own check resolves those citations, and relative links identify their
source files. Such IDs are not declarations in the root repository's namespace.

The README tells a user what `grund` does, never how it is built: it may cite FS, and
it cites no AR. The repository's own `grund.toml` declares `README.md` as a place
with `[citations.readme] must-not = ["AR"]`, so `grund check` refuses an
architecture citation there, and the user-facing guides are where the README sends a
reader for depth.

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

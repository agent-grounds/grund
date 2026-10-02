# REL-traceability-tools: requirements-traceability tools, the neighbours beside grund

Requirements-traceability tools are the neighbours grund is most often compared with: they
also keep requirements as text and give each one an ID, and most link the code back to it.
They serve the same codebases grund is for ([§GRUND-grund.2](../grund.md#2-who-it-is-for)), on a different axis: a
coverage report rather than an agent reading one fact ([§GOAL-agent-grounding.1](../goals.md#1-the-three-layers)).

## work: What the work is

Six tools make up the comparison, in the matrix below. Two more sit beside them and stay
out of it:

- StrictDoc ([strictdoc-project/strictdoc](https://github.com/strictdoc-project/strictdoc)),
  from 2020, is "Software for technical documentation and requirements management",
  written in Python, and exports its documents to static HTML.
- sphinx-modeling ([PyPI](https://pypi.org/project/sphinx-modeling/)), published by
  useblocks, who maintain Sphinx-Needs, adds typed link constraints to Sphinx-Needs:
  "outgoing links must target specific need types or union of types". Its last release
  was in December 2022. The severities
  `info`, `warning` and `violation` are not its own: they belong to Sphinx-Needs'
  [schema validation](https://sphinx-needs.readthedocs.io/en/8.5.0/schema/index.html),
  where "each schema rule can specify one of the severity levels".

### work.matrix: The comparison matrix

| Tool | Since | Markdown-native | Inline code citations | Sectioned IDs `§<ID>.3.1` | Resolver CLI `--brief`/`--toc`/`--full` | Coverage report | Single binary |
|---|---|---|---|---|---|---|---|
| **grund** | 2026 | ✅ | ✅ | ✅ | ✅ | ⏳ [§RM-gap-report](../roadmap.md#rm-gap-report-orphan-and-uncovered-id-reports) | ✅ |
| [OpenFastTrace](https://github.com/itsallcode/openfasttrace) | 2015 | ✅ | ✅ | ❌ | ❌ | ✅ flagship | ❌ JVM |
| [Sphinx-Needs](https://github.com/useblocks/sphinx-needs) | 2016 | ⚠ RST/MyST | ⚠ via sphinx-codelinks | ❌ | ⚠ via Sphinx build | ✅ | ❌ Python+Sphinx |
| [TRLC](https://github.com/bmw-software-engineering/trlc) + [LOBSTER](https://github.com/bmw-software-engineering/lobster) | 2022 | ❌ DSL | ✅ | ❌ | ❌ | ✅ | ❌ Python |
| [Doorstop](https://github.com/doorstop-dev/doorstop) | 2013 | ⚠ Markdown + YAML frontmatter, opt-in per document | ⚠ links only | ❌ | ❌ | ✅ | ❌ Python |
| [Duvet](https://github.com/awslabs/duvet) | 2021 | ⚠ specs only | ✅ | ⚠ anchors | ❌ | ✅ flagship | ✅ |
| [SARA](https://github.com/cledouarec/sara) | 2026 | ✅ + YAML frontmatter | ❌ | ❌ | ⚠ graph queries | ✅ | ✅ |

"Since" is the year of a tool's first repository or release. Sphinx-Needs first shipped on
1 December 2016, as `sphinxcontrib-needs`.

## agrees: Where grund agrees

- Requirements live as text in the repository, each with a stable ID, and code points back
  at the one it realizes ([§GRUND-structure](../grund.md#grund-structure-the-projects-long-term-memory-stays-organized)).
- Checking those links is a job for CI, before merge, which is where grund runs too
  ([§GRUND-grund.2](../grund.md#2-who-it-is-for)).

## departs: Where grund departs

- They are optimized for a coverage report. grund is optimized for an agent reading one
  specific fact: a sectioned citation and a depth-controlled resolver give the smallest
  text that justifies a line of code ([§GOAL-agent-grounding.1](../goals.md#1-the-three-layers), [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)).
- They model each clause as its own atomic item. grund keeps the clause inside the spec it
  belongs to and lets the citation point at its heading ([§GRUND-structure](../grund.md#grund-structure-the-projects-long-term-memory-stays-organized)).
- Sphinx-Needs and StrictDoc render their requirements as documentation. grund generates
  none: it reads, validates and slices ([§FS-non-goals.5](../functional-spec/FS-non-goals.md#5-documentation-generation)).
- grund imports and exports no foreign trace format, ReqIF or OFT's among them: it reads
  only its own citation scheme ([§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)), and two installs never disagree about a
  tree ([§FS-non-goals.13](../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).
- Sphinx-Needs' schema rules each choose a severity. grund's level→surface mapping is fixed
  ([§FS-non-goals.9](../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)).

# REL-traceability-tools: requirements-traceability tools, the neighbours beside grund

Choose a tool by the task: reading a cited fact, tracing tagged specification items,
building a requirements extract, or checking declared constraints. Grund serves the
human and agent working in a repository ([§GRUND-grund.2](../grund.md#2-who-it-is-for)),
with structural reads that bring one supporting fact into context
([§GOAL-agent-grounding.1](../goals.md#1-the-three-layers)).

## work: What the work is

**Primary-source ledger, checked 2026-10-06.** The claims below are limited to the
documented tasks; they do not rank the tools or claim that any task is their only purpose.

- **Check links:** Lychee's [official documentation](https://lychee.cli.rs/),
  unversioned, documents native Markdown and HTML link checking.
- **Trace tagged items and read a report:** OpenFastTrace's unversioned
  [user guide](https://openfasttrace.itsallcode.org/user_guide/user_guide.html)
  describes specification items in Markdown and source-code tags that mark their
  coverage. Its [HTML report instructions](https://openfasttrace.itsallcode.org/user_guide/use_cases/html_tracing_reports.html)
  document `oft trace -o html`. The current release checked was
  [4.10.0](https://github.com/itsallcode/openfasttrace/releases/tag/4.10.0);
  the guides are not pinned to that release.
- **Build an extract and choose schema-rule severity:** Sphinx-Needs 8.5.0's
  [needextract](https://sphinx-needs.readthedocs.io/en/8.5.0/directives/needextract.html)
  directive copies filtered needs during a Sphinx build, including selection by ID.
  Its documentation cautions that complex original content may not render correctly.
  Its [schema rule severity](https://sphinx-needs.readthedocs.io/en/8.5.0/schema/index.html#rule-severity)
  documentation allows each schema rule to select `info`, `warning`, or `violation`.

Other project sources, without capability assertions here:
[TRLC](https://github.com/bmw-software-engineering/trlc),
[LOBSTER](https://github.com/bmw-software-engineering/lobster),
[Doorstop](https://github.com/doorstop-dev/doorstop),
[Duvet](https://github.com/awslabs/duvet),
[SARA](https://github.com/cledouarec/sara),
[StrictDoc](https://github.com/strictdoc-project/strictdoc), and
[sphinx-modeling](https://pypi.org/project/sphinx-modeling/).

### work.matrix: Reader tasks

**Read a cited section.** `grund <ID>.<section>` selects a heading inside its
declaration. Default, brief, TOC and full modes return structural slices or a section
map ([§FS-show.2.2](../functional-spec/FS-show.md#22-section)); a clause can stay in
its containing spec rather than becoming a separate declaration. This is a read of
authored text, not a generated summary or proof that code implements its meaning.

**Check citation targets.** `grund check` reports a dangling ID
([§FS-check.3.1](../functional-spec/FS-check.md#31-dangling-citation)) or an absent
cited section ([§FS-check.3.2](../functional-spec/FS-check.md#32-missing-section)).
A resolving citation establishes a structural connection; reviewing the implementation
and its tests still determines whether that connection is justified.

**Find what a file cites.** `grund cover` groups recognised citations by scanned
file, including files with no citations. It does not judge sufficient implementation
coverage or require a spec/test co-change
([§FS-cover.2](../functional-spec/FS-cover.md#2-behaviour)).

**Check declared constraints.** Grund supports chapter counts
([§FS-rules.3.1](../functional-spec/FS-rules.md#31-chapter-presence)), outbound
citation counts ([§FS-rules.3.2](../functional-spec/FS-rules.md#32-outbound-citation-count)),
counts for each selected target ([§FS-rules.3.3](../functional-spec/FS-rules.md#33-per-target-coverage)),
and inbound counts and citation prohibitions
([§FS-rules.3.4](../functional-spec/FS-rules.md#34-inbound-citation-count-and-prohibition)).
These declarative constraints check the document and citation structure, not arbitrary
properties of the implementation.

## agrees: Where grund agrees

Stable IDs let code refer back to intent held as repository text
([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)).
The OpenFastTrace task above is another concrete use of source tags to connect code
to specification items. Grund's structural checks can run in CI before merge
([§GRUND-grund.2](../grund.md#2-who-it-is-for)).

## departs: Where grund departs

Grund's severity, exit-code mapping and report order are fixed
([§FS-non-goals.9](../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization));
Sphinx-Needs' per-schema-rule severity is a distinct option in the ledger above.
Fixed severity does not prevent Grund's supported chapter and citation constraints.
There are no arbitrary plugins or scripting hooks inside the checking engine;
custom orchestration can call the API from external code
([§FS-non-goals.12.1](../functional-spec/FS-non-goals.md#121-plugins-or-scripting-hooks-inside-the-engine)).

Grund does not publish HTML or PDF documentation
([§FS-non-goals.5](../functional-spec/FS-non-goals.md#5-documentation-generation)).
Its reads and queries can be inputs to downstream tools.

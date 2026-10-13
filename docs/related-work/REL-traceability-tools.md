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

Dated capability claims for OpenFastTrace, Duvet, Doorstop, Sphinx-Needs with
sphinx-codelinks, StrictDoc, TRLC, LOBSTER and lychee, checked 2026-10-13, follow the
reader tasks below. Other project sources, without capability assertions here:
[SARA](https://github.com/cledouarec/sara) and
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

### work.capabilities: Documented capabilities, checked 2026-10-13

Each claim says what the named tool's own pages state for the release named, read on
2026-10-13; it does not rank the tools. A claim that rests on source code because the
documentation is silent says so. A capability not named was not found on the pages read.

- **OpenFastTrace 4.10.0** (GPL-3.0, released 2026-09-20). Its 4.10.0 [manual page](https://github.com/itsallcode/openfasttrace/blob/4.10.0/doc/man/man_1.md) names
  Markdown and reStructuredText as native specification formats, with Gherkin and
  SpecObject XML, and its [input formats](https://github.com/itsallcode/openfasttrace/blob/4.10.0/doc/user_guide/reference/input_format_support.md) list about 30 source and markup formats whose
  comments carry coverage tags; every input links into one item set, read from local files
  with no network use documented. An item ID is `type~name~revision`, holds no path, and
  is separate from an optional title; `Needs:` names the artifact types that must cover an
  item ([writing a specification](https://github.com/itsallcode/openfasttrace/blob/4.10.0/doc/user_guide/use_cases/writing_a_specification.md)). Missing or unwanted coverage, and a link to another
  revision than the item's, are defects ([broken branches](https://github.com/itsallcode/openfasttrace/blob/4.10.0/doc/user_guide/use_cases/understanding_and_fixing_broken_requirement_branches.md)); its [design specification](https://github.com/itsallcode/openfasttrace/blob/4.10.0/doc/spec/design.md)
  gives exit status 1 for a negative trace, where the manual page says only "OFT error".
  It offers Maven and Gradle [build integration](https://github.com/itsallcode/openfasttrace/blob/4.10.0/doc/user_guide/reference/build_integration.md), a [Java fluent API](https://github.com/itsallcode/openfasttrace/blob/4.10.0/doc/user_guide/oft_api/using_oft_from_java.md), importer, exporter
  and reporter [plugins](https://github.com/itsallcode/openfasttrace/blob/4.10.0/doc/plugins.md), and a JetBrains [plugin](https://github.com/itsallcode/openfasttrace/blob/4.10.0/doc/user_guide/tool_support/tools_for_authoring_oft_documents.md); no documented command returns one
  item's text. Pages on its default branch, newer than 4.10.0, add a
  [third-party language server](https://github.com/itsallcode/openfasttrace/blob/main/doc/user_guide/tool_support/tools_for_authoring_oft_documents.md) and say in the [FAQ](https://github.com/itsallcode/openfasttrace/blob/main/doc/faq/faq.md) that it has no MCP server, by design.
- **Duvet** (Apache-2.0; GitHub release [v0.4.2](https://github.com/awslabs/duvet/releases/tag/v0.4.2), 2026-04-02; crates.io [0.4.3](https://docs.rs/crate/duvet/0.4.3),
  2026-08-04). Its unversioned [guide](https://awslabs.github.io/duvet/) documents [annotations](https://github.com/awslabs/duvet/blob/v0.4.2/guide/src/annotations.md) in the comments of any
  language, with the comment style set per source glob, that cite a section of an IETF or
  Markdown specification by URL or path and quote its text, so moving the specification
  changes the address; annotations of every configured language resolve against one set of
  specifications, and citing from prose is not documented. A Markdown section is cited by
  its heading's slug; that an explicit heading id replaces the slug is from its
  [source code](https://github.com/awslabs/duvet/blob/v0.4.2/duvet/src/specification/markdown/parser.rs), not its documentation. `duvet report --ci` fails when the committed
  snapshot differs ([reports](https://github.com/awslabs/duvet/blob/v0.4.2/guide/src/reports.md)), and a GitHub [Action](https://github.com/awslabs/duvet/blob/v0.4.2/action.yml) installs it. `duvet query` ships in
  0.4.3 only, which its [changelog](https://github.com/awslabs/duvet/blob/8899d63/CHANGELOG.md) still lists as unreleased; it [exits 1](https://github.com/awslabs/duvet/blob/8899d63/guide/src/query.md) when an
  in-scope requirement lacks an implementation or test annotation. From its source code,
  not its documentation: the quoted text must occur in the cited section ([reference.rs](https://github.com/awslabs/duvet/blob/v0.4.2/duvet/src/reference.rs)),
  and a specification given by URL is downloaded when no cached copy exists ([target.rs](https://github.com/awslabs/duvet/blob/v0.4.2/duvet/src/target.rs)).
  Its annotation types are fixed; no link rules, rule language, editor support or read of
  one section's text are documented.
- **Doorstop v3.2** (LGPL-3.0, [released 2026-07-10](https://github.com/doorstop-dev/doorstop/releases/tag/v3.2)), described by Jace Browning and
  Robert Adams, *Doorstop: Text-Based Requirements Management Using Version Control*,
  Journal of Software Engineering and Applications 7(3), 2014, DOI 10.4236/jsea.2014.73020
  ([Crossref record](https://api.crossref.org/works/10.4236/jsea.2014.73020)). Its unversioned [documentation](https://doorstop.readthedocs.io/) keeps one item per YAML or Markdown file,
  named by its UID, in one namespace across the tree; a link stores the parent's UID and a
  fingerprint of the parent, which the header does not enter, and a changed fingerprint
  makes the link suspect. An item's `references` search files for a keyword, but citations
  written in code are not parsed ([items](https://github.com/doorstop-dev/doorstop/blob/v3.2/docs/reference/item.md)). Each document names one parent document
  ([documents](https://github.com/doorstop-dev/doorstop/blob/v3.2/docs/reference/document.md)); unlinked items are warnings ([validation](https://github.com/doorstop-dev/doorstop/blob/v3.2/docs/cli/validation.md)), which `-e` reports as errors,
  as its command-line help says; Python validation hooks add rules ([scripting](https://github.com/doorstop-dev/doorstop/blob/v3.2/docs/api/scripting.md)). It
  validates local files, and ships an experimental [desktop GUI](https://github.com/doorstop-dev/doorstop/blob/v3.2/docs/gui/overview.md), a [web server](https://github.com/doorstop-dev/doorstop/blob/v3.2/docs/web.md) and
  [pre-commit hooks](https://github.com/doorstop-dev/doorstop/blob/v3.2/.pre-commit-hooks.yaml). From its source code, not its documentation: the command exits 1 on
  error-level issues ([cli/main.py](https://github.com/doorstop-dev/doorstop/blob/v3.2/doorstop/cli/main.py)), and the server answers a REST request for one item
  ([server/main.py](https://github.com/doorstop-dev/doorstop/blob/v3.2/doorstop/server/main.py)).
- **Sphinx-Needs 8.5.0** (MIT, [released 2026-09-03](https://github.com/useblocks/sphinx-needs/releases/tag/8.5.0)) with **sphinx-codelinks 1.4.0**, a
  separate package (MIT, [released 2026-07-30](https://pypi.org/project/sphinx-codelinks/1.4.0/)). Needs are written in reStructuredText or
  MyST and addressed by ID, a need part as `NEED.part` ([roles](https://sphinx-needs.readthedocs.io/en/8.5.0/roles.html)); links name IDs, not
  files ([configuration](https://sphinx-needs.readthedocs.io/en/8.5.0/configuration.html)), and an omitted ID is a hash of the title, so only an explicit
  ID survives a reworded title ([need](https://sphinx-needs.readthedocs.io/en/8.5.0/directives/need.html)). With sphinx-codelinks, comments in C/C++, Python,
  C#, YAML, Rust, Go, JSONC and Bash ([languages](https://github.com/useblocks/sphinx-codelinks/blob/1.4.0/docs/source/components/configuration.rst)) create needs or link to existing ones
  in the same set as the prose ([introduction](https://codelinks.useblocks.com/en/latest/basics/introduction.html)). Typed links and JSON-schema network rules
  limit what may link to what, and `network_back` with `minContains` requires incoming
  links; schema rules compare a linked need's fields with constants, not with a value at
  the referring need ([schema](https://sphinx-needs.readthedocs.io/en/8.5.0/schema/index.html)). Filter strings and Python functions add checks,
  `sphinx-build -W` fails on its warnings, and a constraint set to `break` stops the
  build. `needimport` from a [URL](https://sphinx-needs.readthedocs.io/en/8.5.0/directives/needimport.html) and [`needservice`](https://sphinx-needs.readthedocs.io/en/8.5.0/directives/needservice.html) fetch remote data; the `needs_id`
  [builder](https://sphinx-needs.readthedocs.io/en/8.5.0/builders.html) writes one JSON file per need. Its editor support, [ubCode](https://sphinx-needs.readthedocs.io/en/8.5.0/installation.html), is the vendor's
  [proprietary](https://ubcode.useblocks.com/usage/ublicense.html) VS Code extension.
- **StrictDoc 0.30.2** (Apache-2.0, [released 2026-10-09](https://github.com/strictdoc-project/strictdoc/releases/tag/0.30.2)). Its [user guide](https://strictdoc.readthedocs.io/en/stable/stable/docs/strictdoc_01_user_guide.html) ([source](https://github.com/strictdoc-project/strictdoc/blob/0.30.2/docs/strictdoc_01_user_guide.sdoc))
  documents SDoc, experimental Markdown, and opt-in source traceability: `@relation`
  markers in Python, C/C++, Rust and Robot Framework files, read by language-aware
  parsers, and in any other file through a generic parser, resolve against the UIDs the
  documents' own relations use. A UID is separate from the title; anchors, and relations
  to a file's line range, function or class, are addressable, and relations name UIDs, not
  paths. The grammar lists the relation types and roles each element allows. Coverage is
  reported on the experimental Tree Map and enforced only by a Python plugin; no value
  comparison is documented. Python-like queries filter an experimental JSON export, editor
  support is syntax highlighting only, ReqIF import and export are "initial", and no
  network step is documented. From its source code, not its documentation: the command
  exits 1 when a relation names a missing UID ([main.py](https://github.com/strictdoc-project/strictdoc/blob/0.30.2/strictdoc/cli/main.py), [file_traceability_index.py](https://github.com/strictdoc-project/strictdoc/blob/0.30.2/strictdoc/core/file_traceability_index.py)).
- **TRLC 3.1.0** (GPL-3.0, [released 2026-09-30](https://github.com/bmw-software-engineering/trlc/releases/tag/trlc-3.1.0)). Types and checks live in `.rsl` files
  and records in `.trlc` files ([basics](https://github.com/bmw-software-engineering/trlc/blob/trlc-3.1.0/documentation/TUTORIAL-BASIC.md)); a record is named `Package.name`, and a package
  may span many files ([packages](https://github.com/bmw-software-engineering/trlc/blob/trlc-3.1.0/documentation/TUTORIAL-PACKAGE.md)). A reference names a record of the type it declares,
  apart from its description ([references](https://github.com/bmw-software-engineering/trlc/blob/trlc-3.1.0/documentation/TUTORIAL-ADVANCED.md)). `checks` are typed Boolean expressions per
  record type that can follow a reference and compare its fields with the referrer's, each
  a `warning`, `error` or `fatal` ([checks](https://github.com/bmw-software-engineering/trlc/blob/trlc-3.1.0/documentation/TUTORIAL-CHECKS.md)). It exits 1 on errors ([CI](https://github.com/bmw-software-engineering/trlc/blob/trlc-3.1.0/documentation/TUTORIAL-CI.md)), its
  [Python API](https://github.com/bmw-software-engineering/trlc/blob/trlc-3.1.0/documentation/TUTORIAL-API.md) returns each record's values, and BMW's [VS Code extension](https://github.com/bmw-software-engineering/trlc-vscode-extension) runs the
  `trlc-lsp` language server.
- **LOBSTER 1.0.6** (AGPL-3.0, [released 2026-07-30](https://github.com/bmw-software-engineering/lobster/releases/tag/lobster-1.0.6)) joins tags extracted from C/C++,
  Python, GoogleTest, JSON, TRLC and Codebeamer, among others, into one report ([README](https://github.com/bmw-software-engineering/lobster/blob/lobster-1.0.6/README.md));
  a tag may carry `@version` ([schemas](https://github.com/bmw-software-engineering/lobster/blob/lobster-1.0.6/documentation/schemas.md)). Its policy's `trace to` and `requires` say which
  level must trace to which ([configuration](https://github.com/bmw-software-engineering/lobster/blob/lobster-1.0.6/documentation/config_files.md)), and a missing required link or a stale
  explicit version is an error ([user manual](https://github.com/bmw-software-engineering/lobster/blob/lobster-1.0.6/documentation/user-manual.md)). Its Codebeamer extractor needs a [server](https://github.com/bmw-software-engineering/lobster/blob/lobster-1.0.6/documentation/manual-lobster_codebeamer.md).
  The 1.0.6 [core README](https://github.com/bmw-software-engineering/lobster/blob/lobster-1.0.6/packages/lobster-core/README.md) calls `lobster-ci-report` "useful for CI"; its 0, 1 and 2 return
  codes are documented only on the [main branch](https://github.com/bmw-software-engineering/lobster/blob/main/packages/lobster-core/README.md) (1.1.0-dev), and that 1.0.6 returns 1 for
  an item neither traced nor justified is from its [source code](https://github.com/bmw-software-engineering/lobster/blob/lobster-1.0.6/lobster/tools/core/ci_report/ci_report.py), not its documentation.
- **lychee 0.24.2** (Apache-2.0 OR MIT, [released 2026-05-01](https://github.com/lycheeverse/lychee/releases/tag/lychee-v0.24.2)). Its unversioned [README](https://github.com/lycheeverse/lychee)
  parses Markdown and HTML and reads other files as plain text. Links are URLs and paths
  in one URL space with no declared targets, so a moved target breaks them. With
  `--include-fragments` it checks anchors, where a heading-derived anchor breaks on a
  reworded heading and an explicit id survives ([anchors](https://lychee.cli.rs/recipes/anchors/)), and text fragments, which must
  occur on the page. Regex filters and remapping select links; it fetches remote URLs
  unless run with `--offline`, exits 2 on a failed link, and has a pre-commit hook and a
  GitHub [Action](https://github.com/lycheeverse/lychee-action). It returns check results, not document text; no link rules, coverage
  requirement or editor support are documented.

## agrees: Where grund agrees

Stable IDs let code refer back to intent held as repository text
([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)).
The OpenFastTrace task above is another concrete use of source tags to connect code
to specification items. Grund's structural checks can run in CI before merge
([§GRUND-grund.2](../grund.md#2-who-it-is-for)).

OpenFastTrace's tags, StrictDoc's `@relation` markers, LOBSTER's extracts and the comment
markers of sphinx-codelinks resolve IDs written in code comments against one set of items
shared with the specifications, as grund resolves one citation grammar in Markdown and
source comments alike ([§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)). OpenFastTrace's `Needs:`, LOBSTER's
policy and Sphinx-Needs' network rules declare which kind must link to which, as grund's
citation directions do ([§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules)), and TRLC checks compare fields through a
reference, as grund's value bindings compare a literal with the component it cites
([§FS-values.3.1](../functional-spec/FS-values.md#31-the-only-binding-grammar)).

Doorstop's link fingerprints, and the revisions and versions OpenFastTrace and LOBSTER
compare with a link's target, are the prior art for grund's section lock, which records a
fingerprint of the text each cited relative section held
([§FS-section-lock.holds.fingerprint](../functional-spec/FS-section-lock.md#holdsfingerprint-the-fingerprint)). Where they flag a link whose target changed, the
lock reports cited text that moved to another address, and `grund fmt --write` refreshes
an edit in place rather than reporting it ([§FS-section-lock.write.rules](../functional-spec/FS-section-lock.md#writerules-add-refresh-drop)).

## departs: Where grund departs

Grund's severity, exit-code mapping and report order are fixed
([§FS-non-goals.9](../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization));
Sphinx-Needs' per-schema-rule severity is a distinct option in the ledger above.
Fixed severity does not prevent Grund's supported chapter and citation constraints.
There are no arbitrary plugins or scripting hooks inside the checking engine;
custom orchestration can call the API from external code
([§FS-non-goals.12.1](../functional-spec/FS-non-goals.md#121-plugins-or-scripting-hooks-inside-the-engine)).
TRLC's choice of `warning`, `error` or `fatal` for each check and Doorstop's `-e`, which
reports warnings as errors, are severity options of the same kind; Doorstop's validation
hooks, StrictDoc's plugin, Sphinx-Needs' Python filter functions and OpenFastTrace's
plugins run user code inside the tool.

Grund does not publish HTML or PDF documentation
([§FS-non-goals.5](../functional-spec/FS-non-goals.md#5-documentation-generation)).
Its reads and queries can be inputs to downstream tools.

Grund formalizes one ID scheme and validates no other citation syntax
([§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)). Duvet cites a specification section by URL or path and quoted text;
OpenFastTrace's typed items with revisions, Doorstop's documents of UID-named items,
TRLC's user-declared record types and StrictDoc's document grammars are item models of
their own, and StrictDoc exchanges [ReqIF](https://www.omg.org/spec/ReqIF/). Grund
resolves its marked citations to declarations of its configured kinds, with no URL-only
resolver.

A check reads no network ([§FS-non-goals.11](../functional-spec/FS-non-goals.md#11-network-access-during-a-check)). Duvet downloads a specification given by URL
when no cached copy exists, from its source code, not its documentation; Sphinx-Needs'
`needimport` and `needservice` fetch remote data; LOBSTER's Codebeamer extractor queries a
server; lychee fetches remote URLs unless run with `--offline`.

Grund does not parse code ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)), so it has no counterpart to StrictDoc's
language-aware relations to a function or class.

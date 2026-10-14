# REL-code-queries: queries and conformance over program facts, rules over what was extracted

Query languages over program facts state a check as rules over relations extracted from
code, and architecture-conformance tools hold what was extracted to a declared model. They
are the precedent for the rules grund declares over a project's links, whose meaning is
stated as Datalog over the facts one scan extracts ([§GRUND-links.1](../grund.md#1-declaring-the-rules), [§FS-rules.5](../functional-spec/FS-rules.md#5-relational-meaning)).

## work: What the work is

Pavel Avgustinov, Oege de Moor, Michael Peyton Jones and Max Schäfer, *QL: Object-oriented
Queries on Relational Data*, ECOOP 2016, LIPIcs 56, pages 2:1–2:26, DOI
10.4230/LIPIcs.ECOOP.2016.2
([Dagstuhl](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.ECOOP.2016.2)),
describes QL, which "compiles to Datalog" yet offers classes and methods "reinterpreted in
logical terms: classes are logical properties describing sets of values, subclassing is
implication". Its authors used it for static analyses "which scale to millions of lines of
code", and as a case study reimplemented the 101 checks of the Error Prone bug finder. The
article gives Core QL a formal semantics by translation to Datalog and states no theorem.
Martin Bravenboer and Yannis Smaragdakis, *Strictly Declarative Specification of
Sophisticated Points-to Analyses*, OOPSLA 2009, pages 243–262, DOI 10.1145/1640089.1640108
([dblp](https://dblp.org/doi/10.1145/1640089.1640108)), present Doop, which describes "the
full end-to-end analysis in Datalog", and report it more than 15 times faster than the
Paddle framework. Herbert Jordan, Bernhard Scholz and Pavle Subotić, *Soufflé: On
Synthesis of Program Analyzers*, CAV 2016, pages 422–430, DOI 10.1007/978-3-319-41540-6_23
([Springer](https://link.springer.com/chapter/10.1007/978-3-319-41540-6_23)), synthesize analyzers from
Datalog, with a points-to analysis of OpenJDK7 that runs in under a minute. In all three
the check is whatever the user-written queries encode, over program elements such as
statements, variables and methods; they hold no links, so whether one survives an edit
does not arise, and the evaluation is scale and analysis performance.

Gail C. Murphy, David Notkin and Kevin Sullivan, *Software Reflexion Models: Bridging the
Gap Between Source and High-Level Models*, FSE 1995, pages 18–28, DOI 10.1145/222124.222136
([dblp](https://dblp.org/doi/10.1145/222124.222136)): "an engineer defines a high-level
model and specifies how the model maps to the source", and a tool computes a reflexion
model that "shows where the engineer's high-level model agrees with and where it differs
from a model of the source". The mapping names source entities by regular expressions over
file and directory names, as in `[ file=.*pager.* mapTo=Pager ]`. The article gives a
formal characterization of reflexion models but no theorem, and its case studies are
NetBSD and Microsoft Excel. Neeraj Sangal, Ev Jordan, Vineet Sinha and Daniel Jackson,
*Using Dependency Models to Manage Complex Software Architecture*, OOPSLA 2005, pages
167–176, DOI 10.1145/1094811.1094824
([PDF](https://groups.csail.mit.edu/sdg/pubs/2005/oopsla05-dsm.pdf)), extract dependencies
"by a conventional static analysis" into a dependency structure matrix, on which "design
rules" "capture the architect's intent about which dependencies are acceptable" and are
"applied repeatedly as the system evolves, to identify violations". A module is a Java
class, grouped by the package hierarchy, and the evaluation is a case study of the Haystack
system. Neither work discusses whether a mapping survives a moved file or a renamed entity.

Trace query languages ask the same kind of question of a trace store kept apart from the
artifacts. Patrick Mäder and Jane Cleland-Huang, *A visual language for modeling and
executing traceability queries*, Software & Systems Modeling 12(3):537–553, 2013, DOI
10.1007/s10270-012-0237-0 ([Springer](https://link.springer.com/article/10.1007/s10270-012-0237-0)), give
VTML, which "constructs trace queries as constraints upon subsets of the model", a
traceability information model in UML class diagrams, where users would otherwise
"construct non-trivial traceability queries using generic query languages such as SQL". It
is evaluated by queries modelled for a web-based health-care system and by "an experiment
with human users" comparing VTML queries with SQL. Jonathan I. Maletic and Michael L.
Collard, *TQL: A query language to support traceability*, TEFSE 2009, DOI
10.1109/TEFSE.2009.5069577 ([IEEE](https://doi.org/10.1109/TEFSE.2009.5069577)), is "based
in XML and supports queries across multiple artifacts and multiple traceability link
types", shown by example queries. Neither abstract says where the links in the store come
from, and neither states a guarantee.

Two articles in *Proceedings of the ACM on Programming Languages* 8, 2024, write rules over
syntax instead of extracted facts. Ameya Ketkar, Daniel Ramos, Lazaro Clapp, Raj Barik and
Murali Krishna Ramanathan, *A Lightweight Polyglot Code Transformation Language*, DOI
10.1145/3656429 ([dblp](https://dblp.org/doi/10.1145/3656429)), introduce "a new
declarative domain-specific language (DSL) for expressing interdependent multi-language
code transformations", implemented as Polyglot Piranha, which extends match-replace tools
with "composition, ordering, and flow"; deployed in industry, "it deleted 210K lines of dead
code and migrated 20K lines, across 1611 pull requests". Gabriel Matute, Wode Ni, Titus
Barik, Alvin Cheung and Sarah E. Chasins, *Syntactic Code Search with Sequence-to-Tree
Matching*, DOI 10.1145/3656460 ([dblp](https://dblp.org/doi/10.1145/3656460)), present
stsearch, whose queries are tokenized rather than parsed and matched with tree automata,
"while still providing results comparable to Semgrep for existing queries".

Three tools work in the same line; QL is CodeQL's query language. They are described below
from their own documentation (tool documentation, not peer-reviewed), at the version named,
checked 2026-10-13. None of them documents a model of references or addresses: a query or
rule reports program elements or matches at locations in files, so whether a reference
survives a moved file or a reworded title does not arise.

The CodeQL Terms and Conditions reference below must cite GitHub's underlying
`LICENSE.md` file in `github/codeql-cli-binaries` at `main`, served directly by
`raw.githubusercontent.com`.

- **CodeQL**, CLI 2.27.2, released 2026-10-07
  ([release](https://github.com/github/codeql-cli-binaries/releases/tag/v2.27.2)); its
  documentation is unversioned and describes the most recent CLI. The
  [QL language reference](https://codeql.github.com/docs/ql-language-reference/about-the-ql-language/)
  calls QL "a declarative, object-oriented query language" and says "the semantics of QL
  are based on Datalog"; a QL program "is compiled into a variant of the logic programming
  language Datalog"
  ([evaluation](https://codeql.github.com/docs/ql-language-reference/evaluation-of-ql-programs/)).
  The [supported languages](https://codeql.github.com/docs/codeql-overview/supported-languages-and-frameworks/)
  are C/C++, C#, Go, Java, Kotlin, JavaScript/TypeScript, Python, Ruby, Rust and Swift, plus
  GitHub Actions workflows; Markdown is not among them. The CLI builds
  [one database for each language](https://docs.github.com/en/code-security/codeql-cli/getting-started-with-the-codeql-cli/preparing-your-code-for-codeql-analysis),
  and a query runs against one database. Its language libraries model comments, Java's
  through the [`Javadoc` class](https://codeql.github.com/docs/codeql-language-guides/javadoc/),
  so a query can read comments, one language at a time. No kind-to-kind link rule is built
  in; one can be written as a QL query. Queries run locally against a local database once
  the query packs are present, and `--download` fetches missing ones
  ([database analyze](https://docs.github.com/code-security/codeql-cli/manual/database-analyze/)).
  [CodeQL for VS Code](https://docs.github.com/en/code-security/codeql-for-vs-code/getting-started-with-codeql-for-vs-code/about-codeql-for-vs-code)
  writes and runs queries, and
  [`codeql execute language-server`](https://docs.github.com/en/code-security/codeql-cli/codeql-cli-manual/execute-language-server)
  gives editors QL support. The documentation describes no command that returns one
  unit's text. As a CI gate, code scanning with
  [merge protection](https://docs.github.com/en/code-security/how-tos/find-and-fix-code-vulnerabilities/manage-your-configuration/set-merge-protection)
  blocks a pull request at an alert threshold; none of the CLI's
  [exit codes](https://docs.github.com/code-security/codeql-cli/exit-codes/) signals alerts
  found. The CLI is proprietary, under the
  [GitHub CodeQL Terms and Conditions](https://raw.githubusercontent.com/github/codeql-cli-binaries/main/LICENSE.md);
  the queries and libraries in [github/codeql](https://github.com/github/codeql) are MIT.
- **Semgrep Community Edition**, 1.180.0, released 2026-10-07
  ([release](https://github.com/semgrep/semgrep/releases/tag/v1.180.0)), LGPL-2.1
  ([repository](https://github.com/semgrep/semgrep)); its documentation is unversioned.
  Rules are written in [YAML](https://docs.semgrep.dev/writing-rules/rule-syntax) with
  code-like patterns, and each names an `id`, a `message`, a `severity`, its `languages` and
  a pattern; the severities are `LOW`, `MEDIUM`, `HIGH` and `CRITICAL`. The rule schema has
  no item kinds or link types. The edition parses the
  [languages it lists](https://docs.semgrep.dev/semgrep-ce-languages), Markdown not among
  them, and reads any text in
  [generic mode](https://docs.semgrep.dev/writing-rules/generic-pattern-matching) or with
  `pattern-regex`. Language rules "generally do not match comments"
  ([knowledge base](https://docs.semgrep.dev/kb/rules/match-comments)), and the repository
  says the edition "can only analyze code within the boundaries of a single function or
  file". With a local rules file it runs locally and sends no metrics, though a version
  check contacts Semgrep's servers unless `--disable-version-check` is given, and rules
  named from the registry or by URL are fetched
  ([metrics](https://docs.semgrep.dev/metrics),
  [CLI reference](https://docs.semgrep.dev/cli-reference)). Its
  [editor extensions](https://docs.semgrep.dev/extensions/overview) for VS Code, IntelliJ
  and Emacs speak the Language Server Protocol through `semgrep lsp`. No command returns
  one unit's text; `semgrep mcp` starts an MCP server that scans. `semgrep scan` does not
  fail on findings by default; with `--error` it exits 1, and a
  [pre-commit hook](https://docs.semgrep.dev/extensions/pre-commit) is documented.
- **ArchUnit**, 1.5.1, released 2026-09-25
  ([release](https://github.com/TNG/ArchUnit/releases/tag/v1.5.1)), Apache-2.0
  ([repository](https://github.com/TNG/ArchUnit)); its
  [user guide](https://www.archunit.org/userguide/html/000_Index.html) is versioned 1.5.1.
  ArchUnit is "a free, simple and extensible library for checking the architecture of your
  Java code", and works "by analyzing given Java bytecode, importing all classes into a Java
  code structure". Rules are a Java fluent API, `classes().that()…should()…`, extended in
  Java with `DescribedPredicate` and `ArchCondition`. Layer rules fix the direction of
  dependencies between code layers, as in
  `.whereLayer("Service").mayOnlyBeAccessedByLayers("Controller")`. The guide describes
  only bytecode import and mentions neither comments nor Javadoc; annotations are visible
  to rules. It runs in process, with no network step described. A rule runs as a JUnit
  test, and a violation fails it with an `AssertionError`. `FreezingArchRule` records
  existing violations so that later runs "only report new violations and ignore known
  violations". The guide names no editor extension or language server, and no command that
  returns one unit's text.

Beneath such tools sit code indexes (project documentation, unversioned):
[SCIP](https://github.com/sourcegraph/scip) is "a language-agnostic protocol for indexing
source code",
[LSIF 0.6](https://microsoft.github.io/language-server-protocol/specifications/lsif/0.6.0/specification/)
a format for tools "to dump their knowledge about a workspace",
[Kythe](https://kythe.io/) "a pluggable, (mostly) language-agnostic ecosystem for building
tools that work with code", and [Glean](https://glean.software/) a system that "stores
typed, schema-defined facts about source code in a queryable database".

## agrees: Where grund agrees

- The meaning of grund's chapter rules is stated as QL's is given and as Doop and Soufflé
  write their analyses: as Datalog, over one finite, complete snapshot of extracted facts
  ([§FS-rules.5](../functional-spec/FS-rules.md#5-relational-meaning)). The facts are relations over declarations, chapters, containment and
  citation sites ([§FS-rules.5.1](../functional-spec/FS-rules.md#51-facts-and-identity)), and each sentence family is a clause over them
  ([§FS-rules.5.2](../functional-spec/FS-rules.md#52-family-clauses)). A conclusion about absence or a count is drawn only from a complete
  snapshot ([§FS-rules.4](../functional-spec/FS-rules.md#4-validation-lifecycle)).
- As a QL query reads relational data that extraction produced beforehand, grund keeps
  facts apart from logic: a producer emits a complete, versioned snapshot, and the engine
  that evaluates rules over it knows no sentences, Markdown or file layout
  ([§DF-chapter-rules.2](../decisions/functional/DF-chapter-rules.md#2-decision)).
- As in a reflexion model or under Sangal's design rules, a declared model is held to what
  was extracted, and each disagreement is reported. Citation directions say which kind
  must, should, should not or must not cite which, and the check reports each edge that
  breaks them ([§GRUND-links.1](../grund.md#1-declaring-the-rules), [§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules)). Like design rules "applied repeatedly as
  the system evolves", they hold every edit: a broken `must` fails the build
  ([§GRUND-links.2](../grund.md#2-holding-every-edit-to-them)).

## departs: Where grund departs

- These works start from facts extracted from code: QL, Doop and Soufflé analyse programs,
  reflexion models and dependency matrices start from a static analysis of the source, and
  CodeQL, Semgrep and ArchUnit parse source or read bytecode. grund parses no code. It reads
  comments and doc-comments line by line and understands no classes, types, scopes or calls
  ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)). Its facts are declarations, chapters and citations only
  ([§FS-rules.5.1](../functional-spec/FS-rules.md#51-facts-and-identity)), and it ingests no code index such as SCIP or LSIF ([§FS-rules.12](../functional-spec/FS-rules.md#12-deliberate-phase-1-absences)), so it
  states no rule about a program's structure.
- The query languages are open: a QL query, a Datalog program for Doop or Soufflé, a
  Semgrep rule and an ArchUnit `ArchCondition` are checks the user writes and the engine
  runs. grund runs nothing a user writes during a check, no plugin and no script
  ([§FS-non-goals.12.1](../functional-spec/FS-non-goals.md#121-plugins-or-scripting-hooks-inside-the-engine)). Its rules are sentences of five fixed families ([§FS-rules.3](../functional-spec/FS-rules.md#3-the-five-sentence-families)).
  Datalog states their meaning but is not their authoring language ([§DF-chapter-rules.3](../decisions/functional/DF-chapter-rules.md#3-alternatives-rejected)),
  and no Datalog surface or runtime is part of the release ([§FS-rules.5](../functional-spec/FS-rules.md#5-relational-meaning)). What grund offers
  is a small fixed surface, not the expressive power of a query language.
- The tools leave severity and failure to the rule or the run. A Semgrep rule names its own
  severity, and whether findings fail a run is the caller's choice, `--error`; the CodeQL
  CLI signals no alerts by its exit code, and a repository sets the threshold at which code
  scanning blocks a merge. In grund the severity model, the exit codes and the report order
  are fixed ([§FS-non-goals.9](../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)): a rule's modality alone decides whether it gates, `must` as
  an error and `should` as a suggestion that never changes the exit status ([§FS-rules.7](../functional-spec/FS-rules.md#7-findings-and-channels)).

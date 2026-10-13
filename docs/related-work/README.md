# Related work

Where grund comes from, and what it deliberately is not, is part of its why
([§GRUND-grund](../grund.md#grund-grund-agents-stay-grounded-in-the-spec)). This
folder gives each body of related work, an idea grund descends from or a tool that sits
beside it, one declaration, so that a goal or a decision can say "this is incremental
formalization, and here is where we depart from it" in one `§` instead of retelling it.

An entry is grund's own account of someone else's work, not a copy of it. Its lead says in
a sentence or two what the work is to grund and cites the GRUND or GOAL point it bears on,
which is the direction `grund.toml` sets for the kind. Its body is exactly three chapters,
in this order, and nothing else at that level:

- `## work: What the work is`: the work itself, under its authors' own names and titles,
  linked to a source every claim about it can be checked against, a DOI where there is one;
- `## agrees: Where grund agrees`: what grund takes from the work, or shares with it;
- `## departs: Where grund departs`: what grund does not do that the work does, each
  departure citing the point that rules it out, most often a non-goal, rather than
  restating it.

A reader who holds an entry can therefore guess its addresses, `<§>REL-<slug>.departs`
for one, and they are permanent once anything cites them
([§REQ-spec-section-names.permanence](../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent)).
A part of one chapter, such as a comparison table, is a named section beneath it, never a
fourth chapter and never a numbered one
([§REQ-spec-section-names.shape](../requirements/REQ-spec-section-names.md#shape-where-a-number-is-allowed)).

Read together, the entries place grund in that lineage: grund is incremental
formalization for software documentation, and the documents it checks are flexiformal.
Prose is legal by default, and a declaration or a `§` citation formalizes one spot at a
time. "Gradual typing for prose" says the same in a programmer's terms, although
human-computer interaction named the idea more than a decade before gradual typing
([§REL-incremental-formalization.work](REL-incremental-formalization.md#work-what-the-work-is), [§REL-gradual-typing.work](REL-gradual-typing.md#work-what-the-work-is)). This
is grund's reading of where it comes from, not a claim of scope. Where those ideas are
about formalization in general, grund formalizes exactly one thing, its ID scheme, and
deliberately does not generalize ([§GRUND-grund.2](../grund.md#2-who-it-is-for),
[§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).

Every entry is cited from outside this folder by a point it bears on. Its line in this
index does not count, because an index names every declaration in its folder by
construction
([§FS-check.4.1.2](../functional-spec/FS-check.md#412-an-index-entry-does-not-count)).
A roadmap point does not count on its own either, because the roadmap deletes a point
once it ships.

- [§REL-incremental-formalization](REL-incremental-formalization.md#rel-incremental-formalization-incremental-formalization-the-idea-grund-descends-from) — incremental formalization, the idea grund descends from
- [§REL-flexiformality](REL-flexiformality.md#rel-flexiformality-flexiformality-documents-that-are-partly-formal) — flexiformality, documents that are partly formal
- [§REL-gradual-typing](REL-gradual-typing.md#rel-gradual-typing-gradual-typing-and-gradual-verification-the-programmers-version) — gradual typing and gradual verification, the programmer's version
- [§REL-information-typing](REL-information-typing.md#rel-information-typing-information-typing-units-of-one-kind-under-one-label) — information typing, units of one kind under one label
- [§REL-graph-shapes](REL-graph-shapes.md#rel-graph-shapes-shapes-and-rule-based-schemas-constraints-over-a-structure) — shapes and rule-based schemas, constraints over a structure
- [§REL-schema-later](REL-schema-later.md#rel-schema-later-pay-as-you-go-structure-schema-added-where-it-pays) — pay-as-you-go structure, schema added where it pays
- [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund) — requirements-traceability tools, the neighbours beside grund
- [§REL-spec-driven-development](REL-spec-driven-development.md#rel-spec-driven-development-spec-driven-development-the-current-wave-of-agent-tools) — spec-driven development, the current wave of agent tools
- [§REL-explicit-traceability](REL-explicit-traceability.md#rel-explicit-traceability-explicit-traceability-links-written-by-hand) — explicit traceability, links written by hand
- [§REL-trace-link-recovery](REL-trace-link-recovery.md#rel-trace-link-recovery-trace-link-recovery-links-proposed-after-the-fact) — trace-link recovery, links proposed after the fact
- [§REL-comment-annotations](REL-comment-annotations.md#rel-comment-annotations-annotation-languages-in-comments-a-checked-sublanguage-beside-the-code) — annotation languages in comments, a checked sublanguage beside the code
- [§REL-comment-code-consistency](REL-comment-code-consistency.md#rel-comment-code-consistency-comment-code-consistency-claims-inferred-from-comments) — comment-code consistency, claims inferred from comments
- [§REL-documentation-decay](REL-documentation-decay.md#rel-documentation-decay-documentation-decay-references-that-fall-behind-the-code) — documentation decay, references that fall behind the code
- [§REL-doc-reference-checkers](REL-doc-reference-checkers.md#rel-doc-reference-checkers-documentation-tools-that-check-their-own-references) — documentation tools that check their own references
- [§REL-name-resolution](REL-name-resolution.md#rel-name-resolution-name-resolution-and-robust-locations-references-that-survive-edits) — name resolution and robust locations, references that survive edits
- [§REL-literate-programming](REL-literate-programming.md#rel-literate-programming-literate-programming-and-executable-documentation-prose-and-code-in-one-source) — literate programming and executable documentation, prose and code in one source
- [§REL-design-rationale](REL-design-rationale.md#rel-design-rationale-architecture-decisions-and-rationale-the-why-kept-as-records) — architecture decisions and rationale, the why kept as records
- [§REL-code-queries](REL-code-queries.md#rel-code-queries-queries-and-conformance-over-program-facts-rules-over-what-was-extracted) — queries and conformance over program facts, rules over what was extracted
- [§REL-controlled-language](REL-controlled-language.md#rel-controlled-language-controlled-natural-languages-english-with-a-fixed-grammar) — controlled natural languages, English with a fixed grammar
- [§REL-agent-context](REL-agent-context.md#rel-agent-context-repository-context-for-coding-agents-what-an-agent-is-given-to-read) — repository context for coding agents, what an agent is given to read

## Where grund stands

The entries are read in two tracks, academic work and open tools, and each track is
compared in a table below. Together they say what grund descends from, what already
exists, what is plausibly new, what grund does not do, and what is not yet shown. Each
claim here about other work rests on an entry, and the entry carries the sources.

**What grund descends from.** Beyond the lineage above, grund descends from:

- specification annotations written in comments, from LCLint in 1996 and JML
  ([§REL-comment-annotations](REL-comment-annotations.md#rel-comment-annotations-annotation-languages-in-comments-a-checked-sublanguage-beside-the-code));
- documentation languages that resolve their own references, Scribble and rustdoc's
  intra-doc links among them ([§REL-doc-reference-checkers](REL-doc-reference-checkers.md#rel-doc-reference-checkers-documentation-tools-that-check-their-own-references));
- explicit, hand-written traceability, as practised in safety-critical domains and
  studied in software engineering ([§REL-explicit-traceability](REL-explicit-traceability.md#rel-explicit-traceability-explicit-traceability-links-written-by-hand));
- architecture decision records ([§REL-design-rationale](REL-design-rationale.md#rel-design-rationale-architecture-decisions-and-rationale-the-why-kept-as-records));
- controlled natural languages and Datalog, as the surface and the stated meaning of
  chapter rules ([§REL-controlled-language](REL-controlled-language.md#rel-controlled-language-controlled-natural-languages-english-with-a-fixed-grammar), [§REL-code-queries](REL-code-queries.md#rel-code-queries-queries-and-conformance-over-program-facts-rules-over-what-was-extracted), [§DF-chapter-rules.3](../decisions/functional/DF-chapter-rules.md#3-alternatives-rejected),
  [§FS-rules.5](../functional-spec/FS-rules.md#5-relational-meaning)).

**What already exists.** Nearly every check grund makes has a precedent in at least one
open tool or published work:

- IDs written in code comments of many languages and resolved into one namespace with
  prose items: OpenFastTrace, Sphinx-Needs with sphinx-codelinks, StrictDoc and LOBSTER
  ([§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund)); issue keys in commits ([§REL-explicit-traceability](REL-explicit-traceability.md#rel-explicit-traceability-explicit-traceability-links-written-by-hand)); Duvet
  annotations.
- Addresses that survive a file move or a reworded title: OpenFastTrace item IDs, Sphinx
  explicit labels, Sphinx-Needs IDs and the parts inside a need, StrictDoc UIDs
  ([§REL-doc-reference-checkers](REL-doc-reference-checkers.md#rel-doc-reference-checkers-documentation-tools-that-check-their-own-references)).
- Rules about which kind of item must or may link to which, and coverage obligations in
  either direction: OpenFastTrace's `Needs:`, Sphinx-Needs schema rules, the LOBSTER
  tracing policy and trace query languages; over code dependencies, reflexion models and
  ArchUnit ([§REL-code-queries](REL-code-queries.md#rel-code-queries-queries-and-conformance-over-program-facts-rules-over-what-was-extracted)).
- Trace links enforced in CI: OpenFastTrace, Duvet, LOBSTER and TRLC exit non-zero, and
  ReqToCode, a preprint, makes trace links a build property.
- Dangling-reference lints: rustdoc's `broken_intra_doc_links`, javadoc DocLint's
  `reference` group, Sphinx's nitpicky mode.
- Marks that flag a changed target, the prior art for the section lock
  ([§FS-section-lock](../functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)): Doorstop's link fingerprints, and the revisions OpenFastTrace and
  LOBSTER compare.
- Checks that compare what the referring side states with the target: Duvet's
  quoted-text match and TRLC checks that read a referenced record's fields.
- Informal text linked to checked formal declarations: Lean blueprint's `checkdecls` and
  Isabelle's document antiquotations ([§REL-flexiformality](REL-flexiformality.md#rel-flexiformality-flexiformality-documents-that-are-partly-formal)).
- A formal model of section references: the document calculus gives sections string
  ids, rejects a reference to an unknown id, and proves that an incremental strategy
  keeps the numbers of section references correct, within one document.
- A trace graph used as an agent's context, TraceDev, and agent reads narrower than a
  file: SWE-agent, CODESTRUCT, Serena, OpenSpec ([§REL-agent-context](REL-agent-context.md#rel-agent-context-repository-context-for-coding-agents-what-an-agent-is-given-to-read),
  [§REL-spec-driven-development](REL-spec-driven-development.md#rel-spec-driven-development-spec-driven-development-the-current-wave-of-agent-tools)).

**What is plausibly new: the combination.** We found no single surveyed tool or
publication that does all of the following together:

1. A declaration is an ordinary Markdown heading or doc-comment. Its sections are
   addressed to any depth by a path of numbers or names written on their headings under
   the one declaration ID (`ID.3.1`, `ID.checks.duplicate`), and the address holds no
   file path and no title text ([§GRUND-schema.2](../grund.md#2-keeping-it)). All of it is cited with one marker
   from comments in many host languages and from Markdown, through one resolver
   ([§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)). The nearest tools address an ID written on each unit
   instead: OpenFastTrace items, StrictDoc node UIDs, and Sphinx-Needs IDs with one
   level of need parts. The document calculus models section ids formally, but within
   one document.
2. Project-wide kind-to-kind citation directions at four RFC 2119 levels
   ([§DF-citation-directions](../decisions/functional/DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels)) run in the same offline check as two other rules: opt-in
   obligations that every source file or unit carry a resolving citation
   ([§FS-check.3.6](../functional-spec/FS-check.md#36-ungrounded-unit-opt-in)), and exact value bindings ([§FS-values.3](../functional-spec/FS-values.md#3-explicit-value-bindings)). A binding makes an authored
   literal beside a citation equal a declared value component. The nearest precedents
   check something else: Duvet checks quoted text, TRLC record fields, and OpenFastTrace
   and LOBSTER revisions, while the comment-checking literature infers its claims from
   prose ([§REL-comment-code-consistency](REL-comment-code-consistency.md#rel-comment-code-consistency-comment-code-consistency-claims-inferred-from-comments)).
3. The coordinate the checker resolves is also what an agent reads, at bounded depth
   ([§FS-show.2.2](../functional-spec/FS-show.md#22-section), [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)). Elsewhere, OpenFastTrace documents no command
   that returns one item's text and deliberately ships no MCP server, Sphinx-Needs
   writes per-need JSON at build time, OpenSpec returns a requirement by its position,
   Serena a code symbol, and Context7 query-ranked snippets of library documentation.

As grund reads it, every rule it checks could be expressed in Datalog over extracted
citation and declaration facts, and CodeQL itself sees one language's database at a time
and no Markdown. So the claim is integration and a small, fixed surface, not expressive
power.

**What grund does not do.** grund checks referential integrity and structural
conformance: a citation resolves, an edge obeys the declared rules, a bound literal
equals its declared value. It reads no meaning from prose ([§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)) and parses
no code ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)), so it does not catch semantic drift, code that no longer does
what the cited section says. By grund's reading, most of the inconsistency types in Wen
et al.'s code-comment taxonomy ([§REL-comment-code-consistency](REL-comment-code-consistency.md#rel-comment-code-consistency-comment-code-consistency-claims-inferred-from-comments)) and most of the 162
documentation-issue types of Aghajani et al. ([§REL-documentation-decay](REL-documentation-decay.md#rel-documentation-decay-documentation-decay-references-that-fall-behind-the-code)) concern meaning;
nobody has yet mapped grund's checks onto those types. Nor does grund:

- recover or suggest links ([§GOAL-agent-grounding.3](../goals.md#3-what-this-rules-out), [§REL-trace-link-recovery](REL-trace-link-recovery.md#rel-trace-link-recovery-trace-link-recovery-links-proposed-after-the-fact));
- validate URLs or Markdown anchors ([§FS-non-goals.1](../functional-spec/FS-non-goals.md#1-markdown-link-validation));
- rename IDs ([§FS-non-goals.4](../functional-spec/FS-non-goals.md#4-cross-workspace-id-renaming));
- render documentation or trace matrices ([§FS-non-goals.5](../functional-spec/FS-non-goals.md#5-documentation-generation));
- keep a trace database or history ([§FS-non-goals.6](../functional-spec/FS-non-goals.md#6-decision-database-audit-log-history-tracking));
- produce certification evidence ([§REL-explicit-traceability.departs](REL-explicit-traceability.md#departs-where-grund-departs)).

**What is not yet shown.**

- *A formal semantics.* Resolution, address stability and determinism are specified in
  prose ([§REQ-no-missed-citation](../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked), [§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess), [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes))
  and held by tests. The only formal statement is [§FS-rules.5](../functional-spec/FS-rules.md#5-relational-meaning), "Datalog over one
  finite, complete snapshot", written in prose and covering chapter rules only. No
  theorem exists; the document calculus is the nearest formal base to extend.
  "Gradual typing for prose" is a reading, and grund makes no gradual guarantee
  ([§REL-gradual-typing.departs](REL-gradual-typing.md#departs-where-grund-departs)).
- *A controlled agent-outcome experiment.* [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) and
  [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file) are metered by fixtures, checks and read sizes
  ([§AR-goal-measurement.2](../architecture/AR-goal-measurement.md#2-goal-meters)); no measurement compares agent task success or tokens with
  and without grund.
- *Independent adoption.* There is no adoption, user or interview study.
- *A measured comparison on a shared corpus.* The tables below are qualitative; no
  neighbouring tool has been run beside grund on the same repositories.

## Academic work, compared

One row per line of work, grund first; each row names the entry that holds its works and
their sources. Each cell was checked on 2026-10-13 against the cited work's text, some
paywalled works from their abstracts only, and a cell the source does not support says
"not stated" or "not addressed". A work marked preprint is not peer-reviewed. The
lineage entries above are ideas grund borrows rather than linking mechanisms, so they are
not rows.

- *Links from* is `declared` (an author writes the link), `recovered` (a tool proposes it
  after the fact), `inferred` (a tool derives a claim from text) or `by construction`
  (prose and code share one source).
- *Stability* is what the source says about a link surviving a file move or a reworded
  title.

| Line of work → entry | Linked | Links from | Checked | Unit of address | Stability | Guarantee | Evaluated by |
|---|---|---|---|---|---|---|---|
| **grund** | prose → prose, code → prose, test → prose | declared | presence; structure (sections, kind rules, obligations, chapter rules); value (explicit bindings); not semantics | declaration ID + section path | the address holds no path and no title; a relative heading's move is caught by the section lock | none formal: prose requirements held by tests | the repository's own tests; no external evaluation |
| Trace-link recovery: Antoniol 2002, T-BERT 2021, TransArC 2024, LiSSA 2025 → [§REL-trace-link-recovery](REL-trace-link-recovery.md#rel-trace-link-recovery-trace-link-recovery-links-proposed-after-the-fact) | requirements or docs ↔ code; issues ↔ commits | recovered | none: candidates are ranked or classified | code ↔ manual page or requirement; issue ↔ commit; doc sentence ↔ code file | not addressed | empirical | P/R/F1/MAP against gold links (TransArC end-to-end F1 0.82) |
| Issue keys in commits and their gaps: Bird 2009, Rath 2018, SEOSS 33 → [§REL-explicit-traceability](REL-explicit-traceability.md#rel-explicit-traceability-explicit-traceability-links-written-by-hand) | commits ↔ issues | declared by convention, unenforced | none described; Rath recommends missing links | issue key | not addressed | empirical | mining (about 60% of commits tagged); classifier recall 96%, precision 33% |
| Trace maintenance: Mäder 2008, ILCom 2020 → [§REL-explicit-traceability](REL-explicit-traceability.md#rel-explicit-traceability-explicit-traceability-links-written-by-hand) | issues ↔ code files (ILCom); UML model elements (Mäder) | inferred from IDE interactions and commit issue IDs | none | issue ↔ code file; model element | maintenance is the subject; moves and retitling not discussed | empirical | precision > 90%, recall ≈ 80% against a vetted gold standard (ILCom) |
| Build-verified links: ReqToCode 2026 (preprint) → [§REL-explicit-traceability](REL-explicit-traceability.md#rel-explicit-traceability-explicit-traceability-links-written-by-hand) | requirements ↔ implementation and test code | declared, through one generated symbol per requirement | presence (the build resolves the symbol); lifecycle warnings, then failures | requirement symbol | survives a reworded requirement title; moves not discussed | none | design and example; no empirical evaluation |
| Comment rules inferred and checked: iComment 2007, @tComment 2012, Jdoctor 2018, RustC4++ 2026 → [§REL-comment-code-consistency](REL-comment-code-consistency.md#rel-comment-code-consistency-comment-code-consistency-claims-inferred-from-comments) | comment ↔ the code it documents | inferred | semantics (analysis, generated tests, SMT) | the commented method | not addressed | empirical | bugs and inconsistencies confirmed; precision and recall (Jdoctor 92%/83%) |
| Just-in-time inconsistency detection: Panthaplackel 2021, C4RLLaMA 2025 → [§REL-comment-code-consistency](REL-comment-code-consistency.md#rel-comment-code-consistency-comment-code-consistency-claims-inferred-from-comments) | comment ↔ method, against a change or the current code | inferred (learned model) | semantics, probabilistic | comment and method | not addressed | empirical | detection metrics; manual check of generated updates (65.0% / 55.9% correct) |
| Fragile comments: Fraco 2017 → [§REL-comment-code-consistency](REL-comment-code-consistency.md#rel-comment-code-consistency-comment-code-consistency-claims-inferred-from-comments) | comment text ↔ identifiers | recovered by rules | comment phrases that name an identifier being renamed | identifier name | the subject: mentions break under rename | empirical | precision and recall on six Java systems |
| Link rot and outdated references: Hata 2019, Xiao 2023, DOCER 2024 → [§REL-documentation-decay](REL-documentation-decay.md#rel-documentation-decay-documentation-decay-references-that-fall-behind-the-code) | comments and commits → URLs; README → code element names | declared (URLs); recovered (names) | presence (target reachable; element still in code) | URL; code element name | URLs decay: about 10% of comment links dead | empirical | mining studies; DOCER over more than 3,000 projects |
| Decision records: Jansen & Bosch 2005, Nygard 2011 (blog), Buchgeher 2023, Ahmeti 2024 → [§REL-design-rationale](REL-design-rationale.md#rel-design-rationale-architecture-decisions-and-rationale-the-why-kept-as-records) | decision records in the repository; decisions as first-class architecture | n/a | none | numbered record file | numbers never reused; superseded records kept (Nygard); moves not addressed | none | mining study of ADRs on GitHub; action research |
| Literate programming: Knuth 1984, noweb 1994, Org Babel 2012 → [§REL-literate-programming](REL-literate-programming.md#rel-literate-programming-literate-programming-and-executable-documentation-prose-and-code-in-one-source) | prose and code in one source | by construction | none on whether prose and code agree | numbered or named section (WEB); named code block (Org) | not addressed | none | essay and system descriptions |
| Executable documentation: doctest, rustdoc doctests, Docable 2020 → [§REL-literate-programming](REL-literate-programming.md#rel-literate-programming-literate-programming-and-executable-documentation-prose-and-code-in-one-source) | example in prose ↔ program behaviour | by construction | the example is run | example block | not addressed | none | Docable: 600+ tutorials; 26% of code blocks ran naively; no tutorial ran end to end |
| Documentation languages with resolved references: Scribble 2009, the document calculus 2024 → [§REL-doc-reference-checkers](REL-doc-reference-checkers.md#rel-doc-reference-checkers-documentation-tools-that-check-their-own-references) | prose ↔ library bindings (Scribble); section references ↔ section ids in one document (calculus) | declared | binding resolution (Scribble); a validity rule rejects a reference to an unknown section id (calculus) | identifier + module; section string id | follows the binding, rename not addressed (Scribble); the id stays while numbers are recomputed (calculus) | none (Scribble); theorem on incremental renumbering (calculus) | system and deployment; formal model with theorems |
| Informal–formal linking: Isabelle/DOF 2018, antiquotations 2011, Lean blueprint (tool) → [§REL-flexiformality](REL-flexiformality.md#rel-flexiformality-flexiformality-documents-that-are-partly-formal) | informal text ↔ formal entities | declared | presence and type of the element, document structure (DOF); declarations exist (blueprint) | formal entity name; typed document element | not addressed | none beyond the prover's checks | example ontologies (DOF); use in projects, no evaluation (blueprint) |
| Specifications in comments: LCLint 1996, ESC/Java 2002, JML 2005 → [§REL-comment-annotations](REL-comment-annotations.md#rel-comment-annotations-annotation-languages-in-comments-a-checked-sublanguage-beside-the-code) | annotation ↔ code | declared (stylized comments) | semantics (static checking, verification conditions; JML runtime checks) | parameters, results, routines, invariants | not addressed | none: LCLint and ESC/Java say their checking is neither sound nor complete, by design | checker experience on real programs |
| Name resolution: scope graphs 2015, island grammars 2001 → [§REL-name-resolution](REL-name-resolution.md#rel-name-resolution-name-resolution-and-robust-locations-references-that-survive-edits) | reference ↔ declaration in a program; islands in surrounding text | resolved from a scope graph built from syntax | resolution, ambiguity admitted (not a uniqueness check) | name occurrence and its syntax position | rename defined by α-equivalence | theorem: the algorithm is sound and complete against the calculus | theorem with example bindings; parser case studies |
| Robust locations: Phelps & Wilensky 2000 → [§REL-name-resolution](REL-name-resolution.md#rel-name-resolution-name-resolution-and-robust-locations-references-that-survive-edits) | annotation or hyperlink ↔ location in a document | user creates the link; the client generates descriptors | reattachment, trying descriptors in turn | unique ID, tree walk, surrounding text | designed to survive edits | none (heuristic) | informal testing: 742 of 754 annotations reattached |
| Query languages over program facts: QL 2016, Doop 2009, Soufflé 2016 → [§REL-code-queries](REL-code-queries.md#rel-code-queries-queries-and-conformance-over-program-facts-rules-over-what-was-extracted) | rules over extracted program facts | n/a | whatever the queries encode | program element | n/a | QL's semantics by translation to Datalog; no theorems | scale and performance |
| Architecture conformance: reflexion models 1995, DSM 2005 → [§REL-code-queries](REL-code-queries.md#rel-code-queries-queries-and-conformance-over-program-facts-rules-over-what-was-extracted) | source entities ↔ a high-level model, through a mapping | declared (model and mapping) | structure: dependencies agree or diverge | files mapped by regular expressions; classes by package | not addressed | none | case studies |
| Trace query languages: VTML 2013, TQL 2009 → [§REL-code-queries](REL-code-queries.md#rel-code-queries-queries-and-conformance-over-program-facts-rules-over-what-was-extracted) | artifacts in a separate trace store | not stated | structure, by queries | trace-model element | n/a | none | user experiment against SQL (VTML); example queries (TQL) |
| Controlled natural languages: ACE 2008, SBVR 2010, Naproche 2010 → [§REL-controlled-language](REL-controlled-language.md#rel-controlled-language-controlled-natural-languages-english-with-a-fixed-grammar) | texts ↔ logic; rules ↔ semantic formulations (SBVR) | n/a | unambiguous translation (ACE); proofs checked (Naproche); none (SBVR) | the whole text | n/a | none stated; ACE's translation is fixed by its rules | language definition; prototype |
| Agent–computer interfaces: SWE-agent 2024, CODESTRUCT 2026, Xu 2026 (preprint) → [§REL-agent-context](REL-agent-context.md#rel-agent-context-repository-context-for-coding-agents-what-an-agent-is-given-to-read) | agent ↔ repository through read and edit commands | n/a | n/a | 100-line window; AST entity; LSP symbol | n/a | empirical | agent success and tokens (18.0% vs 12.7% whole-file; −12 to −38% tokens; tokens-to-success) |
| Context files for agents: Gloaguen 2026, Lulla 2026 (preprints) → [§REL-agent-context](REL-agent-context.md#rel-agent-context-repository-context-for-coding-agents-what-an-agent-is-given-to-read) | instruction file ↔ agent | declared (a whole file) | none | whole file | n/a | empirical | success, cost, steps (no general gain, cost up over 20%); wall-clock without correctness (−28.6%) |
| Trace graphs as agent memory: TraceDev 2026 → [§REL-agent-context](REL-agent-context.md#rel-agent-context-repository-context-for-coding-agents-what-an-agent-is-given-to-read) | requirements ↔ design ↔ code | recovered by an agent (LLM and AST matching) | missing links and inconsistencies, reported by the agent | requirement, design entity, code file | not addressed; the graph lives inside one run | empirical | code-generation success on eTOUR and SMOS (53.6%, 56.8%) |

## Open tools, compared

A capability matrix, with rows grouped by entry and not ranked. Each cell states what the
tool's own documentation says, checked 2026-10-13 at the version in the last column of
the third table. The entry each row names holds the dated sources, and the page behind
every cell is listed in the
[per-cell source ledger](https://github.com/agent-grounds/grund/issues/708#issuecomment-6145741432)
of the issue that added this table. Kiro is proprietary, and its rows are marked so;
the licence column says where a part of any other tool is not open.

- *Cells:* `yes: …` (documented), `partial: …` (a narrower form is documented),
  `none documented` (the relevant pages describe no such feature) or `n/a` (the column
  does not apply). "(source)" marks a cell that rests on the tool's source code because
  its documentation is silent.
- *Sub-document address:* the unit smaller than a file that a reference can name.
- *Move* and *Reword:* whether a reference still resolves, unedited, after its target
  moves to another file, or after the target's title is reworded.
- *One namespace:* whether references from code in several languages and from prose
  resolve against one set of targets.
- *Kind rules:* whether a project can declare which kind may, must or must not link to
  which. *Coverage:* whether every unit of a kind can be required to carry or receive a
  link. *Value checks:* whether a value at the referring site is compared with one
  declared at the target.
- *Agent read:* a command or API that returns one addressed unit's text without the rest
  of its file.

### Addressing and resolution

| Tool → entry | Scans | Sub-document address | Move | Reword | One namespace |
|---|---|---|---|---|---|
| **grund** | Markdown, and comments and doc-comments of source files by configured extension (26 extensions by default, `md` among them); Python docstrings | declaration ID + numbered or named section path, any depth | yes: the ID holds no path | yes: the coordinate is written on the heading; a relative heading's move is caught by the section lock | yes: one grammar, one resolver |
| OpenFastTrace → [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund) | items in Markdown, reStructuredText, Gherkin; tags in comments of about 30 formats | item ID `type~name~revision` | yes | yes: title optional, separate from the ID | yes |
| Duvet → [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund) | comments in any language (style per source glob); IETF or Markdown specifications | specification section anchor + quoted text | no: specification URL or path | partial: Markdown anchors are heading slugs unless given an explicit id (source) | partial: comments in all configured languages; citing from prose not documented |
| Doorstop → [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund) | YAML or Markdown item files; `references` keyword search of files | item UID, one item per file | yes: links store the UID (the file name) | yes: the header is outside the UID and the fingerprint | partial: one UID namespace; citations in code are not parsed |
| Sphinx-Needs (+codelinks) → [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund) | reStructuredText, MyST; with codelinks, comments in C/C++, Python, C#, YAML, Rust, Go, JSONC, Bash | need ID; a need part, `NEED.part` | yes | yes with explicit IDs; an omitted ID is a hash of the title | yes, with sphinx-codelinks (a separate package) |
| StrictDoc → [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund) | SDoc; Markdown (experimental); `@relation` markers in Python, C/C++, Rust, Robot, and any file through a generic parser | node UID; anchors; file relations to a line range, function or class | yes | yes: the UID is separate from the title | yes |
| TRLC/LOBSTER → [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund) | TRLC `.rsl` and `.trlc`; LOBSTER tags in C/C++, Python, tests, TRLC, Codebeamer | TRLC record `Package.name`; LOBSTER item tag with optional `@version` | yes | yes (TRLC: by record name) | yes (LOBSTER joins all extracts) |
| lychee → [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund) | Markdown and HTML; other files as plain text | URL anchor or text fragment, opt-in | no: URLs and paths | partial: heading-derived anchors break, explicit ids survive | partial: one URL space, no declared targets |
| rustdoc → [§REL-doc-reference-checkers](REL-doc-reference-checkers.md#rel-doc-reference-checkers-documentation-tools-that-check-their-own-references) | Rust doc comments and `#[doc]` | Rust item path; a `#fragment` is allowed, whether it is checked is not stated | partial: the path follows the module, not the file | partial: renaming the item breaks it | no: Rust items only |
| javadoc DocLint → [§REL-doc-reference-checkers](REL-doc-reference-checkers.md#rel-doc-reference-checkers-documentation-tools-that-check-their-own-references) | Java doc comments, traditional and Markdown | Java element; `##fragment`, whether it is checked is not stated | partial: moving to another package changes the name | partial: renaming the element breaks it | no: Java elements only |
| Sphinx (nitpicky) → [§REL-doc-reference-checkers](REL-doc-reference-checkers.md#rel-doc-reference-checkers-documentation-tools-that-check-their-own-references) | reStructuredText; MyST Markdown; Python docstrings via autodoc | explicit label; domain object | yes with explicit labels | yes with explicit labels; no with autosectionlabel | partial: prose and Python docstrings |
| CodeQL → [§REL-code-queries](REL-code-queries.md#rel-code-queries-queries-and-conformance-over-program-facts-rules-over-what-was-extracted) | source, one database per language (ten languages plus Actions workflows); not Markdown | n/a | n/a | n/a | n/a: a query sees one language database |
| Semgrep CE → [§REL-code-queries](REL-code-queries.md#rel-code-queries-queries-and-conformance-over-program-facts-rules-over-what-was-extracted) | parsed languages; any text in generic or regex mode | n/a | n/a | n/a | n/a: one function or file at a time |
| ArchUnit → [§REL-code-queries](REL-code-queries.md#rel-code-queries-queries-and-conformance-over-program-facts-rules-over-what-was-extracted) | Java bytecode | n/a | n/a | n/a | n/a |
| Spec Kit → [§REL-spec-driven-development](REL-spec-driven-development.md#rel-spec-driven-development-spec-driven-development-the-current-wave-of-agent-tools) | `spec.md`, `plan.md`, `tasks.md`, read by the agent's LLM | `FR-###`, `SC-###`, `T###` | partial: IDs restart in each spec | yes: keyed by ID | none documented |
| OpenSpec → [§REL-spec-driven-development](REL-spec-driven-development.md#rel-spec-driven-development-spec-driven-development-the-current-wave-of-agent-tools) | Markdown under `openspec/` | requirement header name; position in `show` | no: capability path plus header | no: exact header name; a rename needs a `RENAMED` delta | none documented |
| Kiro (proprietary) → [§REL-spec-driven-development](REL-spec-driven-development.md#rel-spec-driven-development-spec-driven-development-the-current-wave-of-agent-tools) | spec and steering Markdown, read by the agent | partial: steering references to a line range | no: relative paths | partial: line ranges shift | none documented |
| Serena → [§REL-agent-context](REL-agent-context.md#rel-agent-context-repository-context-for-coding-agents-what-an-agent-is-given-to-read) | 40+ languages through language servers (Markdown opt-in) | symbol name path within a file | n/a: stores no references | n/a | none documented |
| Context7 → [§REL-agent-context](REL-agent-context.md#rel-agent-context-repository-context-for-coding-agents-what-an-agent-is-given-to-read) | a hosted index of third-party library docs | library ID with version; content chosen by query | n/a | n/a | n/a |
| Aider repo map → [§REL-agent-context](REL-agent-context.md#rel-agent-context-repository-context-for-coding-agents-what-an-agent-is-given-to-read) | 39 tree-sitter languages; not Markdown | files and key symbols; users write no addresses | n/a | n/a | partial: one map; prose not mapped |

### Rules and checks

| Tool | Kind rules | Coverage | Value checks | Rule language |
|---|---|---|---|---|
| **grund** | yes: `[citations.<KIND>]` with `must`, `should`, `should-not`, `must-not` | yes: opt-in grounding of every file or unit; per-target counts in chapter rules | yes: a bound literal must equal the declared component | yes: controlled-English chapter rules, a fixed grammar; no scripting |
| OpenFastTrace | yes: per-item `Needs:` artifact types; unwanted coverage is a defect | yes: an unmet `Needs:` fails the trace | partial: link revision against item revision | partial: Java fluent API and plugins |
| Duvet | none documented | yes: `duvet query` fails on requirements lacking implementation or test annotations | partial: quoted text must occur in the cited section (source) | none documented |
| Doorstop | partial: each document names one parent document | partial: warnings for unlinked items, errors with `-e` | partial: stored parent fingerprint (suspect links) | yes: Python validation hooks |
| Sphinx-Needs (+codelinks) | yes: typed links and JSON-schema network rules | yes: `network_back` with `minContains` | none documented | yes: JSON-schema rules, filter strings, Python |
| StrictDoc | partial: allowed relation types and roles per element | partial: reported (experimental tree map); enforced only by a plugin | none documented | yes: Python plugin; query expressions |
| TRLC/LOBSTER | yes: typed references (TRLC); `trace to` and `requires` policy (LOBSTER) | yes: a missing trace is an error (LOBSTER) | yes: checks compare fields through references (TRLC); `@version` against the target (LOBSTER) | yes: TRLC `checks` expressions; LOBSTER policy |
| lychee | none documented | none documented | partial: a text fragment must occur on the page | partial: regex filters, remapping |
| rustdoc | none documented | none documented | none documented | none documented |
| javadoc DocLint | none documented | none documented | none documented | partial: doclets and taglets in Java |
| Sphinx (nitpicky) | none documented | partial: the coverage builder reports undocumented Python objects | none documented | partial: Python extension events |
| CodeQL | none built in; expressible in QL | n/a: expressible over comments, one language at a time | n/a | yes: QL, with Datalog-based semantics |
| Semgrep CE | none documented | n/a: language rules generally do not match comments | n/a | yes: YAML pattern rules |
| ArchUnit | partial: dependency direction between code layers | n/a: comments are not in bytecode | n/a | yes: Java fluent API |
| Spec Kit | none documented | partial: an LLM prompt reports gaps; advisory | none documented | partial: a prose constitution the LLM weighs |
| OpenSpec | none documented | partial: structural (each requirement has a scenario) | none documented | none documented |
| Kiro (proprietary) | none documented | none documented | none documented | none documented |
| Serena | none documented | none documented | none documented | partial: Python REPL over its API (v2, unreleased) |
| Context7 | none documented | none documented | none documented | none documented |
| Aider repo map | none documented | none documented | none documented | none documented |

### Operation

| Tool | Offline and deterministic | Editor/LSP | Agent read | CI gate | Licence; latest release (date) |
|---|---|---|---|---|---|
| **grund** | yes | yes: `grund-lsp` | yes: `grund <ID>[.<section>]` at four depths | yes: exit 1 | MIT; v0.17.0 (2026-10-10) |
| OpenFastTrace | yes | yes: JetBrains plugin; a third-party LSP, listed by newer docs | none documented (newer docs: no MCP server, by design) | yes: exit 1; Maven, Gradle | GPL-3.0; 4.10.0 (2026-09-20) |
| Duvet | partial: URL specifications downloaded when not cached (source) | none documented | none documented | yes: `report --ci`; `query` exit code; Action | Apache-2.0; v0.4.2 (2026-04-02), crates.io 0.4.3 (2026-08-04) |
| Doorstop | yes | partial: experimental GUI, web server | partial: REST item endpoint (source) | yes: pre-commit hooks; exit 1 on errors (source) | LGPL-3.0; v3.2 (2026-07-10) |
| Sphinx-Needs (+codelinks) | partial: remote imports and services fetch | yes: ubCode (proprietary) | partial: per-need JSON as build output | yes: `-W`; constraint `break` | MIT; 8.5.0 (2026-09-03), codelinks 1.4.0 (2026-07-30) |
| StrictDoc | yes | partial: syntax highlighting only | partial: filtered JSON export (experimental) | partial: exit 1 on a missing UID (source) | Apache-2.0; 0.30.2 (2026-10-09) |
| TRLC/LOBSTER | partial: the Codebeamer extractor needs a server | yes: `trlc-vscode-extension` with `trlc-lsp` | partial: TRLC Python API | yes: TRLC exit 1; `lobster-ci-report` exit 1 | GPL-3.0 and AGPL-3.0; trlc 3.1.0 (2026-09-30), lobster 1.0.6 (2026-07-30) |
| lychee | partial: network by default; `--offline` | none documented | n/a | yes: exit 2; Action; pre-commit | Apache-2.0 OR MIT; 0.24.2 (2026-05-01) |
| rustdoc | yes | partial: rust-analyzer highlighting and hover | partial: rust-analyzer hover | yes: `deny(rustdoc::broken_intra_doc_links)` | MIT OR Apache-2.0; Rust 1.99.0 (2026-10-01) |
| javadoc DocLint | yes (`-link` may fetch; `-linkoffline` avoids it) | partial: the javavscode extension, no reference checks | partial: `Elements.getDocComment` API | yes: `-Werror` | GPL-2.0 with Classpath Exception; JDK 27 (2026-09-15) |
| Sphinx (nitpicky) | partial: intersphinx downloads; autodoc imports modules | none documented | none documented | yes: `-W` exits 1 | BSD-2-Clause; 9.1.0 (2025-12-31) |
| CodeQL | yes once query packs are present | yes: VS Code extension; QL language server | none documented | partial: code-scanning merge protection; no exit code for alerts | CLI proprietary, queries MIT; 2.27.2 (2026-10-07) |
| Semgrep CE | yes with local rules (version check unless disabled) | yes: `semgrep lsp` for VS Code, IntelliJ, Emacs | none documented | yes: `--error` exits 1; pre-commit | LGPL-2.1; 1.180.0 (2026-10-07) |
| ArchUnit | yes | none documented | none documented | yes: a failing JUnit test | Apache-2.0; 1.5.1 (2026-09-25) |
| Spec Kit | no: analysis is an LLM prompt | none documented | none documented (its MCP server exposes only `version`) | none documented | MIT; 1.1.3 (2026-10-09) |
| OpenSpec | yes: `validate` parses locally (telemetry opt-out) | none documented | yes: `show -r <n>`, one requirement by position | yes: `validate` exits 1 | MIT; 1.14.1 (2026-10-06) |
| Kiro (proprietary) | no: operations need an account | yes: the Kiro IDE | none documented | partial: a headless agent in CI | proprietary ("AWS Content"); IDE 1.2.56 (2026-10-08), CLI 2.30.0 (2026-10-12) |
| Serena | partial: local queries; language servers downloaded; usage report | partial: paid JetBrains backend | yes: `find_symbol` with its body (source) | none documented | MIT up to 1.7.0 (2026-08-09); main is GPL-3.0-or-later |
| Context7 | no: a hosted API | yes: a VS Code extension that registers its MCP server; no language server | partial: query-ranked snippets | none documented | MIT repository, private backend; context7-mcp 4.3.0 (2026-10-09) |
| Aider repo map | partial: local, but selection depends on the chat and the token budget | none documented | none documented | none documented (it produces context, not findings) | Apache-2.0; 0.86.0 (2025-08-09), PyPI 0.86.2 (2026-02-12) |

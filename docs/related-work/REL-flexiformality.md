# REL-flexiformality: flexiformality, documents that are partly formal

Flexiformality names documents that carry informal parts for human readers and formal
parts a machine can act on, side by side. A grounded repository is such a document: prose,
with declarations and `§` citations in it that a checker reads ([§GRUND-schema.1](../grund.md#1-declaring-it)).

## work: What the work is

The term comes from Michael Kohlhase and mathematical knowledge management (MKM).
Constantin Jucovschi's 2010 MSc thesis, *Editing Knowledge in Large Mathematical Corpora.
A case study with Semantic LaTeX (sTeX)* ([arXiv:1010.5935](https://arxiv.org/abs/1010.5935)),
quotes the definition from Andrea Kohlhase, Michael Kohlhase and Christoph Lange: a
representation is flexiform when it "can contain informal (i.e., appealing to a human
reader) and formal (i.e., supporting syntax-driven reasoning processes) components or
both". Michael Kohlhase's *The Flexiformalist Manifesto*, SYNASC 2012, pages 30–35, DOI
10.1109/SYNASC.2012.78 ([record](https://cris.fau.de/publications/106181724/)), states the
program.

*Dimensions of Formality: A Case Study for MKM in Software Engineering*, by Andrea
Kohlhase, Michael Kohlhase and Christoph Lange, MKM 2010, pages 355–369, DOI
10.1007/978-3-642-14128-7_31 ([arXiv:1004.5071](https://arxiv.org/abs/1004.5071),
[record](https://cris.fau.de/publications/106429884/)), applies the question to the
documents of a software engineering project. It does not use the word "flexiformal".

sTeX, Kohlhase's semantic LaTeX ([source](https://github.com/slatex/sTeX)), is the closest system
in mechanism: a symbol is declared, a reference names it, and a reference to a symbol that
is not found is an error.

Lars Vogt's *The Semantic Ladder: A Framework for Progressive Formalization of Natural
Language Content for Knowledge Graphs and AI Systems* (2026,
[arXiv:2603.22136](https://arxiv.org/abs/2603.22136), not peer-reviewed) works under the
name "progressive formalization". Reading it as a recent version of the same idea is
grund's reading, not Vogt's claim: the article cites neither Kohlhase nor Shipman.

In a proof assistant the formal part is a checked development, and the informal text
beside it names parts of that development. An author writes each such link.

Achim D. Brucker, Idir Ait-Sadoune, Paolo Crisafulli and Burkhart Wolff's *Using the
Isabelle Ontology Framework: Linking the Formal with the Informal*, CICM 2018, LNAI 11006,
DOI 10.1007/978-3-319-96812-4_3
([authors' copy](https://www.brucker.ch/bibliography/abstract/brucker.ea-isabelle-ontologies-2018)),
presents Isabelle/DOF. It starts from Isabelle's text antiquotations, such as
`@{thm refl}` inside a paragraph, which "can refer to formal content, can be type-checked
before being displayed". DOF adds document ontologies: document classes with attributes
typed in higher-order logic, text elements that are instances of those classes, and
attributes that refer to other instances, called links. "If the link does not exist or has
a non-compatible type, the text is not validated." A monitor class adds a regular
expression over class names, "enforcing a sequence of text-elements that must belong to the
corresponding classes". The ontology language allows "referring (albeit not reasoning) to
entities of Isabelle/HOL", so nothing is guaranteed beyond what the prover checks. Checking
runs in the editor as the author types and as a batch build. The evidence offered is three
example ontologies, for a scholarly article (the article itself is written with it), a
mathematics exam and a CENELEC certification document.

Makarius Wenzel's *Isabelle as Document-Oriented Proof Assistant*, CICM 2011, pages
244–259, DOI 10.1007/978-3-642-22673-1_17
([Springer](https://link.springer.com/chapter/10.1007/978-3-642-22673-1_17)), discusses how to fit the prover
"into a document-oriented environment with rich semantic annotations for formal sources",
so that an editor can present formal content "without having to understand logic itself".
His *Isabelle/jEdit as IDE for Domain-specific Formal Languages and Informal Text
Documents*, F-IDE 2018, EPTCS 284, pages 71–84, DOI 10.4204/EPTCS.284.6
([arXiv:1811.10819](https://arxiv.org/abs/1811.10819)), covers document antiquotations:
informal text "may again refer to formal languages via document antiquotations", and the
text of a markup command "is checked within the context". Section labels, by contrast, "are
not treated formally".

leanblueprint ([PatrickMassot/leanblueprint](https://github.com/PatrickMassot/leanblueprint),
tool documentation, unversioned, checked 2026-10-13) is "a plasTeX plugin allowing to write
blueprints for Lean 4 projects". In a blueprint's LaTeX, `\lean{}` lists "the Lean
declaration names corresponding to the surrounding definition or statement", `\leanok`
"claims the surrounding environment is fully formalized", and `\uses{}` lists the labels a
statement or proof uses, from which the blueprint draws a dependency graph whose node
colours show formalization status. `leanblueprint checkdecls` checks "that every Lean
declaration name that appear in the blueprint exist in the project (or in a dependency of
the project such as Mathlib)", and it "requires a compiled Lean project". `\leanok` stays
the author's claim: the README describes no check that a statement and its declaration say
the same thing. The README says the tool "has been used by many projects", among them the
[PFR](https://teorth.github.io/pfr/) and [FLT](https://imperialcollegelondon.github.io/FLT/)
formalizations, and reports no evaluation. Verso
([documentation](https://verso.lean-lang.org/doc/latest), tool documentation, unversioned,
checked 2026-10-13) is a framework for writing about Lean whose documents "are just Lean
files". It elaborates the Lean code they contain, and it shares cross-reference databases
so that links "can be automatically inserted and maintained".

The unit a link names is a formal entity's name, or a typed document element in DOF.
Neither the DOF article nor the blueprint README discusses whether a link survives its
target moving or being renamed.

## agrees: Where grund agrees

- A document need not be formal throughout to be useful to a machine. grund checks the
  formal parts, the declarations and the citations, and leaves the rest to human readers
  ([§GRUND-links.2](../grund.md#2-holding-every-edit-to-them), [§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)).
- As in sTeX, a reference is to something declared, and one that names nothing is an
  error ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).
- As in DOF and the blueprint, the author writes each link and the check rejects one that
  names nothing: a DOF link to nothing is not validated, a missing Lean name fails
  `checkdecls`, and a citation of an undeclared ID is an unknown reference ([§FS-check.3.1](../functional-spec/FS-check.md#31-dangling-citation)).
- DOF's document classes type its text elements, and its monitor classes hold them to a
  structure. grund's kinds type its declarations ([§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds)), and a chapter rule holds a
  declaration to a count of chapters of one name ([§FS-rules.3.1](../functional-spec/FS-rules.md#31-chapter-presence)). A monitor states an order
  by a regular expression; a chapter rule states a count.
- A blueprint's `\leanok` is a claim, and a passing `checkdecls` says only that the named
  declarations exist. grund's grounding is the same kind of fact: a unit is grounded by a
  citation that resolves ([§FS-check.3.6](../functional-spec/FS-check.md#36-ungrounded-unit-opt-in)).
- DOF checks as the author types and in a batch build, and grund runs in editors, save
  loops, commits and CI ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).

## departs: Where grund departs

- Flexiformality is a program for whatever formal content a document may carry,
  mathematical knowledge first. grund formalizes exactly one thing, its ID scheme, and
  deliberately does not generalize ([§GRUND-grund.2](../grund.md#2-who-it-is-for), [§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).
- Isabelle/DOF lets each project write an ontology of its own in higher-order logic:
  classes, typed attributes, inheritance, and links between instances. grund keeps to its
  ID scheme and is not an ontology language ([§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).
- Checking that a named formal entity exists, or has the right type, takes the prover:
  `checkdecls` needs a compiled Lean project, and DOF's links are checked by Isabelle's own
  parsers and type checkers. grund parses no code ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)). It checks that a cited
  ID is declared in the repository's text, never that a function or theorem of a given name
  exists, and its citations point into prose, from code comments as well as from prose.
- A blueprint's `\uses{}` feeds a dependency graph that the blueprint draws. grund draws no
  graph of its IDs; `grund refs` answers who cites an ID from the current tree
  ([§FS-non-goals.6](../functional-spec/FS-non-goals.md#6-decision-database-audit-log-history-tracking)).
- leanblueprint builds web and PDF versions of a blueprint, and Verso renders its documents.
  grund publishes nothing ([§FS-non-goals.5](../functional-spec/FS-non-goals.md#5-documentation-generation)).

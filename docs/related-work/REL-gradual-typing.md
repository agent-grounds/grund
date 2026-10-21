# REL-gradual-typing: gradual typing and gradual verification, the programmer's version

Gradual typing lets one program mix checked and unchecked parts and move code between them
one annotation at a time. "Gradual typing for prose" is the programmer's slogan for what
grund does to a repository's text: what is marked is checked, and the rest is left alone
([§GRUND-links.2](../grund.md#2-holding-every-edit-to-them)).

## work: What the work is

- Jeremy G. Siek and Walid Taha, *Gradual Typing for Functional Languages*, Scheme and
  Functional Programming Workshop 2006, pages 81–92
  ([PDF](http://scheme2006.cs.uchicago.edu/13-siek.pdf)), introduced gradual typing, with
  "the notation ? for the dynamic type".
- Jeremy G. Siek, Michael M. Vitousek, Matteo Cimini and John Tang Boyland, *Refined
  Criteria for Gradual Typing*, SNAPL 2015, LIPIcs 32, pages 274–293, DOI
  10.4230/LIPIcs.SNAPL.2015.274
  ([Dagstuhl](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.SNAPL.2015.274)),
  names the gradual guarantee, "that relates the behavior of programs that differ only with
  respect to their type annotations": "the less precise program behaves the same as the
  more precise one except that it might have fewer trapped errors".
- Johannes Bader, Jonathan Aldrich and Éric Tanter, *Gradual Program Verification*, VMCAI
  2018, pages 25–46, DOI 10.1007/978-3-319-73721-8_2
  ([PDF](https://pleiad.cl/papers/2018/baderAl-vmcai2018.pdf)), carries the idea from types
  to specifications: "the programmer can control the trade-off between static and dynamic
  checking by tuning the (im)precision of pre- and postconditions". Of these first three
  works, it is the closest fit.
- Gilad Bracha, *Pluggable Type Systems*, a position statement for a 2004 OOPSLA workshop
  ([PDF](https://www.cs.tufts.edu/~nr/cs257/archive/gilad-bracha/pluggableTypesPosition.pdf)):
  an optional type system "has no effect on the run-time semantics of the programming
  language".
- Ronald Garcia, Alison M. Clark and Éric Tanter, *Abstracting Gradual Typing*, POPL 2016,
  pages 429–442, DOI 10.1145/2837614.2837670
  ([dblp](https://dblp.org/doi/10.1145/2837614.2837670)), derive a gradual type system from
  a static one by abstract interpretation, giving "gradual types a semantics in terms of
  pre-existing static types". Run-time checks arise "by deducing evidence for consistent
  judgments during proof reduction", not from "an externally justified cast calculus".
- Sam Tobin-Hochstadt, Matthias Felleisen, Robert Bruce Findler, Matthew Flatt, Ben
  Greenman, Andrew M. Kent, Vincent St-Amour, T. Stephen Strickland and Asumu Takikawa,
  *Migratory Typing: Ten Years Later*, SNAPL 2017, DOI 10.4230/LIPIcs.SNAPL.2017.17
  ([Dagstuhl](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.SNAPL.2017.17)),
  look back on "typed twins for untyped programming languages so that developers could
  migrate scripts from the untyped world to a typed one in an incremental manner", as
  Typed Racket implements them, with design principles that include "the units of
  migration". Sam Tobin-Hochstadt and Matthias Felleisen's *The Design and Implementation of
  Typed Scheme*, POPL 2008, pages 395–406, DOI 10.1145/1328438.1328486
  ([PDF](https://www2.ccs.neu.edu/racket/pubs/popl08-thf.pdf)), presents "an explicitly
  typed extension of an untyped scripting language".
- Matthew M. Papi, Mahmood Ali, Telmo Luis Correa, Jeff H. Perkins and Michael D. Ernst,
  *Practical Pluggable Types for Java*, ISSTA 2008, pages 201–212, DOI
  10.1145/1390630.1390656
  ([PDF](https://homes.cs.washington.edu/~mernst/pubs/pluggable-checkers-issta2008.pdf)),
  introduce the Checker Framework: "A type system designer defines type qualifiers and their
  semantics, and a compiler plug-in enforces the semantics". Werner Dietl, Stephanie
  Dietzel, Michael D. Ernst, Kıvanç Muşlu and Todd W. Schiller, *Building and Using
  Pluggable Type-Checkers*, ICSE 2011, pages 681–690, DOI 10.1145/1985793.1985889
  ([PDF](https://homes.cs.washington.edu/~mernst/pubs/pluggable-checkers-icse2011.pdf)),
  report experience with such checkers, which let programmers "detect and prevent, at
  compile time, defects that would otherwise have been manifested as run-time errors".
- On refinement types: Tim Freeman and Frank Pfenning, *Refinement Types for ML*, 1991,
  DOI 10.1145/113445.113468
  ([dblp](https://dblp.org/doi/10.1145/113445.113468)), allow "the specification of
  recursively defined subtypes of user-defined datatypes" and keep type inference
  decidable. Patrick M. Rondon, Ming Kawaguchi and Ranjit Jhala, *Liquid Types*, 2008, DOI
  10.1145/1375581.1375602 ([dblp](https://dblp.org/doi/10.1145/1375581.1375602)), combine
  "Hindley-Milner type inference with Predicate Abstraction to automatically infer
  dependent types"; on their benchmarks the manual annotation needed fell "from 31% of
  program text to under 1%".
- K. Rustan M. Leino, *Dafny: An Automatic Program Verifier for Functional Correctness*,
  LPAR 2010, pages 348–370, DOI 10.1007/978-3-642-17511-4_20
  ([Springer](https://link.springer.com/chapter/10.1007/978-3-642-17511-4_20)), tours a language whose
  specifications its verifier discharges with an SMT solver. Andrea Lattuada, Travis Hance,
  Chanhee Cho, Matthias Brun, Isitha Subasinghe, Yi Zhou, Jon Howell, Bryan Parno and Chris
  Hawblitzel, *Verus: Verifying Rust Programs using Linear Ghost Types*, OOPSLA 2023,
  Proceedings of the ACM on Programming Languages 7(OOPSLA1), pages 286–315, DOI
  10.1145/3586037 ([arXiv:2303.05491](https://arxiv.org/abs/2303.05491), extended version),
  present "an SMT-based tool for formally verifying Rust programs", in which "programmers
  express proofs and specifications using the Rust language".
- Gerwin Klein, June Andronick, Kevin Elphinstone, Toby Murray, Thomas Sewell, Rafal
  Kolanski and Gernot Heiser, *Comprehensive Formal Verification of an OS Microkernel*,
  ACM Transactions on Computer Systems 32(1), 2014, DOI 10.1145/2560537
  ([PDF](https://sel4.systems/Research/pdfs/comprehensive-formal-verification-os-microkernel.pdf)),
  give the machine-checked verification of seL4: the functional correctness of its C
  implementation, a proof that its binary implements the C semantics, and proofs of
  access-control enforcement and information-flow noninterference, "backed by
  machine-checked, end-to-end theorems". In 2014 they called it "the only
  example of formal proof of this scale that is kept current as the requirements, design
  and implementation of the system evolve over almost a decade". Its first account is Klein
  et al., *seL4: Formal Verification of an OS Kernel*, SOSP 2009, pages 207–220, DOI
  10.1145/1629575.1629596
  ([abstract](https://trustworthy.systems/publications/nictaabstracts/Klein_EHACDEEKNSTW_09.abstract)),
  which verifies the kernel "from an abstract specification down to its C implementation".

## agrees: Where grund agrees

- The mapping, as grund reads it: unmarked prose is the unchecked part, and a `§`
  citation is the annotated spot where a check applies. A bare ID-shaped token is plain
  text ([§DF-reference-marker.2.4](../decisions/functional/DF-reference-marker.md#24-strict-vs-optional)), and the `<§>` escape is the explicit way to write an ID
  without checking it, as `?` is the explicit way to leave a type unchecked
  ([§DF-escape-position-is-not-a-citation](../decisions/functional/DF-escape-position-is-not-a-citation.md#df-escape-position-is-not-a-citation-an-escape-position-is-not-a-citation-in-either-strict-mode)).
- As Bader, Aldrich and Tanter's specifications may be as precise as their author
  chooses, a grund declaration may carry few citations or many, and the check holds what
  is written.
- As Bracha's optional types, grund's checks change nothing about the files they read: a
  page renders and a source file compiles the same whether or not grund accepts it.
- As migratory typing moves a program to types one unit at a time, a repository adopts
  grund from no configuration ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)), and its grounding obligation is turned on
  per kind and sized per unit later ([§FS-check.3.6](../functional-spec/FS-check.md#36-ungrounded-unit-opt-in)).
- As a Checker Framework type system is declared by its designer and enforced by a
  plug-in, a project declares its kinds and citation directions as data and a fixed checker
  enforces them ([§GRUND-schema.1](../grund.md#1-declaring-it), [§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules)).
- As grund reads it, an explicit value binding is a singleton refinement: a backticked
  literal beside a citation of a value root must equal the declared value ([§FS-values.3.1](../functional-spec/FS-values.md#31-the-only-binding-grammar)).
  It is compared by exact equality, not by a solver ([§FS-values.4](../functional-spec/FS-values.md#4-exact-equality)). The framing is grund's,
  not the refinement-type works' own.

## departs: Where grund departs

- Gradual typing checks what crosses from the unannotated part at run time, by casts or,
  in AGT, by evidence. grund has no run time: a `§` citation is checked statically, before
  anything runs, and must resolve ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).
- The gradual guarantee says a less precise program behaves the same as a more precise
  one. grund makes no such promise: a `must` citation direction ([§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels)) or the opt-in grounding
  floor ([§FS-check.3.6](../functional-spec/FS-check.md#36-ungrounded-unit-opt-in)) makes some formality mandatory, so deleting a citation can fail
  the check.
- The levels are not errors and lints. `must` and `must-not` are errors; `should` and
  `should-not` are suggestions that never appear in `grund check`'s standing output and
  surface only under `--suggestions`. The mapping is fixed ([§FS-config.3.9.1.3](../functional-spec/FS-config.md#3913-the-levelsurface-mapping-is-fixed),
  [§FS-non-goals.9](../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)).
- Every work above checks the program itself: gradual and migratory typing type its terms,
  the Checker Framework and refinement types check its qualifiers and values, Dafny and
  Verus prove its specifications, and seL4's proofs reach its C code and binary. grund
  parses no code ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)). A citation or a value binding in a source file is read
  from a comment, never from the host language ([§FS-values.3.2](../functional-spec/FS-values.md#32-recognized-text-contexts)), so grund cannot say that
  the code agrees with what it cites. seL4 is the fully verified end of that spectrum, and
  grund sits far from it.
- The Checker Framework lets a designer write a new checker as a compiler plug-in. grund's
  checks are fixed, with no plugin or scripting hook inside the engine
  ([§FS-non-goals.12.1](../functional-spec/FS-non-goals.md#121-plugins-or-scripting-hooks-inside-the-engine)).

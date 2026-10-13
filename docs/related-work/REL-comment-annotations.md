# REL-comment-annotations: annotation languages in comments, a checked sublanguage beside the code

Annotation languages put a small formal language inside a host language's comments, where
the compiler ignores it and a separate checker reads it. A `§` citation is such a
sublanguage: one marker and one grammar, written in the comments of many host languages
and in Markdown, and read by one resolver ([§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)).

## work: What the work is

David Evans, *Static Detection of Dynamic Memory Errors*, 1996, DOI 10.1145/231379.231389
([dblp](https://dblp.org/doi/10.1145/231379.231389)), extends the LCLint checker, later
Splint, with "annotations to make certain assumptions explicit at interface points". They
can be written "directly in the source code as syntactic comments", `/*@null@*/` for one,
and they go on type declarations, on "function parameter or return value declarations",
and on "global and static variable declarations"; they are "syntactically similar to C
type qualifiers". A static checker uses them to detect "misuses of null pointers, uses of
dead storage, memory leaks, and dangerous aliasing". The article is "willing to accept an
analysis that is neither sound nor complete", and its evidence is experience: the checking
was used to rework the memory management of LCLint itself, "over 100 000 lines of source
code". The Splint 3.1.1 manual of 2003, tool documentation
([section 1](https://splint.org/manual/html/sec1.html)), says that "All stylized comments
begin with /\*@" and that annotations "may only be used in fixed grammatical contexts".

Cormac Flanagan, K. Rustan M. Leino, Mark Lillibridge, Greg Nelson, James B. Saxe and
Raymie Stata, *Extended Static Checking for Java*, 2002, DOI 10.1145/512529.512558
([dblp](https://dblp.org/doi/10.1145/512529.512558)), describe ESC/Java, a compile-time
checker "powered by verification-condition generation and automatic theorem-proving
techniques". Its annotations are "enclosed in Java comments that begin with an @-sign":
routine preconditions, object invariants such as
`//@ invariant 0 <= size && size <= elements.length`, and ghost fields "known only to the
checker, not to the compiler". The checker "warns of inconsistencies between the design
decisions recorded in the annotations and the actual code". Of soundness and completeness,
the authors "do not take either of these attributes to be a requirement". They report
"experience applying the checker to tens of thousands of lines of Java programs": its own
front end, Javafe, of 41 thousand lines, and portions of the Mercator web crawler, which
together needed "roughly 40–100 annotations" per thousand lines of code.

Lilian Burdy, Yoonsik Cheon, David R. Cok, Michael D. Ernst, Joseph R. Kiniry, Gary T.
Leavens, K. Rustan M. Leino and Erik Poll, *An Overview of JML Tools and Applications*,
International Journal on Software Tools for Technology Transfer 7(3):212–232, 2005, DOI
10.1007/s10009-004-0167-4 ([Springer](https://link.springer.com/article/10.1007/s10009-004-0167-4)), survey
the Java Modeling Language: "JML assertions are written as special annotation comments in
Java code, either after //@ or between /\*@ ... @\*/, so that they are ignored by Java
compilers but can be used by tools that support JML." JML states "the preconditions and
postconditions for methods as well as class invariants, in the Design by Contract style".
Its tools add runtime assertion checking to static checking and verification, ESC/Java
among them; for one of them, the LOOP tool, the logic used is reported "completely
formalized and proven sound" against a semantics of Java and JML in the PVS prover.

The style JML names is Bertrand Meyer's: *Applying "Design by Contract"*, IEEE Computer
25(10):40–51, 1992, DOI 10.1109/2.161279 ([doi.org](https://doi.org/10.1109/2.161279)),
presents "the theory of design by contract, which underlies the design of the Eiffel
analysis, design, and programming language", and discusses "the role of assertions in that
theory".

PEP 484, *Type Hints*, by Guido van Rossum, Jukka Lehtosalo and Łukasz Langa, created in
2014 and final, is a language proposal, not peer-reviewed
([peps.python.org](https://peps.python.org/pep-0484/)). It adds no syntax for a variable's
type; "To help with type inference in complex cases", a comment may carry it,
`x = []  # type: List[Employee]`, placed "on the last line of the statement that contains
the variable definition". Under the proposal "no type checking happens at runtime", and
it assumes "a separate off-line type checker which users can run over their source code
voluntarily".

None of these works discusses how an annotation would survive an edit elsewhere in the
program. Each is written in place, on the declaration or statement it describes.

## agrees: Where grund agrees

- A comment is a good home for a checked sublanguage: the host compiler ignores it, as
  JML's annotation comments "are ignored by Java compilers", and a separate tool reads it,
  as PEP 484's off-line checker reads its type comments. A `§` citation has the same
  marker, the same section grammar and the same resolver in Markdown and in the comments
  of every supported host language ([§GOAL-polyglot-citation.1](../goals.md#1-what-cleanly-means), [§GRUND-links.2](../grund.md#2-holding-every-edit-to-them)).
- A marker opens the sublanguage: `/*@` for LCLint, an `@` at the start of the comment
  for ESC/Java and JML, and `§` for grund, a sign almost never followed by an ID-shaped
  token in unrelated text ([§DF-reference-marker.2.1](../decisions/functional/DF-reference-marker.md#21-marker)).
- The annotation records a decision where it applies. ESC/Java's language is one "with
  which programmer design decisions can be expressed formally", and LCLint's makes
  assumptions explicit at interface points. In grund, code cites the point that says why
  it is the way it is, on its doc-comment or beside the line that enforces it
  ([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)), and a specification may itself be declared in a doc-comment, its body
  beside the code it describes, as a JML specification is ([§AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)).

## departs: Where grund departs

- These checkers analyse the code an annotation describes: LCLint's static checking of how
  C code uses storage, ESC/Java's verification conditions, the runtime assertion checks of
  the JML tools. grund parses no code. It reads comments line by line and knows no
  classes, methods, types or scopes ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)), so a `§` citation stands wherever
  its marker is written, where an LCLint annotation attaches to a declaration as a type
  qualifier does.
- They read what an annotation means: a precondition is a claim about program state, which
  the checker tries to prove or tests at run time. grund reads no meaning from the text it
  checks. The prose between IDs is opaque, and its one comparison is an explicitly
  delimited value binding, an authored literal held against the declared value it cites
  ([§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality), [§FS-values.3](../functional-spec/FS-values.md#3-explicit-value-bindings)). A resolving citation says that a line is grounded in a
  point, never that the line does what the point says.
- LCLint and ESC/Java accept checking that is neither sound nor complete, for questions
  about what a program does when it runs. grund's question is far smaller, whether a
  citation resolves, and for it grund's requirements ask for neither a missed nor a false
  finding: every citation a run reads is checked ([§REQ-no-missed-citation](../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked)), and one that
  resolves is never reported as broken ([§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms)). These are written
  requirements, not a proof of either property.

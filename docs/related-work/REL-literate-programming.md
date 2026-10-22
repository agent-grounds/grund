# REL-literate-programming: literate programming and executable documentation, prose and code in one source

Literate programming writes prose and code as one source, in the order a human reader
needs, and derives both the program and its document from it. It is the other answer to
the gap grund works on: where code and the prose that says why live apart, literate
programming fuses them, and grund leaves them apart and checks the edge between them
([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)).

## work: What the work is

Donald E. Knuth, *Literate Programming*, The Computer Journal 27(2), 1984, pages 97–111,
DOI 10.1093/comjnl/27.2.97 ([dblp](https://dblp.org/doi/10.1093/comjnl/27.2.97),
[open copy](http://www.literateprogramming.com/knuthweb.pdf)), "presents WEB by example":
"Let us change our traditional attitude to the construction of programs: Instead of
imagining that our main task is to instruct a computer what to do, let us concentrate
rather on explaining to human beings what we want a computer to do." One WEB file serves
twice: WEAVE turns it into a typeset document, and TANGLE into the Pascal program. A WEB
program is "broken into small sections, each of which has a serial number", a section of
code may be named and used by name in another, and the woven document carries generated
cross-references ("This code is used in section 2.") and indexes. The article describes
no check that the prose and the code agree.

Two descendants:

- Norman Ramsey, *Literate Programming Simplified*, IEEE Software 11(5), 1994, pages
  97–105, DOI 10.1109/52.311070 ([IEEE](https://doi.org/10.1109/52.311070)), describes
  noweb. Literate-programming tools "let you arrange the parts of a program in any order
  and extract documentation and code from the same source file"; Ramsey argues that
  language dependence and feature complexity hampered their acceptance, and offers a
  simpler alternative (from the abstract).
- Eric Schulte, Dan Davison, Thomas Dye and Carsten Dominik, *A Multi-Language Computing
  Environment for Literate Programming and Reproducible Research*, Journal of Statistical
  Software 46(3), 2012, DOI 10.18637/jss.v046.i03, describe Org-mode's Babel. "A single
  hierarchically-organized plain text source file" holds prose and code in arbitrary
  languages. Code blocks run in place, with their output captured in the file, and a block
  may take the result of a named code block as an argument. The file is exported to LaTeX,
  HTML and other formats, and its code can be extracted as pure code files.

Executable documentation runs the examples a document carries. Its tools are described
here from their own documentation (tool documentation, checked 2026-10-13):

- Python's doctest ([Python 3.15.0 documentation](https://docs.python.org/3/library/doctest.html))
  "searches for pieces of text that look like interactive Python sessions, and then
  executes those sessions to verify that they work exactly as shown".
- Elixir's ExUnit.DocTest ([ExUnit v1.20.4](https://hexdocs.pm/ex_unit/ExUnit.DocTest.html))
  generates tests "from code examples found in @moduledoc and @doc attributes".
- rustdoc's documentation tests (the rustdoc book, unversioned,
  [documentation tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html))
  extract each code block of a doc comment and run it as a test, and ordinary doctests
  pass "if they compile and run without panicking".
- `mdbook test` (the mdBook guide, unversioned,
  [test](https://rust-lang.github.io/mdBook/cli/test.html)) "will run all available tests
  in a book"; "only Rust tests are supported".

Samim Mirhosseini and Chris Parnin, *Docable: Evaluating the Executability of Software
Tutorials*, ESEC/FSE 2020, pages 375–385, DOI 10.1145/3368089.3409706
([open copy](https://par.nsf.gov/servlets/purl/10196169)), ran over 600 tutorials holding
nearly 15,000 code blocks. A naive execution strategy ran 26% of the code blocks (3,916 of
14,876). A strategy using human annotation doubled that, and still no tutorial executed
all its steps. The barriers they found range from interactive prompts to missing steps and
inaccessible resources.

Prose and checked code in one source also reach proof assistants and language standards:

- Clément Pit-Claudel, *Untangling Mechanized Proofs*, SLE 2020, pages 155–174, DOI
  10.1145/3426425.3426940 ([dblp](https://dblp.org/doi/10.1145/3426425.3426940)),
  describes Alectryon: a compiler that "interleaves Coq's output with the original proof
  script to produce interactive webpages", and a literate toolkit that translates "back
  and forth between reStructuredText documents and literate Coq source files", so an
  author can switch between prose- and code-oriented views of the same sources.
- Agda's documentation (pages titled 2.9.0,
  [literate programming](https://agda.readthedocs.io/en/latest/tools/literate-programming.html),
  tool documentation, checked 2026-10-13) describes "a limited form of literate
  programming, i.e. code interspersed with prose", in literate TeX, reStructuredText,
  Markdown, Typst, Org and Forester files. Agda checks the code in such a file and ignores
  the text outside its code blocks.
- Dongjun Youn et al., *Bringing the WebAssembly Standard up to Speed with SpecTec*,
  Proceedings of the ACM on Programming Languages 8, 2024, DOI 10.1145/3656440
  ([dblp](https://dblp.org/doi/10.1145/3656440)), present SpecTec, a domain-specific
  language that "serves as a single source of truth". From one definition of the
  WebAssembly semantics it generates the typeset specification, formal rules and prose
  pseudocode alike, and an interpreter that passes all of the applicable official
  WebAssembly 2.0 test suite. It detected historical errors in the specification and ten
  errors in five proposals.

In WEB, noweb and Babel the link between prose and code holds by construction, because
both come from one file; in the documentation tests the example sits in the documentation
and is run. The sources read do not discuss whether a section's number, a block's name or
an example survives an edit, and none states a formal guarantee: Knuth argues by example,
Ramsey and Schulte et al. describe systems, and Docable measures.

## agrees: Where grund agrees

- Code and the prose that explains it belong together, and a reader should get from one
  to the other. Knuth's generated cross-references lead from a piece of code to the
  sections that use it. grund's citations lead from code to the point that says why
  ([§GRUND-links.2](../grund.md#2-holding-every-edit-to-them)), and `grund refs` lists every site that cites a point ([§FS-refs](../functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)).
- Prose can sit with the code it describes. An architecture spec may be declared inline
  in the doc-comment of the class it describes ([§AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)), and one citation grammar
  reads Markdown and doc-comments alike ([§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)). Alectryon's toolkit
  likewise lets one set of sources be read as prose or as code; the parallel is grund's
  reading, not Alectryon's claim.
- An address finer than a file: WEB numbers and names its sections, and Babel names its
  code blocks. grund addresses a section of a declaration by the number or name written on
  its heading ([§FS-show.2.2](../functional-spec/FS-show.md#22-section)).
- Documentation that nothing checks goes wrong unnoticed: Docable found that only 26% of
  tutorial code blocks ran, and recommends automated tutorial testing. grund shares the
  premise and checks the references a document makes ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)). A
  documentation test and a `§` citation can sit in the same doc-comment: one runs the
  example, the other resolves the reference.
- As in SpecTec, a fact has one defining source. grund keeps it by comparison rather than
  by generation: an explicit value binding is checked equal to its one declaration
  ([§FS-values](../functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value)).

## departs: Where grund departs

- WEB, noweb and Babel derive the program and the document from one source, and Alectryon
  and SpecTec generate webpages and a typeset specification. grund keeps code and prose in
  their own files and checks the edge between them ([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)), and it generates no
  rendered documentation; a static-site generator can use `grund <ID>` as a building
  block ([§FS-non-goals.5](../functional-spec/FS-non-goals.md#5-documentation-generation)).
- Babel runs code blocks in place, Alectryon runs Coq to capture its output, Docable runs
  tutorials, and doctest, ExUnit.DocTest, rustdoc and `mdbook test` run the examples in
  documentation. grund executes nothing during a check: it performs no network I/O and
  runs no configured integration ([§FS-non-goals.11](../functional-spec/FS-non-goals.md#11-network-access-during-a-check)). The barriers Docable found,
  interactive prompts and inaccessible resources among them, show that a run depends on
  the machine and the network it runs on, and grund rules out anything that would let two
  installs of the same version and configuration disagree ([§FS-non-goals.13](../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). It checks
  that the text around an example resolves its references, not that the example runs.
- Alectryon runs Coq over the proof script, and Agda checks the code of a literate file.
  grund parses no code: it scans comments and doc-comments for IDs ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)).

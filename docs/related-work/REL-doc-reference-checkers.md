# REL-doc-reference-checkers: documentation tools that check their own references

Documentation tools that resolve their own references are grund's nearest neighbours on
its most basic check: a reference that names nothing fails
([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)). Each resolves within one host language, one documentation
build or one document, where grund reads one citation grammar in Markdown and in the
comments of many host languages ([§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)).

## work: What the work is

Three tools that can fail a documentation build on a reference they cannot resolve, and
two publications that give such references a meaning. The tool claims say what each
tool's documentation states, at the version named, checked 2026-10-13; they do not rank
the tools.

- **rustdoc intra-doc links**, Rust 1.99.0 (2026-10-01), MIT OR Apache-2.0
  ([licences](https://rust-lang.org/policies/licenses/),
  [releases](https://github.com/rust-lang/rust/releases)). Rust RFC 1946,
  [*intra-rustdoc links*](https://rust-lang.github.io/rfcs/1946-intra-rustdoc-links.html),
  proposed "a notation how to create relative links in documentation comments (based on
  Rust item paths)". The rustdoc book's page on
  [linking to items by name](https://doc.rust-lang.org/rustdoc/write-documentation/linking-to-items-by-name.html)
  describes the result. A link sits in a `///` or `//!` doc comment or a `#[doc]`
  attribute, which can also pull in Markdown with `#[doc = include_str!(..)]`
  ([the doc attribute](https://doc.rust-lang.org/rustdoc/write-documentation/the-doc-attribute.html)).
  It names a Rust item by path, "You can refer to anything in scope", and may append a URL
  fragment, `#fragment`; whether the fragment is checked is not stated. "Links are resolved
  in the scope of the module where the item is defined, even when the item is
  re-exported", so the address follows the module rather than the file, and renaming the
  item breaks it. "Paths in Rust have three namespaces: type, value, and macro": a link
  reaches Rust items only. The page describes no network step, and "If a link from another
  crate fails to resolve, no warning is given." The
  [lints](https://doc.rust-lang.org/rustdoc/lints.html) page says
  `rustdoc::broken_intra_doc_links` "warns by default" and "detects when an intra-doc link
  fails to be resolved"; `#![deny(rustdoc::broken_intra_doc_links)]` makes it an error, and
  Cargo's
  [`RUSTDOCFLAGS`](https://doc.rust-lang.org/cargo/reference/environment-variables.html)
  passes flags to every rustdoc run. No stable rustdoc command prints one item's
  documentation
  ([command-line arguments](https://doc.rust-lang.org/rustdoc/command-line-arguments.html)),
  and the unstable `--output-format json` emits the whole crate
  ([unstable features](https://doc.rust-lang.org/rustdoc/unstable-features.html)). The
  [rust-analyzer book](https://rust-analyzer.github.io/book/features.html), unversioned,
  documents a semantic-token modifier for intra-doc links and a hover that shows one
  definition's documentation; it documents no diagnostic for a broken link.
- **javadoc and DocLint**, JDK 27, generally available 2026-09-15
  ([JDK 27](https://openjdk.org/projects/jdk/27/)); OpenJDK is GPL-2.0 with the Classpath
  Exception ([licence](https://openjdk.org/legal/gplv2+ce.html)). The
  [documentation comment specification](https://docs.oracle.com/en/java/javase/27/docs/specs/javadoc/doc-comment-spec.html)
  has "two kinds of documentation comments: traditional documentation comments, and
  Markdown documentation comments". A reference names a Java element,
  `module/package.class#member`, and "is evaluated in the scope in which the documentation
  comment exists", so moving the element to another package changes its name and renaming
  it breaks the reference; references reach Java program elements only. A `##` separator
  instead names a URI fragment, such as a heading, in an element's generated page; whether
  the fragment is checked is not stated. The
  [javadoc man page](https://docs.oracle.com/en/java/javase/27/docs/specs/man/javadoc.html)
  says "By default, the `-Xdoclint` option is enabled", and that DocLint "checks all
  comments that are used in the generated documentation". Its groups are fixed:
  accessibility, html, missing, reference and syntax. The `reference` group "Checks for
  issues relating to the references to Java API elements from documentation comment tags",
  such as a `@see` or `{@link ...}` target that "cannot be found", and these "are typically
  reported as errors"; `-Werror` "Reports an error if any warnings occur". The man page
  documents no exit status; the
  [`javax.tools.Tool`](https://docs.oracle.com/en/java/javase/27/docs/api/java.compiler/javax/tools/Tool.html)
  API returns "0 for success; nonzero otherwise". Users extend javadoc with doclets and
  taglets written in Java. The man page describes no network step for DocLint itself;
  `-link` reads a list file at a URL, and `-linkoffline` "removes the constraint that the
  `javadoc` tool must have a web connection to generate documentation". No command
  returns one element's comment; the
  [`Elements.getDocComment`](https://docs.oracle.com/en/java/javase/27/docs/api/java.compiler/javax/lang/model/util/Elements.html)
  API "Returns the text of the documentation ("JavaDoc") comment of an element" inside a
  compiler run. Oracle's [VS Code extension](https://github.com/oracle/javavscode)
  documents completion of Javadoc comments, not reference checks.
- **Sphinx**, 9.1.0 (2025-12-31), BSD-2-Clause ([PyPI](https://pypi.org/project/Sphinx/),
  [releases](https://github.com/sphinx-doc/sphinx/releases)). The pages read are the
  `master` documentation, with 9.1.0 the latest release. Sphinx reads
  reStructuredText, Markdown through the separate MyST-Parser project
  ([Markdown](https://www.sphinx-doc.org/en/master/usage/markdown.html)), and the Python
  docstrings that
  [autodoc](https://www.sphinx-doc.org/en/master/usage/extensions/autodoc.html) imports.
  On [cross-referencing](https://www.sphinx-doc.org/en/master/usage/referencing.html): "To
  support cross-referencing to arbitrary locations in any document, the standard
  reStructuredText labels are used". A `:ref:` names a label on a section, figure, table
  or any other location; "it works across files, when section headings are changed", and
  "label names must be unique throughout the entire documentation".
  [Domains](https://www.sphinx-doc.org/en/master/usage/domains/index.html) add objects
  such as Python functions and classes; a Python object reference follows the dotted import
  path, so moving the object to another module breaks it.
  [autosectionlabel](https://www.sphinx-doc.org/en/master/usage/extensions/autosectionlabel.html)
  instead labels a section by its title, optionally prefixed by the document name, so a
  reworded title breaks the reference, and with the prefix so does a moved file. Prose and
  Python docstrings share one set of labels and objects; core Sphinx reads no other
  language's source. A `:ref:` to an undefined label already warns.
  [`nitpicky`](https://www.sphinx-doc.org/en/master/usage/configuration.html), "Default:
  False", makes Sphinx "warn about *all* references where the target cannot be found", and
  `nitpick_ignore` lists the exceptions.
  [`sphinx-build`](https://www.sphinx-doc.org/en/master/man/sphinx-build.html) turns
  nitpicky mode on with `-n`, and with `-W` it "exits with exit status 1 if any warnings
  are generated". The
  [coverage](https://www.sphinx-doc.org/en/master/usage/extensions/coverage.html) builder
  reports Python objects that lack documentation. Extensions written in Python can act on
  a missing reference through
  [events](https://www.sphinx-doc.org/en/master/extdev/event_callbacks.html) such as
  `warn-missing-reference`.
  [intersphinx](https://www.sphinx-doc.org/en/master/usage/extensions/intersphinx.html)
  downloads a remote project's `objects.inv` when it is mapped to a URL, and autodoc
  imports the modules it documents.

None of the three documents a rule a project declares about which kind of target may link
to which. A rustdoc disambiguator such as `struct@`, or a Sphinx role such as `:py:func:`,
picks the kind of one link's target.

Matthew Flatt, Eli Barzilay and Robert Bruce Findler, *Scribble: Closing the Book on Ad
Hoc Documentation Tools*, ICFP 2009, DOI 10.1145/1596550.1596569
([dblp](https://dblp.org/doi/10.1145/1596550.1596569),
[PDF](https://www2.ccs.neu.edu/racket/pubs/icfp09-fbf.pdf)), describe the documentation
system of PLT Scheme, built on its technology for language extension, with "a new
approach to connecting prose references with library bindings" at its heart. "Scribble
uses syntax objects to carry lexical information all the way through document rendering";
"a for-label import introduces bindings for documentation without triggering the
execution of the imported module"; and a reference links through "a cross-reference key
based on the identifier and the source module". For-label declarations "work with the
module system's re-exporting mechanisms". The sections read do not describe how an
unresolved reference is reported, and renaming is not addressed. The system is evaluated
by use: its authors "have used Scribble to produce thousands of pages of documentation for
PLT Scheme".

Will Crichton and Shriram Krishnamurthi, *A Core Calculus for Documents: Or, Lambda: the
Ultimate Document*, POPL 2024, Proceedings of the ACM on Programming Languages 8, article
23, DOI 10.1145/3632865 ([arXiv:2310.04368](https://arxiv.org/abs/2310.04368)), note that
"there are no formal foundations that model" languages mixing documents and programs, and
formalize them as levels of a document calculus. One extension gives sections string
identifiers that a `ref` element names, with TeX's intent: a reference displays the number
of the section it names. Validity is an auxiliary judgment beyond syntax, because "an
article is not valid if it references an unknown identifier, analogous to a free
variable". For a reactive document, Theorem 5.2 shows that an incremental strategy, which
recomputes the map from identifiers to section numbers only when a step may have changed
the numbering, renders the same references as recomputing it at every step. All of this
is within one document.

## agrees: Where grund agrees

- The shared check is that a reference naming nothing fails. rustdoc's lint, DocLint's
  `reference` group and Sphinx in nitpicky mode each report one, and the calculus's
  validity judgment rejects one. In grund a dangling citation or a missing section is an
  error ([§FS-check.3.1](../functional-spec/FS-check.md#31-dangling-citation), [§FS-check.3.2](../functional-spec/FS-check.md#32-missing-section)), and `grund check` exits 1 on it ([§FS-check.2](../functional-spec/FS-check.md#2-outputs)), as
  `deny`, `-Werror` and `-W` make the tools fail a build.
- A reference names a declared target, not the place it lives. Sphinx's explicit labels
  survive a moved file and a reworded heading, and a label name is unique across the
  documentation. A grund address holds no path and no title ([§GRUND-schema.2](../grund.md#2-keeping-it)), and an ID
  declared twice is an error ([§FS-declarations.checks.duplicate](../functional-spec/FS-declarations.md#checksduplicate-duplicate-declaration)).
- The calculus states formally what grund states in prose. A reference to an unknown
  identifier makes the document invalid, and a grund citation resolves only when exactly
  one declaration of its ID exists and its section path exists ([§GOAL-no-dangling-refs.1](../goals.md#1-what-resolves-means)).
  A named grund section plays the part of a string identifier: a name never moves
  ([§REQ-spec-section-names.permanence](../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent)).
- rustdoc's and DocLint's reference checks describe no network step, and grund's checks
  and reads perform no network I/O ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

## departs: Where grund departs

- rustdoc, javadoc and Scribble resolve a reference through the host language's own name
  resolution: Rust module scope, Java scope, PLT Scheme bindings brought in by for-label
  imports. Each reaches one language's items, and its addresses follow the program's
  structure. Sphinx's labels do not, but its Python object references follow the import
  path, and autodoc imports the modules. grund parses no code: a citation names a
  declared ID, never a class, function or scope ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)). The other side of that
  choice is one grammar in Markdown and in the comments of every host language
  ([§GOAL-polyglot-citation.1](../goals.md#1-what-cleanly-means)).
- rustdoc, javadoc, Sphinx and Scribble generate documentation, and DocLint checks the
  comments used in it. The calculus models documents that compute their content,
  including the section numbers its theorem is about. grund renders nothing
  ([§FS-non-goals.5](../functional-spec/FS-non-goals.md#5-documentation-generation)). A section number in grund is written on its heading and is itself an
  address ([§REQ-spec-section-names.shape](../requirements/REQ-spec-section-names.md#shape-where-a-number-is-allowed)), so there is no displayed number to recompute.
- rustdoc's `#fragment` and javadoc's `##fragment` point into generated pages, and
  neither tool's documentation states that the fragment is checked. grund validates no
  URL or Markdown anchor ([§FS-non-goals.1](../functional-spec/FS-non-goals.md#1-markdown-link-validation)); a link checker such as lychee does that, and
  its source is in [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund).
- intersphinx downloads a remote inventory, and javadoc's `-link` reads a list file at a
  URL. A grund check performs no network I/O ([§FS-non-goals.11](../functional-spec/FS-non-goals.md#11-network-access-during-a-check)).
- The calculus proves a theorem about its section references. grund's resolution is
  stated in prose requirements ([§REQ-no-missed-citation](../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked), [§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess)), with no
  calculus or theorem behind it, and it spans every file a run reads rather than one
  document.

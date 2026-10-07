# REL-incremental-formalization: incremental formalization, the idea grund descends from

Incremental formalization is the closest named idea to what grund does with a
repository's prose: people write informally, and structure is added later, one place at a
time, once it is clear and pays for itself. It is the lineage of the declarations that
organize a project's memory one fact at a time inside free prose ([§GRUND-schema.1](../grund.md#1-declaring-it)).

## work: What the work is

Frank M. Shipman III named the idea in his 1993 PhD dissertation at the University of
Colorado, *Supporting Knowledge-Base Evolution with Incremental Formalization*. Its first
conference paper, of the same title, is Shipman and Raymond J. McCall's at CHI '94, pages
285–291, DOI 10.1145/191666.191768
([abstract](https://people.engr.tamu.edu/shipman/chi94-hos/chi94hos_abstract.html)). It
describes an approach "when users express information informally and the system supports
them in formalizing it", and the Hyper-Object Substrate (HOS), a system built to "enable
and support moving information upward in formality", with "knowledge-based techniques to
suggest possible formalizations of this informal information". In HOS, informal text
later gained attributes and took part in inheritance relationships.

Shipman and Catherine C. Marshall's *Formality Considered Harmful: Experiences, Emerging
Themes, and Directions* explains why structure demanded up front fails: users are right
to resist "premature, unnecessary, meaningless, or cognitively expensive formalization".
The [linked text](https://people.engr.tamu.edu/shipman/formality-paper/harmful.html) is
their Xerox PARC report of about 1994. The journal version, titled *Formality Considered
Harmful: Experiences, Emerging Themes, and Directions on the Use of Formal Representations
in Interactive Systems*, appeared in *Computer Supported Cooperative Work* 8(4):333–352 in
1999, DOI 10.1023/A:1008716330212.

## agrees: Where grund agrees

- Prose is legal by default. The text between IDs is opaque content to grund
  ([§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)), and nothing requires a paragraph to carry an ID.
- Formality is added one spot at a time, by the author, where it pays: a heading becomes
  a declaration with a stable address ([§GRUND-schema.2](../grund.md#2-keeping-it)), and a `§` citation becomes an
  edge that is checked ([§GRUND-links.2](../grund.md#2-holding-every-edit-to-them)).
- Shipman and Marshall's cost is kept low at the start: a conformant repository needs no
  configuration at all ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)).

## departs: Where grund departs

- HOS formalizes toward a knowledge base of attributes and relationships. grund formalizes
  exactly one thing, its ID scheme, and deliberately does not generalize ([§GRUND-grund.2](../grund.md#2-who-it-is-for),
  [§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).
- HOS suggests formalizations of informal text. grund never does: it reads no meaning out
  of the prose between IDs ([§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)), so formalizing a spot is always the author's
  act.

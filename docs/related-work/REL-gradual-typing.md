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
  checking by tuning the (im)precision of pre- and postconditions". Of the three, it is
  the closest fit.
- Gilad Bracha, *Pluggable Type Systems*, a 2004 OOPSLA workshop position paper
  ([PDF](https://www.cs.tufts.edu/~nr/cs257/archive/gilad-bracha/pluggableTypesPosition.pdf)):
  an optional type system "has no effect on the run-time semantics of the programming
  language".

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

## departs: Where grund departs

- Gradual typing checks what crosses from the unannotated part at run time, by casts.
  grund has no run time: a `§` citation is checked statically, before anything runs, and
  must resolve ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).
- The gradual guarantee says a less precise program behaves the same as a more precise
  one. grund makes no such promise: a `must` citation direction ([§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels)) or the opt-in grounding
  floor ([§FS-check.3.6](../functional-spec/FS-check.md#36-ungrounded-unit-opt-in)) makes some formality mandatory, so deleting a citation can fail
  the check.
- The levels are not errors and lints. `must` and `must-not` are errors; `should` and
  `should-not` are suggestions that never appear in `grund check`'s standing output and
  surface only under `--suggestions`. The mapping is fixed ([§FS-config.3.9.1.3](../functional-spec/FS-config.md#3913-the-levelsurface-mapping-is-fixed),
  [§FS-non-goals.9](../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)).

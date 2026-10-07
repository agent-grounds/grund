# REL-graph-shapes: shapes and rule-based schemas, constraints over a structure

SHACL validates a graph against declared shapes, and Schematron validates a document by
rules rather than by a grammar. Together they are the precedent for checking a structure
by declared constraints of graded severity, which is what citation directions do to the
citation graph ([§GRUND-links.1](../grund.md#1-declaring-the-rules)).

## work: What the work is

SHACL, the Shapes Constraint Language, is a W3C Recommendation of 20 July 2017
([w3.org/TR/shacl](https://www.w3.org/TR/shacl/)). A shapes graph validates a data graph.
A shape selects the nodes it applies to by targets, `sh:targetClass` among them, and gives
the results it produces one severity through `sh:severity`. SHACL includes three,
`sh:Violation`, `sh:Warning` and `sh:Info`, but "Any IRI can be used as a severity";
"sh:Violation is the default if sh:severity is unspecified", and "the specific values of
sh:severity have no impact on the validation". A shape is open: a node may carry
properties the shape does not mention, unless the shape says `sh:closed true`.

Schematron, standardized as ISO/IEC 19757-3, *Rule-based validation using Schematron*
([schematron.com](https://schematron.com/)), is "a language for making assertions about
the presence or absence of patterns" in documents, rather than a grammar the whole
document must match.

## agrees: Where grund agrees

- Citation directions are constraints over a graph, selected by kind as SHACL selects by
  class: each `[citations.<KIND>]` table holds every declaration of that kind
  ([§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules)).
- Like Schematron, they are rules, not a grammar: a declaration is free in shape, and only
  the stated rules hold it ([§DF-citation-directions.2.1](../decisions/functional/DF-citation-directions.md#21-a-citations-section-keyed-by-citing-kind-with-rfc-2119-levels)).
- The levels are graded, as SHACL's severities are: a `must` gates, and a `should` only
  advises.

## departs: Where grund departs

- SHACL lets each shape choose its severity, and any result, even at `sh:Info`, makes the
  graph non-conforming. In grund the level decides the surface and the mapping is fixed:
  `must` and `must-not` are errors, while `should` and `should-not` are suggestions that
  never reach `grund check`'s standing output or its exit code and surface only under
  `--suggestions`. A `should` is not a warning ([§FS-config.3.9.1.3](../functional-spec/FS-config.md#3913-the-levelsurface-mapping-is-fixed), [§FS-non-goals.9](../functional-spec/FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)).
- Both validate whatever constraint a schema author can express. grund's rules stay
  inside its own ID scheme, directions between kinds and chapter rules over declarations
  ([§FS-rules](../functional-spec/FS-rules.md#fs-rules-grounded-declarations-state-and-enforce-chapter-rules)), and it validates no other structure ([§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).

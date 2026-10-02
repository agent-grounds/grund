# REL-information-typing: information typing, units of one kind under one label

Information typing sorts content into a few kinds of unit, each holding one kind of
information under its own label. It is the precedent for grund's kinds, and for the slice
grund returns: one fact, readable without the file around it ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)).

## work: What the work is

Robert E. Horn's Information Mapping is the origin. In *What Kinds of Writing Have a
Future?*, the speech he prepared for his ACM SIGDOC Lifetime Achievement Award in 2001
([PDF](https://faculty.washington.edu/farkas/TC510-Fall2011/Horn-DocsWithFuture.pdf)), he
describes the information block, a labelled substitute for the paragraph built on four
principles: chunking, relevancy ("Include in one chunk only information that relates to
one main point"), labeling and consistency. Blocks join into information maps. Key blocks
appear on seven kinds of map, and the seven kinds of information came to be called
information types: structure, concept, procedure, process, classification, principle and
fact. Horn's "seven plus or minus two" counts chunks, not sentences: a map joins two to
seven blocks for a reader's "short-term memory capacity of seven plus or minus two chunks
of information".

DITA carries the idea into structured authoring
([DITA 2.0 Architecture Specification](https://dita-lang.org/2.0/dita/archspec/base/information-typing)):
"Information typing is the practice of identifying types of topics, such as concept,
reference, and task". The specification credits "Robert Horn's Information Mapping and
Hughes Aircraft's STOP", and notes that "many DITA topic types are not necessarily
closely connected with traditional Information Mapping".

ASD-STE100, Simplified Technical English ([asd-ste100.org](https://www.asd-ste100.org/)),
applies a length cap at the scale of one sentence. Secondary sources give Issue 9's rule
as 20 words for a procedural sentence and 25 for a descriptive one; the standard itself is
released on request, so this entry does not quote it.

## agrees: Where grund agrees

- A kind is an information type. Every declaration is one unit of one kind under its own
  label, its ID and title, and the kind tells a reader what the unit is for
  ([§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds)).
- Horn's relevancy principle is the read grund serves: one point per unit, returned on its
  own rather than with the file around it ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), [§GRUND-structure](../grund.md#grund-structure-the-projects-long-term-memory-stays-organized)).

## departs: Where grund departs

- grund fixes no set of types. Horn names seven; a grund project declares its own kinds
  ([§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds)).
- grund lints no English ([§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)). It caps no sentence, as ASD-STE100 does, and
  counts no blocks in a map, as Horn does. Its only length caps are on inline citation
  notes ([§FS-inline-citation-style.4.1](../functional-spec/FS-inline-citation-style.md#41-errors--hard-caps)); the size of a file is
  [fissile](https://github.com/agent-grounds/fissile)'s concern, not grund's.

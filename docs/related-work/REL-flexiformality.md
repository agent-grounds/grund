# REL-flexiformality: flexiformality, documents that are partly formal

Flexiformality names documents that carry informal parts for human readers and formal
parts a machine can act on, side by side. A grounded repository is such a document: prose,
with declarations and `§` citations in it that a checker reads ([§GRUND-structure](../grund.md#grund-structure-the-projects-long-term-memory-stays-organized)).

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
Kohlhase, Michael Kohlhase and Christoph Lange, MKM 2010
([arXiv:1004.5071](https://arxiv.org/abs/1004.5071),
[record](https://cris.fau.de/publications/106429884/)), applies the question to the
documents of a software engineering project. It does not use the word "flexiformal".

sTeX, Kohlhase's semantic LaTeX ([CTAN](https://ctan.org/pkg/stex)), is the closest system
in mechanism: a symbol is declared, a reference names it, and a reference to a symbol that
is not found is an error.

Lars Vogt's *The Semantic Ladder: A Framework for Progressive Formalization of Natural
Language Content for Knowledge Graphs and AI Systems* (2026,
[arXiv:2603.22136](https://arxiv.org/abs/2603.22136)) works under the name "progressive
formalization". Reading it as a recent version of the same idea is grund's reading, not
Vogt's claim: the paper cites neither Kohlhase nor Shipman.

## agrees: Where grund agrees

- A document need not be formal throughout to be useful to a machine. grund checks the
  formal parts, the declarations and the citations, and leaves the rest to human readers
  ([§GRUND-consistency](../grund.md#grund-consistency-the-structure-stays-consistent), [§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)).
- As in sTeX, a reference is to something declared, and one that names nothing is an
  error ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).

## departs: Where grund departs

- Flexiformality is a program for whatever formal content a document may carry,
  mathematical knowledge first. grund formalizes exactly one thing, its ID scheme, and
  deliberately does not generalize ([§GRUND-grund.2](../grund.md#2-who-it-is-for), [§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).

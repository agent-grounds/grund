# Related work

Where grund comes from, and what it deliberately is not, is part of its why
([§GRUND-understanding](../grund.md#grund-understanding-the-why-stays-known)). This
folder gives each body of related work, an idea grund descends from or a tool that sits
beside it, one declaration, so that a goal or a decision can say "this is incremental
formalization, and here is where we depart from it" in one `§` instead of retelling it.

An entry is grund's own account of someone else's work, not a copy of it. Its lead says in
a sentence or two what the work is to grund and cites the GRUND or GOAL point it bears on,
which is the direction `grund.toml` sets for the kind. Its body is exactly three chapters,
in this order, and nothing else at that level:

- `## work: What the work is`: the work itself, under its authors' own names and titles,
  linked to a source every claim about it can be checked against, a DOI where there is one;
- `## agrees: Where grund agrees`: what grund takes from the work, or shares with it;
- `## departs: Where grund departs`: what grund does not do that the work does, each
  departure citing the point that rules it out, most often a non-goal, rather than
  restating it.

A reader who holds an entry can therefore guess its addresses, `<§>REL-<slug>.departs`
for one, and they are permanent once anything cites them
([§REQ-spec-section-names.permanence](../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent)).
A part of one chapter, such as a comparison table, is a named section beneath it, never a
fourth chapter and never a numbered one
([§REQ-spec-section-names.shape](../requirements/REQ-spec-section-names.md#shape-where-a-number-is-allowed)).

Read together, the entries place grund in that lineage: grund is incremental
formalization for software documentation, and the documents it checks are flexiformal.
Prose is legal by default, and a declaration or a `§` citation formalizes one spot at a
time. "Gradual typing for prose" says the same in a programmer's terms, although
human-computer interaction named the idea more than a decade before gradual typing. This
is grund's reading of where it comes from, not a claim of scope. Where those ideas are
about formalization in general, grund formalizes exactly one thing, its ID scheme, and
deliberately does not generalize ([§GRUND-grund.2](../grund.md#2-who-it-is-for),
[§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).

Every entry is cited from outside this folder by a point it bears on. Its line in this
index does not count, because an index names every declaration in its folder by
construction
([§FS-check.4.1.2](../functional-spec/FS-check.md#412-an-index-entry-does-not-count)).
A roadmap point does not count on its own either, because the roadmap deletes a point
once it ships.

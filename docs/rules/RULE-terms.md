# RULE-terms: Each FS must have exactly one Terms chapter.

A slice read on its own only answers on its own if its words mean one thing
([§FS-terms](../functional-spec/FS-terms.md#fs-terms-the-shared-vocabulary-of-the-functional-spec-and-the-architecture)),
and the chapter is where a document says which shared senses it leans on. A spec
that carries no chapter is not a spec that uses no shared word — it is one whose
reader has to guess, which is the drift the vocabulary exists to close
([§GOAL-token-economy.1](../goals.md#1-what-this-requires)).

The sentence holds presence and cardinality and nothing else. What the chapter
says — that it opens with a lean line, that its words resolve, that no word is
defined twice — is held beside it by
`tests/integration/test_functional_spec_terms_content.py`, because a rule
sentence can require a chapter and cannot read one.

`Each FS` is the whole kind, `FS-terms` included: the vocabulary carries the
chapter that holds the shared groups, so the subject needs no exception and the
grammar offers none ([§FS-rules.2](../functional-spec/FS-rules.md#2-subject-selectors)).

Where the chapter sits and what it is titled belong to the `FS.terms` field
this repository's `grund.toml` declares
([§FS-fields](../functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it)),
not to this sentence: the field finds the section *named* `terms` and holds it
first, titled Terms, while the rule counts the chapters *titled* Terms. The two
are kept side by side because each sees a defect the other cannot — the field a
Terms chapter out of place, the rule two chapters titled Terms under different
names. An FS with no Terms chapter is one defect, and it is reported once: the
field's finding stands and the rule's is dropped
([§DF-kind-fields.2.5](../decisions/functional/DF-kind-fields.md#25-a-field-and-a-rule-are-paired-at-the-merge-not-deduplicated-before-evaluation),
[§FS-rules.6](../functional-spec/FS-rules.md#6-semantic-deduplication)).

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

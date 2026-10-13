# REL-controlled-language: controlled natural languages, English with a fixed grammar

A controlled natural language restricts English to a fixed grammar, so that a sentence a
person reads as English a machine reads one way. grund's chapter rules are such sentences:
a rule declaration states a convention about the shape of a project's knowledge as one
sentence of a closed controlled-English grammar, and the check applies it ([§GRUND-schema.1](../grund.md#1-declaring-it)).

## work: What the work is

Norbert E. Fuchs, Kaarel Kaljurand and Tobias Kuhn, *Attempto Controlled English for
Knowledge Representation*, Reasoning Web 2008, LNCS, pages 104–124, DOI
10.1007/978-3-540-85658-0_3 ([Springer](https://link.springer.com/chapter/10.1007/978-3-540-85658-0_3)),
describe Attempto Controlled English (ACE), which "can automatically and unambiguously be
translated into first-order logic" and "is supported by a number of tools". The Attempto
project's [description](https://attempto.ifi.uzh.ch/site/description/description.html)
calls ACE "a rich subset of standard English designed to serve as knowledge representation
language": ACE texts "can be unambiguously translated into discourse representation
structures", and "a small set of ACE construction and interpretation rules" fixes that
translation. The unit translated is a whole text, and no theorem about the translation is
stated.

Tobias Kuhn, *A Survey and Classification of Controlled Natural Languages*, Computational
Linguistics 40(1):121–170, 2014, DOI 10.1162/COLI_a_00168
([ACL Anthology](https://aclanthology.org/J14-1005/)), lists and describes "100 languages
from 1930 until today", all English-based, and gives a general classification scheme for
them. Such languages "are applied to improve communication among humans, to improve
translation, or to provide natural and intuitive representations for formal notations",
and together they form "a single scattered cloud filling the conceptual space between
natural languages such as English on the one end and formal languages such as
propositional logic on the other".

Silvie Spreeuwenberg and Keri Anderson Healy, *SBVR's Approach to Controlled Natural
Language*, CNL 2009 (LNCS 5972, 2010), pages 155–169, DOI 10.1007/978-3-642-14418-9_10
([Springer](https://link.springer.com/chapter/10.1007/978-3-642-14418-9_10)), explain how SBVR captures a
rule's semantic formulation rather than standardizing one surface language; SBVR Structured
English is one notation for writing those formulations, and the abstract describes no
checker. The OMG standard, [SBVR 1.5](https://www.omg.org/spec/SBVR/) of December 2019,
"defines the vocabulary and rules for documenting the semantics of business vocabularies
and business rules".

Marcos Cramer, Bernhard Fisseni, Peter Koepke, Daniel Kühlwein, Bernhard Schröder and Jip
Veldman, *The Naproche Project: Controlled Natural Language Proof Checking of Mathematical
Texts*, CNL 2009 (LNCS 5972, 2010), pages 170–186, DOI 10.1007/978-3-642-14418-9_11
([Springer](https://link.springer.com/chapter/10.1007/978-3-642-14418-9_11)), give a controlled language for
mathematical texts whose semantics is Proof Representation Structures, an adaptation of
discourse representation structures. A prototype parses texts in that language and checks
the proofs in them for logical correctness; as in ACE, the unit is a whole text.

Two tools carry controlled English into everyday programming (tool documentation, not
peer-reviewed, unversioned, checked 2026-10-13).
[Inform 7](https://ganelson.github.io/inform-website/) "is a natural-language-based
programming language for writers of interactive fiction". Gherkin, Cucumber's language,
"uses a set of special keywords to give structure and meaning to executable
specifications" ([reference](https://cucumber.io/docs/gherkin/reference/)): `Feature`,
`Rule`, `Scenario`, `Given`, `When`, `Then` and a few more. "The trailing portion (after
the keyword) of each step is matched to a code block, called a step definition", whose
expression "can either be a Regular Expression or a Cucumber Expression"
([step definitions](https://cucumber.io/docs/cucumber/step-definitions/)).

## agrees: Where grund agrees

- A rule is one English sentence from a closed grammar: fixed words that are
  case-sensitive, exactly one semantic verb, one terminal period, and five sentence
  families ([§FS-rules.3](../functional-spec/FS-rules.md#3-the-five-sentence-families)), with one canonical spelling for each meaning
  ([§DF-chapter-rules.2](../decisions/functional/DF-chapter-rules.md#2-decision)). In the terms of Kuhn's survey it serves the third purpose, a
  natural representation of a formal notation: Datalog states the meaning precisely but is
  not the friendly instruction surface grund requires ([§DF-chapter-rules.3](../decisions/functional/DF-chapter-rules.md#3-alternatives-rejected),
  [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)).
- As an ACE text translates into logic one way, a grund rule has one meaning, stated in
  logic: the relational clauses of [§FS-rules.5.2](../functional-spec/FS-rules.md#52-family-clauses). Where a sentence could be read two ways,
  grund does not pick a reading; it refuses the sentence and names every candidate
  ([§FS-rule-refusals.sentences](../functional-spec/FS-rule-refusals.md#sentences-strict-refusals)).
- As SBVR separates a rule's semantic formulation from the words that express it, grund's
  normative meaning is the relational clauses, not the wording, and SBVR is named as
  informative heritage only ([§DF-chapter-rules.2](../decisions/functional/DF-chapter-rules.md#2-decision)).

## departs: Where grund departs

- ACE is designed to serve as a knowledge representation language, and SBVR documents a
  business's vocabulary and rules. grund's grammar is five productions about chapters and
  citations. It has no general SBVR parser, imports no external standard, and accepts no
  synonyms ([§DF-chapter-rules.3](../decisions/functional/DF-chapter-rules.md#3-alternatives-rejected)); a paraphrase is refused, and the refusal gives the
  accepted form wherever one can be recovered ([§FS-rule-refusals.sentences](../functional-spec/FS-rule-refusals.md#sentences-strict-refusals)).
- grund offers no controlled language for arbitrary knowledge. What it formalizes is its
  own ID scheme, and it deliberately does not generalize beyond it ([§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).
- ACE translates a whole text and Naproche checks the proofs of a whole text; Gherkin
  matches the text of each step to code. grund reads controlled English only in rule
  sentences, such as a rule declaration's title, which is a grammar island ([§FS-rules.1](../functional-spec/FS-rules.md#1-rule-declarations-and-opt-in)).
  The rationale beneath it and every other paragraph are opaque content grund never
  interprets ([§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)), and text reaches its target through a cited ID that must
  resolve to a declaration, not by matching its phrases ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).

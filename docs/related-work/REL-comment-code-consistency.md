# REL-comment-code-consistency: comment-code consistency, claims inferred from comments

Comment-code consistency research asks whether a comment still agrees with the code it
documents, and answers by inferring a claim from the comment's prose and testing the code
against it. It works the edge grund holds, between code and the text that says why, by
inference where grund checks only what an author declared ([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)).

## work: What the work is

Rules inferred from comments and checked against the code:

- Lin Tan, Ding Yuan, Gopal Krishna and Yuanyuan Zhou, */\* iComment: Bugs or Bad
  Comments? \*/*, SOSP 2007, DOI 10.1145/1294261.1294276
  ([PDF](https://www.cs.purdue.edu/homes/lintan/publications/icomment_sosp07.pdf)),
  combine natural language processing, machine learning, statistics and program analysis
  to extract implicit program rules from comments such as "Caller must hold instance
  lock", and check the code against them. On Linux, Mozilla, Wine and Apache, iComment
  extracted 1,832 rules with 90.8–100% accuracy and found 60 inconsistencies, 33 new bugs
  and 27 bad comments; developers had confirmed 19 of them.
- Shin Hwei Tan, Darko Marinov, Lin Tan and Gary T. Leavens, *@tComment: Testing Javadoc
  Comments to Detect Comment-Code Inconsistencies*, ICST 2012, DOI 10.1109/ICST.2012.106
  ([DOI](https://doi.org/10.1109/ICST.2012.106)), infer likely properties about null values
  and related exceptions from the English text of Javadoc comments, then generate random
  tests that check them. On seven open-source projects @tComment found 29 inconsistencies;
  of the 16 reported, 5 were confirmed and fixed.
- Yu Zhou, Ruihang Gu, Taolue Chen, Zhiqiu Huang, Sebastiano Panichella and Harald C. Gall,
  *Analyzing APIs Documentation and Code to Detect Directive Defects* (DRONE), ICSE 2017,
  DOI 10.1109/ICSE.2017.11 ([DOI](https://doi.org/10.1109/ICSE.2017.11)), analyse the
  directives of API documents about parameter constraints and exceptions with program
  comprehension and natural language processing, and detect defects with a first-order
  logic constraint solver. On parts of the JDK 1.8 APIs they report 1,158 defective
  directives among about 2,000 usage constraints, at 81.6% precision and 82.0% recall.
- Arianna Blasi, Alberto Goffi, Konstantin Kuznetsov, Alessandra Gorla, Michael D. Ernst,
  Mauro Pezzè and Sergio Delgado Castellanos, *Translating Code Comments to Procedure
  Specifications* (Jdoctor), ISSTA 2018, DOI 10.1145/3213846.3213872
  ([PDF](https://homes.cs.washington.edu/~mernst/pubs/comments-specs-issta2018.pdf)),
  translate semi-structured Javadoc into executable procedure specifications written as
  Java expressions, by pattern, lexical and semantic matching, with 92% precision and 83%
  recall. The specifications serve the test generator Randoop as oracles.
- Juan Zhai et al., *CPC: Automatically Classifying and Propagating Natural Language
  Comments via Program Analysis*, ICSE 2020, DOI 10.1145/3377811.3380427
  ([dblp](https://dblp.org/doi/10.1145/3377811.3380427)), build a taxonomy of comments and
  a classifier for it, and propagate comments to other code locations by program analysis.
  On five projects CPC derived 41,573 new comments with 88% accuracy, detected 37 new bugs,
  30 of them confirmed and fixed, and found 304 defects in existing comments.
- Yichi Zhang and Yang Feng, *RustC4++: Improving Rust Code-Comment Inconsistency Detection
  via Hybrid LLM and Static Analysis*, ACM TOSEM 2026, DOI 10.1145/3800689
  ([dblp](https://dblp.org/doi/10.1145/3800689)), have a large language model extract
  interval, boundary, existence, option and finiteness constraints from Rust comments,
  propagate them across trait and implementation boundaries, and check the code against
  them with static analysis and SMT-based bidirectional verification. On 17 Rust projects
  it outperforms prior tools in precision and recall, and some of its findings led to
  confirmed fixes upstream. Its precursor is Yichi Zhang, Zixi Liu, Yang Feng and Baowen
  Xu, *Leveraging Large Language Model to Assist Detecting Rust Code Comment
  Inconsistency*, ASE 2024, DOI 10.1145/3691620.3695010
  ([dblp](https://dblp.org/doi/10.1145/3691620.3695010)).

Specifications drawn from natural-language intent:

- Madeline Endres, Sarah Fakhoury, Saikat Chakraborty and Shuvendu K. Lahiri, *Can Large
  Language Models Transform Natural Language Intent into Formal Method Postconditions?*,
  Proceedings of the ACM on Software Engineering (FSE 2024), DOI 10.1145/3660791
  ([arXiv:2310.01831](https://arxiv.org/abs/2310.01831)), have large language models turn
  code comments and function documentation into postconditions written as program
  assertions. The generated postconditions caught 64 real historical bugs from Defects4J.
- Dongjae Lee and Kihong Heo, *Expecto: Extracting Formal Specifications from Natural
  Language Description for Trustworthy Oracles*, Proceedings of the ACM on Programming
  Languages 10, 2026, DOI 10.1145/3808332 ([dblp](https://dblp.org/doi/10.1145/3808332)),
  combine a large language model with program synthesis to extract formal specifications,
  written in a domain-specific language, from natural-language intents. Applied to buggy
  programs in Defects4J, its specifications detect more bugs than the baselines.

Inconsistency caught at the change, and comments rewritten to match:

- Sheena Panthaplackel, Junyi Jessy Li, Milos Gligoric and Raymond J. Mooney, *Deep
  Just-In-Time Inconsistency Detection Between Comments and Source Code*, AAAI 2021, DOI
  10.1609/aaai.v35i1.16119 ([DOI](https://doi.org/10.1609/aaai.v35i1.16119)), train a
  deep-learning model to correlate a comment with changes to the code it documents, so
  that an inconsistency is caught before it is committed. The model handles the `@return`,
  `@param` and summary comments of Java methods, outperforms several baselines on a large
  corpus of comment/code pairs, and is combined with a comment update model to detect and
  resolve inconsistent comments. The same group's *Learning to Update Natural Language
  Comments Based on Code Changes* (Sheena Panthaplackel, Pengyu Nie, Milos Gligoric, Junyi
  Jessy Li and Raymond J. Mooney), ACL 2020, DOI 10.18653/v1/2020.acl-main.168
  ([ACL Anthology](https://aclanthology.org/2020.acl-main.168/)), learns to generate a
  sequence of edits to an existing comment from changes to the method it accompanies.
- Zhongxin Liu, Xin Xia, Meng Yan and Shanping Li, *Automating Just-In-Time Comment
  Updating*, ASE 2020, DOI 10.1145/3324884.3416581
  ([dblp](https://dblp.org/doi/10.1145/3324884.3416581)), present CUP, a neural
  sequence-to-sequence model that generates a new comment from the old comment and the
  code change. On a dataset of over 108,000 comment-code co-changes, 16.7% of its
  generated comments were identical to the developers' updates.
- Guoping Rong, Yongda Yu, Song Liu, Xin Tan, Tianyi Zhang, Haifeng Shen and Jidong Hu,
  *Code Comment Inconsistency Detection and Rectification Using a Large Language Model*
  (C4RLLaMA), ICSE 2025, DOI 10.1109/ICSE55347.2025.00035
  ([DOI](https://doi.org/10.1109/ICSE55347.2025.00035)), fine-tune CodeLlama to detect
  inconsistent comments and rectify them. C4RLLaMA surpasses both post hoc detectors,
  which judge a comment against the current code, and just-in-time ones, which judge it
  against a change. In a manual evaluation, 65.0% (just-in-time) and 55.9% (post hoc) of
  its comment updates were correct.

How comments and code evolve together:

- Beat Fluri, Michael Würsch and Harald C. Gall, *Do Code and Comments Co-Evolve? On the
  Relation between Source Code and Comment Changes*, WCRE 2007, DOI 10.1109/WCRE.2007.21
  ([DOI](https://doi.org/10.1109/WCRE.2007.21)), map code to comments across the versions
  of ArgoUML, Azureus and JDT Core. Newly added code barely gets commented, and 97% of
  comment changes are made in the same revision as the associated code change.
- Fengcai Wen, Csaba Nagy, Gabriele Bavota and Michele Lanza, *A Large-Scale Empirical
  Study on Code-Comment Inconsistencies*, ICPC 2019, DOI 10.1109/ICPC.2019.00019
  ([PDF](https://www.inf.usi.ch/faculty/lanza/PUBS/P/Wen2019a.pdf)), mined 1.3 billion
  AST-level changes from the history of 1,500 systems and manually analysed 500 commits,
  362 of which changed comments. From these they define a taxonomy of comment-related
  changes: 69 types, 25 of them relevant to fixing code-comment inconsistencies, under six
  root categories, Application Logic, Code Design/Quality, Maintenance,
  Formatting/Readability, Copyright/License and Others. The largest, Application Logic
  (136 commits), holds comments about what the code does, such as the steps of an
  algorithm, its parameters or its return value, that were wrong, had been outdated by a
  new implementation, or were made more detailed. Smaller types cover comments updated
  after a refactoring renamed a method or variable (11 commits) and references in comments
  to other code elements that had become obsolete.
- Inderjot Kaur Ratol and Martin P. Robillard, *Detecting fragile comments* (Fraco), ASE
  2017, DOI 10.1109/ASE.2017.8115624
  ([DOI](https://doi.org/10.1109/ASE.2017.8115624)), start from textual
  references to an identifier in comment text, which "cannot be formally detected through
  the syntax of the language, and are thus fragile with respect to identifier renaming".
  Their rule-based Fraco detects such comments from the identifier's type, morphology and
  scope and the comment's location. Measured for precision and recall against
  hand-annotated benchmarks for six Java systems, it was near-optimal on most components
  and generally outperformed Eclipse's in-comment identifier replacement. More than half of
  the identifiers in the data set had fragile comments after renaming.

A position, a preprint, and two corpora:

- Michael D. Ernst, *Natural Language is a Programming Language: Applying Natural Language
  Processing to Software Development*, SNAPL 2017, DOI 10.4230/LIPIcs.SNAPL.2017.4
  ([Dagstuhl](https://drops.dagstuhl.de/entities/document/10.4230/LIPIcs.SNAPL.2017.4)),
  argues that analysis tools should use all of a program's artifacts, documentation and
  variable names among them, not source code alone, and gives four results that apply
  natural language processing to software artifacts to find bugs and generate code.
- Sungmin Kang, Louis Milliken and Shin Yoo, *Identifying Inaccurate Descriptions in
  LLM-generated Code Comments via Test Execution*, 2024
  ([arXiv:2406.14836](https://arxiv.org/abs/2406.14836)), not peer-reviewed, report that
  roughly a fifth of the comments written by the best of three large language models
  contained demonstrably inaccurate statements, that existing code-comment consistency
  detectors showed no statistically significant relationship with comment accuracy, and
  that "document testing", running tests a model generates from the comment, did.
- Hyeonseok Lee, Gabin An and Shin Yoo, *METAMON: Finding Inconsistencies between Program
  Documentation and Behavior using Metamorphic LLM Queries*, LLM4Code 2025 workshop
  ([arXiv:2502.02794](https://arxiv.org/abs/2502.02794)), is evaluated on 9,482 pairs of
  documentation and code from five Defects4J projects, where it classifies inconsistencies
  at 0.72 precision and 0.48 recall. Kunal Pai, Premkumar Devanbu and Toufique Ahmed,
  *CoDocBench: A Dataset for Code-Documentation Alignment in Software Maintenance*, MSR 2025
  data track ([arXiv:2502.00519](https://arxiv.org/abs/2502.00519)), is a dataset whose
  samples are commits that changed a function's code and its docstring together.

Across the inferred-rule and just-in-time works, the link runs from a comment to the code
it documents and is inferred, by rules, natural language processing, machine learning or a
large language model, rather than written by an author. What is checked is meaning: by
static analysis, generated tests or SMT for the inferred rules, and by a learned model's
probabilistic verdict for the just-in-time detectors. The unit is the commented method or
procedure, which RustC4++ follows across traits and implementations; iComment's rules take
parameters such as a lock and a function. Their sources do not address whether that link
survives a moved file or a renamed method, which is Fraco's subject. All are
evaluated empirically, by bugs and inconsistencies that developers confirmed and by
precision and recall.

## agrees: Where grund agrees

- Comments carry intent that drifts from the code, as iComment's bad comments and the
  inconsistencies developers fix in Wen et al.'s taxonomy show. grund starts from the
  same place: the why lives beside the code, and the edge between them rots unless every
  edit is held to it ([§GRUND-links.2](../grund.md#2-holding-every-edit-to-them)).
- A comment can carry a claim a tool checks, as @tComment's null properties and Jdoctor's
  procedure specifications do. grund's value bindings are such a claim: an authored
  literal written beside a citation must equal the declared value component it cites,
  exactly ([§FS-values.3](../functional-spec/FS-values.md#3-explicit-value-bindings), [§FS-values.4](../functional-spec/FS-values.md#4-exact-equality)).
- Panthaplackel et al. catch an inconsistency before it is committed, and Fluri, Würsch
  and Gall found nearly all comment changes in the same revision as the code change. grund
  checks at the same moment, on every save, commit and CI run ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).
- Fraco names the trouble with a reference written in free text: no syntax marks it, so
  nothing finds it when its target changes. A grund citation carries a marker, so it is
  recognized without guessing ([§DF-reference-marker.2.1](../decisions/functional/DF-reference-marker.md#21-marker)); one that resolves to nothing is
  an error ([§FS-check.3.1](../functional-spec/FS-check.md#31-dangling-citation)); and its address survives a moved file and a reworded heading
  ([§GRUND-schema.2](../grund.md#2-keeping-it)).

## departs: Where grund departs

- These works read meaning out of a comment's prose: a rule, a property, a specification,
  or a verdict of consistency. grund reads the text between IDs as opaque, compares only
  the explicit binding form, and never infers a value from the surrounding prose
  ([§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality), [§DA-explicit-value-bindings](../decisions/architectural/DA-explicit-value-bindings.md#da-explicit-value-bindings-compare-only-authored-delimited-value-bindings)). So it does not notice a comment that no
  longer says what the code does, the subject of Wen et al.'s largest root category; it
  notices a citation that no longer resolves. Where Ernst asks analysis tools to read
  documentation, grund reads only what an author marked.
- They test the claim against the code, by static analysis, constraint solving, SMT, or
  tests generated and run. grund does not parse code and knows no classes, methods, types
  or call graphs ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)), so a bound literal is compared with a declared value,
  never with what the program computes.
- Many of these works rest on a learned model: Panthaplackel et al.'s detector, CUP,
  C4RLLaMA, RustC4++'s constraint extraction, and Endres et al.'s and Expecto's
  specifications. grund's
  verdict may never differ between two installs of the same version with the same
  configuration ([§FS-non-goals.13](../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)), and its output is byte-identical for the same input
  ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)).
- Several rewrite the comment: CUP generates the new comment, Panthaplackel et al.'s
  update model edits it, and C4RLLaMA rectifies it, while Fraco's setting is the rename
  refactoring that should carry an identifier's new name into the comments that mention
  it. grund proposes no comment wording and offers no rename: an ID is permanent, and
  renaming one is a deliberate edit, not an automated operation ([§FS-non-goals.4](../functional-spec/FS-non-goals.md#4-cross-workspace-id-renaming)).

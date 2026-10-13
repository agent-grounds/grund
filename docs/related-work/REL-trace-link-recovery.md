# REL-trace-link-recovery: trace-link recovery, links proposed after the fact

Trace-link recovery proposes links between documentation and code after both are written,
by ranking or classifying candidate pairs, and scores its proposals against links that
people wrote. It answers grund's problem from the other side: grund rules out guessed
citations and checks the ones an author writes ([§GOAL-agent-grounding.3](../goals.md#3-what-this-rules-out)).

## work: What the work is

**Information retrieval.** G. Antoniol, G. Canfora, G. Casazza, A. De Lucia and E. Merlo,
*Recovering Traceability Links between Code and Documentation*, IEEE Transactions on
Software Engineering 28(10):970–983, 2002, DOI 10.1109/TSE.2002.1041053
([IEEE](https://doi.org/10.1109/TSE.2002.1041053)), propose "a method based on
information retrieval to recover traceability links between source code and free text
documents", on the premise "that programmers use meaningful names for program items".
Two case studies, with a probabilistic and a vector-space model, "trace C++ source code
onto manual pages and Java code to functional requirements". The same authors' *Recovering
Traceability Links Between Code and Documentation: A Retrospective*, IEEE Transactions on
Software Engineering 51(3):825–832, 2025, DOI 10.1109/TSE.2025.3534027
([IEEE](https://doi.org/10.1109/TSE.2025.3534027)), says the 2002 article "paved the way
to the adoption of IR in software engineering". Andrian Marcus and Jonathan I. Maletic,
*Recovering Documentation-to-Source-Code Traceability Links Using Latent Semantic
Indexing*, ICSE 2003, pages 125–135, DOI 10.1109/ICSE.2003.1201194
([IEEE](https://doi.org/10.1109/ICSE.2003.1201194)), apply latent semantic indexing to the
same task, on the LEDA library and Albergate. Markus Borg, Per Runeson and Anders Ardö,
*Recovering from a Decade: A Systematic Mapping of Information Retrieval Approaches to
Software Traceability*, Empirical Software Engineering 19(6):1565–1616, 2014, DOI
10.1007/s10664-013-9255-y ([open copy](https://lup.lub.lu.se/record/3737413)), classify
79 publications and find that "most previous studies do not go beyond reporting precision
and recall of candidate trace links from evaluations using datasets containing less than
500 artifacts".

**Learned models.** Jin Guo, Jinghui Cheng and Jane Cleland-Huang, *Semantically Enhanced
Software Traceability Using Deep Learning Techniques*, ICSE 2017, pages 3–14, DOI
10.1109/ICSE.2017.9 ([arXiv](https://arxiv.org/abs/1804.02438)), train a tracing network
of word embeddings and recurrent neural networks "using existing trace links in the
Positive Train Control domain"; of 360 configurations, a bidirectional GRU did best and
outperformed the vector space model and latent semantic indexing. Jinfeng Lin, Yalin Liu,
Qingkai Zeng, Meng Jiang and Jane Cleland-Huang, *Traceability Transformed: Generating
More Accurate Links with Pre-Trained BERT Models* (T-BERT), ICSE 2021, pages 324–335, DOI
10.1109/ICSE43902.2021.00040 ([arXiv](https://arxiv.org/abs/2102.04411)), transfer
knowledge from a related task with a richer dataset to "recover links between issues and
commits". On three open-source projects the best model "outperformed the VSM model with
average improvements of 60.31% measured using Mean Average Precision (MAP)".

**Documentation to code.** Jan Keim, Sophie Corallo, Dominik Fuchß, Tobias Hey, Tobias
Telge and Anne Koziolek, *Recovering Trace Links Between Software Documentation And Code*
(TransArC), ICSE 2024, DOI 10.1145/3597503.3639130
([open copy](https://publikationen.bibliothek.kit.edu/1000165692)), go from architecture
documentation to code through architecture models, chaining ArDoCo (documentation to
model) with ArCoTL (model to code). Their gold standard maps "the sentences of the SADs"
(software architecture documentation) to "the corresponding source code files". Over five
open-source projects the model-to-code step reaches an average F1 of 0.98, and the
end-to-end documentation-to-code result an average F1 of 0.82. Barthélémy Dagenais and
Martin P. Robillard, *Recovering Traceability Links between an API and Its Learning
Resources* (RecoDoc), ICSE 2012, pages 47–57, DOI 10.1109/ICSE.2012.6227207
([IEEE](https://doi.org/10.1109/ICSE.2012.6227207)), start from learning resources that
"are not explicitly linked to the frameworks' API". They identify "code-like terms in
documents" and link them to API elements such as methods, with an average precision and
recall of 96% on four open-source systems.

**Large language models.** Alberto D. Rodriguez, Katherine R. Dearstyne and Jane
Cleland-Huang, *Prompts Matter: Insights and Strategies for Prompt Engineering in
Automated Software Traceability*, REW 2023, pages 455–464, DOI 10.1109/REW57809.2023.00087
([arXiv](https://arxiv.org/abs/2308.00229)), study how to "extract link predictions from
an LLM" and report that refining the prompt improved "the ranking of candidate links".
Dominik Fuchß, Tobias Hey, Jan Keim, Haoyu Liu, Niklas Ewald, Tobias Thirolf and Anne
Koziolek, *LiSSA: Toward Generic Traceability Link Recovery Through Retrieval-Augmented
Generation*, ICSE 2025, pages 1396–1408, DOI 10.1109/ICSE55347.2025.00186
([open copy](https://publikationen.bibliothek.kit.edu/1000179816)), give an LLM retrieved
context and ask it whether a link exists, for requirements to code, documentation to
code, and architecture documentation to models, at a configurable granularity such as
requirement file to code file. LiSSA "can significantly outperform the state-of-the-art
on the code-related tasks", and its authors add that "further research is required" for
such approaches "to be applicable in practice". A companion by Tobias Hey, Dominik
Fuchß, Jan Keim and Anne Koziolek, *Requirements Traceability Link Recovery via
Retrieval-Augmented Generation*, REFSQ 2025, DOI 10.1007/978-3-031-88531-0_27
([open copy](https://publikationen.bibliothek.kit.edu/1000178589)), carries the approach
to requirements.

**Gold links, datasets and measures.** Jane Huffman Hayes, Alex Dekhtyar and Senthil
Karthikeyan Sundaram, *Advancing Candidate Link Generation for Requirements Tracing: The
Study of Methods*, IEEE Transactions on Software Engineering 32(1):4–19, 2006, DOI
10.1109/TSE.2006.3 ([IEEE](https://doi.org/10.1109/TSE.2006.3)), derive a tracing tool's
goals from the analyst's responsibilities, introduce measures for them, and build analyst
feedback into the prototype RETRO. Yonghee Shin, Jane Huffman Hayes and Jane
Cleland-Huang, *Guidelines for Benchmarking Automated Software Traceability Techniques*,
SST 2015, pages 61–67, DOI 10.1109/SST.2015.13
([IEEE](https://doi.org/10.1109/SST.2015.13)), note that "there is currently no consensus
on how a benchmark should be constructed" and propose guidelines. Waleed Zogaan, Palak
Sharma, Mehdi Mirakhorli and Venera Arnaoudova, *Datasets from Fifteen Years of Automated
Requirements Traceability Research: Current State, Characteristics, and Quality*, RE
2017, pages 110–121, DOI 10.1109/RE.2017.80 ([IEEE](https://doi.org/10.1109/RE.2017.80)),
review the datasets the field has used and their threats to validity, and propose a
framework for choosing among them. Dominik Fuchß, Sophie Corallo, Jan Keim, Janek Speit
and Anne Koziolek, *Establishing a Benchmark Dataset for Traceability Link Recovery
Between Software Architecture Documentation and Models*, ECSA 2022 Tracks and Workshops,
LNCS, 2023, DOI 10.1007/978-3-031-36889-9_30
([Springer](https://link.springer.com/chapter/10.1007/978-3-031-36889-9_30)), publish such a benchmark.
TransArC's
[replication package](https://github.com/ArDoCo/Replication-Package-ICSE24_Recovering-Trace-Links-Between-Software-Documentation-And-Code)
holds the documentation, the models and the gold standards of its evaluation.

Read as one line of work through Antoniol et al., T-BERT, TransArC and LiSSA, recovery
links requirements or documentation to code, or issues to commits. The links are
recovered, not written, and nothing checks them: each candidate is only ranked or
classified. The unit is source code and a manual page or requirement (Antoniol et al.),
an issue and a commit (T-BERT), a documentation sentence and a code file (TransArC), or a
requirement file and a code file (LiSSA). None of the four says what becomes of a link
when a file moves or a document is reworded. Their results are empirical: precision,
recall, F1 or mean average precision against gold links, such as TransArC's end-to-end
F1 of 0.82.

## agrees: Where grund agrees

- The link that matters runs between code and the text that explains it. RecoDoc starts
  from documentation that is not explicitly linked to the API it describes; grund starts
  from the same edge between code and prose, which "nothing on either side sees"
  ([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)).
- An authored link is the reference. Recovery scores its candidates against gold links
  that people wrote and vetted, and the benchmark studies above exist to make that
  comparison fair. A grund citation is a link of that authored kind, written as the work
  is done and checked on every edit ([§GRUND-links.2](../grund.md#2-holding-every-edit-to-them)).
- Recovered links are not yet links to rely on. Even TransArC's end-to-end F1 of 0.82
  leaves links missed or misplaced, and LiSSA's authors say more research is needed
  before such approaches apply in practice. grund needs citations an agent can trust,
  because one it cannot trust grounds nothing ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).

## departs: Where grund departs

- Recovery proposes. Its output is a set of candidate links, ranked by similarity
  (Antoniol et al.; Marcus and Maletic) or classified by a trained model or an LLM
  (T-BERT; LiSSA), and Hayes, Dekhtyar and Sundaram design their tool around the analyst
  who works through the candidates. grund proposes nothing and accepts no guessed
  citation: the author writes each one, and the check resolves it or reports it
  ([§GOAL-agent-grounding.3](../goals.md#3-what-this-rules-out)).
- Recovery reads meaning from text: identifier names and document words (Antoniol et
  al.), latent semantic structure (Marcus and Maletic), "the sentence semantics of
  requirements artifacts" (Guo et al.), or code-like terms in prose (RecoDoc). grund
  reads the text between IDs as opaque content ([§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)), so a link exists only
  where a citation is written.
- The learned and LLM-based methods depend on a trained model and on the data or the
  prompt it was given: Guo et al. train on existing trace links, T-BERT transfers from a
  related task, and Rodriguez et al. show that refining the prompt changes the ranking of
  candidate links. A verdict that rests on weights, training data or a prompt is one two
  installs could disagree on, and grund rules out anything that would let two installs
  disagree ([§FS-non-goals.13](../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).

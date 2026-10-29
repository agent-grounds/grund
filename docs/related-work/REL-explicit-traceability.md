# REL-explicit-traceability: explicit traceability, links written by hand

Explicit traceability is the practice of writing links by hand between requirements,
issues, design, code and tests, as safety-critical domains demand and as commit messages
carry in issue keys. grund descends from it and makes those links data the check holds:
citation directions say which kind cites which, and grounding makes the link from code to
text an obligation rather than a habit ([§GRUND-links.1](../grund.md#1-declaring-the-rules)).

## work: What the work is

**The field and its problem.** O. C. Z. Gotel and C. W. Finkelstein, *An Analysis of the
Requirements Traceability Problem*, ICRE 1994, pages 94–101, DOI 10.1109/ICRE.1994.292398
([IEEE](https://doi.org/10.1109/ICRE.1994.292398)), draw on "empirical studies, involving
over 100 practitioners". They distinguish pre-requirements-specification from
post-requirements-specification traceability, and report that most problems blamed on
poor traceability come from the pre-specification side. Jane Cleland-Huang, Orlena Gotel,
Jane Huffman Hayes, Patrick Mäder and Andrea Zisman, *Software Traceability: Trends and
Future Directions*, Future of Software Engineering at ICSE 2014, pages 55–69, DOI
10.1145/2593882.2593891 ([dblp](https://dblp.org/doi/10.1145/2593882.2593891)), record
that traceability is "Required in safety-critical systems by many certifying bodies", yet
"In practice, traceability is often conducted in an ad-hoc, after-the-fact manner".

**What the links are worth.** Patrick Mäder and Alexander Egyed, *Do Developers Benefit
from Requirements Traceability When Evolving and Maintaining a Software System?*,
Empirical Software Engineering 20(2):413–441, 2015, DOI 10.1007/s10664-014-9314-z
([Springer](https://link.springer.com/article/10.1007/s10664-014-9314-z)), ran a controlled experiment in
which 71 subjects re-performed real maintenance tasks on two third-party projects, half of
the tasks with traceability and half without. With it, subjects were on average 24%
faster and produced 50% more correct solutions.

**Safety-critical practice.** Patrick Mäder, Paul L. Jones, Yi Zhang and Jane
Cleland-Huang, *Strategic Traceability for Safety-Critical Projects*, IEEE Software
30(3):58–66, 2013, DOI 10.1109/MS.2013.60 ([IEEE](https://doi.org/10.1109/MS.2013.60)),
evaluate the traceability information in ten filings that manufacturers prepared for
review at the US Food and Drug Administration. They find nine widespread problems that
"affected regulators' ability to evaluate the product's safety in a timely manner", and
propose guidelines for "strategic traceability". Salome Maro, Jan-Philipp Steghöfer and
Miroslaw Staron, *Software Traceability in the Automotive Domain: Challenges and
Solutions*, Journal of Systems and Software 141:85–110, 2018, DOI
10.1016/j.jss.2018.03.060 ([Elsevier](https://doi.org/10.1016/j.jss.2018.03.060)), start
from safety standards such as ISO 26262, which "require established traceability". They
find 22 challenges and 16 unique solutions in the literature; a case study at an
automotive supplier met 17 of the challenges, six of them unsolved. The standards
themselves are RTCA DO-178C for airborne software (2011,
[RTCA](https://www.rtca.org/do-178/)), ISO 26262-8:2018, second edition, for road
vehicles ([EVS catalogue](https://www.evs.ee/en/iso-26262-8-2018)) and
IEC 62304:2006/Amd 1:2015, consolidated as edition 1.1, for medical-device software
([IEC](https://webstore.iec.ch/en/publication/22794)). Their texts were
not read: what they require of trace data is known here only from secondary sources, and
no sentence of this entry rests on them.

**Issue keys in commits.** Christian Bird, Adrian Bachmann, Eirik Aune, John Duffy,
Abraham Bernstein, Vladimir Filkov and Premkumar Devanbu, *Fair and Balanced? Bias in
Bug-Fix Datasets*, ESEC/FSE 2009, pages 121–130, DOI 10.1145/1595696.1595716
([open copy](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/bird2009fbb.pdf)),
observe that "only a fraction of bug fixes are actually labelled in source code version
histories" and find "strong evidence of systematic bias" in the labelled part.
Adrian Bachmann, Christian Bird,
Foyzur Rahman, Premkumar Devanbu and Abraham Bernstein, *The Missing Links: Bugs and
Bug-Fix Commits*, FSE 2010, pages 97–106, DOI 10.1145/1882291.1882308
([open copy](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/bachmann2010mlb.pdf)),
had a core developer of the Apache HTTP server annotate 493 commits exhaustively, and
report "serious and consequential problems in the data". Michael Rath,
Jacob Rendall, Jin L. C. Guo, Jane
Cleland-Huang and Patrick Mäder, *Traceability in the Wild: Automatically Augmenting
Incomplete Trace Links*, ICSE 2018, pages 834–845, DOI 10.1145/3180155.3180207
([arXiv](https://arxiv.org/abs/1804.02433)), note that version control lets developers
"tag each commit with one or more issue ID". Across six open-source projects "on average
only 60% of the commits were linked to specific issues", and their classifier recommends
the missing tags "at an average of 96% recall and 33% precision". Michael Rath and Patrick
Mäder, *The SEOSS 33 Dataset — Requirements, Bug Reports, Code History, and Trace Links
for Entire Projects*, Data in Brief 25, 2019, DOI 10.1016/j.dib.2019.104005
([Elsevier](https://doi.org/10.1016/j.dib.2019.104005)), publish 33 open-source projects
whose typed artifacts come from the issue tracker and version control, with the trace
links between them, time stamps and release versions; the data has its own DOI,
10.7910/DVN/PDDZ4Q ([Dataverse](https://doi.org/10.7910/DVN/PDDZ4Q)). In these studies
the issue key is a convention, written by hand and enforced by nothing. None describes a
check of the keys beyond Rath et al.'s recommender of missing ones, and none says what
becomes of a key when files move or an issue is retitled.

**Keeping links current.** Patrick Mäder, Orlena Gotel and Ilka Philippow, *Enabling
Automated Traceability Maintenance by Recognizing Development Activities Applied to
Models*, ASE 2008, pages 49–58, DOI 10.1109/ASE.2008.15
([IEEE](https://doi.org/10.1109/ASE.2008.15)), observe that "Without conscious effort,
traceability relations become increasingly inaccurate and irrelevant as the artifacts
they associate evolve". They keep the relations among UML model elements up to date by
recognizing the development activities applied to the models; the work concerns models,
not code. Paul Hübner and Barbara Paech, *Interaction-Based Creation and Maintenance of
Continuously Usable Trace Links between Requirements and Source Code*, Empirical Software
Engineering 25(5):4350–4377, 2020, DOI 10.1007/s10664-020-09831-w
([open access](https://link.springer.com/content/pdf/10.1007/s10664-020-09831-w.pdf)),
present ILCom, which links requirements in an issue tracker to source code files. It
"uses the interactions of developers with files in an integrated development environment
and issue identifiers provided in commit messages to create trace links continuously
after each commit", and removes likely wrong candidates using the structure of the source
code. Against a gold standard of link candidates vetted by the projects' developers, its
links reach a precision "above 90%" and a recall "almost at 80%". Its systematic review
covers 16 trace-maintenance approaches; the maintenance process it sketches for ILCom is
not yet implemented. Neither work discusses moved files or retitled requirements in
particular.

**A shared corpus.** Jane Cleland-Huang, Michael Vierhauser and Sean Bayley, *Dronology:
An Incubator for Cyber-Physical Systems Research*, ICSE-NIER 2018, pages 109–112, DOI
10.1145/3183399.3183408 ([dblp](https://dblp.org/doi/10.1145/3183399.3183408)), release
the artifacts of a safety-critical system that coordinates small unmanned aircraft,
"produced across multiple releases". Its [dataset page](https://dronology.info/datasets/),
project documentation, lists requirements, design definitions, tasks and source code,
with code traced to design definitions "explicitly by the development team" and "as jira
ID tags to a github commit message".

**Links verified by the build.** Thorsten Schlathölter, *ReqToCode: Embedding
Requirements Traceability as a Structural Property of the Codebase*, arXiv 2603.13999,
2026 ([arXiv](https://arxiv.org/abs/2603.13999)), a preprint and not peer-reviewed,
generates for each requirement one Traceable, "a language-native, generated code element
that represents a single requirement". Implementation and test code reference it, and
the links are "validated automatically during the build process". When a requirement
changes, a graduated lifecycle runs "from deprecation warnings to build failures". In its
worked example a requirement's title is reworded and the code "still references SWR_101
and compiles successfully — the identifier has not changed", while a comparison of
timestamps flags possible drift without failing the build. Moving files is not
discussed. The work sets itself against "recovering broken links after the fact", and
offers a design and an illustrative example, with no empirical evaluation.

## agrees: Where grund agrees

- Links between code and the text that says why are worth writing. Mäder and Egyed
  measured the gain for the people who maintain code; grund asks everything in a project
  to cite the point that says why it is the way it is ([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)).
- Links left voluntary are incomplete. Bird et al. and Rath et al. find issue keys missing
  from a large share of commits, and the labelled share biased. grund lets a project make
  the link from code to text an obligation: with grounding on, every source file, or
  every unit in it, must carry a citation that resolves ([§GRUND-links.1](../grund.md#1-declaring-the-rules), [§FS-check.3.6](../functional-spec/FS-check.md#36-ungrounded-unit-opt-in)).
- A link is typed by what it joins. Rath et al. trace between feature requests, bug fixes,
  commits and source code, and SEOSS 33 types the artifacts it links. grund types each
  declaration by kind, and a project's citation directions say which kind must, should,
  should not or must not cite which ([§DF-citation-directions](../decisions/functional/DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels)).
- A link is held when it is written, not reconstructed later. Where Cleland-Huang et al.
  find traceability "often conducted in an ad-hoc, after-the-fact manner", grund checks
  every citation as the work is done and fails the build on one that dangles
  ([§GRUND-links.2](../grund.md#2-holding-every-edit-to-them)).

## departs: Where grund departs

- Under certification, traceability is delivered as information a regulator reads: Mäder
  et al. evaluate what manufacturers filed with the FDA, and Cleland-Huang et al. survey
  "visualization techniques that use trace links". grund renders no matrix, report or
  view; its reads are building blocks for tools that do ([§FS-non-goals.5](../functional-spec/FS-non-goals.md#5-documentation-generation)).
- Trace maintenance works from history and events: SEOSS 33 joins issue-tracker artifacts
  with version-control history and release versions, ILCom creates links from IDE
  interaction logs after each commit, and Mäder, Gotel and Philippow update relations as
  they recognize activities applied to a model. grund keeps no trace database, reads no
  history and records no events; `grund refs` answers who cites an ID now, from one scan
  of the tree ([§FS-non-goals.6](../functional-spec/FS-non-goals.md#6-decision-database-audit-log-history-tracking)).
- Traceability spans whatever artifacts a project keeps, wherever they live: UML model
  elements (Mäder, Gotel and Philippow), commits tagged with an issue tracker's IDs (Rath
  et al.), requirements in an issue tracker (Hübner and Paech). grund validates no
  citation syntax outside its configured kinds and recognizes no markerless ticket key; a
  project that wants to cite a ticket commits it as a declaration ([§FS-non-goals.8](../functional-spec/FS-non-goals.md#8-generalization-beyond-the-id-scheme)).
- Some approaches lean on the structure of the code: ILCom filters its candidate links
  with it, and ReqToCode (not peer-reviewed) goes further, turning each requirement into a
  symbol the compiler resolves. grund does not parse code: it scans comments line by line
  and understands no classes, methods or types, so a citation resolves against grund's
  declarations, never against a symbol of the host language ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)).
- Much of this practice answers to a certifying body: Cleland-Huang et al. say so in
  general, Maro et al. for ISO 26262, Mäder et al. for the FDA. grund is written for the
  codebase, the people and agents working in it, and its CI ([§GRUND-grund.2](../grund.md#2-who-it-is-for)); it produces
  no certification evidence and claims nothing about what any standard would accept.

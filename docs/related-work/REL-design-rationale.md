# REL-design-rationale: architecture decisions and rationale, the why kept as records

Design-rationale research names the loss grund is built against: the reasons behind a
design that live only in someone's head disappear, which Jansen and Bosch call knowledge
vaporization. Decision records are the practice that answers it, and grund's own decisions
are kept as records of that kind ([§GRUND-grund](../grund.md#grund-grund-agents-stay-grounded-in-the-spec)).

## work: What the work is

A. Jansen and J. Bosch, *Software Architecture as a Set of Architectural Design
Decisions*, WICSA 2005, pages 109–120, DOI 10.1109/WICSA.2005.61
([IEEE](https://doi.org/10.1109/WICSA.2005.61)), coin "knowledge vaporization". Knowledge
about the design decisions an architecture rests on is "implicitly embedded in the
architecture" and lacks "a first-class representation", so it "disappears into the
architecture", which they hold partly responsible for high change cost and erosion. Their
remedy views an architecture "as a composition of a set of explicit design decisions".
This entry rests on the article's abstract only. J. Tyree and A. Akerman, *Architecture
Decisions: Demystifying Architecture*, IEEE Software 22(2), 2005, pages 19–27, DOI
10.1109/MS.2005.27 ([IEEE](https://doi.org/10.1109/MS.2005.27)), hold that "explicitly
documenting major architecture decisions" makes an architecture more transparent and
clarifies its rationale for all stakeholders (from the abstract).

Antony Tang, Muhammad Ali Babar, Ian Gorton and Jun Han, *A survey of architecture design
rationale*, Journal of Systems and Software 79(12), 2006, pages 1792–1804, DOI
10.1016/j.jss.2006.04.029 ([Elsevier](https://doi.org/10.1016/j.jss.2006.04.029)),
surveyed practitioners (81 valid responses). They "recognize the importance of documenting
design rationale" and use it to reason about their design choices, but report barriers to
using and documenting it (from the abstract).

Michael Nygard's blog post *Documenting Architecture Decisions* (2011,
[Cognitect](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions),
not peer-reviewed) proposes the architecture decision record (ADR): one short text file
per "architecturally significant" decision, kept "in the project repository under
doc/arch/adr-NNN.md", with a title, context, decision, status and consequences. "ADRs will
be numbered sequentially and monotonically. Numbers will not be reused." A reversed
decision is kept and marked superseded, "with a reference to its replacement". The post
describes no link from a record to the code it governs, nothing that checks the records,
and nothing about a record's file being moved or retitled.

Two studies look at ADRs in use:

- Georg Buchgeher, Stefan Schöberl, Verena Geist, Bernhard Dorninger, Philipp Haindl and
  Rainer Weinreich, *Using Architecture Decision Records in Open Source Projects — An MSR
  Study on GitHub*, IEEE Access 11, 2023, pages 63725–63740, DOI 10.1109/ACCESS.2023.3287654
  ([IEEE](https://doi.org/10.1109/ACCESS.2023.3287654)), is a mining study of ADR use on
  GitHub. Adoption is "still low" but rises every year, about half of the repositories
  with ADRs hold one to five, and most use the template Nygard proposed (from the
  abstract).
- Bardha Ahmeti, Maja Linder, Raffaela Groner and Rebekka Wohlrab, *Architecture Decision
  Records in Practice: An Action Research Study*, ECSA 2024, pages 333–349, DOI
  10.1007/978-3-031-70797-1_22 ([Springer](https://link.springer.com/chapter/10.1007/978-3-031-70797-1_22)),
  introduced ADRs at one company and observed them for three months. They addressed the
  challenges of documentation culture, knowledge transfer and prioritization, left those
  of distributed systems open, and "the decision on where documentation is stored has a
  massive influence on its perceived usefulness" (from the abstract).

Rudra Dhar, Karthik Vaidhyanathan and Vasudeva Varma, *Can LLMs Generate Architectural
Design Decisions? An Exploratory Empirical Study*, ICSA 2024, pages 79–89, DOI
10.1109/ICSA59870.2024.00016 ([IEEE](https://doi.org/10.1109/ICSA59870.2024.00016)), had
GPT and T5-based models write the decision of an ADR from its context. In a zero-shot
setting GPT-4 produced relevant and accurate decisions that still "fall short of
human-level performance" (from the abstract).

Neither Nygard's post nor, from its abstract, the Buchgeher study describes a mechanism
that links a record to code or checks the records. None of these works states a formal
guarantee; the evidence they report comes from a survey, a mining study, action research
and an exploratory study.

## agrees: Where grund agrees

- The loss is the one grund names: the why that lives in someone's head moves on with
  them ([§GRUND-grund](../grund.md#grund-grund-agents-stay-grounded-in-the-spec)). Tang et al.'s practitioners value rationale and find it hard to
  record; grund's remedy is to write the why down as a structure and hold every change to
  it.
- A decision is explicit and first-class, as Jansen and Bosch and as Tyree and Akerman
  ask. grund's default kinds include two decision kinds, `DF` for product behavior and
  `DA` for architecture, each declaration the H1 of its own file with its own ID
  ([§FS-config.3.4.4](../functional-spec/FS-config.md#344-the-default-kinds)).
- The records live in the repository, as in the repositories Buchgeher et al. mined, and
  Ahmeti et al. found that where documentation is stored shapes its usefulness. grund
  scans them in the same run as the code ([§GRUND-grund.1](../grund.md#1-what-grund-does-about-it)), and code, docs, decisions and
  tests cite the point that says why ([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold)). In this repository the spec point a
  decision settles cites it, `grund refs` lists every site that cites a decision now
  ([§FS-refs](../functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)), and a decision nobody cites is reported as unused ([§FS-check.4.1](../functional-spec/FS-check.md#41-unused-declaration)).

## departs: Where grund departs

- A decision record carries a status, and a reversed decision is kept, marked superseded,
  with a reference to its replacement; that is Nygard's template, the one most of the
  repositories Buchgeher et al. mined use. grund keeps no decision log and no status
  history: a status line is prose to it, and git is the audit log ([§FS-non-goals.6](../functional-spec/FS-non-goals.md#6-decision-database-audit-log-history-tracking)). What
  it checks is narrower. Where a record names its replacement with a `§` citation, as
  this repository's superseded records do ([§DA-pypi-package-name](../decisions/architectural/DA-pypi-package-name.md#da-pypi-package-name-pypi-uses-gnd-cli-as-the-package-name)), that citation must
  resolve ([§FS-check.3.1](../functional-spec/FS-check.md#31-dangling-citation)).
- An ADR follows a template, most often Nygard's. grund fixes no record template and no
  kinds: a project declares its own kinds ([§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds)), and declaring any replaces the
  defaults entirely ([§FS-config.3.4.4](../functional-spec/FS-config.md#344-the-default-kinds)). `DF` and `DA` are defaults and this repository's
  choice, and the status, context and decision parts of its records are a convention grund
  does not check. A project that wants named chapters required can state that as a
  chapter rule ([§FS-rules.3.1](../functional-spec/FS-rules.md#31-chapter-presence)).
- Dhar et al. have a model write the decision. grund writes none: it checks and retrieves
  what people and agents wrote ([§FS-non-goals.7](../functional-spec/FS-non-goals.md#7-inter-agent-messaging-or-workflow)).

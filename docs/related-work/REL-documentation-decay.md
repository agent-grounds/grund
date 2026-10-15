# REL-documentation-decay: documentation decay, references that fall behind the code

Documentation decay is what mining studies find where nothing checks a reference: links in
comments and commit messages die, and documentation goes on naming code that has been
deleted. It is the measured failure beneath grund's correctness floor, that every citation
resolves ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).

## work: What the work is

- Hideaki Hata, Christoph Treude, Raula Gaikovina Kula and Takashi Ishio, *9.6 Million
  Links in Source Code Comments: Purpose, Evolution, and Decay*, ICSE 2019, DOI
  10.1109/ICSE.2019.00123 ([arXiv:1901.07440](https://arxiv.org/abs/1901.07440)), studied
  about 9.6 million links in source code comments. Licenses, software homepages and
  specifications are common link targets. Links "are rarely updated, but many link targets
  evolve", and "almost 10% of the links included in source code comments are dead". Most
  of the link-fixing pull requests the authors then opened were merged.
- Tao Xiao, Sebastian Baltes, Hideaki Hata, Christoph Treude, Raula Gaikovina Kula, Takashi
  Ishio and Kenichi Matsumoto, *18 million links in commit messages: purpose, evolution,
  and decay*, Empirical Software Engineering 28, 2023, DOI 10.1007/s10664-023-10325-8
  ([arXiv:2305.16591](https://arxiv.org/abs/2305.16591)), asked the same questions of
  18,201,165 links in the commit messages of 23,110 GitHub repositories: 14% of the links
  prone to evolve became unavailable over time, and 70% of the distinct links suffer from
  decay.
- Emad Aghajani, Csaba Nagy, Olga Lucero Vega-Márquez, Mario Linares-Vásquez, Laura Moreno,
  Gabriele Bavota and Michele Lanza, *Software Documentation Issues Unveiled*, ICSE 2019,
  DOI 10.1109/ICSE.2019.00122
  ([DOI](https://doi.org/10.1109/ICSE.2019.00122)), mined 878
  documentation-related artifacts from mailing lists, Stack Overflow discussions, issue
  trackers and pull requests into a taxonomy of 162 types of documentation issues, in four
  categories: what the documentation's content says, how that content is written and
  organized, the documentation process, and documentation tools. Up-to-dateness,
  documentation that has fallen out of sync with the rest of the system, accounts for 39%
  of the content issues; among them the authors observed documented code that no longer
  exists, and they suggest that up-to-dateness and completeness issues can benefit from
  traceability between documentation and code. The follow-up, Emad Aghajani, Csaba Nagy,
  Mario Linares-Vásquez, Laura Moreno, Gabriele Bavota, Michele Lanza and David C.
  Shepherd, *Software Documentation: The Practitioners' Perspective*, ICSE 2020, DOI
  10.1145/3377811.3380405, surveyed 146 practitioners on which documentation issues they
  find most relevant and which types of documentation they consider important for specific
  tasks.
- Wen Siang Tan, Markus Wagner and Christoph Treude, *Detecting outdated code element
  references in software repository documentation* (DOCER), Empirical Software Engineering
  29, 2024, DOI 10.1007/s10664-023-10397-6
  ([Springer](https://link.springer.com/article/10.1007/s10664-023-10397-6)), detect code element references in
  README files and wiki pages that "survive in the documentation after all source code
  instances have been deleted", finding the candidate references with regular expressions.
  Of more than 3,000 GitHub projects, most contained at least one outdated code element
  reference at some point in their history, and more than a quarter of the 1,000 most
  popular contained one; some issues the authors filed have led to documentation fixes.
  Their *Wait, wasn't that code here before? Detecting Outdated Software Documentation*,
  ICSME 2023, DOI 10.1109/ICSME58846.2023.00071
  ([DOI](https://doi.org/10.1109/ICSME58846.2023.00071)), packages the approach as a
  GitHub Actions tool that scans a project's documentation whenever a pull request is
  opened.
- Barthélémy Dagenais and Martin P. Robillard, *Using Traceability Links to Recommend
  Adaptive Changes for Documentation Evolution* (AdDoc), IEEE TSE 40(11), 2014, DOI
  10.1109/TSE.2014.2347969 ([DOI](https://doi.org/10.1109/TSE.2014.2347969)), discover
  documentation patterns, coherent sets of code elements documented together, and report
  violations of those patterns as the code and the documentation evolve. In a
  retrospective analysis of four Java open-source projects, at least 50% of all changes to
  the documentation were related to existing patterns.

In the link-rot studies and DOCER, the reference runs from a comment or a commit message to
a URL its author wrote, or from documentation to a code element name that a tool recovers
heuristically. What is checked is presence: whether a URL's target is still reachable, or
whether the named element still exists in the code. URLs decay, about 10% of the links in
comments being dead. The evaluations are mining studies, DOCER's over more than 3,000
projects.

## agrees: Where grund agrees

- References that nothing checks rot, and these studies measure how fast. grund holds its
  own references to the opposite floor: a passing repository has zero dangling citations
  and zero broken section coordinates, and a missed one is a bug ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration),
  [§FS-check.3.1](../functional-spec/FS-check.md#31-dangling-citation), [§FS-check.3.2](../functional-spec/FS-check.md#32-missing-section)).
- Hata et al. find specifications a common target of links written in code. That edge,
  from code to the text that says why it is so, is the one grund checks, from comments in
  many languages through one resolver ([§GRUND-links](../grund.md#grund-links-the-cross-linking-rules-are-hard-to-define-and-to-hold), [§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful)). A grund
  address holds no file path and no title, so moving the file or rewording its heading
  does not break it ([§GRUND-schema.2](../grund.md#2-keeping-it)).
- Tan, Wagner and Treude's GitHub Action scans for outdated references whenever a pull
  request is opened. grund's check is likewise a pre-merge check, and fast enough to run
  on every save ([§GRUND-grund.2](../grund.md#2-who-it-is-for), [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).
- Aghajani et al. suggest traceability between documentation and code as a help against
  outdated documentation, and AdDoc uses traceability links to recommend documentation
  changes as the code evolves. In grund the links are written by their author and
  checked, and when a declaration changes, `grund refs` names every site that cites it
  ([§FS-refs](../functional-spec/FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)).

## departs: Where grund departs

- The link-rot studies measure URLs. grund validates no URL and no Markdown anchor and
  leaves both to a link checker such as lychee ([§FS-non-goals.1](../functional-spec/FS-non-goals.md#1-markdown-link-validation)), which
  [§REL-traceability-tools](REL-traceability-tools.md#rel-traceability-tools-requirements-traceability-tools-the-neighbours-beside-grund) lists.
- Whether a URL's target is still there can only be asked of the network. grund's check
  performs no network I/O, and a citation is grounded by repository bytes alone
  ([§FS-non-goals.11](../functional-spec/FS-non-goals.md#11-network-access-during-a-check), [§REQ-runs-offline](../requirements/REQ-runs-offline.md#req-runs-offline-verification-never-depends-on-an-external-service)).
- DOCER matches names found in documentation against the code, and AdDoc mines which code
  elements are documented together; both read the code's elements. grund does not parse
  code and knows no classes, methods or types ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)), so it cannot tell that a
  name in the documentation is gone from the code. Its references point at declared IDs,
  not at code elements.
- Aghajani et al.'s taxonomy covers what documentation says, how it is written, the
  process that produces it and the tools around it. grund judges none of these: it reads
  the text between IDs as opaque and lints no English ([§FS-non-goals.2](../functional-spec/FS-non-goals.md#2-spelling-grammar-prose-quality)). Documentation
  that describes behaviour the code no longer has is invisible to it, unless a citation in
  that documentation stops resolving.

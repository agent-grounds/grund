# grund — agent instructions

<!-- BEGIN GRUND MANAGED BLOCK -->
## Grounding with grund (block v42)

This project uses [`grund`](https://github.com/agent-grounds/grund): every spec, goal, decision, and end-to-end test has a stable ID `<KIND>-<slug>[.<section>]` (`KIND ∈ {GRUND, GOAL, FS, REQ, AR, DF, DA, RM, DISC, RULE, REL}`), cited with the marker `§` — e.g. `<§>FS-user-login.3.1`, where the ID is a shape illustration rather than a real one in this repo, which is what the `<§>` escape says. Type `$$` in a grund-aware editor and it becomes `§`. Bare ID-shaped tokens are ignored — config version 2 recognizes only `§`-prefixed citations, with no opt-out.

### Grounding from a citation

A `§<ID>` is a pointer to a fact, not a file path. Resolve it with `grund` and climb only as far as needed:

- `grund <ID>` — the lead (heading-less, cut at the first child section). The cheap first read for a bare `§<ID>` citation.
- `grund <ID> --toc` — the lead plus the nested section map. Use to choose which subsection to fetch next.
- `grund <ID> --full` — the entire body. Escalate to this when narrower reads aren't enough.
- `grund <ID> --brief` — heading + first paragraph only.
- `grund refs <ID>` — every site that cites the ID; add `--summary` for one line per file. Run before renaming or moving a declaration.
  After the rename or move, edit the declaration and affected authored citation IDs, then run `grund fmt --write` from the config/project root or workspace root. When cross-reference emission applies, it re-derives generated Markdown link destinations and anchors; the root-scoped run also updates eligible section locks. Run `grund check` and `grund fmt --check`, then commit the lock changes. Locks are keyed by ID, so a file move alone does not rename one. fmt does not replace authored IDs, repair arbitrary URLs, or silently approve a protected section move; fix citations or leave a pointer for that move first. See [Maintaining declarations](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/structure.md#maintaining-declarations).
- `grund list` / `grund list --kind FS,REQ` — discover IDs if you get lost
- `grund list --unused` — the declarations nothing cites yet, in one call
- `grund list --size=words --top 10` — find heavy leads and move detail into citable child points before reading them in full
- `grund list --glob '<KIND>-*' --total` — count the coordinates a pattern selects; `grund show --batch --glob '<KIND>-*' --toc --format json` then reads each one's section map in the same scan, from `.result.sections[]` (`path`, `title`, `depth`). `*` matches within one component, `**` any number of whole components, and `{a,b}` either alternative. More recipes: [Querying grund](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md).

### Project map

- [GRUND](docs/grund.md): Why: project motivation
- [GOAL](docs/goals.md): Where: project direction and outcomes
- [FS](docs/functional-spec): What: behavior, requirements, and constraints
- [REQ](docs/requirements): Hard requirements: what grund must never break
- [AR](docs/architecture): How: high-level implementation, structure, and design
- [DF](docs/decisions/functional): Product behavior decisions and tradeoffs
- [DA](docs/decisions/architectural): Architecture decisions and tradeoffs
- [tests/e2e/](tests/e2e): User scenarios: black-box proof of the spec
- [tests/integration/](tests/integration): Integration tests: proof that the parts fit as designed
- [RM](docs/roadmap.md): Planned milestones and sequencing
- [DISC](docs/discussions): Design discussions and proposals
- [RULE](docs/rules): Executable conventions: the rules grund check holds this repository to
- [REL](docs/related-work): Related work: the ideas grund descends from and the tools beside it
- [skills/](skills): Agent review and automation skills
- [examples/](examples): Worked examples: user-facing walkthroughs that double as fixtures
- [.github/workflows/](.github/workflows): CI and release workflows: the gate on GitHub, and how a release ships
- [scripts/](scripts): Repository scripts: hook checks, release preparation, benchmarks, PGO build
- [crates/grund-core/assets/templates/](crates/grund-core/assets/templates): Init scaffold templates: what grund init writes, verbatim
- [README.md](README.md): The README: the shop window for users, which links the user guides and cites no spec
- [AGENTS.md](AGENTS.md), [docs/](docs), [crates/](crates), [tests/bindings/](tests/bindings): The engine, CLI and bindings, and every scanned file no other row claims

### Declarations and citations

Declarations are heading lines `# <ID>: …` in markdown. In a code doc-comment (Rustdoc, Javadoc, JSDoc, Python docstring, Go `//`, …) drop the `#` — write `/// <ID>: …` directly. Named sections are enabled: use explicit complete paths (`## goals: Goals`, `### goals.performance: Performance`, `### goals.3: Ordered child`) so `§<ID>.goals.performance` resolves; handles are letter-first lowercase names, and `name.number` and `number.name` are both legal; a cited address with a name under a number is recorded in the section lock. Heading depth must match each path component (an error). Purely numbered headings remain citable. In a declaration body, Markdown or doc-comment, a heading may instead write only the component it adds and take the rest from the heading one level up: `### .1: Username` under `## format: Formatting` declares `§<ID>.format.1`, the one coordinate it is cited and reported by. After citing such a heading, run `grund fmt --write` so the section lock records it, and commit the lock. Every non-declaration heading inside a Markdown declaration body must carry a numbered or enabled named section path; file titles, headings that close the body, fenced examples, non-ATX text, and source doc-comments stay exempt, while bold labels remain the non-citable alternative. One doc-comment may declare multiple IDs (e.g. an `AR-` and an `FS-` on the same class) — each gets its own body. An inline source declaration is reachable from the configured kind home via a one-line stub: `# <ID>: [<path>](<path>)`.

### Rules

- **Spec first.** For behavior or design changes, write or update the most-specific spec point before code.
- **Cite as you write.** Place `§<ID>` at the point a claim or behavior is made — on the doc-comment for a whole behavior, inline beside the clause it enforces.
- **Marker = live citation.** A `§`-prefixed token resolves and is checked wherever it appears — including inside Markdown backticks. To mention an ID without citing it, write `<§><ID>`, omit the marker, or use a fenced code block.
- **Inline citation style.** Inline notes: ≤ 1 line preferred, hard cap 3 lines; ≤ 100 columns. A note is one comment block: a blank line splits it, an empty comment line does not. Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell, SQL or Lisp comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there.
- **Always cite the most-specific point.**

### Citation directions

`must`/`never` are `grund check` errors; `should`/`avoid` are suggestions (`grund check --suggestions`). Every source file must cite a declared ID or declare one inline; every file under tests/e2e/, tests/integration/, examples/, .github/workflows/, scripts/, and README.md must cite one.

- Each **GOAL** declaration should cite GRUND or GOAL.
- Each **FS** declaration should cite GOAL or FS; never cite AR.
- Each **REQ** declaration should cite GRUND or GOAL; never cite AR.
- Each **AR** declaration should cite FS or GOAL.
- Each **DF** declaration should cite FS or GOAL.
- Each **DA** declaration should cite AR or FS.
- Each file in **tests/e2e/** must cite FS; avoid citing AR.
- Each file in **tests/integration/** should cite AR.
- Each **RULE** declaration should cite FS or GOAL.
- Each **REL** declaration should cite GRUND or GOAL.
- Each file in **examples/** must cite FS; never cite AR.
- Each file in **.github/workflows/** should cite FS or AR.
- Each file in **scripts/** should cite FS or AR.
- The file **README.md** must not cite FS and AR.
- Each source file outside the Project map (**code**: The engine, CLI and bindings, and every scanned file no other row claims) that cites anything should cite FS or AR.
Anything not listed above is allowed.

### Declaration shapes

`grund check` holds each declaration of these kinds to these fields, as `grund schema` lists them:

- **FS**
  - field terms must · first
- **AR**
  - field placement must · first
  - field terms must · after placement
- **DF**
  - field status must · first · one of Accepted, Superseded
- **DA**
  - field status must · first · one of Accepted, Superseded

### Chapter rules

`must`/`must not` are `grund check` errors; `should`/`should not` are suggestions (`grund check --suggestions`).

- Each FS must have exactly one Terms chapter. [§RULE-terms](docs/rules/RULE-terms.md#rule-terms-each-fs-must-have-exactly-one-terms-chapter)

### Clickable citations

On repository web surfaces, link `§<ID>` to the PR branch in PR bodies, the reviewed commit in reviews, an exact commit for permalinks, and the default branch otherwise; fall back to plain when unsure. In local conversations, follow `§<ID>` with its declaration location as plain `path:line` text; fall back to the bare citation when unsure. If a user-level grund block states a local-conversation rendering, follow that instead: that machine knows what its surface can open.
<!-- END GRUND MANAGED BLOCK -->

## Repository workflow

- A pull request writes nothing for the changelog: its title is its release line, because a release lists the pull requests merged since the previous tag by their titles ([§FS-distribution.4.6.2](docs/functional-spec/FS-distribution.md#462-one-line-per-pull-request-its-title-linked)), so write the title for a reader of the release notes. A change that moves a verdict writes its compatibility notice as a `## release-note: Release note` section on its decision record, and a verdict correction must ([§FS-distribution.4.6.4](docs/functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions), [§REQ-backwards-compatibility.5](docs/requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)).
- Edit `AGENTS.md`, never the `CLAUDE.md` symlink ([§REQ-agents-md.1](docs/requirements/REQ-agents-md.md#1-one-source-symlinked-companions)); this entrypoint's contract is [§REQ-agents-md](docs/requirements/REQ-agents-md.md#req-agents-md-the-agent-entrypoint-stays-managed-and-grounded), the README's is [§REQ-readme](docs/requirements/REQ-readme.md#req-readme-the-readme-is-the-grounded-shop-window).
- One fact, one point, one implementation: settle a fact's defining point in the spec before the code, have other specs cite it rather than restate it, and derive it once in code that every other place reads ([§AR-system.principle](docs/architecture/README.md#principle-one-fact-one-point-one-implementation)).

<!-- BEGIN FISSILE MANAGED BLOCK -->
## Keeping Files Small With fissile (v3)

This repository caps file size with [`fissile`](https://github.com/agent-grounds/fissile)
so that agents spend fewer tokens reading. Run `fissile check --staged` before
claiming work is done; its findings say what to split and how. Where the
pre-commit hook is installed it runs that same check — never get past it with
`--no-verify`.
<!-- END FISSILE MANAGED BLOCK -->

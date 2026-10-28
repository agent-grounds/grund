# repo — agent instructions

<!-- BEGIN GRUND MANAGED BLOCK -->
## Grounding with grund (block v43)

This project uses [`grund`](https://github.com/agent-grounds/grund): every spec, goal, decision, and end-to-end test has a stable ID `<KIND>-<slug>[.<section>]` (`KIND ∈ {ADR}`), cited with the marker `§` — e.g. `<§>FS-user-login.3.1`, where the ID is a shape illustration rather than a real one in this repo, which is what the `<§>` escape says. Type `$$` in a grund-aware editor and it becomes `§`. Bare ID-shaped tokens are ignored — config version 2 recognizes only `§`-prefixed citations, with no opt-out.

### Grounding from a citation

A `§<ID>` is a pointer to a fact, not a file path. Resolve it with `grund` and climb only as far as needed:

- `grund <ID>` — the lead, cut at the first child section: heading-less for a whole declaration, while `grund <ID>.<section>` opens with that section's heading line. The cheap first read for a bare `§<ID>` citation.
- `grund <ID> --toc` — the lead plus the nested section map. Use to choose which subsection to fetch next.
- `grund <ID> --full` — the entire body. Escalate to this when narrower reads aren't enough.
- `grund <ID> --brief` — heading + first paragraph only.
- `grund refs <ID>` — every site that cites the ID; add `--summary` for one line per file. Run before renaming or moving a declaration.
  After the rename or move, edit the declaration and affected authored citation IDs, then run `grund fmt --write` from the config/project root or workspace root. When cross-reference emission applies, it re-derives generated Markdown link destinations and anchors; the root-scoped run also updates eligible section locks. Run `grund check` and `grund fmt --check`, then commit the lock changes. Locks are keyed by ID, so a file move alone does not rename one. fmt does not replace authored IDs, repair arbitrary URLs, or silently approve a protected section move; fix citations or leave a pointer for that move first. See [Maintaining declarations](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/structure.md#maintaining-declarations).
- `grund list` / `grund list --kind ADR` — discover IDs if you get lost
- `grund list --unused` — the declarations nothing cites yet, in one call
- `grund list --size=words --top 10` — find heavy leads and move detail into citable child points before reading them in full
- `grund list --glob '<KIND>-*' --total` — count the coordinates a pattern selects; `grund show --batch --glob '<KIND>-*' --toc --format json` then reads each one's section map in the same scan, from `.result.sections[]` (`path`, `title`, `depth`). `*` matches within one component, `**` any number of whole components, and `{a,b}` either alternative. More recipes: [Querying grund](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md).

### Project map

- [ADR](docs/adr): Architecture decisions
- [docs/runbooks/](docs/runbooks): Operational runbooks

### Declarations and citations

Declarations are heading lines `# <ID>: …` in markdown. In a code doc-comment (Rustdoc, Javadoc, JSDoc, Python docstring, Go `//`, …) drop the `#` — write `/// <ID>: …` directly. Named sections are enabled: use explicit complete paths (`## goals: Goals`, `### goals.performance: Performance`, `### goals.3: Ordered child`) so `§<ID>.goals.performance` resolves; handles are letter-first lowercase names, and `name.number` and `number.name` are both legal; a cited address with a name under a number is recorded in the section lock. Heading depth must match each path component (an error). Purely numbered headings remain citable. In a declaration body, Markdown or doc-comment, a heading may instead write only the component it adds and take the rest from the heading one level up: `### .1: Username` under `## format: Formatting` declares `§<ID>.format.1`, the one coordinate it is cited and reported by. After citing such a heading, run `grund fmt --write` so the section lock records it, and commit the lock. Every non-declaration heading inside a Markdown declaration body must carry a numbered or enabled named section path; file titles, headings that close the body, fenced examples, non-ATX text, and source doc-comments stay exempt, while bold labels remain the non-citable alternative. One doc-comment may declare multiple IDs (e.g. an `AR-` and an `FS-` on the same class) — each gets its own body. An inline source declaration is reachable from the configured kind home via a one-line stub: `# <ID>: [<path>](<path>)`.

### Rules

- **Spec first.** For behavior or design changes, write or update the most-specific spec point before code.
- **Cite as you write.** Place `§<ID>` at the point a claim or behavior is made — on the doc-comment for a whole behavior, inline beside the clause it enforces.
- **Marker = live citation.** A `§`-prefixed token resolves and is checked wherever it appears — including inside Markdown backticks. To mention an ID without citing it, write `<§><ID>`, omit the marker, or use a fenced code block.
- **Inline citation style.** Inline notes: ≤ 1 line preferred, hard cap 3 lines; ≤ 100 columns. A note is one comment block: a blank line splits it, an empty comment line does not. Doc-comments (`///`, `//!`, `/** */`, a docstring, a Go, Ruby, shell, SQL or Lisp comment right above a definition) are documentation, not notes: they are never measured, so cite in-sentence there.
- **Always cite the most-specific point.**

### Clickable citations

On repository web surfaces, link `§<ID>` to the PR branch in PR bodies, the reviewed commit in reviews, an exact commit for permalinks, and the default branch otherwise; fall back to plain when unsure.
<!-- END GRUND MANAGED BLOCK -->

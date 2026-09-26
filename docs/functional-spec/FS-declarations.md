# FS-declarations: a declaration is addressable once, from one allowed place, and holds nothing that is neither a coordinate nor a finding

A declaration is the line that introduces an ID, and it is what every citation in a
grund repository addresses. One invariant makes that address worth storing: an ID is
declared exactly once, it is declared where its kind's home allows it, and every heading
inside its body is either a coordinate a citation can reach or a finding `check` reports.
The checks below are that sentence enforced. Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and
[§GOAL-token-economy.1](../goals.md#1-what-this-requires).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms)
(source declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, workspace, member),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, caution), and [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (rule).

## checks: Checks

Each section below is one check `grund check` enforces about a declaration, and its name
is the diagnostic code verbatim — the token `--only` and `--ignore` take, as
[§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) publishes it and [§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code) requires. Severity is not part of
the address: each code's row in that catalog carries it, so a promotion edits a cell and
moves no coordinate. How a finding is rendered, selected and exited on is the command's
business, in [§FS-check.2](FS-check.md#2-outputs) and [§FS-check.1.4](FS-check.md#14-selecting-diagnostics-with---only-and---ignore).

### checks.duplicate: Duplicate declaration

The same ID declared more than once: any two declarations that are not stubs ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)), whether headings or inline doc-comment declarations and whether in one file or several. Reported per [§FS-check.2.1](FS-check.md#21-report-format): one error anchored at the lexicographically-first site, with the remaining sites listed in the message.

Duplicate JSON keys, cross-file JSON IDs, Markdown/JSON collisions, and overlapping opted-in ownership feed this same ambiguity rule even when their components agree. A duplicate target cannot be value-compared ([§FS-values.2.3](FS-values.md#23-duplicates-and-ownership)).

### checks.declaration-near-miss: Declaration near miss

A heading that opens the way a declaration does and does not match its effective ID format remains a declaration for read compatibility ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). The classic stumble is `# FS-login: …` under the default `{kind}-{number}-{slug}` — the `-NNN-` left out. Before grund 0.15.0, `check` emits one **warning** per such declaration, at the line a contributor has to edit:

```
docs/spec.md:1: `FS-login` resolves for compatibility but does not match [id] format = "{kind}-{number}-{slug}" — rename it or change the effective format; this warning becomes an error in grund 0.15.0
```

What counts is [§FS-declarations.checks.declaration-near-miss.1](FS-declarations.md#checksdeclaration-near-miss1-what-counts): the declaration colon is its discriminator ([§FS-declarations.checks.declaration-near-miss.2](FS-declarations.md#checksdeclaration-near-miss2-the-declaration-colon-is-the-discriminator)), and inline code, prose and fenced blocks never count ([§FS-declarations.checks.declaration-near-miss.3](FS-declarations.md#checksdeclaration-near-miss3-never-in-inline-code-prose-or-a-fenced-block)). The message states facts rather than a guessed rename ([§FS-declarations.checks.declaration-near-miss.4](FS-declarations.md#checksdeclaration-near-miss4-facts-not-a-guessed-rename)), the warning becomes an error in 0.15.0 ([§FS-declarations.checks.declaration-near-miss.5](FS-declarations.md#checksdeclaration-near-miss5-a-warning-before-0150-an-error-in-it)), and there is no opt-out ([§FS-declarations.checks.declaration-near-miss.6](FS-declarations.md#checksdeclaration-near-miss6-no-opt-out-no-rewrite)).

- **Code:** `declaration-near-miss` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### checks.declaration-near-miss.1: What counts

A line in declaration position — a Markdown heading, or a comment-prefixed line in a source file under the rules of [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments) — whose first token unambiguously begins with a configured citable kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), which the effective ID grammar rejects, and which is **followed by the declaration colon**. This also covers a per-kind format whose first literal after `{kind}` differs from the persisted token.

#### checks.declaration-near-miss.2: The declaration colon is the discriminator

A line opening with an ID-shaped token and no colon is prose far more often than it is a declaration attempt — a comment wrapped across lines whose continuation begins with one is the case that proved it, in this repository's own source. So the rule reads exactly the shape a declaration attempt has, `<KIND>-…: <title>`, and says nothing about the rest. A near miss written without a title is not reported; that is the cost, and it buys a rule that stays quiet on prose.

#### checks.declaration-near-miss.3: Never in inline code, prose, or a fenced block

The token stops at a backtick, so an inline-code mention is not a near miss. The position rules are the declaration rules exactly, so a near miss is only ever read where a declaration would have been: a bare `FS-login: …` in Markdown prose is not one ([§DF-code-declarations-drop-hash](../decisions/functional/DF-code-declarations-drop-hash.md#df-code-declarations-drop-hash-code-resident-declarations-may-drop-the--prefix)), and neither is anything inside a fenced block.

#### checks.declaration-near-miss.4: Facts, not a guessed rename

The message names the token as written and the effective template, states that lookup remains compatible, and offers the two real migration choices: rename the declaration and its citations, or change the effective format. It does **not** propose a corrected ID; assembling one from component patterns would guess what the author meant.

#### checks.declaration-near-miss.5: A warning before 0.15.0, an error in it

Before 0.15.0 it is a warning, so like every warning it leaves the exit code alone ([§FS-check.2](FS-check.md#2-outputs)): a run with no errors exits successfully but prints the located warning and no `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)). The declaration still appears in `list` and resolves through every reader; severity never changes recognition. In grund 0.15.0 the same code and location become an error, the deadline clause becomes the past-tense release report required by [§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name), and `check` exits `1`. The release guard and [§RM-off-grammar-declaration-error](../roadmap.md#rm-off-grammar-declaration-error-make-off-grammar-declarations-a-check-error-in-0150) prevent shipping the warning at or beyond that version.

#### checks.declaration-near-miss.6: No opt-out, no rewrite

There is no line-oriented opt-out or automatic rewrite: the position and colon rules bound recognition, and migration remains the repository author's choice.

### checks.broken-stub: Broken inline-spec stub

A `docs/` file whose H1 has the stub shape `# <ID>: [<text>](<path>)` where either the path does not exist, or the file at that path contains no inline declaration of the same ID. Relative stub links resolve as normal Markdown links first — relative to the stub file's directory — so `lychee` and rendered docs see the same target. If that path does not exist, `grund` falls back to resolving the path relative to the config root for compatibility with older stubs that wrote repo-root paths.

### checks.misplaced-declaration: Misplaced declaration (configured kind home)

A declaration that sits where a configured kind home does not allow it is a misplaced-declaration error, anchored at the declaration line. Three placements are refused: a single-file kind's declaration outside its file ([§FS-declarations.checks.misplaced-declaration.1](FS-declarations.md#checksmisplaced-declaration1-a-single-file-kind)), a declaration inside another kind's home ([§FS-declarations.checks.misplaced-declaration.2](FS-declarations.md#checksmisplaced-declaration2-another-kinds-home)), and any declaration in a non-citable home ([§FS-declarations.checks.misplaced-declaration.3](FS-declarations.md#checksmisplaced-declaration3-a-non-citable-home)). The home rules ([§FS-declarations.checks.misplaced-declaration.2](FS-declarations.md#checksmisplaced-declaration2-another-kinds-home), [§FS-declarations.checks.misplaced-declaration.3](FS-declarations.md#checksmisplaced-declaration3-a-non-citable-home)) apply to declaration lines and stub lines, not citations or prose mentions. A file that belongs to no configured home, or that matches several because configured homes overlap or nest, is not checked by them, because its expected kind is ambiguous.

#### checks.misplaced-declaration.1: A single-file kind

A kind configured with `file = "<path>"` in [[kinds]] ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) is a *single-file kind*: every declaration of that kind must live in that exact document, and one whose H1/H2 is found in any other scanned file is reported:

```
docs/notes.md:42: GOAL-foo must be declared in docs/goals.md (single-file kind)
```

Stubs (`# <ID>: [<text>](<path>)`) are exempt from this exact-file requirement: a stub points from a kind's home folder to an inline declaration elsewhere, a multi-file-kind feature, and a single-file kind has no folder to redirect from. This rule is the canonical mechanism that keeps `GRUND`, `GOAL`, and `RM` declarations in their documents, and what makes "one file, all goals inline" a checked invariant rather than a convention.

#### checks.misplaced-declaration.2: Another kind's home

Every configured `file` and `folder` is also a declaration-home boundary: a declaration line in a file that belongs to exactly one configured kind home must declare that home's kind. A `file` home matches only that exact path; a `folder` home matches files below that directory. The error names the declared kind, the expected home kind, and the configured home:

```
docs/functional-spec/FS-lsp.md:42: AR-router declares kind AR inside FS home docs/functional-spec
```

#### checks.misplaced-declaration.3: A non-citable home

A **non-citable home** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) admits no declaration of any kind. It has no kind an author could have declared instead, so the message names the place and says why rather than pointing at a kind that does not exist:

```
skills/review/SKILL.md:1: FS-review must not be declared in skills/ (not a citable home)
```

That is the rule working as designed, not a gap in it: `citable = false` says the directory is a place, and a place with a declaration in it is one of the two facts in conflict.

### checks.duplicate-section: Duplicate section path

Two or more citable section headings inside one declaration claiming the same dotted path ([AR-scanner.2.2](../architecture/AR-scanner.md#22-section-detection)) — either two `## 1. …` headings or two `## goals: …` headings under one `# FS-001-login`. Reported per [§FS-check.2.1](FS-check.md#21-report-format) in [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)'s shape: one error anchored at the first heading in file order, with every other heading line named in the message. Named and numeric coordinates use the same `duplicate-section` code ([§FS-errors.5](FS-errors.md#5-json-format)), the same multi-site `sites` record [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) carries, and the same no-ranking rule.

```
docs/functional-spec/FS-001-login.md:5: duplicate section FS-001-login.1 (also declared at docs/functional-spec/FS-001-login.md:9)
```

This is [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) one level down. A section path is a citation target, so two headings claiming it give `§FS-001-login.1` two destinations, and picking one silently is the guess [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) forbids by name. Decided in [§DF-duplicate-section-path](../decisions/functional/DF-duplicate-section-path.md#df-duplicate-section-path-a-section-coordinate-names-one-heading-or-the-run-says-so). The collision is scoped to one declaration ([§FS-declarations.checks.duplicate-section.1](FS-declarations.md#checksduplicate-section1-scoped-to-one-declaration)) and its body ([§FS-declarations.checks.duplicate-section.2](FS-declarations.md#checksduplicate-section2-scoped-to-that-declarations-body)), independent of the heading-level mode ([§FS-declarations.checks.duplicate-section.3](FS-declarations.md#checksduplicate-section3-independent-of-id-section_heading_levels)), and read from the record `show` reads ([§FS-declarations.checks.duplicate-section.4](FS-declarations.md#checksduplicate-section4-the-same-record-show-reads)).

#### checks.duplicate-section.1: Scoped to one declaration

Section paths are addressed as `<ID>.<path>`, so the same `1.` under two different declarations is two distinct coordinates and not a finding. Only headings sharing a declaration collide.

#### checks.duplicate-section.2: Scoped to that declaration's body

The headings judged are the ones inside the body [§FS-show.2.1](FS-show.md#21-whole-declaration-default) and [§FS-show.2.3.1](FS-show.md#231-what-counts-as-the-comment-block) delimit — in Markdown down to the next same-or-shallower heading, in a source file to the end of the comment block the declaration line opens. A `## 1.` further down the file — in the *next* item's doc-comment, or under a later unrelated heading — is not one of this declaration's sections: `grund <ID>.1` never reaches it, and reporting it would ask for a renumbering that changes what nothing points at. A stub ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)) is one link line whose tail is a path rather than a body, so it declares no sections at all and is never reported here; the headings that count are the inline home's, which is also the file `grund <ID>.<path>` reads.

#### checks.duplicate-section.3: Independent of `[id] section_heading_levels`

The mode ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)) governs how deep a heading must sit for the path it writes, which is a different fact; `## 1.` and `### 1.` under an H1 declaration both claim path `1` and are a duplicate in every mode, `"loose"` included. [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch) judges only the first heading, the one the path resolves to; a later claimant is no section target and is not additionally judged for depth, so it yields this finding alone ([§DF-duplicate-section-path.2.4](../decisions/functional/DF-duplicate-section-path.md#24-the-heading-level-rule-judges-only-the-heading-the-path-resolves-to)).

#### checks.duplicate-section.4: The same record `show` reads

This rule and [§FS-show.2.2.2](FS-show.md#222-ambiguous-section) answer from one recorded section set, so `grund <ID>.<path>` refuses exactly when this rule reports `<ID>.<path>` and returns a body exactly when it does not. Two readers that each decided for themselves would disagree — a fenced example, a heading past the end of the body — and a coordinate `check` calls clean but `show` will not resolve is [§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess) failing quietly in the other direction.

### checks.orphan-section: Orphan name-bearing section path

With `[id] named_sections = true`, every proper prefix of a name-bearing section path must be recorded in the same declaration before the descendant can be addressed. `### goals.performance: Performance` therefore requires a recorded `goals`; `### missing.performance: Performance` is one error at that heading's line even when another Markdown heading visually contains it. The check uses the complete recorded path set and is independent of heading-depth validation, so a heading with both a missing prefix and a wrong level produces both findings. Purely numeric paths are unchanged and are never judged by this rule.

The message names the orphan coordinate and its first absent prefix. Its code is `orphan-section`. This is a declaration-side structural error, not a missing-citation error: it is reported even when nobody cites the orphan, and a citation to it may independently be present and resolve to the recorded coordinate.

### checks.section-heading-level: Section heading level mismatch

`[id] section_heading_levels` ([§FS-config.3.3.2](FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)) sets how a citable section heading's Markdown depth must match its dotted path ([AR-scanner.2.2](../architecture/AR-scanner.md#22-section-detection)), and whether a mismatch is an error, a warning, or not reported. A mismatch the mode reports is anchored at the heading line. This point judges the depth of headings that already carry coordinates; the project-wide in-body Markdown ATX rule for a heading that carries none is [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading), independent of this mode. Bold labels are not headings and remain unchecked.

### checks.section-outside-declaration: Section outside a declaration

When the scanner encounters a numeric section heading, or an enabled named section heading, that is deeper than its stale declaration context but whose line lies inside no declaration body, `check` emits one located error at the heading. A numeric heading's exact message is `numbered section outside any declaration`; an enabled named heading's is `named section outside any declaration`. Both use the public code `section-outside-declaration`. Ownership is the declaration's body span ([§FS-declarations.checks.section-outside-declaration.1](FS-declarations.md#checkssection-outside-declaration1-ownership-is-the-body-span)), the rejected heading leaves the section map every consumer reads ([§FS-declarations.checks.section-outside-declaration.2](FS-declarations.md#checkssection-outside-declaration2-the-rejected-heading-leaves-the-section-map)), and the finding is reported like any other hard error ([§FS-declarations.checks.section-outside-declaration.3](FS-declarations.md#checkssection-outside-declaration3-an-ordinary-hard-finding)).

#### checks.section-outside-declaration.1: Ownership is the body span

Ownership is the body span already used for extraction and citing-side classification, not a second section-only approximation. In Markdown, a same-or-higher heading ends a declaration body even when that boundary heading is plain; a later deeper section-like heading is outside. In source, the end of a doc-comment or docstring ends ownership. A later declaration in the same comment block ends the earlier body and begins its own. An inline-spec stub owns only its single heading line, so numbered prose below the stub belongs to no stubbed declaration. A deeper section-like heading before any of those boundaries stays valid. A heading inside a Markdown fence is content and emits nothing ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)). Legal unmarked or plain headings are not errors under this point; adopting a general unmarked-heading policy is a separate change ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)).

#### checks.section-outside-declaration.2: The rejected heading leaves the section map

The rejected heading is excluded from the shared body-local section map before any consumer runs ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)). It cannot resolve a citation or query, enter completion or list/size output, become an embedded-value root, participate in duplicate-section detection, or acquire an LSP navigation target. `show` and other failed queries retain their ordinary missing-section semantics rather than printing this check-only message.

#### checks.section-outside-declaration.3: An ordinary hard finding

This is an ordinary hard finding under §[§FS-check.2](FS-check.md#2-outputs)–3. Text uses the located `<path>:<line>: error: <message>` form. JSON emits `{"severity":"error","path":<path>,"line":<line>,"code":"section-outside-declaration","message":<message>,"sites":null}`. `--only section-outside-declaration` retains it and `--ignore section-outside-declaration` removes it; a retained finding contributes exit `1`, while selecting it away restores the ordinary selected-report result. Parallel and workspace scans merge the record once under the workspace-relative path, never once per stale declaration. A narrowed scan judges the complete selected file, and `--full` applies the same code and message to otherwise out-of-scope files it adds. The LSP transports the same error severity, code, message, and heading range through its shared snapshot ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).

### checks.unmarked-heading: Unmarked Markdown heading

Before grund 0.15.0, a Markdown ATX heading that is deeper than a declaration heading and whose line is still inside that declaration's body is a fixed warning when it is neither another declaration nor a recognized numeric or enabled named section. The containing declaration is the nearest enclosing body, so a plain heading beneath a deeper child declaration names that child, not an overlapping ancestor. This is a project-wide rule with no configuration, severity selector, or permanent opt-out ([§DF-unmarked-markdown-headings](../decisions/functional/DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph)).

Which headings participate is [§FS-declarations.checks.unmarked-heading.1](FS-declarations.md#checksunmarked-heading1-which-headings-participate). The message is [§FS-declarations.checks.unmarked-heading.2](FS-declarations.md#checksunmarked-heading2-the-message) and its suggested coordinate [§FS-declarations.checks.unmarked-heading.3](FS-declarations.md#checksunmarked-heading3-the-suggested-coordinate); its rendering is [§FS-declarations.checks.unmarked-heading.4](FS-declarations.md#checksunmarked-heading4-rendering-exit-and-selection), its flip to an error [§FS-declarations.checks.unmarked-heading.5](FS-declarations.md#checksunmarked-heading5-an-error-in-grund-0150), and what it leaves unchanged [§FS-declarations.checks.unmarked-heading.6](FS-declarations.md#checksunmarked-heading6-no-command-numbers-the-heading).

#### checks.unmarked-heading.1: Which headings participate

Only ATX headings in scanned Markdown files participate. A heading inside a backtick or tilde fence is content. A file title before the first declaration, a same-or-shallower heading that closes a declaration body, source doc-comment text, setext text, and a bold label are outside the rule. A deeper declaration and a valid numeric or enabled named section already participate in the graph and are not unmarked. `--full` keeps this convention-scoped warning narrowed to the configured scan scope, as it does other convention findings ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)).

#### checks.unmarked-heading.2: The message

The warning is anchored at the heading line, uses code `unmarked-heading`, and has this text:

```text
unmarked heading inside <ID>; number it (<suggested heading>) as <ID>.<path>, declare an ID, or use a bold label; this warning becomes an error in grund 0.15.0
```

#### checks.unmarked-heading.3: The suggested coordinate

The suggested coordinate is guidance, not a rewrite. Its path depth follows the written ATX depth relative to the containing declaration. At each depth, grund uses the nearest preceding citable or already-suggested parent and appends one above the largest existing or earlier-suggested numeric sibling; it never fills a hole or reuses a coordinate. With no parent it starts one above the largest root numeric coordinate, or at `1` when none exists. If the authored heading skips a depth, missing parents are filled with `.1`. A named parent may therefore receive a numeric child such as `goals.1`. The suggested heading preserves the authored `#` depth and title and inserts the complete coordinate in that valid numeric or mixed form. A titleless ATX heading has no authored title to preserve, so its otherwise-identical suggestion uses the literal title `Untitled`; applying that complete suggested heading produces a recognized section and clears the warning.

#### checks.unmarked-heading.4: Rendering, exit, and selection

Text output uses the `<path>:<line>: warning: <message>` form. A warning leaves the exit at `0` but stands in place of the `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-marker)). JSON emits the same path, line, code, and message with `"severity":"warning"` and `"sites":null`. `--only unmarked-heading` retains it and `--ignore unmarked-heading` removes it through the ordinary exact-code selection rules. The LSP carries the same core finding with warning severity and the complete ATX heading as its range ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).

#### checks.unmarked-heading.5: An error in grund 0.15.0

In grund 0.15.0 the same site, code, suggestion, and all non-severity bytes stay stable except that the deadline clause becomes `this became an error in grund 0.15.0`; severity becomes error and a retained finding contributes exit `1`. [§RM-unmarked-heading-error](../roadmap.md#rm-unmarked-heading-error-make-unmarked-markdown-headings-errors-in-0150) owns that scheduled flip. Until then the warning window serves [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

#### checks.unmarked-heading.6: No command numbers the heading

No command numbers the heading: `grund fmt` remains unchanged. `show`, `list`, `refs`, `cover`, formatting, section resolution, and source scanning otherwise keep their current behavior, including the body boundary and rejected section behavior of [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration). `grund_config_version` stays `1`; the managed agent block is the existing mechanical repair surface and moves to v10 under `grund init` ([§FS-init.2.3.4.5](FS-init.md#2345-declaration-forms)).

### checks.oversized-lead: Oversized lead *(opt-in)*

When the effective project config contains the opt-in key from [§FS-config.3.1](FS-config.md#31-reference--citation-form):

```toml
[reference]
lead_size_warning = { max = <N>, unit = "<unit>" }
```

`check` measures each declaration and citable section lead in that project by [§FS-list.3.4](FS-list.md#34---size--per-point-lead-and-full-body-measurements). A lead whose selected measurement is strictly greater than `max` produces one warning at that site's heading line. Equality passes. A broken stub has no measurable lead and produces no size warning; duplicate declaration homes and duplicate section claimants are judged separately from their own site-local slices.

The message is [§FS-declarations.checks.oversized-lead.1](FS-declarations.md#checksoversized-lead1-the-message) and its exit and rendering [§FS-declarations.checks.oversized-lead.2](FS-declarations.md#checksoversized-lead2-exit-code-and-rendering). Only the key activates it ([§FS-declarations.checks.oversized-lead.3](FS-declarations.md#checksoversized-lead3-only-the-key-activates-it)), and which sites it judges is [§FS-declarations.checks.oversized-lead.4](FS-declarations.md#checksoversized-lead4-which-sites-it-judges).

#### checks.oversized-lead.1: The message

The fixed code is `oversized-lead`, severity is `warning`, and the exact text after `<path>:<line>: ` is:

```text
<coordinate> lead is <actual> <unit>, over the configured maximum of <max>; move detail into citable child sections, or promote a child section to its own ID after running grund refs <coordinate> --summary
```

The coordinate is local for a member-local check and workspace-qualified for a workspace-root check. The two remedies preserve grounding and citation stability; the message never suggests shortening or deleting it.

#### checks.oversized-lead.2: Exit code and rendering

A warning-only run exits `0` and replaces the text `success` marker; JSON uses the ordinary located diagnostic object ([§FS-errors.5](FS-errors.md#5-json-format)). The LSP publishes the same message, line, code, and warning severity as the CLI ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)). Any simultaneous error, including `duplicate` or `duplicate-section`, still decides exit `1`; the warning neither suppresses it nor changes its priority.

#### checks.oversized-lead.3: Only the key activates it

The absent key activates no measurement or finding and leaves the existing text and JSON check output byte-identical. `--only oversized-lead` and `--ignore oversized-lead` select the finding after the complete check and never activate it.

#### checks.oversized-lead.4: Which sites it judges

An explicit-path check judges only declaration and section sites scanned at that path. `--full` adds its existing out-of-scope reference findings but does not extend this repository policy beyond declarations in the configured scan scope. In a workspace, each member's effective key governs only that member's sites; a member without the key remains silent even when another member opts in.

## why: Why this exists

A declaration's address is the one thing a repository stores about it in a thousand
places at once, so what may go wrong with a declaration is the same question as what may
go wrong with an address. Gathering the ten checks here rather than under the command
that happens to report them puts each one beside the invariant it defends, and names each
one what a reader already holds when they need it: the code out of a finding they have
just read ([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)). `check` is one reader of this contract, and not the
only one — `show` refuses the coordinate this spec calls ambiguous, `fmt` rewrites toward
the canonical form it names, and the LSP transports every finding here unchanged — which
is the other reason the checks do not live inside the command.

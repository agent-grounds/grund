# FS-check: grund validates every citation in a repo

The `check` command scans a repo and reports every violation of the grund citation scheme. Validation is explicit as `grund check [<path>]`; the bare `grund <ID>` default belongs to [§FS-show.1](FS-show.md#1-inputs). Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, qualified citation, shorthand,
canonical form, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration, stub, doc-comment, note),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace, member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings)
(finding, severity, suggestion, caution, verdict, anchor), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction, level,
rule, grounded), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, component, binding, snapshot).

- **grounding unit** — What `require_grounding` measures: a whole scanned file, or one
  doc-comment block where the row's `grounding_level` says so. A unit is grounded when it holds
  at least one recognized citation.
- **obligation** — A `must` or `should` entry of `[citations.<kind>]`, asking whether each unit
  of the citing kind cites the target kind at least once. It is answered per unit.
- **prohibition** — A `must-not` or `should-not` entry, which fires once per offending citation
  site rather than once per unit.
- **blind spot** — A place the default scope never reads, bounded and declared rather than
  discovered, and what `--full` exists to look into.
- **full-tree scope** — The scope `--full` selects: every file the scan can reach, in place of
  the configured default scope.

## requirements: What every check run holds to

Four requirements on the command as a whole, marked as in [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). There is no fifth for the exit code, which is [§REQ-never-crashes.2](../requirements/REQ-never-crashes.md#2-exit-codes-are-the-api)'s frozen mapping and [§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)'s refusal to pass an incomplete view, and none for severity, whose set no flag or key moves ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)) and which changes only along [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path. Nor is speed one: its targets are [§GOAL-fast-feedback.1](../goals.md#1-performance-targets)'s, measured rather than specified, and [§FS-check.6](FS-check.md#6-watch-mode---watch)'s watch loop serves it; what this document adds is that no flag or scope buys time with a verdict ([§FS-check.requirements.flags](FS-check.md#requirementsflags-a-flag-adds-findings-filters-the-complete-report-or-changes-only-how-it-is-written--realized-directional-for-the-next-flag), [§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)).

### requirements.success-earned: `success` is earned, never a default — realized

Text `check` prints exactly `success` only for an empty selected report with no unselectable run-level line beside it ([§FS-check.2.1.3](FS-check.md#213-the-success-line)), and each way a run is known to read less than the scope it was given speaks in that line's place: nothing to scan ([§FS-check.2.2](FS-check.md#22-empty-scan)), an obligation — a `[citations]` entry or a `must` or `should` rule — with no unit to judge, in a narrowed run too ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)), nothing recognized in what was read ([§FS-check.4.5](FS-check.md#45-nothing-recognized)), a file the resolution scope could not read ([§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)), an absent optional member ([§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)), the files an `include_root = false` block leaves unread ([§FS-check.4.10.10](FS-check.md#41010-it-stands-in-place-of-the-success-line-and-never-moves-the-exit-code)), a `--full` with nothing to widen ([§FS-check.1.3.7](FS-check.md#137-a-path-the-flag-cannot-widen-earns-a-caution-not-a-refusal)), a file root whose extension keeps it unread ([§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read)), a rule that could not run under a narrowing `--only` ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), and a selected check the run leaves off ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). So the run that checked less never prints the word the run that checked everything prints. Serves [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements) and [§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips).

### requirements.narrow-never-lenient: A narrower run never passes what a wider run fails, about the files it reports — realized

About a file at or under its path, `grund check <path>` fails whatever `grund check .` fails: the path narrows the report and not the resolution ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)), and a finding that also names a site outside it is kept whole ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)). Where narrowing does shrink what is read, the run grows stricter, never quieter: one that loads no workspace reports a qualified citation it cannot resolve as an error ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure), [§FS-workspace.5.1](FS-workspace.md#51-a-member-run)), a rule object it cannot judge still fails as `invalid-rule` ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)), and a citation declared in a file it could not read is reported unresolved, with a caution saying why ([§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)). The plain run stands to `--full` the same way ([§FS-check.1.3.4](FS-check.md#134-purely-additive)). The promise is about verdicts on reported files: an unread file outside the report leaves the exit code alone ([§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)), and a slice is never asked the nothing-recognized question ([§FS-check.4.5.4](FS-check.md#454-asked-only-of-the-whole-project)). Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded).

### requirements.flags: A flag adds findings, filters the complete report, or changes only how it is written — realized, directional for the next flag

Each `check` flag but `--watch` is one of three. An **adder** — `--full` ([§FS-check.1.3.4](FS-check.md#134-purely-additive)), `--require-grounding` ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)), `--suggestions` ([§FS-check.2.3.2](FS-check.md#232-selecting-and-printing-suggestions)), `--rule` ([§FS-rules.8](FS-rules.md#8-command-surfaces)) — may turn a green run red, never the reverse, or open a channel that never moves the exit code. A **filter** — `--only`, `--ignore`, `--only-rule` — is a query over the complete report: each retained finding keeps its message, location and order, and the filter turns red green only over findings it visibly deselects, never over an incomplete run or a rule that could not run ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report), [§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). A **rendering** flag — `--format`, `--path-base` ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)) — changes bytes and no finding. `--watch` changes when the check runs, not what it reports ([§FS-check.6.2](FS-check.md#62-each-run-is-a-plain-grund-check)). A flag added from here on says which of the three it is; one that is none — that turns red green other than by a visible deselection, or skips work a finding depends on — is a new verdict and needs its own decision. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) and [§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips).

### requirements.no-declined-fix: An error never names a fix grund declines to make — realized, directional for the next finding

An error whose only named remedy the tool refuses to perform is one a repository can never clear ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite), [§FS-check.3.17.5](FS-check.md#3175-anything-else-is-not-an-entry)). So a fix a finding names is one the author can get made: `grund fmt --write` only where `fmt` would write the site, or would once the author lifted a refusal of their own configuring, such as `[fmt] exclude` ([§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)), and otherwise the hand edit, since not every fix is `fmt`'s ([§FS-check.3.15.3](FS-check.md#3153-the-rule-this-one-is-an-exception-to)). The resolving shorthand form is withheld where the text forbids the rewrite ([§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite)) and out of scope ([§FS-check.3.14.4](FS-check.md#3144-the-mechanical-shorthand-rewrite-is-withheld)), and a declaration-local citation's command clause in each case [§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld) lists. Stated today one finding at a time, it binds each finding added from here on. Serves [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## 1. Inputs

- Optional path argument, a directory or a single file, which narrows the report; with none, the run covers the project discovered from the current directory ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)).
- The scanned tree may hold Markdown and source files (Rust, Go, Java, TS, Python, etc.).
- Optional `grund.toml` configuring the run per [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) ([§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up)).
- Optional `[workspace]` config; when present and `check` is run at the workspace root, `check` validates alias-qualified cross-project citations per [§FS-workspace](FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace).
- `--watch` — check immediately, then stay resident and check again when an effective local input changes, each run a plain `check` ([§FS-check.6](FS-check.md#6-watch-mode---watch)).
- `--require-grounding` — turn the grounding check ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) on for this run regardless of `[reference] require_grounding` in `grund.toml` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). It only ever *adds* the check; it cannot switch off a config that already sets it. For findings the flag and the key are **one knob** ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)): the flag sets the same global default, so a `[[kinds]]` row that says `require_grounding = false` is still exempt under it, and `grounding_level` has no flag at all ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). A run-level flag that overrode the row would make the flag mean something the key cannot say.
- `--suggestions` — emit the suggestions channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)) for this run. The flag never adds an error or changes the exit code: it only surfaces the advisory records [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) lists, which the default run withholds.
- `--rule "<sentence>"` — add one ad-hoc chapter rule to the configured rules
  for this run. It never disables them; validation, deduplication, and exits are
  [§FS-rules.4](FS-rules.md#4-validation-lifecycle) and
  [§FS-rules.8](FS-rules.md#8-command-surfaces)'s.
- `--only <code>` — retain only findings whose exact code is in the selected set, and every `invalid-rule` when that set names a code a rule produces ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
- `--ignore <code>` — remove findings whose exact code is in the selected set ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
- `--only-rule` — retain only findings the `--rule` sentence authored, by their `authority` rather than by their code. It requires `--rule`; given alone it is an invocation error with exit `2` ([§FS-rules.8](FS-rules.md#8-command-surfaces), [§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
- `--full` — scan the whole config root past `[scan] include`, reporting unresolved citations as out-of-scope findings ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full), [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)) plus the scanner-invariant `section-outside-declaration` error ([§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)). It only ever *adds* findings; the in-scope report is unchanged.
- `--format text|json` — output shape, per [§FS-errors.5](FS-errors.md#5-json-format). The global flags `--version` and `--help` are handled before any scan ([§FS-cli](FS-cli.md#fs-cli-grunds-command-line-surface-conventions)).

### 1.1 Recognized citations

What a recognized citation is, and where one may appear, is [§FS-citations.recognized](FS-citations.md#recognized-recognized-citations).

#### 1.1.1 Off-grammar citations

How an off-grammar citation that a catalog declaration backs is read is [§FS-citations.recognized.off-grammar](FS-citations.md#recognizedoff-grammar-off-grammar-citations).

#### 1.1.2 Named-section candidates

How a named-section candidate is read is [§FS-citations.recognized.named-candidates](FS-citations.md#recognizednamed-candidates-named-section-candidates).

#### 1.1.3 String literals in source files

How a token inside a source string literal is read is [§FS-citations.recognized.string-literals](FS-citations.md#recognizedstring-literals-string-literals-in-source-files).

##### 1.1.3.1 Assigned Python triple-quoted data

Where an assigned Python triple-quoted string holds no citation is [§FS-citations.recognized.string-literals.assigned-data](FS-citations.md#recognizedstring-literalsassigned-data-assigned-python-triple-quoted-data).

#### 1.1.4 Markdown link destinations

How a token inside a Markdown link destination is read is [§FS-citations.recognized.link-destinations](FS-citations.md#recognizedlink-destinations-markdown-link-destinations).

#### 1.1.5 Contexts read as neither prose nor code

Which fenced code blocks hold no citation, and which other contexts skip one, is [§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code). What opens and closes a fence, and its tag, is [§FS-languages.fence](FS-languages.md#fence-fences).

##### 1.1.5.1 A raw HTML block holds no heading

A third Markdown context is read as raw text for structure but as prose for citations: a CommonMark **raw-text HTML block** (CommonMark's HTML block of type 1). It opens on a line whose text, after at most three leading spaces, begins with `<pre`, `<script`, `<style` or `<textarea`, case-insensitively, followed by whitespace, `>` or the end of the line. It closes after the first line, the opening line included, that contains the matching end tag `</pre>`, `</script>`, `</style>` or `</textarea>`, again case-insensitively. An unclosed block runs to end of file. No line of the block, its opening and closing lines included, is a heading: it declares no ID, opens no section, ends no lead and bounds no unit, however much it looks like `# <ID>: …` or `### 3.2 …`. CommonMark renders these lines as preformatted text and never as headings, so a heading recognized there would be a declaration no reader of the rendered page can see.

That a citation inside the block is live, unlike one in a fence, is [§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code). The two contexts do not nest. Inside a fence an HTML opener is fence content and opens nothing; inside the block a fence opener is block content and opens nothing. Every surface that asks whether a Markdown line is a heading reads the block by this one rule, as it reads the fence by [§FS-languages.fence](FS-languages.md#fence-fences). Other CommonMark HTML blocks, including comments, keep the existing reading.

##### 1.1.5.2 Front matter holds no heading

A fourth Markdown context is the file's **front matter**, the YAML or TOML block that Jekyll, Hugo, Zola, Pandoc and Docusaurus read at the top of a page. It opens only on the file's first line, and only when that line is exactly `---` or `+++`; a trailing `\r` is the line ending, not part of the line. It closes at the next line that is exactly the same delimiter, and a `---` block also closes at a line that is exactly `...`, as Pandoc allows; `...` never closes a `+++` block. The block is decided from its lines alone, before any YAML or TOML in it is read, so a line that is exactly the delimiter closes it even where it sits inside a multi-line string. A block that never closes is not front matter, and the file reads as it would without this rule. A first line that begins with a byte-order mark is not exactly a delimiter and opens nothing ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)). CommonMark and GFM define no front matter; this is the convention those tools share.

No line of the block, its delimiter lines included, is a heading: a YAML or TOML comment `# <ID>: …` declares no ID, opens no section, ends no lead, bounds no unit and is no near miss ([§FS-declarations.requirements.mentions](FS-declarations.md#requirementsmentions-writing-about-an-id-never-declares-it--realized-one-case-deferred)). Inside the block a fence opener and a raw-text HTML opener ([§FS-check.1.1.5.1](FS-check.md#1151-a-raw-html-block-holds-no-heading)) are block content and open nothing; the block is decided first, from line 1, so neither of those contexts can contain it. Its citations stay live, as in a raw-text HTML block ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)): each resolves, counts as a use, and is reported where it dangles, at its own line and column. `grund fmt` rewrites nothing in the block ([§FS-citations.zones](FS-citations.md#zones-where-citation-text-is-never-rewritten)). A `grund:fmt off` or `grund:fmt on` directive line inside it still toggles, because a directive is inert exactly where a citation is, and a front-matter citation is live ([§FS-fmt.2.5.2](FS-fmt.md#252-grundfmt-off--grundfmt-on--a-region-at-a-time)).

The rule applies to the files the scan reads as Markdown, a file a `[schema]` source maps to Markdown included, and to no other: a source file whose first line is `---` reads as it does without it. No YAML or TOML is parsed and no value is read, except in one case. Where the file's first declaration is of a kind that declares a `front-matter` field, its block is read as that field, a `---` block as YAML and a `+++` block as TOML, and its top-level keys as the field's sections ([§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field)). `show`, `--toc` and the section locks of the declaration are unchanged either way, because a declaration's body starts at its heading and front matter sits above every heading. Every surface that asks whether a Markdown line is a heading or may be rewritten reads the block by this one rule, as it reads the fence and the raw-text HTML block. Decided in [§DF-front-matter](../decisions/functional/DF-front-matter.md#df-front-matter-a-markdown-files-front-matter-is-a-region-of-its-own-owned-by-the-files-first-declaration-and-never-rewritten).

###### 1.1.5.2.1 Its citations belong to the file's first declaration

A citation in front matter, and every line of the block, delimiters included, is owned by the file's **first declaration**: the one whose body starts earliest in the file, whichever heading precedes it. Its section is none, as for a site in that declaration's lead, so `enclosing_section` is `null`. The exception is a declaration whose kind declares a `front-matter` field. There a block line's section is `front-matter.<key>` inside a key's slice, and `front-matter` elsewhere in the block ([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)). That owner is the one every ownership reading takes: the enclosing declaration of `refs` and `cover` ([§FS-cover.3.2](FS-cover.md#32---format-json), [§FS-cover.6.2](FS-cover.md#62-ownership-rules)), the body a `refs --from`, a rule or an obligation reads ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), the citing kind ([§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as)), and the owner a local shorthand expands against ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)). The declaration's body does not grow to cover the block: it still starts at its heading ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)). The blank lines between the closing delimiter and the first heading stay unowned. In a file that declares nothing, a front-matter citation has no owner, and falls through to the file's home as any ownerless site does.

#### 1.1.6 Value bindings

How the token of a value binding is read is [§FS-citations.recognized.value-bindings](FS-citations.md#recognizedvalue-bindings-value-bindings).

#### 1.1.7 Fetched snapshots

How a fetched snapshot is read is [§FS-citations.recognized.snapshots](FS-citations.md#recognizedsnapshots-fetched-snapshots).

#### 1.1.8 Declaration-local numeric section candidates

How a marker followed by a bare section number is read is [§FS-citations.recognized.local-sections](FS-citations.md#recognizedlocal-sections-declaration-local-numeric-section-candidates).

#### 1.1.9 An ID in an escape position

How an ID in an escape position is read is [§FS-citations.recognized.escapes](FS-citations.md#recognizedescapes-an-id-in-an-escape-position).

#### 1.1.10 The configured marker establishes the citation start

Where a marked citation starts is [§FS-citations.recognized.marker-start](FS-citations.md#recognizedmarker-start-the-configured-marker-establishes-the-citation-start).

#### 1.1.11 A glob operator never shortens a marked citation

How a glob operator inside a marked token is read is [§FS-citations.recognized.globs](FS-citations.md#recognizedglobs-a-glob-operator-never-shortens-a-marked-citation).

#### 1.1.12 A marked token its kind's format does not spell

How a marked token of a configured kind that its kind's format does not spell, and that nothing declares, is read is [§FS-citations.recognized.off-format](FS-citations.md#recognizedoff-format-a-marked-token-its-kinds-format-does-not-spell-is-reported-never-dropped).

### 1.2 The number-only shorthand

What the number-only shorthand is, and what bounds its recognition, is [§FS-citations.shorthand](FS-citations.md#shorthand-the-number-only-shorthand).

#### 1.2.1 The shorthand shape

The shorthand's shape is [§FS-citations.shorthand.shape](FS-citations.md#shorthandshape-the-shorthand-shape).

#### 1.2.2 The marker is required

That a shorthand needs its marker is [§FS-citations.shorthand.marker](FS-citations.md#shorthandmarker-the-marker-is-required).

#### 1.2.3 The full ID always wins

That the full ID claims a token before the shorthand does is [§FS-citations.shorthand.full-id-wins](FS-citations.md#shorthandfull-id-wins-the-full-id-always-wins).

#### 1.2.4 A resolved shorthand is a real edge

That a resolved shorthand is an edge like any other is [§FS-citations.shorthand.edge](FS-citations.md#shorthandedge-a-resolved-shorthand-is-a-real-edge).

#### 1.2.5 A shorthand in a numeric run

How a shorthand glued to a second number is read is [§FS-citations.shorthand.numeric-run](FS-citations.md#shorthandnumeric-run-a-shorthand-in-a-numeric-run).

#### 1.2.6 The shorthand as a CLI ID argument

That the same shape is accepted as a CLI ID argument is [§FS-citations.shorthand.cli](FS-citations.md#shorthandcli-the-shorthand-as-a-cli-id-argument).

### 1.3 The full-tree scope (`--full`)

`[scan] include` and every scanned kind home ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)) are the roots the scan starts from, so a citation in a file outside them is not merely unchecked — it is invisible: it does not resolve, does not dangle, and appears in no report, so dangling IDs accumulate there unnoticed. A clean `check` then means "clean *within* the scope" but reads as "clean" — a false negative in the one command the workflow trusts, invisible by construction, because the citations that most need checking are the ones somebody forgot to bring into scope. [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) calls that class a bug, so it must be *seen* without first editing the config to guess where to look; [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) accepts the bounded blind spot only because this flag exists to look into it.

`grund check --full` is that way. It cancels `[scan] include` for the scan, and with it the `scan = false` prune of an unscanned kind home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) — and nothing else. Decided in [§DF-check-full-scope](../decisions/functional/DF-check-full-scope.md#df-check-full-scope-check---full-walks-past-scan-include-and-reports-unresolved-references-plus-orphaned-section-headings-out-there).

#### 1.3.1 The scan covers the whole config root

`[scan] exclude`, `.gitignore` and every other ignore file, hidden names — directory and file alike ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)) — workspace member boundaries ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), the E2E case-directory boundary wherever a citable `E2E` kind is configured ([§FS-declarations.e2e-case.walk](FS-declarations.md#e2e-casewalk-the-walk-stops-at-a-case-directory)), and `[scan] extensions` all still apply exactly as they do without the flag. A file type `grund` does not scan stays unscanned; widening `extensions` is a config decision, and one flag that widened both would make "what did this run read" unanswerable. A boundary is a *declared* member, not any nested `grund.toml`: a project directory the workspace never declared is ordinary tree to both scans, so `--full` reads it and judges its citations under *this* project's grammar and kinds. That is what the plain scan does with it too, but the flag is what makes it reachable by default — a vendored, generated, or example project belongs in `[scan] exclude`, or in `[workspace] members` if it is one of ours, before a run adds `--full`.

#### 1.3.2 The wider scan reads a superset, each file once

Every root the plain scan starts from — each `[scan] include` entry and every kind home it scans ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)) — is scanned under `--full` too, whether or not `exclude`, an ignore file, or the hidden-directory rule would otherwise prune it: those three rules prune *descendants*, never the directory a scan starts at, so a gitignored, excluded, or hidden root is read by the plain run and must be read here. Without that, `--full` could read *fewer* files than `grund check` and hide a finding instead of adding one. A hidden **file**, and a file whose extension the extension list leaves out, are the exceptions, read by neither run even as a root ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned), [§FS-scan.scope.extensions.root](FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so)); additivity survives them because the file is missing from both scans, not from one, and both runs name such a root the same way ([§FS-scan.checks.unread-scan-root.scope](FS-scan.md#checksunread-scan-rootscope-asked-of-the-configured-scope-by-check-alone)). Overlapping roots — an `include` entry inside another, or inside the config root the flag adds — name one file once; a file read twice would be a declaration duplicated with itself ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)). "Once" is per *file*, not per path: an `include` root that is a symlink to a directory inside the config root, or a case alias of one on a case-insensitive filesystem, reaches its files under a spelling the config-root scan never produces, so a byte-identical compare cannot see the reread. The scan therefore starts at those roots *before* the config root and keeps the first spelling of each file — the one `grund check` prints without the flag. Every in-scope line is the plain run's, character for character, which holds for a file with two names because a line names only the spellings the plain run reaches ([§FS-scan.identity.findings.content](FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name)): with `include = ["notes"]` the plain run never meets `docs/fs/a.md`, so both runs print `notes/link.md:5: error: unknown reference FS-gone`, and a place finding at a name only `--full` reaches is a new out-of-scope line; outside citation errors add the `outside [scan] include:` scope prefix, while [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)'s scanner invariant keeps its ordinary code and message.

#### 1.3.3 Two scopes, two rule sets

Inside the default scope, the report is the ordinary one. Outside it, [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)'s citation-resolution errors and its declaration-local form verdict ([§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)) are reported, alongside [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)'s single declaring-side exception. No style, grounding, placement, direction, duplicate or unused rule is applied there, and neither is a verdict about what the formatter rewrote — [§FS-check.3.14.2](FS-check.md#3142-what-is-judged-outside-the-configured-scope) states the test each of those is held to, and the one subtraction it makes from that test. A `--full` that failed on conventions in directories that never opted into them would be run once and never again.

#### 1.3.4 Purely additive

The findings inside the default scope are exactly the ones `grund check` reports on the same tree, so `--full` can only ever turn a green run red, never the reverse. It is the ordinary check plus unresolved citations and section-like headings that violate declaration-body ownership in the wider scan.

#### 1.3.5 The unused-declaration warning is unchanged out there

A declaration inside `include` cited *only* from outside it keeps its `declared but never cited` warning ([§FS-check.4.1](FS-check.md#41-unused-declaration)) under `--full`, and the citation that would have retired it resolves and is reported by nothing. That is what additivity costs, and it is the right side of the trade: counting the wider scan's citations toward [§FS-check.4.1](FS-check.md#41-unused-declaration) would *remove* an in-scope finding, the one direction this flag must never move, and it would make the warning mean something different depending on a flag. The remedy is the one the finding already names — widen `include` so the citing file is governed, and the edge counts everywhere.

#### 1.3.6 An explicit path still narrows

`--full` cancels `include`, never a path the caller typed: `grund check <path> --full` reports exactly `<path>` ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)), which `include` does not bound, and has no out-of-scope findings. A path that *resolves to the config root* is the root scope and does widen — `grund check .`, `grund check ./`, and `grund check <abs-root>` are the bare `grund check --full`, out-of-scope findings included. Any other path leaves the flag nothing to cancel. What the path narrows is the report and not the resolution ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)), so the finding set shrinks and gains no tier.

##### 1.3.6.1 A path scope narrows the report, not the resolution

What a path scope reads and what it reports is [§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports); decided in [§DF-path-scope-resolves-project-wide](../decisions/functional/DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path). That narrowed and whole runs agree is [§REQ-one-answer.scope.narrowed](../requirements/REQ-one-answer.md#scopenarrowed-a-narrowed-run-answers-as-the-whole-run-does), across a workspace too: a path below a workspace root loads the projects a run from it would see and resolves each completely ([§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see)), and only a member directory, which means a member run, still refuses a qualified citation the root run resolves ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)) — stricter, never more lenient. So outside every member the root run's `unknown project alias` verdict is this run's too ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)). Within a project three verdicts move, and each is the whole-project run's own answer arriving where it was not asked before:

- A citation whose declaration lives outside the path no longer dangles — and the `ungrounded source file` error derived from that verdict clears with it ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)).
- A declaration cited only from outside the path is cited ([§FS-check.4.1.4](FS-check.md#414-a-citation-anywhere-in-the-resolution-scope-counts)).
- A duplicate declaration whose twin lies outside the path is reported, so a path-scoped run that was silent exits 1 ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)). That is the one direction this rule adds a finding.

Two classes of diagnostic are not about a scanned file and keep exactly the behaviour they had. A run-level finding carries no path — the config findings, the scope cautions, the workspace run warnings — and the report filter never sees one. And the agent-entrypoint check ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) is a probe over the project root rather than a finding about a scanned file, and already runs when no source file is scanned at all; its finding is exempted by its code — the second class of path-anchored diagnostic the filter lets through, beside the multi-site findings of [§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them). It is exempt as *that* finding and not as a file: an ordinary citation, style or declaration finding about `AGENTS.md` is dropped like any other outside the path, because the entrypoint being in `[scan] include` is the ordinary configuration rather than a contrived one, and the report stays the path plus the sites [§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them) keeps with it. The scope cautions are computed against the **report scope**, so a path that recognized nothing still earns its empty-scan caution ([§FS-check.2.2](FS-check.md#22-empty-scan)) however much the wider walk read — and the [§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) caution about a file out there does not suppress it ([§FS-check.2.2.3](FS-check.md#223-exit-code-json-and-what-suppresses-it)). The nothing-recognized caution ([§FS-check.4.5](FS-check.md#45-nothing-recognized)) is a different matter: a narrowed run never earns it at all ([§FS-check.4.5.4](FS-check.md#454-asked-only-of-the-whole-project)).

##### 1.3.6.2 A finding that spans several sites is in scope at any of them

A finding that inherently spans several sites ([§FS-check.2.1](FS-check.md#21-report-format)) is one finding about all of them, so the report-scope test is its anchor **or** any site it names. That is the mechanism behind the duplicate above: the message is located at the first site ([§FS-errors.4.1.1](FS-errors.md#411-the-order-of-a-duplicates-sites)), which may be the twin the caller did not type, and the path-scoped run must still report it.

The diagnostic is kept whole — same anchor, same line, same bytes, under either scope — and is never re-anchored at the in-scope site. Re-anchoring would make `grund check <path>` print a line `grund check .` never prints, which is the disagreement this point exists to end; and the message already names every other site in its text, so a second path in the report surprises nobody. A content finding on a file with two names names each spelling as a site ([§FS-scan.identity.findings.content](FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name)), so `grund check notes/link.md` prints the whole run's line at `docs/fs/a.md:5`, where it printed one at `notes/link.md:5`. The refusal of a file two names read differently is the same ([§FS-scan.checks.reading-conflict](FS-scan.md#checksreading-conflict-two-names-of-one-file-read-it-differently)).

The rule is general rather than written for one code, and the other two multi-site findings are decided by it rather than left to be found:

- [§FS-values.5.2](FS-values.md#52-fixed-value-errors)'s value mismatch is anchored at the binding and names the declaration as its site, so `grund check <the declaring file>` reports a mismatch it was silent about before. That is the same unkept promise as the duplicate wearing a different code, not a second direction.
- [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)'s sites all lie in the anchor's own file ([§FS-declarations.checks.duplicate-section.1](FS-declarations.md#checksduplicate-section1-scoped-to-one-declaration)), so the rule leaves it exactly as it was.

##### 1.3.6.3 A file the wider walk could not read is said out loud

A file the resolution scope could not read is outside the report scope like any other fact about a file out there: it is none of this run's errors and does not reach its exit code, which stays `1` for a report of errors and is [§FS-check.2.4](FS-check.md#24-an-incomplete-run)'s `2` only for a file at or under the path. A run whose *report* is the path is complete about the path. A failure is at or under the path when any name the walk met it by is, not only the one the whole run prints, as a finding is ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)): the names of a directory entry the walk reached through several paths ([§FS-check.2.4.2](FS-check.md#242-one-failure-one-line)) and of a file it could not read ([§FS-scan.identity.findings](FS-scan.md#identityfindings-a-finding-about-a-file-with-two-names-names-each-spelling-once)) alike. At a workspace root holding `real/s.md` and a directory link `aa -> real`, the walk keeps `aa/s.md` because it sorts first, and `grund check real` still exits `2`, so which name a link carries never turns a run over an unreadable path under its path green.

Silence about it is not allowed, though. The citation whose declaration lay in that file is reported as unresolved, and a false positive must stay legible as one ([§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms)). So each unread file earns one CLI-level caution on **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) carrying the read failure's own reason and saying where the file stands:

```
warning: <path>: <reason> — outside the report scope, so a citation declared there may be reported as unresolved
```

It covers every class of scan failure the walk records — a permission error, an undecodable file, a link the scan cannot resolve ([§FS-config.3.5.5](FS-config.md#355-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into)), a file that was read and does not parse — and like every warning it leaves the exit code alone and stands in place of the `success` line ([§FS-check.2.1.3](FS-check.md#213-the-success-line)).

It carries the code of the failure behind it ([§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix)), `io` or `unparsable-file`, and is unhideable with it: neither `--only` nor `--ignore` can turn it off ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore), [§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), and `--only` naming that code selects it beside the [§FS-check.2.4](FS-check.md#24-an-incomplete-run) failures that do reach an exit code. It is the one finding of either code that is unhideable without exiting `2`, and the consequence is stated rather than regretted: a tree holding one unreadable file in its resolution scope prints that line in place of `success` for every `grund check <path>` in it, with no selector able to restore the line. That is the honest answer — the run genuinely could not read part of what it resolved against, and the whole-tree run over the same tree exits `2`. Minting a code of its own to buy suppressibility would add a row to [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)'s closed selector vocabulary for an advisory line, which is a wider change than this point needs.

For the same reason the chapter rules ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)) are asked whether the **resolution** scope was read in full, not whether the report scope was: an absence fact must not be asserted over a read the run knows failed, and the narrowed list would make a run that could not read half its tree claim it had.

In a workspace a path inside a member resolves against that member and never a sibling: `grund check <member>/src/lib.rs` resolves against that member's `include` and homes, reports that one file, and crosses no member boundary ([§FS-workspace.5](FS-workspace.md#5-command-scope)), so its `unknown project alias` for a citation the workspace root resolves is a scoped answer, which a member run keeps ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)). A path below the root and outside every member resolves against every project the root run loads, so it refuses no alias the root resolves ([§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see)).

The cost is that a path-scoped run walks what the whole-project run walks. That is what the correct answer costs, and it is the number [§DF-path-scope-resolves-project-wide](../decisions/functional/DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path) records.

#### 1.3.7 A path the flag cannot widen earns a caution, not a refusal

With `--full` and an explicit non-root path, emit one CLI-level `warning:` on **stderr**, retaining the unflagged findings, streams and exit code. V1 says:

```
warning: --full has no effect with an explicit PATH — sim already resolves against [scan] include, and the report is the path either way; drop --full, or drop the path to check the whole project
```

For v2, describe the configured resolution scope ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)), without calling it `[scan] include`. The two-scope rule ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)) already resolves project-wide. Warn; neither ignore the flag nor refuse the run. Replace `success` on a clean run; JSON emits one stderr finding, leaving stdout empty ([§FS-check.2.1](FS-check.md#21-report-format)).

#### 1.3.8 Workspaces widen per project

Run at a workspace root, `--full` applies to the root project and to every member ([§FS-workspace.5](FS-workspace.md#5-command-scope)): each scans its own tree past its own `[scan] include` and judges its findings against its own default scope, because `include` is a per-project statement. It widens the projects a run already has and never invents one, so under `[workspace] include_root = false` ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)) a file at the workspace root outside every member is read by nothing, with or without the flag — there is no root project whose `include` there would be to cancel.

#### 1.3.9 An empty default scope is still reported

A `--full` run whose *default* scope read no files gets the [§FS-check.2.2](FS-check.md#22-empty-scan) caution as well as its out-of-scope findings: the findings say where the citations actually are, the caution says the config has not been told.

#### 1.3.10 A flag, never a key

`--full` is a flag, never a `grund.toml` key. A project that wants its whole tree governed widens `include` and gets the whole rule set; `--full` exists for the tree whose config has drifted from where the code moved, and a config key for it would be a second, weaker `include`.

### 1.4 Selecting findings with `--only` and `--ignore`

Both flags repeat: repeated values form a union, duplicate values are harmless, and each occurrence takes exactly one code — comma-separated values are not split into a selector language. An absent `--only` set retains every code. The two compose: a finding is retained when its code is in `--only` (or no `--only` was given) and is not in `--ignore`, so ignore wins when both sets name the same code.

A code a rule can produce carries `invalid-rule` with it. When `--only` names any of the rule-produced codes [§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits) lists, `invalid-rule` counts as selected too, and every `invalid-rule` finding passes the code axis — not only those of the rule that would have produced the selected code, because a rule that fails to parse has no family to match on. A rule that could not run is otherwise a check the narrowed run never evaluated, and its empty selection would print `success` and exit `0`, the same bytes as a real pass ([§DF-narrowed-check-keeps-invalid-rule](../decisions/functional/DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run)). The same selection carries every `empty-citation-obligation` row, the table's and the rules' alike, because an obligation with no unit is a check the narrowed run judged nothing with ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing), [§DF-narrowed-check-judged-nothing](../decisions/functional/DF-narrowed-check-judged-nothing.md#df-narrowed-check-judged-nothing-a-narrowed-check-says-when-what-it-selected-judged-nothing)). `agents-init` carries `invalid-rule` the same way in a run where an invalid rule left a project's blocks uncompared ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)). Nothing else about the code axis moves: `--ignore invalid-rule` and `--ignore empty-citation-obligation` still win, as an explicit opt-out, and the authority axis below still applies to the carried rows. A run whose `--only` set names no rule-produced code selects exactly what it names.

Codes are the documented lowercase kebab-case vocabulary in [§FS-errors.5](FS-errors.md#5-json-format), not categories or message fragments. `io` and `unparsable-file` are selectable and hide nothing: neither axis removes one ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)), so `--ignore unparsable-file` is accepted and the line is still printed, as `--ignore io` behaves. Selecting an opt-in code such as `oversized-lead` is valid even when its config key is absent, but selection never activates the finding ([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)); a selected code the run leaves off earns one warning that says so instead of `success` ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)).

Selection has a second axis, and the two intersect. `--only-rule` ([§FS-rules.8](FS-rules.md#8-command-surfaces)) selects by a finding's `authority` — the rules that produced it ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)) — rather than by its code, because an authority is not a code and no composition of the code vocabulary can name one: a trial sentence emits the same codes the declared rules emit, by construction. A finding is retained when its code passes the code axis *and*, if `--only-rule` was given, its `authority` contains the `--rule` origin. `--ignore` still wins over both, so `--only-rule --ignore <the trial sentence's own code>` legitimately empties a scoped report; adding a second exception to ignore's precedence would make selection unpredictable. The `--only` and `--ignore` value grammar is untouched by this axis: `--ignore RULE-count` remains the invalid-code refusal below, because those flags take codes and a rule identity is not one.

Selector values are validated before config discovery or scanning. Lowercase kebab-case means `[a-z0-9]+(?:-[a-z0-9]+)*`: no uppercase, underscore, comma, empty segment, or leading/trailing hyphen. A missing or empty value is rejected as `error: --only requires a finding code` or `error: --ignore requires a finding code`; a value outside that grammar is rejected as `error: invalid finding code "<value>" (expected lowercase kebab-case)`; and a well-formed value outside the public catalog is rejected as ``error: unknown check finding code "<value>"; run `grund check --help` for supported codes``. These are CLI failures: stdout is empty and the exit is `2` ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)). Validation is independent of flag spelling: both `--only value` / `--ignore value` and `--only=value` / `--ignore=value` have the same behavior. `--only-rule` is decided in the same place and refused the same way — `error: --only-rule requires --rule` when no trial sentence was given, empty stdout, exit `2` — and being a boolean it takes no value, so it has no `=value` spelling to validate and repeats harmlessly.

#### 1.4.1 A selected check the run leaves off says so

A code is **armed** in a run when the run's configuration and flags turn its check on, whether or not any unit then gives it something to find: each gate that decides whether a check runs records the codes it armed, so whether a check is on is the one fact the gate already decides. In a workspace a code is armed when any project of the run arms it. A code the [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) catalog enables with `—`, or by the very thing it judges, is always armed. A code its cell turns on more than one way is armed by any of them: the value codes by a kind's values slot, whole declarations or a declared chapter, and by a value tag the scan found, which needs no switch ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)), and `optional-member-absent` by an `optional_members` list in any `[workspace]` block the run expands, a nested one included. A cell that needs two things is armed only where both hold: `missing-snapshot` by a fetched kind, of the project or of a workspace target, whose resolution in that project's rules is `should` (`warn` under v2), because `fetch` alone resolves `must` and selects the `dangling` error ([§FS-config.3.4.10.3](FS-config.md#34103-resolve-selects-a-finding-class)); `empty-citation-obligation` by an obligation on a scanned kind with a `folder` home, the only kind it warns about ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)); and `ungrounded` by a place that grounding reaches, because a row's own `false` wins over `[reference] require_grounding` and `--require-grounding` alike ([§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global)), as a kind's own ladder replaces `[rules.citations.grounding]` under v2 ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). When `--only` selects a code that no project of the run armed, and `--ignore` does not name it, `check` writes one CLI-level warning for that code, in code order, with the code `selected-check-off`:

```text
warning: --only oversized-lead judged nothing — the check is off until [schema.leads.words] warn = <N> is set
warning: --only out-of-scope-dangling judged nothing — the check is off until --full is passed
warning: --only missing-field judged nothing — a version-1 config cannot turn the check on; migrate it with grund config migrate --to 2
warning: --only excluded-stub-target judged nothing — the check is spent from grund 0.19.0; select broken-stub instead
```

The advice names what the run lacks of the code's *Enabled by* cell, spelled in the config version the run read: `[reference] lead_size_warning = { max = <N>, unit = "words" }` under version 1. A switch the version read cannot hold, such as a field under version 1, names `grund config migrate --to 2` and never a key that file cannot carry. A workspace whose projects read different config versions names a key only where every version it read spells it alike, and otherwise names `grund config migrate --to 2` for its version-1 projects. The advice is the catalog's own words, read from the one rendered catalog `grund config explain` prints ([§FS-config-explain.4](FS-config-explain.md#4-a-codes-answer)): `the check is off until ` and the clauses of the code's *Enabled by* cell the run lacks, without their remarks and joined by ` and `, or for a spent code `the check is ` and its *Severity* cell after `none — `, each with its backticks dropped ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)). The words around them are the cases of `selected-check-off`'s own *Fix* cell — `off` before the clauses, `one-version` and `mixed-versions` after the migrate reason — so they too are worded once ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)). So the line and the answer cannot disagree, and no table of its own restates the catalog.

The line is about the selection, not about a finding, so it passes both selection axes, `--only-rule` included, and only `--ignore selected-check-off` removes it. It has no path, line, sites or authority, so with `--format=json` it is one warning object on stderr with all four `null` ([§FS-errors.5](FS-errors.md#5-json-format)). It is a retained warning: it holds back `success` ([§FS-check.2.1.3](FS-check.md#213-the-success-line)) and never moves the exit code. A code the run armed whose findings the channel holds back, such as a `should`-level field without `--suggestions`, did run, and earns no line ([§FS-rules.8](FS-rules.md#8-command-surfaces)). Why the narrowed run says this rather than failing is [§DF-narrowed-check-judged-nothing](../decisions/functional/DF-narrowed-check-judged-nothing.md#df-narrowed-check-judged-nothing-a-narrowed-check-says-when-what-it-selected-judged-nothing).

## 2. Outputs

`check` writes its findings to **stdout**, because a linter's findings are its output ([§FS-errors.1](FS-errors.md#1-streams)), writes what it says about the run to **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), and gives its verdict as the exit code:

- `0` — no retained error. Retained warnings and suggestions never move the exit code.
- `1` — at least one retained error.
- `2` — the run failed: an invalid invocation or `grund.toml`, or a path the scan could not read, list or parse ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)).

[§FS-output-shapes](FS-output-shapes.md#fs-output-shapes-machine-readable-output-shapes) shows the text and JSON reports.

### 2.1 Report format

Each retained finding is one line on stdout, `<path>:<line>: <channel>: <message>` ([§FS-errors.2.1](FS-errors.md#21-located-finding)), and its channel is `error:` for [§FS-check.3](FS-check.md#3-errors-detected), `warning:` for [§FS-check.4](FS-check.md#4-warnings) and `suggestion:` for [§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in) ([§FS-errors.2.1](FS-errors.md#21-located-finding)). Text groups the lines by channel and JSON keeps one global order ([§FS-errors.4.1](FS-errors.md#41-ordering)). A finding that spans several sites, such as a duplicate declaration ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)), is located at its first site in `path` then `line` order, a duplicate's sites in the order of [§FS-errors.4.1.1](FS-errors.md#411-the-order-of-a-duplicates-sites), and names the other sites in its message. Every retained finding is printed whole, and nothing summarizes them.

#### 2.1.1 CLI-level messages

A line about the run rather than about a site in the repository is a CLI-level message: on **stderr**, never stdout, with the shape, exit and JSON rules of [§FS-errors.2.2](FS-errors.md#22-cli-level-message). In `check` these are the invocation and config errors, the scan failures ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)), and run-level cautions such as the empty scan ([§FS-check.2.2](FS-check.md#22-empty-scan)), an obligation that applies to nothing ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)), a selected check left off ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)) and nothing recognized ([§FS-check.4.5](FS-check.md#45-nothing-recognized)). A `grund.toml` schema error ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), the absorbed-scan error of [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and the workspace warning of [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread) point at a config line inside their text ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)).

#### 2.1.2 Selection filters the complete report

`check` completes the scan and every checker pass before `--only`, `--ignore` and `--only-rule` apply ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)): selection is a query over the complete report by exact code and by rule authority, never a way to skip work. It runs after `--suggestions`, `--full` and `--require-grounding` have decided which findings exist, and before the sort, the render and the exit decision. A retained finding keeps its message, location, code, sites, authority, channel and relative order, and the order or repetition of the flags changes nothing ([§FS-errors.4](FS-errors.md#4-determinism)).

Some lines no selector removes. CLI and config failures are outside the report, and neither axis hides `io` or `unparsable-file`, so an incomplete run ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)) exits `2` whatever the selectors. `import-live`, `import-missing` ([§FS-imports.checks](FS-imports.md#checks-checks)), `reading-conflict` ([§FS-scan.checks.reading-conflict](FS-scan.md#checksreading-conflict-two-names-of-one-file-read-it-differently)) and the unread-source caution ([§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)) stay too, each for the reason its own section gives. Narrowing to a code a rule produces carries the `invalid-rule` and `empty-citation-obligation` findings with it ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), so a narrowed run never passes a rule it could not run or an obligation with nothing to judge.

#### 2.1.3 The `success` line

When the selected report retains no error, warning or suggestion and no unselectable run-level line, the default text form writes exactly `success` and a newline to stdout. It says that the selected report is empty, not that the repository has no findings. There is no summary footer: the exit code is the verdict, and the finding lines are the detail.

#### 2.1.4 JSON

With `--format=json`, the retained findings are NDJSON on stdout, one object each in the shape of [§FS-errors.5](FS-errors.md#5-json-format), in the global order of [§FS-errors.4.1](FS-errors.md#41-ordering) rather than text's channel groups. Stdout holds findings only: an empty selected report writes nothing, with no object in place of `success`, so `grund check --format=json | jq …` sees only findings. Run-level lines go to stderr ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), so a clean JSON run is empty on both streams and an exit `2` always has something on stderr.

### 2.2 Empty scan

A run that read no file in its default scope, or under the path it was given ([§FS-check.1.3.9](FS-check.md#139-an-empty-default-scope-is-still-reported)), is almost always a misconfigured scope rather than a clean repository, so in place of `success` `check` prints one CLI-level warning ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), code `empty-scan`, that names the likely cause ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)). A workspace asks it per project ([§FS-workspace.5.3](FS-workspace.md#53-the-empty-scan-caution-names-each-project-from-the-report-base)).

#### 2.2.1 Citation-direction obligation applies to nothing

`check` prints one CLI-level warning ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), code `empty-citation-obligation`, for each table or rule whose citation obligation has nothing to judge. A `[citations.<kind>]` table earns it when all of these hold:

1. The table has a non-empty `must` or `should` list; a table of `must-not` or `should-not` entries only has no obligation and never warns. A table with both levels is named by `must`.
2. The citing kind has a `folder` home, and the kind is scanned.
3. The run read at least one file in that folder besides its entry file and a generated scoped entrypoint ([§FS-init.2.3.5.15](FS-init.md#23515-a-generated-scoped-entrypoint-is-not-a-unit-of-its-place)). The entry is the effective `index` of a citable kind (`README.md` when the key is omitted) and the literal `README.md` of a non-citable one; `index = false` excludes no file.
4. The obligation found no unit to judge: no declaration of a citable kind, or no scanned file carrying a citation for a non-citable one.

A `must` or `should` rule, declared or given with `--rule`, whose subject is that kind earns it by conditions 2 to 4, decided by the same test as the table, so the two never disagree about whether a folder is empty; a `must not` or `should not` rule never earns it. A workspace asks per project ([§FS-workspace.5](FS-workspace.md#5-command-scope)), and an explicit path asks it of the files it scans, so `grund check skills` can earn it, and it does not broaden the path to unrelated homes.

```text
warning: [citations.SKILL] must applies to nothing — skills/ declares no SKILL ID; did you mean `citable = false`?
warning: [citations.skill] must applies to nothing — no scanned file in skills/ carries a citation; set require_grounding = true on the skills/ row to make that an error
warning: RULE-e2e applies to nothing — tests/e2e/ declares no e2e ID
```

The non-citable line gives its advice only where the row's effective `require_grounding` is off ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)), and otherwise ends at the fact. A rule's line names the rule by its ID, or as `--rule`, and ends at the fact; it carries the rule's authority ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)), so `--only-rule` keeps the trial sentence's line and drops the others ([§DF-narrowed-check-judged-nothing.2.1](../decisions/functional/DF-narrowed-check-judged-nothing.md#21-a-rule-joins-empty-citation-obligation)). No other finding suppresses the warning.

#### 2.2.2 The message names the likely cause

The `empty-scan` message names the first of these causes that holds:

1. A `[workspace]` block put no project in scope: `include_root = false`, and every member it has is an absent optional one ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)).
2. The path is a hidden file whose extension is read: the message names the hidden-name rule ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), since the extension list is the one rule that did not skip it. A hidden file of an unread extension, and a directory whose only files of a read extension are hidden, fall through to the causes below.
3. The run loaded no config ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), is scoped to its root, and a project config lies below that root: the message names the configs and two routes that read them as configured, a run from their directory and a root `[workspace]` that lists them with `include_root = false` ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)). It never offers the scaffold, which would lay a second project above the one declared.
4. Under version 1, the run is scoped to the root (no path, or `.`): the message names `[scan] include` and its list, with the scaffold, config and path advice, and a project that declares no kinds ([§FS-config.3.4.4.5](FS-config.md#3445-kinds---declares-no-kinds)) gets the `[[kinds]]` advice in place of the scaffold. Any v2 repair instead names its places ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)) or source exclusions ([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)), never `[scan] include` or `[[kinds]]`; a first kind it recommends is `[schema.kinds.<NAME>]`.
5. The path is a directory link out of the project root, which the walk does not follow ([§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)): the message names the link as the user spelled it and the way past it.
6. Under version 2, an `exclude` pattern pruned the first file the walk would otherwise read, on its way down from the path or, for a run given no path, from the project's roots ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)): the message names the pattern, a cause the run proved, ahead of any link it did not look past.
7. The walk refused a directory link out of the project root below the path, or below the project's root in a run scoped to it: the message names the first such link in path order ([§FS-errors.4.1](FS-errors.md#41-ordering)), spelled from the path as written, or from the report base for an absolute path or a run given no path ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), and says only that nothing under the path was read.
8. Under version 2, no listed language is read for declarations ([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)): the message names the key that admits one or, where a `[schema.sources.extensions]` entry maps a suffix a listed language would read to a `data` language, the first such suffix in byte order and the entry to drop ([§DF-reading-at-fetch-and-unread-homes.2.6](../decisions/functional/DF-reading-at-fetch-and-unread-homes.md#26-an-empty-scan-under-version-2-names-the-languages)).
9. Otherwise the message names the path and the extensions grund reads.

```text
warning: nothing to scan — this [workspace] block put no project in scope: include_root = false and every member it has is an absent optional one, each named in the report. Nothing is misconfigured; a fuller checkout is what changes it.
warning: nothing to scan — `<path>` is a hidden file. grund reads no file whose own name begins with `.`, whatever `[scan] extensions` says. Rename it, or move what needs checking into a file that is not hidden.
warning: nothing to scan — no grund.toml applies here, so grund used its defaults and found no files. `suite/grund.toml` declares a project this run does not read: run grund from `suite/` (or `grund check suite`), or add a grund.toml here whose [workspace] has members = ["suite"] and include_root = false.
warning: nothing to scan — grund looked under [scan] include = ["requirements.md", "docs", "e2e", "src"] and found no files. Run `grund init --scaffold` to scaffold the canonical docs/ and e2e/ trees, point `[scan] include` in `grund.toml` at your sources, or pass a path explicitly (`grund check <dir>`).
warning: nothing to scan — grund looked under [scan] include = ["requirements.md", "docs", "e2e", "src"] and found no files. This project declares no kinds yet: replace `kinds = []` in `grund.toml` with a `[[kinds]]` table for its first kind, point `[scan] include` at your sources, or pass a path explicitly (`grund check <dir>`).
warning: nothing to scan — `x` is a directory link out of the project root, and grund does not follow one (name a file below it, or move the target inside the project).
warning: nothing to scan — every file under `m` that grund would read is excluded by the [schema.sources] exclude pattern `gen`.
warning: nothing to scan — nothing under `real` was read: `real/subx` is a directory link out of the project root, and grund does not follow one (name a file below it, or move the target inside the project).
warning: nothing to scan — no language in [schema.sources] languages is read for declarations, so no file under `.` was read; admit one in [schema.sources] languages, such as `markdown`.
warning: nothing to scan — no language in [schema.sources] languages is read for declarations, so no file under `.` was read; [schema.sources.extensions] maps `rs` to `json`: drop that entry.
warning: nothing to scan — no files under `<path>` matched grund's extensions (<list>).
```

For cause 3, a project config is the file discovery would load in its directory ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)), so `suite/.agents/grund.toml` is named as itself, and a route names its config root; both are spelled from the report base ([§FS-workspace.5.3](FS-workspace.md#53-the-empty-scan-caution-names-each-project-from-the-report-base)). Only a run that loaded no config searches, at most three levels below its root, only into directories the scan would enter ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent), [§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)), and never below a config it found. Several configs are named in path order, at most three and then a count of the rest, as in `` `a/grund.toml`, `b/grund.toml`, `c/grund.toml` and 2 more declare projects ``; the run route then reads `` run grund from one of their directories (or `grund check a`) ``, and the workspace route spells `members = [...]` only when it named every config, and otherwise asks for a `[workspace]` that `lists their directories in members and has include_root = false`.

#### 2.2.3 Exit code, JSON, and what suppresses it

`empty-scan` is a warning, because a genuinely empty tree is not a failure, so it never moves the exit code; its JSON form is [§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)'s. Any finding about the default scope replaces it, the agent-entrypoint check of [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block) among them, which runs even when nothing is scanned. Four findings are about something other than the scope the run scanned, and the caution stands beside them: the redundant config pair ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)), an out-of-scope citation ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only), [§FS-check.1.3.9](FS-check.md#139-an-empty-default-scope-is-still-reported)), an absent optional member ([§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)) and the unread-source caution ([§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)).

### 2.3 Suggestions channel *(opt-in)*

Suggestions are findings on a third channel, not at a third severity, because [§FS-config.6](FS-config.md#6-what-is-not-configured-here) keeps the severities at `{error, warning}` ([§FS-terms.terms.5](FS-terms.md#terms5-findings), [§DF-citation-directions.2.3](../decisions/functional/DF-citation-directions.md#23-suggestions-are-a-third-channel-not-a-third-severity)). The `should` and `should-not` levels of `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), the `should` and `should-not` strengths of version 2 ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)) and the `should` and `should not` results of chapter rules ([§FS-rules.7](FS-rules.md#7-findings-and-channels)) produce them. A `should` citation a declaration lacks is `suggested-citation` and a `should-not` citation is `discouraged-citation`, whether a table or a rule asks for it ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)); an escape that resolves is `escaped-citation-resolves` ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)); and every other suggestion keeps the code it has at any strength ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities), [§FS-rules.7](FS-rules.md#7-findings-and-channels)).

#### 2.3.1 Escaped citation resolves

An escape, `<§>ID` or `<§>alias/ID`, is not a citation ([§FS-check.1.1.9](FS-check.md#119-an-id-in-an-escape-position)), which is how prose shows a citation's shape. `check` reports `escaped-citation-resolves` at an escape whose coordinate resolves, because it may be a live citation escaped by mistake ([§DF-escape-position-is-not-a-citation.2.4](../decisions/functional/DF-escape-position-is-not-a-citation.md#24-one-rule-read-by-the-reader-and-the-writer)). The coordinate resolves when its ID names a declaration and any section it writes is found there as the missing-section check finds one ([§FS-check.3.2](FS-check.md#32-missing-section)), a stub's sections being its target's ([§FS-check.3.2.1](FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not)); an escape of a section that does not exist earns nothing, as an escape of an undeclared ID does. The message names the live form of the same coordinate. It keeps the section, since the declaration alone would widen a citation of one section to the whole declaration. The escape is read with the citing project's grammar, and the live form is written with the separator of the project the coordinate names ([§FS-workspace.1.2](FS-workspace.md#12-whose-grammar-reads-the-id)), so that it reads back as the coordinate the escape named; here member `api` separates sections with `:`:

```text
docs/guide.md:5: suggestion: escaped citation <§>api/FS-x.2 resolves to a declaration; write §api/FS-x:2 for a live citation, or leave it escaped if it is only an illustration
```

#### 2.3.2 Selecting and printing suggestions

Suggestions are withheld unless `--suggestions` asks for them ([§DF-citation-directions.2.2](../decisions/functional/DF-citation-directions.md#22-errors-gate-suggestions-report--a-fixed-split-not-a-knob)). With the flag, each is reported as [§FS-check.2.1](FS-check.md#21-report-format) reports every finding, and under `--format=json` it is an object on stdout with `"channel": "suggestion"` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)). Without it none is printed in either form, so a run whose only findings are suggestions prints `success` ([§FS-check.2.1.3](FS-check.md#213-the-success-line)). `--only` and `--ignore` select among suggestions only once the flag has enabled them, so a selector never surfaces one the run did not request.

### 2.4 An incomplete run

An invalid `grund.toml` aborts the run before any file is read ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). A failure during the scan does not: a file that cannot be read or decoded, a directory that cannot be listed, a link the scan cannot resolve ([§FS-config.3.5.5](FS-config.md#355-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into)), or a file that was read and does not parse is one `error: <path>: <reason>; <fix>` line on stderr ([§FS-errors.2.2.5](FS-errors.md#225-one-scan-failure-one-rendering)), the scan goes on over the rest, every finding from the readable files is printed as usual, and the run exits `2`. So `2` means the report is not complete, but the findings it printed are real. Malformed input is answered with a line naming its path and a truthful code, never an abort ([§REQ-never-crashes](../requirements/REQ-never-crashes.md#req-never-crashes-garbage-in-diagnostic-out)).

`check` calls no ID absent from a home it failed to read: a citation of a kind with `fetch` whose ID, or shorthand, matches nothing in the target's catalog earns no finding while one of the run's own `error:` lines is a failure at or under that kind's one home in the project the citation targets.

#### 2.4.1 One reason for one scan I/O failure

The reason for a scan I/O failure is its I/O error kind, without an OS error number, and one kind reads the same for a file, a directory, a value source and a value home: a denied permission, for example, reads `permission denied`. A value read keeps its context around the reason, as in `could not read value JSON source: permission denied` or `read value home: permission denied`. A reason that is not I/O, such as a parse failure or a symlink loop, keeps its own description, and callers that expose causes get the raw error, its OS error number included. Config-load messages are outside this rule ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)).

#### 2.4.2 One failure, one line

A path the scan failed on for one reason is one line from each command that reports scan failures, one JSON record and one record in the embedding API, however many walks and passes met it ([§DF-e2e-case-on-the-one-walk.2.3](../decisions/functional/DF-e2e-case-on-the-one-walk.md#23-once)). Two reports are one failure when they have the same reason before any context ([§FS-check.2.4.1](FS-check.md#241-one-reason-for-one-scan-io-failure)) and the same path: a file under any of its spellings ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once)), and, for two reports the walk makes, one directory entry, its parent resolved and its own name not. A link and its target are therefore two failures ([§FS-scan.scope.links.unresolvable.unopenable](FS-scan.md#scopelinksunresolvableunopenable-a-directory-link-whose-target-cannot-be-opened-fails-at-the-link-once)), and two different failures stay two lines even where they read alike ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)).

The line is at the path that failed, never at a walk root, configured folder or case directory standing in for it ([§DF-e2e-case-on-the-one-walk.2.2](../decisions/functional/DF-e2e-case-on-the-one-walk.md#22-at-the-path-that-failed)), spelled from the report base, and its reason holds no absolute path ([§FS-errors.4](FS-errors.md#4-determinism)). A file met under several spellings is named by the one the run's own path reached it under, else by the one listed first ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once)). `grund check <path>` names a failure the walk reports by its name under the path given, in a plain project and at a workspace root alike, and every other surface names it by the whole run's spelling. Under a path scope the failure is either the caution of [§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud), outside the report scope, or the `error:` line, inside it, never both. Where the directory a symlink loop reaches back into lies under the config root, the loop's reason names it by its one physical location, spelled from the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so every route to the link has one reason; where that location is the report base itself, the reason is `symlink loop: the target contains the link`.

#### 2.4.3 What failed and where decide the code and the fix

A scan failure's code says what happened and its *Fix* case in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) says what can work at its path ([§DF-scan-failure-code-and-fix](../decisions/functional/DF-scan-failure-code-and-fix.md#df-scan-failure-code-and-fix-a-scan-failure-carries-its-own-code-and-a-fix-that-works-at-its-path)). A file that was read and does not parse is `unparsable-file`, and its case is the kind of file:

| Case | The file |
|---|---|
| `exit` | a case's `expected.exit` that is not one integer |
| `value` | a JSON or YAML value source the parser rejects ([§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output)) |
| `ignore` | an ignore file with a line that is not a valid pattern |

Everything else that could not be read or decoded, malformed UTF-8 among it, is `io`, and its case is the place, the first of these that holds, because `exclude` cannot keep out a path it does not reach:

| Case | The path |
|---|---|
| `passed` | a path given on the command line that a walk reads, since passing it is what made the run read it. Not the config root, which `check` treats as no narrowing; not a path given at a workspace root, which narrows only the report and takes the case the walk gives it; and not a value source, a section lock, or a case's file or directory no walk read, which its pass reads anyway and which takes the case of what it is |
| `readable` | the config root, or a section lock, which no walk reaches and no row lists |
| `value` | a value source ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)) |
| `root` | a scan root: a kind's folder or file, nested in another root or not, a value home, or a version-1 `[scan] include` entry ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)) |
| `entry` | anything else, which `exclude` reaches |

A link is judged as the path it is: at a scan root it is `root`, below one `entry`. A member's scan root is judged in its own project, so the row its fix names is in that member's config ([§FS-workspace.5](FS-workspace.md#5-command-scope)). The same code and fix reach every surface that publishes the failure ([§FS-errors.2.2.5](FS-errors.md#225-one-scan-failure-one-rendering)), the caution of [§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) among them.

## 3. Errors detected

Each of the following is an error and contributes to a non-zero exit code.

### 3.1 Dangling citation

A recognized citation (per [§FS-check.1.1](FS-check.md#11-recognized-citations)) for which no declaration is found: `unknown reference <ID>`. A declared member candidate ([§FS-check.3.1.5](FS-check.md#315-an-unqualified-id-declared-in-a-loaded-member)), a near ID ([§FS-check.3.1.1](FS-check.md#311-a-near-id)) or an inline-code context ([§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code)) adds a hint, a fetch-enabled kind adds the fetch action ([§FS-check.3.1.3](FS-check.md#313-a-kind-that-fetches)), and a kind none of whose places exists names them ([§FS-check.3.1.4](FS-check.md#314-a-kind-none-of-whose-places-exists)).

Resolution is computed over the run's **resolution scope**, which for a run given an explicit path is the whole enclosing project rather than the path ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)), so a path-scoped run never calls a project-resolvable ID unknown. The message and the code are the same under either scope.

A number-only shorthand citation ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) is exempt from this rule and reported by [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) instead — never both, because `unknown reference FS-042` would name a token that is not a full ID under the repo's own grammar. For a value binding, this ordinary resolution finding suppresses value comparison at the same site ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)).

#### 3.1.1 A near ID

When no declared member candidate ([§FS-check.3.1.5](FS-check.md#315-an-unqualified-id-declared-in-a-loaded-member)) applies, if the target project contains a declared ID of the same kind that is close by deterministic edit distance, the finding appends one hint: `unknown reference FS-chek; did you mean FS-check?`. If no same-kind candidate is close enough, this rule adds no hint, so unrelated missing IDs do not produce noisy guesses.

#### 3.1.2 An illustration in inline code

When the dangling citation sits inside a Markdown inline-code span ([§FS-fmt.2.3.5](FS-fmt.md#235-an-inline-code-span-closes-on-a-run-of-its-own-length)) — where a `§`-citation is as often an illustration as a live one — the finding also offers the `<§>` escape ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)): `unknown reference api/FS-zzz; write <§>api/FS-zzz if this is an illustration`. The selected suggestion, a declared member candidate ([§FS-check.3.1.5](FS-check.md#315-an-unqualified-id-declared-in-a-loaded-member)) or a local near-ID match, combines with an inline-code context once, using the citing project's configured marker for the escape: `unknown reference api/FS-login; did you mean api/FS-logout? (or write <§>api/FS-login if this is an illustration)`. Outside inline code the escape hint is withheld, so an ordinary prose typo is nudged toward the near ID, not toward escaping.

#### 3.1.3 A kind that fetches

This is also the fixed finding for a fetch-enabled kind whose effective target-side resolution is `must` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)). In that case the exact message is `unknown reference <qualified-ID>; no snapshot in <home> — run grund fetch <qualified-ID>`, or another tail at a home `fetch` refuses ([§FS-check.3.1.3.1](FS-check.md#3131-a-home-fetch-refuses)) or for a key `fetch` has nothing to run for ([§FS-check.3.1.3.2](FS-check.md#3132-a-key-fetch-has-nothing-to-run-for)), or `run grund fetch <ID below the last import edge> in <target of that edge>` for an ID reached through an import, nested or not ([§FS-imports.writes.answer](FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer)), its JSON code remains `dangling`, its severity is `error`, and it contributes exit 1. For a kind without `fetch`, the `unknown reference <qualified-ID>` base remains unchanged, with only the applicable hints added. A site whose home this run could not read earns neither this message nor the warning of [§FS-check.4.12](FS-check.md#412-missing-snapshot), as [§FS-check.2.4](FS-check.md#24-an-incomplete-run) says.

The declared-member, near-ID and escaped-inline-code hints ([§FS-check.3.1.5](FS-check.md#315-an-unqualified-id-declared-in-a-loaded-member)) ([§FS-check.3.1.1](FS-check.md#311-a-near-id), [§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code)) take precedence over the fetch action. The message retains `unknown reference <qualified-ID>; no snapshot in <home>` and substitutes the existing conditional tail for the em-dash fetch tail: `; did you mean <candidate>?`, `; write <§><qualified-ID> if this is an illustration`, or their existing combined form. One citation site still produces one finding.

##### 3.1.3.1 A home `fetch` refuses

`fetch` refuses a `file` home that is a value source, in either format ([§FS-fetch.4](FS-fetch.md#4-file-home-write)), and one the text scan reads as source ([§FS-fetch.4.2](FS-fetch.md#42-a-home-read-as-source-is-refused)), so `check` does not send a reader to that command. The em-dash tail, in the `dangling` message of [§FS-check.3.1.3](FS-check.md#313-a-kind-that-fetches) and the `missing-snapshot` message of [§FS-check.4.12](FS-check.md#412-missing-snapshot) alike, is `— declare it in that value source` where the target kind is a whole-value kind whose `file` home is a value source ([§FS-values.terms](FS-values.md#terms-terms)); `— declare it in that source file` where a non-value kind's `file` home is read as source, since a comment there is what the project reads as a declaration; and `— move the kind to a Markdown file or folder` where a value kind's is, since no comment is a valid value declaration. `fetch` also refuses a home the text scan does not read ([[§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)]), and there the tail is `— give the file a name the scan reads as Markdown`, for a `file` home such as `fs.dat` or a hidden `.fs.py`, whatever the kind. The base of the message, the code, the severity and the exit stay what they are, and a declared-member, near-ID or inline-code hint still takes the tail's place. The kind and its home are asked of the project the citation targets, not of the citing one. A folder home's `<ID>.md` that an ignore file or an `exclude` rule keeps out of the walk, which `fetch` refuses by the same predicate, takes `— the scan does not read <path>: <reason>; <repair>`, where `<path>` is that file from the report base and `<reason>` and `<repair>` are the rule and its repair as [§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused) words them without `, and fetch again`, such as `— the scan does not read docs/t/T-7.md: docs/t/.ignore ignores it by T-7.md; stop ignoring it in docs/t/.ignore`. A value kind whose `file` home is Markdown and any other folder home keep `— run grund fetch <qualified-ID>`, because `fetch` does not refuse them by their path; a non-value kind's `.yaml` `file` home the text scan does not read takes the unread tail, and one `[scan] extensions` lists is read as source, and takes that reading's tail.

##### 3.1.3.2 A key `fetch` has nothing to run for

Where `grund fetch` would have nothing to run for the kind ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)), `check` does not send a reader to that command either. The question is the one `grund fetch` asks, of the project the ID is in: the kind's `fetch` value names no executable, and no file of that very name stands where that project would run it. The em-dash tail, in the `dangling` message of [§FS-check.3.1.3](FS-check.md#313-a-kind-that-fetches) and the `missing-snapshot` message of [§FS-check.4.12](FS-check.md#412-missing-snapshot) alike, is then the key's name and the one repair ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)), in the spelling of that project's config version:

```text
— [[kinds]] `AR` fetch names no executable; name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors
— `[schema.kinds.AR] fetch` names no executable; name the program, or if this kind is not fetched remove the key and any `[rules.resolution] AR`, and its missing IDs are then errors
```

The first is a version 1 row. The second is the spelling a version 2 row took while such a key loaded; from grund 0.23.0 version 2 refuses the key when the config loads, so only version 1 reaches this tail ([§FS-config-v2.checks.config-fetch-names-nothing](FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)). The base of the message, the code, the severity and the exit stay what they are. A declared-member, near-ID or inline-code hint still takes the tail's place, and a home `fetch` refuses keeps the tail of [§FS-check.3.1.3.1](FS-check.md#3131-a-home-fetch-refuses), because that refusal comes first at `grund fetch` as well. For an ID reached through an import, the key, the root and the version are the target's and never the citing project's ([§FS-imports.writes.answer](FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer)). A whitespace-only value that names a file at that root, one literally named `" "`, is still run by `grund fetch`, so it keeps `— run grund fetch <qualified-ID>`, and so do a named program and a nonblank path nothing stands at, which `grund fetch` tries and fails to spawn. Answering looks up that one path and runs nothing ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

#### 3.1.4 A kind none of whose places exists

A v2 place is a path from the config root, and one that does not exist is quiet ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places), [§FS-config-v2.schema.places.boundaries.4](FS-config-v2.md#schemaplacesboundaries4-a-missing-place-is-quiet)). So a place written as a pattern, `files = ["specs/*/spec.md"]` or `folders = ["specs/*"]`, loads as a literal path that names nothing, its files go unread, and the only findings are dangling citations at the citers. The fix is one line of `grund.toml`, so the dangling finding names that line.

The hint applies when the dangling ID's kind is a row of the v2 config of the project the ID resolves against, the row has at least one place, and none of its places exists on disk. Then the message appends one tail, naming every place of the row in the row's order ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)), each as its path normalized from the config root (a written `./specs/*` prints `specs/*`) and followed by the config file and the line of its entry, the config path written the way the run writes finding paths, so a workspace member's is spelled from the workspace root (`m/grund.toml:4`):

```text
unknown reference SPEC-upload; no place of kind SPEC exists: docs/specs (grund.toml:4)
```

Several places are joined with `, `: `no place of kind SPEC exists: docs/a.md (grund.toml:4), docs/specs (grund.toml:5)`. When any named place holds one of the glob characters `*`, `?`, `[`, `]`, `{`, `}` or `!`, the set [§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language) refuses in a suffix, the tail adds what was meant:

```text
unknown reference SPEC-upload; no place of kind SPEC exists: specs/*/spec.md (grund.toml:4); a place is a path from the config root, not a pattern, so list each file or name the folder
```

The condition is every place. A row with one existing place and one missing place earns no hint, because the existing place was read and the ID may be a plain typo; a row with no place at all is declared in doc-comments only and earns none either. A kind that is no row of the config, and a v1 config, keep the bare message.

The tail is the last resort among the hints. A declared member candidate ([§FS-check.3.1.5](FS-check.md#315-an-unqualified-id-declared-in-a-loaded-member)), a near ID ([§FS-check.3.1.1](FS-check.md#311-a-near-id)) or an inline-code context ([§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code)) takes precedence and the message is theirs alone, and a fetch-enabled kind keeps its fetch action ([§FS-check.3.1.3](FS-check.md#313-a-kind-that-fetches)), which already names the missing home. One citation site still produces one finding. The JSON code stays `dangling`, the severity `error`, and the exit code `1`; a missing place still earns no finding of its own.

#### 3.1.5 An unqualified ID declared in a loaded member

For an unqualified dangling citation in a workspace run, offer the qualified spellings backed by actual declarations in the other loaded projects, using the candidate-selection rules of [§FS-workspace.8.1.1](FS-workspace.md#811-an-unqualified-id-another-project-declares). The current project for this search is the citation's own project, including when a root run checks a member's citations. For example: `unknown reference AR-005-invariant-monitor; did you mean worldsim/AR-005-invariant-monitor?`.

These candidates take precedence over the local near-ID hint ([§FS-check.3.1.1](FS-check.md#311-a-near-id)), which remains the fallback when no member candidate applies. Render the selected alternatives once with the existing `near` remedy case ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)), or its `near-escape` case inside inline code ([§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code)), using the citing project's configured marker in the escape.

The selected suggestion, alone or combined with the escape, takes the same place as the local near-ID hint: it replaces the fetch action while preserving the `no snapshot in <home>` base ([§FS-check.3.1.3](FS-check.md#313-a-kind-that-fetches)), and suppresses the missing-place tail ([§FS-check.3.1.4](FS-check.md#314-a-kind-none-of-whose-places-exists)). Never append a second competing remedy.

This is guidance only. Resolution remains project-local ([§FS-workspace.4](FS-workspace.md#4-resolution)); the finding keeps its source path and line, `error` severity, `dangling` JSON code and exit `1`, with candidates inside `message` and no JSON shape change. Qualified unknowns and missing sections on locally declared IDs retain their existing behavior. Citation recognition is unchanged: a token rejected before dangling resolution does not gain this hint ([§FS-check.1.1](FS-check.md#11-recognized-citations)).

Use only the projects the run already loaded, without fetching or loading another project. Respect its existing project scope ([§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see)) and report versus resolution scope ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)): candidate lookup changes neither which citations are reported nor which project resolves them.

### 3.2 Missing section

A citation with a section suffix (`§FS-<user-login>.3.1` or, in an opted-in repository, `§FS-<user-login>.goals`) where the declaration exists but the requested section heading does not. A missing marker-prefixed named coordinate is never shortened to its declaration; it produces the ordinary `section not found` error. Every missing section, numeric or named, marked or not, names its fix in its message, and where the section lock knows where the cited text went it names that section too ([§FS-errors.3.11](FS-errors.md#311-the-missing-section-remedy)). A number-only shorthand carrying a missing named section produces both the existing shorthand finding and this finding: the persisted ID form and the requested target are independent facts ([AR-checker.2.12](../../crates/grund-core/src/checker/report.rs)).

An owned declaration-local numeric candidate ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)) follows the same rule. Its
[§FS-check.3.24](FS-check.md#324-declaration-local-section-citation) form
finding and this missing-section finding are independent, so a missing local path reports both.

For a value binding the explicit numeric component must resolve here before comparison; a missing component produces this finding alone, not a mismatch ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)).

#### 3.2.1 A stub's sections are its target's, scanned or not

A citation of a stub's ID names a section of the declaration the stub pairs with
([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so the section exists exactly where that declaration
records a heading at the requested path, read from the stub's target whether or not
`[scan] include` reaches it. The stub declares no sections of its own
([§FS-declarations.checks.duplicate-section.2](FS-declarations.md#checksduplicate-section2-scoped-to-that-declarations-body)), so its own record never answers; the target's
declaration is the one `grund <ID>.<path>` slices ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)), and `check` and `show`
answer from that one section set ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)). A section citation reads as it would were
the target scanned: stubs to one target are one home and so one section set
([§FS-declarations.stubs.pairing.2](FS-declarations.md#stubspairing2-stubs-to-one-target-stand-for-its-declarations-once)), and a `--full` run, whose report inside the default scope
is the plain run's ([§FS-check.1.3.4](FS-check.md#134-purely-additive)), resolves it as the plain run does.

That answer is one fact, and every reader of it in `check` takes it: this finding, the `cites`
fact a rule counts a resolved citation by ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)), the value a binding is compared with once
its component resolves ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)), the value authority that makes a malformed binding an invalid
attempt rather than prose ([§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts)) and that `fmt --cross-refs` protects from a rewrite
([§FS-values.8](FS-values.md#8-formatting-stability)), the gate that admits a bare index entry `fmt` can wrap ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)), and the
declaration-local rewrite that declines an absent section ([§FS-fmt.2.4.6](FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)) with the command clause
it withholds there ([§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)).

What else a target outside the include gives the stub — a section it declares twice, and its
declaration's citations and chapters — and what of it stays unjudged, is [§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged).
Decided in [§DF-stub-sections-from-unscanned-target](../decisions/functional/DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it).

### 3.3 Duplicate declaration

Moved to [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration). This address is kept so citations written before the move still resolve.

### 3.4 Broken inline-spec stub

Moved to [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub). This address is kept so citations written before the move still resolve.

### 3.5 Invalid agent entrypoint init block

If `<path>/AGENTS.md` exists, `check` verifies the versioned `grund init` block defined by [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints); it also verifies the companion agent entrypoints [§FS-check.3.5.1](FS-check.md#351-which-entrypoints-are-verified) names. A missing managed block when one is required, an older block version, a newer unsupported block version, or a block that differs anywhere from what `grund init` would write there ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check)) is an error in scaffolded-entrypoint mode; a legacy block is an older version and broken delimiters are a distinct error ([§FS-check.3.5.2](FS-check.md#352-legacy-and-malformed-blocks)). This lets CI catch repos whose managed agent entry points were initialized and later drifted or need to be refreshed with `grund init`: a green `check` means every managed block it verified is exactly what `grund init` would write, save the two workspace sections in the runs [§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout) names. Under scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) the check is no longer only about `AGENTS.md` at the root: each destination's `AGENTS.md` that is missing or carries no block is a missing block, one whose block differs from the render is a stale one, and an orphan the root block's list on disk names that still carries a block is a stale one whose section is `scoped guidance`, each reported at that file ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check)). Every variant is an error with code `agents-init`, with the reporting order, message text, and selector behavior of [§FS-check.3.5.3](FS-check.md#353-code-and-message).

#### 3.5.1 Which entrypoints are verified

Scoped entrypoints are expected only where the project already carries a managed block in a root entrypoint, so a repository that adopted only the config stays unpoliced, as below. Where it does, every destination of the live render is verified, with the standalone `CLAUDE.md` or `GEMINI.md` beside it, and every orphan the list on disk names ([§FS-init.2.3.5.13.1](FS-init.md#235131-the-list-on-disk-records-what-init-owns)); a companion symlink to the scoped `AGENTS.md` is covered by that file, as at the root.

`check` verifies known companion agent entrypoints whenever they exist and are not symlinks to `AGENTS.md` — `.rules` only where the owning `.zed/` directory or a managed block already in it says so, the evidence [§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent) attributes that too-generic name by; for example, existing standalone `AGENTS.override.md`, `CLAUDE.md`, `.claude/CLAUDE.md`, `GEMINI.md`, and `.github/copilot-instructions.md` files must carry the managed block as rendered for that file's own agent ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)), while `CLAUDE.md -> AGENTS.md` is already covered by the canonical file. `check` does not require absent agent-directory-triggered companions from [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place); once `grund init` creates one, it is validated because it exists.

If `AGENTS.md` does not exist, existing companion agent files without a managed block are treated as project-owned instructions and are not validated by `grund check`; this keeps config-only adoption from modifying or policing an existing agent setup. A companion that already contains a managed `grund init` block is still version-checked even without `AGENTS.md`, so repos initialized directly into `CLAUDE.md`, `GEMINI.md`, or another explicit entrypoint still get drift detection.

#### 3.5.2 Legacy and malformed blocks

A legacy H2-bounded block from v3 or earlier ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) is still recognized and reported as an *older block version* — `` run `grund init` `` is its transition path to the delimited form, so existing repositories are told how to migrate rather than treated as malformed. Broken delimiters are a distinct error: a file whose delimiters [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints) defines as **malformed** is reported as a malformed managed block, located at the offending delimiter line and naming the defect; `check` never rewrites the file, and `grund init` refuses to splice against broken delimiters for the same reason.

#### 3.5.3 Code and message

Every variant remains an error with code `agents-init`, and the default run still reports it after every content pass completes. Its text is the final template of [§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages), which classifies the work as repository maintenance and states that citation validity is unaffected; the three-release migration that led to it ended in `0.16.0` ([§FS-errors.3](FS-errors.md#3-message-text)). A selector may retain or remove this ordinary coded error, but never changes the checker pass that produced it.

A stale block is reported once per section that differs, each error naming its section by a lower-case noun: the paragraph under the block's version heading is the `citation scheme` ([§FS-init.2.3.4.2](FS-init.md#2342-citation-scheme)), and every `###` section is its heading lower-cased — `grounding from a citation`, `project map`, `project namespaces`, `workspace members`, `declarations and citations`, `rules`, `citation directions`, `clickable citations`, `declaration shapes`, `chapter rules`, `scoped guidance`. A section expected and absent, or present and not expected, is one that differs. The source a stale error names is `grund.toml` for every section but the two workspace sections, `project namespaces` and `workspace members`, whose source is `the workspace`: what moved them is another project's file, not the reader's own config.

Where the block belongs to another project than the one the run started in — a member's block reported by a run at the workspace root — every variant that names `grund init` also names the directory to run it in: the root of the project that owns the block, written from the same base as the finding's path ([§FS-errors.3](FS-errors.md#3-message-text)), so `packages/billing` for `packages/billing/AGENTS.md` and for `packages/billing/.claude/CLAUDE.md` alike. Where the block belongs to the run's own project the directory is left out. The exact placement of the directory in each template is [§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages).

##### 3.5.3.1 First differing workspace entry

For `workspace members` and `project namespaces`, the finding adds a parenthesized detail before the refresh command: `expected <entry>; found <entry>`. Expected is the section the existing re-render produces; found is the section in the block on disk. An entry is a nonblank body line, excluding the section heading: a member bullet, or a prose line in a section without bullets. Compare these lines verbatim in their existing order and report the first unequal pair, including an unmatched line when one side ends first. Use `<missing>` for the absent side. This positional comparison is deterministic; it does not sort entries, parse their Markdown, or diagnose later differences in the same finding.

The detail therefore names the differing member through its rendered entry, and shows a changed link, or a mark an earlier render left, as expected versus found text. A missing section supplies no entries, as does an unexpected section's absent expected side. For example, a newly added member can report ``expected - [`notify`](packages/notify/); found <missing>``, and a block still carrying an earlier render's mark can report ``expected - [`billing`](packages/billing/); found - [`billing`](packages/billing/) *(not yet initialized)*``. These are explanatory text, not additional findings or filesystem probes.

When the section bytes differ but no unequal body entry exists — blank-line, heading or section-order drift, for example — use `section text or order differs` as the detail. Entry selection explains the existing byte comparison ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)); it never replaces or narrows it. The code, severity, exit verdict, number and location of findings, owner-directory hint, and workspace comparison scope ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)) remain unchanged.

#### 3.5.4 No part of the block is exempt from the comparison

The re-render covers the whole block the live config and the workspace produce — the fixed text and every substituted line ([§FS-init.2.3.8](FS-init.md#238-substituted-content)) — and only two things go uncompared: the workspace sections in the runs [§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout) lists, and every block of a project while one of its rules is genuinely invalid ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)). `grund init` refuses to write while such a rule stands, so its located `invalid-rule` error is the finding to act on, and a run narrowed to `agents-init` keeps it ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). A chapter rule unverifiable in this scope still earns its bullet ([§FS-rules.4.1.2](FS-rules.md#412-what-init-writes-anyway)), so `### Chapter rules` is compared like any other section. A comparison the checker performs in one tree and silently omits in another is the undeclared blind spot [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) forbids.

A workspace run compares each project's block against the vocabulary of the workspace it loaded. That is what lets a run at the workspace root catch a member's managed block that is missing a bullet for a rule the root resolves, which is the disagreement [§FS-rules.9.1](FS-rules.md#91-one-tree-renders-one-block) exists to prevent. The render reads the config, the workspace and the scanned rule declarations ([§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules)), never a path argument ([§FS-rules.9.1.1](FS-rules.md#911-a-path-scoped-check-compares-against-the-same-render)) or a run flag such as `--full` or `--require-grounding`, so every `check` of one tree compares it with the block `grund init` writes there.

#### 3.5.5 The workspace sections are judged at the outermost root of a full checkout

`### Project namespaces` and `### Workspace members` ([§FS-init.2.3.4.16](FS-init.md#23416-project-boundaries), [§FS-init.2.3.4.15](FS-init.md#23415-workspace-members)) are rendered from the whole workspace: the outermost root and every member's alias, root and description. Only a run at the outermost workspace root whose expansion found every member has loaded all of that, so only such a run compares them, in every block it verifies. Every other run removes the two sections from both sides before comparing, and compares the rest:

1. a run inside a member, `grund check <member>` from the root among them, which would otherwise have to climb to the outermost root and expand the whole workspace on every run;
2. a run at the root of a nested group, for the same reason;
3. a run in a standalone project, whose block carries neither section;
4. a run in a workspace where an optional member is absent ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)), anywhere in it: the sections it would render are not the ones a full checkout renders, so comparing them would turn exactly that checkout red, and `grund init` there keeps the member list it cannot render ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)).

The boundary is declared here and bounded by the outermost root's run ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)). In the fourth case the run also says so on its own output: the `optional-member-absent` warning gains a clause naming the workspace sections whenever the run verified a managed block ([§FS-check.4.9.1](FS-check.md#491-one-warning-per-absent-entry)). A project removed from `members` keeps workspace sections no `check` compares; its own `grund init --check` still sees them ([§FS-init.1.4](FS-init.md#14-why---check-exists)).

### 3.6 Ungrounded unit *(opt-in)*

Off by default. Two config keys decide it, each written in `[reference]` as the default for every `[[kinds]]` row and settable on the row itself ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)): `require_grounding` says **whether** a place's files must be grounded ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs)), and `grounding_level` says **what the unit is** inside each of them ([§FS-check.3.6.2](FS-check.md#362-the-unit)). `grund check --require-grounding` ([§FS-check.1](FS-check.md#1-inputs)) sets the same global default, and an explicit `require_grounding = false` on a row wins over it.

A unit is **grounded** when it contains at least one recognized citation ([§FS-check.1.1](FS-check.md#11-recognized-citations)) whose ID resolves to a declaration — **or**, in a source file outside every non-citable home, when it declares an ID inline (a declaring file is grounded in the declaration it *is*, [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments)). A unit that is neither is an error ([§FS-check.3.6.3](FS-check.md#363-findings)). A unit whose only citation is dangling ([§FS-check.3.1](FS-check.md#31-dangling-citation)) is *not* grounded — it gets both findings; fixing the citation clears both. The rule is a pure function of `(tree, config)`: it reads no git history and parses no code ([§FS-check.3.6.4](FS-check.md#364-a-pure-function-of-the-tree)). Decided in [§DF-require-grounding](../decisions/functional/DF-require-grounding.md#df-require-grounding-an-opt-in-check-that-every-source-file-cites-a-spec).

Under v2 the two keys are one ladder of strength-and-unit rungs, executed in order: each rung reports the units of its size that are ungrounded on its strength's channel, so a `warn` rung's ungrounded unit is a `warning:` with this message and code, and a narrower ladder replaces the wider one whole ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). Inline declarations ground a unit as above, and the scanner still collects heading structure only for files a rung finer than `file` governs.


#### 3.6.1 Which files a row governs

Every scanned file resolves to exactly one `[[kinds]]` row, and that row's effective `require_grounding` decides whether the file is checked at all:

- **A non-citable kind's home** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) governs every scanned file in it, Markdown included: a `folder` home's files, or the one document of a `file` home ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), which takes both keys on its row ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). A folder home's entry file ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)) and a generated scoped entrypoint ([§FS-init.2.3.5.15](FS-init.md#23515-a-generated-scoped-entrypoint-is-not-a-unit-of-its-place)) are not among them.
- **A citable kind's folder home** governs the **source files** in it — a file the scan reads that is not read as Markdown ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule), [AR-scanner.1](../architecture/AR-scanner.md#1-tree-walk)), whatever its name.
- **The homeless kind** ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) governs the source files no home claims, and a file claimed by two overlapping homes falls to it as well, the way its citing side already does ([§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as)).

So Markdown is exempt except inside a non-citable home, and `require_grounding = true` on an unscanned home's row is a config error ([§FS-check.3.6.1.1](FS-check.md#3611-markdown-is-governed-only-in-a-non-citable-home)). A repository that sets only the global key keeps every level `1` and the file as the unit ([§FS-check.3.6.1.2](FS-check.md#3612-the-global-key-alone-is-the-rule-it-always-was)).

##### 3.6.1.1 Markdown is governed only in a non-citable home

Markdown is therefore exempt except inside a non-citable home, and that exception is a home rather than an extension. The Markdown exemption reasons about implementation versus document; a non-citable home is neither guess — it is a directory the maintainer declared matters, and it is usually *all* Markdown, a skill, a runbook, a prompt library. Inheriting the exemption there would switch the rule off exactly where it was turned on. An unscanned home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) has no scanned files, so the rule never reaches it — which is why `require_grounding = true` on such a row is a config error.

##### 3.6.1.2 The global key alone is the rule it always was

A repository that sets only `[reference] require_grounding = true` and configures no non-citable kind sees this rule exactly as it did before these keys existed: every row inherits the global `true`, every level is `1`, and the unit is the file.

#### 3.6.2 The unit

`grounding_level` is an integer in Markdown heading levels ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). Each level **contains the one below it**, so the file itself is always a unit and nothing passes vacuously for lacking structure. In a Markdown file, level `L` adds a unit for every heading subtree of depth `2` to `L` ([§FS-check.3.6.2.1](FS-check.md#3621-in-a-markdown-file)); in a source file, the units below the file are doc-comment blocks, ranked by indentation ([§FS-check.3.6.2.2](FS-check.md#3622-in-a-source-file)); and a source declaration grounds its doc-comment block and its file, except inside a non-citable home ([§FS-check.3.6.2.3](FS-check.md#3623-the-inline-declaration-escape)).

##### 3.6.2.1 In a Markdown file

Level `L` makes a unit of the whole file and of every heading subtree whose depth is between `2` and `L`. A subtree runs from its heading to the line before the next heading at the same or a shallower depth, so a parent is satisfied by any descendant and a leaf must cite directly; text before the first heading belongs to the file rather than to a section. At level `1` there are no section units and the file is the only one, which is the unit every config had before the key existed. A file with no heading between `2` and `L` is one unit — the file — for the same reason.

##### 3.6.2.2 In a source file

There are two ranks, and they are read by indentation rather than by syntax ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)): at level `2` every **unindented** doc-comment block is a unit — a parse-free stand-in for a top-level item, which holds across Rust, Python, Java, Go, and Kotlin — and at any higher level every doc-comment block is. What counts as a doc-comment is the per-language rule of [§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites), already read once per file by the scanner. The file is a unit at every level, as in Markdown. A Lisp doc-comment, which the Lisp row of [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages) made one in the release that added it, is a unit like any other. Through 0.18.x an ungrounded one was reported on the `warning:` channel, with the release in which it became an error appended to the message: `src/router.lisp:3: warning: ungrounded doc-comment: no § citation to a declared ID; this warning becomes an error in grund 0.19.0`. Since 0.19.0 it is the error every other ungrounded doc-comment is, `src/router.lisp:3: error: ungrounded doc-comment: no § citation to a declared ID; cite a declared ID within it`, with no release clause appended ([§DF-line-prefix-forms-declare.grounding](../decisions/functional/DF-line-prefix-forms-declare.md#grounding-a-lisp-doc-block-that-cites-nothing-warns-first), [§DF-lisp-doc-grounding-lands-in-0-19](../decisions/functional/DF-lisp-doc-grounding-lands-in-0-19.md#df-lisp-doc-grounding-lands-in-0-19-an-ungrounded-lisp-doc-comment-is-the-ordinary-error-from-0190)).

##### 3.6.2.3 The inline-declaration escape

The inline-declaration escape of [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in) applies per unit: a doc-comment block that declares an ID is grounded by that declaration, and so is the file it sits in. It has no effect inside a non-citable home, where a declaration is a misplaced declaration to begin with ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)) — there the only way to ground a unit is to cite one.

#### 3.6.3 Findings

A finding is located at its unit — line 1 for a file, the heading line for a section, the block's first line for a doc-comment — and names the unit and, when the unit sits in a non-citable home, the home:

```
src/foo.rs:1: ungrounded source file: no § citation to a declared ID; cite a declared ID within it
skills/triage/SKILL.md:1: ungrounded file in kind home skills/: no § citation to a declared ID; cite a declared ID within it
skills/review/SKILL.md:14: ungrounded section `## Steps` in kind home skills/: no § citation to a declared ID; cite a declared ID within it
src/walk.rs:41: ungrounded doc-comment: no § citation to a declared ID; cite a declared ID within it
```

The marker in the message is the configured one ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). Section units arise only inside a non-citable home, since that is the only place Markdown is governed, so a section finding always names one. Every failing unit is reported ([§FS-check.3.6.3.1](FS-check.md#3631-every-failing-unit-is-reported)).

##### 3.6.3.1 Every failing unit is reported

A file that cites nothing at level `2` earns the file finding *and* one per section, which is what "each level contains the one below it" means on the reporting side — the file is genuinely ungrounded, and so is each of its sections.

#### 3.6.4 A pure function of the tree

The rule is a pure function of `(tree, config)` like every other `check` rule ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)): it reads no git history ([§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking)) and parses no code ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)) — "source file" is decided by extension, a unit by heading depth or comment indentation, and "grounded" by the citations the scanner already collected. It is the floor of the grounding discipline — the verification-at-rest layer of [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers), on top of which `grund cover` exposes the citation graph ([§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file)) and [§RM-cochange-gate](../roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits) tracks the diff-aware co-change gate.

### 3.7 Misplaced declaration (configured kind home)

Moved to [§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home). This address is kept so citations written before the move still resolve.

### 3.8 Cross-project citation failure

An alias-qualified citation whose alias path is unknown is reported at the citation site in every run, with or without a `[workspace]` ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)); in a workspace run, so is one whose target declaration or target section is missing. The alias resolution rules live in [§FS-workspace.4](FS-workspace.md#4-resolution).

A member-local finding's verified enclosing-workspace explanation and root-check remedy are defined by [§FS-workspace.5.1.1](FS-workspace.md#511-verified-enclosing-workspace-guidance).

An unknown alias path carries its fix in the finding: the projects it could have meant, or the `[workspace]` entry that would load the project it names ([§FS-check.3.8.5](FS-check.md#385-an-unlisted-project-is-named-with-its-members-entry)). The outermost workspace root searches every project in scope ([§FS-check.3.8.2](FS-check.md#382-candidates-at-the-outermost-root)); a run narrowed to a subtree searches only for a strict extension of its scope ([§FS-check.3.8.1](FS-check.md#381-a-strict-extension-of-the-narrowed-scope-is-safe-to-hint)) and otherwise names the subtree it covers ([§FS-check.3.8.3](FS-check.md#383-a-narrowed-run-offers-no-candidate)), in the wording [§FS-check.3.8.4](FS-check.md#384-the-scope-only-message-across-three-releases) stages. The citation remains unresolved and the run still fails.

#### 3.8.1 A strict extension of the narrowed scope is safe to hint

A narrowed run may search for a candidate only when its non-empty scope path is a strict, segment-wise prefix of the written alias path: the written path starts with every scope segment and has at least one segment after them. Thus `group/alph` is eligible in scope `group`, and `group/alpha/bet` in scope `group/alpha`. The search sees only the aliases the narrowed run loaded, so every candidate it can name is inside the subtree the run can judge.

An eligible path gets the outermost root's ordered bands, sorting, three-result limit, prose joining, and `; did you mean …?` rendering ([§FS-check.3.8.2](FS-check.md#382-candidates-at-the-outermost-root)); when the loaded aliases yield no candidate, the message is the bare `unknown project alias <path>` form. Outermost-root runs keep their unconditional candidate search.

Shorter and equal paths, paths whose segments do not begin with the scope, and merely lexical prefixes remain ineligible: in scope `group`, that excludes `alpha`, `group`, `outside/alpha`, and `grouped/alpha` alike. Each gets the scope-only message of [§FS-check.3.8.4](FS-check.md#384-the-scope-only-message-across-three-releases) with `<scope>` = `group` — in `0.13.2` through `0.15.0` the staged form with its exact compatibility suffix, from `0.16.0` the final template. Eligibility, candidate selection, resolution, and the failing verdict do not change in any of these releases.

#### 3.8.2 Candidates at the outermost root

At the outermost workspace root — the scope CI runs, and the only one that can see every project a path could name — candidates are taken from the projects in scope in one band only, best first: a project whose slash-separated path has every written segment as a **proper prefix** and at least one further segment, else one whose path **ends with** what was written (a dropped prefix — the mistake full alias paths invite, [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)), else one whose **last segment** matches (a wrong prefix), else one a **typo** away under the near-match rule of [§FS-check.3.1.1](FS-check.md#311-a-near-id). Thus `group` matches `group/alpha`, and `group/alpha` matches `group/alpha/beta`; an exact `group`, `grouped/alpha`, and `other/group` do not match the proper-prefix band. The first non-empty band alone wins, without candidates from any lower band. Its candidates are deduplicated, sorted by alias bytes, and then limited to three — `grund list` is the catalogue, a finding is not — and a path with no candidate gets no `did you mean`.

```text
docs/FS-root.md:3: unknown project alias sprayer; did you mean hardware/sprayer?
docs/FS-root.md:4: unknown project alias api; did you mean left/api or right/api?
docs/FS-root.md:5: unknown project alias group; did you mean group/alpha?
```

#### 3.8.3 A narrowed run offers no candidate

A run narrowed to a subtree ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)) holds only part of the tree, so a path naming a project outside it is unknown *here* while being exactly right at the workspace root. Except for the visibly in-scope case of [§FS-check.3.8.1](FS-check.md#381-a-strict-extension-of-the-narrowed-scope-is-safe-to-hint), such a run therefore offers **no candidate at all**. It cannot tell a dropped prefix from a path that correctly names a project outside its subtree, and every band reads it as the former: the dropped-prefix band included, because a *shorter* written path is itself a complete alias path whenever a top-level project carries that name, so re-pointing it at a deeper namesake rewrites a citation CI accepts into a different project's — green before and green after, so nothing catches it.

In every ineligible case it names the subtree it covers — as a *subtree*, since that scope's own alias path is one project among the several it holds — rather than reporting the path bare, which is neither "delete this" nor "re-prefix this" but "check this from the root."

#### 3.8.4 The scope-only message across three releases

A narrowed run's scope-only finding reads exactly:

```text
unknown project alias <path>; the <scope> project and its descendants are in scope here — check from the workspace root for a path outside that subtree
```

It says what "the subtree" holds — the named project and its descendants — before it says where to check from instead. From `0.13.2` through `0.15.x` it was a compatibility form that kept the earlier `only the <scope> subtree is in scope here` wording as a contiguous prefix and ended with a suffix naming `0.16.0` as the release its wording would change in; that form retired in `0.16.0`.

#### 3.8.5 An unlisted project is named with its members entry

A run that is not narrowed and finds no candidate still names the repair when the alias path is a directory beneath the run's root whose `grund.toml` or `.agents/grund.toml` no `[workspace]` lists ([§FS-check.3.29.2](FS-check.md#3292-an-unanswered-claim-is-not-reported-and-is-asked-quietly)): the message gives the `unknown-project` row's `unlisted` case, and stays bare where membership cannot be decided. A root with no `[workspace]` citing `<§>api/FS-orders` into an `api/` laid by `grund init` reports `docs/FS-root.md:3: unknown project alias api; add "api" to [workspace] members in grund.toml`.

### 3.9 Section heading level mismatch

Moved to [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch). This address is kept so citations written before the move still resolve.

### 3.10 Inline citation style violation

A citation site in a code comment that violates the configured inline citation style — `inline_style = "citation-only"` with prose present, an inline note that exceeds `inline_note_max_lines`, or one that exceeds `inline_note_max_columns`. A site is an *inline* comment block, never a doc-comment block, so nothing in this rule reaches a citation written inside a `///`, a `/** … */`, a docstring, or a comment documenting the definition below it ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)). The full mode and budget contract, and how multi-cap violations split into multiple findings, lives in [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps); the schema for the controlling keys is in [§FS-config.3.1](FS-config.md#31-reference--citation-form). An opt-in layout check adds one further form ([§FS-check.3.10.1](FS-check.md#3101-layout-deviation)).

#### 3.10.1 Layout deviation

With `[reference] inline_note_layout` set to a layout and `inline_note_layout_check = "error"`, each line that [§FS-inline-citation-style.3.3.1](FS-inline-citation-style.md#331-per-line-not-per-site--and-only-where-a-note-opens) judges, in a citation site that carries a note ([§FS-inline-citation-style.3.3.2](FS-inline-citation-style.md#332-only-sites-that-carry-a-note)), and that does not match the configured form is an error located at that line ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit), [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)). The same deviation is a warning under `inline_note_layout_check = "warn"` ([§FS-check.4.4](FS-check.md#44-inline-note-layout-deviation-opt-in)) and silent at the default `off`. It is the one member of this rule that anchors per line rather than at the site's first line, because a layout deviation is a property of the line an author has to edit.

### 3.11 Missing required citation

When `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) sets a `must` obligation for a citing kind, every top-level declaration of that kind must carry at least one citation satisfying each `must` entry, anywhere in its body. A declaration that does not is reported as `missing-citation` at the declaration line, naming the unmet target:

```
docs/architecture/AR-router.md:1: AR-router must cite FS or GOAL (citation direction); add such a citation within it
```

The body extent and the citing-side classification come from the scanner ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)); the obligation pass is [AR-checker.2.9](../../crates/grund-core/src/checker/report.rs). The homeless kind ([§FS-check.3.11.1](FS-check.md#3111-the-homeless-kind)) and a non-citable kind ([§FS-check.3.11.2](FS-check.md#3112-a-non-citable-kind)) are asked per file instead, at the row's `grounding_level` ([§FS-check.3.11.3](FS-check.md#3113-the-unit-follows-grounding_level)); every failing unit is reported ([§FS-check.3.11.4](FS-check.md#3114-every-failing-unit-is-reported)), a file with no citation is no unit ([§FS-check.3.11.5](FS-check.md#3115-a-file-with-no-citation-is-no-unit)), and an `E2E` case has its own unit ([§FS-check.3.11.6](FS-check.md#3116-an-e2e-case)). The parallel `should` obligation reports `suggested-citation` instead ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). A declaration a stub pairs with is the unit wherever its target is: an outside target lends it its citations ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)), so the obligation is met by what the target cites, a unit only that target supplies keeps the obligation from applying to nothing ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)), and the finding of one that cites nothing is reported at the declaration's line in the target ([§FS-declarations.stubs.pairing.4](FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target), [§FS-rules.7.11](FS-rules.md#711-a-citation-or-chapter-read-through-a-stub-warns-until-0190)).

The ordinary rule sentence `<subject> must cite at least one <target-set>.` reuses this code only when its actual count is zero. Rule-to-rule and the narrow config-to-rule bridge deduplicate as [§FS-rules.6](FS-rules.md#6-semantic-deduplication) specifies; when config participates, this section's existing message stays byte-for-byte unchanged.

Under v2 the obligation is written as `must`, `warn` or `should` ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)); `warn` reports this code too, at the severity its row gives it. v1's `[citations]` keeps exactly the channels above.


#### 3.11.1 The homeless kind

A **homeless-kind** obligation ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) — `code`, or whatever the project named it — is per file rather than per declaration: a source file that contains at least one citation but none satisfying the obligation is the finding, located at line 1.

#### 3.11.2 A non-citable kind

**A non-citable kind's obligation is per file too** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), and its unit is every scanned file in the kind's home that carries at least one citation — **a file read as Markdown included** ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)), unlike `code`. Obligations attach to declarations, and a kind that declares nothing would otherwise yield no units at all and let `must` pass vacuously; inheriting `code`'s Markdown exemption would do the same thing a second time, since such a home is usually all Markdown. The finding names the **home**, because the unit has no ID to print:

```
skills/review/SKILL.md:1: skills/ must cite FS (citation direction); add such a citation within it
```

#### 3.11.3 The unit follows `grounding_level`

Both per-file units follow the row's `grounding_level` ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)): *whether* a place's files must cite and *what* they must cite are asked of the same thing ([§FS-check.3.6.2](FS-check.md#362-the-unit)). At level `2` the unit of a non-citable Markdown home is every `##` subtree that carries a citation, and of a source file every unindented doc-comment block that does; the file stays a unit at every level, satisfied by any citation under it. A row at level `1` — which is every configuration written before the key existed — sees no change, and a citable kind's unit stays its declaration at every level, a declaration already being a unit inside a file.

#### 3.11.4 Every failing unit is reported

As for grounding ([§FS-check.3.6.3.1](FS-check.md#3631-every-failing-unit-is-reported)), at level `2` or above a file whose citations satisfy no `must` entry earns the finding on the file unit *and* one on each section unit that satisfies none either: the file genuinely cites no such target, and neither does the section. The two lines differ only in the anchor, which is what tells the reader whether the miss is local to one section.

#### 3.11.5 A file with no citation is no unit

Units are built from citations, so a file carrying none produces no unit and `must` cannot fire on it — except that a scanned folder with real non-entry content earns the run-level warning of [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing). The same zero-unit boundary [§FS-config.3.9.2](FS-config.md#392-the-homeless-kind) states for the homeless kind remains intentionally unwarned. In a non-citable home `require_grounding` closes the per-file grounding hole ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)): there the grounding rule follows the home rather than the file extension, so "cite something" and "cite an `FS`" are two keys that compose, while the warning of [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing) points at the row's key when grounding is off.

#### 3.11.6 An `E2E` case

An `E2E`-kind obligation ([§FS-config.3.9.1](FS-config.md#391-levels)) is per case declaration ([§FS-declarations.e2e-case](FS-declarations.md#e2e-case-what-an-e2e-case-directory-declares)), can be satisfied by the case's `spec.refs` manifest entries, and is reported when the case has no scanned citations or matching manifest entry.

### 3.12 Forbidden citation

When `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) sets a `must-not` prohibition for a citing kind, every citation site of that kind to a prohibited target is reported as `forbidden-citation` at the citation site:

```
docs/functional-spec/FS-login.md:42: FS must not cite AR (citation direction) — re-point the citation or downgrade it to a plain Markdown link
```

The prohibition pass is [AR-checker.2.10](../../crates/grund-core/src/checker/report.rs); how it reads the citing and the cited kind is [§FS-check.3.12.1](FS-check.md#3121-how-the-two-kinds-are-read). The parallel `should-not` prohibition reports `discouraged-citation` instead ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)). The sanctioned way to keep a discouraged downward pointer is a plain Markdown link, which is not a citation under `strict = true` and so is exempt from this rule. A citation a stub's target outside `[scan] include` lends the stubbed declaration ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)) is a citation site of its kind, reported at its line in the target, as with the target scanned ([§FS-rules.7.11](FS-rules.md#711-a-citation-or-chapter-read-through-a-stub-warns-until-0190)).

A rule sentence `<subject> must not cite any <target-set>.` reuses this code and
the exact citation-site anchor. Outside the index-entry repair below, its message
replaces only `(citation direction)` with `(<RULE-ID>)`; a config-derived
duplicate keeps this section's bytes unchanged ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)).

Under v2 the prohibition is written as `must-not`, `warn-not` or `should-not` ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)); `warn-not` reports this code too, at the severity its row gives it.


#### 3.12.1 How the two kinds are read

The citing kind is the one [§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as) gives the site, named by kind for a citable kind, by **home** for a non-citable one the way [§FS-check.3.11.2](FS-check.md#3112-a-non-citable-kind) names it (`skills/ must not cite AR`), and by name for the homeless kind, which has no home to name it by ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)). The cited kind and alias come from the citation token, matched against the rule's alias grammar ([§FS-config.3.9.3](FS-config.md#393-alias-matching)).

#### 3.12.2 Repair advice preserves a required index entry

For a kind's own index entry ([§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is)), a `forbidden-citation` finding keeps its code, severity, citation site, authority and structured fields, but replaces the plain-link repair with advice to retain the marked entry and change the applicable citation rule. The unused-count exemption ([§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count)) changes no direction verdict. Merely being in an index file does not make another citation an entry; ordinary sites keep this section's existing advice.

When a kind or global negative default supplies the prohibition, name the citing kind's configuration table and advise adding the target kind to its `may` list without removing existing permissions ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)). For v2 REQ with `default = "must-not"` and `may = ["GOAL"]`, name `[rules.citations.REQ]` and adding REQ to `may`: the accepted edit is `may = ["GOAL", "REQ"]`. It clears the direction error while the marked entry still satisfies the index rule; a markerless link remains an index error. This repair also applies to `warn-not`, on its existing warning channel; `should-not` retains its suggestion without a repair suffix.

For an explicit prohibition, identify the matching prohibition list and table to change instead; for an independent rule sentence, identify that rule as the authority to change. Do not claim that adding `may` overrides either. Both diagnostic producers obey this point, including a rule sentence whose citation site is an index entry. No configuration precedence or index-entry requirement changes.

### 3.13 Number-only shorthand citation

A recognized shorthand citation ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) persisted in a scanned file is governed by the target project's `[reference] shorthand` policy ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). Under the default `canonical` policy, a uniquely resolving site is reported with its replacement text and `grund fmt --write` applies that mechanical fix in bulk ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)). Under `accepted`, the same marker-origin site is valid and produces no shorthand-form finding; its full canonical citation may coexist in the same file. The policy gate changes only the unique result: a shorthand matching no declaration or several still earns the unknown or ambiguous form of [§FS-check.3.13.3](FS-check.md#3133-one-finding-per-site-in-three-forms), and no policy permits grund to guess.

Under `canonical`, an error rather than a warning or a suggestion: a warning leaves the exit code alone, so a repo could accumulate shorthand citations forever while CI stayed green ([§DF-number-only-citation-shorthand.2.3](../decisions/functional/DF-number-only-citation-shorthand.md#23-it-is-an-error-not-a-warning-or-a-suggestion)). A citation of a kind whose effective format has no `{number}` or no `{slug}` never earns this finding, because that kind has no shorthand ([§FS-check.1.2.1](FS-check.md#121-the-shorthand-shape)). Where the text itself forbids the rewrite, the resolving form does not fire ([§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite), [§FS-check.3.13.2](FS-check.md#3132-a-python-docstring-is-not-a-string-literal)).

#### 3.13.1 Where the text forbids the rewrite

Where the text exempts a shorthand from the resolving form of this error is [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite).

#### 3.13.2 A Python docstring is not a string literal

A Python docstring is not a string literal for this exemption: its delimiters are doc-comment syntax, so the question is asked of its content ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)), and a shorthand anywhere inside one — the opening line, a one-line docstring, an interior line, the closing line — is reported and rewritten exactly like one in a `#` comment. A shorthand inside a `"…"` or `'…'` literal on a **code** line is what stays exempt.

#### 3.13.3 One finding per site, in three forms

At most one *shorthand* finding per site, in one of three forms. Other rules judge the site on their own terms — a bad section ([§FS-check.3.2](FS-check.md#32-missing-section)) or a forbidden direction ([§FS-check.3.12](FS-check.md#312-forbidden-citation)) is a separate fact about the same citation and is reported separately:

```
docs/notes.md:5: shorthand citation §FS-042; write §FS-042-user-login
docs/notes.md:6: shorthand citation §FS-999 matches no declaration; write the full ID it means
docs/notes.md:7: shorthand citation §FS-042 is ambiguous: FS-042-user-login, FS-042-user-logout; write the full ID it means
```

The candidate list in the ambiguous form is sorted and complete — `grund` names every match and resolves none, because choosing one would be a guess and `check` reports facts about the tree ([§FS-check.5](FS-check.md#5-what-grund-does-not-check), [§GOAL-agent-grounding.3](../goals.md#3-what-this-rules-out), [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)). Duplicate *numbers* are not otherwise an error: [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) catches duplicate full IDs, and a repo may legitimately hold `FS-042-user-login` alongside `FS-042-user-logout` as long as nothing abbreviates them. The marker rendered in the message is the configured one ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), and the qualified form names its alias (`<§>api/FS-042`, escaped here because this repo has no `api` member) so the replacement can be pasted as written.

### 3.14 Out-of-scope unresolvable citation *(`--full` only)*

Under `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), a citation in a file outside the default scope that resolves to nothing is an error there too: an ID declared nowhere ([§FS-check.3.1](FS-check.md#31-dangling-citation)), a section its declaration does not have ([§FS-check.3.2](FS-check.md#32-missing-section)), an unknown alias ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)), and a number-only shorthand that matches zero or several declarations ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). So is a declaration-local section citation, in each shape of [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation), a verdict no consumer can settle from outside the file holding the token, and so is a marked token its kind's format does not spell and nothing declares ([§FS-check.checks.off-format-citation](FS-check.md#checksoff-format-citation-off-format-citation)). A citation whose declaration is also out of scope resolves normally, because this reports citations that point at nothing, not citations that point outside the default scope.

What is reported out there is a verdict on a citing token, reached from that token, the declaration structure of the file holding it and the resolved graph, and from nothing the project assigned the file: not its kind, its home or its citation obligations, and no convention a configured key switched on ([§DF-check-full-scope.2.2](../decisions/functional/DF-check-full-scope.md#22-out-of-scope-findings-are-the-verdicts-a-citing-token-and-the-files-own-structure-settle)). The rules [§FS-check.1.3.3](FS-check.md#133-two-scopes-two-rule-sets) keeps to the default scope do not run there, and one declaring-side finding does besides the six below: [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration), with its own unprefixed code and its ordinary message. A verdict about a rewrite `grund fmt` did not make is withheld there, because `fmt` scopes by the default scope too and an error never names a fix `grund` declines to make ([§FS-check.requirements.no-declined-fix](FS-check.md#requirementsno-declined-fix-an-error-never-names-a-fix-grund-declines-to-make--realized-directional-for-the-next-finding)): [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s resolving form and [§FS-check.3.15](FS-check.md#315-shorthand-citation-in-a-numeric-run)'s finding are not reported, and [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)'s finding drops its `` ; run `grund fmt --write` `` clause.

Each of the six citing-token findings carries `out-of-scope-` and then the code of its in-scope equivalent: `out-of-scope-dangling`, `out-of-scope-missing-section`, `out-of-scope-local-section-citation`, `out-of-scope-unknown-project`, `out-of-scope-shorthand-citation`, `out-of-scope-off-format-citation`. A JSON consumer ([§FS-errors.5](FS-errors.md#5-json-format)) filters the scope by the prefix and the rule by the exact code, and the shape gains no field. The message leads with the scope the file lies outside and then gives the rule's own message. Under version 1 that scope is `[scan] include`; under version 2 it names the configured places and never `[scan] include` ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)). The first line is version 1's, the second version 2's:

```
sim/world.py:2: error: outside [scan] include: missing section FS-001-login.4; write <§> before it to show the shape without citing it
scripts/notes.md:3: error: outside configured places and source scope: unknown reference FS-ghost
```

A file out there that cannot be read or decoded gets the scan-failure line of [§FS-check.2.4](FS-check.md#24-an-incomplete-run) and the run exits `2`, as in the default scope, so a tree whose plain `check` exits `0` can exit `2` under `--full`.

### 3.15 Shorthand citation in a numeric run

A number-only shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) that resolves to exactly one declaration but sits glued to a second number, so `grund fmt` will not rewrite it ([§FS-fmt.2.4.1](FS-fmt.md#241-a-shorthand-in-a-numeric-run-is-not-rewritten)), is `shorthand-numeric-run`. It takes [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s place at that site rather than adding a second shorthand finding, while findings about other facts of the citation still report beside it. It names both fixes ([§DF-shorthand-numeric-run.2.5](../decisions/functional/DF-shorthand-numeric-run.md#25-the-site-is-reported-and-the-report-names-both-exits)). Decided in [§DF-shorthand-numeric-run](../decisions/functional/DF-shorthand-numeric-run.md#df-shorthand-numeric-run-a-marked-shorthand-glued-to-another-number-is-a-numeral-not-a-citation).

```
docs/notes.md:1: error: shorthand §FS-042 sits in a numeric run and was not rewritten; write §FS-042-user-login, or <§>FS-042 if these are old numbers
```

It is withheld in the three contexts where the text itself forbids the rewrite ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite)), as [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s resolving form is, and it fires in a suppressed scope and at an external file-symlink target, as that form does ([§DF-fmt-suppression.2.4](../decisions/functional/DF-fmt-suppression.md#24-check-is-unchanged-and-the-repository-pays-for-that)). Under `--full` it is withheld outside the default scope ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)). A shorthand in a run that matches zero or several declarations keeps [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s message.

### 3.16 Duplicate section path

Moved to [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path). This address is kept so citations written before the move still resolve.

### 3.17 Index entry is not a link

A bare, marked citation of a covered ID ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) in its kind's index file, where the index holds no entry for that ID, is `unlinked-index-entry` if the next `grund fmt --write` would wrap it ([§FS-fmt.6.1.1](FS-fmt.md#611-a-kinds-index-entries-are-always-linkified), [§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped), [§FS-fmt.6.5](FS-fmt.md#65-interaction-with---marker)). An ID is reported once, at the first such citation in file order:

```
docs/fs/README.md:3: error: index entry §FS-001-login is not a link; unchecked in grund 0.11.0, an error in 0.12.0 — run `grund fmt --write`
```

The repair is the link of [§FS-fmt.6.2](FS-fmt.md#62-form) ([§DF-index-entry-form.2.1](../decisions/functional/DF-index-entry-form.md#21-the-entry-is-a-recognized-citation-of-the-id-written-as-a-full-markdown-link)), and `check` judges an entry's shape, never its target ([§FS-citations.wrap.shape](FS-citations.md#wrapshape-the-shape-never-the-target)). Nothing else in the index is reported here, because an error never names a fix `grund` declines to make ([§FS-check.requirements.no-declined-fix](FS-check.md#requirementsno-declined-fix-an-error-never-names-a-fix-grund-declines-to-make--realized-directional-for-the-next-finding)). A citation inside a link its author wrote is an entry where the link goes somewhere, and otherwise no entry ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)); any other bare citation `fmt` would not wrap is no entry. An ID the index then holds no entry for falls to [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index), whose fix is an edit by hand, so one cause never yields both findings.

#### 3.17.2 The shape, never the target

That `check` requires the wrap's shape and never its target is [§FS-citations.wrap.shape](FS-citations.md#wrapshape-the-shape-never-the-target).

### 3.18 Declaration missing from its kind's index

A kind with a `folder` and an index file — `README.md` unless `index` names another or opts out ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)) — promises that the index lists the folder's declarations. Each covered declaration the index does not list is one `missing-index-entry` error, located at the declaration's heading and naming the index file. Decided in [§DF-index-entry-form](../decisions/functional/DF-index-entry-form.md#df-index-entry-form-an-index-entry-is-one-full-link-per-id-and-nothing-else-about-the-page), [§DF-index-compatibility-ramp](../decisions/functional/DF-index-compatibility-ramp.md#df-index-compatibility-ramp-a-findings-ramp-follows-its-fix-command-not-the-size-of-the-offence), and [§DF-index-not-an-inbound-citation](../decisions/functional/DF-index-not-an-inbound-citation.md#df-index-not-an-inbound-citation-an-index-entry-is-navigation-not-use).

A declaration is covered when its kind declares IDs, since a `citable = false` kind has no index ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), and it has a declaration site anywhere in the folder's subtree, not only its top level. A stub under the folder puts its ID there, and one entry satisfies a stub-and-inline pair ([§FS-list.2](FS-list.md#2-behaviour), [§DF-index-entry-form.2.5](../decisions/functional/DF-index-entry-form.md#25-the-walk-is-the-folders-whole-subtree-and-a-stub-and-inline-pair-collapses)). A declaration of another kind in the folder is [§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)'s and is not demanded here.

An entry is one recognized citation ([§FS-check.1.1](FS-check.md#11-recognized-citations)) of the ID in the index file, written as a full Markdown link: a wrap ([§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets)), or a citation in a link its author wrote that goes somewhere, in any form of [§FS-citations.wrap.enclosing.scope](FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in) ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)). A link is an entry whatever `fmt` would do with it. A bare citation `fmt` would not wrap ([§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link)) is no entry, so an index that mentions the ID only that way is reported here and never under [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link). Layout is free — table or list, grouped or flat, in any order, with any prose around it ([§DF-index-entry-form.2.4](../decisions/functional/DF-index-entry-form.md#24-layout-is-free)) — and one link per ID is enough: every other occurrence of the ID in the index is never a finding ([§DF-index-entry-form.2.3](../decisions/functional/DF-index-entry-form.md#23-one-link-per-id-not-every-mention)).

An index may also enroll one external source declaration, with no stub under `folder`: an unqualified, marked citation of the bare ID whose declaration is in a non-Markdown source file outside `folder`, wrapped as exactly the link `grund fmt --cross-refs` derives from that index to the source home ([§FS-fmt.6.2](FS-fmt.md#62-form)). That link is both the membership and the entry ([§DF-index-entry-form.2.7](../decisions/functional/DF-index-entry-form.md#27-a-canonical-bare-id-link-enrolls-an-external-inline-declaration)). Nothing else enrolls. A number-only shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)), a citation in a link its author wrote, whatever its destination, a citation-only image ([§FS-citations.checks.image-index-entry](FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release)) and a citation on a declaration heading ([§FS-citations.recognized.title-lines](FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names)) are ordinary citations, which [§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count) counts as uses. Where two kinds share one `folder` and `index`, a link enrolls for the kind its ID names. Removing the link removes the membership, so nothing is reported for an external declaration the index no longer claims.

Where the index mentions the ID but holds no entry for it, the message says the ID appears there but not as an entry; otherwise it says the ID is not listed and gives an entry to paste:

```
docs/fs/FS-001-login.md:1: error: FS-001-login appears in docs/fs/README.md but not as an entry: an entry is a `§`-marked Markdown link to the declaration — became an error in grund 0.13.0
docs/fs/FS-002-session.md:1: error: FS-002-session is not listed in docs/fs/README.md: an entry is a `§`-marked Markdown link to the declaration; write `[§FS-002-session](FS-002-session.md#fs-002-session-user-session)` in docs/fs/README.md — became an error in grund 0.13.0
```

The form both wordings name follows `[reference] strict`: the marked link above under `strict = true`, and off strict mode, where a bare token is a citation too ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), a Markdown link to the declaration whose text is the ID, with or without the marker. It is a form that is enough, not the definition of an entry. The entry to paste has the marker and the ID as its text and the link `grund fmt --cross-refs` writes from that index as its destination ([§DF-index-entry-form.2.2](../decisions/functional/DF-index-entry-form.md#22-full-link-is-the-link-fmt-would-write-here-not-has-an-anchor)); written in, it clears this finding, raises no [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link) one, and the next `grund fmt --write` leaves it alone. Where `fmt` derives no link — an ID declared more than once, or a broken stub ([§FS-fmt.6.4.2](FS-fmt.md#642-a-citation-of-a-broken-stub-or-of-an-ambiguous-id-is-not-wrapped)) — the message names the form alone. An external source declaration never reaches the not-listed wording, since its canonical link is its entry.

The index mentions the ID when one of its lines carries the ID as `grund` renders it, as a whole ID-shaped token with or without the marker or a section suffix, or the whole file name of the declaration the finding is located at. Lines inside a fence that `[schema.sources] fenced` does not select ([§FS-languages.fence](FS-languages.md#fence-fences)), and declaration headings, are not looked at; inline code is. A mention raises and satisfies nothing, and only chooses the words.

A missing index file is this same finding, once for each declaration in the folder. Its message is the not-listed wording, never the appears one, with a parenthesis after the index's name saying why it could not be read — `(the index file does not exist)`, `(the index file is a directory)` or `(the index file could not be read)` — and the form and the entry after it. The rule is skipped for an index file the run did not scan, such as one the `[scan]` set excludes, while a missing index, or a directory in its place, is still reported; a path scope reads every index the whole run reads ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)).

### 3.19 Orphan name-bearing section path

Moved to [§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path). This address is kept so citations written before the move still resolve.

### 3.20 Invalid value declaration

What [§FS-values.2](FS-values.md#2-value-declarations) refuses as a value is `invalid-value-declaration`. In a kind opted into whole values ([§FS-values.1](FS-values.md#1-per-kind-opt-in-and-identity)), that is a readable Markdown or value-source declaration that breaks it, located at the exact invalid heading, key or element. In any kind, it is also an embedded tag in an invalid place or a marked root of an invalid shape ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)), and content a declared value chapter may not hold or a chapter root whose run is invalid ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)), each located at the tag, line or root heading those points name. A duplicate declaration keeps [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) instead, a duplicate section may carry this finding beside its own, and a value source [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete makes the run exit `2` ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)).

### 3.21 Invalid value binding

An attempted backtick-delimited binding that breaks the grammar of [§FS-values.3.1](FS-values.md#31-the-only-binding-grammar) is `invalid-value-binding`, located at the attempted form. So is one whose literal closes its line onto a next-line citation ([§FS-values.3.1.1.1](FS-values.md#3111-a-literal-that-closes-its-line-onto-the-citation)), one aimed at a declared value chapter heading itself or below one of a root's immediate components, and one aimed at a root one of whose components contains an ASCII space ([§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)). What is not an attempt, and stays ordinary prose and citations, is [§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts)'s.

### 3.22 Value mismatch

After ordinary citation and section resolution succeed uniquely, a binding whose authored component differs from its whole declaration or marked root under [§FS-values.4](FS-values.md#4-exact-equality) — or, aimed at the root, whose literal does not split into that root's components ([§FS-values.5.2.2](FS-values.md#522-a-root-aimed-mismatch-names-the-first-unequal-component)) — is `value-mismatch`, located at the binding and naming the component site: the first unequal component's, or component 1's when the counts differ. Its canonical text and NDJSON parity are [§FS-values.5](FS-values.md#5-resolution-findings-and-exit-status)'s. Every resolution, declaration and section finding [§FS-values.5.1](FS-values.md#51-resolve-before-comparison) lists runs first and suppresses the comparison at that site, so one bad citation is never also reported as a mismatch.

### 3.23 Section outside a declaration

Moved to [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration). This address is kept so citations written before the move still resolve.

### 3.24 Declaration-local section citation

Every candidate from [§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates) earns one `local-section-citation` error whose CLI and LSP range is its whole authored token, in every location, under configured markers and both strict modes, and every shape of the message carries a release clause naming the two releases its verdict moved between, as [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) asks ([§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). Decided in [§DF-declaration-local-section-shorthand](../decisions/functional/DF-declaration-local-section-shorthand.md#df-declaration-local-section-shorthand-local-numeric-section-citations-are-recognized-but-never-canonical).

- **An owned numeric path.** Where the owner has the cited section, the error names the full citation to write. Where it has not, the error names the absence and offers a full citation of whichever document was meant or the escape, beside the `missing section` [§FS-check.3.2](FS-check.md#32-missing-section) reports on the same line ([§DF-local-section-absent-target-escape](../decisions/functional/DF-local-section-absent-target-escape.md#df-local-section-absent-target-escape-an-owned-local-section-citation-whose-section-is-absent-is-answered-with-the-escape-in-place)). Either way the site stays a real edge for every graph consumer.
- **An ownerless or ambiguous site.** No enclosing declaration can be chosen, so the error asks for a full citation or the escape, and names no guessed ID or navigation target.
- **A digit-starting mixed, named or glued tail.** The error names the whole unsupported token and gives the same guidance, and no numeric prefix becomes an edge.

```
docs/FS-a.md:3: error: local section citation §2; write §FS-a.2 — unchecked in grund 0.13.1, an error in 0.14.0; run `grund fmt --write`
docs/FS-a.md:5: error: local section citation §9.9; FS-a has no section 9.9, so write a full citation or <§>9.9 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0
docs/outside.md:3: error: local section citation §2 has no enclosing declaration; write a full citation or <§>2 to show the shape without citing it — unchecked in grund 0.13.1, an error in 0.14.0
```

The full citation joins the owner and the path with `[id] section_separator`. The path and the escape keep the `.` a local path is always written with ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)), and the escape is the token with its configured marker in angle brackets, `<@>5.1` under `@`. The absent-target wording is printed exactly where [§FS-check.3.2](FS-check.md#32-missing-section) reports `missing section` for the same coordinate on the same line, a stub's sections read as [§FS-check.3.2.1](FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not) reads them, in any context.

`` ; run `grund fmt --write` `` follows the release clause only at an owned site whose section exists and whose text permits the rewrite, inside the default scope. It is withheld in the three contexts of [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite), in a Markdown file's front matter ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)), whose citations belong to the file's first declaration, and outside the default scope under `--full` ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)), where the error still fires with its manual remedy; and at an ownerless, ambiguous or unsupported token, which [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical) cannot expand, and at an absent target section, which it does not rewrite ([§FS-fmt.2.4.6](FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)). A `[fmt] exclude` file ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes)), a `grund:fmt off` region and a file reached as an external file-symlink target ([§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)) keep the clause though `fmt` writes nothing there, because the author can lift that refusal ([§FS-check.requirements.no-declined-fix](FS-check.md#requirementsno-declined-fix-an-error-never-names-a-fix-grund-declines-to-make--realized-directional-for-the-next-finding)).

### 3.25 Invalid rule

A rule declaration whose title, rationale, vocabulary, named-section gate, or
exact literal subject is invalid produces `invalid-rule` at its heading. The
exact parse and resolution messages, and the pre-scan distinction for
`check --rule`, are [§FS-rules.4](FS-rules.md#4-validation-lifecycle) and
[§FS-rules.7.1](FS-rules.md#71-invalid-rule)'s.

### 3.26 Chapter cardinality

A `have` sentence whose named direct-chapter count is outside its constraint
produces `chapter-cardinality` at the subject declaration title, including
zero and surplus counts. Its fixed message is
[§FS-rules.7.2](FS-rules.md#72-chapter-cardinality)'s.

### 3.27 Citation cardinality

An outbound count not covered by [§FS-check.3.11](FS-check.md#311-missing-required-citation), and each off-count target of a
`cite each` sentence, produces `citation-cardinality` at the subject title.
Multiplicity, self-inclusion, per-target rows, message shape, and ordering are
[§FS-rules.3.2](FS-rules.md#32-outbound-citation-count),
[§FS-rules.3.3](FS-rules.md#33-per-target-coverage), and
[§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)'s.

### 3.28 Uncited unit

An inbound `be cited by` count outside its constraint produces `uncited-unit`
at the subject declaration or named-chapter title. Its count and message are
[§FS-rules.3.4](FS-rules.md#34-inbound-citation-count-and-prohibition) and
[§FS-rules.7.4](FS-rules.md#74-inbound-citation-cardinality)'s.

### 3.29 Unlisted `[workspace]` block

A directory the run's scan reached whose config, under either discovery name ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), declares `[workspace]`, and whose canonical root no `[workspace]` block above it names among its `members` or `optional_members` entries, is claimed by nobody: the enclosing project's scan absorbs its subtree, while a run started at it names every project from itself ([§FS-workspace.6.1.8](FS-workspace.md#618-a-block-no-enclosing-block-lists-is-outside-the-chain)). The two scopes then spell the same projects differently — `c/FS-c` inside the block, `root/FS-c` at the repository root — so a citation passes the inner check and fails the run CI does ([§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)). Every run whose scan meets such a block reports `unlisted-workspace-block`. Decided in [§DF-unlisted-workspace-block](../decisions/functional/DF-unlisted-workspace-block.md#df-unlisted-workspace-block-an-unlisted-workspace-block-is-reported-by-the-walk-that-meets-it).

The claim is the ancestor climb of [§FS-workspace.6.1.7.2](FS-workspace.md#6172-the-quiet-climb-asks-the-same-ancestors), asked of the directory the scan found: it reads the `members` and `optional_members` entry text alone and climbs past the run's root, and a tree with no `[workspace]` block above it at all is the same case. Both discovery names are probed at each scanned directory, so a `.agents/grund.toml` block is found though the scan enters no hidden directory. The question is asked quietly, without [§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)'s undecidable-claim warning. An ancestor that names the block and cannot answer — a member list that will not expand, a config that will not parse — or whose `members` or `optional_members` text cannot be obtained at all leaves the block unreported ([§FS-workspace.6.1.8.1](FS-workspace.md#6181-two-shapes-stay-unreported), [§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)); a block no ancestor names is never silenced by an ancestor's breakage, because the claim is read off the entry text before anything is expanded.

A nested `grund.toml` with no `[workspace]` table is ordinary tree ([§FS-check.1.3.1](FS-check.md#131-the-scan-covers-the-whole-config-root)), a listed block is claimed at any depth, and a project root of this run — its own root and each member root the scan stops at ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)) — is never a candidate. Only the outermost unlisted block of a chain is reported, since it claims the blocks below it, and a block the scan reached under two spellings, such as through a directory symlink, is one finding at the spelling the scan met first. `include_root = false` on the block changes nothing ([§DF-unlisted-workspace-block.2.5](../decisions/functional/DF-unlisted-workspace-block.md#25-include_root--false-on-the-block-changes-nothing)); what it costs the block's own files is [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread), which can fire on the same block.

Every command whose run scans a project tree reports it — `check`, [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk) and the ID read of [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id) — because each spells projects from the same map ([§DF-unlisted-workspace-block.2.3](../decisions/functional/DF-unlisted-workspace-block.md#23-every-command-that-walks-carries-it-not-check-alone)). What it sees is the scan the run already makes: the roots of the default scope and their subtrees, minus member boundaries and what `[scan] exclude`, the ignore files and hidden directories prune below a root ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned), [§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)). A block outside that scan — under a narrowed `grund check <path>` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), in a pruned directory, or behind a member boundary — is not reported, a known limit ([§DF-unlisted-workspace-block.2.2](../decisions/functional/DF-unlisted-workspace-block.md#22-the-scope-is-the-walk-the-run-already-makes-and-the-residue-stays-recorded)), and `grund check --full` may report a block the plain run does not ([§FS-check.1.3.4](FS-check.md#134-purely-additive)).

In `check` it is a located error at the block's `[workspace]` line, on stdout ([§FS-check.2.1](FS-check.md#21-report-format)) and under `--format=json` with that `path` and `line` and `sites` `null` ([§FS-errors.5](FS-errors.md#5-json-format)), and the editor publishes it from there ([§FS-lsp.1.1.3](FS-lsp.md#113-workspace-warnings-on-the-runs-warning-channel)). The other five surfaces have no error channel for a fact about the run, so each prints one CLI-level `warning:` on stderr with the location inside its text ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), and its exit code does not move ([§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed)). Decided in [§DF-unlisted-workspace-error-shape](../decisions/functional/DF-unlisted-workspace-error-shape.md#df-unlisted-workspace-error-shape-check-and-the-editor-take-the-located-shape-the-five-walking-surfaces-keep-the-cli-level-line).

```
b/grund.toml:7: error: this [workspace] is listed by no enclosing workspace — the projects under it are absorbed into `root` instead of named under their own alias path; add "b" to [workspace] members in grund.toml, or keep it out of that project's [scan] — an unlisted [workspace] became an error in grund 0.15.0
warning: b/grund.toml:7: this [workspace] is listed by no enclosing workspace — the projects under it are absorbed into `root` instead of named under their own alias path; add "b" to [workspace] members in grund.toml, or keep it out of that project's [scan] — an unlisted [workspace] became an error in grund 0.15.0
```

Paths are spelled from the run's report base ([§FS-errors.3](FS-errors.md#3-message-text)); the entry to add is the block's directory relative to the enclosing project's root, where both remedies are written; and the absorbing project is named by the alias path this run spells it with, as [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id) prints it. The second remedy names an outcome rather than a key because `[scan] exclude` never prunes the directory a scan starts at ([§FS-check.1.3.2](FS-check.md#132-the-wider-scan-reads-a-superset-each-file-once)): a block that is itself an `include` root is cleared through `include`, and one below a root through `exclude`.

### 3.30 A workspace member swallows the block's own scan

A `[workspace]` block every one of whose scan roots lies inside one of its own members ([§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan)) reads nothing: its declarations are unreachable and its citations are never checked. Every command that resolves the block's member boundary — `check`, `list`, `refs`, `show`, `cover`, `fmt` and the rest — refuses its configuration with the error [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) defines for a bad `members` line, which exits `2` and stays text under `--format json` ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)); like every config error it carries no code, so no selector reaches it ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). Only `grund init` does not refuse: when expanding the workspace above its target fails, it leaves its Workspace members section out and the error to `check`, and exits as it otherwise would ([§FS-init.2.3.4.15](FS-init.md#23415-workspace-members)). Decided in [§DF-absorbed-scan-warning](../decisions/functional/DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error).

```
error: grund.toml:16: [workspace] members swallows this project's whole scan — every scan root is inside a member: `docs` in `docs` — so its declarations are unreachable and its citations are never checked. Point [scan] include at a directory that is not a member, or set include_root = false; this became an error in grund 0.16.0
```

The message names each covered root beside the member entry it is inside, in config order. The entry is spelled as the config wrote it, and the root by its path under the block root, folded lexically on the spelling and never by reading the disk, so `./docs/` is `docs`, `docs/../src` is `src`, and a root that folds to the block root itself, such as `docs/..`, or `../group` in a block at `group/`, is `.` ([§FS-errors.4](FS-errors.md#4-determinism)).

The question is asked of every block in a nested tree against its own `members` line, and of the run's root block before its nested members expand, so where a later expansion refusal would also apply, this is the error reported and the run stops there, [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) reporting the first problem only. Every line is rendered from the root the run was launched at, for the blocks above it, at it and below it alike ([§FS-errors.3](FS-errors.md#3-message-text)).

## 4. Warnings

**`4.8` is vacant.** The unlisted-`[workspace]` rule held that number until the ramp of [§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived) ended and the finding became an error; it is [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) now, under the heading its verdict belongs to. Its siblings kept the numbers they had rather than closing the gap: a section number is an address inside this repository and not a promise outside it, so a vacancy costs a reader this sentence while renumbering `4.9` to `4.14` would have moved 371 citations in 152 files to say the same thing.

### 4.1 Unused declaration

An ID that is declared but never cited, reported as `unused`. Newly declared IDs may not yet have citations, which is why the code's row does not make it an error. A resolving shorthand counts as a citation ([§FS-check.4.1.1](FS-check.md#411-a-resolving-shorthand-counts)), a kind's own index entry does not ([§FS-check.4.1.2](FS-check.md#412-an-index-entry-does-not-count)), and `E2E` declarations are exempt ([§FS-check.4.1.3](FS-check.md#413-e2e-declarations-are-exempt)). A citation counts wherever the run read it, which under an explicit path is the whole project ([§FS-check.4.1.4](FS-check.md#414-a-citation-anywhere-in-the-resolution-scope-counts)).

#### 4.1.1 A resolving shorthand counts

A number-only shorthand citation that resolves counts here like any other citation ([§FS-check.1.2.4](FS-check.md#124-a-resolved-shorthand-is-a-real-edge)): a declaration abbreviated as `<§>FS-042` everywhere is cited, and reporting it as unused would state the opposite of the truth.

#### 4.1.2 An index entry does not count

A citation that is a kind's own **index entry** ([§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is)) does not count here. An index names every declaration in its folder by construction, so counting its entries would leave every ID in an indexed folder permanently cited and delete the signal this warning exists to give ([§DF-index-not-an-inbound-citation](../decisions/functional/DF-index-not-an-inbound-citation.md#df-index-not-an-inbound-citation-an-index-entry-is-navigation-not-use)). The exclusion is exactly the entry: a citation in an index file of an ID whose home lies *outside* that folder is an ordinary citation and counts like any other **unless that exact site is the canonical link that enrolls an external source declaration** ([§FS-check.3.18.3](FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link)). Other citations of the enrolled ID on the same page still count. The exclusion leaves every index entry in `grund refs` — they are real citations, and a reader asking who points at an ID wants to be told that its index does. When an unused declaration has citations excluded by this rule, retain `declared but never cited: <ID>` as a contiguous message prefix and append an explanation: identify the excluded index entry by path and line and say that it does not count as a use. With multiple excluded sites, order them by path, line, then column; either locate every site in that order or locate the first and give the remaining number as `N more`. Embedded paths use the report's base ([§FS-errors.3](FS-errors.md#3-message-text)), including when relative paths are disabled. A declaration with no citations keeps its existing message unchanged. This is a human-readable message append ([§FS-errors.3](FS-errors.md#3-message-text)), including in JSON and library findings that carry that message. Diagnostic code, severity, declaration location, record fields, use accounting and exit status stay unchanged. The same excluded sites are labeled in text `refs` ([§FS-refs.3.1.1](FS-refs.md#311-label-the-excluded-index-sites)); the section and binding exceptions remain [§FS-check.4.1.2.1](FS-check.md#4121-a-section-citation-or-a-value-binding-in-the-index-is-not-the-entry)'s.

##### 4.1.2.1 A section citation or a value binding in the index is not the entry

For an ID under the index's own folder, the entry is a citation of the declaration itself ([§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is)), so only such a citation is excluded. A citation of the same ID in the same index that names a section of it (such as `DOC-offer.values.price`), or that is the citation inside a recognized value binding ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts)) — including one aimed at a whole value root that names no section ([§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)) — or inside an attempt at one `check` refuses ([§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts)), is not the entry and counts as use, here and under `grund list --unused`. An index that keeps a table binding its folder's values therefore leaves those declarations cited, exactly as the same table on any other page would. A further plain citation of the declaration itself in its own index is still navigation ([§DF-index-not-an-inbound-citation.2.2](../decisions/functional/DF-index-not-an-inbound-citation.md#22-the-exclusion-is-the-entry-not-the-file)), unless it rides on a declaration heading: there it is a citation from the declaration that heading declares, never the entry, so it counts as use here and under `grund list --unused`, and `refs` does not label it ([§FS-citations.recognized.title-lines](FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names)).

#### 4.1.3 `E2E` declarations are exempt

`E2E` declarations ([§FS-declarations.e2e-case](FS-declarations.md#e2e-case-what-an-e2e-case-directory-declares)) are exempt: an end-to-end case is exercised by being run, not by being cited, so a `§E2E-<name>` that nothing cites is not a warning. Every other kind is subject to this rule, including Markdown and value-source declarations ([§FS-values.terms](FS-values.md#terms-terms)); the citation inside a recognized value binding counts as a use ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts)). `grund list --unused` reads the same signal, and what it does with a configured `E2E` kind's cases is [§FS-list.1.3.1](FS-list.md#131-a-configured-e2e-kind-is-left-out-unless-selected)'s.

#### 4.1.4 A citation anywhere in the resolution scope counts

A citation counts wherever the run read it, so a declaration cited only from a file outside an explicit path is cited and earns no warning: a path-scoped run's `declared but never cited` warnings are exactly the whole-project run's ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)).

This is not the `--full` case of [§FS-check.1.3.5](FS-check.md#135-the-unused-declaration-warning-is-unchanged-out-there), which leaves the warning standing, and the difference is a contract rather than a preference. `--full` owes additivity ([§FS-check.1.3.4](FS-check.md#134-purely-additive)) and so may never retire an in-scope finding. A path scope owes nothing of the kind: it is a subset run by construction, and it counts an edge **exactly where `grund check .` counts it** and never anywhere else, so it cannot disagree with the run additivity protects. [§FS-check.1.3.5](FS-check.md#135-the-unused-declaration-warning-is-unchanged-out-there)'s own remedy — widen `include` so the citing file is governed, and the edge counts everywhere — is what a path scope already reads.

### 4.2 Inline note soft-cap overrun *(opt-in)*

Off by default. When `[reference] warn_on_suggested = true` is set in the project's `grund.toml` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), a note whose line count exceeds `inline_note_suggested_lines` but stays within `inline_note_max_lines` is reported as a warning. The full contract — what counts as a soft-cap overrun and how it interacts with the hard-cap error in [§FS-check.3.10](FS-check.md#310-inline-citation-style-violation) — lives in [§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap). Off by default because the soft cap is primarily agent-facing guidance ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)); flipping the toggle escalates it to a `check`-time signal.

### 4.3 Redundant config pair

A directory that carries both a bare `grund.toml` and `.agents/grund.toml` ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)). The bare file is the config; the `.agents/` one is read by nothing, so a user who edits it changes nothing and is told so, in a CLI-level warning ([§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning)) that `grund config validate` and `grund config show` also emit ([§FS-check.4.3.2](FS-check.md#432-reported-by-check-config-validate-and-config-show)):

```
warning: .agents/grund.toml is ignored — grund.toml takes precedence; delete one
```

#### 4.3.1 A CLI-level warning

It is a CLI-level `warning:` on **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), not a per-finding line: it is about which file the run read, not a finding at a site in the citation graph, and there is no offending line to point at — the whole file is ignored. Both paths are report paths and follow `[output] relative_paths` ([§FS-config.3.6](FS-config.md#36-output--report-format)) — relative to the config root by default — so the message names the two files a user has to choose between. Like every warning it leaves the exit code alone ([§FS-check.2](FS-check.md#2-outputs)), because the pair is the ordinary transient state of a migration between the two forms ([§DF-config-file-location.2.2](../decisions/functional/DF-config-file-location.md#22-the-bare-grundtoml-wins-a-tie-and-check-warns-about-the-pair)).

#### 4.3.2 Reported by `check`, `config validate` and `config show`

The same warning is emitted by `grund config validate` and `grund config show` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path), [§FS-config.4.2](FS-config.md#42-grund-config-show-path)) — those are the surfaces a user reaches for when the answer to "why is my config not taking effect" is that `grund` is reading the other file. No other command reports it: a redundant pair is a fact about the repository's configuration, and `show`, `list`, `refs`, `cover`, and `fmt` answer questions about its content.

#### 4.3.3 The deprecated-location sibling

[§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location) is this finding's sibling and inherits every sentence of this section: it fires where the run *read* the `.agents/` file rather than ignored it, which is the case this one cannot be in.

### 4.4 Inline note layout deviation *(opt-in)*

Off by default. When `[reference] inline_note_layout` names a layout and `[reference] inline_note_layout_check = "warn"` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), every line of a note that [§FS-inline-citation-style.3.3.1](FS-inline-citation-style.md#331-per-line-not-per-site--and-only-where-a-note-opens) judges and that does not match the configured form is reported as a warning, located at that line. Setting the key to `"error"` reports the identical message as an error instead ([§FS-check.3.10](FS-check.md#310-inline-citation-style-violation)); `"off"`, or a `inline_note_layout` left at `any`, reports nothing. The form itself, the per-line rule, and the exemption for sites that carry no note live in [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit), and the channel table in [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations).

Off by default because a layout is a house style rather than a correctness property, and the two severities exist so a repository can migrate on `warn` before it gates on `error` — the same ladder [§FS-check.4.2](FS-check.md#42-inline-note-soft-cap-overrun-opt-in) gives the soft cap.

### 4.5 Nothing recognized

A scan that read at least one file and recognized **nothing in it** — no declaration and no citation — is [§FS-check.2.2](FS-check.md#22-empty-scan)'s empty scan one step further in: the scope was right and the files were read, and the grammar matched none of their content. `check` then emits one CLI-level `warning:` line ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) on **stderr**, whose text is [§FS-check.4.5.2](FS-check.md#452-the-message). It has two usual causes ([§FS-check.4.5.1](FS-check.md#451-the-usual-causes)), is asked per project ([§FS-check.4.5.3](FS-check.md#453-asked-per-project)) and only of a run over that project's root ([§FS-check.4.5.4](FS-check.md#454-asked-only-of-the-whole-project)), and is withheld beside any other finding about the scope ([§FS-check.4.5.5](FS-check.md#455-a-warning-withheld-beside-any-other-finding-about-the-scope)); the per-heading half is [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)'s ([§FS-check.4.5.6](FS-check.md#456-the-per-heading-half)). Decided in [§DF-nothing-recognized](../decisions/functional/DF-nothing-recognized.md#df-nothing-recognized-a-run-that-recognized-nothing-says-so-and-says-it-as-a-warning).

- **Code:** `nothing-recognized` ([§FS-errors.5](FS-errors.md#5-json-format)), with `path` and `line` null like every CLI-level finding.

#### 4.5.1 The usual causes

The usual causes are a tree nobody has declared in yet and a docs tree whose headings open with a prefix that is no configured kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), either of which leaves every heading in the tree a non-declaration and the run's verdict `success` over a repository where nothing is grounded.

#### 4.5.2 The message

Name the file count, global declaration/citation shapes and configured kinds. Labels follow [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure): `[id] format` under v1, `[schema] id_format` under v2. Kind overrides may differ. With no kinds ([§FS-citations.recognized.no-kinds](FS-citations.md#recognizedno-kinds-with-no-kind-no-token-is-a-citation)), say: ``nothing recognized — grund read 3 files, and this project declares no kinds, so no heading in them is a declaration and no token a citation. Declare its first kind in grund.toml to start.`` Follow that instruction with the shell-ready command that explains a kind's keys in the project's configuration version: `grund config explain '[schema.kinds.<NAME>]'` under v2, `grund config explain '[[kinds]]'` under v1. The named command answers even when no kind is declared and teaches the keys for its declaration places ([§FS-config-explain.1.2](FS-config-explain.md#12-rungs)). This adds help to the same `nothing-recognized` warning; its conditions, severity, structured fields and exit remain unchanged.

Otherwise the v1 text is:

```
warning: nothing recognized — grund read 3 files and found no declaration and no citation in them. A declaration heading reads `# <KIND>-<NNN>-<slug>: <title>` and a citation `<marker><KIND>-<NNN>-<slug>`, under [id] format = "{kind}-{number}-{slug}" with <KIND> one of {AR, FS}. Either nothing is declared yet, or the headings are written to a different shape than that.
```

Render shapes from the global template using [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)'s substitution and configured marker. Keep both explanations: an unused scaffold and another grammar look identical here. Never assemble an example ID from number/slug patterns or guess a corrected ID.

#### 4.5.3 Asked per project

The question is asked **per project**, like [§FS-check.2.2](FS-check.md#22-empty-scan): in a workspace ([§FS-workspace.5](FS-workspace.md#5-command-scope)) each project is judged against its own config, since one project's grammar mismatch says nothing about another's. It asks *recognized*, not *declared* — a member that only cites declarations from another member declares nothing and is working as intended, so a citation anywhere in the project answers the question.

#### 4.5.4 Asked only of the whole project

It is asked only of a run whose scope **is** that project's root (no path argument, or a path that resolves to it). A narrowed `grund check <dir>` is a slice the caller chose, and a slice holding no declaration and no citation is an answer rather than a misconfiguration — the claim this caution makes is about a whole project, and a run that read part of one cannot make it.

#### 4.5.5 A warning, withheld beside any other finding about the scope

Like [§FS-check.2.2](FS-check.md#22-empty-scan) it is a warning, and like [§FS-check.2.2](FS-check.md#22-empty-scan) it is withheld from a run that has any other finding about the default scope: the exit code stays `0` (a tree with nothing in it yet is the ordinary first day of a repository), and a report that already says something about that scope is not the silent verdict this rule exists to break. It inherits every exception [§FS-check.2.2.3](FS-check.md#223-exit-code-json-and-what-suppresses-it) lists unchanged, and for the same reasons. A redundant-config pair ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)) is a fact about which file was read — and a repository mid-migration between the two config names is exactly where a mismatched `[id] format` hides, in the file that is no longer read. The out-of-scope finding ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)) is a fact about the tree beyond the scope, and a `--full` run that reports every citation out there while the default scope holds nothing is the strongest form of this diagnosis, not a reason to withhold half of it. What it buys is the `success` line — a warning stands in its place ([§FS-check.2.1](FS-check.md#21-report-format)), so the run that recognized nothing stops printing the same word as the run that checked everything.

#### 4.5.6 The per-heading half

The per-heading half — naming each heading that looks like a declaration and does not match — is [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss), a different rule asking a different question: this one is arithmetic over what the scan recorded, that one is about what a single line came close to being. Where both could speak, [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss) does and this one is withheld under the rule above, because "these two headings, at these lines" is the same fact said usefully.

### 4.6 Declaration near miss

Moved to [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss). This address is kept so citations written before the move still resolve.

### 4.7 A workspace member swallows the block's own scan

Moved to [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) when the warning became an error in grund 0.16.0. This address is kept so citations written before the move still resolve.

#### 4.7.9 A later expansion refusal does not discard it

Retired in grund 0.16.0: the absorbed scan is now a config error, so when a later expansion refusal would also apply, the run stops at the absorbed scan and reports only it ([§FS-check.3.30.2](FS-check.md#3302-every-command-that-loads-the-workspace-refuses)). This address is kept so citations written before the change still resolve.

### 4.9 A workspace member declared optional is absent

A member listed in `[workspace] optional_members` whose path is not a directory in this checkout ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)). What it would have contributed was not read: no declaration in it reached a catalog, and no citation into it was resolved or reported ([§FS-workspace.4](FS-workspace.md#4-resolution)). The run is a report about less of the repository than it looks like, and saying so is this finding's whole job.

It is one located warning per absent entry ([§FS-check.4.9.1](FS-check.md#491-one-warning-per-absent-entry)), naming the entry and its member ([§FS-check.4.9.2](FS-check.md#492-the-entry-as-written-the-member-by-its-alias-path)) and no remedy ([§FS-check.4.9.3](FS-check.md#493-it-names-no-remedy-because-nothing-here-is-broken)). Unlike [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) it is printed on stdout ([§FS-check.4.9.4](FS-check.md#494-a-located-finding-on-stdout-not-a-cli-level-caution)), because the exit code cannot carry it ([§FS-check.4.9.5](FS-check.md#495-the-exit-code-forces-the-shape)); it names no release ([§FS-check.4.9.6](FS-check.md#496-it-names-no-release)), and its JSON form is [§FS-check.4.9.7](FS-check.md#497-its-json-form-is-on-stdout).

- **Code:** `optional-member-absent` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### 4.9.1 One warning per absent entry

One warning per absent entry, in the report's ordinary sort ([§FS-errors.4](FS-errors.md#4-determinism)) — by entry name, since all of them share one path and line — located at the `optional_members` line of the block that holds it:

```
grund.toml:5: optional workspace member `vendored` is absent — citations into namespace `vendored` were not checked, so this run does not cover it
```

Where the run verified at least one managed block ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)), the warning ends in one more clause, because that run leaves the blocks' workspace sections out of the comparison ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)):

```
grund.toml:9: optional workspace member `packages/secret` is absent — citations into namespace `secret` were not checked, so this run does not cover it, nor the managed blocks' workspace sections
```

A run that verified no managed block keeps the shorter form: the clause would describe blocks that are not there. The clause names no remedy either ([§FS-check.4.9.3](FS-check.md#493-it-names-no-remedy-because-nothing-here-is-broken)): it says what the run did not cover, as the sentence before it does.

#### 4.9.2 The entry as written, the member by its alias path

The entry is named **as the config wrote it** and the member by the **whole alias path this run spells it with** — `vendored` and `sub/vendored` for one entry inside a nested block — which is the pair [§FS-check.3.30.1](FS-check.md#3301-the-message) and [§FS-check.3.29.7](FS-check.md#3297-the-location-sits-in-the-prefix-where-the-finding-is-located-and-inside-the-text-where-it-is-not) already use, for the same two reasons: the entry is what an author can edit ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), and the alias path is what a citation has to write ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)).

#### 4.9.3 It names no remedy, because nothing here is broken

Most warnings in [§FS-check.4](FS-check.md#4-warnings) end in an edit, because they report a configuration that says something its author did not mean. This one reports a state the author declared in advance and a checkout that happens to be partial; the only thing that would "fix" it is a checkout with the member in it, which is not grund's to ask for and is often not available where the run happens. So the message stops at the fact.

#### 4.9.4 A located finding on stdout, not a CLI-level caution

It is a located finding on stdout, which departs deliberately from its two nearest neighbours — [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) both point at a line of their block's config, and [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) prints as a CLI-level `error:` on stderr ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) — as [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) still does on the five scanning surfaces that have no report to put it in ([§FS-check.3.29.9](FS-check.md#3299-every-command-that-scans-reports-it-not-check-alone)), though in `check` it is an error now ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)). Those two report a **misconfiguration**, and the reason for that CLI-level form is that every command in the tree is wrong under them: a block that reads nothing makes `grund list` silent, an unlisted block makes every command spell the same project two ways. So both are carried by every command that scans — [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) refusing where the workspace loads, [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) where the scan meets the block ([§FS-check.3.29.10](FS-check.md#32910-knowable-only-from-the-scan)). This finding is not that. Nothing is misconfigured — the repository said this may happen, and it happened — and no other command's output is wrong, because a member that is not there has nothing to list and nothing to point at. What is at stake is only the verdict `check` renders, and a statement about the coverage of a report belongs in the report.

#### 4.9.5 The exit code forces the shape

[§FS-check.2](FS-check.md#2-outputs) gives grund one way to say "do not trust this report as complete", and it is exit `2`. This is the one case where a run is deliberately incomplete and still exits `0` ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)), so the exit code carries nothing here and stdout has to. A CLI-level line would leave stdout saying only that the `success` line was withheld — a signal made of an absence, which under `2>/dev/null` or a folded CI log is indistinguishable from silence, and which is exactly the warning that scrolls past. The `<path>:<line>:` prefix is also honest here in a way it is not for [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning): there is one line, in a file the repository wrote, and it is the line a reader has to open to understand the run. `success` is withheld all the same, because it is withheld for every warning ([§FS-check.2.1.3](FS-check.md#213-the-success-line)) — but that is now a consequence of the finding rather than the whole of it.

#### 4.9.6 It names no release

[§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) were warnings on the way to being errors ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), and both have become one ([§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived)). This one is permanent. An unverified member is not a state to be migrated off; it is the standing price of the opt-out, paid on every run of every checkout that takes it, and a repository that stops wanting to pay it deletes the entry.

#### 4.9.7 Its JSON form is on stdout

Under `--format=json` it is one warning finding on **stdout** with `path` and `line` set and `sites` null, like every other located finding ([§FS-errors.5](FS-errors.md#5-json-format)) — which is the other half of what the CLI-level shape would have cost, since a consumer filtering the report for coverage facts would have had to parse text on a second stream to find this one.

### 4.10 `include_root = false` leaves the block's own files unread

A `[workspace]` block that sets `include_root = false` is not a project, and its own files are read by nobody ([§FS-check.4.10.1](FS-check.md#4101-nobody-reads-the-blocks-own-files)): a declaration there reaches no catalog and a citation there is never checked, which is [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) failing through one config key. `grund` emits one CLI-level `warning:` ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) on **stderr** for such a block **whose own tree actually holds a file a scan would have read**. Decided in [§DF-unread-opted-out-block](../decisions/functional/DF-unread-opted-out-block.md#df-unread-opted-out-block-the-unread-files-of-an-opted-out-block-are-a-conditional-warning-that-never-ramps).

What counts is [§FS-check.4.10.2](FS-check.md#4102-what-counts), and [§FS-check.4.10.3](FS-check.md#4103-a-block-with-no-member-in-scope-is-not-this-finding) and [§FS-check.4.10.4](FS-check.md#4104-what-stays-silent-and-why-the-silence-is-the-point) are what does not. The message is [§FS-check.4.10.5](FS-check.md#4105-the-message), and it names one root ([§FS-check.4.10.6](FS-check.md#4106-one-root-not-every-root)). Every command that scans says it, once per block ([§FS-check.4.10.7](FS-check.md#4107-every-command-that-scans-says-it-and-each-block-says-it-once)), except over a failed workspace expansion ([§FS-check.4.10.8](FS-check.md#4108-a-failed-workspace-expansion-withholds-it)). It keeps its text under `--format json` ([§FS-check.4.10.9](FS-check.md#4109-a-launch-time-message-keeps-its-text-under---format-json)), stands in place of `success` without moving the exit code ([§FS-check.4.10.10](FS-check.md#41010-it-stands-in-place-of-the-success-line-and-never-moves-the-exit-code)), reaches every frontend ([§FS-check.4.10.11](FS-check.md#41011-one-of-the-runs-warnings-rendered-by-every-frontend)), and never becomes an error ([§FS-check.4.10.12](FS-check.md#41012-a-warning-permanently-with-no-release-it-becomes-an-error-in)).

#### 4.10.1 Nobody reads the block's own files

An intermediate opted-out block is still a segment in every alias path below it ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)), yet its own files are read by nobody: it has no scan of its own, an enclosing scan, where there is one, stops at the member boundary ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), and `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)) widens a project's scope and has no project to widen here. No run said so: over a grouping directory holding two dangling citations, `check`, `check --full` and `list` were all silent and exited `0` ([grund#71](https://github.com/agent-grounds/grund/issues/71)).

#### 4.10.2 What counts

**What counts is one question: would this block have read something, had it been a project?** That is the mirror of [§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan), which asks whether a block that *is* a project reads nothing — so one notion of "this block's own scope" answers both, and a `[[kinds]]` home or an unscanned home moves the two rules together. Take the block's **default scope** — the roots `[scan] include` and the scanned `[[kinds]]` homes give it ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), the set [§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)'s boundary prunes, asked of the default scope whatever `--full` says, because this is a property of the configuration and not of one scan. Drop a root that is not on disk: the scan skips it before it prunes, so it is read by nobody and costs nobody anything. Drop a root at or inside one of the block's own expanded member roots: those files *are* read, by the member — compared as canonical paths, the way the scan's own prune compares them. On what is left, probe for one file the scan would have read, under the block's own `[scan] extensions`, `exclude`, ignore files and hidden-name rules applied exactly as the scanner applies them ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)). The probe **stops at the first hit**: the cost is "is there one file here", not the size of the tree, and only a block that opted out ever pays it ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).

#### 4.10.3 A block with no member in scope is not this finding

`include_root = false` with no members — an absent `members` key, an empty list, or a non-empty list of globs that match no directories, beside no `optional_members` entry ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)) — is already a config error at that block's own line ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)). A configuration the run refuses is not one it also cautions about, and the caution's two remedies are not the repair that block needs.

#### 4.10.4 What stays silent, and why the silence is the point

Four shapes are deliberately not this finding, and each is a correct configuration that [§FS-check.4.10.2](FS-check.md#4102-what-counts) answers "no" for. A grouping directory that only groups — `grund.toml` and its members and nothing else — has no root on disk to read. A block whose own roots are all inside its members is [§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan)'s shape read from the other side. A tree whose only files are of unscanned types, or excluded, gitignored, or hidden, is a tree the block would not have read as a project either. And a root the config names that is not on disk rescues nothing. Firing on any of them would be a warning with no edit that clears it, which is permanent output tools learn to filter — the outcome [§DF-absorbed-scan-warning](../decisions/functional/DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error) already rejected once for a neighbouring finding.

#### 4.10.5 The message

Name the config line to open, unread tree, cost and two remedies. Use the same source-scope remedy for both config versions: the other project's version may differ.

```
warning: group/grund.toml:17: no project scans `docs`, so its citations are never checked. Set include_root = true, or configure another project's source scope to read it.
```

Never prescribe `[scan] include` for another project. Anchor at the block's `include_root` line, falling back to `[workspace]` if absent (unreachable with the default `true`). Render the location from this run's root ([§FS-errors.3](FS-errors.md#3-message-text)), and the unread tree as a normalized path under the block root, never a resolved path ([§FS-check.3.30.1](FS-check.md#3301-the-message)). When the unread tree is the block root itself — a grouping block keeping a README of its own beside its members — that path is `.`, never an empty span, whatever directory the run starts from: the path is relative to the block root, not to the run's root.

#### 4.10.6 One root, not every root

[§FS-check.3.30.1](FS-check.md#3301-the-message) lists every covered root because its claim is universal — *every* one is inside a member — and the list is the evidence for it. This claim is existential: one unread root is the whole finding, one edit clears all of them, and probing the rest would buy nothing the answer depends on. The one named is the first in scope order, `[scan] include` in config order and then the `[[kinds]]` homes, which is fixed; the first *file* under it is whatever the filesystem handed back, which is not, and is therefore never named ([§FS-errors.4](FS-errors.md#4-determinism)).

#### 4.10.7 Every command that scans says it, and each block says it once

The question is asked where a run populates a block's member boundary — the block the run is rooted at, and each block below it — so `check`, [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk) and every other command that resolves that boundary carry it, on exactly the surfaces that load the workspace ([§FS-check.3.30.2](FS-check.md#3302-every-command-that-loads-the-workspace-refuses)). The reproduction in the ticket used `check` and `list`, which is the whole argument: a silent-scope defect only `check` reports is half-reported. It is **once per block per run** — a workspace-wide run that expands the same block a second time still says it once — and a tree with two opted-out blocks earns one line each, the block the run is rooted at first and the blocks below it after, in the order the run reaches them. A run narrowed inside a member never populates the enclosing block's boundary, so it stays silent about a block it is not reading through.

#### 4.10.8 A failed workspace expansion withholds it

A run whose **workspace expansion fails** does not carry this finding for the blocks below its root, a silence declared and bounded ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)) rather than a gap: the answer depends on where the run's other projects are, and a failed expansion is exactly the case where that list was never produced, so the alternative is not an earlier warning but a wrong one. The run is exiting `2` on a config error the reader has to repair before anything else it says is worth reading, and the finding returns on the next run. [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) is asked of the run's root block before that expansion, because what it asks is answered by the block's own members alone, so where it applies it is the error the run stops at ([§FS-check.3.30.2](FS-check.md#3302-every-command-that-loads-the-workspace-refuses)).

#### 4.10.9 A launch-time message keeps its text under `--format json`

It is a launch-time message, so it keeps its text under `--format json` ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)) and carries no JSON `code` and no selector of its own ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). That is the rendered shape of every launch-time message, [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)'s error included ([§FS-check.3.30.5](FS-check.md#3305-it-stays-text-under---format-json)), rather than [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)'s, and the difference between them is *when the fact exists*. [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)'s is knowable only from the scan that meets a nested config ([§FS-check.3.29.10](FS-check.md#32910-knowable-only-from-the-scan)), so it is one of `check`'s report errors and renders as a JSON finding with `path` and `line` set and `sites` null ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)). This one is settled at boundary population, before a scan and before a report exists, and is carried by every command that resolves that boundary ([§FS-check.4.10.7](FS-check.md#4107-every-command-that-scans-says-it-and-each-block-says-it-once)), not by `check` alone — so the shape it renders in is fixed here rather than borrowed from `check`'s, and giving `check` a different one would make the text a consumer greps for depend on which command produced it.

#### 4.10.10 It stands in place of the `success` line, and never moves the exit code

[§FS-check.2.1.3](FS-check.md#213-the-success-line) is unchanged: a run with a warning prints the warning rather than `success`. This is the first `[workspace]` caution that has to say so out loud, because it is the first that can fire on an otherwise clean run — [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)'s block earned the empty-scan caution beside it while it was a warning ([§FS-check.2.2](FS-check.md#22-empty-scan)) and stops the run now ([§FS-check.3.30.4](FS-check.md#3304-no-empty-scan-caution-follows-it)), and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)'s is in the report already, as an error ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)). The exit code stays where it was ([§FS-check.2](FS-check.md#2-outputs)): opting out is a legitimate choice, and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization) fixes what a warning means.

#### 4.10.11 One of the run's warnings, rendered by every frontend

The fact is settled before any report exists, but what the engine hands back is a finding in the run's warning channel — carried on whatever the scanning command returns and rendered by whichever frontend asked ([§FS-distribution.3.1](FS-distribution.md#31-rust-grund-core-crate)) — rather than a line the engine wrote to a stream. It **is located at the block's `include_root` line**, falling back to its `[workspace]` line exactly as the breadcrumb of [§FS-check.4.10.5](FS-check.md#4105-the-message) does, and carries that anchor as the finding's own location so a frontend never parses the message for one. Every byte is the byte it is today, on stderr and under `--format json` alike, and the `success` line it stands in place of is unaffected. The editor is the surface that gains a reader: it publishes the warning on that `include_root` line ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)), which is where the one edit that clears it is written.

#### 4.10.12 A warning permanently, with no release it becomes an error in

This is one of the two `[workspace]` findings that do not ramp, beside [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) ([§FS-check.4.9.6](FS-check.md#496-it-names-no-release)), and the difference is not how wrong the repository is. A finding is eligible to become an error only when **every repository in that state is wrong** *and* **the configuration has a way to say "I meant it"** — the rule for all four, argued in [§DF-unread-opted-out-block.2.3](../decisions/functional/DF-unread-opted-out-block.md#23-what-makes-a-workspace-finding-ramp-and-what-makes-one-permanent). [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) and [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) pass both: a block that claims to be a project and reads nothing, and a block whose projects are spelled two ways, are wrong on every reading of them, and each has an edit that records the intent instead — `include_root = false` for one, listing the block for the other — so an error is a verdict their author can act on before it lands, and [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path has got both there ([§FS-check.3.29.14](FS-check.md#32914-an-error-because-the-deadline-the-warning-named-has-arrived), [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)). [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) fails the first: an absent optional member is a state its repository declared in advance ([§FS-check.4.9.3](FS-check.md#493-it-names-no-remedy-because-nothing-here-is-broken)). This finding passes neither. A grouping directory holding a README nobody needs checked is doing exactly what its author meant, and neither remedy records that: making the block a project and pointing another project's `[scan] include` at it both change what the repository *is*. That the finding is a property of the configuration **and** the tree — the same `grund.toml` silent on Monday and reportable on Tuesday because somebody added `group/docs/notes.md` — is the symptom that makes it recognizable, not the reason: [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index) depends on the tree in the same way and ramped anyway. There is therefore no version constant, no roadmap milestone, and no clause in the message naming a release.

### 4.11 Config read from the deprecated `.agents/` location

The config this run read is an `.agents/grund.toml` ([§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated)). The file is still read and still governs the project — the location is deprecated, not withdrawn — and the run says so, naming the file it read and the bare `grund.toml` beside it that should hold it instead:

```
warning: .agents/grund.toml is a deprecated config location — move it to grund.toml
```

It is [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger ([§FS-check.4.11.1](FS-check.md#4111-fs-check43s-finding-in-everything-but-its-trigger)), on the same three surfaces and no more ([§FS-check.4.11.2](FS-check.md#4112-the-same-three-surfaces-as-fs-check43-and-no-more)), once per config ([§FS-check.4.11.3](FS-check.md#4113-once-per-config-not-once-per-scope)), and never beside [§FS-check.4.3](FS-check.md#43-redundant-config-pair) about one directory ([§FS-check.4.11.4](FS-check.md#4114-a-directory-carrying-both-names-earns-fs-check43-and-not-this)). Its JSON form is [§FS-check.4.11.5](FS-check.md#4115-under---formatjson-one-warning-finding-on-stderr). It stands in place of the `success` line ([§FS-check.4.11.6](FS-check.md#4116-it-stands-in-place-of-the-success-line)) and never becomes an error ([§FS-check.4.11.7](FS-check.md#4117-a-warning-permanently-with-no-release-it-becomes-an-error-in)).

#### 4.11.1 [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger

This is [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger, and for [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning)'s reason: it is a fact about *which file the run read*, not about that file's content and not about a site in the citation graph. So it is a CLI-level `warning:` on **stderr** ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) and never a per-finding line — there is no offending line to point at, the whole file is the subject. Both paths are report paths and follow `[output] relative_paths` as [§FS-check.4.3.1](FS-check.md#431-a-cli-level-warning)'s do ([§FS-config.3.6](FS-config.md#36-output--report-format)), so the message names the move as the `git mv` a reader can type from the report's base. Like every warning it leaves the exit code alone ([§FS-check.2](FS-check.md#2-outputs)), because the location it names is a supported one and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization) fixes what a warning means.

#### 4.11.2 The same three surfaces as [§FS-check.4.3](FS-check.md#43-redundant-config-pair), and no more

`grund check`, `grund config validate` and `grund config show` ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path), [§FS-config.4.2](FS-config.md#42-grund-config-show-path)) carry it, byte-identically. `list`, `refs`, `cover`, `fmt` and a bare ID read stay silent, on [§FS-check.4.3.2](FS-check.md#432-reported-by-check-config-validate-and-config-show)'s argument unchanged. The silence is not an oversight to be widened later — a fact that never changes between runs, printed by every command, is permanent output tools learn to filter, and the surface a user reaches for to ask *where is my config* is `config show`.

#### 4.11.3 Once per config, not once per scope

A workspace names the root's config, each project the workspace expansion returns, and each block with `include_root = false` that the expansion reaches, at any depth and whether or not any of its members is present, each at the path that config was loaded under ([§FS-errors.4](FS-errors.md#4-determinism)) — `packages/beta/.agents/grund.toml`, moving to `packages/beta/grund.toml`. Such a block is no project ([§FS-workspace.2.1.2.2](FS-workspace.md#2122-include_root--false-has-nothing-to-lose)) and is named for its config alone, as a member is: expansion loads it to read its members, so the run read it. A `[workspace]` block that no enclosing block lists is outside the chain, never loaded, and never named ([§FS-workspace.6.1.8](FS-workspace.md#618-a-block-no-enclosing-block-lists-is-outside-the-chain)); an imported tree's blocks are named by that tree's own run, not by its consumer's ([§FS-imports.mount](FS-imports.md#mount-an-import-is-a-mounted-root-read-live-and-read-only)). Root and member are separate configs and a workspace may mix the two forms ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), so a member on the old path earns its own line while a member on the bare form earns none, and a root and a member both on the old path earn one line each. That set is what `grund check` and `grund config validate` name at a workspace root ([§FS-config.4.1.1](FS-config.md#411-at-a-workspace-root)); `grund config show` names only the config it prints, the root's there ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)), and a path inside a member names that member's alone on all three, nothing of a block above it.

#### 4.11.4 A directory carrying both names earns [§FS-check.4.3](FS-check.md#43-redundant-config-pair) and not this

The bare `grund.toml` won the tie ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)), so the config in force is on the home path already and nothing about it is deprecated; the `.agents/` file beside it is the one read by nothing, which is exactly what [§FS-check.4.3](FS-check.md#43-redundant-config-pair) reports. Emitting both would name one move twice and disagree about which of the two files is the problem.

#### 4.11.5 Under `--format=json`, one warning finding on stderr

Under `--format=json` it is one warning finding on stderr, carrying the `code` `deprecated-config-location`, with `path`, `line` and `sites` null ([§FS-errors.5](FS-errors.md#5-json-format)) — [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s shape, because it arrives the same way: one of the report's warnings, knowable from the config the run loaded rather than from the scan.

#### 4.11.6 It stands in place of the `success` line

It stands in place of the `success` line ([§FS-check.2.1.3](FS-check.md#213-the-success-line)), as every warning does, and that is the whole cost of the finding rather than a detail of it: a clean repository on the old path now prints this line where it printed `success`. [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) governs that as a verdict change and permits it, since the exit code does not move. This repository pays it in its own e2e corpus, which is why the fixtures that had no reason to be on `.agents/` are on the bare form and the ones that remain are the ones whose subject is discovery itself.

#### 4.11.7 A warning permanently, with no release it becomes an error in

The promise lives in [§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated), because it is a fact about the location rather than about this message: the fallback is never removed, so there is no version constant, no roadmap milestone, and no clause in the text naming a release.

### 4.12 Missing snapshot

A recognized citation has no declaration and targets a fetch-enabled kind whose effective `resolve` is `should` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)). It is a distinct fixed warning, not a softened dangling error:

```text
<path>:<line>: no snapshot for <qualified-ID> in <home> — run grund fetch <qualified-ID>
```

The em-dash remedy tail is exactly `— run grund fetch <qualified-ID>`, with no inner backticks, except at a home `fetch` refuses ([§FS-check.3.1.3.1](FS-check.md#3131-a-home-fetch-refuses)), for a key `fetch` has nothing to run for ([§FS-check.3.1.3.2](FS-check.md#3132-a-key-fetch-has-nothing-to-run-for)), and for an ID reached through an import or a nested import, where it is `— run grund fetch <ID below the last import edge> in <target of that edge>` ([§FS-imports.writes.answer](FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer)). The JSON code is `missing-snapshot`, severity is `warning`, and the message field is the text after `<path>:<line>: `. A warning-only run exits 0 and prints no `success` line. Every site gets one finding, remains in what `refs` and `cover` report, and may resolve after an explicit [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot), or, at a home `fetch` refuses, once the source declares it, or, for a key `fetch` has nothing to run for, once the key is repaired. The hints that take the fetch tail's place are [§FS-check.4.12.1](FS-check.md#4121-a-hint-takes-the-fetch-tails-place), and its workspace spelling is [§FS-check.4.12.2](FS-check.md#4122-in-a-workspace). A site whose home this run could not read earns neither this warning nor the error of [§FS-check.3.1.3](FS-check.md#313-a-kind-that-fetches), as [§FS-check.2.4](FS-check.md#24-an-incomplete-run) says.

Under v2 the same warning is selected by `[rules.resolution] <KIND> = "warn"`, and `must` selects the error; v1's `resolve = "should"` keeps selecting it. An explicit v2 entry needs `fetch` on the kind's row ([§FS-config-v2.rules.resolution](FS-config-v2.md#rulesresolution-rulesresolution)).


#### 4.12.1 A hint takes the fetch tail's place

The near-ID and escaped-inline-code hints of [§FS-check.3.1.1](FS-check.md#311-a-near-id) and [§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code) take precedence over the fetch action. The warning retains `no snapshot for <qualified-ID> in <home>` and substitutes the existing conditional `; did you mean …`, `; write <§>…`, or combined tail for the em-dash fetch tail. Declared-member hints apply only to dangling errors ([§FS-check.3.1.5](FS-check.md#315-an-unqualified-id-declared-in-a-loaded-member)). One site never produces both findings.

#### 4.12.2 In a workspace

In a workspace, the ID and remedy use the complete alias-qualified spelling while `<home>` is rendered from the run's report base ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)). The remedy for an ID reached through an import is the exception: the message's base keeps the complete spelling, and the remedy names the ID below the last import edge, in that edge's target ([§FS-imports.writes.answer](FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer)).

### 4.13 Oversized lead *(opt-in)*

Moved to [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in). This address is kept so citations written before the move still resolve.

### 4.14 Unmarked Markdown heading

Moved to [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading). This address is kept so citations written before the move still resolve.

## 5. What grund does not check

See [§FS-non-goals](FS-non-goals.md#fs-non-goals-what-grund-will-deliberately-not-do) — in particular [§FS-non-goals.1](FS-non-goals.md#1-markdown-link-validation) (markdown links / URLs), [§FS-non-goals.2](FS-non-goals.md#2-spelling-grammar-prose-quality) (spelling/grammar outside the explicit value form), and the convention that ID numbers are stable names, not ordinal positions. Value checking adds only the exact binding in [§FS-values.3.1](FS-values.md#31-the-only-binding-grammar): no surrounding-number inference, bare-literal lint, range/unit semantics, rendering, fingerprint/history check, or new reconciliation verb is performed ([§FS-values.9](FS-values.md#9-compatibility-and-explicit-exclusions)). The near misses that used to be listed here are [§FS-check.5.1](FS-check.md#51-the-near-misses-are-no-longer-here).

### 5.1 The near misses are no longer here

The declaration-side near miss is **no longer** in this section: a heading shaped like `# <KIND>-…: <title>` whose ID does not match the configured `[id] format` is reported per heading by [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss), and a tree in which every heading misses that way says so twice over — once per line, and once as the run that recognized nothing ([§FS-check.4.5](FS-check.md#45-nothing-recognized)). Neither guesses the corrected ID. The citation-side near miss — a `§`-marked token in the shorthand shape — left this section earlier, when [§FS-check.1.2](FS-check.md#12-the-number-only-shorthand) and [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) began recognizing and reporting it.

## 6. Watch mode (`--watch`)

`grund check --watch [<path>]` checks immediately, then stays resident and checks again when effective local inputs change. It is the terminal counterpart to [§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server), serving [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible). The approved lifecycle and output choices are recorded in [§DF-watch-terminal-loop](../decisions/functional/DF-watch-terminal-loop.md#df-watch-terminal-loop-a-terminal-watch-loop-preserves-ordinary-check-reports); delivery is tracked by [§RM-watch](../roadmap.md#rm-watch-implement-grund-check---watch).

How it notices a change is [§FS-check.6.1](FS-check.md#61-change-detection), what each run prints is [§FS-check.6.2](FS-check.md#62-each-run-is-a-plain-grund-check), how it ends is [§FS-check.6.3](FS-check.md#63-lifecycle), and which command takes the flag is [§FS-check.6.4](FS-check.md#64-scope).

### 6.1 Change detection

The watcher uses native `notify` filesystem notifications on Linux, macOS and Windows. Mounts and pseudo-filesystems that do not deliver usable notifications are unsupported. There is no polling fallback or configurable polling interval. Read/access events do not trigger checks.

#### 6.1.1 Subscribe before reading

Subscribe before the initial scan. Newly discovered inputs must be subscribed before they are read or probed for existence, including each candidate that can affect resolution. This ordering also holds in parallel workspace project scans. On refresh, add coverage, re-resolve under that coverage, then retire obsolete subscriptions. Events during discovery, subscription changes or scans must not disappear between a read and subscription; they arrange a subsequent check against the latest state.

#### 6.1.2 Bounded debounce and serialized work

While idle, run after 100 ms without a relevant event, or after 500 ms from the first pending relevant event, whichever comes first. A burst of writes or an atomic-save rename sequence within that window coalesces. Long saves can expose intermediate states; debounce is not a filesystem transaction.

During a scan/publication retain one pending rerun, rather than a queue entry per event. Scans and report publications are serialized, with no concurrent scans or interleaved reports. A pending rerun observes the latest inputs after the current run; continued writes cannot postpone idle work indefinitely.

#### 6.1.3 Effective input inventory

Observe every effective local input that can change the equivalent one-shot result, including paths outside the selected reporting subtree ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)):

- Both `grund.toml` and `.agents/grund.toml` at upward discovery and ancestor-claim candidates, including absent candidates and config precedence changes.
- Workspace roots, nested members, member-glob parents and absent member candidates; resolution-wide source roots and kind homes.
- Value sources ([§FS-values.terms](FS-values.md#terms-terms)), agent entrypoints, indexes, stub targets and other checker probes, and followed file-link targets outside source roots.
- Effective ancestor and nested `.gitignore`/`.ignore`, `.git/info/exclude`, global Git excludes, and the `.gitconfig`/XDG inputs through which those exclusions are discovered.

This is the shared discovery/checker's inventory, not a second set of resolution semantics. Re-resolve config, membership, ignores, catalogs and output defaults on each run. Narrowed checks retain resolution-wide coverage. Hidden discovery inputs such as `.agents/grund.toml` remain observed when hidden source directories are excluded.

Recursive subscriptions cover resolved source roots; ancestor/home coverage is shallow and path-filtered. Broad recursive roots may consume watches for ignored descendants. Missing or replaced inputs retain nearest-existing-parent anchors so creation, deletion, rename, atomic replacement, member additions/removals, and deleted/recreated roots refresh coverage rather than leaving dead subscriptions.

Lexical directory aliases retain their link-replacement anchors while sharing physical native coverage. Retiring an alias must preserve any coverage still required by another input, including recursive coverage; replacement or retargeting re-resolves that ownership under the anchors.

#### 6.1.4 Uncertain notifications and recovery

Lost events, overflow, unknown-path/uncertain events and internal queue saturation force full input rediscovery, rescan and subscription reconciliation. Replacement uses parent anchors to reattach. A coverage path that no longer exists by the time it is resolved or subscribed — deleted after the event or check that named it — is a missing input, not a failed subscription: its nearest existing parent anchor covers it, as [§FS-check.6.1.3](FS-check.md#613-effective-input-inventory) says, and a rescan follows so that its recreation is seen. Otherwise a failed setup/runtime subscription or failed watcher recovery is fatal under [§FS-check.6.3.2](FS-check.md#632-fatal-watcher-failures); the process must not silently remain resident with stale results. An ordinary input failure during that rescan remains recoverable under [§FS-check.6.3.1](FS-check.md#631-recoverable-runs) if usable discovery coverage can be retained.

### 6.2 Each run is a plain `grund check`

Each run preserves the ordinary check's stdout/stderr bytes, finding order and status on that tree state ([§FS-check.2](FS-check.md#2-outputs), [§FS-errors.4](FS-errors.md#4-determinism)), except the terminal controls in [§FS-check.6.2.2](FS-check.md#622-owned-terminal-screen). Reuse the ordinary checking, selection, sorting, rendering and status mapping. Every run preserves `--full`, grounding, suggestions, trial-rule selection, repeatable `--only`/`--ignore`, their precedence, and unhideable operational failures. Explicit `--format` overrides config defaults, including defaults changed while watching.

#### 6.2.1 Exact stream contract

Clean text prints the ordinary `success` marker ([§FS-check.2.1.3](FS-check.md#213-the-success-line)); JSON is ordinary finding NDJSON ([§FS-check.2.1.4](FS-check.md#214-json)). Clean JSON emits no record. Concatenated JSON therefore exposes neither clean runs nor every run boundary. No envelope, banner, timestamp or production run/completion record is added. Findings, warnings and operational errors keep their ordinary stream ownership. Redirected streams append the successive ordinary outputs.

#### 6.2.2 Owned terminal screen

Text owns an alternate screen only if stdout is an alternate-screen-capable terminal and stderr either shares that terminal or is redirected. Enter through stdout with `\x1b[?1049h`; immediately before each publication clear only that owned screen with `\x1b[H\x1b[2J`; restore through stdout with `\x1b[?1049l` on exit or before JSON publication. These spellings denote the corresponding escape bytes, not literal backslash text.

Shared-terminal stderr is cleared with the report; redirected stderr appends and receives no clearing bytes. Redirected stdout, distinct stdout/stderr terminals and dumb terminals append on both streams, with no enter/clear/restore bytes. JSON never enters or clears a screen. A text-to-JSON format change restores the owned screen before any JSON bytes; a later eligible text run can acquire it again. No stream changes owner. Exit restores pre-watch terminal content rather than retaining the last report. Fatal failure restores first, then prints its error.

### 6.3 Lifecycle

The process runs until interrupted or a fatal watcher failure. It is non-interactive: no TUI, key bindings or prompt ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)). It performs no network I/O ([§FS-non-goals.11](FS-non-goals.md#11-network-access-during-a-check)), launches no configured processes and writes no filesystem inputs. It may observe local metadata and parent anchors needed by [§FS-check.6.1.3](FS-check.md#613-effective-input-inventory), beyond files whose contents the scan reads.

#### 6.3.1 Recoverable runs

Ordinary findings and config/read failures publish the ordinary status `0`, `1` or `2` and remain resident for repair. Initial invalid config retains discovery and parent coverage. Later failed runs retain the last usable inventory plus discovery anchors until correction. Config-dependent trial-rule refusals are recoverable runs; static invocation errors terminate before entering the loop ([§FS-check.6.4](FS-check.md#64-scope)).

#### 6.3.2 Fatal watcher failures

Setup/runtime subscription failures and failed watcher recovery restore any owned screen, print an actionable stderr `error:` diagnostic identifying the failed watching operation/input and exit `2`. This status overrides any previously completed check status. Release subscriptions and join workers on fatal exit as on interruption.

#### 6.3.3 Interrupt and completion

On Ctrl-C/SIGINT discard pending work. Let an active synchronous scan finish privately and discard its unpublished result. Publication already begun finishes and counts as completed only when both streams have been fully published and flushed. Shutdown may therefore wait for an active scan/publication.

Return the most recently completed run's status (`0`/`1`/`2`). If none has completed, restore any owned screen and print `error: interrupted before the first check completed` on stderr, returning `2`. Restore the screen, release subscriptions and join workers before exit; no watcher or worker remains resident. Release is bounded: a native backend that has not released its subscriptions within 5 seconds of being closed, on refresh or at exit, is a fatal watcher failure under [§FS-check.6.3.2](FS-check.md#632-fatal-watcher-failures), never an indefinite wait. An unpublished interrupted result never changes the last completed status.

### 6.4 Scope

`--watch` is a `check` flag spelled as `grund check --watch [<path>]` ([§FS-cli](FS-cli.md#fs-cli-grunds-command-line-surface-conventions)). Other subcommands reject it. Parse and validate static invocation choices once, preserving ordinary validation diagnostics and their precedence; reuse those choices for every run. Config-dependent validity and output defaults are re-evaluated with the tree, rather than frozen at startup. There is no daemon/service protocol, public binding watch API, incremental engine or LSP dependency.

## checks: Checks

Each section below is one check named by its finding code, as [§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code) requires; its severity is its row in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog), so a promotion edits that cell and moves no coordinate.

### checks.glob-citation: Glob citation

A marked candidate that [§FS-check.1.1.11](FS-check.md#1111-a-glob-operator-never-shortens-a-marked-citation) reads as a pattern is reported once, located at its marker's line and column, with code `glob-citation` and this message:

```text
glob citation §<token>: a citation names one exact point, not a pattern; cite the point, or escape it as <§><token>; this became an error in grund 0.19.0
```

`<token>` is the candidate as written after the marker, and both `§` and `<§>` are spelled with the configured marker. Text output is `<path>:<line>: error: <message>`, contributing exit `1`; JSON carries the same path, line, code, and message with `"severity":"error"` and `"sites":null`; the LSP carries the same finding over the candidate ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)), and the Python binding reports it with the same code ([§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms)). `--only glob-citation` and `--ignore glob-citation` select it by the ordinary rules ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). `--full` keeps it narrowed to the configured scan scope, as other convention findings are; outside that scope the pattern still records no citation.

#### checks.glob-citation.1: A warning before 0.19.0, an error in it

Every release before grund 0.19.0 reported the finding as a warning at exit `0`, ending `this warning becomes an error in grund 0.19.0`, so a repository that wrote a pattern where a citation belongs had a release to cite the point or escape it ([§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)). In grund 0.19.0 only the severity moved to error, contributing exit `1`, and the closing clause became `this became an error in grund 0.19.0` ([§DF-glob-citation-lands-in-0-19](../decisions/functional/DF-glob-citation-lands-in-0-19.md#df-glob-citation-lands-in-0-19-a-glob-citation-is-an-error-from-0190)); the site, code, and every other byte stayed. The prefix edge is gone in both: that is not this ramp but the correction of [§DF-glob-citation](../decisions/functional/DF-glob-citation.md#df-glob-citation-a-glob-operator-ends-no-citation-and-the-prefix-it-left-behind-is-removed).

### checks.off-format-citation: Off-format citation

A marked token that [§FS-check.1.1.12](FS-check.md#1112-a-marked-token-its-kinds-format-does-not-spell) reads as an off-format citation is reported once, located at its marker's line and column, with code `off-format-citation` and this message:

```text
off-format citation §<token>: <KIND> IDs follow <key> = "<template>" with <patterns>, and nothing declares this one; cite an ID in that format, or write <§><token> for an illustration — unchecked in grund 0.20.0, an error in 0.21.0
```

`<token>` is the token as written after the marker, its alias and section path included, and both `§` and `<§>` are spelled with the configured marker. `<KIND>` is the kind it opens with, and `<key> = "<template>"` is that kind's effective template with the key that owns it, in the vocabulary of [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure) and in the project the token names: `[schema] id_format` or `[schema.kinds.<KIND>] id_format` under version 2, `[id] format` or `[[kinds]] format` under version 1. `<patterns>` names the pattern of each component the template holds, in the template's order and joined by ` and `: `number_pattern = "<pattern>"` for `{number}` and `slug_pattern = "<pattern>"` for `{slug}`, each written as configured between double quotes, unescaped, and by its bare key, because the patterns are schema-wide. The message states facts and guesses no corrected ID ([§FS-declarations.checks.declaration-near-miss.4](FS-declarations.md#checksdeclaration-near-miss4-facts-not-a-guessed-rename)). A token with a section, a numbered format and a kind's own format read:

```text
docs/FS/FS-a.md:3: error: off-format citation §REQ-999.2: REQ IDs follow [schema] id_format = "{kind}-{slug}" with slug_pattern = "[a-z][a-z0-9-]*", and nothing declares this one; cite an ID in that format, or write <§>REQ-999.2 for an illustration — unchecked in grund 0.20.0, an error in 0.21.0
docs/FS/FS-001-a.md:3: error: off-format citation §FS-login: FS IDs follow [schema] id_format = "{kind}-{number}-{slug}" with number_pattern = "\d+" and slug_pattern = "[a-z][a-z0-9-]*", and nothing declares this one; cite an ID in that format, or write <§>FS-login for an illustration — unchecked in grund 0.20.0, an error in 0.21.0
docs/FS/FS-a.md:4: error: off-format citation §TICKET-abc: TICKET IDs follow [schema.kinds.TICKET] id_format = "{kind}-{number}" with number_pattern = "\d+", and nothing declares this one; cite an ID in that format, or write <§>TICKET-abc for an illustration — unchecked in grund 0.20.0, an error in 0.21.0
```

Text output is `<path>:<line>: error: <message>`, contributing exit `1`; JSON carries the same path, line, code, and message with `"severity":"error"`, `"sites":null` and `"authority":null`; the LSP carries the same finding over the token ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)), and the Python binding reports it with the same code ([§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms)). `--only off-format-citation` and `--ignore off-format-citation` select it by the ordinary rules ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), and an ignored finding takes its exit code with it. Under `--full` a site outside the default scope is `out-of-scope-off-format-citation`, this message after the scope ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)).

#### checks.off-format-citation.1: Unchecked in 0.20.0, an error in 0.21.0

Every release through grund 0.20.0 dropped such a token without a word, so a citation of a missing ID written this way passed. grund 0.21.0 reports it as an error with no warning release first: the construct had no defined meaning and produced no output ([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)), so the flip is taken under [§REQ-backwards-compatibility.4](../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), as [§FS-check.3.24.1](FS-check.md#3241-the-release-attribution-and-where-the-command-clause-is-withheld)'s was, and the closing clause is its release attribution in the vocabulary the release guard reads ([§FS-distribution.4.2.2](FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)). Decided in [§DF-off-format-citation](../decisions/functional/DF-off-format-citation.md#df-off-format-citation-a-marked-token-its-kinds-format-does-not-spell-and-nothing-declares-is-reported-rather-than-dropped).

### checks.fs-home-moves: The built-in FS home moves in grund 0.19.0

Retired in grund 0.19.0: the built-in `FS` home is `folder = "docs/functional-spec"` ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), so a project with no config whose `FS` declarations sit in `requirements.md` has its answer rather than a warning about it, and the warning, its code and its catalog row are gone ([§DF-fs-home-lands-in-0-19](../decisions/functional/DF-fs-home-lands-in-0-19.md#df-fs-home-lands-in-0-19-the-built-in-fs-home-is-docsfunctional-spec-from-0190)). grund 0.18.0 warned `fs-home-moves` at the first such declaration. No tree gets an `fs-home-moves` finding any more, and an `--only fs-home-moves` or `--ignore fs-home-moves` left in a script names an unknown code and is refused as any unknown code is ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). This address is kept so citations written before the change still resolve.

### checks.config-version-1: A config file read as version 1

A config file read as version 1 ([§FS-config.5.3](FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired)) earns one warning with code `config-version-1`, naming the file and the command that migrates it:

```text
warning: grund.toml is version 1; migrate it with grund config migrate --to 2
```

The message is the sentence [§FS-config-migrate.8](FS-config-migrate.md#8-workspaces) prints after `member: ` for a member still on version 1, and one renderer writes both, so the two cannot drift apart. Each caller gives it the directory: here it is a report path, while migrate's is a command spelled from the working directory ([§FS-config-migrate.6.5.1](FS-config-migrate.md#651-the-commands-a-migrate-report-prints-are-spelled-from-the-working-directory)). So run at the workspace root, `check` and `config migrate` print the same bytes for a member unless migrate quotes its directory or writes `./` before it, and run anywhere else they can differ in that directory alone. `<file>` is the config file as a report path ([§FS-config.3.6](FS-config.md#36-output--report-format)); after `--to 2` comes the directory the file configures — a project's root, or an `include_root = false` block's own directory — as a report path, and that directory is left out when it is the report's base, which is why a single project's line ends at `--to 2`. For an `.agents/grund.toml` the directory is the project root, not `.agents/`. It fires for each config of the set a run names ([§FS-check.4.11.3](FS-check.md#4113-once-per-config-not-once-per-scope)) that is read as version 1, whether that file writes `grund_config_version = 1` or writes no version, and never for a version 2 file or a project with no config file.

It is [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)'s finding in everything but its trigger and its text: a CLI-level `warning:` on stderr with no site ([§FS-check.4.11.1](FS-check.md#4111-fs-check43s-finding-in-everything-but-its-trigger)), on `check`, `config validate` and `config show` and no other command, `init` included ([§FS-check.4.11.2](FS-check.md#4112-the-same-three-surfaces-as-fs-check43-and-no-more)), once per config rather than once per scope ([§FS-check.4.11.3](FS-check.md#4113-once-per-config-not-once-per-scope)), and under `--format=json` one warning finding on stderr with `path`, `line` and `sites` null ([§FS-check.4.11.5](FS-check.md#4115-under---formatjson-one-warning-finding-on-stderr)). It leaves the exit code alone and stands in place of the `success` line ([§FS-check.4.11.6](FS-check.md#4116-it-stands-in-place-of-the-success-line)); a caller that wants the old bytes passes `--ignore config-version-1` ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).

#### checks.config-version-1.1: Order beside the other config findings

The lines take the report's order ([§FS-errors.4.1](FS-errors.md#41-ordering)), not load order: a version 1 member `aaa` prints before a version 1 root, and an `.agents/grund.toml` on version 1 earns [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)'s line, then this one. Version 2 earns no line: a version 2 root with one version 1 member at `packages/beta` prints exactly `warning: packages/beta/grund.toml is version 1; migrate it with grund config migrate --to 2 packages/beta`.

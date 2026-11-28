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

What every `check` run holds to, whatever its flags, path and scope.

### requirements.success-earned: `success` is earned, never a default

Text `check` prints exactly `success` only for an empty selected report with no unselectable run-level line beside it ([§FS-check.2.1.3](FS-check.md#213-the-success-line)), and each way a run is known to read less than the scope it was given speaks in that line's place: nothing to scan ([§FS-check.2.2](FS-check.md#22-empty-scan)), an obligation — a `[citations]` entry or a `must` or `should` rule — with no unit to judge, in a narrowed run too ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)), nothing recognized in what was read ([§FS-check.4.5](FS-check.md#45-nothing-recognized)), a file the resolution scope could not read ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)), an absent optional member ([§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)), the files an `include_root = false` block leaves unread ([§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)), a `--full` with nothing to widen ([§FS-check.1.3.7](FS-check.md#137-a-path-the-flag-cannot-widen-earns-a-caution-not-a-refusal)), a file root whose extension keeps it unread ([§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read)), a rule that could not run under a narrowing `--only` ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), and a selected check the run leaves off ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)). Serves [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements) and [§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips).

### requirements.narrow-never-lenient: A narrower run never passes what a wider run fails, about the files it reports

About a file at or under its path, `grund check <path>` fails whatever `grund check .` fails: the path narrows the report and not the resolution, and a finding that also names a site outside it is kept whole ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)). Where narrowing does shrink what is read, the run grows stricter, never quieter: one that loads no workspace reports a qualified citation it cannot resolve as an error ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure), [§FS-workspace.5.1](FS-workspace.md#51-a-member-run)), a rule object it cannot judge still fails as `invalid-rule` ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)), and a citation declared in a file it could not read is reported unresolved, with a caution saying why ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)). The plain run stands to `--full` the same way ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)). The promise is about verdicts on reported files: an unread file outside the report leaves the exit code alone ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)), and a slice is never asked the nothing-recognized question ([§FS-check.4.5](FS-check.md#45-nothing-recognized)). Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded).

### requirements.flags: A flag adds findings, filters the complete report, or changes only how it is written

Each `check` flag but `--watch` is one of three. An **adder** — `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), `--require-grounding` ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)), `--suggestions` ([§FS-check.2.3.2](FS-check.md#232-selecting-and-printing-suggestions)), `--rule` ([§FS-rules.8](FS-rules.md#8-command-surfaces)) — may turn a green run red, never the reverse, or open a channel that never moves the exit code. A **filter** — `--only`, `--ignore`, `--only-rule` — is a query over the complete report: each retained finding keeps its message, location and order, and the filter turns red green only over findings it visibly deselects, never over an incomplete run or a rule that could not run ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report), [§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). A **rendering** flag — `--format`, `--path-base` ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)) — changes bytes and no finding. `--watch` changes when the check runs, not what it reports ([§FS-check.6.2](FS-check.md#62-each-run-is-a-plain-grund-check)). A new flag says which of the three it is; one that is none — that turns red green other than by a visible deselection, or skips work a finding depends on — is a new verdict and needs its own decision. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) and [§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips).

### requirements.no-declined-fix: An error never names a fix grund declines to make

An error whose only named remedy the tool refuses to perform is one a repository can never clear ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite), [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link)). So a fix a finding names is one the author can get made: `grund fmt --write` only where `fmt` would write the site, or would once the author lifted a refusal of their own configuring, such as `[fmt] exclude` ([§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)), and otherwise the hand edit, since not every fix is `fmt`'s ([§FS-check.3.15](FS-check.md#315-shorthand-citation-in-a-numeric-run)). The resolving shorthand form is withheld where the text forbids the rewrite ([§FS-check.3.13.1](FS-check.md#3131-where-the-text-forbids-the-rewrite)) and out of scope ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)), and a declaration-local citation's command clause in each case [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation) lists. Every finding holds to this, and so must each finding added. Serves [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## 1. Inputs

- `<path>`, optional — a directory or a single file, which narrows the report and not the resolution ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)). With none, the run covers the project discovered from the current directory ([§FS-scan.path.none](FS-scan.md#pathnone-a-run-with-no-path-covers-the-project-discovery-finds)). What a run reads, Markdown and source files alike, is [§FS-scan](FS-scan.md#fs-scan-what-a-path-argument-means-which-projects-a-run-loads-and-which-files-it-reads).
- `grund.toml`, optional, configures the run ([§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up)). With a `[workspace]` block, a run at the workspace root validates alias-qualified cross-project citations ([§FS-workspace](FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace)).
- `--full` — scan past the configured scope to the whole config root, adding there the out-of-scope findings of [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full).
- `--only <code>` and `--ignore <code>` — keep or drop findings by exact code, `--only` keeping too the findings a selected code carries ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
- `--rule "<sentence>"` — add one ad-hoc chapter rule to the configured rules for this run, never disabling them ([§FS-rules.4](FS-rules.md#4-validation-lifecycle), [§FS-rules.8](FS-rules.md#8-command-surfaces)).
- `--only-rule` — keep only the findings the `--rule` sentence authored; it requires `--rule` ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore), [§FS-rules.8](FS-rules.md#8-command-surfaces)).
- `--require-grounding` — turn the grounding check ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) on for this run. It sets the same global default as `[reference] require_grounding`, so it never switches a configured check off and a row's own `require_grounding = false` still wins ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)).
- `--suggestions` — print the suggestions channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)), which adds no error and never moves the exit code.
- `--format text|json` — the report's shape ([§FS-check.2.1](FS-check.md#21-report-format), [§FS-errors.5](FS-errors.md#5-json-format)).
- `--watch` — check now, then again whenever an effective local input changes, each run a plain `check` ([§FS-check.6](FS-check.md#6-watch-mode---watch)).

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

A CommonMark **raw-text HTML block** (CommonMark's HTML block of type 1) is read as raw text for structure. It opens on a line whose text, after at most three leading spaces, begins with `<pre`, `<script`, `<style` or `<textarea`, case-insensitively, followed by whitespace, `>` or the end of the line. It closes after the first line, the opening line included, that contains the matching end tag `</pre>`, `</script>`, `</style>` or `</textarea>`, again case-insensitively, and an unclosed block runs to end of file. No line of the block, its opening and closing lines included, is a heading: it declares no ID, opens no section, ends no lead and bounds no unit, however much it looks like `# <ID>: …` or `### 3.2 …`, because CommonMark renders these lines as preformatted text and never as headings, so a heading recognized there would be a declaration no reader of the rendered page can see ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)).

Its citations are live, unlike a fence's ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)). The two contexts do not nest: inside a fence an HTML opener is fence content, and inside the block a fence opener is block content. Every surface that asks whether a Markdown line is a heading reads the block by this one rule, as it reads the fence by [§FS-languages.fence](FS-languages.md#fence-fences). Other CommonMark HTML blocks, comments included, keep the ordinary Markdown reading.

##### 1.1.5.2 Front matter holds no heading

A Markdown file's **front matter**, the YAML or TOML block at the top of a page, is a context of its own. It opens only on the file's first line, and only when that line is exactly `---` or `+++`; a first line that begins with a byte-order mark opens nothing ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)). It closes at the next line that is exactly the same delimiter, and a `---` block also at a line that is exactly `...`, which never closes a `+++` block. The block is decided from its lines alone, before any YAML or TOML in it is read, so a delimiter line closes it even inside a multi-line string, and a block that never closes is not front matter.

No line of the block, its delimiter lines included, is a heading: a YAML or TOML comment `# <ID>: …` declares no ID, opens no section, ends no lead, bounds no unit and is no near miss ([§FS-declarations.requirements.mentions](FS-declarations.md#requirementsmentions-writing-about-an-id-never-declares-it)). The block is decided first, from line 1, so a fence opener or a raw-text HTML opener ([§FS-check.1.1.5.1](FS-check.md#1151-a-raw-html-block-holds-no-heading)) inside it opens nothing. Its citations stay live ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code)). `grund fmt` rewrites nothing in the block ([§FS-citations.zones](FS-citations.md#zones-where-citation-text-is-never-rewritten)), and a `grund:fmt off` or `grund:fmt on` directive line inside it still toggles ([§FS-fmt.2.5.2](FS-fmt.md#252-grundfmt-off--grundfmt-on--a-region-at-a-time)). Every surface that asks whether a Markdown line is a heading or may be rewritten reads the block by this one rule.

The rule applies to the files the scan reads as Markdown, a file a `[schema]` source maps to Markdown included, and to no other. No YAML or TOML is parsed, except where the file's first declaration is of a kind that declares a `front-matter` field: there a `---` block is read as YAML and a `+++` block as TOML, and its top-level keys are the field's sections ([§FS-fields.front-matter](FS-fields.md#front-matter-front-matter-is-a-field)). Decided in [§DF-front-matter](../decisions/functional/DF-front-matter.md#df-front-matter-a-markdown-files-front-matter-is-a-region-of-its-own-owned-by-the-files-first-declaration-and-never-rewritten).

Every line of the block, its citations and delimiters included, belongs to the file's **first declaration**, the one whose body starts earliest in the file, whichever heading precedes it. Its section is none, as for a site in that declaration's lead, so `enclosing_section` is `null`, except under a kind that declares a `front-matter` field ([§FS-fields.front-matter.surfaces](FS-fields.md#front-mattersurfaces-what-each-surface-does-with-them)). Every ownership reading takes that owner: the enclosing declaration of `refs` and `cover` ([§FS-cover.3.2](FS-cover.md#32---format-json), [§FS-cover.6.2](FS-cover.md#62-ownership-rules)), the body a `refs --from`, a rule or an obligation reads ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), the citing kind ([§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as)), and the owner a local shorthand expands against ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)). The declaration's body still starts at its heading ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), so the blank lines between the closing delimiter and that heading are unowned, and in a file that declares nothing a front-matter citation has no owner and falls through to the file's home, as any ownerless site does.

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

A citation outside the `[scan] include` roots and scanned kind homes ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)) is invisible to a plain run, and `grund check --full` checks it without first editing the config ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)). It cancels `[scan] include`, and with it the `scan = false` prune of an unscanned kind home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)), and nothing else: `[scan] exclude`, every ignore file, hidden names, directory and file alike ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), workspace member boundaries ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), the E2E case-directory boundary wherever a citable `E2E` kind is configured ([§FS-declarations.e2e-case.walk](FS-declarations.md#e2e-casewalk-the-walk-stops-at-a-case-directory)), and `[scan] extensions` apply exactly as they do without it ([§DF-check-full-scope.2.1](../decisions/functional/DF-check-full-scope.md#21-a-flag-that-cancels-include-and-nothing-else)). A boundary is a declared member, not any nested `grund.toml`, so `--full` reads a project directory the workspace never declared and judges its citations under this project's grammar and kinds; a vendored, generated or example project belongs in `[scan] exclude`, or in `[workspace] members`, before a run adds `--full`.

The wider scan reads a superset of the plain one, each file once. Every root the plain scan starts from is read under `--full` too, a gitignored, excluded or hidden root included ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)); a hidden file and a file of an extension the list leaves out are read by neither run, even as a root, and both runs name such a root alike ([§FS-scan.scope.extensions.root](FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so), [§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read)). A file two roots reach, under two spellings included, is read once, under the spelling the plain run prints ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once), [§FS-scan.identity.findings.content](FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name)).

Inside the default scope the report is exactly the one `grund check` prints on the same tree, character for character, so `--full` can only turn a green run red, never the reverse ([§DF-check-full-scope.2.4](../decisions/functional/DF-check-full-scope.md#24-the-in-scope-report-is-untouched)). Outside it, only [§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)'s citation-resolution errors, its declaration-local form verdict ([§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)) and [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration)'s scanner invariant are reported: no style, grounding, placement, direction, duplicate or unused rule applies there, and no verdict about what the formatter rewrote ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only), [§DF-check-full-scope.2.2](../decisions/functional/DF-check-full-scope.md#22-out-of-scope-findings-are-the-verdicts-a-citing-token-and-the-files-own-structure-settle)). The wider scan's citations never count toward an in-scope finding, so a declaration inside `include` cited only from outside it keeps its `declared but never cited` warning ([§FS-check.4.1](FS-check.md#41-unused-declaration)); widening `include` so the citing file is governed is what makes that edge count.

Run at a workspace root, `--full` widens the root project and every member, each past its own `[scan] include` and judged against its own default scope ([§FS-workspace.5](FS-workspace.md#5-command-scope)). It invents no project, so under `[workspace] include_root = false` ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)) a file at the workspace root outside every member is read by nothing, with or without the flag. A `--full` run whose default scope read no file still earns the empty-scan caution ([§FS-check.2.2](FS-check.md#22-empty-scan)) beside its out-of-scope findings. `--full` is a flag and never a `grund.toml` key ([§DF-check-full-scope.2.5](../decisions/functional/DF-check-full-scope.md#25-no-grundtoml-key)).

#### 1.3.6 An explicit path still narrows

A path narrows the report and not the resolution: the run reads what the whole project's run reads, so that citations resolve and edges count, and reports only the files at or under the path ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports), [§REQ-one-answer.scope.narrowed](../requirements/REQ-one-answer.md#scopenarrowed-a-narrowed-run-answers-as-the-whole-run-does), [§DF-path-scope-resolves-project-wide](../decisions/functional/DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path)). A path that resolves to the config root, `.`, `./` or the absolute root, is the root scope rather than a narrowing. Below a workspace root, the projects a path loads are [§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see)'s.

Three verdicts move with the resolution rather than the report, each the whole-project run's own answer:

- A citation whose declaration lives outside the path resolves, and so grounds its unit ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)).
- A declaration cited only from outside the path is cited ([§FS-check.4.1](FS-check.md#41-unused-declaration)).
- A duplicate declaration whose twin lies outside the path is reported, and the run exits `1` ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)).

A finding that spans several sites ([§FS-check.2.1](FS-check.md#21-report-format)) is in scope at its anchor or at any site it names, and is kept whole, with the same anchor, line and bytes under either scope. So a duplicate may be located at the twin the caller did not type ([§FS-errors.4.1.1](FS-errors.md#411-the-order-of-a-duplicates-sites)), and a content finding on a file with two names, like the refusal of a file two names read differently, is the whole run's one line, located at the name listed first even when that name is outside the path ([§FS-scan.identity.findings.content](FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name), [§FS-scan.checks.reading-conflict](FS-scan.md#checksreading-conflict-two-names-of-one-file-read-it-differently)).

Two classes of diagnostic are not about a scanned file, and the path filters neither: a run-level finding, which carries no path (the config findings, the scope cautions, the workspace run warnings), and the agent-entrypoint finding ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)), a probe over the project root that runs even when nothing is scanned and is exempt by its code, so any other finding about an `AGENTS.md` outside the path is dropped. The scope cautions are asked of the report scope: a path that recognized nothing earns its empty-scan caution ([§FS-check.2.2](FS-check.md#22-empty-scan)) however much the wider walk read, and a narrowed run is never asked the nothing-recognized question ([§FS-check.4.5](FS-check.md#45-nothing-recognized)).

A file the resolution scope could not read fails the run with exit `2` ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)) when any name the walk met it by is at or under the path ([§FS-check.2.4.2](FS-check.md#242-one-failure-one-line), [§FS-scan.identity.findings](FS-scan.md#identityfindings-a-finding-about-a-file-with-two-names-names-each-spelling-once)). Otherwise it is none of the run's errors and leaves the exit code alone, since a run whose report is the path is complete about the path, but a citation declared in it is reported unresolved, so each such file earns one CLI-level caution ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) carrying the read failure's own reason:

```
warning: <path>: <reason> — outside the report scope, so a citation declared there may be reported as unresolved
```

It covers every class of scan failure the walk records ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)), carries the code of the failure behind it, `io` or `unparsable-file` ([§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix)), suppresses no empty-scan caution ([§FS-check.2.2.3](FS-check.md#223-exit-code-json-and-what-suppresses-it)), and no selector hides it ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)). The chapter rules likewise ask whether the resolution scope was read in full, not the report scope ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)).

#### 1.3.7 A path the flag cannot widen earns a caution, not a refusal

`--full` cancels `include`, never a path the caller typed, so with a path other than the config root ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)) the run reports exactly the path, with no out-of-scope findings, and streams and exits as the run without the flag does, but for one CLI-level warning ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), code `full-scope-ignored`. Under version 1 it reads:

```
warning: --full has no effect with an explicit PATH — sim already resolves against [scan] include, and the report is the path either way; drop --full, or drop the path to check the whole project
```

`sim` is the path the caller typed, as a report path ([§FS-config.3.6](FS-config.md#36-output--report-format)). Under version 2 the line describes the configured resolution scope ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)), never calling it `[scan] include`. Under `--format=json` it is a run-level warning object on stderr ([§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)).

### 1.4 Selecting findings with `--only` and `--ignore`

Both flags repeat, and each occurrence takes exactly one code: repeated values form a union, duplicates are harmless, and a comma-separated value is not split. A finding is retained when its code is in the `--only` set, or no `--only` was given, and is not in the `--ignore` set, so `--ignore` wins where both name a code. Codes are the lowercase kebab-case vocabulary of [§FS-errors.5](FS-errors.md#5-json-format), not categories or message fragments. `io` and `unparsable-file` are selectable, and neither axis hides one ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)). Selecting an opt-in code such as `oversized-lead` is valid when its config key is absent, but never turns the check on ([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)); a selected code the run leaves off earns a warning that says so ([§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)).

When `--only` names any of the rule-produced codes [§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits) lists, `invalid-rule` counts as selected too, and every `invalid-rule` finding passes the code axis, not only those of the rule that would have produced the selected code, since a rule that fails to parse has no family to match on ([§DF-narrowed-check-keeps-invalid-rule](../decisions/functional/DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run)). The same selection carries every `empty-citation-obligation` line, the table's and the rules' alike ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing), [§DF-narrowed-check-judged-nothing](../decisions/functional/DF-narrowed-check-judged-nothing.md#df-narrowed-check-judged-nothing-a-narrowed-check-says-when-what-it-selected-judged-nothing)), and `agents-init` carries `invalid-rule` the same way in a run where an invalid rule left a project's blocks uncompared ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). `--ignore invalid-rule` and `--ignore empty-citation-obligation` still win, and the authority axis below still applies to the carried findings. A run whose `--only` set names no rule-produced code selects exactly what it names.

`--only-rule` ([§FS-rules.8](FS-rules.md#8-command-surfaces)) is a second axis, by a finding's `authority`, the rules that produced it ([§FS-rules.7.6](FS-rules.md#76-selection-json-ordering-and-exits)), since a trial sentence emits the declared rules' codes ([§DF-rule-authority-is-a-field](../decisions/functional/DF-rule-authority-is-a-field.md#df-rule-authority-is-a-field-a-findings-rule-authority-is-a-record-field-and-the-trial-sentence-selector-is-a-query-over-it)). A finding is retained when its code passes the code axis and, if `--only-rule` was given, its `authority` contains the `--rule` origin. `--ignore` wins over both, so `--only-rule --ignore <the trial sentence's own code>` can empty a report.

Selector values are validated before config discovery or scanning, alike for `--only value` and `--only=value`. A code matches `[a-z0-9]+(?:-[a-z0-9]+)*`. A missing or empty value is refused as `error: --only requires a finding code` or `error: --ignore requires a finding code`; a value outside that grammar as `error: invalid finding code "<value>" (expected lowercase kebab-case)`; a well-formed value outside the public catalog as ``error: unknown check finding code "<value>"; run `grund check --help` for supported codes``; and `--only-rule` without `--rule` as `error: --only-rule requires --rule`. Each is a CLI failure: stdout is empty and the exit is `2` ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)). `--only-rule` is a boolean, so it takes no value and repeats harmlessly.

#### 1.4.1 A selected check the run leaves off says so

A code is **armed** in a run when the run's configuration and flags turn its check on, whether or not any unit then gives it something to find, and in a workspace when any project of the run arms it. What arms a code is its *Enabled by* cell in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog): a cell of `—`, or one naming the very thing the code judges, always arms it; a cell naming several ways is armed by any of them; and one that needs two things, only where both hold. A workspace target's fetched kind arms `missing-snapshot` where the arming project's own rules resolve that kind as `should` (`warn` under version 2).

When `--only` selects a code that no project of the run armed, and `--ignore` does not name it, `check` writes one CLI-level warning ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) for that code, in code order, with the code `selected-check-off`:

```text
warning: --only oversized-lead judged nothing — the check is off until [schema.leads.words] warn = <N> is set
warning: --only missing-field judged nothing — a version-1 config cannot turn the check on; migrate it with grund config migrate --to 2
warning: --only excluded-stub-target judged nothing — the check is spent from grund 0.19.0; select broken-stub instead
```

The advice names what the run lacks of the code's *Enabled by* cell, spelled in the config version the run read. A switch the version read cannot hold, such as a field under version 1, names `grund config migrate --to 2` and never a key that file cannot carry, and a workspace whose projects read different versions names a key only where every version it read spells it alike, and otherwise `grund config migrate --to 2` for its version-1 projects. The advice is the catalog's own words, read from the rendered catalog `grund config explain` prints ([§FS-config-explain.4](FS-config-explain.md#4-a-codes-answer)): `the check is off until ` and the clauses of the code's *Enabled by* cell the run lacks, without their remarks and joined by ` and `, or for a spent code `the check is ` and its *Severity* cell after `none — `, each with its backticks dropped ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)). The words around them are the cases of `selected-check-off`'s own *Fix* cell — `off` before the clauses, `one-version` and `mixed-versions` after the migrate reason — so they too are worded once ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)).

The line is about the selection, not about a finding, so it passes both selection axes, `--only-rule` included, and only `--ignore selected-check-off` removes it. Under `--format=json` it is a run-level warning object on stderr ([§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)), its `authority` `null` too. A code the run armed whose findings the channel holds back, such as a `should`-level field without `--suggestions`, did run, and earns no line ([§FS-rules.8](FS-rules.md#8-command-surfaces)). Decided in [§DF-narrowed-check-judged-nothing](../decisions/functional/DF-narrowed-check-judged-nothing.md#df-narrowed-check-judged-nothing-a-narrowed-check-says-when-what-it-selected-judged-nothing).

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

Some lines no selector removes. CLI and config failures are outside the report, and neither axis hides `io` or `unparsable-file`, so an incomplete run ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)) exits `2` whatever the selectors. `import-live` ([§FS-imports.checks.import-live](FS-imports.md#checksimport-live-an-import-is-read-live-unpinned)), `import-missing` ([§FS-imports.checks.import-missing](FS-imports.md#checksimport-missing-an-import-names-no-directory)), `reading-conflict` ([§FS-scan.checks.reading-conflict](FS-scan.md#checksreading-conflict-two-names-of-one-file-read-it-differently)) and the unread-source caution ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)) stay too, each for the reason its own section gives. Narrowing to a code a rule produces carries the `invalid-rule` and `empty-citation-obligation` findings with it ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), so a narrowed run never passes a rule it could not run or an obligation with nothing to judge.

#### 2.1.3 The `success` line

When the selected report retains no error, warning or suggestion and no unselectable run-level line, the default text form writes exactly `success` and a newline to stdout. It says that the selected report is empty, not that the repository has no findings. There is no summary footer: the exit code is the verdict, and the finding lines are the detail.

#### 2.1.4 JSON

With `--format=json`, the retained findings are NDJSON on stdout, one object each in the shape of [§FS-errors.5](FS-errors.md#5-json-format), in the global order of [§FS-errors.4.1](FS-errors.md#41-ordering) rather than text's channel groups. Stdout holds findings only: an empty selected report writes nothing, with no object in place of `success`, so `grund check --format=json | jq …` sees only findings. Run-level lines go to stderr ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), so a clean JSON run is empty on both streams and an exit `2` always has something on stderr.

### 2.2 Empty scan

A run that read no file in its default scope, or under the path it was given ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), is almost always a misconfigured scope rather than a clean repository, so in place of `success` `check` prints one CLI-level warning ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), code `empty-scan`, that names the likely cause ([§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)). A workspace asks it per project ([§FS-workspace.5.3](FS-workspace.md#53-the-empty-scan-caution-names-each-project-from-the-report-base)).

#### 2.2.1 Citation-direction obligation applies to nothing

`check` prints one CLI-level warning ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), code `empty-citation-obligation`, for each table or rule whose citation obligation has nothing to judge. A `[citations.<kind>]` table earns it when all of these hold:

1. The table has a non-empty `must` or `should` list; a table of `must-not` or `should-not` entries only has no obligation and never warns. A table with both levels is named by `must`.
2. The citing kind has a `folder` home, and the kind is scanned.
3. The run read at least one file in that folder besides its entry file and a generated scoped entrypoint ([§FS-init.2.3.5](FS-init.md#235-citation-directions)). The entry is the effective `index` of a citable kind (`README.md` when the key is omitted) and the literal `README.md` of a non-citable one; `index = false` excludes no file.
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
4. Under version 1, the run is scoped to the root (no path, or `.`): the message names `[scan] include` and its list, with the scaffold, config and path advice, and a project that declares no kinds ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)) gets the `[[kinds]]` advice in place of the scaffold. Any v2 repair instead names its places ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)) or source exclusions ([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)), never `[scan] include` or `[[kinds]]`; a first kind it recommends is `[schema.kinds.<NAME>]`.
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

`empty-scan` is a warning, because a genuinely empty tree is not a failure, so it never moves the exit code; its JSON form is [§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)'s. Any finding about the default scope replaces it, the agent-entrypoint check of [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block) among them, which runs even when nothing is scanned. Four findings are about something other than the scope the run scanned, and the caution stands beside them: the redundant config pair ([§FS-check.4.3](FS-check.md#43-redundant-config-pair)), an out-of-scope citation ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only), [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), an absent optional member ([§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)) and the unread-source caution ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)).

### 2.3 Suggestions channel *(opt-in)*

Suggestions are findings on a third channel, not at a third severity, because [§FS-config.6](FS-config.md#6-what-is-not-configured-here) keeps the severities at `{error, warning}` ([§FS-terms.terms.5](FS-terms.md#terms5-findings), [§DF-citation-directions.2.3](../decisions/functional/DF-citation-directions.md#23-suggestions-are-a-third-channel-not-a-third-severity)). The `should` and `should-not` levels of `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), the `should` and `should-not` strengths of version 2 ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities)) and the `should` and `should not` results of chapter rules ([§FS-rules.7](FS-rules.md#7-findings-and-channels)) produce them. A `should` citation a declaration lacks is `suggested-citation` and a `should-not` citation is `discouraged-citation`, whether a table or a rule asks for it ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)); an escape that resolves is `escaped-citation-resolves` ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)); and every other suggestion keeps the code it has at any strength ([§FS-config-v2.rules.strengths](FS-config-v2.md#rulesstrengths-one-strength-vocabulary-two-severities), [§FS-rules.7](FS-rules.md#7-findings-and-channels)).

#### 2.3.1 Escaped citation resolves

An escape, `<§>ID` or `<§>alias/ID`, is not a citation ([§FS-check.1.1.9](FS-check.md#119-an-id-in-an-escape-position)), which is how prose shows a citation's shape. `check` reports `escaped-citation-resolves` at an escape whose coordinate resolves, because it may be a live citation escaped by mistake ([§DF-escape-position-is-not-a-citation.2.4](../decisions/functional/DF-escape-position-is-not-a-citation.md#24-one-rule-read-by-the-reader-and-the-writer)). The coordinate resolves when its ID names a declaration and any section it writes is found there as the missing-section check finds one ([§FS-check.3.2](FS-check.md#32-missing-section)), a stub's sections being its target's ([§FS-check.3.2](FS-check.md#32-missing-section)); an escape of a section that does not exist earns nothing, as an escape of an undeclared ID does. The message names the live form of the same coordinate. It keeps the section, since the declaration alone would widen a citation of one section to the whole declaration. The escape is read with the citing project's grammar, and the live form is written with the separator of the project the coordinate names ([§FS-workspace.1.2](FS-workspace.md#12-whose-grammar-reads-the-id)), so that it reads back as the coordinate the escape named; here member `api` separates sections with `:`:

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

A path the scan failed on for one reason is one line from each command that reports scan failures, one JSON record and one record in the embedding API, however many walks and passes met it ([§DF-e2e-case-on-the-one-walk.2.3](../decisions/functional/DF-e2e-case-on-the-one-walk.md#23-once)). Two reports are one failure when they have the same reason before any context ([§FS-check.2.4.1](FS-check.md#241-one-reason-for-one-scan-io-failure)) and the same path: a file under any of its spellings ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once)), and, for two reports the walk makes, one directory entry, its parent resolved and its own name not. A link and its target are therefore two failures ([§FS-scan.scope.links.unresolvable](FS-scan.md#scopelinksunresolvable-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into)), and two different failures stay two lines even where they read alike ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)).

The line is at the path that failed, never at a walk root, configured folder or case directory standing in for it ([§DF-e2e-case-on-the-one-walk.2.2](../decisions/functional/DF-e2e-case-on-the-one-walk.md#22-at-the-path-that-failed)), spelled from the report base, and its reason holds no absolute path ([§FS-errors.4](FS-errors.md#4-determinism)). A file met under several spellings is named by the one the run's own path reached it under, else by the one listed first ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once)). `grund check <path>` names a failure the walk reports by its name under the path given, in a plain project and at a workspace root alike, and every other surface names it by the whole run's spelling. Under a path scope the failure is either the caution of [§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows), outside the report scope, or the `error:` line, inside it, never both. Where the directory a symlink loop reaches back into lies under the config root, the loop's reason names it by its one physical location, spelled from the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), so every route to the link has one reason; where that location is the report base itself, the reason is `symlink loop: the target contains the link`.

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

A link is judged as the path it is: at a scan root it is `root`, below one `entry`. A member's scan root is judged in its own project, so the row its fix names is in that member's config ([§FS-workspace.5](FS-workspace.md#5-command-scope)). The same code and fix reach every surface that publishes the failure ([§FS-errors.2.2.5](FS-errors.md#225-one-scan-failure-one-rendering)), the caution of [§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows) among them.

## 3. Errors detected

Each check below reports its findings as errors that contribute exit `1`, unless its code's row in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) gives a strength or a setting that makes a finding a warning or a suggestion. The row also says what turns the check on and holds its *Fix*. Every finding is located and rendered as [§FS-check.2.1](FS-check.md#21-report-format) says.

### 3.1 Dangling citation

A recognized citation ([§FS-check.1.1](FS-check.md#11-recognized-citations)) whose ID no declaration declares is `dangling`, reported at the citation as `unknown reference <ID>`. The ID is resolved over the run's resolution scope, the whole enclosing project even for a run given a path ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)). A number-only shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) is reported by [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) instead, never by both. At a value binding this finding suppresses value comparison at the same site ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)).

For a kind with `fetch` whose effective target-side resolution is `must` ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)), the message is `unknown reference <qualified-ID>; no snapshot in <home>`, and a citation into a home this run failed to read earns no finding ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)). A kind without `fetch` has no such clause.

The message then ends in at most one fix: a near ID or escape, else a fetch tail, else a missing place, whichever applies first. Where the fix is a case of the `dangling` row of [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog), it is named by its case below. The kind, its home, its places and its config are those of the project the citation targets, never the citing one's.

A **near ID or escape** follows `; `. In a workspace run, an unqualified ID that other loaded projects declare takes the `near` case with those qualified spellings, chosen as [§FS-workspace.8.1.1](FS-workspace.md#811-an-unqualified-id-another-project-declares) chooses them from the citation's own project, a member's even in a root run: `unknown reference AR-005-monitor; did you mean worldsim/AR-005-monitor?`. Otherwise, where the target project declares an ID of the same kind close by deterministic edit distance, the fix is the `near` case; with none close enough, no ID is guessed. Inside a Markdown inline-code span ([§FS-fmt.2.3.5](FS-fmt.md#235-an-inline-code-span-closes-on-a-run-of-its-own-length)), and only there, the `<§>` escape ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)) is offered in the citing project's marker: the `escape` case alone, or `near-escape` with a candidate. Either one outranks the fetch tail and the missing place. A member candidate only advises: it is drawn from the projects the run loaded, fetching none, and leaves resolution project-local ([§FS-workspace.4](FS-workspace.md#4-resolution)) and the finding's scope, code, severity, exit and JSON shape as they are.

A **fetch tail** follows the `no snapshot` clause after ` — `, and is the first row of this table that holds for the home:

| The home | The tail |
|---|---|
| a `file` home that is a value source, of a whole-value kind ([§FS-values.terms](FS-values.md#terms-terms)) | `value-source` |
| a `file` home the text scan reads as source ([§FS-fetch.4.2](FS-fetch.md#42-a-home-read-as-source-is-refused)) | `source-file` for a non-value kind, `source-file-value` for a value kind |
| a `file` home the text scan does not read ([§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)), of any kind, such as `fs.dat`, a hidden `.fs.py`, or a non-value kind's `.yaml` that `[scan] extensions` does not list | `give the file a name the scan reads as Markdown` |
| a folder home whose `<ID>.md` an ignore file or an `exclude` rule keeps out of the walk | `the scan does not read <path>: <reason>; <repair>`, with `<path>` spelled from the report base and `<reason>` and `<repair>` as [§FS-fetch.4.2.1](FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused) words them, less `, and fetch again` |
| a version 1 row whose `fetch` value names no executable and no file of that very name stands where its project would run it, as `grund fetch` asks it ([§FS-fetch.2.1](FS-fetch.md#21-a-value-that-names-no-executable)); version 2 refuses such a key at load ([§FS-config-v2.checks.config-fetch-names-nothing](FS-config-v2.md#checksconfig-fetch-names-nothing-a-rows-fetch-names-no-executable)) | ``[[kinds]] `<KIND>` fetch names no executable; `` and the `fetch-names-nothing` case, its setting `resolve` |
| any other home, such as a value kind's Markdown `file` home or any other folder home | `fetch`, naming the qualified ID, or for an ID reached through an import, nested or not, `<ID below the last import edge> in <target of that edge>` ([§FS-imports.writes.answer](FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer)) |

The same tails end the `missing-snapshot` warning ([§FS-check.4.12](FS-check.md#412-missing-snapshot)). Asking whether `fetch` has nothing to run looks up one path and runs nothing ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

A **missing place** follows `; `. Under a version 2 config, where the kind is a row with at least one place and none of its places exists ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)), the tail is `no place of kind <KIND> exists: <place> (<config>:<line>)`, with one `<place> (<config>:<line>)` for every place of the row in the row's order, joined by `, `. Each `<place>` is normalized from the config root (`./specs/*` prints `specs/*`), and `<config>:<line>` is its entry, the config path spelled as finding paths are (`m/grund.toml:4` for a workspace member). When a named place holds one of `*`, `?`, `[`, `]`, `{`, `}` or `!`, the `pattern` case follows after `; `. No tail is added when one place of the row exists, since the ID may then be a plain typo. A missing place earns no finding of its own ([§FS-config-v2.schema.places.boundaries](FS-config-v2.md#schemaplacesboundaries-what-a-place-may-name)).

```text
docs/guide.md:3: error: unknown reference FS-logon; did you mean FS-login? (or write <§>FS-logon if this is an illustration)
docs/offer.md:3: error: unknown reference CONST-other; no snapshot in values/limits.md — run grund fetch CONST-other
docs/offer.md:3: error: unknown reference FS-other; no snapshot in fs.py — declare it in that source file
README.md:1: error: unknown reference T-7; no snapshot in docs/t — the scan does not read docs/t/T-7.md: docs/t/.ignore ignores it by T-7.md; stop ignoring it in docs/t/.ignore
docs/arch.md:7: error: unknown reference AR-002-other; no snapshot in docs/arch.md — [[kinds]] `AR` fetch names no executable; name the program, or if this kind is not fetched remove the key and any `resolve`, and its missing IDs are then errors
src/main.rs:1: error: unknown reference SPEC-upload; no place of kind SPEC exists: specs/*/spec.md (grund.toml:4); a place is a path from the config root, not a pattern, so list each file or name the folder
```

### 3.2 Missing section

A citation with a section suffix (`§FS-<user-login>.3.1` or, in an opted-in repository, `§FS-<user-login>.goals`) whose declaration exists but has no heading at the requested path is `missing-section`. A marked named coordinate is never shortened to its declaration. Every missing section, numeric or named, marked or not, takes the message and fix of [§FS-errors.3.11](FS-errors.md#311-the-missing-section-remedy).

The form written and the target requested are independent facts: a number-only shorthand carrying a missing named section gets both its shorthand finding ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)) and this one, and so does an owned declaration-local numeric candidate ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)) whose local path is missing, with the finding of [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation). At a value binding the explicit numeric component must resolve before comparison, and a missing one gives this finding alone, not a mismatch ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)).

A citation of a stub's ID names a section of the declaration the stub pairs with ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so the section exists exactly where that declaration records a heading at the requested path, read from the stub's target whether or not `[scan] include` reaches it; the stub declares no sections of its own ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)). `check` and `show` answer from that one section set ([§FS-show.2.2.2](FS-show.md#222-ambiguous-section)), and so does every other reader of a section ([§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map), [§REQ-one-answer.stubs.outside-include](../requirements/REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned)). What else a target outside the include gives the stub is [§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged). Decided in [§DF-stub-sections-from-unscanned-target](../decisions/functional/DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it).

### 3.3 Duplicate declaration

Moved to [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration). This address is kept so citations written before the move still resolve.

### 3.4 Broken inline-spec stub

Moved to [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub). This address is kept so citations written before the move still resolve.

### 3.5 Invalid agent entrypoint init block

`check` verifies the versioned `grund init` block ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) of the agent entrypoints below, and reports `agents-init` for a block that is missing where one is required, of an older version, of a newer version this grund does not support, malformed, or different anywhere from what `grund init` would write there ([§FS-init.2.3.7](FS-init.md#237-the-block-version)). The findings come after every content pass, in the templates of [§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages); those that name `grund init` also name the directory to run it in when the block belongs to another project than the run's.

If `<path>/AGENTS.md` exists, it is verified, and so is every known companion that exists and is not a symlink to it, such as a standalone `CLAUDE.md` or `.github/copilot-instructions.md`, each against the block rendered for its own agent ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)); `.rules` counts only on the evidence [§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent) attributes it by. An absent companion is not required ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)); once `grund init` creates one, it is verified because it exists. If `AGENTS.md` does not exist, only a companion that already carries a managed block is verified, so a config-only adoption never polices an existing agent setup.

Under scoped guidance ([§FS-init.2.3.5](FS-init.md#235-citation-directions)), and only where a root entrypoint already carries a managed block, every destination of the live render is verified, with the standalone `CLAUDE.md` or `GEMINI.md` beside it, and so is every orphan the list on disk names ([§FS-init.2.3.5](FS-init.md#235-citation-directions)), each reported at that file as [§FS-init.2.3.5](FS-init.md#235-citation-directions) says; a companion symlink to a scoped `AGENTS.md` is covered by that file.

A legacy H2-bounded block of v3 or earlier ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) is reported as an older version. A block whose delimiters are **malformed** ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) is its own error, located at the offending delimiter line and naming the defect.

The comparison covers the whole block, its fixed text and every substituted line ([§FS-init.2.3.8](FS-init.md#238-substituted-content)), `### Chapter rules` included, since a chapter rule unverifiable in this scope still earns its bullet ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)). Two things go uncompared: the workspace sections in the runs [§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout) lists, and every block of a project while one of its rules is genuinely invalid ([§FS-rules.4](FS-rules.md#4-validation-lifecycle)), where `grund init` refuses to write and the located `invalid-rule` error, which a run narrowed to `agents-init` keeps ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), is the finding to act on. A workspace run compares each project's block against the vocabulary of the workspace it loaded ([§FS-rules.9.1](FS-rules.md#91-one-tree-renders-one-block)). The render reads the config, the workspace and the scanned rule declarations ([§FS-init.2.3.5](FS-init.md#235-citation-directions)), never a path argument ([§FS-rules.9.1](FS-rules.md#91-one-tree-renders-one-block)) or a run flag such as `--full` or `--require-grounding`, so every `check` of one tree compares against the block `grund init` writes there.

A stale block is reported once per section that differs, a section expected and absent, or present and not expected, among them. Each names its section by a lower-case noun: `citation scheme` for the paragraph under the block's version heading ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)), and for every `###` section its heading lower-cased: `grounding from a citation`, `project map`, `project namespaces`, `workspace members`, `declarations and citations`, `rules`, `citation directions`, `clickable citations`, `declaration shapes`, `chapter rules`, `scoped guidance`. The source it names is `the workspace` for `project namespaces` and `workspace members`, since another project's file moved them, and `grund.toml` for every other section.

For those two workspace sections the finding adds a detail before the command, `expected <entry>; found <entry>`, the first unequal pair by position between the re-render and the block on disk. An entry is a nonblank body line below the section heading, a member bullet or a prose line, compared verbatim; a side that ends first, or an absent section, reads `<missing>`. Where the bytes differ but no entry does, as with blank-line, heading or section-order drift, the detail is `section text or order differs`. The detail explains the byte comparison and never narrows it.

```text
packages/billing/AGENTS.md:3: error: repo maintenance: stale grund init block: the workspace members section differs from the workspace (expected - [`notify`](../notify/); found - [`shop`](../../)) — run `grund init` in packages/billing to refresh (does not affect citation validity)
```

#### 3.5.5 The workspace sections are judged at the outermost root of a full checkout

`### Project namespaces` and `### Workspace members` ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)) are rendered from the whole workspace. Only a run at the outermost workspace root whose expansion found every member has loaded all of that, so only such a run compares them, in every block it verifies ([§DF-check-compares-the-whole-block.2.2](../decisions/functional/DF-check-compares-the-whole-block.md#22-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)). Every other run removes the two sections from both sides before comparing, and compares the rest:

1. a run inside a member, `grund check <member>` from the root among them;
2. a run at the root of a nested group;
3. a run in a standalone project, whose block carries neither section;
4. a run in a workspace where an optional member is absent ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)), anywhere in it, where `grund init` keeps the member list it cannot render ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)).

The boundary is declared here and bounded by the outermost root's run ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)). In the fourth case the run also says so: the `optional-member-absent` warning gains a clause naming the workspace sections whenever the run verified a managed block ([§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)). A project removed from `members` keeps workspace sections no `check` compares; its own `grund init --check` still sees them ([§FS-init.1.4](FS-init.md#14-what---check-gates)).

### 3.6 Ungrounded unit *(opt-in)*

Where a row's `require_grounding` and `grounding_level` ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) ask it, each unit ([§FS-check.3.6.2](FS-check.md#362-the-unit)) of a file the row governs ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs)) must be grounded: it contains at least one recognized citation ([§FS-check.1.1](FS-check.md#11-recognized-citations)) whose ID resolves to a declaration, or, in a source file outside every non-citable home, it declares an ID inline. A unit that is neither is `ungrounded`. A unit whose only citation is dangling ([§FS-check.3.1](FS-check.md#31-dangling-citation)) is not grounded and gets both findings. Under version 2 the two keys are one ladder of strength-and-unit rungs ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). Decided in [§DF-require-grounding](../decisions/functional/DF-require-grounding.md#df-require-grounding-an-opt-in-check-that-every-source-file-cites-a-spec).

A finding is located at its unit, line 1 for a file, the heading line for a section, the block's first line for a doc-comment, and names the unit and, when the unit sits in a non-citable home, the home. Section units arise only there. The marker in the message is the configured one ([§FS-config.3.1](FS-config.md#31-reference--citation-form)).

```text
src/util.rs:1: error: ungrounded source file: no § citation to a declared ID; cite a declared ID within it
skills/triage.md:1: error: ungrounded file in kind home skills/: no § citation to a declared ID; cite a declared ID within it
skills/review.md:5: error: ungrounded section `## Steps` in kind home skills/: no § citation to a declared ID; cite a declared ID within it
src/main.rs:3: error: ungrounded doc-comment: no § citation to a declared ID; cite a declared ID within it
```

#### 3.6.1 Which files a row governs

Every scanned file resolves to exactly one `[[kinds]]` row, and that row's effective `require_grounding` decides whether the file is checked at all:

- **A non-citable kind's home** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) governs every scanned file in it, Markdown included: a `folder` home's files, or the one document of a `file` home ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), which takes both keys on its row. A folder home's entry file ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)) and a generated scoped entrypoint ([§FS-init.2.3.5](FS-init.md#235-citation-directions)) are not among them.
- **A citable kind's folder home** governs the **source files** in it, a source file being a file the scan reads that is not read as Markdown ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)), whatever its name.
- **The homeless kind** ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) governs the source files no home claims, and a file two overlapping homes claim falls to it as well, the way its citing side does ([§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as)).

So Markdown is governed only inside a non-citable home, and there by the home rather than by the extension ([§DF-non-citable-kinds.2.5](../decisions/functional/DF-non-citable-kinds.md#25-obligations-get-a-per-file-unit-and-grounding-follows-the-home)). An unscanned home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) has no scanned files for the rule to reach, so `require_grounding = true` on its row is a config error.

#### 3.6.2 The unit

`grounding_level` is a Markdown heading level, and each level **contains the one below it**, so the file itself is a unit at every level and nothing passes for lacking structure. Every failing unit is reported: a file that cites nothing at level `2` earns the file finding and one for each of its sections.

- **In a Markdown file**, level `L` adds a unit for every heading subtree of depth `2` to `L`. A subtree runs from its heading to the line before the next heading at the same or a shallower depth, so a parent is satisfied by any descendant and a leaf must cite directly; text before the first heading belongs to the file.
- **In a source file**, the units below the file are doc-comment blocks, ranked by indentation rather than syntax ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)): at level `2` every unindented block is a unit, a parse-free stand-in for a top-level item, and at any higher level every block is. What counts as a doc-comment is the per-language rule of [§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites).
- **An inline declaration** grounds the doc-comment block that declares it and the file it sits in, except inside a non-citable home, where a declaration is misplaced to begin with ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)) and only a citation grounds a unit.

### 3.7 Misplaced declaration (configured kind home)

Moved to [§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home). This address is kept so citations written before the move still resolve.

### 3.8 Cross-project citation failure

An alias-qualified citation whose alias path is unknown is `unknown-project`, reported at the citation site in every run, with or without a `[workspace]` ([§FS-workspace.1](FS-workspace.md#1-citation-syntax)); in a workspace run, one whose target declaration or target section is missing is reported at the citation site too, as [§FS-check.3.1](FS-check.md#31-dangling-citation) and [§FS-check.3.2](FS-check.md#32-missing-section) say. Aliases resolve as [§FS-workspace.4](FS-workspace.md#4-resolution) says, and a member-local finding's verified enclosing-workspace explanation and root-check remedy are [§FS-workspace.5.1.1](FS-workspace.md#511-verified-enclosing-workspace-guidance)'s.

An unknown alias path names its fix, by where the run stands:

- **At the outermost workspace root**, the `near` case names candidates from the first band that has any: a project whose slash-separated path has every written segment as a **proper prefix** and at least one segment more; one whose path **ends with** what was written, a dropped prefix ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)); one whose **last segment** matches, a wrong prefix; one a **typo** away under the near-ID rule of [§FS-check.3.1](FS-check.md#31-dangling-citation). So `group` matches `group/alpha`, while an exact `group`, `grouped/alpha` and `other/group` do not match the proper-prefix band. The band's candidates are deduplicated, sorted by alias bytes and limited to three, and a path with no candidate gets no `did you mean`.
- **A run narrowed to a subtree** ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)) searches only when its scope path is a strict, segment-wise prefix of the written path, which starts with every scope segment and has at least one more (`group/alph` in scope `group`), and then only among the aliases it loaded, with the bands above; finding none, the message is bare. Any other path, shorter, equal, not starting with the scope or only lexically prefixed (`alpha`, `group`, `outside/alpha`, `grouped/alpha` in scope `group`), gets no candidate but `; the <scope> project and its descendants are in scope here — ` and the `subtree` case. Such a run cannot tell a dropped prefix from a correct path to a project outside its subtree, so a hint could re-point a citation CI accepts at another project.
- **A run that is not narrowed** and finds no candidate gives the `unlisted` case when the alias path is a directory beneath its root whose `grund.toml` or `.agents/grund.toml` no `[workspace]` lists ([§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)), and stays bare where membership cannot be decided.

```text
docs/FS-root.md:3: error: unknown project alias sprayer; did you mean hardware/sprayer?
docs/FS-root.md:4: error: unknown project alias api; did you mean left/api or right/api?
docs/FS-left.md:3: error: unknown project alias api; the left project and its descendants are in scope here — check from the workspace root for a path outside that subtree
docs/fs/FS-checkout.md:3: error: unknown project alias api; add "api" to [workspace] members in grund.toml
```

### 3.9 Section heading level mismatch

Moved to [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch). This address is kept so citations written before the move still resolve.

### 3.10 Inline citation style violation

A citation site in a code comment that breaks the configured inline citation style ([§FS-config.3.1](FS-config.md#31-reference--citation-form)) is `inline-citation-style`: prose under `inline_style = "citation-only"`, a note over `inline_note_max_lines`, or one over `inline_note_max_columns`. [§FS-inline-citation-style.4.1](FS-inline-citation-style.md#41-errors--hard-caps) holds the modes, the budgets, the messages and how many findings a note earns. A site is an *inline* comment block, never a doc-comment block, so a citation inside a `///`, a `/** … */`, a docstring, or a comment documenting the definition below it is never judged here ([§FS-inline-citation-style.1.1](FS-inline-citation-style.md#11-doc-comments-are-not-sites)).

With `[reference] inline_note_layout` set to a layout, each line [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit) judges, in a site that carries a note ([§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)), that does not match the configured form is a finding located at that line rather than at the site's first line, reported as `inline_note_layout_check` asks ([§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)).

### 3.11 Missing required citation

A `must` obligation of `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) or `[rules.citations.<kind>]` ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)) holds each unit of its citing kind to at least one citation satisfying each `must` entry; a unit without one is `missing-citation`, naming the unmet target. The unit is the citing kind's:

- **A citable kind's** unit is each top-level declaration, satisfied anywhere in its body and reported at its declaration line. A declaration a stub pairs with is the unit wherever its target is: a target outside the include lends it its citations ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)), which meet the obligation, a unit only that target supplies keeps the obligation from applying to nothing ([§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing)), and its finding is reported at the declaration's line in the target ([§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at), [§FS-rules.5.1](FS-rules.md#51-facts-and-identity)).
- **The homeless kind's** unit ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) is each source file that carries at least one citation, named `source file` and reported at line 1.
- **A non-citable kind's** unit ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is each scanned file in its home that carries at least one citation, a file read as Markdown included ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)), unlike the homeless kind's, named by its home since it has no ID, and reported at line 1 ([§DF-non-citable-kinds.2.5](../decisions/functional/DF-non-citable-kinds.md#25-obligations-get-a-per-file-unit-and-grounding-follows-the-home)).
- **An `E2E` kind's** unit ([§FS-config.3.9.1](FS-config.md#391-levels)) is each case declaration ([§FS-declarations.e2e-case](FS-declarations.md#e2e-case-what-an-e2e-case-directory-declares)), which its `spec.refs` manifest entries can satisfy too, reported when the case has no scanned citation or matching manifest entry.

```text
requirements.md:3: error: FS-example must cite GOAL (citation direction); add such a citation within it
src/util.rs:1: error: source file must cite FS (citation direction); add such a citation within it
skills/review.md:1: error: skills/ must cite FS (citation direction); add such a citation within it
```

The two per-file units follow the row's `grounding_level` ([§FS-check.3.6.2](FS-check.md#362-the-unit)): at level `2` each `##` subtree of a non-citable home's Markdown file, and each unindented doc-comment block of a source file, that carries a citation is a unit too, and the file stays one at every level, satisfied by any citation under it. A citable kind's unit stays its declaration at every level. Every failing unit is reported: the file and each section unit that satisfies no entry either. A section unit's finding is the file's message located at the section's heading line, so the two differ only in the anchor, which tells the reader whether the miss is local to one section.

A homeless or non-citable file with no citation is no unit, so `must` cannot fire on it. A scanned folder with real non-entry content earns the warning of [§FS-check.2.2.1](FS-check.md#221-citation-direction-obligation-applies-to-nothing) instead, the homeless kind's zero-unit case stays unwarned ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), and in a non-citable home `require_grounding` ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) closes the hole, so "cite something" and "cite an `FS`" are two keys that compose.

A `must` rule judges its subject instead, a declaration or a named chapter ([§FS-rules.2](FS-rules.md#2-subject-selectors)), located at the subject's declaration line or chapter heading; its sentence `<subject> must cite at least one <target-set>.` reuses this code only when its actual count is zero ([§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality)). Rule-to-rule and config-to-rule duplicates collapse as [§FS-rules.6](FS-rules.md#6-semantic-deduplication) specifies, and where config takes part the message stays this section's. The `should` level reports `suggested-citation` instead ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)).

### 3.12 Forbidden citation

A `must-not` prohibition of `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)) or `[rules.citations.<kind>]` ([§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)) makes every citation site of its citing kind to a prohibited target a `forbidden-citation`, reported at the citation site. The citing kind is the one [§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as) gives the site, named by kind for a citable kind, by **home** for a non-citable one (`skills/ must not cite AR`), and by name for the homeless kind, which has no home to name it by ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)). The cited kind and alias come from the citation token, matched against the rule's alias grammar ([§FS-config.3.9.3](FS-config.md#393-alias-matching)). The `should-not` level reports `discouraged-citation` instead ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)).

```text
docs/fs.md:3: error: FS must not cite AR (citation direction) — re-point the citation or downgrade it to a plain Markdown link
```

A plain Markdown link is not a citation under `strict = true`, which makes it the sanctioned way to keep a pointer this rule forbids. A citation a stub's target outside `[scan] include` lends the stubbed declaration ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)) is a citation site of its kind, reported at its line in the target, as with the target scanned ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)).

A `must not` rule judges the citation sites within its subject ([§FS-rules.2](FS-rules.md#2-subject-selectors)) instead; its sentence `<subject> must not cite any <target-set>.` reuses this code and the citation-site anchor ([§FS-rules.7.5](FS-rules.md#75-prohibition-and-recommendation-reuse)), and outside the index-entry repair below its message replaces only `(citation direction)` with `(<RULE-ID>)`; a config-derived duplicate keeps the config's message ([§FS-rules.6](FS-rules.md#6-semantic-deduplication)).

At a kind's own index entry ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) only the fix changes: in place of the `re-point` case it keeps the marked entry and changes the authority. Being in an index file does not make another citation an entry. The case names the authority that forbids it: `index-rule` names an independent rule sentence, `index-prohibition` the matching prohibition list and its table, and `index-may`, where a kind or global negative default supplies the prohibition ([§FS-config-v2.rules.citations.default](FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)), the citing kind's table and adding the target kind to its `may` list without removing existing permissions. For version 2 `REQ` with `default = "must-not"` and `may = ["GOAL"]` that is `[rules.citations.REQ]`, and the edit `may = ["GOAL", "REQ"]` clears the direction error while the marked entry still satisfies the index rule; a markerless link remains an index error. Adding to `may` never overrides an explicit prohibition or a rule. Both producers, config and rule sentence, give this repair, `warn-not` too on its warning, while `should-not` keeps its suggestion with no repair suffix. The unused-count exemption of [§FS-check.4.1](FS-check.md#41-unused-declaration) changes no direction verdict.

### 3.13 Number-only shorthand citation

A recognized shorthand citation ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) persisted in a scanned file is governed by the target project's `[reference] shorthand` policy ([§FS-config.3.1](FS-config.md#31-reference--citation-form)). Under the default `canonical`, a uniquely resolving site is a `shorthand-citation` error carrying its replacement ([§DF-number-only-citation-shorthand.2.3](../decisions/functional/DF-number-only-citation-shorthand.md#23-it-is-an-error-not-a-warning-or-a-suggestion)), and `grund fmt --write` applies the replacement in bulk ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)). Under `accepted` that site gives no shorthand finding, and its full citation may stand beside it in the same file. The policy changes only the unique result: a shorthand matching no declaration or several is an error under every policy, and no policy lets grund guess. A kind whose effective format has no `{number}` or no `{slug}` has no shorthand and never earns this finding ([§FS-check.1.2.1](FS-check.md#121-the-shorthand-shape)).

Where the text forbids the rewrite, the resolving form does not fire ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite)). A Python docstring is not a string literal for that: its content is judged, so a shorthand anywhere inside one is reported and rewritten exactly like one in a `#` comment ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)).

A site earns at most one shorthand finding, in one of three forms:

```text
docs/notes.md:1: error: shorthand citation §FS-042 is ambiguous: FS-042-user-login, FS-042-user-logout; write the full ID it means
docs/notes.md:3: error: shorthand citation §FS-999 matches no declaration; write the full ID it means
web/docs/notes.md:1: error: shorthand citation §api/FS-042; write §api/FS-042-session
```

A missing section ([§FS-check.3.2](FS-check.md#32-missing-section)) or a forbidden direction ([§FS-check.3.12](FS-check.md#312-forbidden-citation)) is a separate fact about the same citation and is reported separately. The ambiguous form names every match, sorted, and resolves none, because choosing one would be a guess ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)). Duplicate *numbers* are no error by themselves: `FS-042-user-login` may stand beside `FS-042-user-logout` as long as nothing abbreviates them, and [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) catches only duplicate full IDs. The marker in the message is the configured one ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), and the qualified form names its alias so the replacement can be pasted as written.

#### 3.13.1 Where the text forbids the rewrite

Where the text exempts a shorthand from the resolving form of this error is [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite).

### 3.14 Out-of-scope unresolvable citation *(`--full` only)*

Under `--full` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), a citation in a file outside the default scope that resolves to nothing is an error there too: an ID declared nowhere ([§FS-check.3.1](FS-check.md#31-dangling-citation)), a section its declaration does not have ([§FS-check.3.2](FS-check.md#32-missing-section)), an unknown alias ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)), and a number-only shorthand that matches zero or several declarations ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). So is a declaration-local section citation, in each shape of [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation), a verdict no consumer can settle from outside the file holding the token, and so is a marked token its kind's format does not spell and nothing declares ([§FS-check.checks.off-format-citation](FS-check.md#checksoff-format-citation-off-format-citation)). A citation whose declaration is also out of scope resolves normally, because this reports citations that point at nothing, not citations that point outside the default scope.

What is reported out there is a verdict on a citing token, reached from that token, the declaration structure of the file holding it and the resolved graph, and from nothing the project assigned the file: not its kind, its home or its citation obligations, and no convention a configured key switched on ([§DF-check-full-scope.2.2](../decisions/functional/DF-check-full-scope.md#22-out-of-scope-findings-are-the-verdicts-a-citing-token-and-the-files-own-structure-settle)). The rules [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full) keeps to the default scope do not run there, and one declaring-side finding does besides the six below: [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration), with its own unprefixed code and its ordinary message. A verdict about a rewrite `grund fmt` did not make is withheld there, because `fmt` scopes by the default scope too and an error never names a fix `grund` declines to make ([§FS-check.requirements.no-declined-fix](FS-check.md#requirementsno-declined-fix-an-error-never-names-a-fix-grund-declines-to-make)): [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)'s resolving form and [§FS-check.3.15](FS-check.md#315-shorthand-citation-in-a-numeric-run)'s finding are not reported, and [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation)'s finding drops its `` ; run `grund fmt --write` `` clause.

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

A bare, marked citation of a covered ID ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) in its kind's index file, where the index holds no entry for that ID, is `unlinked-index-entry` if the next `grund fmt --write` would wrap it ([§FS-fmt.6.1](FS-fmt.md#61-scope), [§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped), [§FS-fmt.6.5](FS-fmt.md#65-interaction-with---marker)). An ID is reported once, at the first such citation in file order:

```
docs/fs/README.md:3: error: index entry §FS-001-login is not a link; unchecked in grund 0.11.0, an error in 0.12.0 — run `grund fmt --write`
```

The repair is the link of [§FS-fmt.6.2](FS-fmt.md#62-form) ([§DF-index-entry-form.2.1](../decisions/functional/DF-index-entry-form.md#21-the-entry-is-a-recognized-citation-of-the-id-written-as-a-full-markdown-link)), and `check` judges an entry's shape, never its target ([§FS-citations.wrap.shape](FS-citations.md#wrapshape-the-shape-never-the-target)). Nothing else in the index is reported here, because an error never names a fix `grund` declines to make ([§FS-check.requirements.no-declined-fix](FS-check.md#requirementsno-declined-fix-an-error-never-names-a-fix-grund-declines-to-make)). A citation inside a link its author wrote is an entry where the link goes somewhere, and otherwise no entry ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)); any other bare citation `fmt` would not wrap is no entry. An ID the index then holds no entry for falls to [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index), whose fix is an edit by hand, so one cause never yields both findings.

#### 3.17.2 The shape, never the target

That `check` requires the wrap's shape and never its target is [§FS-citations.wrap.shape](FS-citations.md#wrapshape-the-shape-never-the-target).

### 3.18 Declaration missing from its kind's index

A kind with a `folder` and an index file — `README.md` unless `index` names another or opts out ([§FS-config.3.4.2](FS-config.md#342-index--the-kinds-index-file)) — promises that the index lists the folder's declarations. Each covered declaration the index does not list is one `missing-index-entry` error, located at the declaration's heading and naming the index file. Decided in [§DF-index-entry-form](../decisions/functional/DF-index-entry-form.md#df-index-entry-form-an-index-entry-is-one-full-link-per-id-and-nothing-else-about-the-page), [§DF-index-compatibility-ramp](../decisions/functional/DF-index-compatibility-ramp.md#df-index-compatibility-ramp-a-findings-ramp-follows-its-fix-command-not-the-size-of-the-offence), and [§DF-index-not-an-inbound-citation](../decisions/functional/DF-index-not-an-inbound-citation.md#df-index-not-an-inbound-citation-an-index-entry-is-navigation-not-use).

A declaration is covered when its kind declares IDs, since a `citable = false` kind has no index ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), and it has a declaration site anywhere in the folder's subtree, not only its top level. A stub under the folder puts its ID there, and one entry satisfies a stub-and-inline pair ([§FS-list.2](FS-list.md#2-behaviour), [§DF-index-entry-form.2.5](../decisions/functional/DF-index-entry-form.md#25-the-walk-is-the-folders-whole-subtree-and-a-stub-and-inline-pair-collapses)). A declaration of another kind in the folder is [§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)'s and is not demanded here.

An entry is one recognized citation ([§FS-check.1.1](FS-check.md#11-recognized-citations)) of the ID in the index file, written as a full Markdown link: a wrap ([§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets)), or a citation in a link its author wrote that goes somewhere, in any form of [§FS-citations.wrap.enclosing.shape](FS-citations.md#wrapenclosingshape-which-links-enclose) ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)). A link is an entry whatever `fmt` would do with it. A bare citation `fmt` would not wrap ([§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link)) is no entry, so an index that mentions the ID only that way is reported here and never under [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link). Layout is free — table or list, grouped or flat, in any order, with any prose around it ([§DF-index-entry-form.2.4](../decisions/functional/DF-index-entry-form.md#24-layout-is-free)) — and one link per ID is enough: every other occurrence of the ID in the index is never a finding ([§DF-index-entry-form.2.3](../decisions/functional/DF-index-entry-form.md#23-one-link-per-id-not-every-mention)).

An index may also enroll one external source declaration, with no stub under `folder`: an unqualified, marked citation of the bare ID whose declaration is in a non-Markdown source file outside `folder`, wrapped as exactly the link `grund fmt --cross-refs` derives from that index to the source home ([§FS-fmt.6.2](FS-fmt.md#62-form)). That link is both the membership and the entry ([§DF-index-entry-form.2.7](../decisions/functional/DF-index-entry-form.md#27-a-canonical-bare-id-link-enrolls-an-external-inline-declaration)). Nothing else enrolls. A number-only shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)), a citation in a link its author wrote, whatever its destination, a citation-only image ([§FS-citations.checks.image-index-entry](FS-citations.md#checksimage-index-entry-a-citation-only-image-is-no-index-entry-a-spent-check)) and a citation on a declaration heading ([§FS-citations.recognized.title-lines](FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names)) are ordinary citations, which [§FS-check.4.1](FS-check.md#41-unused-declaration) counts as uses. Where two kinds share one `folder` and `index`, a link enrolls for the kind its ID names. Removing the link removes the membership, so nothing is reported for an external declaration the index no longer claims.

Where the index mentions the ID but holds no entry for it, the message says the ID appears there but not as an entry; otherwise it says the ID is not listed and gives an entry to paste:

```
docs/fs/FS-001-login.md:1: error: FS-001-login appears in docs/fs/README.md but not as an entry: an entry is a `§`-marked Markdown link to the declaration — became an error in grund 0.13.0
docs/fs/FS-002-session.md:1: error: FS-002-session is not listed in docs/fs/README.md: an entry is a `§`-marked Markdown link to the declaration; write `[§FS-002-session](FS-002-session.md#fs-002-session-user-session)` in docs/fs/README.md — became an error in grund 0.13.0
```

The form both wordings name follows `[reference] strict`: the marked link above under `strict = true`, and off strict mode, where a bare token is a citation too ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), a Markdown link to the declaration whose text is the ID, with or without the marker. It is a form that is enough, not the definition of an entry. The entry to paste has the marker and the ID as its text and the link `grund fmt --cross-refs` writes from that index as its destination ([§DF-index-entry-form.2.2](../decisions/functional/DF-index-entry-form.md#22-full-link-is-the-link-fmt-would-write-here-not-has-an-anchor)); written in, it clears this finding, raises no [§FS-check.3.17](FS-check.md#317-index-entry-is-not-a-link) one, and the next `grund fmt --write` leaves it alone. Where `fmt` derives no link — an ID declared more than once, or a broken stub ([§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)) — the message names the form alone. An external source declaration never reaches the not-listed wording, since its canonical link is its entry.

The index mentions the ID when one of its lines carries the ID as `grund` renders it, as a whole ID-shaped token with or without the marker or a section suffix, or the whole file name of the declaration the finding is located at. Lines inside a fence that `[schema.sources] fenced` does not select ([§FS-languages.fence](FS-languages.md#fence-fences)), and declaration headings, are not looked at; inline code is. A mention raises and satisfies nothing, and only chooses the words.

A missing index file is this same finding, once for each declaration in the folder. Its message is the not-listed wording, never the appears one, with a parenthesis after the index's name saying why it could not be read — `(the index file does not exist)`, `(the index file is a directory)` or `(the index file could not be read)` — and the form and the entry after it. The rule is skipped for an index file the run did not scan, such as one the `[scan]` set excludes, while a missing index, or a directory in its place, is still reported; a path scope reads every index the whole run reads ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)).

### 3.19 Orphan name-bearing section path

Moved to [§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path). This address is kept so citations written before the move still resolve.

### 3.20 Invalid value declaration

What [§FS-values.2](FS-values.md#2-value-declarations) refuses as a value is `invalid-value-declaration`. In a kind opted into whole values ([§FS-values.1](FS-values.md#1-per-kind-opt-in-and-identity)), that is a readable Markdown or value-source declaration that breaks it, located at the exact invalid heading, key or element. In any kind, it is also an embedded tag in an invalid place or a marked root of an invalid shape ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)), and content a declared value chapter may not hold or a chapter root whose run is invalid ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)), each located at the tag, line or root heading those points name. A duplicate declaration keeps [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) instead, a duplicate section may carry this finding beside its own, and a value source [§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output) counts as incomplete makes the run exit `2` ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)).

### 3.21 Invalid value binding

An attempted backtick-delimited binding that breaks the grammar of [§FS-values.3.1](FS-values.md#31-the-only-binding-grammar) is `invalid-value-binding`, located at the attempted form. So is one whose literal closes its line onto a next-line citation ([§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts)), one aimed at a declared value chapter heading itself or below one of a root's immediate components, and one aimed at a root one of whose components contains an ASCII space ([§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)). What is not an attempt, and stays ordinary prose and citations, is [§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts)'s.

### 3.22 Value mismatch

After ordinary citation and section resolution succeed uniquely, a binding whose authored component differs from its whole declaration or marked root under [§FS-values.4](FS-values.md#4-exact-equality) — or, aimed at the root, whose literal does not split into that root's components ([§FS-values.5.2](FS-values.md#52-fixed-value-errors)) — is `value-mismatch`, located at the binding and naming the component site: the first unequal component's, or component 1's when the counts differ. Its canonical text and NDJSON parity are [§FS-values.5](FS-values.md#5-resolution-findings-and-exit-status)'s. Every resolution, declaration and section finding [§FS-values.5.1](FS-values.md#51-resolve-before-comparison) lists runs first and suppresses the comparison at that site, so one bad citation is never also reported as a mismatch.

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

The full citation joins the owner and the path with `[id] section_separator`. The path and the escape keep the `.` a local path is always written with ([§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)), and the escape is the token with its configured marker in angle brackets, `<@>5.1` under `@`. The absent-target wording is printed exactly where [§FS-check.3.2](FS-check.md#32-missing-section) reports `missing section` for the same coordinate on the same line, a stub's sections read as [§FS-check.3.2](FS-check.md#32-missing-section) reads them, in any context.

`` ; run `grund fmt --write` `` follows the release clause only at an owned site whose section exists and whose text permits the rewrite, inside the default scope. It is withheld in the three contexts of [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite), in a Markdown file's front matter ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)), whose citations belong to the file's first declaration, and outside the default scope under `--full` ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)), where the error still fires with its manual remedy; and at an ownerless, ambiguous or unsupported token, which [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical) cannot expand, and at an absent target section, which it does not rewrite ([§FS-fmt.2.4.6](FS-fmt.md#246-an-absent-target-section-withholds-this-rewrite-and-only-this-one)). A `[fmt] exclude` file ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes)), a `grund:fmt off` region and a file reached as an external file-symlink target ([§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)) keep the clause though `fmt` writes nothing there, because the author can lift that refusal ([§FS-check.requirements.no-declined-fix](FS-check.md#requirementsno-declined-fix-an-error-never-names-a-fix-grund-declines-to-make)).

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

The claim is the ancestor climb of [§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run), asked of the directory the scan found: it reads the `members` and `optional_members` entry text alone and climbs past the run's root, and a tree with no `[workspace]` block above it at all is the same case. Both discovery names are probed at each scanned directory, so a `.agents/grund.toml` block is found though the scan enters no hidden directory. The question is asked quietly, without [§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)'s undecidable-claim warning. An ancestor that names the block and cannot answer — a member list that will not expand, a config that will not parse — or whose `members` or `optional_members` text cannot be obtained at all leaves the block unreported ([§FS-workspace.6.1.8](FS-workspace.md#618-a-block-no-enclosing-block-lists-is-outside-the-chain), [§FS-workspace.6.1.7](FS-workspace.md#617-a-claiming-block-that-cannot-answer-fails-the-run)); a block no ancestor names is never silenced by an ancestor's breakage, because the claim is read off the entry text before anything is expanded.

A nested `grund.toml` with no `[workspace]` table is ordinary tree ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), a listed block is claimed at any depth, and a project root of this run — its own root and each member root the scan stops at ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)) — is never a candidate. Only the outermost unlisted block of a chain is reported, since it claims the blocks below it, and a block the scan reached under two spellings, such as through a directory symlink, is one finding at the spelling the scan met first. `include_root = false` on the block changes nothing ([§DF-unlisted-workspace-block.2.5](../decisions/functional/DF-unlisted-workspace-block.md#25-include_root--false-on-the-block-changes-nothing)); what it costs the block's own files is [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread), which can fire on the same block.

Every command whose run scans a project tree reports it — `check`, [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id), [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id), [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file), [§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk) and the ID read of [§FS-show](FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id) — because each spells projects from the same map ([§DF-unlisted-workspace-block.2.3](../decisions/functional/DF-unlisted-workspace-block.md#23-every-command-that-walks-carries-it-not-check-alone)). What it sees is the scan the run already makes: the roots of the default scope and their subtrees, minus member boundaries and what `[scan] exclude`, the ignore files and hidden directories prune below a root ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned), [§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)). A block outside that scan — under a narrowed `grund check <path>` ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), in a pruned directory, or behind a member boundary — is not reported, a known limit ([§DF-unlisted-workspace-block.2.2](../decisions/functional/DF-unlisted-workspace-block.md#22-the-scope-is-the-walk-the-run-already-makes-and-the-residue-stays-recorded)), and `grund check --full` may report a block the plain run does not ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)).

In `check` it is a located error at the block's `[workspace]` line, on stdout ([§FS-check.2.1](FS-check.md#21-report-format)) and under `--format=json` with that `path` and `line` and `sites` `null` ([§FS-errors.5](FS-errors.md#5-json-format)), and the editor publishes it from there ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)). The other five surfaces have no error channel for a fact about the run, so each prints one CLI-level `warning:` on stderr with the location inside its text ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), and its exit code does not move ([§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed)). Decided in [§DF-unlisted-workspace-error-shape](../decisions/functional/DF-unlisted-workspace-error-shape.md#df-unlisted-workspace-error-shape-check-and-the-editor-take-the-located-shape-the-five-walking-surfaces-keep-the-cli-level-line).

```
b/grund.toml:7: error: this [workspace] is listed by no enclosing workspace — the projects under it are absorbed into `root` instead of named under their own alias path; add "b" to [workspace] members in grund.toml, or keep it out of that project's [scan] — an unlisted [workspace] became an error in grund 0.15.0
warning: b/grund.toml:7: this [workspace] is listed by no enclosing workspace — the projects under it are absorbed into `root` instead of named under their own alias path; add "b" to [workspace] members in grund.toml, or keep it out of that project's [scan] — an unlisted [workspace] became an error in grund 0.15.0
```

Paths are spelled from the run's report base ([§FS-errors.3](FS-errors.md#3-message-text)); the entry to add is the block's directory relative to the enclosing project's root, where both remedies are written; and the absorbing project is named by the alias path this run spells it with, as [§FS-list](FS-list.md#fs-list-grund-lists-every-declared-id) prints it. The second remedy names an outcome rather than a key because `[scan] exclude` never prunes the directory a scan starts at ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)): a block that is itself an `include` root is cleared through `include`, and one below a root through `exclude`.

### 3.30 A workspace member swallows the block's own scan

A `[workspace]` block every one of whose scan roots lies inside one of its own members ([§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan)) reads nothing: its declarations are unreachable and its citations are never checked. Every command that resolves the block's member boundary — `check`, `list`, `refs`, `show`, `cover`, `fmt` and the rest — refuses its configuration with the error [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) defines for a bad `members` line, which exits `2` and stays text under `--format json` ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)); like every config error it carries no code, so no selector reaches it ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). Only `grund init` does not refuse: when expanding the workspace above its target fails, it leaves its Workspace members section out and the error to `check`, and exits as it otherwise would ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)). Decided in [§DF-absorbed-scan-warning](../decisions/functional/DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error).

```
error: grund.toml:16: [workspace] members swallows this project's whole scan — every scan root is inside a member: `docs` in `docs` — so its declarations are unreachable and its citations are never checked. Point [scan] include at a directory that is not a member, or set include_root = false; this became an error in grund 0.16.0
```

The message names each covered root beside the member entry it is inside, in config order. The entry is spelled as the config wrote it, and the root by its path under the block root, folded lexically on the spelling and never by reading the disk, so `./docs/` is `docs`, `docs/../src` is `src`, and a root that folds to the block root itself, such as `docs/..`, or `../group` in a block at `group/`, is `.` ([§FS-errors.4](FS-errors.md#4-determinism)).

The question is asked of every block in a nested tree against its own `members` line, and of the run's root block before its nested members expand, so where a later expansion refusal would also apply, this is the error reported and the run stops there, [§FS-config.4.3](FS-config.md#43-invalid-config-behavior) reporting the first problem only. Every line is rendered from the root the run was launched at, for the blocks above it, at it and below it alike ([§FS-errors.3](FS-errors.md#3-message-text)).

## 4. Warnings

A warning is a finding on the `warning:` channel, reported as [§FS-check.2.1](FS-check.md#21-report-format) says, or as a CLI-level message where its section says so ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)); a retained warning stands in place of the `success` line ([§FS-check.2.1.3](FS-check.md#213-the-success-line)) and never moves the exit code ([§FS-check.2](FS-check.md#2-outputs)).

### 4.1 Unused declaration

A declared ID that nothing cites earns `unused` ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)) at its declaration. A citation counts where the run's resolution scope reads it ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)): a declaration cited only from a file outside an explicit path is cited and earns no warning ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)), and one cited only from the wider tree `--full` adds keeps it ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)). A number-only shorthand that resolves counts like any other citation ([§FS-check.1.2.4](FS-check.md#124-a-resolved-shorthand-is-a-real-edge)). `E2E` declarations ([§FS-declarations.e2e-case](FS-declarations.md#e2e-case-what-an-e2e-case-directory-declares)) are exempt, because a case is exercised by being run, not by being cited; every other kind is held to it, Markdown and value-source declarations included ([§FS-values.terms](FS-values.md#terms-terms)). `grund list --unused` reads the same signal, and what it does with a configured `E2E` kind's cases is [§FS-list.1.3](FS-list.md#13---unused)'s.

In a kind's own index, the entry of a declaration under that index's folder ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)), every further plain citation of that declaration there, and the canonical link that enrolls an external source declaration ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)) are navigation, not use, and do not count ([§DF-index-not-an-inbound-citation.2.1](../decisions/functional/DF-index-not-an-inbound-citation.md#21-a-kinds-own-index-entry-does-not-count-as-an-inbound-citation), [§DF-index-not-an-inbound-citation.2.2](../decisions/functional/DF-index-not-an-inbound-citation.md#22-the-exclusion-is-the-entry-not-the-file)). Every other citation there counts, here and under `grund list --unused`: one of an ID whose home lies outside the folder, an enrolled ID's other citations included; one that names a section of the ID, such as `DOC-offer.values.price`; the citation inside a recognized value binding ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts)), one aimed at a whole value root included ([§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)), or inside an attempt at one `check` refuses ([§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts)); and one riding on a declaration heading, which is a citation from the declaration that heading declares ([§FS-citations.recognized.title-lines](FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names)). Every excluded site stays in `grund refs`, labelled in text `refs` ([§FS-refs.3.1](FS-refs.md#31---format-text-default)).

An unused declaration with excluded index sites names them after its ID, in path, line, then column order, either every site or the first and `N more`, each path spelled from the report's base even when relative paths are off ([§FS-errors.3](FS-errors.md#3-message-text)):

```text
docs/competition/COMP-zz-a.md:1: warning: declared but never cited: COMP-zz-a (its index entry at docs/competition/README.md:3 does not count as a use); cite it
```

### 4.2 Inline note soft-cap overrun *(opt-in)*

A note whose line count exceeds `inline_note_suggested_lines` and stays within `inline_note_max_lines` is an `inline-citation-style` warning where the project's config makes an overrun a warning (the row's *Severity*, [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)). What counts as an overrun, its message, and how it meets the hard-cap error of [§FS-check.3.10](FS-check.md#310-inline-citation-style-violation) are [§FS-inline-citation-style.4.2](FS-inline-citation-style.md#42-warnings--opt-in-soft-cap)'s.

### 4.3 Redundant config pair

A directory that carries both a bare `grund.toml` and `.agents/grund.toml` is configured by the bare file alone ([§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both)), and `redundant-config` ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)) reports the other file as ignored ([§DF-config-file-location.2.2](../decisions/functional/DF-config-file-location.md#22-the-bare-grundtoml-wins-a-tie-and-check-warns-about-the-pair)), in a CLI-level warning ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)) that names both files as report paths ([§FS-errors.3](FS-errors.md#3-message-text)) and under `--format=json` is one warning finding on stderr with `path`, `line` and `sites` null ([§FS-errors.5.2.3](FS-errors.md#523-run-level-findings-in-checks-report)):

```text
warning: .agents/grund.toml is ignored — grund.toml takes precedence; delete one
```

`grund check`, `grund config validate` and `grund config show` print it ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path), [§FS-config.4.2](FS-config.md#42-grund-config-show-path)), and no other command does. A config read from `.agents/grund.toml` with no bare file beside it earns [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location) instead.

### 4.4 Inline note layout deviation *(opt-in)*

Each line of a note that [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit) judges and that does not match the configured `inline_note_layout` is an `inline-citation-style` warning, located at that line, where the project's config makes a deviation a warning (the row's *Severity*, [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)); where the config makes it an error, the identical message is [§FS-check.3.10](FS-check.md#310-inline-citation-style-violation)'s error instead ([§DF-inline-note-layout.2.1](../decisions/functional/DF-inline-note-layout.md#21-two-keys-the-style-and-the-level-it-is-enforced-at)). The form, the per-line rule and the sites that carry no note are [§FS-inline-citation-style.3.3](FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)'s, and the levels are [§FS-inline-citation-style.4.4](FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)'s.

### 4.5 Nothing recognized

A run that read at least one file and recognized no declaration and no citation in them earns `nothing-recognized` ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), one CLI-level warning ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)). It is [§FS-check.2.2](FS-check.md#22-empty-scan)'s empty scan one step further in: the scope was right and the files were read, but the grammar matched none of their content, as in a tree nobody has declared in yet or one whose headings open with a prefix that is no configured kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)). Decided in [§DF-nothing-recognized](../decisions/functional/DF-nothing-recognized.md#df-nothing-recognized-a-run-that-recognized-nothing-says-so-and-says-it-as-a-warning).

The message names the number of files read, the configured kinds, and the declaration and citation shapes of the global ID template, rendered by [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)'s substitution with the configured marker and labelled as [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure) labels the template, `[id] format` under version 1 and `[schema] id_format` under version 2; it shows that global template even where a kind sets a format of its own. It never assembles an example ID or guesses a corrected one ([§DF-nothing-recognized.2.3](../decisions/functional/DF-nothing-recognized.md#23-the-caution-names-shapes-never-a-corrected-id)). A project that declares no kinds ([§FS-citations.recognized.no-kinds](FS-citations.md#recognizedno-kinds-with-no-kind-no-token-is-a-citation)) is told that instead, and pointed at the command that explains a kind's keys in its config version, `grund config explain '[[kinds]]'` under version 1, which answers before any kind is declared ([§FS-config-explain.1.2](FS-config-explain.md#12-rungs)):

```text
warning: nothing recognized — grund read 3 files and found no declaration and no citation in them. A declaration heading reads `# <KIND>-<slug>: <title>` and a citation `§<KIND>-<slug>`, under [schema] id_format = "{kind}-{slug}" with <KIND> one of {GOAL, FS}. Either nothing is declared yet, or the headings are written to a different shape than that.
warning: nothing recognized — grund read 1 file, and this project declares no kinds, so no heading in them is a declaration and no token a citation. Declare its first kind in grund.toml to start. See `grund config explain '[schema.kinds.<NAME>]'` for every key a kind takes.
```

It is asked per project, against that project's own config ([§FS-workspace.5](FS-workspace.md#5-command-scope)), and only by a run whose scope is that project's root: no path, or a path that resolves to it. A citation alone answers it, so a member that only cites another member's declarations earns nothing. It is withheld from a run that has any other finding about the default scope, and stands beside the four findings [§FS-check.2.2.3](FS-check.md#223-exit-code-json-and-what-suppresses-it) lets the empty-scan warning stand beside ([§DF-nothing-recognized.2.4](../decisions/functional/DF-nothing-recognized.md#24-it-is-withheld-from-a-run-that-has-any-other-finding)). A heading that comes close to being a declaration is named by [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss), and this warning is withheld beside it by the same rule.

### 4.6 Declaration near miss

Moved to [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss). This address is kept so citations written before the move still resolve.

### 4.7 A workspace member swallows the block's own scan

Moved to [§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan) when the warning became an error in grund 0.16.0. This address is kept so citations written before the move still resolve.

#### 4.7.9 A later expansion refusal does not discard it

Retired in grund 0.16.0: the absorbed scan is now a config error, so when a later expansion refusal would also apply, the run stops at the absorbed scan and reports only it ([§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)). This address is kept so citations written before the change still resolve.

### 4.9 A workspace member declared optional is absent

A member listed in `[workspace] optional_members` whose path is not a directory in this checkout ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)) contributed nothing: no declaration in it reached a catalog, no citation into it was resolved, and none is reported at its site ([§FS-workspace.4](FS-workspace.md#4-resolution)). The run covers less of the repository than it looks like, and `optional-member-absent` ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)) says so, one warning per absent entry, located at the `optional_members` line of the block that holds it, in the report's order ([§FS-errors.4.1](FS-errors.md#41-ordering)), which puts the entries of one line in name order:

```text
grund.toml:5: warning: optional workspace member `vendored` is absent — citations into namespace `vendored` were not checked, so this run does not cover it
grund.toml:9: warning: optional workspace member `packages/secret` is absent — citations into namespace `secret` were not checked, so this run does not cover it, nor the managed blocks' workspace sections
```

The entry is named as the config wrote it, which is what an author edits ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), and the namespace by the whole alias path this run spells it with, which is what a citation writes ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)): `vendored` and `sub/vendored` for one entry of a nested block. The second form is for a run that verified at least one managed block ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)), which leaves the blocks' workspace sections out of the comparison ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)).

It is a located finding in `check`'s report on stdout, not a CLI-level message, and no other command prints it ([§DF-optional-workspace-members.2.4](../decisions/functional/DF-optional-workspace-members.md#24-exit-0-and-the-announcement-is-what-buys-it)). It never becomes an error, because the repository declared the absence in advance ([§DF-unread-opted-out-block.2.3](../decisions/functional/DF-unread-opted-out-block.md#23-what-makes-a-workspace-finding-ramp-and-what-makes-one-permanent)).

### 4.10 `include_root = false` leaves the block's own files unread

A `[workspace]` block that sets `include_root = false` is not a project, so nobody reads its own files: a declaration there reaches no catalog and a citation there is never checked ([§FS-workspace.6.1.2](FS-workspace.md#612-include_root-on-a-block)). When such a block puts a member in scope and its own tree holds a file it would have read as a project, every command that loads the workspace ([§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan)) prints one CLI-level warning for it ([§FS-check.2.1.1](FS-check.md#211-cli-level-messages)), located at its `include_root` line, where the editor publishes it too ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)). "Would have read" is asked of the block's default scope, whatever `--full` says: its `[scan] include` roots and scanned kind homes that exist and lie outside its members, under its own extensions, excludes, ignore files and hidden-name rule ([§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads)). Decided in [§DF-unread-opted-out-block](../decisions/functional/DF-unread-opted-out-block.md#df-unread-opted-out-block-the-unread-files-of-an-opted-out-block-are-a-conditional-warning-that-never-ramps).

```text
warning: group/grund.toml:17: no project scans `docs`, so its citations are never checked. Set include_root = true, or configure another project's source scope to read it.
```

The warning names one unread root, the first in scope order (`[scan] include` in config order, then the kind homes), as a normalized path under the block, never a resolved one, and `.` when that root is the block itself; the config line is spelled from the run's root ([§FS-errors.3](FS-errors.md#3-message-text)). Each block is reported once per run, the block the run is rooted at first and the blocks below it in the order the run reaches them, and a run rooted inside a member says nothing of the blocks above it. It keeps its text under `--format json`, as every launch-time message does ([§FS-errors.5.2.2](FS-errors.md#522-launch-time-messages-stay-text)), and carries no code and no selector of its own ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). A block with no member in scope is a config error where it lists no `optional_members` entry ([§FS-workspace.6.1.2](FS-workspace.md#612-include_root-on-a-block)), and earns [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent)'s warning where all its members are absent optional ones ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)). A failed workspace expansion withholds this warning for the blocks below the run's root, since the run then exits `2` on that error.

Opting out is a legitimate choice, so a block whose tree holds nothing it would read earns nothing, and the warning never becomes an error, because neither remedy records that the author meant it ([§DF-unread-opted-out-block.2.2](../decisions/functional/DF-unread-opted-out-block.md#22-a-warning-permanently-with-no-release-it-becomes-an-error-in), [§DF-unread-opted-out-block.2.3](../decisions/functional/DF-unread-opted-out-block.md#23-what-makes-a-workspace-finding-ramp-and-what-makes-one-permanent)).

### 4.11 Config read from the deprecated `.agents/` location

The config this run read is an `.agents/grund.toml`, a location that works, is deprecated and is never removed ([§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated)), so `deprecated-config-location` ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)) names the file it read and the bare `grund.toml` it should move to ([§DF-config-file-location.2.5](../decisions/functional/DF-config-file-location.md#25-the-agents-form-is-deprecated-and-never-removed)):

```text
warning: .agents/grund.toml is a deprecated config location — move it to grund.toml
```

It is [§FS-check.4.3](FS-check.md#43-redundant-config-pair)'s finding in everything but its trigger: its form, the commands that print it and its JSON shape are that section's. A directory that also carries a bare `grund.toml` earns [§FS-check.4.3](FS-check.md#43-redundant-config-pair) and never this ([§FS-config.1.2](FS-config.md#12-the-agents-location-is-deprecated)).

Each config the run loaded from an `.agents/` path earns one line, at the path that config was loaded under ([§FS-errors.4](FS-errors.md#4-determinism)). A workspace run loads the root's config, each project the workspace expansion returns, and each block with `include_root = false` the expansion reaches, at any depth and whether or not any of its members is present, though such a block is no project ([§FS-workspace.2.1](FS-workspace.md#21-a-member-that-swallows-the-blocks-own-scan), [§DF-grouping-block-config-findings](../decisions/functional/DF-grouping-block-config-findings.md#df-grouping-block-config-findings-at-a-workspace-root-check-and-config-validate-name-the-config-findings-of-each-include_root--false-block-the-expansion-reaches)); an imported tree's blocks are named by that tree's own run, not its consumer's ([§FS-imports.mount](FS-imports.md#mount-an-import-is-a-mounted-root-read-live-and-read-only)). `grund check` and `grund config validate` name that set at a workspace root ([§FS-config.4.1](FS-config.md#41-grund-config-validate-path)); `grund config show` names only the config it prints, the root's there ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)), and a path inside a member names that member's config alone on all three, nothing of a block above it.

### 4.12 Missing snapshot

A recognized citation with no declaration, of a fetched kind ([§FS-config.3.4.10](FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)), earns `missing-snapshot` in place of [§FS-check.3.1](FS-check.md#31-dangling-citation)'s error where the project's config turns it on for that kind (the row's *Enabled by*, [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), one finding per site:

```text
docs/guide.md:3: warning: no snapshot for TICKET-1234 in docs/tickets.md — run grund fetch TICKET-1234
```

In a workspace the ID and the fetch command spell the complete alias-qualified ID, and the home is spelled from the run's report base ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)); an ID reached through an import is fetched as the ID below the last import edge, in that edge's target ([§FS-imports.writes.answer](FS-imports.md#writesanswer-where-a-qualified-id-is-fetched-has-one-answer)). Its remedy is the row's *Fix*, which takes each case from `dangling`'s, so the fetch tail gives way to a near ID's or an escaped illustration's hint ([§FS-check.3.1](FS-check.md#31-dangling-citation)), though never to a member candidate, which only the error offers, and to the remedies for a home `fetch` refuses and a key it has nothing to run for ([§FS-check.3.1](FS-check.md#31-dangling-citation)). The citation stays in what `refs` and `cover` report. A site whose home this run could not read earns neither this warning nor [§FS-check.3.1](FS-check.md#31-dangling-citation)'s error ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)).

### 4.13 Oversized lead *(opt-in)*

Moved to [§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in). This address is kept so citations written before the move still resolve.

### 4.14 Unmarked Markdown heading

Moved to [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading). This address is kept so citations written before the move still resolve.

## 5. What grund does not check

See [§FS-non-goals](FS-non-goals.md#fs-non-goals-what-grund-will-deliberately-not-do) — in particular [§FS-non-goals.1](FS-non-goals.md#1-markdown-link-validation) (Markdown links and URLs), [§FS-non-goals.2](FS-non-goals.md#2-spelling-grammar-prose-quality) (spelling and grammar outside the explicit value form), and the convention that ID numbers are stable names, not ordinal positions. Value checking adds only the exact binding of [§FS-values.3.1](FS-values.md#31-the-only-binding-grammar), and none of what [§FS-values.9](FS-values.md#9-compatibility-and-explicit-exclusions) excludes.

## 6. Watch mode (`--watch`)

`grund check --watch [<path>]` checks immediately, then stays resident and checks again when an effective local input changes: the terminal counterpart to [§FS-lsp](FS-lsp.md#fs-lsp-grund-ships-an-optional-lsp-server), serving [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible). Decided in [§DF-watch-terminal-loop](../decisions/functional/DF-watch-terminal-loop.md#df-watch-terminal-loop-a-terminal-watch-loop-preserves-ordinary-check-reports).

### 6.1 Change detection

The watcher uses native filesystem notifications on Linux, macOS and Windows, with no polling fallback or polling interval; a mount or pseudo-filesystem that delivers no usable notification is unsupported, and a read or access event triggers nothing.

No change is lost: a change made while an input is being discovered, read or subscribed to, during the initial scan included, arranges another check against the latest state. While idle, a check runs 100 ms after the last relevant event or 500 ms after the first pending one, whichever comes first, so a burst of writes or an atomic save coalesces, though a long save can still expose an intermediate state. Scans and report publications are serialized, with at most one pending rerun, which sees the latest inputs.

It observes every effective local input that can change the equivalent one-shot result, paths outside the reported subtree included ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)):

- both `grund.toml` and `.agents/grund.toml` at every upward-discovery and ancestor-claim candidate, absent candidates and precedence changes included;
- workspace roots, nested members, member-glob parents and absent member candidates, and resolution-wide source roots and kind homes;
- value sources ([§FS-values.terms](FS-values.md#terms-terms)), agent entrypoints, indexes, stub targets and other checker probes, and followed file-link targets outside source roots;
- effective ancestor and nested `.gitignore` and `.ignore` files, `.git/info/exclude`, global Git excludes, and the `.gitconfig` and XDG inputs through which those are found.

Config, membership, ignores, catalogs and output defaults are re-resolved on each run, and creating, deleting, renaming or atomically replacing an input, changing members, and deleting and recreating a root are all seen. Source roots are watched recursively, so a broad root spends native watches on its ignored descendants too, and running out of them is a fatal watcher failure ([§FS-check.6.3.2](FS-check.md#632-fatal-watcher-failures), [§DF-watch-terminal-loop.3](../decisions/functional/DF-watch-terminal-loop.md#3-consequences-and-proof)).

A lost, overflowing or uncertain event, or a saturated internal queue, forces full rediscovery and a rescan. A path gone by the time it is subscribed is a missing input, which the rescan sees, and not a failed subscription. Any other failed subscription or failed recovery is fatal ([§FS-check.6.3.2](FS-check.md#632-fatal-watcher-failures)), and an ordinary input failure during the rescan stays a recoverable run ([§FS-check.6.3.1](FS-check.md#631-recoverable-runs)) where usable discovery coverage remains.

### 6.2 Each run is a plain `grund check`

Each run is the ordinary check with the same flags: it writes the stdout and stderr bytes, finding order and exit status that check writes on that tree state ([§FS-check.2](FS-check.md#2-outputs), [§FS-errors.4](FS-errors.md#4-determinism)), except the terminal controls of [§FS-check.6.2.2](FS-check.md#622-owned-terminal-screen). An explicit `--format` overrides the config's default, a default changed while watching included.

#### 6.2.1 Exact stream contract

A clean text run prints the ordinary `success` ([§FS-check.2.1.3](FS-check.md#213-the-success-line)); JSON is the ordinary finding NDJSON ([§FS-check.2.1.4](FS-check.md#214-json)), and a clean JSON run writes no record, so concatenated JSON shows neither clean runs nor every run boundary. No envelope, banner, timestamp or run record is added, every line keeps its ordinary stream, and a redirected stream appends the successive outputs.

#### 6.2.2 Owned terminal screen

Text owns an alternate screen only when stdout is an alternate-screen-capable terminal and stderr shares that terminal or is redirected. It enters through stdout with `\x1b[?1049h`, clears only that screen immediately before each publication with `\x1b[H\x1b[2J`, and restores through stdout with `\x1b[?1049l` on exit or before a JSON publication; these spellings denote the escape bytes, not backslash text.

Shared-terminal stderr is cleared with the report, and redirected stderr appends and receives no clearing bytes. A redirected stdout, distinct stdout and stderr terminals, and a dumb terminal append on both streams, with no enter, clear or restore bytes. JSON never enters or clears a screen: a change from text to JSON restores the owned screen before any JSON byte, and a later eligible text run can take it again. No stream changes owner. Exit restores the terminal's pre-watch content rather than keeping the last report, and a fatal failure restores first, then prints its error.

### 6.3 Lifecycle

The process runs until interrupted or until a watcher failure is fatal. It is non-interactive, with no TUI, key binding or prompt ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)), performs no network I/O ([§FS-non-goals.11](FS-non-goals.md#11-network-access-during-a-check)), launches no configured process and writes no input; beyond the files the scan reads, it observes only local metadata.

#### 6.3.1 Recoverable runs

Findings and config or read failures publish the ordinary status `0`, `1` or `2`, and the watcher stays resident for the repair: an invalid initial config leaves the config candidates watched, and a later failed run leaves the last usable run's inputs watched too, so the correction triggers the next run. A config-dependent trial-rule refusal is a recoverable run; a static invocation error ends the command before the loop starts ([§FS-check.6.4](FS-check.md#64-scope)).

#### 6.3.2 Fatal watcher failures

A failed subscription, at setup or at run time, or a failed watcher recovery restores any owned screen, prints a stderr `error:` naming the watching operation or input that failed, and exits `2`, whatever status the last completed check had.

#### 6.3.3 Interrupt and completion

On Ctrl-C or SIGINT the watcher discards pending work. A scan in progress finishes privately and its unpublished result is discarded; a publication already begun finishes, and counts as completed only once both streams are fully published and flushed, so shutdown may wait for it.

The exit status is the most recently completed run's, `0`, `1` or `2`; an unpublished interrupted result never changes it. If no run completed, the watcher restores any owned screen, prints `error: interrupted before the first check completed` on stderr and exits `2`. Nothing stays resident after exit. A native backend that has not released its subscriptions within 5 seconds of being closed, on refresh or at exit, is a fatal watcher failure ([§FS-check.6.3.2](FS-check.md#632-fatal-watcher-failures)), never an indefinite wait.

### 6.4 Scope

`--watch` is a `check` flag, `grund check --watch [<path>]` ([§FS-cli](FS-cli.md#fs-cli-grunds-command-line-surface-conventions)), and every other subcommand rejects it. Static invocation choices are validated once, before the first run, with the ordinary diagnostics and their precedence; config-dependent validity and output defaults are re-evaluated on each run. There is no daemon, service protocol or binding watch API.

## checks: Checks

Each section below is named by its finding code ([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)), and a code's severity is its row in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog). A located finding below is reported as [§FS-check.2.1](FS-check.md#21-report-format) says, over its token in the LSP ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)) and with the same code in the Python binding ([§GOAL-multi-language](../goals.md#goal-multi-language-same-engine-three-platforms)), and is selected by `--only` and `--ignore` ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), its exit code going with it when ignored.

### checks.glob-citation: Glob citation

A marked candidate that [§FS-check.1.1.11](FS-check.md#1111-a-glob-operator-never-shortens-a-marked-citation) reads as a pattern is reported once, at its marker's line and column, with code `glob-citation`:

```text
glob citation §<token>: a citation names one exact point, not a pattern; cite the point, or escape it as <§><token>; this became an error in grund 0.19.0
```

`<token>` is the candidate as written after the marker, and both `§` and `<§>` are spelled with the configured marker. `--full` keeps it to the configured scan scope, as other convention findings are, and outside that scope the pattern still records no citation. Decided in [§DF-glob-citation-lands-in-0-19](../decisions/functional/DF-glob-citation-lands-in-0-19.md#df-glob-citation-lands-in-0-19-a-glob-citation-is-an-error-from-0190).

### checks.off-format-citation: Off-format citation

A marked token that [§FS-check.1.1.12](FS-check.md#1112-a-marked-token-its-kinds-format-does-not-spell) reads as an off-format citation is reported once, at its marker's line and column, with code `off-format-citation`:

```text
off-format citation §<token>: <KIND> IDs follow <key> = "<template>" with <patterns>, and nothing declares this one; cite an ID in that format, or write <§><token> for an illustration — unchecked in grund 0.20.0, an error in 0.21.0
```

`<token>` is the token as written after the marker, its alias and section path included, and both `§` and `<§>` are spelled with the configured marker. `<KIND>` is the kind it opens with, and `<key> = "<template>"` is that kind's effective template with the key that owns it, in the vocabulary of [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure) and in the project the token names: `[schema] id_format` or `[schema.kinds.<KIND>] id_format` under version 2, `[id] format` or `[[kinds]] format` under version 1. `<patterns>` names the pattern of each component the template holds, in the template's order and joined by ` and `: `number_pattern = "<pattern>"` for `{number}` and `slug_pattern = "<pattern>"` for `{slug}`, each as configured, between double quotes and unescaped, under its bare key because the patterns are schema-wide. The message guesses no corrected ID ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)). Under `--full` a site outside the default scope is `out-of-scope-off-format-citation`, this message after the scope ([§FS-check.3.14](FS-check.md#314-out-of-scope-unresolvable-citation---full-only)). The closing clause is its release attribution ([§FS-distribution.4.2.2](FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)). Decided in [§DF-off-format-citation](../decisions/functional/DF-off-format-citation.md#df-off-format-citation-a-marked-token-its-kinds-format-does-not-spell-and-nothing-declares-is-reported-rather-than-dropped).

### checks.fs-home-moves: FS home moves (a removed check)

No finding has this code: the built-in `FS` home is `folder = "docs/functional-spec"` ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds), [§DF-fs-home-lands-in-0-19](../decisions/functional/DF-fs-home-lands-in-0-19.md#df-fs-home-lands-in-0-19-the-built-in-fs-home-is-docsfunctional-spec-from-0190)), and `--only fs-home-moves` or `--ignore fs-home-moves` is refused as any unknown code is ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).

### checks.config-version-1: A config file read as version 1

A config file read as version 1 ([§FS-config.5.3](FS-config.md#53-a-version-1-config-is-cautioned-and-never-retired)), whether it writes `grund_config_version = 1` or no version, earns one warning, code `config-version-1`, naming the file and the command that migrates it:

```text
warning: grund.toml is version 1; migrate it with grund config migrate --to 2
```

The leading path is the config file as a report path ([§FS-config.3.6](FS-config.md#36-output--report-format)). After `--to 2` comes the directory the file configures, as a report path: a project's root, the project root for an `.agents/grund.toml`, or an `include_root = false` block's own directory, left out when it is the report's base, which is why a single project's line ends at `--to 2`. `grund config migrate` prints the same sentence after `member: ` for a member still on version 1, its directory spelled from the working directory instead ([§FS-config-migrate.8](FS-config-migrate.md#8-workspaces), [§FS-config-migrate.6.5.1](FS-config-migrate.md#651-the-commands-a-migrate-report-prints-are-spelled-from-the-working-directory)). It fires for each config of the set a run names that is read as version 1 ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)), and never for a version 2 file or a project with no config file.

It is [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)'s finding in everything but its trigger and its text: a CLI-level `warning:` on stderr with no site ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)), carried by `check`, `config validate` and `config show` and no other command ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)), once per config, under `--format=json` one warning object on stderr with `path`, `line` and `sites` `null` ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)), standing in place of `success` and leaving the exit code alone ([§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)); a caller that does not want the line passes `--ignore config-version-1`. The lines take the report's order ([§FS-errors.4.1](FS-errors.md#41-ordering)), not load order: a version 1 member `aaa` prints before a version 1 root, and an `.agents/grund.toml` on version 1 earns [§FS-check.4.11](FS-check.md#411-config-read-from-the-deprecated-agents-location)'s line, then this one.

### checks.closed-citation-grounding: Grounding asks for citations a closed row permits nowhere

A scanned non-citable home earns one `closed-citation-grounding` warning per row when its citation row is closed to every citation ([§FS-config.3.9.4.1](FS-config.md#3941-a-row-closed-to-every-citation)) and grounding reaches it at any strength. The grounding predicate is the same `row_grounds_at_any_strength` that Citation directions reads ([§FS-init.2.3.5](FS-init.md#235-citation-directions)); the closed-row predicate is shared too. This adds no precedence rule and changes no citation-direction or grounding finding.

The grounding setting cannot be satisfied: inside a non-citable home an inline declaration does not ground a unit, and the file is always a unit, even for an `h2` or finer rung ([§FS-check.3.6.2](FS-check.md#362-the-unit)). The warning is permanent, with no release in which it becomes an error ([§DF-closed-citation-grounding-warns](../decisions/functional/DF-closed-citation-grounding-warns.md#df-closed-citation-grounding-warns-warn-when-grounding-asks-for-citations-a-closed-home-permits-nowhere)). A `must`, `warn` or `should` rung triggers it, with or without `--suggestions`; `may` does not. A warning or suggestion rung keeps its own severity and does not fail `check`.

No warning is earned by a complement, a citable row, an unscanned place, or an `include_root = false` block that scans no project. Source files outside a non-citable home can ground themselves with inline declarations ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)). An explicit target exception, including `may`, `should-not` or `warn-not`, makes the row open by the shared closed-row predicate. A v1 row's `require_grounding = false` overrides both the project default and the flag ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)). A v2 row's ladder replaces the project ladder whole; an empty ladder disables grounding even under the flag ([§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)).

The message names the config path, the row, the strongest effective grounding rung and its source, the effective `must-not` fallback and its source, and both remedies. Strength order is `must`, `warn`, `should`; the unit is the rung's configured unit, even where the home checks whole files. When the config already grounds the row, its setting is named even under `--require-grounding`; the flag is named only when the flag alone grounds it. The verb is `asks for citations` at every strength.

```text
warning: grund.toml: row `ce`: [rules.citations.ce.grounding] must = "file" asks for citations, but [rules.citations.ce] default = "must-not" permits no target; empty [rules.citations.ce.grounding] to disable grounding for this row, or permit a target in [rules.citations.ce]
warning: grund.toml: row `ce`: require_grounding = true on its [[kinds]] row asks for citations, but [citations.ce] default = "must-not" permits no target; set require_grounding = false on its [[kinds]] row to disable grounding for this row, or permit a target in [citations.ce]
```

The setting fragments are spelled in the file's version:

| Fact | Version 2 | Version 1 |
|---|---|---|
| Row grounding | `[rules.citations.ce.grounding] must = "file"` | `require_grounding = true on its [[kinds]] row` |
| Inherited grounding | `[rules.citations.grounding] must = "file"` | `[reference] require_grounding = true` |
| Flag alone | `--require-grounding` | `--require-grounding` |
| Row fallback | `[rules.citations.ce] default = "must-not"` | `[citations.ce] default = "must-not"` |
| Inherited fallback | `[rules.citations] default = "must-not"` | `[citations] default = "must-not"` |

`ce`, `must` and `file` stand for the actual row, strength and unit. The v1 row fragment uses the same spelling as `grounding_setting` ([§FS-init.2.3.5](FS-init.md#235-citation-directions)). The v2 remedy is `empty [rules.citations.ce.grounding]` for an own ladder, or `write an empty [rules.citations.ce.grounding]` for inherited or flag grounding, followed by ` to disable grounding for this row, or permit a target in [rules.citations.ce]`. V1 always advises `set require_grounding = false on its [[kinds]] row to disable grounding for this row, or permit a target in [citations.ce]`.

The shared config diagnostics carry the warning on `check`, `config validate` and `config show`, as a CLI-level `warning:` on stderr, in report order ([§FS-errors.4.1](FS-errors.md#41-ordering)). Each selected loaded project config earns its rows once, including empty homes; workspace root and members use their loaded config paths from the report base. `show` names only its own config; member-limited check and validate name only that member. Text has exactly one config breadcrumb, and all three commands print the same warning bytes.

Under `check --format=json` it is one structured stderr warning with code `closed-citation-grounding`; `path`, `line` and `sites` are null, as for [§FS-check.checks.config-version-1](FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1), and the message retains its config breadcrumb. It leaves every exit code unchanged and replaces `success` when it is the only finding. Normal selection applies ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)): `--ignore closed-citation-grounding`, `--only` another code, or `--only-rule` filters it out. Its catalog *Enabled by* is `—`, so selecting this code never earns `selected-check-off`. Queries, `fmt` and `init` stay silent. The catalog supplies `grund config explain closed-citation-grounding` in both versions ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), and migration compares its facts, never discards it ([§FS-config-migrate.5.2](FS-config-migrate.md#52-the-report-gate)).

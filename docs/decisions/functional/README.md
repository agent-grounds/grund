# Functional decisions

Why `grund` behaves the way it does. Each file here is one product-behavior decision: the context that forced it, the call, the alternatives that lost, and what the call costs. The H1 declares a `DF-<slug>` ID, and the spec point a decision settles cites it — so a rule in `docs/functional-spec/` is always one hop from its argument. Each record's first chapter is `## status:`, holding `Accepted` or `Superseded`; its `Date`, `Superseded by` and other prose lines stay above it, in the lead ([§DISC-core-concerns.10.2](../../discussions/proposals/2026-09-30-core-concerns.md#102-the-five-decisions-the-plan-makes-one-way)).

- [§DF-watch-terminal-loop](DF-watch-terminal-loop.md#df-watch-terminal-loop-a-terminal-watch-loop-preserves-ordinary-check-reports) — Preserve ordinary reports in the terminal watch loop, including recovery, interruption and screen ownership.

Read a decision when the spec tells you *what* and you need *why*. Do not read them for the current behavior: a decision records the state of the argument on its date, and a superseded one is kept for its reasoning, not its verdict.

## The citation form

How a citation is written, and what counts as one.

- [§DF-reference-marker](DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger) — Use § as the reference marker, with $$ as the typing trigger
- [§DF-word-character-citation-markers](DF-word-character-citation-markers.md#df-word-character-citation-markers-recognize-accepted-word-character-markers) — recognize accepted word-character markers
- [§DF-code-declarations-drop-hash](DF-code-declarations-drop-hash.md#df-code-declarations-drop-hash-code-resident-declarations-may-drop-the--prefix) — code-resident declarations may drop the `#` prefix
- [§DF-number-only-citation-shorthand](DF-number-only-citation-shorthand.md#df-number-only-citation-shorthand-the-number-only-shorthand-is-authoring-sugar-and-a-persisted-one-is-a-check-error) — the number-only shorthand is authoring sugar, and a persisted one is a check error
- [§DF-declaration-local-section-shorthand](DF-declaration-local-section-shorthand.md#df-declaration-local-section-shorthand-local-numeric-section-citations-are-recognized-but-never-canonical) — local numeric section citations are recognized but never canonical
- [§DF-shorthand-numeric-run](DF-shorthand-numeric-run.md#df-shorthand-numeric-run-a-marked-shorthand-glued-to-another-number-is-a-numeral-not-a-citation) — a marked shorthand glued to another number is a numeral, not a citation
- [§DF-off-grammar-declaration-compatibility](DF-off-grammar-declaration-compatibility.md#df-off-grammar-declaration-compatibility-persisted-declarations-remain-readable-without-relaxing-the-authoring-grammar) — persisted declarations remain readable without relaxing the authoring grammar
- [§DF-canonical-slug-declarations](DF-canonical-slug-declarations.md#df-canonical-slug-declarations-configured-slug-punctuation-retains-its-canonical-declaration-identity) — configured slug punctuation retains its canonical declaration identity
- [§DF-glob-citation](DF-glob-citation.md#df-glob-citation-a-glob-operator-ends-no-citation-and-the-prefix-it-left-behind-is-removed) — a glob operator ends no citation, and the prefix it left behind is removed
- [§DF-inline-note-layout](DF-inline-note-layout.md#df-inline-note-layout-inline-note-layout-is-a-configured-house-style-checked-per-line-and-never-normalized) — inline note layout is a configured house style, checked per line and never normalized
- [§DF-note-columns-are-characters](DF-note-columns-are-characters.md#df-note-columns-are-characters-a-note-column-is-one-character-not-one-byte-and-not-one-display-cell) — a note column is one character, not one byte and not one display cell
- [§DF-doc-comments-are-not-notes](DF-doc-comments-are-not-notes.md#df-doc-comments-are-not-notes-a-doc-comment-is-documentation-not-a-note-and-is-never-an-inline-citation-site) — a doc comment is documentation, not a note, and is never an inline citation site
- [§DF-declarations-in-doc-comments](DF-declarations-in-doc-comments.md#df-declarations-in-doc-comments-a-v2-source-declares-in-documentation-not-in-a-note) — a v2 source declares in documentation, not in a note
- [§DF-python-assigned-triple-quoted-data](DF-python-assigned-triple-quoted-data.md#df-python-assigned-triple-quoted-data-module-level-assigned-triple-quoted-strings-are-data-not-docstrings) — module-level assigned triple-quoted strings are data, not docstrings
- [§DF-multi-line-string-literals](DF-multi-line-string-literals.md#df-multi-line-string-literals-a-string-literal-that-spans-lines-is-one-literal-for-a-fixed-set-of-forms) — a string literal that spans lines is one literal, for a fixed set of forms
- [§DF-unmarked-markdown-headings](DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph) — in-body Markdown ATX headings participate in the knowledge graph
- [§DF-escape-position-is-not-a-citation](DF-escape-position-is-not-a-citation.md#df-escape-position-is-not-a-citation-an-escape-position-is-not-a-citation-in-either-strict-mode) — an escape position is not a citation, in either strict mode
- [§DF-inline-code-span-closes-on-its-own-run](DF-inline-code-span-closes-on-its-own-run.md#df-inline-code-span-closes-on-its-own-run-an-inline-code-span-closes-on-a-run-of-its-own-length) — an inline code span closes on a run of its own length

## Cross-reference links

The rendered view of a citation — `[§ID](path#anchor)` — and who owns it.

- [§DF-md-link-emission](DF-md-link-emission.md#df-md-link-emission-grund-fmt-may-emit-clickable-markdown-links-alongside--prefixed-citations) — grund fmt may emit clickable Markdown links alongside §-prefixed citations
- [§DF-md-link-default-on](DF-md-link-default-on.md#df-md-link-default-on-markdown-cross-reference-links-default-on-for-github-review-and-discovery) — Markdown cross-reference links default on for GitHub review and discovery
- [§DF-md-link-anchor-strategy](DF-md-link-anchor-strategy.md#df-md-link-anchor-strategy-heading-text-slugs-re-derived-on-every-fmt-pass) — heading-text slugs, re-derived on every fmt pass
- [§DF-github-anchor-fidelity](DF-github-anchor-fidelity.md#df-github-anchor-fidelity-the-github-anchor-profile-reproduces-github-slugger-exactly) — the github anchor profile reproduces github-slugger exactly
- [§DF-repeated-heading-anchor](DF-repeated-heading-anchor.md#df-repeated-heading-anchor-a-repeated-headings-anchor-is-numbered-as-the-renderer-numbers-it) — a repeated heading's anchor is numbered as the renderer numbers it
- [§DF-declaration-anchor](DF-declaration-anchor.md#df-declaration-anchor-a-bare-id-markdown-link-points-at-the-declarations-heading-anchor) — a bare-ID Markdown link points at the declaration's heading anchor
- [§DF-show-anchor-data](DF-show-anchor-data.md#df-show-anchor-data-show-json-carries-the-heading-anchor-grund-already-derives) — show JSON carries the heading anchor grund already derives
- [§DF-stub-heading-from-unscanned-target](DF-stub-heading-from-unscanned-target.md#df-stub-heading-from-unscanned-target-a-bare-id-link-through-a-stub-anchors-on-its-targets-heading-whether-or-not-the-scan-reaches-it) — a bare-ID link through a stub anchors on its target's heading whether or not the scan reaches it
- [§DF-show-cross-ref-flattening](DF-show-cross-ref-flattening.md#df-show-cross-ref-flattening-grund-show-flattens-cross-reference-link-wrappers) — grund show flattens cross-reference link wrappers
- [§DF-fmt-suppression](DF-fmt-suppression.md#df-fmt-suppression-fmt-suppression-is-per-file-and-per-region-and-the-index-carve-out-outranks-both) — fmt suppression is per file and per region, and the index carve-out outranks both
- [§DF-fmt-never-writes-into-a-link-destination](DF-fmt-never-writes-into-a-link-destination.md#df-fmt-never-writes-into-a-link-destination-fmt-never-writes-into-a-link-destination-it-does-not-maintain) — `fmt` never writes into a link destination it does not maintain
- [§DF-fmt-and-editor-answers-from-0-19](DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them) — `fmt`'s link wrapper and the editor's title span change in 0.19.0, after a release that announces them

## The index a folder kind keeps

The rules that make this file, and the ones beside it, checked rather than hoped for.

- [§DF-index-entry-form](DF-index-entry-form.md#df-index-entry-form-an-index-entry-is-one-full-link-per-id-and-nothing-else-about-the-page) — an index entry is one full link per ID, and nothing else about the page
- [§DF-index-compatibility-ramp](DF-index-compatibility-ramp.md#df-index-compatibility-ramp-a-findings-ramp-follows-its-fix-command-not-the-size-of-the-offence) — a finding's ramp follows its fix command, not the size of the offence
- [§DF-index-not-an-inbound-citation](DF-index-not-an-inbound-citation.md#df-index-not-an-inbound-citation-an-index-entry-is-navigation-not-use) — an index entry is navigation, not use
- [§DF-index-always-linkified](DF-index-always-linkified.md#df-index-always-linkified-the-cross-reference-pass-always-runs-on-a-kinds-index-file) — the cross-reference pass always runs on a kind's index file

## What `check` reports, and how loudly

- [§DF-check-full-scope](DF-check-full-scope.md#df-check-full-scope-check---full-walks-past-scan-include-and-reports-unresolved-references-plus-orphaned-section-headings-out-there) — `check --full` walks past `[scan] include` and reports unresolved references plus orphaned section headings out there
- [§DF-require-grounding](DF-require-grounding.md#df-require-grounding-an-opt-in-check-that-every-source-file-cites-a-spec) — an opt-in check that every source file cites a spec
- [§DF-path-scope-resolves-project-wide](DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path) — a path-scoped `check` resolves against the whole project and reports only the path
- [§DF-path-narrowing-warns](DF-path-narrowing-warns.md#df-path-narrowing-warns-a-path-that-narrows-what-show-list-and-id-resolve-or-what-check--resolves-an-alias-against-says-so-for-one-release) — a path that narrows what show, list and id resolve, or what check <dir> resolves an alias against, says so for one release
- [§DF-path-narrows-the-report-in-0-19](DF-path-narrows-the-report-in-0-19.md#df-path-narrows-the-report-in-0-19-in-0190-a-path-narrows-what-show-list-and-id-report-not-what-they-resolve-and-workspace-check--resolves-every-project-it-sees) — in 0.19.0 a path narrows what show, list and id report, not what they resolve, and workspace check <dir> resolves every project it sees
- [§DF-nothing-recognized](DF-nothing-recognized.md#df-nothing-recognized-a-run-that-recognized-nothing-says-so-and-says-it-as-a-warning) — a run that recognized nothing says so, and says it as a warning
- [§DF-duplicate-section-path](DF-duplicate-section-path.md#df-duplicate-section-path-a-section-coordinate-names-one-heading-or-the-run-says-so) — a section coordinate names one heading, or the run says so
- [§DF-relative-section-headings](DF-relative-section-headings.md#df-relative-section-headings-a-heading-may-write-only-the-component-it-adds) — a heading may write only the component it adds
- [§DF-doc-comment-relative-headings](DF-doc-comment-relative-headings.md#df-doc-comment-relative-headings-a-relative-heading-in-a-doc-comment-warns-in-0180-and-is-a-section-from-0190) — a relative heading in a doc-comment warns in 0.18.0 and is a section from 0.19.0
- [§DF-section-lock-relative-fingerprint](DF-section-lock-relative-fingerprint.md#df-section-lock-relative-fingerprint-a-locked-sections-nested-headings-are-fingerprinted-relative-to-it-so-a-renumber-is-still-caught) — a locked section's nested headings are fingerprinted relative to it, so a renumber is still caught
- [§DF-mixed-path-citation-errors](DF-mixed-path-citation-errors.md#df-mixed-path-citation-errors-mixed-citation-findings-use-ordinary-errors-in-0190) — mixed citation findings use ordinary errors in 0.19.0
- [§DF-number-name-sections](DF-number-name-sections.md#df-number-name-sections-a-name-may-follow-a-number-1goals-and-a-cited-address-with-one-is-locked) — a name may follow a number (`1.goals`), and a cited address with one is locked
- [§DF-stub-pairs-with-unscanned-target](DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it) — a stub pairs with its target whether or not the scan reaches it
- [§DF-stub-sections-from-unscanned-target](DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it) — a stub's sections are its target's whether or not the scan reaches it
- [§DF-stub-target-declared-twice](DF-stub-target-declared-twice.md#df-stub-target-declared-twice-a-stubs-target-that-declares-its-id-twice-is-two-homes-scanned-or-not) — a stub's target that declares its ID twice is two homes, scanned or not
- [§DF-stub-target-exclusions](DF-stub-target-exclusions.md#df-stub-target-exclusions-excluded-and-ignored-stub-targets-are-broken-from-grund-0190) — excluded and ignored stub targets are broken from 0.19.0
- [§DF-stub-target-section-declared-twice](DF-stub-target-section-declared-twice.md#df-stub-target-section-declared-twice-a-section-a-stubs-target-declares-twice-is-reported-where-it-is-written-scanned-or-not) — a section a stub's target declares twice is reported where it is written, scanned or not
- [§DF-stubs-by-home-and-at-the-target](DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190) — a rule declared through a stub applies now, and a stub is read by its home and shown at its target from 0.19.0
- [§DF-stub-target-citations-and-chapters](DF-stub-target-citations-and-chapters.md#df-stub-target-citations-and-chapters-a-stubs-target-lends-the-stubbed-declaration-its-citations-and-chapters-and-what-that-newly-raises-warns-until-0190) — a stub's target lends the stubbed declaration its citations and chapters, and what that newly raises warns until 0.19.0
- [§DF-a-buffer-is-the-file](DF-a-buffer-is-the-file.md#df-a-buffer-is-the-file-an-editor-buffer-is-the-file-it-names-under-every-spelling-and-with-no-file-saved-behind-it) — an editor buffer is the file it names, under every spelling and with no file saved behind it
- [§DF-four-editor-checks-read-saved-text](DF-four-editor-checks-read-saved-text.md#df-four-editor-checks-read-saved-text-four-editor-checks-read-the-saved-file-for-one-more-release) — four editor checks read the saved file for one more release
- [§DF-closing-hashes-are-not-the-heading](DF-closing-hashes-are-not-the-heading.md#df-closing-hashes-are-not-the-heading-a-headings-closing-hashes-are-not-part-of-its-anchor) — a heading's closing hashes are not part of its anchor
- [§DF-a-title-citation-is-a-citation](DF-a-title-citation-is-a-citation.md#df-a-title-citation-is-a-citation-a-citation-on-a-declarations-heading-is-a-citation-unless-it-cites-the-id-that-heading-declares) — a citation on a declaration's heading is a citation, unless it cites the ID that heading declares
- [§DF-bare-block-comment-line-declares](DF-bare-block-comment-line-declares.md#df-bare-block-comment-line-declares-a-line-inside-a-block-comment-declares-with-or-without-its-gutter-behind-any-of-its-openers) — a line inside a block comment declares with or without its gutter, behind any of its openers
- [§DF-line-prefix-forms-declare](DF-line-prefix-forms-declare.md#df-line-prefix-forms-declare-lua-haskell-and-lisp-doc-comment-forms-declare-and-a-lisp-doc-block-is-not-a-note) — Lua, Haskell and Lisp doc-comment forms declare, and a Lisp doc block is not a note
- [§DF-doc-comment-fenced-heading](DF-doc-comment-fenced-heading.md#df-doc-comment-fenced-heading-a-heading-inside-a-fence-in-a-comment-block-is-not-a-section) — a heading inside a fence in a comment block is not a section
- [§DF-one-parse-answers-from-0-19](DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves) — one parse's other answers ship in 0.19.0, after a release that announces them and warns where a verdict moves
- [§DF-one-parse-lands-in-0-19](DF-one-parse-lands-in-0-19.md#df-one-parse-lands-in-0-19-every-command-answers-from-one-parse-in-0190-and-the-managed-block-teaches-doc-comment-sections) — every command answers from one parse in 0.19.0, and the managed block teaches doc-comment sections
- [§DF-citation-directions](DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels) — encode citation directions as checked config with RFC-2119 levels
- [§DF-chapter-rules](DF-chapter-rules.md#df-chapter-rules-chapter-rules-are-grounded-controlled-english-declarations-over-producer-neutral-facts) — chapter rules are grounded controlled-English declarations over producer-neutral facts
- [§DF-unverifiable-rule-scope](DF-unverifiable-rule-scope.md#df-unverifiable-rule-scope-a-rule-the-scope-cannot-judge-is-reported-rendered-and-written) — a rule the scope cannot judge is reported, rendered, and written
- [§DF-rule-authority-is-a-field](DF-rule-authority-is-a-field.md#df-rule-authority-is-a-field-a-findings-rule-authority-is-a-record-field-and-the-trial-sentence-selector-is-a-query-over-it) — a finding's rule authority is a record field, and the trial-sentence selector is a query over it
- [§DF-narrowed-check-keeps-invalid-rule](DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run) — a check narrowed to a rule's code keeps the error that says the rule could not run
- [§DF-direction-absorbs-findings-not-rules](DF-direction-absorbs-findings-not-rules.md#df-direction-absorbs-findings-not-rules-a-citations-direction-absorbs-a-duplicate-rules-finding-for-a-unit-it-reports-not-the-rule) — a `[citations]` direction absorbs a duplicate rule's finding for a unit it reports, not the rule
- [§DF-section-citation-counts-in-rules](DF-section-citation-counts-in-rules.md#df-section-citation-counts-in-rules-a-resolved-citation-to-a-numbered-section-counts-in-chapter-rules) — a resolved citation to a numbered section counts in chapter rules
- [§DF-nested-alias-rule-targets](DF-nested-alias-rule-targets.md#df-nested-alias-rule-targets-a-rule-target-pinned-at-its-own-nested-alias-matches-like-the-bare-kind) — a rule target pinned at its own nested alias matches like the bare kind
- [§DF-selector-refusal-rewrites](DF-selector-refusal-rewrites.md#df-selector-refusal-rewrites-a-refused-selector-is-answered-with-a-selector-and-its-old-lines-are-replaced-not-appended-to) — a refused selector is answered with a selector, and its old lines are replaced, not appended to
- [§DF-rule-after-enabling-rewrites](DF-rule-after-enabling-rewrites.md#df-rule-after-enabling-rewrites-a-rule-subject-that-needs-named-sections-is-answered-with-one-they-make-valid-in-place) — a rule subject that needs named sections is answered with one they make valid, in place
- [§DF-local-section-absent-target-escape](DF-local-section-absent-target-escape.md#df-local-section-absent-target-escape-an-owned-local-section-citation-whose-section-is-absent-is-answered-with-the-escape-in-place) — an owned local section citation whose section is absent is answered with the escape, in place
- [§DF-rule-refusal-reasons](DF-rule-refusal-reasons.md#df-rule-refusal-reasons-a-chapter-path-is-refused-for-the-component-that-failed-corrected-in-place) — a chapter path is refused for the component that failed, corrected in place
- [§DF-rule-refusal-rewrites](DF-rule-refusal-rewrites.md#df-rule-refusal-rewrites-a-refused-rules-accepted-form-is-the-typed-sentence-with-the-failed-part-replaced-in-place) — a refused rule's accepted form is the typed sentence with the failed part replaced, in place
- [§DF-stub-target-fenced-heading](DF-stub-target-fenced-heading.md#df-stub-target-fenced-heading-a-heading-inside-a-fence-of-a-stubs-target-does-not-declare-its-id) — a heading inside a fence of a stub's target does not declare its ID

## Config, discovery, and workspaces

- [§DF-config-file-location](DF-config-file-location.md#df-config-file-location-grundtoml-is-discovered-at-two-names-per-directory-and-init-writes-the-bare-one) — grund.toml is discovered at two names per directory, and init writes the bare one
- [§DF-config-scope-override](DF-config-scope-override.md#df-config-scope-override-the-committed-scopes-are-one-relation-stated-once) — the committed scopes are one relation, stated once
- [§DF-config-concerns](DF-config-concerns.md#df-config-concerns-a-keys-concern-is-derived-from-what-a-finding-from-it-can-be-about) — a key's concern is derived from what a finding from it can be about
- [§DF-verdict-vocabulary-freeze](DF-verdict-vocabulary-freeze.md#df-verdict-vocabulary-freeze-the-freeze-is-on-the-verdict-vocabulary-not-on-which-rules-a-project-holds-in-force) — the freeze is on the verdict vocabulary, not on which rules a project holds in force
- [§DF-non-citable-kinds](DF-non-citable-kinds.md#df-non-citable-kinds-a-kind-may-declare-no-ids-and-stays-one-kinds-table-when-it-does) — a kind may declare no IDs, and stays one `[[kinds]]` table when it does
- [§DF-unwalked-kind-home](DF-unwalked-kind-home.md#df-unwalked-kind-home-a-kind-may-be-a-place-that-is-listed-but-not-walked) — a kind may be a place that is listed but not walked
- [§DF-v2-place-ownership](DF-v2-place-ownership.md#df-v2-place-ownership-in-v2-the-row-named-code-is-the-complement-and-a-file-belongs-to-its-listed-row-or-else-its-deepest-folder) — in v2 the row named `code` is the complement, and a file belongs to its listed row or else its deepest folder
- [§DF-config-migrate](DF-config-migrate.md#df-config-migrate-migration-refuses-what-version-2-cannot-spell-names-a-version-1-repair-and-writes-only-past-two-gates) — migration refuses what version 2 cannot spell, names a version-1 repair, and writes only past two gates
- [§DF-config-version-1-caution](DF-config-version-1-caution.md#df-config-version-1-caution-a-config-file-on-version-1-is-cautioned-once-in-place-of-success-with-no-deadline) — a config file on version 1 is cautioned, once, in place of `success`, with no deadline
- [§DF-symlink-scan](DF-symlink-scan.md#df-symlink-scan-a-symlink-in-the-scanned-tree-is-followed-and-the-report-names-the-link) — a symlink in the scanned tree is followed, and the report names the link
- [§DF-subproject-namespaces](DF-subproject-namespaces.md#df-subproject-namespaces-alias-namespace-model-for-sub-projects-and-external-repos) — alias-namespace model for sub-projects and external repos
- [§DF-nested-workspaces](DF-nested-workspaces.md#df-nested-workspaces-a-nested-project-is-named-by-its-whole-alias-path) — a nested project is named by its whole alias path
- [§DF-unlisted-workspace-block](DF-unlisted-workspace-block.md#df-unlisted-workspace-block-an-unlisted-workspace-block-is-reported-by-the-walk-that-meets-it) — an unlisted workspace block is reported by the walk that meets it
- [§DF-unlisted-workspace-error-shape](DF-unlisted-workspace-error-shape.md#df-unlisted-workspace-error-shape-check-and-the-editor-take-the-located-shape-the-five-walking-surfaces-keep-the-cli-level-line) — `check` and the editor take the located shape, the five walking surfaces keep the CLI-level line
- [§DF-workspace-member-descriptions](DF-workspace-member-descriptions.md#df-workspace-member-descriptions-member-side-project_description-for-workspace-member-lists) — member-side `project_description` for workspace member lists
- [§DF-cover-workspace-scope](DF-cover-workspace-scope.md#df-cover-workspace-scope-cover-indexes-the-whole-run-and-counts-cross-project-citations) — cover indexes the whole run and counts cross-project citations
- [§DF-cli-base-parent-paths](DF-cli-base-parent-paths.md#df-cli-base-parent-paths-relative_paths--false-keeps-one-cli-base-and-may-climb-within-the-loaded-root) — `relative_paths = false` keeps one CLI base and may climb within the loaded root
- [§DF-absorbed-scan-warning](DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error) — a scan its own members swallowed is a warning with a named release, not an error
- [§DF-optional-workspace-members](DF-optional-workspace-members.md#df-optional-workspace-members-an-absent-member-is-declared-in-a-sibling-list-and-the-run-announces-the-namespace-it-did-not-check) — an absent member is declared in a sibling list, and the run announces the namespace it did not check
- [§DF-unread-opted-out-block](DF-unread-opted-out-block.md#df-unread-opted-out-block-the-unread-files-of-an-opted-out-block-are-a-conditional-warning-that-never-ramps) — the unread files of an opted-out block are a conditional warning that never ramps
- [§DF-remote-projects](DF-remote-projects.md#df-remote-projects-a-remote-project-is-a-workspace-member-whose-bytes-were-fetched-mounted-as-its-own-root) — a remote project is a workspace member whose bytes were fetched, mounted as its own root
- [§DF-undeclared-blind-spots](DF-undeclared-blind-spots.md#df-undeclared-blind-spots-two-skips-no-section-named-become-located-findings) — two skips no section named become located findings
- [§DF-default-extensions-follow-language-table](DF-default-extensions-follow-language-table.md#df-default-extensions-follow-language-table-the-scan-extensions-default-reads-every-language-grund-classifies) — the `[scan] extensions` default reads every language grund classifies
- [§DF-fs-home-moves-from-0-19](DF-fs-home-moves-from-0-19.md#df-fs-home-moves-from-0-19-the-built-in-fs-home-moves-back-to-docsfunctional-spec-in-0190-and-the-template-declares-req-now) — the built-in FS home moves back to docs/functional-spec/ in 0.19.0, and the template declares REQ now
- [§DF-fs-home-lands-in-0-19](DF-fs-home-lands-in-0-19.md#df-fs-home-lands-in-0-19-the-built-in-fs-home-is-docsfunctional-spec-from-0190) — the built-in FS home is docs/functional-spec/ from 0.19.0
- [§DF-global-excludes-anchor-at-the-repository-top](DF-global-excludes-anchor-at-the-repository-top.md#df-global-excludes-anchor-at-the-repository-top-a-global-excludes-pattern-anchors-at-the-repository-top-wherever-grund-runs-from) — a global excludes pattern anchors at the repository top, wherever grund runs from

## The command surface

- [§DF-configured-title-metadata](DF-configured-title-metadata.md#df-configured-title-metadata-kind-titles-are-separate-target-metadata) — kind titles are separate target metadata

- [§DF-show-default-token-cheap](DF-show-default-token-cheap.md#df-show-default-token-cheap-grund-show-defaults-to-the-cheap-read-the-full-body-is-opt-in) — grund show defaults to the cheap read; the full body is opt-in
- [§DF-show-token-cheap-reads](DF-show-token-cheap-reads.md#df-show-token-cheap-reads-grund-show-keeps-the-full-body-default-token-cheap-slices-are-opt-in) — *(superseded)* grund show keeps the full-body default; token-cheap slices are opt-in
- [§DF-show-keep-explicit-form](DF-show-keep-explicit-form.md#df-show-keep-explicit-form-grund-keeps-show-as-a-subcommand-alongside-the-bare-id-default) — grund keeps `show` as a subcommand alongside the bare-ID default
- [§DF-keep-id-for-pure-id-allocation-and-reserve-new-for-stub](DF-keep-id-for-pure-id-allocation-and-reserve-new-for-stub.md#df-keep-id-for-pure-id-allocation-and-reserve-new-for-stub-keep-id-for-pure-id-allocation-and-reserve-new-for-stub-creation) — Keep `id` for pure ID allocation and reserve `new` for stub creation
- [§DF-id-number-width](DF-id-number-width.md#df-id-number-width-grund-id-zero-pads-minted-numbers-to-a-default-width-of-3) — grund id zero-pads minted numbers to a default width of 3
- [§DF-integrations-command](DF-integrations-command.md#df-integrations-command-integrations-earns-a-cli-slot-as-one-time-setup-where-a-per-citation-link-command-did-not) — integrations earns a CLI slot as one-time setup, where a per-citation `link` command did not
- [§DF-neural-link-generation](DF-neural-link-generation.md#df-neural-link-generation-agents-compose-clickable-citation-links-themselves-grund-does-not-grow-a-link-command) — agents compose clickable citation links themselves; grund does not grow a `link` command
- [§DF-fmt-one-model](DF-fmt-one-model.md#df-fmt-one-model-fmt-is-the-shared-verified-model-plus-a-write-step-and-completeness-is-a-precondition-rather-than-a-convention) — `fmt` is the shared verified model plus a write step, and completeness is a precondition rather than a convention
- [§DF-refs-resolver-rejection](DF-refs-resolver-rejection.md#df-refs-resolver-rejection-an-id-rejected-by-a-selected-grammar-is-a-failed-query) — an ID rejected by a selected grammar is a failed query
- [§DF-bare-grund-lands-on-an-error](DF-bare-grund-lands-on-an-error.md#df-bare-grund-lands-on-an-error-bare-grund-lands-on-a-cli-level-error-not-the-top-level-help-page) — bare `grund` lands on a CLI-level error, not the top-level help page
- [§DF-schema-view](DF-schema-view.md#df-schema-view-the-schema-view-is-its-own-command-prints-no-json-yet-shows-a-workspaces-root-and-draws-an-any-of-entry-through-one-junction) — the schema view is its own command, prints no JSON yet, shows a workspace's root, and draws an any-of entry through one junction

## Agent-facing surfaces

- [§DF-project-namespaces-in-workspaces](DF-project-namespaces-in-workspaces.md#df-project-namespaces-in-workspaces-teach-project-boundaries-in-workspace-blocks-and-member-splitting-in-setup-instructions) — teach project boundaries in workspace blocks and member splitting in setup instructions
- [§DF-scoped-guidance](DF-scoped-guidance.md#df-scoped-guidance-a-version-2-rows-guidance-is-written-above-its-places-with-a-root-pointer-for-agents-that-do-not-load-nested-files) — a version-2 row's guidance is written above its places, with a root pointer for agents that do not load nested files
- [§DF-managed-block-delimiters](DF-managed-block-delimiters.md#df-managed-block-delimiters-standard-beginend-delimiters-for-the-managed-agent-instructions-block) — standard BEGIN/END delimiters for the managed agent-instructions block
- [§DF-skill-init-existing-specs](DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding) — `grund-init` adopts existing specs before scaffolding
- [§DF-init-templates](DF-init-templates.md#df-init-templates-grund-init-lays-a-named-template-and---scaffold-replaces---docs) — `grund init` lays a named template, and `--scaffold` replaces `--docs`
- [§DF-v2-onboarding](DF-v2-onboarding.md#df-v2-onboarding-grund-init-writes-a-minimal-version-2-file-and-grund-config-show-does-the-teaching) — `grund init` writes a minimal version-2 file, and `grund config show` does the teaching
- [§DF-repo-conversation-opinion](DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion) — repositories may commit a link-only conversation-rendering opinion
- [§DF-conversation-link-target](DF-conversation-link-target.md#df-conversation-link-target-the-conversation-link-form-is-a-markdown-link-over-an-absolute-uri-addressed-per-machine) — the conversation link form is a Markdown link over an absolute URI, addressed per machine
- [§DF-directions-render](DF-directions-render.md#df-directions-render-the-citation-directions-wording-is-chosen-once-against-a-canonical-config) — the citation-directions wording is chosen once, against a canonical config
- [§DF-chapter-rule-reaches-every-declaration](DF-chapter-rule-reaches-every-declaration.md#df-chapter-rule-reaches-every-declaration-a-chapter-scoped-citation-rule-reports-the-declaration-that-has-no-such-chapter) — a chapter-scoped citation rule reports the declaration that has no such chapter
- [§DF-scan-exclude-component-names](DF-scan-exclude-component-names.md#df-scan-exclude-component-names-a-scan-exclude-entry-containing--is-a-config-error) — a `[scan] exclude` entry containing `/` is a config error
- [§DF-block-teaches-structural-queries](DF-block-teaches-structural-queries.md#df-block-teaches-structural-queries-the-managed-block-teaches-the-structural-query-inline-and-links-the-query-guide) — the managed block teaches the structural query inline and links the query guide
- [§DF-kind-fields](DF-kind-fields.md#df-kind-fields-a-kind-states-its-fields-by-section-name-and-a-present-optional-field-is-still-checked) — a kind states its fields by section name, and a present optional field is still checked

## The one-answer rulings

The rulings the one-answer work is built from, one record per step family, each naming the compatibility route its rulings ship by.

- [§DF-one-answer-rulings-run](DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held) — a path narrows the report, a stub reads what the walk reads, and a file is one file however it is spelled or held
- [§DF-one-answer-rulings-parse](DF-one-answer-rulings-parse.md#df-one-answer-rulings-parse-one-parse-gives-every-command-the-same-extents-comment-blocks-anchors-and-title-citations) — one parse gives every command the same extents, comment blocks, anchors and title citations
- [§DF-one-answer-rulings-stubs](DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is) — a stub is found by its home and shape, read as the scan reads, and shown where its target is
- [§DF-one-answer-rulings-fmt-editor](DF-one-answer-rulings-fmt-editor.md#df-one-answer-rulings-fmt-editor-fmt-writes-only-into-a-complete-link-wrapper-never-into-a-link-destination) — `fmt` writes only into a complete link wrapper, never into a link destination
- [§DF-one-answer-rulings-grammar](DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary) — one grammar keeps the documented refusal bytes and order, and one kind universe keeps each command's vocabulary
- [§DF-one-kind-verdict](DF-one-kind-verdict.md#df-one-kind-verdict-every-command-reads-one-kind-verdict-in-its-own-scope-and-refuses-a-rule-sentence-for-the-first-stage-it-fails) — every command reads one kind verdict in its own scope, and refuses a rule sentence for the first stage it fails
- [§DF-homeless-code-declares-no-ids](DF-homeless-code-declares-no-ids.md#df-homeless-code-declares-no-ids-code-with-no-row-is-refused-as-declaring-no-ids-alone-from-0190) — `code` with no row is refused as declaring no IDs alone from 0.19.0
- [§DF-unread-scan-root](DF-unread-scan-root.md#df-unread-scan-root-a-configured-file-root-whose-extension-is-unlisted-is-named-not-read) — a configured file root whose extension is unlisted is named, not read
- [§DF-unread-hidden-root](DF-unread-hidden-root.md#df-unread-hidden-root-a-configured-file-root-with-a-hidden-name-is-named-not-read) — a configured file root with a hidden name is named, not read

- [§DF-global-excludes-literal-tilde](DF-global-excludes-literal-tilde.md#df-global-excludes-literal-tilde-preserve-literal-tildes-in-the-global-excludes-pathname) — preserve literal tildes in the global excludes pathname

This index is navigational — citations should target the decision ID directly, never this file.

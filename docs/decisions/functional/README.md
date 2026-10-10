# Functional decisions

Why `grund` behaves the way it does. Each file here is one product-behavior decision: the context that forced it, the call, the alternatives that lost, and what the call costs. The H1 declares a `DF-<slug>` ID, and the spec point a decision settles cites it — so a rule in `docs/functional-spec/` is always one hop from its argument.

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
- [§DF-inline-note-layout](DF-inline-note-layout.md#df-inline-note-layout-inline-note-layout-is-a-configured-house-style-checked-per-line-and-never-normalized) — inline note layout is a configured house style, checked per line and never normalized
- [§DF-note-columns-are-characters](DF-note-columns-are-characters.md#df-note-columns-are-characters-a-note-column-is-one-character-not-one-byte-and-not-one-display-cell) — a note column is one character, not one byte and not one display cell
- [§DF-doc-comments-are-not-notes](DF-doc-comments-are-not-notes.md#df-doc-comments-are-not-notes-a-doc-comment-is-documentation-not-a-note-and-is-never-an-inline-citation-site) — a doc comment is documentation, not a note, and is never an inline citation site
- [§DF-python-assigned-triple-quoted-data](DF-python-assigned-triple-quoted-data.md#df-python-assigned-triple-quoted-data-module-level-assigned-triple-quoted-strings-are-data-not-docstrings) — module-level assigned triple-quoted strings are data, not docstrings
- [§DF-unmarked-markdown-headings](DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph) — in-body Markdown ATX headings participate in the knowledge graph
- [§DF-escape-position-is-not-a-citation](DF-escape-position-is-not-a-citation.md#df-escape-position-is-not-a-citation-an-escape-position-is-not-a-citation-in-either-strict-mode) — an escape position is not a citation, in either strict mode
- [§DF-inline-code-span-closes-on-its-own-run](DF-inline-code-span-closes-on-its-own-run.md#df-inline-code-span-closes-on-its-own-run-an-inline-code-span-closes-on-a-run-of-its-own-length) — an inline code span closes on a run of its own length

## Cross-reference links

The rendered view of a citation — `[§ID](path#anchor)` — and who owns it.

- [§DF-md-link-emission](DF-md-link-emission.md#df-md-link-emission-grund-fmt-may-emit-clickable-markdown-links-alongside--prefixed-citations) — grund fmt may emit clickable Markdown links alongside §-prefixed citations
- [§DF-md-link-default-on](DF-md-link-default-on.md#df-md-link-default-on-markdown-cross-reference-links-default-on-for-github-review-and-discovery) — Markdown cross-reference links default on for GitHub review and discovery
- [§DF-md-link-anchor-strategy](DF-md-link-anchor-strategy.md#df-md-link-anchor-strategy-heading-text-slugs-re-derived-on-every-fmt-pass) — heading-text slugs, re-derived on every fmt pass
- [§DF-github-anchor-fidelity](DF-github-anchor-fidelity.md#df-github-anchor-fidelity-the-github-anchor-profile-reproduces-github-slugger-exactly) — the github anchor profile reproduces github-slugger exactly
- [§DF-declaration-anchor](DF-declaration-anchor.md#df-declaration-anchor-a-bare-id-markdown-link-points-at-the-declarations-heading-anchor) — a bare-ID Markdown link points at the declaration's heading anchor
- [§DF-show-anchor-data](DF-show-anchor-data.md#df-show-anchor-data-show-json-carries-the-heading-anchor-grund-already-derives) — show JSON carries the heading anchor grund already derives
- [§DF-stub-heading-from-unscanned-target](DF-stub-heading-from-unscanned-target.md#df-stub-heading-from-unscanned-target-a-bare-id-link-through-a-stub-anchors-on-its-targets-heading-whether-or-not-the-scan-reaches-it) — a bare-ID link through a stub anchors on its target's heading whether or not the scan reaches it
- [§DF-show-cross-ref-flattening](DF-show-cross-ref-flattening.md#df-show-cross-ref-flattening-grund-show-flattens-cross-reference-link-wrappers) — grund show flattens cross-reference link wrappers
- [§DF-fmt-suppression](DF-fmt-suppression.md#df-fmt-suppression-fmt-suppression-is-per-file-and-per-region-and-the-index-carve-out-outranks-both) — fmt suppression is per file and per region, and the index carve-out outranks both

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
- [§DF-nothing-recognized](DF-nothing-recognized.md#df-nothing-recognized-a-run-that-recognized-nothing-says-so-and-says-it-as-a-warning) — a run that recognized nothing says so, and says it as a warning
- [§DF-duplicate-section-path](DF-duplicate-section-path.md#df-duplicate-section-path-a-section-coordinate-names-one-heading-or-the-run-says-so) — a section coordinate names one heading, or the run says so
- [§DF-stub-pairs-with-unscanned-target](DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it) — a stub pairs with its target whether or not the scan reaches it
- [§DF-stub-sections-from-unscanned-target](DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it) — a stub's sections are its target's whether or not the scan reaches it
- [§DF-stub-target-declared-twice](DF-stub-target-declared-twice.md#df-stub-target-declared-twice-a-stubs-target-that-declares-its-id-twice-is-two-homes-scanned-or-not) — a stub's target that declares its ID twice is two homes, scanned or not
- [§DF-citation-directions](DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels) — encode citation directions as checked config with RFC-2119 levels
- [§DF-chapter-rules](DF-chapter-rules.md#df-chapter-rules-chapter-rules-are-grounded-controlled-english-declarations-over-producer-neutral-facts) — chapter rules are grounded controlled-English declarations over producer-neutral facts
- [§DF-unverifiable-rule-scope](DF-unverifiable-rule-scope.md#df-unverifiable-rule-scope-a-rule-the-scope-cannot-judge-is-reported-rendered-and-written) — a rule the scope cannot judge is reported, rendered, and written
- [§DF-rule-authority-is-a-field](DF-rule-authority-is-a-field.md#df-rule-authority-is-a-field-a-findings-rule-authority-is-a-record-field-and-the-trial-sentence-selector-is-a-query-over-it) — a finding's rule authority is a record field, and the trial-sentence selector is a query over it
- [§DF-narrowed-check-keeps-invalid-rule](DF-narrowed-check-keeps-invalid-rule.md#df-narrowed-check-keeps-invalid-rule-a-check-narrowed-to-a-rules-code-keeps-the-error-that-says-the-rule-could-not-run) — a check narrowed to a rule's code keeps the error that says the rule could not run
- [§DF-section-citation-counts-in-rules](DF-section-citation-counts-in-rules.md#df-section-citation-counts-in-rules-a-resolved-citation-to-a-numbered-section-counts-in-chapter-rules) — a resolved citation to a numbered section counts in chapter rules
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

## Agent-facing surfaces

- [§DF-managed-block-delimiters](DF-managed-block-delimiters.md#df-managed-block-delimiters-standard-beginend-delimiters-for-the-managed-agent-instructions-block) — standard BEGIN/END delimiters for the managed agent-instructions block
- [§DF-skill-init-existing-specs](DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding) — `grund-init` adopts existing specs before scaffolding
- [§DF-repo-conversation-opinion](DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion) — repositories may commit a link-only conversation-rendering opinion
- [§DF-conversation-link-target](DF-conversation-link-target.md#df-conversation-link-target-the-conversation-link-form-is-a-markdown-link-over-an-absolute-uri-addressed-per-machine) — the conversation link form is a Markdown link over an absolute URI, addressed per machine
- [§DF-directions-render](DF-directions-render.md#df-directions-render-the-citation-directions-wording-is-chosen-once-against-a-canonical-config) — the citation-directions wording is chosen once, against a canonical config
- [§DF-chapter-rule-reaches-every-declaration](DF-chapter-rule-reaches-every-declaration.md#df-chapter-rule-reaches-every-declaration-a-chapter-scoped-citation-rule-reports-the-declaration-that-has-no-such-chapter) — a chapter-scoped citation rule reports the declaration that has no such chapter
- [§DF-scan-exclude-component-names](DF-scan-exclude-component-names.md#df-scan-exclude-component-names-a-scan-exclude-entry-containing--is-a-config-error) — a `[scan] exclude` entry containing `/` is a config error
- [§DF-block-teaches-structural-queries](DF-block-teaches-structural-queries.md#df-block-teaches-structural-queries-the-managed-block-teaches-the-structural-query-inline-and-links-the-query-guide) — the managed block teaches the structural query inline and links the query guide

This index is navigational — citations should target the decision ID directly, never this file.

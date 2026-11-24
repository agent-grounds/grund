# FS-lsp: grund ships an optional LSP server

`grund-lsp` is a Language Server Protocol server, a binary separate from `grund`, that gives any LSP-aware editor the answers `grund` gives on the command line: diagnostics, hover, navigation, completion and the live trigger transform. It is optional: a user who wants editor integration installs it and configures the editor once, and CI pipelines, pre-commit hooks and contributors who only run `grund check` install nothing extra and pay no dependency cost ([§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary)).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, qualified citation, shorthand, citation site),
[§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope,
workspace, member, alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion),
[§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction, rule), and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, component, binding,
fetcher, snapshot).

- **diagnostic** — The LSP protocol object a finding is published as. The word names the
  protocol object here and nothing else.
- **workspace folder** — A root the editor supplies in its `initialize` request, from which
  config discovery walks up.
- **snapshot (narrowed)** — [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations). In this spec, within the server's own state, it
  also names the immutable scan result one request is answered from; elsewhere the shared
  definition applies.
- **typing request** — A `textDocument/completion` or `textDocument/onTypeFormatting`
  request: one that reads only the document's live text and its project's declarations, so
  it can be answered before the snapshot is rebuilt ([§FS-lsp.responsiveness.4](FS-lsp.md#responsiveness4-a-typing-request-waits-for-no-rebuild-while-the-declarations-stand)).
- **snapshot request** — A hover, definition, references, occurrence-highlight or
  document-link request: one that reads positions and bodies a rebuild recorded, so it is
  answered from a snapshot built from exactly the text it was sent against
  ([§FS-lsp.responsiveness.3](FS-lsp.md#responsiveness3-a-citation-hover-reads-nothing-the-snapshot-did-not)).

## requirements: What the editor surface holds to

Five promises bind the server as a whole, and what an answer may cost is [§FS-lsp.responsiveness](FS-lsp.md#responsiveness-what-an-answer-may-cost). Every surface also holds to three shared ones: the server writes no file and reaches no network on any request ([§REQ-no-data-loss.1](../requirements/REQ-no-data-loss.md#1-the-query-surface-never-writes), [§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)); an edit it returns touches only what it owns in the user's buffer, the live transform's by [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) and a completion's by [§FS-lsp.1.6](FS-lsp.md#16-declared-id-completion); and it ships no per-editor plugin ([§FS-non-goals.12.2](FS-non-goals.md#122-first-party-per-editor-plugins)).

### requirements.inputs: An answer is a function of three named inputs

The server answers from three inputs: the scanned tree, which takes each open document's notified text, the text a save would write, in place of its file ([§FS-scan.text.buffer](FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write)) and includes every tree an import names whether or not a folder holds it ([§FS-imports.editor](FS-imports.md#editor-the-editor-reads-the-import-graph-the-command-line-reads)); the effective config of the project that owns the document ([§FS-lsp.3](FS-lsp.md#3-configuration)); and the workspace folders the session holds, which decide that owner as the invocation decides scope on the command line ([§FS-lsp.2.2.1](FS-lsp.md#221-workspace-folders-anchor-discovery), [§FS-lsp.2.2.2](FS-lsp.md#222-one-project-answers-each-document)). These are the tree, config and invocation of [§REQ-deterministic-output.2](../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation), the buffer read as [§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer) reads it. A client capability changes an answer's shape, never its content ([§FS-lsp.1.3.4](FS-lsp.md#134-origin-span-and-result-shape)), and what initialization advertises decides when a client asks, never what it is told ([§FS-lsp.1.6](FS-lsp.md#16-declared-id-completion)). The point is directional: a dependence on further state is named here by the change that introduces it, or it is a defect. The one named is the last good scan a config that will not load leaves in place ([§FS-lsp.requirements.config-errors](FS-lsp.md#requirementsconfig-errors-a-config-mistake-reaches-the-editor-as-loudly-as-the-command-line)).

### requirements.parity: Every answer is one the command line gives

A diagnostic is the located `grund check` finding ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)); a citation hover is the bytes of `grund <ID> --toc` ([§FS-lsp.1.2.1](FS-lsp.md#121-citation-preview)); a title's count and its citation list are the set `grund refs` lists, with the clause `--total` prints ([§FS-lsp.1.2.3](FS-lsp.md#123-title-hover), [§FS-lsp.1.3.1](FS-lsp.md#131-citations-from-declarations)); the number-only shorthand's expansion previews `grund fmt --write` ([§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform)); and the server recognizes nothing the command line does not ([§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli)). This carries [§REQ-one-answer.text.parity](../requirements/REQ-one-answer.md#textparity-the-editor-answers-as-the-command-line-does), which names diagnostics, hover and definition, to every answer. A project whose config will not load answers with the command line's error ([§FS-lsp.requirements.config-errors](FS-lsp.md#requirementsconfig-errors-a-config-mistake-reaches-the-editor-as-loudly-as-the-command-line)). Each editor row of [§REQ-one-answer.differences](../requirements/REQ-one-answer.md#differences-the-differences-known-when-this-requirement-landed) is deferred, owed to that ledger. Serves [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree).

### requirements.no-guess: No answer is a guess

Where an answer is not unique the server gives none. A document is answered by at most one project, independent projects are never merged, and a document two projects reach only through their scans is nobody's ([§FS-lsp.2.2.1](FS-lsp.md#221-workspace-folders-anchor-discovery), [§FS-lsp.2.2.2](FS-lsp.md#222-one-project-answers-each-document)); a missing, ownerless or ambiguous token has no definition, link, hover or highlight ([§FS-lsp.1.3](FS-lsp.md#13-go-to-definition)); a shorthand expands only when it names exactly one declaration ([§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform)); and completion offers only uniquely resolving spellings, withholding edits while discovery is incomplete ([§FS-lsp.1.6](FS-lsp.md#16-declared-id-completion)). Serves [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), against the one source of ambiguity no command-line run has: several projects in one session.

### requirements.degrades: A session degrades and never ends

Nothing the editor sends ends the session. A failing request returns an LSP error and a notification that does not parse is skipped ([§FS-lsp.1](FS-lsp.md#1-capabilities)); a folder with a non-`file:` URI or a config that will not load is passed over while every other folder stays current, a folder change applies whatever of it is usable, and a session with no usable folder starts and answers nothing ([§FS-lsp.2.2.1](FS-lsp.md#221-workspace-folders-anchor-discovery), [§FS-lsp.2.2.4](FS-lsp.md#224-an-unusable-folder-is-skipped)); and an argument never enters the protocol loop, so a batch mistake cannot reach a session's stdout ([§FS-lsp.2.4](FS-lsp.md#24-installed-editor-integrations)). Serves [§REQ-never-crashes](../requirements/REQ-never-crashes.md#req-never-crashes-garbage-in-diagnostic-out), in the form a process with no exit code to report can take.

### requirements.config-errors: A config mistake reaches the editor as loudly as the command line

The command line refuses a config that will not load, naming its line, and exits `2` ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)), as [§FS-config.requirements.5](FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly) requires. The server keeps that project on its last good scan and publishes the same error as a diagnostic on the config file at the line the command line names, saying that the other diagnostics are from that last good scan, and clears it once the config loads again ([§FS-lsp.2.2.4](FS-lsp.md#224-an-unusable-folder-is-skipped)); so the error never rests only on the server's stderr, which most clients hide in a log pane.

## 1. Capabilities

A single malformed or failing message is not fatal: a request that fails returns an LSP error response, a notification whose params do not parse is logged and skipped, and the server goes on serving the messages after it.

### 1.1 Diagnostics

`textDocument/publishDiagnostics` pushes `grund check`'s located findings as the user edits: each is one diagnostic with the path, line, code, severity and message of the shared core report ([§FS-rules.9](FS-rules.md#9-managed-guidance-and-editor-parity)), a scan failure as well as a content finding. The server derives no finding of its own, and publishing never runs a fetcher: a missing snapshot is the report's `dangling` error or `missing-snapshot` warning ([§FS-check.4.12](FS-check.md#412-missing-snapshot)). The advisory suggestions channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)) is not published, and severity is the engine's, not configurable ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization)).

A diagnostic spans the whole written token of the citation its finding concerns, never merely the first citation on its line, so each of several citations in one comment carries its own and a resolving one beside them stays unmarked. A finding about a declaration, section or stub spans its title ([§FS-lsp.1.3.4](FS-lsp.md#134-origin-span-and-result-shape)), a named heading's whole title included. A line-anchored finding, such as the ungrounded-file check of [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in), borrows no citation's range from its line, so an editor that shows diagnostics in a hover never stacks a line's message onto a citation's own error.

A diagnostic names its file as the shared report does: by its in-tree link name, relative to the root of the project or workspace member that owns it, with every site the report gives a file of several names ([§FS-scan.identity.link-path](FS-scan.md#identitylink-path-a-finding-names-the-in-tree-link-path), [§FS-scan.identity.findings.content](FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name), [§FS-scan.identity.findings.place](FS-scan.md#identityfindingsplace-a-place-finding-is-judged-at-each-spelling)). The URI it is published under resolves a linked project root but keeps that in-project spelling below it. A scan failure is named by the whole run's spelling ([§FS-check.2.4.2](FS-check.md#242-one-failure-one-line)), while whether it is one of the project's errors is judged as `grund check <root>` judges it, under every name the walk met it by ([§FS-check.1.3.6](FS-check.md#136-an-explicit-path-still-narrows)).

When edits arrive faster than the server rechecks, it checks the latest text rather than every keystroke's, so a version merged into a later recheck publishes nothing of its own. Each publish for an open document carries the document version it was checked at, and a publish for an older version never follows one for a newer.

Two run-level warnings that `check` prints on stderr are published too, each on the `grund.toml` it is located at with the CLI's message byte for byte ([§DA-engine-renders-nothing.2](../decisions/architectural/DA-engine-renders-nothing.md#2-decision)): the unread opted-out block of [§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread) at its line, and the ancestor claim [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces) leaves undecidable at the file's first line, because the line is what it could not read. A block that swallows its own scan is not one of them: it is a config that will not load ([§FS-check.3.30](FS-check.md#330-a-workspace-member-swallows-the-blocks-own-scan), [§FS-lsp.2.2.4](FS-lsp.md#224-an-unusable-folder-is-skipped)).

### 1.2 Hover preview

`textDocument/hover` answers on both sides of a citation: on a citation that resolves it previews the body the command line prints for it, and on a declaration-side title it shows how much of the tree cites that title.

#### 1.2.1 Citation preview

A hover on a resolving citation returns, before linkification, the bytes `grund <coordinate> --toc` prints for the coordinate it names ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)), a section's or named slice's included ([§FS-show.2.2](FS-show.md#22-section)), and where a stub points at a source declaration the comment-stripped prose of [§FS-show.2.3.2](FS-show.md#232-stripping-comment-prefixes). A Markdown, source-comment or value-source binding previews the same `--toc` slice; no rendered or interpolated value is invented ([§FS-values.6](FS-values.md#6-shared-catalog-consumers)). The full hover may add the kind paragraph of [§FS-lsp.1.2.8](FS-lsp.md#128-target-kind-metadata) after those bytes.

The preview is Markdown. Each resolving citation in it that the one reading of [§FS-citations.wrap](FS-citations.md#wrap-the-wrap) reads as unwrapped becomes a link to its declaration target, so a reader follows the grounding graph without closing the hover. A wrap, a citation in a link its author wrote in any form ([§FS-citations.wrap.enclosing.shape](FS-citations.md#wrapenclosingshape-which-links-enclose)), a citation-only image and every never-rewrite zone ([§FS-citations.zones](FS-citations.md#zones-where-citation-text-is-never-rewritten)) stay as written, so the hover never shows a link inside a link.

A citation that does not resolve, and so carries a diagnostic such as an unknown reference with its nearest-ID hint, has no hover: the diagnostic is the one place its message appears, and an editor that draws diagnostics inside the hover popup would otherwise show it twice.

#### 1.2.3 Title hover

On a declaration-side title — a Markdown declaration heading, its doc-comment form, a numbered or named section heading, or an inline-spec stub title — hover returns one line of Markdown, before any kind paragraph ([§FS-lsp.1.2.8](FS-lsp.md#128-target-kind-metadata)): the title token as inline code, ` — `, and a usage clause, over the whole title span ([§FS-lsp.1.3.4](FS-lsp.md#134-origin-span-and-result-shape)). It is a count rather than a body preview because the cursor is already in that body; the sites themselves are the list of [§FS-lsp.1.3.1](FS-lsp.md#131-citations-from-declarations).

```text
`FS-001-alpha: Alpha` — cited at 3 sites across 2 files
```

The token is the title as the document writes it, on a numbered section heading the heading text alone (`1. Capabilities`, not a composed `FS-lsp.1: Capabilities`). A relative heading ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)) is the exception: its token is the canonical path with the title kept, `` `format.1: Username` `` for `### .1: Username`, while its range and definition stay on the written line ([§DF-relative-section-headings](../decisions/functional/DF-relative-section-headings.md#df-relative-section-headings-a-heading-may-write-only-the-component-it-adds)). A named heading's token and range cover its whole rendered title, name and colon included; a marked root's leave out the separating space and tag ([§FS-values.6](FS-values.md#6-shared-catalog-consumers)). A title containing backticks is fenced with a backtick run one longer than its longest, padded with one space at each end when the title starts or ends with a backtick.

The clause is the line `grund refs --total` prints, `cited at <n> site(s) across <m> file(s)`, in which only `site` and `file` vary, taking a plural `s` at every count but one; a title with no citations reads `not cited` in place of the whole clause, beside the unused-declaration warning where that is published ([§FS-check.4.1](FS-check.md#41-unused-declaration)). It counts exactly the set [§FS-lsp.1.3.1](FS-lsp.md#131-citations-from-declarations) returns for the same title, the one `grund refs` lists from the root of the project that answers the document ([§FS-lsp.2.2.2](FS-lsp.md#222-one-project-answers-each-document)), so the count and the list never disagree:

- On a whole-ID title, a declaration heading, its doc-comment form or a stub pointing at it, the ID's citations, those in scanned source comments included and `[reference] strict` honoured: `grund refs <ID> --total` ([§FS-refs.3.4](FS-refs.md#34---total)). Where the workspace root's project answers, that is `grund refs <alias>/<ID>` from the root, a member's own `§<ID>` and a sibling's `§<alias>/<ID>` together ([§FS-workspace.8.2](FS-workspace.md#82-grund-refs)); where a member opened as its own folder answers, `grund refs <ID>` inside the member.
- On a section heading, numbered or named, the citations of that section and every section below it: `grund refs <ID> --section <s> --descendants --total` ([§FS-refs.2](FS-refs.md#2-behaviour)).

#### 1.2.8 Target-kind metadata

When the resolved target's kind has an effective title ([§FS-config.3.4.3](FS-config.md#343-title)), a successful hover ends with `"\n\nKind: "` and that title as a CommonMark code span, fenced and padded as a title token is ([§FS-lsp.1.2.3](FS-lsp.md#123-title-hover)), after the otherwise unchanged hover. Citation, value-binding, declaration, section, inline-source title and stub hovers carry it. A kind with no title adds nothing, an empty configured title still adds the paragraph, and the title is literal, never read as Markdown or linkified ([§DF-configured-title-metadata](../decisions/functional/DF-configured-title-metadata.md#df-configured-title-metadata-kind-titles-are-separate-target-metadata)).

### 1.3 Go-to-definition

`textDocument/definition` on a citation jumps to the declaration or section it names, a named citation to its named heading. A citation of a stubbed declaration, and the stub's own ID or title, lands on the source declaration the stub points at ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)). A value binding lands on its component's heading or exact value-source key or element, and a root-aimed binding ([§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)) on its root: the whole declaration's heading or value-source key, or the marked or chapter root's own heading. Over a Markdown declaration heading's title span, definition answers the other way and returns the ID's citations, as [§FS-lsp.1.3.1](FS-lsp.md#131-citations-from-declarations) does; on a numbered or named section heading, the citations of that section and every section below it.

Recognition is the shared one, configured word-character markers included ([§FS-citations.recognized.marker-start](FS-citations.md#recognizedmarker-start-the-configured-marker-establishes-the-citation-start)): with marker `_` and format `{kind}_{slug}`, a cursor on the marker or inside the ID in `//! 😀 _FS_login` resolves to the `FS_login` declaration. Positions and ranges count UTF-16 code units, so the non-ASCII prefix does not shift the origin span.

A declaration-local numeric citation its declaration owns ([§FS-citations.recognized.local-sections](FS-citations.md#recognizedlocal-sections-declaration-local-numeric-section-candidates)) answers as its canonical full citation, landing on the exact numeric section heading and taking part in that declaration's and section's lookups and highlights, with its own written range. A missing, ownerless, ambiguous or unsupported one has no definition, citation identity, document link, hover target or highlight.

#### 1.3.1 Citations from declarations

`textDocument/references` on a declaration ID or anywhere in its title returns every citation of that ID, those in scanned source-code comments included: the set `grund refs <ID>` reports ([§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)). On a numbered or named section heading it returns the citations of that section and every section below it. The same request on a citation returns the citations of its target, a named citation's being that same section-scoped set, so an editor finds usages from either side.

#### 1.3.2 Document links

`textDocument/documentLink` marks each resolving citation token as a link to its declaration target, a named citation as one whole range to its named heading line, and each inline-spec stub title as a link to the source declaration it points at, so they are single navigable units even in source files, where cross-reference emission cannot run ([§FS-fmt.6.1](FS-fmt.md#61-scope)). The target is a file URI with a `#L<n>` line fragment; an editor that ignores the fragment opens the file without moving to the line, and definition remains the exact fallback.

A Markdown declaration title, a numbered section heading and an explicit named heading are not links: a link there would shadow the editor's link gesture, which on a title is go-to-definition and returns its citations ([§FS-lsp.1.3](FS-lsp.md#13-go-to-definition)).

#### 1.3.3 Occurrence highlight

`textDocument/documentHighlight` marks the whole token under the cursor, a citation or a declaration, section or stub title, as one span, and every other occurrence of the same ID in that document: its declarations, section headings, stub titles and sibling citations. Usages in other files are [§FS-lsp.1.3.1](FS-lsp.md#131-citations-from-declarations)'s. One span per token keeps an editor from falling back to its word pattern, which splits a citation on `§`, `-` and `.` and boxes one word.

A complete named citation and a complete named heading title are each one unit, and so is a mixed coordinate ([§FS-config.3.3.1](FS-config.md#331-named-components)) such as `<§>FS-x.1.goals`, which hover and definition answer as the section it names. A highlight never shortens a missing or mixed letter-bearing tail to the bare ID or its numeric prefix.

#### 1.3.4 Origin span and result shape

A definition result's origin span is the whole token it started from, a citation or a declaration, section or stub title, so the editor underlines it as one unit on the link gesture. The server sends `LocationLink` results unless a client explicitly sets `textDocument.definition.linkSupport = false`; a client that omits the flag still gets them, because a plain `Location` cannot carry the origin span.

A declaration or section title's span runs from the ID or the section's number to where the `grund <ID> --toc` title ends: on a C-family heading line such as ` * ## 1. One */` it ends before the `*/`, which is comment envelope ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)), and a Markdown heading's closing hashes are left out wherever the `--toc` title leaves them out ([§DF-fmt-and-editor-answers-from-0-19](../decisions/functional/DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them)).

### 1.4 Live trigger transform

`textDocument/onTypeFormatting` replaces the configured trigger (default `$$`, [§DF-reference-marker.2.2](../decisions/functional/DF-reference-marker.md#22-trigger)) with the marker (default `§`) the moment the text after it reads as an ID in its kind's effective format ([§FS-config.3.2](FS-config.md#32-id--id-grammar)): `FS-007` under a numbered format, `FS-login` under the slug-only form. It is the live counterpart of `grund fmt`'s trigger pass ([§FS-fmt.2.1](FS-fmt.md#21-trigger-to-marker)). Where the typed token is a number-only shorthand ([§FS-citations.shorthand](FS-citations.md#shorthand-the-number-only-shorthand)) that resolves to exactly one declaration, the server also expands it under either `[reference] shorthand` policy, as [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical) would: typing `$$FS-042` leaves `<§>FS-042-user-login`, not a `<§>FS-042` that under the default `canonical` policy earns a finding ([§DF-number-only-citation-shorthand.2.2](../decisions/functional/DF-number-only-citation-shorthand.md#22-where-the-shorthand-is-accepted-and-where-it-is-an-error)).

The two rewrites fire on different keystrokes. The trigger converts on the first keystroke after which the text reads as an ID, under the default format the first digit, and replaces only the `$$`. The expansion waits for the keystroke that ends the token, a character that cannot continue an ID by the one-character boundary of [§FS-citations.shorthand](FS-citations.md#shorthand-the-number-only-shorthand), in which a `format` literal just typed continues the token; so `$$FS-12` is never expanded as `FS-1` with a digit left behind. A period ends the token and is kept: `$$FS-042.1` lands on `<§>FS-042-user-login.1`.

The expansion reads the declarations of the scan the server already holds for the document's own project ([§FS-lsp.2.2.2](FS-lsp.md#222-one-project-answers-each-document)), never a fresh scan, and in a workspace never a sibling member's: `<§>FS-042` typed in one member means that member's `FS-042-…` ([§FS-workspace.4](FS-workspace.md#4-resolution)). A shorthand that matches no declaration or several only has its trigger converted, and `check` reports it ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). A shorthand already in the document, whatever finding its policy earns, answers hover, definition, citation lookups, document links and highlights as the declaration it resolves to, over its written token.

Only the number-only shorthand expands as it is typed: a declaration-local numeric citation ([§FS-citations.recognized.local-sections](FS-citations.md#recognizedlocal-sections-declaration-local-numeric-section-candidates)), which `grund fmt --write` expands to its declaration's full ID ([§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)), is never expanded here, so typing `$$2` expands nothing.

The expansion previews `grund fmt --write` and rewrites nothing it would not: it refuses in every context [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten) and [§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes) name, a fenced code block whose citations `[schema.sources] fenced` makes live included ([§FS-languages.fence.language](FS-languages.md#fencelanguage-a-fences-tag-and-the-fences-fenced-selects)). The server hands the core the whole document, not the edited line, so a context one line cannot show, such as whether the line is inside a Python docstring ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)), is judged as `fmt` judges it.

The trigger conversion replaces only the two characters just typed and leaves the ID token byte-identical, so it stays eager in a fenced code block, selected by `[schema.sources] fenced` or not, on a declaration heading and inside a suppressed scope. It refuses only where the marker may not be typed at all: an ordinary string literal in a source file, which a Python docstring's content is not, and, in Markdown, every zone [§FS-citations.zones](FS-citations.md#zones-where-citation-text-is-never-rewritten) names, a link destination, a reference key and an HTML tag among them.

The trigger, marker and recognized kinds are those of the edited document's own project, a member's own overrides included, as `grund fmt` reads them ([§FS-workspace.5](FS-workspace.md#5-command-scope)), and the defaults of [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger) and [§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up) where no config applies.

### 1.6 Declared-ID completion

`textDocument/completion` offers declared IDs, never sections or value-specific choices, while an author writes a citation, and accepting one inserts one resolving citation through a plain-text edit.

A request is eligible after a complete introducer, the document owner's effective marker or trigger, followed by an empty or partial ID: the longest complete introducer counts, one is required even with `strict = false`, and an empty marker disables marker entry while a nonempty trigger stays usable. A manual request right after the trigger offers choices but converts nothing, so a `$$` in money or prose stays as written unless an item is accepted. Completion is offered nowhere `grund fmt` refuses a whole token ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten), [§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes)), while comments and real Python docstrings stay eligible; these are not the trigger conversion's exclusions ([§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform)).

Initialization advertises as trigger characters the deduplicated final single characters of the nonempty markers and triggers of the projects and members loaded at start, never a multi-character string. A trigger character that a later config or workspace-folder change introduces triggers automatically only after a client restart; a manual request always uses the current configuration.

Candidates come from the owner's cached catalog. A complete known alias and `/` select the loaded target project's catalog and grammar through the qualified resolver ([§FS-workspace.4](FS-workspace.md#4-resolution)), and an unknown or unavailable alias yields no items. Each alias an import reached, the import's own and every alias reached through it, is offered with its `/` wherever it continues past the typed token, by the rule `complete ids` offers an alias path by ([§FS-workspace.8.4.1](FS-workspace.md#841-a-prefix-containing-), [§FS-imports.editor](FS-imports.md#editor-the-editor-reads-the-import-graph-the-command-line-reads)). An unqualified sibling declaration is never offered. Only uniquely resolving declaration spellings are offered, an off-grammar declaration's exact spelling included ([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)), and incomplete declaration discovery withholds edits. Each item carries its ID, declaration title and project-relative source path. Matching is case-sensitive prefix matching with no fuzzy fallback; exact-prefix matches come first, then configured kind order, then bytewise ID, as explicit `sortText` encodes, and `filterText` holds the original introducer and the candidate ID with any written qualification.

An item's edit is one minimal, single-line plain-text edit replacing the whole active citation token, the old suffix right of the cursor and any existing section suffix included, with the effective marker and the chosen full ID; a request within a section yields no candidates. Token boundaries respect the configured grammar's punctuation and citation delimiters.

A request reads the document's live text, that of the last notification before it, and answers as a full rebuild would ([§FS-lsp.responsiveness.4](FS-lsp.md#responsiveness4-a-typing-request-waits-for-no-rebuild-while-the-declarations-stand)). Completion lists have `isIncomplete = true`, so further typing refreshes the choices. However completion and the live transform interleave, they leave one canonical citation, which `grund fmt --check` accepts wherever `fmt` may rewrite, and replaying the on-type request on the result changes nothing. A plain text edit cannot stop a client from applying a stale response, so after applying either edit a client discards competing obsolete responses and asks again. Acceptance keys, Tab among them, are the client's ([§FS-lsp.2.3](FS-lsp.md#23-editor-configuration-one-time-per-editor)); the server intercepts none.

## 2. Installation and lifecycle

### 2.1 Install

`grund-lsp` is a Cargo package of its own, published where the target table of [§FS-distribution.1](FS-distribution.md#1-targets) says: `cargo install grund-lsp` installs a release and `cargo install --path crates/grund-lsp` a checkout. `cargo install grund` does not pull it in, and neither install requires the other. The package embeds the editor configuration of [§FS-lsp.2.4](FS-lsp.md#24-installed-editor-integrations), so an installed binary needs no `editor/` tree from this repository. Its npm and PyPI packages exist only as a local candidate ([§FS-distribution.4.13](FS-distribution.md#413-a-candidate-is-rehearsed-before-anything-is-published)), so `npm install -g grund-lsp` and `pipx install grund-lsp` are documented against a candidate's own files, never as registry availability; each installs the server alone, with no CLI and no Rust toolchain, and puts `grund-lsp` on `PATH`, the command an editor launches however it was installed ([§FS-distribution-candidate.5.2](FS-distribution-candidate.md#52-every-registry-row-installs-fresh-and-without-rust)).

### 2.2 Lifecycle

Users do not run `grund-lsp` directly: the editor's LSP client starts it as a child process when a Markdown file or a file of a scanned extension is opened in a workspace holding a `grund.toml`, in either discovery location, or an `AGENTS.md`, and ends it when the workspace closes. It speaks LSP over stdio, with no daemon, socket or background service, and a batch context never invokes it: there the CLI is the only entry point.

#### 2.2.1 Workspace folders anchor discovery

The folders of the `initialize` request are where config discovery starts, not scan boundaries: from each `workspaceFolders` entry the server walks upward by the CLI's discovery rules ([§FS-lsp.3](FS-lsp.md#3-configuration)) and scans the found config's project root, so its `[scan] include` paths and sibling source trees stay visible when the editor opened only a nested directory. With `workspaceFolders` absent or empty it starts from the deprecated `rootUri`, then its current directory; with no config found, the start is the zero-config scan root with the canonical defaults ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)). Folders that reach the same project root share one scan; different roots get scans of their own and are never merged, because identical local IDs in two folders belong to unrelated projects ([§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess)).

The server advertises workspace-folder support with change notifications. On `workspace/didChangeWorkspaceFolders` it discovers added folders by the same rules, drops what removed ones contributed and republishes diagnostics from the resulting scans; an unusable entry is skipped ([§FS-lsp.2.2.4](FS-lsp.md#224-an-unusable-folder-is-skipped)) and the rest of the event applies, and a project stays active while any folder still resolves to it.

#### 2.2.2 One project answers each document

Each document is answered by at most one project: one whose root contains it, the deepest when project trees nest, so a member opened as its own folder answers for its own files. Where no root contains it, a project that scans it answers, since `[scan] include` is a scan scope, not a fence ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), and may reach outside the project directory. Where nested projects both read a file, the owner's verdict is published alone, so a file never collects one finding twice or two projects' readings of one citation.

A document reachable only below a directory symlink out of the project root is pruned ([§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)) and has neither diagnostics nor answers. A document two projects reach only through their scans is nobody's: requests about it return no result and neither project publishes its diagnostics, because choosing by folder order or root shape would guess between independent projects ([§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess)). A document under an import's root is not nobody's, however many consumers import it: the import's own project answers it, read-only ([§FS-imports.editor](FS-imports.md#editor-the-editor-reads-the-import-graph-the-command-line-reads)).

#### 2.2.4 An unusable folder is skipped

A folder the server cannot turn into a project is skipped, never fatal ([§REQ-never-crashes](../requirements/REQ-never-crashes.md#req-never-crashes-garbage-in-diagnostic-out)): a folder URI with a non-`file:` scheme is passed over with a note on stderr, and so is a folder whose config will not load, whose project keeps its last good scan while every other folder keeps its diagnostics current. A session with no usable folder left starts and answers nothing rather than exiting.

A config that will not load is also one diagnostic, severity `error` and source `grund`, on the config file its load error names, `grund.toml` at the root or under `.agents/`, at the line that error names, a semantic refusal after parsing included: an error naming `grund.toml:8:` sits at zero-based line 7, and one naming no line at the file's first line. Its message is the error `grund check` reports for the same file, followed, when the project has a last good scan, by a sentence saying that its other diagnostics, which are not withdrawn, are from that scan. The next successful load of that folder clears it with an empty publish for the config file.

### 2.3 Editor configuration (one-time, per editor)

The user-facing LSP setup guide ships a client configuration for Helix (three lines in `languages.toml`), Neovim (a built-in LSP snippet, compatible with `nvim-lspconfig` setups), Zed, Emacs (`eglot-server-programs` or `lsp-mode`), Sublime Text (the LSP package, for Markdown and scanned source syntaxes), VSCode and VSCodium (a generic LSP client extension pointed at `grund-lsp`), and the IntelliJ family (LSP4IJ's import of the directory [§FS-lsp.2.4](FS-lsp.md#24-installed-editor-integrations) generates, through **Settings | Languages & Frameworks | Language Servers**, **+ | New Language Server**, **Import from custom template...**).

For VSCode the guide says that under default Workspace Trust handling Simple LSP Client 0.1.22 and `zsol.vscode-glspc` 0.2.1 need a trusted folder, how Restricted Mode shows (a status-bar badge, no hover or diagnostics, no `grund-lsp` process), and that trusting a parent folder through **Workspaces: Manage Workspace Trust** covers the checkouts beneath it, while user settings register the server but grant no trust. For IntelliJ it teaches the client configuration of [§FS-lsp.2.4](FS-lsp.md#24-installed-editor-integrations). It teaches completion ([§FS-lsp.1.6](FS-lsp.md#16-declared-id-completion)): its exclusions, the restart and manual-request fallback, obsolete responses, and that acceptance, Tab included, is the client's, with a minimal mapping only where an editor needs one. This repository tracks no file under IntelliJ's `.idea/`, which `.gitignore` ignores, so the LSP4IJ hint lives in the guide, never in editor settings the repository owns.

### 2.4 Installed editor integrations

`grund-lsp integrations` lists on stdout the editor-client configurations the installed `grund-lsp` carries: one entry, `lsp4ij`, with a one-line description. It writes nothing, leaves stderr empty and exits `0`. It is not `grund integrations`, which configures clickable-citation rendering clients ([§FS-integrations](FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations)).

`grund-lsp integrations lsp4ij` is a read-only preview: it prints the complete generated `template.json` object and then the IntelliJ import steps of [§FS-lsp.2.3](FS-lsp.md#23-editor-configuration-one-time-per-editor) on stdout, writes nothing, leaves stderr empty and exits `0`. `grund-lsp integrations lsp4ij --write <directory>` makes `<directory>` an LSP4IJ import root holding exactly `template.json` and `README.md`, the README holding the same steps, prints `created <directory>` and the steps on stdout, and exits `0`; repeating it against those two byte-identical files succeeds and prints `unchanged <directory>` and the steps on stdout. A root that exists with a missing, extra or byte-different entry is a conflict: every existing byte stays, stdout is empty, one `error:` message on stderr says to move or remove the root before retrying ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)), and there is no force option.

After the numbered import steps, the steps, wherever they are printed or written, tell the user to select the imported **grund LSP** server under **Settings | Languages & Frameworks | Language Servers** and set or merge this JSON into its **Client configuration**, keeping other client settings and other entries within `format`:

```json
{"format":{"existingFormatterOverrideable":true}}
```

They say why: where IntelliJ has a formatter for a language, Java among them and Markdown once a formatter is installed, LSP4IJ otherwise withholds on-type formatting requests from `grund-lsp`, so the default `$$` would never become `§` ([§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform)).

`template.json` maps languages from the effective `[scan] extensions` ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)) of the config discovered upward from the process's current directory ([§FS-lsp.3](FS-lsp.md#3-configuration)), or the canonical defaults where there is none: each extension is one `*.ext` pattern with its conventional language ID, `md` → `markdown`, `rs` → `rust`, `ts` → `typescript` and every canonical default among them, and an unknown extension is its own non-empty language ID. The mapping is fixed at generation; a later config change needs regeneration. Its `programArgs.default`, for a POSIX shell, and `programArgs.windows`, for Windows `cmd`, each change to `$PROJECT_DIR$` and launch the quoted absolute path of the running `grund-lsp`, never through Cargo or a shell `PATH` lookup, and the one selected on the generating host is executable there. No space, non-ASCII character or JSON or command metacharacter in a path can change the command or corrupt the JSON.

An unknown template, malformed arguments, an invalid discovered config, or a render, path or write failure or conflict leaves stdout empty, prints one `error:` message on stderr and exits `2`; top-level and `integrations` help show the list, preview and write forms and both exits. Any argument is handled or refused as batch input and never enters the protocol loop; with no arguments, `grund-lsp` is the stdio session of [§FS-lsp.2.2](FS-lsp.md#22-lifecycle), with protocol stdout pristine.

## 3. Configuration

The server reads `grund.toml` by the discovery `grund check` uses ([§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up)), walking up from every workspace folder the editor supplies, with the fallbacks of [§FS-lsp.2.2.1](FS-lsp.md#221-workspace-folders-anchor-discovery), while `grund-lsp integrations` walks up from the process's current directory ([§FS-lsp.2.4](FS-lsp.md#24-installed-editor-integrations)). Both consume the same effective core configuration; there is no separate LSP config and no second parser. The editor's own LSP configuration, its server arguments and workspace folders, is the user's ([§FS-lsp.2.3](FS-lsp.md#23-editor-configuration-one-time-per-editor)) and no part of `grund.toml`.

## 4. Determinism and parity with the CLI

Configured word-character markers yield the same citation edges in the CLI and the editor;
definition navigation has no editor-only recognition fallback
([§FS-lsp.1.3](FS-lsp.md#13-go-to-definition),
[§FS-check.1.1.10](FS-check.md#1110-the-configured-marker-establishes-the-citation-start)).

Same input, same answer: the parity is [§REQ-one-answer.text.parity](../requirements/REQ-one-answer.md#textparity-the-editor-answers-as-the-command-line-does).

Completion candidates, metadata, ordering and edit ranges obey the same promise
([§FS-lsp.1.6](FS-lsp.md#16-declared-id-completion)).

The LSP server does not have an "interactive" mode or a confirmation prompt ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)). It is the same engine with a different transport.

## responsiveness: What an answer may cost

[§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli) fixes what the server answers; this chapter fixes what answering may cost. The budget is [§GOAL-fast-feedback.1](../goals.md#1-performance-targets)'s, the one the scan itself is held to, and it binds the work the server does after a snapshot exists and what a request waits for. Each bound is a complexity, and instruction counts gate it ([§DA-benchmark-instruction-counting](../decisions/architectural/DA-benchmark-instruction-counting.md#da-benchmark-instruction-counting-the-performance-harness-counts-instructions-not-wall-clock-seconds)). The one deadline in milliseconds, that of [§FS-lsp.responsiveness.4](FS-lsp.md#responsiveness4-a-typing-request-waits-for-no-rebuild-while-the-declarations-stand), names its reference: the GraalVM workspace and the machine recorded in `docs/benchmarks.md`, with the build and commit it was measured at. That record holds the deadline; CI does not.

### responsiveness.1: Publishing a diagnostic set is linear in the findings

For a project whose check reports `F` findings over a snapshot holding `E` records, publishing the diagnostics of [§FS-lsp.1.1](FS-lsp.md#11-diagnostics) costs on the order of `F + E`, never `F × E`: anchoring one finding does an amount of work that does not grow with how many records the snapshot holds. A workspace part-way through adoption, where `F` is large because the tree is not conformant yet, is published in about the time the CLI reports it in.

### responsiveness.2: A path is resolved once per answer, never once per comparison

Resolving a path to the one spelling a scan gave it reads the filesystem once for every component of the path, so a resolution inside a lookup is what would make [§FS-lsp.responsiveness.1](FS-lsp.md#responsiveness1-publishing-a-diagnostic-set-is-linear-in-the-findings) quadratic. The records in a snapshot already carry their paths in that spelling, so every answer, a published diagnostic, a hover, a definition, an occurrence highlight, a document link or a references reply, finds its document by resolving the request's own path once and comparing resolved values; no comparison resolves either side a second time.

### responsiveness.3: A citation hover reads nothing the snapshot did not

Once a snapshot exists, a citation hover ([§FS-lsp.1.2.1](FS-lsp.md#121-citation-preview)) answers from it: it loads no workspace, and reads no file and observes no checking input that the snapshot's own build did not already read, so its cost grows with the cited body and not with the tree. A hover, highlight or link sent after an edit is a snapshot request, answered from a snapshot built from exactly the text it was sent against: it waits for the rebuild that covers its edit, never for a backlog of rebuilds behind it. That holds for a client that cancels the requests it no longer needs, because a snapshot request left uncancelled seals the rebuild of its own version; one the client cancels is answered `RequestCancelled`.

### responsiveness.4: A typing request waits for no rebuild while the declarations stand

A typing request, a completion ([§FS-lsp.1.6](FS-lsp.md#16-declared-id-completion)) or the live transform's on-type request ([§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform)), is answered from its own text and the project's declarations, without waiting for the snapshot to be rebuilt, whenever the edits before it left those declarations unchanged. Between reading an edit and answering the typing request after it the server reads no input but the edited document, so the cost is that document plus the catalog it filters, never the tree; on the reference [§FS-lsp.responsiveness](FS-lsp.md#responsiveness-what-an-answer-may-cost) names, `grund`'s share of such an answer is under 10 ms, which leaves the rest of [§GOAL-fast-feedback.1](../goals.md#1-performance-targets)'s 100 ms to the editor.

"Unchanged" is judged by reading the edited document again as the scan reads it, under its project's config, and comparing the declarations that reading gives with the ones the installed catalog holds for that file. It is judged again each time a rebuild is installed, against the catalog installed at that moment, so a rebuild of an older text never stands in for a later one: a heading renamed, and renamed back before the rebuild of the rename lands, never offers the name it held in between. A document the one-file reading cannot judge exactly counts as changed: a config or ignore file, a stub or a stub's target, a file no project scanned. After an edit that did change the declarations, a typing request waits for the rebuild of every project that rebuild covers, not only the document's owner, because each may offer the IDs that changed. Either way its answer is the one a full rebuild gives, never an ID that has gone and never one short of an ID that is new ([§FS-lsp.requirements.no-guess](FS-lsp.md#requirementsno-guess-no-answer-is-a-guess)).

## 5. Out of scope

- **Per-editor wrappers**: no first-party VSCode, IntelliJ, Vim or Emacs plugin duplicates the LSP server or is published to a marketplace ([§FS-non-goals.12.2](FS-non-goals.md#122-first-party-per-editor-plugins)). The LSP server is the executable surface; [§FS-lsp.2.4](FS-lsp.md#24-installed-editor-integrations) ships importable configuration data, and the user installs the generic client and performs the import. The terminal-link extension the `grund` binary carries is not an LSP client ([§FS-integrations.3.2](FS-integrations.md#32-editor-clients-vscode-codium)).
- **Code actions, symbols and commands**: the server advertises no `textDocument/codeAction`, no `workspace/symbol` and no `workspace/executeCommand`, so a missing snapshot has no "Fetch" action and the server never runs `grund fetch`.
- **Refactoring (rename ID)**: `grund` does not rename IDs ([§FS-non-goals.4](FS-non-goals.md#4-cross-workspace-id-renaming)).
- **Inline editing of declaration bodies from the hover popup**: `grund-lsp` does not implement it.
- **Network access**: the server performs no network I/O ([§FS-non-goals.11](FS-non-goals.md#11-network-access-during-a-check)). All scanning is local.

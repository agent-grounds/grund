# FS-lsp: grund ships an optional LSP server

`grund` ships an optional Language Server Protocol server, `grund-lsp`, as a separate binary that any LSP-aware editor can talk to: VSCode, Neovim, Emacs (eglot or lsp-mode), Helix, Zed, Sublime Text, and the IntelliJ family via LSP4IJ. Users who want editor integration install `grund-lsp` and configure their editor once; users who do not — CI pipelines, pre-commit hooks, contributors who only run `grund check` — install nothing extra and pay no dependency cost. The architectural choice (separate binary rather than a Cargo feature or a bundled library) is decided in [§DA-lsp-optional](../decisions/architectural/DA-lsp-optional.md#da-lsp-optional-lsp-server-ships-as-a-separate-optional-binary). The Cargo crate and its release path shipped in 0.4.1; npm/PyPI packaging of the same server rides with [§RM-distribution](../roadmap.md#rm-distribution-cargo--npm--pypi-from-one-engine).

`grund` does not ship per-editor wrappers ([§FS-non-goals.12.2](FS-non-goals.md#122-first-party-per-editor-plugins)): the first-party executable editor surface is the LSP server, and per-editor configuration is one-time work the user does, with example snippets and importable configuration data in the user-facing LSP setup guide.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, qualified citation, shorthand, citation site),
[§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, workspace, member,
alias), [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction, rule),
and [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, component, binding, fetcher, snapshot).

- **diagnostic** — The LSP protocol object a finding is published as. The word names the
  protocol object here and nothing else.
- **workspace folder** — A root the editor supplies in its `initialize` request, from which
  config discovery walks up.
- **snapshot (narrowed)** — [§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations). In this spec, within the server's own state, it
  also names the immutable scan result one request is answered from; elsewhere the shared
  definition applies.

## 1. Capabilities

The minimum viable set — everything the server speaks at version 1.0. Diagnostics ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)), hover ([§FS-lsp.1.2](FS-lsp.md#12-hover-preview)), and go-to-definition ([§FS-lsp.1.3](FS-lsp.md#13-go-to-definition)) are illustrated in the project README; each illustration is captured for both light and dark editor themes (paired `<name>-light.png` and `<name>.png` `prefers-color-scheme` sources), so a screenshot refresh updates both variants together.

A single malformed or failing message is not fatal to the session: a request that fails returns an LSP error response, a notification whose params do not parse is logged and skipped, and the server keeps serving subsequent messages rather than dropping the connection.

### 1.1 Diagnostics

The server publishes every declaration-local numeric citation verdict from
[§FS-check.3.24](FS-check.md#324-declaration-local-section-citation) through the ordinary core
report: the same code, severity, message, and complete authored-token range as `check`. A missing
owned path also publishes the independent missing-section finding. Ownerless and unsupported
forms receive no guessed target.

`textDocument/publishDiagnostics` pushes `grund check` results as the user edits. Each unknown reference, missing section, duplicate declaration, broken stub, citation-direction violation, hard chapter-rule violation, and `invalid-rule` becomes a diagnostic with the same path, line, code, severity, message, and title/citation range as the CLI's shared core report ([§FS-rules.9](FS-rules.md#9-managed-guidance-and-editor-parity)). The advisory `should` / `should-not` suggestions channel ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)) is opt-in on the CLI and is not pushed as diagnostics. Severity follows the engine's severity model ([§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization) — not configurable).

Where each diagnostic anchors is [§FS-lsp.1.1.1](FS-lsp.md#111-where-a-diagnostic-anchors). Value, named-section, and missing-snapshot findings are transported from the core report rather than derived by the server ([§FS-lsp.1.1.2](FS-lsp.md#112-findings-the-core-report-shapes)), and the `[workspace]` warnings on the run's warning channel are published too ([§FS-lsp.1.1.3](FS-lsp.md#113-workspace-warnings-on-the-runs-warning-channel)).

#### 1.1.1 Where a diagnostic anchors

The diagnostic position is the start column of the citation the finding concerns — the offending token, not merely the first citation on the line. A single comment can carry several citations, so each finding anchors to its own token, and a resolving citation beside it stays unmarked. Diagnostics that are line-anchored rather than citation-anchored, such as the opt-in ungrounded-file check ([§FS-check.3.6](FS-check.md#36-ungrounded-source-file-opt-in)), do not borrow a citation range from the same line; otherwise VSCode-style diagnostic hovers would stack unrelated line-level messages onto the citation's own error. Declaration-side findings may still use the declaration, section, or stub title span on that line. Precise column information is computed once per scan and reused across the open editor session.

#### 1.1.2 Findings the core report shapes

The shared core report includes all three value errors for whole declarations and marked roots with the same primary and declaration/component spans as CLI text/NDJSON, so editor diagnostics neither rescan nor reinterpret bindings ([§FS-values.5](FS-values.md#5-resolution-diagnostics-and-exit-status)).

Named-section diagnostics are not a parallel editor rule. In an opted-in repository, missing named coordinates, duplicate named coordinates, orphan name-bearing paths, and named heading-depth mismatches are transported from the core report with the same message, severity, line, and range as the CLI. A citation-side finding selects the complete written citation token; an orphan or depth finding selects the complete named heading title. Independent core findings remain independent diagnostics.

A missing fetch-backed snapshot is not a separate editor rule either. The LSP transports the engine's `dangling` error or `missing-snapshot` warning with the same message, code, severity, and range as the CLI ([§FS-check.4.12](FS-check.md#412-missing-snapshot)), and never executes the configured fetcher while publishing diagnostics.

#### 1.1.3 Workspace warnings on the run's warning channel

A block that swallows its own scan ([§FS-check.4.7](FS-check.md#47-a-workspace-member-swallows-the-blocks-own-scan)), one whose opted-out tree nobody reads ([§FS-check.4.10](FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)), and an ancestor claim left undecidable ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)) reach the server in the same warning channel as every other finding. Three, not four: a `[workspace]` block no enclosing workspace lists is an **error** in `check` ([§FS-check.3.29.13](FS-check.md#32913-in-check-one-of-the-reports-errors)), so the server takes it from `check`'s report like any other located finding and publishes it as an error at the same anchor — the block's `[workspace]` line — rather than off this channel ([§FS-check.3.29.15](FS-check.md#32915-every-frontend-renders-it-anchored-at-the-blocks-workspace-line)). Each is published on the `grund.toml` it anchors at — at the anchored line, or at that file's first line for the undecidable claim, which names no line because the line is what it could not read. The message is the CLI's, byte for byte, and the server neither re-derives the location nor reads it back out of the message text. Only a terminal used to say these, though a configuration that leaves part of the tree unread, or spells its projects two ways, is what a reader of that `grund.toml` needs told ([§DA-engine-renders-nothing.2](../decisions/architectural/DA-engine-renders-nothing.md#2-decision)).

### 1.2 Hover preview

`textDocument/hover` answers on both sides of a citation. On a citation that resolves, it previews the body the CLI prints for it ([§FS-lsp.1.2.1](FS-lsp.md#121-citation-preview)); on a citation that carries a diagnostic, it returns nothing ([§FS-lsp.1.2.2](FS-lsp.md#122-no-preview-for-a-citation-with-a-diagnostic)). On a declaration-side title — a Markdown declaration heading, the same declaration written inline in a doc-comment, a numbered section heading (`<ID>.<section>`, [§FS-lsp.1.3.1](FS-lsp.md#131-references-from-declarations)), or an inline-spec stub title — it returns the title token and how much of the tree leans on it ([§FS-lsp.1.2.3](FS-lsp.md#123-title-hover) to [§FS-lsp.1.2.7](FS-lsp.md#127-a-count-never-a-finding)).

A committed fetched snapshot is an ordinary declaration on this path: hover, definition, references, document links, and highlights use its scanner spans without contacting or executing the integration. A missing snapshot has no hover or navigation target.

#### 1.2.1 Citation preview

`textDocument/hover` on a citation returns the body `grund <ID> --toc` would print ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)), or the `--toc` body of the requested section if the citation includes one ([§FS-show.2.2](FS-show.md#22-section)); a named citation previews the exact named section slice that `grund <ID>.<path> --toc` returns. When the declaration's home is in source code (a stub points at `src/bus.rs`), the hover body is the comment-stripped prose per [§FS-show.2.3.2](FS-show.md#232-stripping-comment-markers) — the same content the CLI returns. Hovering a Markdown, source-comment, or JSON value binding likewise uses the exact `show --toc` slice; no rendered or interpolated value surface is invented ([§FS-values.6](FS-values.md#6-shared-catalog-consumers)). The unadorned preview and the `show --toc` query produce the same bytes before editor linkification; the full hover may also carry the separate kind-metadata paragraph ([§FS-lsp.1.2.8](FS-lsp.md#128-target-kind-metadata)).

The citation hover content is Markdown. Any resolving `§<ID>` citation inside it is emitted as a normal link to its declaration target, so users can keep following the grounding graph without closing the hover.

#### 1.2.2 No preview for a citation with a diagnostic

If a citation has a diagnostic instead (for example an unknown reference with a nearest-ID hint), hover returns nothing: the diagnostic already carries the actionable text through `publishDiagnostics`, and an editor that renders diagnostics inside the hover popup (VSCode among them) would otherwise show that text twice. The diagnostic is the single source of the error message; hover stays reserved for previewing citations that resolve.

#### 1.2.3 Title hover

On a declaration-side title, hover returns the title token and its usage count, with the hover range set to the whole title span. The cursor is already inside the declaration body, so a body preview would only repeat what is on screen; the *usage* is the one fact about a declaration that is visible nowhere on that screen. The whole-title range also gives editors such as Codium the hover affordance. The citation sites themselves are reached on demand through go-to-definition ([§FS-lsp.1.3](FS-lsp.md#13-go-to-definition)) and references ([§FS-lsp.1.3.1](FS-lsp.md#131-references-from-declarations)): the hover is the count, not the list.

An explicit named heading is a declaration-side title just like a numbered heading: its hover range covers the complete rendered heading title, including the handle and colon, and its usage count covers citations of that path and its descendants. A declaration-side marked root retains the ordinary section-title hover and usage count: its semantic title and UTF-16 hover range exclude the separating space and marker, while a raw preview of that section includes the marker on its heading ([§FS-values.6](FS-values.md#6-shared-catalog-consumers)).

#### 1.2.4 The title hover's text

The original title-and-usage content is one line of Markdown — the title token as inline code, then ` — `, then the usage clause:

```
`FS-user-login: Users sign in` — cited at 12 sites across 5 files
```

The token is the title exactly as the document writes it, which on a numbered section heading is the heading text alone — `1. Capabilities`, not a composed `FS-lsp.1: Capabilities` — because the popup is drawn over that very heading, and a string written nowhere in the file would be one more place for file and editor to disagree. A title that itself contains backticks is fenced with a backtick run one longer than the longest run inside it, padded with a single space at each end when the title starts or ends with a backtick, because a backslash does not escape a backtick inside a code span.

The clause is `cited at <n> site(s) across <m> file(s)`, where `site` and `file` take a plural `s` at every count except one, so a lone citation reads `cited at 1 site across 1 file`. Nothing else varies with the numbers — `across` at every count, no separate one-citation phrasing — so only the digits need reading. A title with no citations reads `not cited` in place of the entire clause.

#### 1.2.5 What a whole-ID title counts

`<n>` counts citation **sites** and `<m>` the distinct files those sites live in, over exactly the set [§FS-lsp.1.3.1](FS-lsp.md#131-references-from-declarations) returns for that same title, so the number a reader sees and the reference list they open next from the same token can never disagree. For a whole-ID title — a Markdown declaration heading, its doc-comment form, or the stub that points at it — that set is *by definition* the one `grund refs` reports **from the same root the server was started at** ([§FS-refs.2](FS-refs.md#2-behaviour)): citations inside scanned source comments included, `[reference] strict` honoured. In a workspace the two roots ask different questions: a server rooted at the workspace root counts what `grund refs <alias>/<ID>` reports from there — a member's own `§<ID>` and a sibling's `§<alias>/<ID>` together ([§FS-workspace.8.2](FS-workspace.md#82-grund-refs)) — while a server rooted at the member itself counts what `grund refs <ID>` reports from inside the member, where the sibling's qualified citation lies outside the tree. These are the CLI's numbers rendered in an editor, not a second tally kept beside them ([§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli)).

#### 1.2.6 What a section heading counts

On a numbered section heading the set is the section-scoped one [§FS-lsp.1.3.1](FS-lsp.md#131-references-from-declarations) defines: `§<ID>.<section>` and its deeper subsections — the subtree `grund <ID>.<section> --full` prints ([§FS-show.2.2](FS-show.md#22-section)), and the set that heading's own definition and references return — so the count totals what the reader can navigate to from the token they are hovering. That is deliberately wider than `grund refs <ID> --section <s>`, which keeps only citations whose section coordinate is *exactly* `<s>` ([§FS-refs.1](FS-refs.md#1-inputs)): the flag answers "who cites this section itself", a declaration-side title answers "who leans on this", and only the whole-ID form is a `refs` invocation counted byte for byte. Where the two readings conflict, the one that keeps a title's hover and its reference list in agreement wins — those are one gesture apart in the same editor, while the terminal comparison is one the user has to go looking for.

#### 1.2.7 A count, never a finding

The clause is a count, never a finding. An uncited declaration already earns the unused-declaration warning through `publishDiagnostics` ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics), [§FS-check.4.1](FS-check.md#41-unused-declaration)), and hover does not restate it: `not cited` is the count at zero, worded as a count, so a popup that draws both over an uncited title carries the warning naming the ID and the count answering the hover — one statement each. Nor is the zero case suppressed in favour of the warning, because the warning does not cover every title that can reach zero — `E2E` declarations are exempt from it ([§FS-check.4.1](FS-check.md#41-unused-declaration)) and section headings never carry one — so a hover that fell silent at zero would go quiet exactly where nothing else speaks, indistinguishable from a server that shows no counts.

The counts are read from the scan of the project that owns the document ([§FS-lsp.2.2.2](FS-lsp.md#222-one-project-answers-each-document)), the one diagnostics and navigation answer from, so the same tree and config produce the same bytes ([§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli)) and no hover re-scans to answer.

#### 1.2.8 Target-kind metadata

When the resolved target kind has an effective title, append `"\n\nKind: "`
and that title as a CommonMark code span to the existing successful hover,
without trimming or rewriting its content. Use the title-token backtick fencing
and padding convention of [§FS-lsp.1.2.4](FS-lsp.md#124-the-title-hovers-text). This applies to citation and value-binding
previews, declarations, sections, inline-source titles and stubs. No title
returns the existing hover unchanged; an empty configured title still adds
the paragraph. Metadata is literal: do not interpret Markdown or linkify
citations inside it. Only the existing preview is linkified. Resolve metadata
from the same target context or snapshot, with no extra per-hover scan, fetch
or network execution. Usage counts, ranges, navigation and missing-target or
diagnostic suppression remain unchanged
([§FS-config.3.4.3](FS-config.md#343-title)).

### 1.3 Go-to-definition

An owned declaration-local numeric citation has the same definition target as its canonical full
citation: the exact numeric section heading. It participates in declaration and exact-section
references and in document highlights while retaining its local origin range. A missing,
ownerless, ambiguous, or unsupported local token has no definition, reference identity, document
link, hover target, or highlights. This adds no `$$2` on-type expansion and no quick-fix;
[§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform) continues to transform only the existing
trigger and number-only ID shorthand forms.

`textDocument/definition` on a citation jumps to the declaration's `path:line`. For a stub-and-inline-source pair ([§FS-check.3.4](FS-check.md#34-broken-inline-spec-stub)), the server follows the stub's link and lands on the inline declaration line directly — the user does not stop at the stub — and the same is true anywhere on the stub heading's ID or title text, which is one navigable title span. A normal Markdown declaration heading uses the same whole-title span for declaration-side requests: definition-as-usages and references return the citations of that ID. A numbered section heading inside a declaration body — `## 1. …`, addressable as `<ID>.<section>` — is itself a declaration-side title with the same behaviour, scoped to the section: definition and references on it return the citations of that section (`§<ID>.<section>` and any deeper subsection) rather than of the whole ID.

The result's shape is [§FS-lsp.1.3.4](FS-lsp.md#134-origin-span-and-result-shape); how values and named sections navigate is [§FS-lsp.1.3.5](FS-lsp.md#135-values-and-named-sections).

#### 1.3.1 References from declarations

`textDocument/references` on a declaration ID or anywhere in its Markdown title returns every citation of that ID, including citations in scanned source-code comments, the same set `grund refs <ID>` reports ([§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id)). On a numbered section heading it returns the citations of that section ID (`§<ID>.<section>` and deeper), the section-scoped counterpart of the whole-ID title. The same request on a citation returns that citation's target ID usages too, so editors can show "find usages" from either side of the relationship.

On a named section heading, references returns citations of that exact path and all descendant paths, matching the section-title usage count. A references request on a named citation resolves to that same set.

#### 1.3.2 Document links

`textDocument/documentLink` marks each resolving citation token as a link to its declaration target — a resolving named citation is one whole link range whose target is the named heading line — and each inline-spec stub title as a link to the source doc-comment declaration it points at. Editors that render LSP document links therefore show `§<ID>` references and stub titles as single navigable units even in source files where Markdown cross-reference emission cannot run ([§FS-fmt.6.1](FS-fmt.md#61-scope)).

Ordinary Markdown declaration titles are deliberately **not** document links: a self-pointing link covers the same span the editor's Ctrl-click would otherwise resolve to go-to-definition, so it would shadow that gesture and navigate the title onto its own line instead of showing the declaration's usages. Numbered section headings and explicit named headings are likewise declaration-side titles, not self-links. All of them stay navigable through go-to-definition and references, which return the citation sites ([§FS-lsp.1.3](FS-lsp.md#13-go-to-definition), [§FS-lsp.1.3.1](FS-lsp.md#131-references-from-declarations)).

The link target is a file URI with a line fragment (`#L<n>`) for the resolved declaration or source line. An editor that ignores file-URI fragments may open the file without moving the cursor to the line; `textDocument/definition` remains its exact-position fallback.

#### 1.3.3 Occurrence highlight

`textDocument/documentHighlight` marks the whole `§<ID>` citation — or declaration, section, or stub title — under the cursor as one span, plus every other occurrence of the same ID in the open document. Without it, an editor with no highlight provider falls back to its language word pattern, which splits a citation such as `§FS-lsp.1.3` on `§`, `-`, and `.` and boxes only the bare word at the cursor (`lsp`). Highlights are scoped to the document the request names: the citing token plus same-document declarations, section headings, stub titles, and sibling citations of the same ID; usages in other files are reached through references ([§FS-lsp.1.3.1](FS-lsp.md#131-references-from-declarations)).

For a named coordinate, the complete `§<ID>.<path>` token and the complete named heading title are occurrence units. Highlights never shorten a missing or reserved letter-bearing tail to the bare ID or numeric prefix.

#### 1.3.4 Origin span and result shape

Definition results report the whole originating token as their origin span — the entire `§<ID>` citation or declaration, section, or stub title — so the editor underlines that span as one navigable unit on the link gesture rather than only the word under the cursor. The server sends `LocationLink` results unless a client explicitly sets `textDocument.definition.linkSupport = false`; clients that omit the flag still get the richer range-carrying shape because plain `Location` results cannot express the origin span.

#### 1.3.5 Values and named sections

A value binding navigates through the shared resolver to its existing Markdown/source component heading or exact JSON key/element span. A marked root and component keep their ordinary dotted identities; no synthetic definition or value badge is exposed ([§FS-values.7](FS-values.md#7-workspaces-and-editor-consumers)).

For a named coordinate, definition navigates from the whole citation token to the exact named heading, and declaration-side definition on that heading returns its section-scoped usages. Named and numeric headings use the same scanned ranges and result shapes.

### 1.4 Live trigger transform

`textDocument/onTypeFormatting` watches the configured trigger sequence (default `$$`, per [§DF-reference-marker.2.2](../decisions/functional/DF-reference-marker.md#22-trigger)) and replaces it with the marker (default `§`) the moment the trigger is followed by a token matching its kind's effective format ([§FS-config.3.2](FS-config.md#32-id--id-grammar) — `FS-007` under a numbered format, `FS-login` under the slug-only form). This is the live counterpart to `grund fmt`'s bulk trigger pass ([§FS-fmt.2.1](FS-fmt.md#21-trigger-to-marker)) and is what makes the marker practical to type without leaving the keyboard.

Where the typed token is a number-only shorthand ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand)) that resolves to exactly one declaration, the server also **expands it**, under either `[reference] shorthand` policy: typing `$$FS-042` leaves `§FS-042-user-login` behind, not a `§FS-042` followed — under the default `canonical` policy — by a diagnostic telling the author to finish the job ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). This is the live counterpart to [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical), and the authoring surface is where the shorthand, as authoring sugar, pays ([§DF-number-only-citation-shorthand.2.2](../decisions/functional/DF-number-only-citation-shorthand.md#22-where-the-shorthand-is-accepted-and-where-it-is-an-error)).

#### 1.4.1 When each rewrite fires

The two rewrites fire on **different keystrokes**, and that separation is what makes the expansion correct rather than merely eager. The trigger becomes rewritable the moment the text after `$$` first reads as an ID — under the default format that is the *first digit*, because `FS-0` is already a well-formed shorthand — and converting there is harmless, since it replaces only the `$$` and the author types straight through it. Expanding there is not: the number is unfinished, so `$$FS-12` would be rewritten to whatever `FS-1` happens to name with the remaining digits left trailing behind it. The expansion therefore waits for the keystroke that **ends** the token — a character that cannot continue an ID, judged by the one-character form of the boundary [§FS-check.1.2](FS-check.md#12-the-number-only-shorthand) uses, in which any `format` literal continues the token because a separator just typed has no component after it yet — at which point the typed number is known to be complete. A period is such a character, and the expansion preserves it, so `$$FS-042.1` still lands on `§FS-042-user-login.1`.

#### 1.4.2 What the expansion resolves against

The expansion reads the declaration set from the scan the server already holds for the document's owner ([§FS-lsp.2.2.2](FS-lsp.md#222-one-project-answers-each-document)), never a fresh scan, so the per-keystroke path stays within [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible). In a workspace that scan holds every member's declarations, and only the edited file's own project is consulted: `§FS-042` typed in one member means that member's `FS-042-…` and never a sibling's ([§FS-workspace.4](FS-workspace.md#4-resolution)). A shorthand that matches no declaration, or more than one, converts the trigger and nothing more: typing never stalls, and the resulting `§FS-042` earns the [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) diagnostic that names the problem. The same is true of a token the author never terminates — `grund check` and `grund fmt` are the backstop, and they agree with the editor about what resolves — except under `[reference] shorthand = "accepted"`, where one that resolves to exactly one declaration is a valid persisted shorthand that neither reports nor rewrites ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation), [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical)).

#### 1.4.3 Where the expansion refuses

Because the expansion can rewrite a citation the author did not just type, it honours every context `grund fmt` refuses ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)) — including a file outside the configured scan set and an external file-symlink target ([§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)), the whole-line skips a single line cannot reveal, a fenced code block and a declaration heading, and, in a Python file, whether the edited line is inside a docstring, which is what decides the never-rewrite question there ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)). A **suppressed scope** ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes)) is one of those contexts and is honoured the same way: no expansion in a file the `[fmt] exclude` list names, and none inside a region a `grund:fmt off` above the cursor has opened, since the editor is where a protected region is actually edited.

The server therefore hands the core the document, not just the edited line, so the editor neither expands where `grund fmt --write` would not — an illustration inside a fence, a diagram a region protects — nor refuses where it would, such as `$$FS-042` on a docstring's opening line: it previews that tool and may not disagree with it.

#### 1.4.4 Where the trigger conversion refuses

The trigger conversion, which replaces only the two characters just typed and leaves the ID token byte-identical, stays eager in a fenced code block, on a declaration heading, and inside a suppressed scope. It refuses only where the marker may not be typed at all: an ordinary string literal in a source file — a Python docstring's content is not one, which is what [§FS-lsp.1.4.3](FS-lsp.md#143-where-the-expansion-refuses) decides — and, in Markdown, an inline code span or a link destination.

#### 1.4.5 A shorthand already in the document

A shorthand citation already in the document is a citation like any other for every other capability — hover ([§FS-lsp.1.2](FS-lsp.md#12-hover-preview)), go-to-definition ([§FS-lsp.1.3](FS-lsp.md#13-go-to-definition)), references ([§FS-lsp.1.3.1](FS-lsp.md#131-references-from-declarations)), document links ([§FS-lsp.1.3.2](FS-lsp.md#132-document-links)), and occurrence highlight ([§FS-lsp.1.3.3](FS-lsp.md#133-occurrence-highlight)) all treat `§FS-042` as the declaration it resolves to, and the range they return is the written token, not the canonical one it stands for.

#### 1.4.6 Which configuration drives it

The trigger, marker, and recognized `KIND` set are read from the discovered `grund.toml` so the editor experience matches the project's choices. In a workspace, the replacement uses the config resolved for the edited document, so member-local trigger and marker overrides behave the same as `grund fmt` ([§FS-workspace.5](FS-workspace.md#5-command-scope)). If no config is present, the defaults from [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger) and [§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up) apply.

### 1.5 Capabilities reserved for later

These are out of scope for the first version but compatible with the architecture:

- `textDocument/completion` — autocomplete `§F` to declared `FS-…` IDs from the workspace.
- `textDocument/codeAction` — quick fixes for "unknown reference" (suggest similarly-named IDs) and "section not found" (suggest sibling sections).
- `workspace/symbol` — fuzzy-find IDs across the project.

Each addition is a separate roadmap item if and when it is taken on.

Fetching a missing snapshot is reserved with code actions: this version
advertises neither a “Fetch <ID>” action nor `workspace/executeCommand` and
never runs `grund fetch` from the server.

## 2. Installation and lifecycle

### 2.1 Install

`grund-lsp` is a separate Cargo package per [§FS-distribution](FS-distribution.md#fs-distribution-grund-distribution-targets): after a release, users install it with `cargo install grund-lsp`; from a checkout, contributors install it with `cargo install --path crates/grund-lsp`. The package embeds the editor-configuration artifacts [§FS-lsp.2.4](FS-lsp.md#24-installed-editor-integrations) exposes, so an installed binary does not need this repository's `editor/` tree. It is not pulled in by `cargo install grund`. The npm and PyPI `grund-lsp` packages are future distribution targets, so `npm install -g grund-lsp` and `pipx install grund-lsp` are not documented as available until those frontends exist. A user with no editor integration installs the CLI alone.

### 2.2 Lifecycle

Users do not run `grund-lsp` directly. The editor's LSP client spawns it as a child process when a relevant file (markdown or any extension in the configured `[scan] extensions`) is opened in a workspace containing a `grund.toml` (in either discovery location) or `AGENTS.md`, and kills it when the workspace closes. The server speaks LSP over stdio; there is no daemon, no socket, no background service. CI pipelines that happen to have `grund-lsp` installed never invoke it — the only entry point in batch contexts is the CLI.

#### 2.2.1 Workspace folders anchor discovery

The folders in an LSP `initialize` request are config-discovery starts, not scan boundaries. For every `workspaceFolders` entry, the server walks upward with the same discovery rules as the CLI ([§FS-lsp.3](FS-lsp.md#3-configuration)); when it finds a Grund config, it scans that config's project root so configured `[scan] include` paths and sibling source trees remain visible even when the editor opened only a nested directory. If `workspaceFolders` is absent or empty, the deprecated `rootUri` is the discovery start, then the server process's current directory as the final fallback. With no discovered config under either name, the discovery start itself remains the zero-config scan root, with the canonical defaults ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)).

Entries that discover the same project root share one scan. Entries that discover different roots each get their own scan, and independent projects are never merged: identical local IDs in two editor folders are unrelated namespaces, and a reference answered from the wrong one would be a wrong citation ([§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess)).

#### 2.2.2 One project answers each document

Each document is answered by at most one of those projects. A project whose root contains the document claims it — the deepest such root when project trees nest, so a member opened as its own folder answers for its own files. A project that merely *scans* the document claims it when no root contains it: `[scan] include` is a scan scope, not a fence ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)), so a parent-relative include root may reach outside the project directory, and a document there is checked by the CLI and must stay answerable in the editor.

One owner per document is also what keeps diagnostics honest: where nested projects both read a file, the containing owner's verdict is published alone rather than merged with the other's, so a file cannot collect the same finding twice or two projects' disagreeing readings of one citation side by side.

#### 2.2.3 A document no project answers

A document reachable only below a directory symlink whose canonical target is outside the project root is pruned by [§FS-config.3.5.1](FS-config.md#351-a-symlink-in-the-tree-is-followed), so that project publishes no diagnostics for it and hover and navigation return no result. If multiple projects only reach the same external document through their scans, none owns it: choosing by folder order or root shape would guess between independent namespaces, so requests return no result and neither project's diagnostics are published for that file ([§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess)).

#### 2.2.4 An unusable folder is skipped

A folder the server cannot turn into a project is skipped, never fatal ([§REQ-never-crashes](../requirements/REQ-never-crashes.md#req-never-crashes-garbage-in-diagnostic-out)). A folder URI with a non-`file:` scheme — editors mix virtual and remote folders into one window — is passed over with a note on stderr. So is a folder whose config will not load: a half-typed `grund.toml` in one folder reports itself and leaves that project on its last good scan, while every other folder in the session keeps its diagnostics current. A session with no usable folder left still starts and answers nothing, rather than exiting.

#### 2.2.5 Folder changes

The server advertises workspace-folder support with change notifications. On `workspace/didChangeWorkspaceFolders`, added folders are discovered and included by the same rules, removed folders stop contributing, and diagnostics are republished from the resulting set of scans. Unusable entries are skipped as [§FS-lsp.2.2.4](FS-lsp.md#224-an-unusable-folder-is-skipped) says, so the rest of a mixed event still applies. Keeping a nested folder that still resolves to a project keeps that project active even when another folder for the same project is removed. Thus the initial folder order and later add/remove order cannot silently narrow references or diagnostics.

### 2.3 Editor configuration (one-time, per editor)

The user-facing LSP setup guide ships example LSP-client snippets for the editors most contributors use:

- **Helix** — three lines in `languages.toml`.
- **Neovim** — a built-in LSP snippet, compatible with `nvim-lspconfig`-based setups.
- **Zed** — central LSP registry entry; one config block locally if not yet upstreamed.
- **Emacs** — `eglot-server-programs` or `lsp-mode` registration (~5 lines).
- **VSCode** — install a generic LSP client extension and point it at `grund-lsp`. No first-party VSCode extension duplicates the LSP server or is published to a marketplace ([§FS-non-goals.12.2](FS-non-goals.md#122-first-party-per-editor-plugins)); the terminal-link extension the binary carries is not an LSP client ([§FS-integrations.3.2](FS-integrations.md#32-editor-clients-vscode-codium)).
- **Sublime Text** — LSP package client configuration for Markdown and scanned source syntaxes.
- **IntelliJ family** — generate the import directory with [§FS-lsp.2.4.1](FS-lsp.md#241-preview-and-write), then explicitly import it through LSP4IJ's **Settings | Languages & Frameworks | Language Servers**, **+ | New Language Server**, **Import from custom template...** flow.

Adding a new editor's snippet to the user-facing guide is a small contribution; it does not require a release.

### 2.4 Installed editor integrations

`grund-lsp integrations` is the batch surface for editor-client configuration carried by the installed `grund-lsp` version. It is distinct from `grund integrations`, which configures clickable-citation rendering clients ([§FS-integrations](FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations)). The initial catalog contains one entry, `lsp4ij`, with a one-line description. Listing prints the catalog on stdout, writes nothing, leaves stderr empty, and exits `0`.

#### 2.4.1 Preview and write

`grund-lsp integrations lsp4ij` is a read-only preview. It prints the complete generated `template.json` object followed by the exact explicit IntelliJ import steps from [§FS-lsp.2.3](FS-lsp.md#23-editor-configuration-one-time-per-editor) on stdout, writes no file or directory, leaves stderr empty, and exits `0`. `grund-lsp integrations lsp4ij --write <directory>` treats `<directory>` as the LSP4IJ import root and creates exactly `template.json` and `README.md`; success prints `created <directory>` and the import steps on stdout and exits `0`; the written README contains the same explicit import steps as preview. Repeating the write against those two byte-identical files is an idempotent success that prints `unchanged <directory>` and the same steps.

A pre-existing root with a missing, extra, or byte-different entry is a conflict: every existing byte remains untouched, stdout is empty, and one `error:` diagnostic on stderr tells the user to move or remove the root before retrying ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)). There is no force option.

#### 2.4.2 Language mappings

Generation starts at the process current working directory, uses the same upward configuration discovery as [§FS-lsp.3](FS-lsp.md#3-configuration), and reads the resulting effective `[scan].extensions` ([§FS-config.3.5](FS-config.md#35-scan--what-gets-walked)). With no config it therefore uses the canonical default extensions. Each extension becomes one `*.ext` file pattern. Conventional language IDs are used for known extensions — including `md` → `markdown`, `rs` → `rust`, `ts` → `typescript`, and every canonical default — while an unknown extension uses the extension itself as its non-empty language ID. Mappings are a deterministic snapshot of the effective config at generation time; a later config change requires regeneration.

#### 2.4.3 The launch command

The embedded LSP4IJ template contains both `programArgs.default` and `programArgs.windows`. Both change to `$PROJECT_DIR$` before launch and quote the absolute path returned for the running `grund-lsp` executable, the former for a POSIX shell and the latter for Windows `cmd`; the command selected on the generating host must be executable there. Spaces, non-ASCII characters, and JSON or command metacharacters in paths cannot change the command or corrupt the JSON. Neither command invokes Cargo or relies on shell `PATH` lookup.

#### 2.4.4 Help, failures, and the protocol boundary

The generator does not install LSP4IJ, edit JetBrains-owned files, or add editor-specific protocol behavior ([§FS-lsp.5](FS-lsp.md#5-out-of-scope)). Top-level help and `integrations` help explain this boundary, show list, preview, and write forms, and document exits `0` and `2`. `--version` retains its existing output. An unknown template, malformed arguments, invalid discovered config, render failure, path failure, or write conflict leaves stdout empty, prints one `error:` diagnostic on stderr, and exits `2`. Any argument is handled or rejected as batch input and never enters the protocol loop; exactly no arguments retains the stdio lifecycle of [§FS-lsp.2.2](FS-lsp.md#22-lifecycle) with protocol stdout pristine.

## 3. Configuration

The server reads the `grund.toml` via the same discovery logic as `grund check` ([§FS-config](FS-config.md#fs-config-grund-reads-a-toml-config-file-found-by-walking-up)), walking up from every workspace folder supplied by the editor's LSP `initialize` request, with the fallbacks of [§FS-lsp.2.2.1](FS-lsp.md#221-workspace-folders-anchor-discovery). Batch integration generation instead walks upward from the process current working directory ([§FS-lsp.2.4.2](FS-lsp.md#242-language-mappings)). Both consume the same effective core configuration; there is no separate LSP config or second parser.

Editor-side LSP configuration (server arguments, workspace folders) is the user's responsibility per [§FS-lsp.2.3](FS-lsp.md#23-editor-configuration-one-time-per-editor) and is not part of `grund.toml`.

## 4. Determinism and parity with the CLI

Same input + same config → same diagnostics, same hover body, same definition target, byte-for-byte ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)).

The LSP server does not have an "interactive" mode or a confirmation prompt ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)). It is the same engine with a different transport.

### 4.1 How parity is held

The implementation enforces this parity by routing LSP state through `grund-core` scan, check, show, refs, and formatting APIs, plus focused LSP tests for linkification, configured trigger handling, workspace member marker resolution, UTF-16 ranges, and document-link targets. The full child-process sweep over `tests/e2e/cases/*` ships as `tests/integration/lsp_cli_parity.rs`: for every plain-`check` case, the diagnostics the server publishes are the located findings the CLI prints, or the build is red. The three workspace warnings of [§FS-lsp.1.1.3](FS-lsp.md#113-workspace-warnings-on-the-runs-warning-channel) are held the same way against that case's stderr golden, so neither surface may carry one the other does not. The unlisted `[workspace]` block is not among them and is not held that way: it is a located finding on both surfaces now ([§FS-check.3.29.15](FS-check.md#32915-every-frontend-renders-it-anchored-at-the-blocks-workspace-line)), so the sweep compares it like any other — same path, line, `error` severity, code and message — which is what would catch the terminal and the editor drifting apart on the one finding whose location has two places to come from.

The sweep includes the plain-check rules fixture. Rule parsing and evaluation
remain core behavior: the server transports hard findings and `invalid-rule`
and contains no parallel rule implementation
([§FS-rules.9](FS-rules.md#9-managed-guidance-and-editor-parity)).

### 4.2 Embedded values

For an embedded value, this parity covers the CLI's marked-root shape and comparison diagnostics, raw `show --toc` hover slice, marker-free semantic title range, component definition target, and existing dotted-token references, highlights, and document links. Shell completion remains the core catalog's ordinary section completion and LSP completion remains reserved; neither surface adds a value-specific candidate.

### 4.3 Off-grammar declarations

This parity includes exact off-grammar declarations and their declaration-backed marked citations ([§FS-config.3.2](FS-config.md#32-id--id-grammar)): the LSP publishes the same located `declaration-near-miss` as `check`, while hover, definition, references, highlights, and document links navigate the declaration and its sections from the shared scan. No editor-only fallback recognition is permitted.

## 5. Out of scope

- **Per-editor wrappers**: no first-party VSCode/IntelliJ/Vim/Emacs plugin duplicates the LSP server or is published to a marketplace ([§FS-non-goals.12.2](FS-non-goals.md#122-first-party-per-editor-plugins)). The LSP server is the executable surface; [§FS-lsp.2.4](FS-lsp.md#24-installed-editor-integrations) ships importable configuration data but the user installs the generic client and performs the import.
- **Refactoring (rename ID)**: `grund` does not rename IDs; the scheme says IDs are forever ([§FS-non-goals.4](FS-non-goals.md#4-cross-workspace-id-renaming)).
- **Inline editing of declaration bodies from the hover popup**: editors already do this well; `grund-lsp` does not implement it.
- **Network access**: the server performs no network I/O ([§FS-non-goals.11](FS-non-goals.md#11-network-access-during-a-check)). All scanning is local.

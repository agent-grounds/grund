# FS-citations: what a citation is, and where its text is never rewritten

What a citation is, which tokens a scan records as one, and which spans of text no writer rewrites, defined once for every command that reads or writes a citation: `check` judges these citations ([§FS-check](FS-check.md#fs-check-grund-validates-every-citation-in-a-repo)) and `fmt` rewrites them ([§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk)), and neither restates what this page says. Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration), which a token recognized here is held to, and [§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful), which is why a citation is recognized in prose, comments and doc-comments alike.

## terms: Terms

Leans on [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, qualified citation, shorthand, canonical form,
citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (doc-comment), and [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root).
The suppressed scope is [§FS-fmt.terms](FS-fmt.md#terms-terms)'s.

- **recognized citation** — A token the scan records as a citation edge. It is what `refs` lists,
  what counts as a use and grounds a unit, and what every check judges.
- **never-rewrite zone** — A span whose bytes every pass preserves exactly, because a rewrite there
  would change what the file means. Which spans they are is [§FS-citations.zones](FS-citations.md#zones-where-citation-text-is-never-rewritten).
- **wrap** — A citation written as a Markdown link whose text is the citation: the form
  `grund fmt --cross-refs` writes and keeps current.

## requirements: What every reading of a citation holds to

Two requirements on how a token becomes a citation, each carrying a mark of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). Three more are left out because they are stated elsewhere: *every marked citation in the text a run reads is recognized, and every context that skips one is named* is [§REQ-no-missed-citation](../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked), with its list of blind spots at [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded); *`refs`, `cover`, every check and the editor read one set of citations* is [§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer); and *an error never names a rewrite the text forbids* is [§FS-check.requirements.no-declined-fix](FS-check.md#requirementsno-declined-fix-an-error-never-names-a-fix-grund-declines-to-make--realized-directional-for-the-next-finding), which [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite) serves.

### requirements.whole-token: A token cites what it spells whole, or nothing — realized, one case deferred

Each recognition rule reads its whole candidate before deciding what it is, so no token is recorded as a citation of a prefix of what its author wrote: a number-only shorthand claims a token only where the token ends ([§FS-citations.shorthand.full-id-wins](FS-citations.md#shorthandfull-id-wins-the-full-id-always-wins)), an unmarked letter-bearing tail never falls back to a bare ID and a mixed path such as `1.goals` is never cut back to its numeric prefix ([§FS-citations.recognized.named-candidates](FS-citations.md#recognizednamed-candidates-named-section-candidates)), a glob operator never leaves a citation of the text before it ([§FS-citations.recognized.globs](FS-citations.md#recognizedglobs-a-glob-operator-never-shortens-a-marked-citation)), and a declaration-local candidate with a partial tail is kept whole for its unsupported-syntax verdict ([§FS-citations.recognized.local-sections](FS-citations.md#recognizedlocal-sections-declaration-local-numeric-section-candidates)). What is not a citation whole is a pattern, unsupported syntax or prose, and earns no edge to, and no finding about, an ID or section its author did not write. One case is **deferred**, owed to its own change: a canonical citation whose section component runs on into word characters, `<§>FS-042-user-login.1x`, is recorded as a citation of section `1`, with named sections on or off, while the shorthand `<§>FS-042.1x` is no citation at all, so the two forms disagree about the one boundary [§FS-citations.shorthand.full-id-wins](FS-citations.md#shorthandfull-id-wins-the-full-id-always-wins) says they share. Serves [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) and [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms).

### requirements.edge-or-finding: A context removes a token's edge or withholds one finding, and says which — realized, directional for the next context

Every context that changes how a token is read does one of two things, to every consumer at once. An **exclusion** removes the edge: the token is no citation for `refs`, `cover`, unused-declaration counting, grounding, citation directions or chapter rules ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code), [§FS-citations.recognized.link-destinations](FS-citations.md#recognizedlink-destinations-markdown-link-destinations), [§FS-citations.recognized.escapes](FS-citations.md#recognizedescapes-an-id-in-an-escape-position), [§FS-citations.recognized.globs](FS-citations.md#recognizedglobs-a-glob-operator-never-shortens-a-marked-citation)). An **exemption** keeps the edge whole and withholds one finding, the mechanical demand of the shorthand error ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite)). A token the scan does record — off-grammar, behind a word-character marker, or a shorthand — is one edge every consumer reads ([§FS-citations.recognized.off-grammar](FS-citations.md#recognizedoff-grammar-off-grammar-citations), [§FS-citations.recognized.marker-start](FS-citations.md#recognizedmarker-start-the-configured-marker-establishes-the-citation-start), [§FS-citations.shorthand.edge](FS-citations.md#shorthandedge-a-resolved-shorthand-is-a-real-edge)). A context added from here on says which of the two it is, as these do, and reaches every consumer accordingly; one that would keep an edge from some consumers and not others is neither, and needs its own decision. Serves [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), whose blind spots are bounded only because each is total, and [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms).

## recognized: Recognized citations

Per [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger), a citation is the marker followed by an ID, e.g. `§FS-check.3.1`. The default marker is `§`; configurable via `grund.toml`.

In default mode (`[reference] strict = true`), only marker-prefixed citations are recognized — bare tokens are treated as plain text and do not trigger dangling citation errors. Repositories that still rely on bare citations may set `[reference] strict = false` as a compatibility mode after checking the migration surface with `grund fmt --marker` ([§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk)).

Citations may appear in markdown prose, in source-file line/block comments, and in language doc-comments (Javadoc, JSDoc, Rustdoc, Python docstrings, etc.) — see [AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection) and [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments) for the exact contexts. `E2E` citations (`§E2E-<name>`) resolve against case directories under the configured `E2E` kind home per [AR-scanner.6](../architecture/AR-scanner.md#6-e2e-case-declarations).

### recognized.off-grammar: Off-grammar citations

An off-grammar citation that a catalog declaration backs ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) participates in section and dangling checks, inbound counts, grounding, citation directions, `refs`, formatting, and editor navigation exactly like a conforming citation. The declaration's mismatch is reported at its heading ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)), not at every citation.

### recognized.named-candidates: Named-section candidates

When `[id] named_sections = true`, the scanner consumes a whole ID-and-dot-tail candidate before deciding what it means ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). A marker-prefixed legal named or mixed coordinate is a citation, including when the named section is missing. An unmarked candidate with a letter-bearing tail is one prose token and is suppressed whole even under `strict = false`; it never falls back to a bare-ID citation. A marked mixed candidate ([§FS-config.3.3.1](FS-config.md#331-named-components)), `<§>FS-x.1.goals` or `<§>FS-x.1.goals.latency`, is resolved whole like any other named coordinate and is never truncated to its numeric prefix; until 0.19.0 a finding it alone raises is a warning ([§FS-citations.checks.mixed-path](FS-citations.md#checksmixed-path-a-citation-of-a-name-under-a-number-is-checked-and-warns-until-0190)). Inside its own declaration the local form `<§>2.goals` stays one unsupported token ([§DF-declaration-local-section-shorthand.2.3](../decisions/functional/DF-declaration-local-section-shorthand.md#23-whole-tokens-receive-one-verdict)). Full IDs claim candidates before number-only shorthand ([§FS-citations.shorthand.full-id-wins](FS-citations.md#shorthandfull-id-wins-the-full-id-always-wins)), and shorthand form and section existence stay independent facts ([§FS-citations.shorthand.edge](FS-citations.md#shorthandedge-a-resolved-shorthand-is-a-real-edge)), exactly as for numeric coordinates.

### recognized.string-literals: String literals in source files

In source files, a **bare** ID-shaped token whose start column falls inside a string literal is not treated as a citation (the same deterministic quote-tracking rule `grund fmt` uses — [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule), [AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)), so an ID-shaped substring inside runtime data does not raise a false dangling citation. A marker-prefixed **unqualified** citation is recognized everywhere, string or not, except inside the assigned Python data span bounded by [§FS-citations.recognized.string-literals.assigned-data](FS-citations.md#recognizedstring-literalsassigned-data-assigned-python-triple-quoted-data) — the marker remains the signal of intent in every other string context. Markdown files have no string literals and neither source-file carve-out applies there. Decided in [§DF-python-assigned-triple-quoted-data](../decisions/functional/DF-python-assigned-triple-quoted-data.md#df-python-assigned-triple-quoted-data-module-level-assigned-triple-quoted-strings-are-data-not-docstrings).

#### recognized.string-literals.assigned-data: Assigned Python triple-quoted data

In a `.py` file scanned with `[scan] docstring_python = true`, a simple unindented assignment whose right-hand side begins with a triple-quoted string opens an **assigned-data span**, not a docstring. Its left-hand side is one Python identifier, optionally followed by a same-line annotation and then `=`; whitespace may surround the annotation separator and `=`. The string may use `'''` or `"""` and may carry, case-insensitively, no prefix or one of `r`, `u`, `b`, `f`, `t`, `br`/`rb`, `fr`/`rf`, or `tr`/`rt`. The prefix and delimiter must be the first non-whitespace text after `=`. Parenthesized assignments, destructuring targets, computed targets, indented assignments, and a string reached only after another expression are outside this rule: this is a bounded lexical distinction, not Python scope or AST analysis ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)).

The span begins at the prefix when present, otherwise at the opening delimiter, and ends immediately after the next matching delimiter that is not escaped by an odd-length run of backslashes. It may open and close on one line or cross lines; quote-like text using the other delimiter and an escaped matching delimiter do not close it. No citation form, declaration, section heading, value tag or binding, or note is recognized inside the span. Source columns remain columns in the raw file. Text before the span retains ordinary source treatment, text after a same-line close resumes ordinary source treatment at its raw column, and the next line after a multiline close begins in neutral state. A later delimiter at the first non-whitespace column therefore opens the same real module, function, or method docstring it would have opened had the assignment not existed.

With `docstring_python = false`, no assigned-data state is introduced and every line retains the existing raw-line source-string behavior. A leading triple-quote with no qualifying assignment remains the existing docstring form when the gate is enabled. Ordinary one-line strings, other Python triple-quoted contexts, other source languages, Markdown, strict-mode bare-token handling, and qualified-citation suppression retain their existing rules.

### recognized.link-destinations: Markdown link destinations

In a Markdown file, the parallel carve-out is the link destination: a **bare** ID-shaped token whose start column falls inside an inline link's `(…)` half — `[text](…)` — is likewise not treated as a citation, because `grund fmt` never rewrites a link destination ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)) and a finding whose only named fix the tool refuses to perform is one a repository can never clear ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). The exclusion is total, not just a withheld error: such a token is not a citation for `refs`, for unused-declaration counting ([§FS-check.4.1](FS-check.md#41-unused-declaration)), or for grounding ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) either. A marker-prefixed citation inside a link destination is unaffected — the marker is the signal of intent there too, exactly as it is inside a source-file string literal.

### recognized.contexts: Contexts read as neither prose nor code

Two contexts are read as neither prose nor code, so nothing inside them is a citation. A **fenced code block** in Markdown is skipped entirely: this is what makes an example ID safe to write in documentation without the `<§>` escape, and it is why the illustrations throughout these specs resolve to nothing. What opens and closes a fence is [§FS-languages.fence](FS-languages.md#fence-fences). In **source files only**, the qualified form `§<alias>/<ID>` is additionally skipped inside an inline-code span or a string literal, because `alias/ID` is shaped like a path, module reference, or URL ([AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)); the unqualified form stays live there, and neither skip applies in Markdown. Every other skip is a property of the *scan* rather than of the text — a file the scan skips, for any of the reasons [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) lists, is never read at all ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)).

A CommonMark raw-text HTML block ([§FS-check.1.1.5.1](FS-check.md#1151-a-raw-html-block-holds-no-heading)) is read as raw text for structure but as prose for citations. Citations inside the block stay live, unlike a fence's: the diagrams of [§FS-fmt.2.5.4](FS-fmt.md#254-the-case-this-exists-for) are written in `<pre>` precisely because a citation there still resolves, still counts as a use, and is still reported when it dangles.

### recognized.value-bindings: Value bindings

An exact explicit value binding additionally records its authored component, but its marker-prefixed token remains one citation under every rule above. The binding grammar and its narrower recognized source-comment contexts are [§FS-values.3](FS-values.md#3-explicit-value-bindings); `[reference] strict = false` never removes the binding's marker requirement.

### recognized.snapshots: Fetched snapshots

A snapshot written by [§FS-fetch](FS-fetch.md#fs-fetch-grund-materializes-one-external-fact-snapshot) is not a special input. Its configured file or folder home is in the ordinary scan scope ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)); its declaration, body, sections, and marked body citations are recognized by the rules above. `check` never invokes its configured integration.

### recognized.local-sections: Declaration-local numeric section candidates

A configured marker followed immediately by one or more decimal components separated by literal
dots is a declaration-local section candidate. Recognition is marker-gated under both strict and
non-strict scanning; an unmarked number remains prose. The path separator here is always `.`,
independent of the configured separator between a full ID and its section. Full-ID citations and
the number-only ID shorthand of [§FS-citations.shorthand](FS-citations.md#shorthand-the-number-only-shorthand) claim their tokens first. Escapes and every scanner
exclusion in [§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code) retain their precedence.

The candidate must end as a whole token. If digits are followed by a tail that would otherwise
make the numeric prefix partial, such as `<§>2.goals`, `<§>2abc`, or a path with an empty
component such as `<§>2..1`, the scanner retains the complete digit-starting token for the
unsupported-syntax verdict in [§FS-check.3.24](FS-check.md#324-declaration-local-section-citation); it never emits an edge to
section `2`. One dot after an otherwise complete token is sentence punctuation and is not part of
the candidate; a repeated terminal dot run is retained as unsupported syntax rather than reduced
to that punctuation case. Named and mixed declaration-local shorthand are not recognized.

After declaration bodies are assigned, the candidate is owned only by the existing enclosing-body
rule ([AR-scanner.2.4](../architecture/AR-scanner.md#24-citing-side-classification)): Markdown
same-or-higher headings, source comment or docstring ends, and the nearest preceding declaration
inside a multi-declaration comment are boundaries. Adjacent citations and other files do not
supply an owner. A uniquely owned candidate becomes an ordinary citation edge to the owner's ID
and numeric path while preserving its local token text. An ownerless or genuinely ambiguous site
records no target. [§DF-declaration-local-section-shorthand](../decisions/functional/DF-declaration-local-section-shorthand.md#df-declaration-local-section-shorthand-local-numeric-section-citations-are-recognized-but-never-canonical)
settles why recognition is loud but the persisted form is never canonical.

### recognized.escapes: An ID in an escape position

A **bare** ID-shaped token that begins immediately after the literal escape — the configured marker wrapped in `<` and `>` — is not treated as a citation, in Markdown and in source files alike, and under both strict modes. [§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves) already says that the escaped form is inert because the marker is not immediately followed by the ID, so *no pass* treats it as a citation; the bare-token pass is a pass, and it is the escape rather than the mode that suppresses it. The exclusion is **total**, not just a withheld error: such a token is not a citation for `refs`, for unused-declaration counting ([§FS-check.4.1](FS-check.md#41-unused-declaration)), or for grounding ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) either — the same reach as [§FS-citations.recognized.link-destinations](FS-citations.md#recognizedlink-destinations-markdown-link-destinations) and for the same reason, because `grund fmt` never rewrites an escape position ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)) and a finding whose only named fix the tool refuses to perform is one a repository can never clear. Both escape forms are covered by this one rule, so the qualified `<§>alias/ID` is exempt here and not only by [§FS-workspace.1.3](FS-workspace.md#13-the-shape-outside-a-workspace)'s unmarked-`alias/ID` rule. The escape is spelled with the **configured** marker: where the marker is `@`, `<@>ID` is the escape and a `<§>`-wrapped token is an ordinary bare one. The `escaped-citation-resolves` suggestion of [§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves) is unaffected, in both modes, and remains the only report of such a site. Recorded in [§DF-escape-position-is-not-a-citation](../decisions/functional/DF-escape-position-is-not-a-citation.md#df-escape-position-is-not-a-citation-an-escape-position-is-not-a-citation-in-either-strict-mode).

### recognized.marker-start: The configured marker establishes the citation start

The configured marker immediately followed by a full ID establishes a citation start even
when the marker ends in a word character, such as `_`. No word boundary is required between
that marker and the ID. With `[reference] marker = "_"` and `[id] format = "{kind}_{slug}"`,
`_FS_login` is exactly one marked citation of `FS_login`, under either strict mode. Its record
retains the authored token, source line, and column at the marker; `refs`, `cover`, inbound
counts, grounding, and dangling checking consume that same edge. A missing target therefore
receives the ordinary located dangling finding rather than disappearing.

Only the explicit marker establishes this start: under `strict = false`, an unmarked ID still
requires its ordinary token boundary and cannot be discovered inside a longer word. Default
markers, qualified citations, section suffixes, and full-ID precedence retain their existing
meaning. Escapes and every context exclusion above apply before recognizing a marked token;
the marker does not make a fenced example or assigned Python data into a citation.

The compatibility impact is recorded in
[§DF-word-character-citation-markers](../decisions/functional/DF-word-character-citation-markers.md#df-word-character-citation-markers-recognize-accepted-word-character-markers).

### recognized.globs: A glob operator never shortens a marked citation

A citation names one exact point, so a glob operator — `*`, `?`, a `[` class, or a `{` alternation — never ends a marked token early to leave a citation of its prefix. The marked **candidate** runs from the marker over every address character, section separator, and operator that follows; a `[` is an operator only when an address character or `!` follows it, a `{` only when an address character or `,` follows it, and `[^…]` is a Markdown footnote reference, never a class. The candidate is then read in this order:

1. **Catalog first.** A candidate that is a declared ID, with or without a section path, is that citation whatever characters it holds: under `{kind}*-{slug}`, `<§>FS*-login` cites `FS*-login`, and under a slug pattern admitting `*`, `<§>FS-tail*` cites `FS-tail*` ([§FS-declarations.line.configured-slug](FS-declarations.md#lineconfigured-slug-characters-admitted-by-the-slug-pattern-belong-to-the-canonical-id)).
2. **Grammatical but undeclared.** A candidate the effective address grammar ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) accepts whole is a citation with the ordinary verdicts: an undeclared `<§>FS*-logout` dangles ([§FS-check.3.1](FS-check.md#31-dangling-citation)), and a missing section is [§FS-check.3.2](FS-check.md#32-missing-section).
3. **A pattern.** Any other candidate holding an operator that opens a component (`<§>FS-login.*`, `<§>FS-login.requirements.**`, `<§>FS-*.requirements`, `<§>FS-[a,b]`) or sits inside one (`<§>FS-log*in.requirements`) is consumed whole and is a pattern, not a citation. No edge to its prefix or to anything else is recorded, so `refs`, `cover`, inbound counts, grounding, citation directions, and chapter rules never see it; `grund fmt` never rewrites it; and its escaped form earns no `escaped-citation-resolves` suggestion ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)). It is reported as [§FS-check.checks.glob-citation](FS-check.md#checksglob-citation-glob-citation).

**Punctuation stays punctuation.** A `*` or `?` run that directly follows a complete ID or section path, and is followed by no address character, operator, or section separator leading to one, is not part of the candidate. Terminal emphasis `*<§>FS-login*` and `**<§>FS-login**.`, a sentence-final `<§>FS-login?`, a footnote `<§>FS-login[^1]`, and a trailing `<§>FS-c*` stay ordinary citations of the ID before them, exactly as before.

Under `strict = false` an **unmarked** candidate holding an operator by the same reading is one prose token, suppressed whole like the letter-bearing tail of [§FS-citations.recognized.named-candidates](FS-citations.md#recognizednamed-candidates-named-section-candidates): no citation and no finding. Escapes and the context exclusions of [§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code) apply first. No configured ID grammar is restricted: an operator character a format or slug pattern admits is read by rules 1 and 2. Removing the prefix edge is the correction recorded in [§DF-glob-citation](../decisions/functional/DF-glob-citation.md#df-glob-citation-a-glob-operator-ends-no-citation-and-the-prefix-it-left-behind-is-removed).

### recognized.title-lines: A declaration's heading cites what it names

A declaration's heading line is read for citations like any other line, with one exception: a citation of the ID that heading declares is not a citation, because a declaration does not use itself. In `# <§>FS-a: Alpha, built on <§>FS-b`, the marked `FS-b` is a citation from `FS-a`'s declaration: `refs` lists it at the heading's line, it counts against [§FS-check.4.1](FS-check.md#41-unused-declaration), it dangles when nothing declares it ([§FS-check.3.1](FS-check.md#31-dangling-citation)), and the citation directions judge it from the declaration it sits on. The same holds for a section heading inside a declaration, which was always read this way. `grund fmt` still never rewrites a declaration's heading line ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)), and a title citation is still not an index entry. Counting it is the correction recorded in [§DF-a-title-citation-is-a-citation](../decisions/functional/DF-a-title-citation-is-a-citation.md#df-a-title-citation-is-a-citation-a-citation-on-a-declarations-heading-is-a-citation-unless-it-cites-the-id-that-heading-declares).

### recognized.no-kinds: With no kind, no token is a citation

A citation is the marker followed by an ID, and an ID opens with a configured kind ([§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates)), as a token whose prefix no kind configures is text today. A project whose config declares no kinds ([§FS-config.3.4.4.5](FS-config.md#3445-kinds---declares-no-kinds)) therefore recognizes no citation at all: `check` reports none of its marked tokens, dangling or otherwise, and its run ends on the caution that says the project declares no kinds ([§FS-check.4.5.2](FS-check.md#452-the-message)), at exit `0`, because a project that has declared no kind yet is an unfinished setup rather than a broken one.

## shorthand: The number-only shorthand

When a kind's effective format carries **both** `{number}` and `{slug}` ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) — the default `{kind}-{number}-{slug}` that `grund init` writes — the number alone already identifies a declaration within its kind, so `§FS-042` is an abbreviation of `§FS-042-user-login` rather than a different ID. `check` **recognizes** and resolves that shape independently of the project's persisted-form policy. Under the default `[reference] shorthand = "canonical"` it reports a unique shorthand to be rewritten; under `"accepted"` the same resolved edge may persist ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). It is never silently ignored, which is what [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) means by "false negatives are bugs".

Three rules bound the recognition ([§FS-citations.shorthand.marker](FS-citations.md#shorthandmarker-the-marker-is-required), [§FS-citations.shorthand.full-id-wins](FS-citations.md#shorthandfull-id-wins-the-full-id-always-wins), [§FS-citations.shorthand.edge](FS-citations.md#shorthandedge-a-resolved-shorthand-is-a-real-edge)), decided in [§DF-number-only-citation-shorthand](../decisions/functional/DF-number-only-citation-shorthand.md#df-number-only-citation-shorthand-the-number-only-shorthand-is-authoring-sugar-and-a-persisted-one-is-a-check-error).

### shorthand.shape: The shorthand shape

The shorthand shape is the kind's effective format with the `{slug}` placeholder and one adjacent literal separator removed. A kind whose effective format has no `{number}` (`{kind}-{slug}`, the form `grund` itself uses) or no `{slug}` (`{kind}-{number}`) has no shorthand and is untouched by this clause ([§FS-id.4.1](FS-id.md#41-number-less-id-formats)).

### shorthand.marker: The marker is required

A bare `FS-042` is plain text even under `strict = false`, where a bare *full* ID would count ([§FS-citations.recognized](FS-citations.md#recognized-recognized-citations)). `KIND-NNN` occurs constantly in the wild as issue keys, part numbers, and standards references, and unlike a full ID it carries no slug to make an accidental match unlikely — so the marker is what supplies the intent ([§DF-number-only-citation-shorthand.2.4](../decisions/functional/DF-number-only-citation-shorthand.md#24-the-marker-is-required-a-bare-shorthand-is-text)).

### shorthand.full-id-wins: The full ID always wins

The token must also end where the shorthand does. The full-ID pass claims its tokens first; the shorthand pass only sees what is left, and it claims a token only when the character after the match cannot continue an ID — an alphanumeric, `_`, or a literal from `format` that itself has a component after it. Without that trailing boundary the shorthand is a *prefix* of every longer ID-shaped token, so a full ID whose slug the grammar rejects (`§FS-042-User-Login`, `§FS-042_user_login`) would be read as `§FS-042` with a tail hanging off it. Such a token is not a citation at all: it is reported by nothing here and rewritten by nothing in [§FS-fmt.2.4](FS-fmt.md#24-shorthand-to-canonical). The separator has to be *followed* by a component to count, or a citation ending a sentence would be lost in any repo whose `format` uses `.` as a literal. And `/` never counts: it can only precede a kind, so `§FS-042/x` is a citation of `FS-042` exactly as `§FS-042-user-login/x` is one of the full ID — the shorthand and the canonical form must never disagree about the same boundary.

### shorthand.edge: A resolved shorthand is a real edge

When a shorthand matches exactly one declaration, the citation participates in the graph like any other: [§FS-refs](FS-refs.md#fs-refs-grund-lists-every-citation-of-an-id) lists it, [§FS-cover](FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file) groups it, the declaration stops being reported as unused ([§FS-check.4.1](FS-check.md#41-unused-declaration)), it grounds its file under `require_grounding` ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)), and it counts for citation directions ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)). The `.<section>` suffix works as it does on any citation, which means the section check ([§FS-check.3.2](FS-check.md#32-missing-section)) applies to it independently and on its own terms: a shorthand carrying a section that does not exist earns *both* findings, because the canonical form [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) names is the right ID and still the wrong section.

### shorthand.numeric-run: A shorthand in a numeric run

A recognized shorthand is not always *being used* as a citation: one glued to a second number — `§SPEC-001→SPEC-003` — is a numeral in a run, and while it resolves and counts like any other edge, `grund fmt` will not rewrite it and [§FS-check.3.15](FS-check.md#315-shorthand-citation-in-a-numeric-run) reports it instead of [§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation) ([§FS-fmt.2.4.1](FS-fmt.md#241-a-shorthand-in-a-numeric-run-is-not-rewritten)).

### shorthand.cli: The shorthand as a CLI ID argument

The same shape is accepted as a **CLI ID argument** — `grund FS-042`, `grund FS-042.1`, `grund refs FS-042` — where nothing is persisted and the caller gets the declaration ([§FS-show.1](FS-show.md#1-inputs), [§FS-refs.1](FS-refs.md#1-inputs)). That is also what makes a clicked `§FS-042` open in a terminal or editor, since those clients hand the token straight to `grund` ([§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2)).

## zones: Where citation text is never rewritten

Some spans hold text that is legitimate exactly as it stands, so no pass of `grund fmt` rewrites a citation there and each keeps its bytes ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)). These zones are forced by the text itself rather than asked for by the repository:

- Citations inside string literals on a source line, where rewriting would change runtime behavior ([§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule)).
- Citations inside a Markdown **fenced code block**, in every pass ([§FS-fmt.2.3.3](FS-fmt.md#233-a-fenced-code-block-is-an-illustration)).
- Citations inside Markdown inline code spans (where rewriting would change a literal command, path, or example), a span ending where [§FS-citations.zones.inline-code](FS-citations.md#zonesinline-code-an-inline-code-span-closes-on-a-run-of-its-own-length) says it does.
- ID-shaped text inside Markdown link destinations, except a destination `fmt` itself maintains ([§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url)).
- ID-shaped text in an escape position ([§FS-citations.recognized.escapes](FS-citations.md#recognizedescapes-an-id-in-an-escape-position)). `--marker` would splice a marker inside the escape brackets and turn an illustration into a live citation; every later pass, the cross-reference wrap included, follows the marker and stops with it.

Where `check` withholds the demand for a rewrite in some of these zones is [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite), which names its zones itself.

### zones.link-destination: A link destination is the URL

ID-shaped text inside a Markdown link destination is not rewritten, because rewriting it would change the URL rather than the visible citation. Off strict mode, such a **bare** token is not recognized as a citation there at all ([§FS-citations.recognized](FS-citations.md#recognized-recognized-citations)), the same never-rewrite zone keeping `check` from demanding an edit `fmt` refuses to make. The one exception is the destination of a cross-reference wrap `fmt` itself maintains, which every cross-reference pass re-derives ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)); a hand-authored destination keeps its bytes.

A destination runs from the `(` after an unescaped `](` to the `)` that balances it: each unescaped `(` inside it opens one more level and each unescaped `)` closes one, and a backslash-escaped parenthesis counts for neither. A destination whose `(` is never balanced on its line runs to the end of the line. Every pass of `fmt`, the cross-reference pass included, and every reading `check` makes of the zone take this one extent, so a citation after a `](` that never closes is inside a destination, and is left as written, wherever it stands on the rest of the line.

The destination `fmt` maintains is a **closed** wrap's: the bracket test of [§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets) passes and the extent above balances the wrap's `(`. After a `](` that never closes, every byte belongs to the author, and no pass rewrites or deletes one of them. Decided in [§DF-fmt-never-writes-into-a-link-destination](../decisions/functional/DF-fmt-never-writes-into-a-link-destination.md#df-fmt-never-writes-into-a-link-destination-fmt-never-writes-into-a-link-destination-it-does-not-maintain).

### zones.inline-code: An inline code span closes on a run of its own length

Where a Markdown inline code span begins and ends is read one line at a time, the way CommonMark reads it:

- it opens at a run of *n* backticks whose first backtick is not backslash-escaped;
- it closes at the next run of **exactly** *n* backticks, so a shorter or a longer run inside it is content;
- inside the span a backslash is literal: it escapes nothing and does not stop the closing run, so `` `C:\` `` is a closed span;
- an opener with no closing run on its line leaves the rest of that line code.

So on the second line of

```markdown
Plain [§GOAL-a](goals.md#goal-a-stale) here.
Escaped `` \ ` * `` then [§GOAL-a](goals.md#goal-a-stale) here.
```

the span opened by two backticks ends at the next two, the single backtick between them is content, and the citation after the span is prose: every cross-reference pass re-derives its anchor exactly as it does the first line's ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)). A citation written inside such a span, before or after the single backtick, is code and keeps its bytes.

The last rule departs from CommonMark on purpose. CommonMark reads an opener that never closes as literal backticks, but a code span may continue onto the next line of its paragraph, which hard-wrapped prose makes ordinary, and a reader of one line cannot tell that span from a stray backtick. Reading the rest of the line as prose would let `fmt` write into the first line of a wrapped code span, a guess persisted into the tree ([§REQ-no-wrong-citation.3](../requirements/REQ-no-wrong-citation.md#3-no-wrong-write)); reading it as code costs at most a rewrite not made, and `check` withholds its demand for that rewrite at the same site.

One reading serves every caller, because they have to agree. Every pass of `grund fmt` and the LSP on-type transform ([§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform)) stop at the span it finds, and so does every reading `check` makes of inline code: the canonical-form exemption ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite)), the illustration hint on a dangling citation ([§FS-check.3.1.2](FS-check.md#312-an-illustration-in-inline-code)), the kind-index entry form ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)), and the source-file skip of a qualified citation ([§FS-citations.recognized](FS-citations.md#recognized-recognized-citations)). So `check` never demands an edit at a site `fmt` reads as code, and `fmt` never leaves unmaintained a citation `check` reads as prose. Decided in [§DF-inline-code-span-closes-on-its-own-run](../decisions/functional/DF-inline-code-span-closes-on-its-own-run.md#df-inline-code-span-closes-on-its-own-run-an-inline-code-span-closes-on-a-run-of-its-own-length).

## rewrite-exemption: Where the text forbids the rewrite

A shorthand inside inline code ([§FS-citations.zones.inline-code](FS-citations.md#zonesinline-code-an-inline-code-span-closes-on-a-run-of-its-own-length)), a Markdown link destination ([§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url)), or a source string literal is exempt from the resolving form of the shorthand error ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)), because [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten) forbids the rewrite in those three contexts, where the text is legitimate as it stands, and an error whose only named fix the tool declines to perform there is one a repository can never clear. These three are the whole list: a zone added to what `fmt` never rewrites does not join it by being added there. A suppressed scope and an external file-symlink target are not among them: `fmt` will not rewrite there either, but the error still fires and the author clears it by hand ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes), [§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)). The exempt citation is untouched in every other respect — it resolves, `refs` lists it, and it keeps its declaration from being reported unused ([§FS-citations.shorthand.edge](FS-citations.md#shorthandedge-a-resolved-shorthand-is-a-real-edge)). The exemption is for the *mechanical* form only: a shorthand matching zero or several declarations is still reported in those contexts, because that is a dangling citation rather than a formatting nit.

## wrap: The wrap

A wrap is the form `grund fmt --cross-refs` writes ([§FS-fmt.6.2](FS-fmt.md#62-form)). Two questions about one are answered here once: what shape `check` asks of a kind's index entry, and how `fmt` tells a citation it already wrapped from one it has not.

### wrap.shape: The shape, never the target

`check` requires the **shape** — the citation wrapped as `[§<ID>…](<target>)` — and never the target. A wrap's URL is re-derived on every `grund fmt --cross-refs` pass ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)), so a heading rename that rots an anchor is a one-line `fmt` diff rather than a second finding here, and re-deriving it in `check` would put the anchor algorithm in a second command for no new coverage.

### wrap.recognized: An existing wrap is recognized by its brackets

A citation is already wrapped when its immediately-preceding character is `[` and its immediately-following text begins `](`. The one test decides both the re-derive of [§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive) and the no-double-wrap rule; a citation that fails it is wrapped fresh. One function holds the test, and `fmt`, `show`'s flatten ([§FS-show.3.2.1](FS-show.md#321-the-wrap-shape-it-collapses)) and `check`'s kind-index entry form ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)) each call it rather than writing it again.

Until grund 0.19.0, what `fmt --cross-refs` does with a wrap turns on whether its destination closes by the extent of [§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url), and where it ends is still read at the first `)`:

- **A wrap whose destination closes** is re-derived up to the first `)` after its `](`, as it has been since the pass shipped. Where parentheses inside the URL balance, as in `[§<ID>](old(1).md)`, the text after that first `)` stays.
- **A wrap whose destination never closes, with a later `)` on the line**, as in `[§<ID>](stale and an aside (keep me) end.`, is left as written: neither re-derived nor wrapped fresh. That `)` closes the author's own parenthesis rather than the wrap's, so the bytes before it are the author's ([§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url)), and `check` warns at the line ([§FS-citations.checks.unclosed-wrapper](FS-citations.md#checksunclosed-wrapper-unclosed-wrapper-read-differently-from-grund-0190)).
- **A `[§<ID>](` with no `)` after it on the line**, as in `[§<ID>](stale and more`, is wrapped fresh, as before.

From grund 0.19.0 a wrap is `[§<ID>](…)` closed by the `)` that balances its `(`, and only that: its whole destination is re-derived, so `[§<ID>](old(1).md)` keeps no `.md)`, and a `[§<ID>](` that never closes is no wrap and is wrapped fresh, the later `)` and the text before it kept. The route is [§DF-fmt-and-editor-answers-from-0-19](../decisions/functional/DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them).

## citing-kind: Which kind a citation site counts as

Every citation site has exactly one **citing kind**, the kind a `[citations.<kind>]` rule
names it by ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), and it is decided in this order:

1. **The enclosing declaration's kind.** The nearest preceding declaration whose body contains
   the site ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)) gives its kind. A site later in the file than that body
   does not inherit it and falls through.
2. **The kind of the file's home.** Where no declaration encloses the site, the kind whose
   `folder` or `file` contains the site's file gives its kind — but only when exactly one
   configured home contains it.
3. **The homeless kind.** Otherwise — outside every home, or in a file two overlapping homes
   contain — the site is the homeless kind's ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)).

The second step asks nothing about declarations, which is why the files of a non-citable
kind, which declare none, count as that kind because of where they are
([§FS-config.3.4.1.1](FS-config.md#3411-what-a-non-citable-kind-keeps)). An `E2E` case is outside the rule: its obligations are judged per case
over its `spec.refs` lines ([§FS-config.3.9.1.2](FS-config.md#3912-e2e-obligations-are-per-case)), not over citation sites.

## checks: Checks

The checks a citation's wrap is held to. Each is a section named by its diagnostic code ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)).

### checks.unclosed-wrapper: Unclosed wrapper read differently from grund 0.19.0

A citation `fmt --cross-refs` would link ([§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)) that has `[` immediately before it and `](` immediately after it, whose destination never closes by the extent of [§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url) but would close at a later `)` by the first-`)` reading of [§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets), is one **warning** at its line:

```text
docs/notes.md:3: warning: the link after FS-x never closes, so grund fmt --cross-refs leaves it as written, and from grund 0.19.0 it is not read as FS-x's link and FS-x is linked on its own — add the ')' that ends the link to keep it
```

for a `docs/notes.md` whose third line reads

```markdown
Then [§FS-x](stale-dest and an aside (keep me) end.
```

Those are exactly the lines whose `fmt --check` verdict moves in 0.19.0: today's `fmt` leaves them as written, and 0.19.0's wraps them fresh. A `[§<ID>](` with no `)` after it on the line gets nothing, because it is wrapped fresh in every release, and so does a wrap whose destination closes, whatever it holds. Adding the `)` that ends the link gives the same answer in both releases:

```markdown
Then [§FS-x](stale-dest) and an aside (keep me) end.
```

is a closed wrap whose destination `fmt` re-derives, and the aside keeps its bytes.

The warning is read off the citation's own line on a Markdown line the scan reads, in prose: not in a fenced code block, an inline code span, a suppressed scope, or a destination, and not on a declaration heading line. A kind's index entry is judged the same way, because its entries are always linked ([§FS-fmt.6.1.2](FS-fmt.md#612-the-carve-out-is-scoped-to-the-entries-not-the-page)). The scan records each site as it reads the line, and `check` reports what was recorded, so no command reads the file a second time for it.

- **Code:** `unclosed-wrapper`, with `"severity":"warning"` and `"sites":null` in JSON ([§FS-errors.5](FS-errors.md#5-json-format)). It leaves the exit status where the rest of the run put it ([§FS-check.2](FS-check.md#2-outputs)). `--only unclosed-wrapper` keeps it and `--ignore unclosed-wrapper` removes it ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)), and the editor shows it at the line with warning severity ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).
- **Release:** grund 0.19.0 reads a wrap as closed only by the `)` that balances its `(`, and with that reading removes every line this check reports. The route is [§DF-fmt-and-editor-answers-from-0-19](../decisions/functional/DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them).

### checks.mixed-path: A citation of a name under a number is checked, and warns until 0.19.0

Before 0.18.0 a marked citation of a mixed path ([§FS-config.3.3.1](FS-config.md#331-named-components)) was skipped without a word. From 0.18.0 it is resolved like any other citation, so one that resolves to nothing starts failing in a repository that passed. For one release window each such finding is carried on the warnings channel rather than the errors channel: the citation's `missing-section` ([§FS-check.3.2](FS-check.md#32-missing-section)) when its declaration exists and the section does not, and its `dangling` ([§FS-check.3.1](FS-check.md#31-dangling-citation)) when nothing declares its ID. The warning is the error's message unchanged, followed by `; a citation of a name under a number is now checked, and this warning becomes an error in grund 0.19.0`, the pending clause of [§FS-distribution.4.2.3](FS-distribution.md#423-the-vocabulary-is-closed)'s closed vocabulary:

```text
docs/use.md:3: warning: section not found: FS-x.1.nope; write <§> before it to show the shape without citing it; a citation of a name under a number is now checked, and this warning becomes an error in grund 0.19.0
docs/use.md:5: warning: unknown reference FS-nope; a citation of a name under a number is now checked, and this warning becomes an error in grund 0.19.0
```

The finding keeps the code it will carry as an error, so `--only` and `--ignore` select it now as they will then, and it leaves the exit status where the rest of the run put it. The catalog rows of these codes keep `error` ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), because only the findings this section names warn. The rule and `[citations]` findings that only such citations raise take the same window ([§FS-rules.7.12](FS-rules.md#712-a-finding-only-a-citation-of-a-name-under-a-number-raises-warns-until-0190)). A citation that resolves takes no window: it is counted at once by `refs`, `cover`, `unused` and the editor, and one without a lock entry is [§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold), immediately.

- **Release:** the clause names one constant release, 0.19.0, held ahead of the crate version by a unit test as every pending clause is. At 0.19.0 these findings are errors and this section becomes a pointer ([§RM-mixed-path-citation-error](../roadmap.md#rm-mixed-path-citation-error-make-a-citation-of-a-name-under-a-number-that-resolves-to-nothing-an-error-in-0190)). Decided in [§DF-number-name-sections.decision.citations](../decisions/functional/DF-number-name-sections.md#decisioncitations-a-citation-that-was-skipped-is-now-checked).

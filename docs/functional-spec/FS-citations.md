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
- **enclosing link** — A Markdown inline link its author wrote whose text holds a citation and
  more besides, `[the §<ID> record](…)`: the author's, never a wrap. Which links enclose is
  [§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author).

## requirements: What every reading of a citation holds to

Two requirements on how a token becomes a citation, each carrying a mark of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). Three more are left out because they are stated elsewhere: *every marked citation in the text a run reads is recognized, and every context that skips one is named* is [§REQ-no-missed-citation](../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked), with its list of blind spots at [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded); *`refs`, `cover`, every check and the editor read one set of citations* is [§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer); and *an error never names a rewrite the text forbids* is [§FS-check.requirements.no-declined-fix](FS-check.md#requirementsno-declined-fix-an-error-never-names-a-fix-grund-declines-to-make--realized-directional-for-the-next-finding), which [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite) serves.

### requirements.whole-token: A token cites what it spells whole, or nothing — realized, one case deferred

Each recognition rule reads its whole candidate before deciding what it is, so no token is recorded as a citation of a prefix of what its author wrote: a number-only shorthand claims a token only where the token ends ([§FS-citations.shorthand.full-id-wins](FS-citations.md#shorthandfull-id-wins-the-full-id-always-wins)), an unmarked letter-bearing tail never falls back to a bare ID and a mixed path such as `1.goals` is never cut back to its numeric prefix ([§FS-citations.recognized.named-candidates](FS-citations.md#recognizednamed-candidates-named-section-candidates)), a glob operator never leaves a citation of the text before it ([§FS-citations.recognized.globs](FS-citations.md#recognizedglobs-a-glob-operator-never-shortens-a-marked-citation)), and a declaration-local candidate with a partial tail is kept whole for its unsupported-syntax verdict ([§FS-citations.recognized.local-sections](FS-citations.md#recognizedlocal-sections-declaration-local-numeric-section-candidates)). What is not a citation whole is a pattern, unsupported syntax or prose, and earns no edge to, and no finding about, an ID or section its author did not write. One case is **deferred**, owed to its own change: a canonical citation whose section component runs on into word characters, `<§>FS-042-user-login.1x`, is recorded as a citation of section `1`, with named sections on or off, while the shorthand `<§>FS-042.1x` is no citation at all, so the two forms disagree about the one boundary [§FS-citations.shorthand.full-id-wins](FS-citations.md#shorthandfull-id-wins-the-full-id-always-wins) says they share. Serves [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) and [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms).

### requirements.edge-or-finding: A context removes a token's edge or withholds one finding, and says which — realized, directional for the next context

Every context that changes how a token is read does one of two things, to every consumer at once. An **exclusion** removes the edge: the token is no citation for `refs`, `cover`, unused-declaration counting, grounding, citation directions or chapter rules ([§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code), [§FS-citations.recognized.link-destinations](FS-citations.md#recognizedlink-destinations-markdown-link-destinations), [§FS-citations.recognized.escapes](FS-citations.md#recognizedescapes-an-id-in-an-escape-position), [§FS-citations.recognized.globs](FS-citations.md#recognizedglobs-a-glob-operator-never-shortens-a-marked-citation)). An **exemption** keeps the edge whole and withholds one finding, the mechanical demand of the shorthand error ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite)). A token the scan does record — off-grammar, behind a word-character marker, or a shorthand — is one edge every consumer reads ([§FS-citations.recognized.off-grammar](FS-citations.md#recognizedoff-grammar-off-grammar-citations), [§FS-citations.recognized.marker-start](FS-citations.md#recognizedmarker-start-the-configured-marker-establishes-the-citation-start), [§FS-citations.shorthand.edge](FS-citations.md#shorthandedge-a-resolved-shorthand-is-a-real-edge)). A context added from here on says which of the two it is, as these do, and reaches every consumer accordingly; one that would keep an edge from some consumers and not others is neither, and needs its own decision. Serves [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), whose blind spots are bounded only because each is total, and [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms).

## recognized: Recognized citations

Per [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger), a citation is the marker followed by an ID, e.g. `§FS-check.3.1`. The default marker is `§`; configurable via `grund.toml`.

In default mode (`[reference] strict = true`), only marker-prefixed citations are recognized — bare tokens are treated as plain text and do not trigger dangling citation errors. Repositories that still rely on bare citations may set `[reference] strict = false` as a compatibility mode after checking the migration surface with `grund fmt --marker` ([§FS-fmt](FS-fmt.md#fs-fmt-grund-normalizes-citations-in-bulk)).

Citations may appear in markdown prose, in source-file line/block comments, and in language doc-comments (Javadoc, JSDoc, Rustdoc, Python docstrings, etc.): which comment forms each language has is [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block), and the contexts read as neither prose nor code are [§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code). `E2E` citations (`§E2E-<name>`) resolve against the case directories of the configured `E2E` kind home, each of which declares the ID its name reads as ([§FS-declarations.e2e-case.id](FS-declarations.md#e2e-caseid-a-case-directory-and-the-id-it-declares)).

### recognized.off-grammar: Off-grammar citations

An off-grammar citation that a catalog declaration backs ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) participates in section and dangling checks, inbound counts, grounding, citation directions, `refs`, formatting, and editor navigation exactly like a conforming citation. The declaration's mismatch is reported at its heading ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)), not at every citation.

### recognized.named-candidates: Named-section candidates

When `[id] named_sections = true`, the scanner consumes a whole ID-and-dot-tail candidate before deciding what it means ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). A marker-prefixed legal named or mixed coordinate is a citation, including when the named section is missing. An unmarked candidate with a letter-bearing tail is one prose token and is suppressed whole even under `strict = false`; it never falls back to a bare-ID citation. A marked mixed candidate ([§FS-config.3.3.1](FS-config.md#331-named-components)), `<§>FS-x.1.goals` or `<§>FS-x.1.goals.latency`, is resolved whole like any other named coordinate and is never truncated to its numeric prefix; its findings use the ordinary reference verdicts ([§FS-citations.checks.mixed-path](FS-citations.md#checksmixed-path-a-citation-of-a-name-under-a-number-is-checked-and-warns-until-0190)). Inside its own declaration the local form `<§>2.goals` stays one unsupported token ([§DF-declaration-local-section-shorthand.2.3](../decisions/functional/DF-declaration-local-section-shorthand.md#23-whole-tokens-receive-one-verdict)). Full IDs claim candidates before number-only shorthand ([§FS-citations.shorthand.full-id-wins](FS-citations.md#shorthandfull-id-wins-the-full-id-always-wins)), and shorthand form and section existence stay independent facts ([§FS-citations.shorthand.edge](FS-citations.md#shorthandedge-a-resolved-shorthand-is-a-real-edge)), exactly as for numeric coordinates.

### recognized.string-literals: String literals in source files

In source files, a **bare** ID-shaped token whose start column falls inside a string literal is not treated as a citation (the same deterministic quote-tracking rule `grund fmt` uses — [§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule), [AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)), so an ID-shaped substring inside runtime data does not raise a false dangling citation. A marker-prefixed **unqualified** citation is recognized everywhere, string or not, except inside the assigned Python data span bounded by [§FS-citations.recognized.string-literals.assigned-data](FS-citations.md#recognizedstring-literalsassigned-data-assigned-python-triple-quoted-data) — the marker remains the signal of intent in every other string context. A string literal here includes every line a recognized multi-line form spans after its opener, up to its closer ([§FS-languages.multi-line-string](FS-languages.md#multi-line-string-a-string-literal-that-spans-lines)): on those lines both carve-outs apply exactly as they do inside a one-line literal, and the assigned-data span keeps its own rule. Markdown files have no string literals and neither source-file carve-out applies there. Decided in [§DF-python-assigned-triple-quoted-data](../decisions/functional/DF-python-assigned-triple-quoted-data.md#df-python-assigned-triple-quoted-data-module-level-assigned-triple-quoted-strings-are-data-not-docstrings).

#### recognized.string-literals.assigned-data: Assigned Python triple-quoted data

In a `.py` file scanned with `[scan] docstring_python = true`, a simple unindented assignment whose right-hand side begins with a triple-quoted string opens an **assigned-data span**, not a docstring. Its left-hand side is one Python identifier, optionally followed by a same-line annotation and then `=`; whitespace may surround the annotation separator and `=`. The string may use `'''` or `"""` and may carry, case-insensitively, no prefix or one of `r`, `u`, `b`, `f`, `t`, `br`/`rb`, `fr`/`rf`, or `tr`/`rt`. The prefix and delimiter must be the first non-whitespace text after `=`. Parenthesized assignments, destructuring targets, computed targets, indented assignments, and a string reached only after another expression are outside this rule: this is a bounded lexical distinction, not Python scope or AST analysis ([§FS-non-goals.3](FS-non-goals.md#3-code-ast-parsing)).

The span begins at the prefix when present, otherwise at the opening delimiter, and ends immediately after the next matching delimiter that is not escaped by an odd-length run of backslashes. It may open and close on one line or cross lines; quote-like text using the other delimiter and an escaped matching delimiter do not close it. No citation form, declaration, section heading, value tag or binding, or note is recognized inside the span. Source columns remain columns in the raw file. Text before the span retains ordinary source treatment, text after a same-line close resumes ordinary source treatment at its raw column, and the next line after a multiline close begins in neutral state. A later delimiter at the first non-whitespace column therefore opens the same real module, function, or method docstring it would have opened had the assignment not existed.

With `docstring_python = false`, no assigned-data state is introduced and every line retains the existing raw-line source-string behavior. A leading triple-quote with no qualifying assignment remains the existing docstring form when the gate is enabled. Ordinary one-line strings, other Python triple-quoted contexts, other source languages, Markdown, strict-mode bare-token handling, and qualified-citation suppression retain their existing rules.

### recognized.link-destinations: Markdown link destinations

In a Markdown file, the parallel carve-out is the link destination: a **bare** ID-shaped token whose start column falls inside an inline link's `(…)` half — `[text](…)` — is likewise not treated as a citation, because `grund fmt` never rewrites a link destination ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)) and a finding whose only named fix the tool refuses to perform is one a repository can never clear ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)). The exclusion is total, not just a withheld error: such a token is not a citation for `refs`, for unused-declaration counting ([§FS-check.4.1](FS-check.md#41-unused-declaration)), or for grounding ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)) either. A marker-prefixed citation inside a link destination is unaffected — the marker is the signal of intent there too, exactly as it is inside a source-file string literal.

### recognized.contexts: Contexts read as neither prose nor code

Two contexts are read as neither prose nor code, so nothing inside them is a citation. A **fenced code block** in Markdown is skipped entirely: this is what makes an example ID safe to write in documentation without the `<§>` escape, and it is why the illustrations throughout these specs resolve to nothing. What opens and closes a fence is [§FS-languages.fence](FS-languages.md#fence-fences). In **source files only**, the qualified form `§<alias>/<ID>` is additionally skipped inside an inline-code span or a string literal, a literal that spans lines included ([§FS-languages.multi-line-string](FS-languages.md#multi-line-string-a-string-literal-that-spans-lines)), because `alias/ID` is shaped like a path, module reference, or URL ([AR-scanner.2.3](../architecture/AR-scanner.md#23-citation-detection)); the unqualified form stays live there, and neither skip applies in Markdown. Every other skip is a property of the *scan* rather than of the text — a file the scan skips, for any of the reasons [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) lists, is never read at all ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)).

A CommonMark raw-text HTML block ([§FS-check.1.1.5.1](FS-check.md#1151-a-raw-html-block-holds-no-heading)) is read as raw text for structure but as prose for citations. Citations inside the block stay live, unlike a fence's: the diagrams of [§FS-fmt.2.5.4](FS-fmt.md#254-the-case-this-exists-for) are written in `<pre>` precisely because a citation there still resolves, still counts as a use, and is still reported when it dangles. A Markdown file's front matter
([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)) is read the same way: no line of it is a heading, and every citation in it is
live, belonging to the file's first declaration ([§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)).

In supported source comments, a prose apostrophe does not open a string literal, nor does
a Rust lifetime or Haskell identifier prime before a trailing comment. A qualified citation
in `// Don't drop <qualified>`, after `fn f() -> &'static str { "x" } //`, or after
`f' = 1 --` is recognized just like the same comment without the quote. This holds both
when the alias is configured and when it is unknown: the site contributes its ordinary
edge or finding, including `unknown project alias`, rather than disappearing. Genuine
quoted data and inline-code spans retain the source-only qualified exclusion above,
including the bounded carried multi-line literal spans.

This qualified-comment distinction does not change the raw quote-toggle policy of
[§FS-fmt.2.3.1](FS-fmt.md#231-string-literal-exclusion-rule), or its adoption for **bare**
tokens by [§FS-citations.recognized.string-literals](FS-citations.md#recognizedstring-literals-string-literals-in-source-files).
Those conservative rewrite and bare-token exclusions do not define whether a marked
qualified citation in a supported comment is string data. Decided in
[§DF-qualified-comment-quotes](../decisions/functional/DF-qualified-comment-quotes.md#df-qualified-comment-quotes-recognize-qualified-comment-citations-after-nonliteral-single-quotes).

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
rule ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends), as for [§FS-citations.citing-kind](FS-citations.md#citing-kind-which-kind-a-citation-site-counts-as)): Markdown
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

A citation names one exact point, so a glob operator — `*`, `?`, a `[` class, or a `{` alternation — never ends a marked token early to leave a citation of its prefix. The marked **candidate** runs from the marker over every address character, section separator, and operator that follows; a `[` is an operator only when an address character or `!` follows it, a `{` only when an address character, `,` or the section separator follows it, and `[^…]` is a Markdown footnote reference, never a class. So a brace group that crosses the separator, as a coordinate glob's may ([§FS-coordinate-globs.grammar.braces](FS-coordinate-globs.md#grammarbraces-brace-groups)), is read whole: `<§>FS-login{.1,.2}` is a pattern, never a citation of `FS-login` ([§DF-coordinate-globs.4](../decisions/functional/DF-coordinate-globs.md#4-the-citation-guards-verdict-correction)). The candidate is then read in this order:

1. **Catalog first.** A candidate that is a declared ID, with or without a section path, is that citation whatever characters it holds: under `{kind}*-{slug}`, `<§>FS*-login` cites `FS*-login`, and under a slug pattern admitting `*`, `<§>FS-tail*` cites `FS-tail*` ([§FS-declarations.line.configured-slug](FS-declarations.md#lineconfigured-slug-characters-admitted-by-the-slug-pattern-belong-to-the-canonical-id)).
2. **Grammatical but undeclared.** A candidate the effective address grammar ([§FS-config.3.2](FS-config.md#32-id--id-grammar)) accepts whole is a citation with the ordinary verdicts: an undeclared `<§>FS*-logout` dangles ([§FS-check.3.1](FS-check.md#31-dangling-citation)), and a missing section is [§FS-check.3.2](FS-check.md#32-missing-section).
3. **A pattern.** Any other candidate holding an operator that opens a component (`<§>FS-login.*`, `<§>FS-login.requirements.**`, `<§>FS-*.requirements`, `<§>FS-[a,b]`) or sits inside one (`<§>FS-log*in.requirements`) is consumed whole and is a pattern, not a citation. No edge to its prefix or to anything else is recorded, so `refs`, `cover`, inbound counts, grounding, citation directions, and chapter rules never see it; `grund fmt` never rewrites it; and its escaped form earns no `escaped-citation-resolves` suggestion ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)). It is reported as [§FS-check.checks.glob-citation](FS-check.md#checksglob-citation-glob-citation).

**Punctuation stays punctuation.** A `*` or `?` run that directly follows a complete ID or section path, and is followed by no address character, operator, or section separator leading to one, is not part of the candidate. Terminal emphasis `*<§>FS-login*` and `**<§>FS-login**.`, a sentence-final `<§>FS-login?`, a footnote `<§>FS-login[^1]`, and a trailing `<§>FS-c*` stay ordinary citations of the ID before them, exactly as before.

Under `strict = false` an **unmarked** candidate holding an operator by the same reading is one prose token, suppressed whole like the letter-bearing tail of [§FS-citations.recognized.named-candidates](FS-citations.md#recognizednamed-candidates-named-section-candidates): no citation and no finding. Escapes and the context exclusions of [§FS-citations.recognized.contexts](FS-citations.md#recognizedcontexts-contexts-read-as-neither-prose-nor-code) apply first. No configured ID grammar is restricted: an operator character a format or slug pattern admits is read by rules 1 and 2. Removing the prefix edge is the correction recorded in [§DF-glob-citation](../decisions/functional/DF-glob-citation.md#df-glob-citation-a-glob-operator-ends-no-citation-and-the-prefix-it-left-behind-is-removed).

### recognized.title-lines: A declaration's heading cites what it names

A declaration's heading line is read for citations like any other line, with one exception: a citation of the ID that heading declares is not a citation, because a declaration does not use itself. In `# <§>FS-a: Alpha, built on <§>FS-b`, the marked `FS-b` is a citation from `FS-a`'s declaration: `refs` lists it at the heading's line, it counts against [§FS-check.4.1](FS-check.md#41-unused-declaration), it dangles when nothing declares it ([§FS-check.3.1](FS-check.md#31-dangling-citation)), and the citation directions judge it from the declaration it sits on. The same holds for a section heading inside a declaration, which was always read this way. So does the heading of a declaration [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss) flags, which declares all the same ([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)): the ID it declares is its token as written, and only a citation of that token is dropped ([§DF-a-near-miss-title-citation-is-a-citation](../decisions/functional/DF-a-near-miss-title-citation-is-a-citation.md#df-a-near-miss-title-citation-is-a-citation-a-citation-on-a-near-miss-declarations-heading-is-a-citation-as-on-any-declarations)). `grund fmt` still never rewrites a declaration's heading line ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)), and a title citation is still not an index entry. Counting it is the correction recorded in [§DF-a-title-citation-is-a-citation](../decisions/functional/DF-a-title-citation-is-a-citation.md#df-a-title-citation-is-a-citation-a-citation-on-a-declarations-heading-is-a-citation-unless-it-cites-the-id-that-heading-declares).

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
- ID-shaped text inside Markdown link destinations, in each of their four spellings, except a destination `fmt` itself maintains ([§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url)).
- Citations inside a Markdown file's **front matter**, in every pass, because a Markdown link is not a valid YAML or TOML value ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)).
- A reference key, and the bytes of a raw HTML tag outside an `href` value ([§FS-citations.zones.authored-markup](FS-citations.md#zonesauthored-markup-a-reference-key-and-an-html-tag-keep-their-bytes)).
- ID-shaped text in an escape position ([§FS-citations.recognized.escapes](FS-citations.md#recognizedescapes-an-id-in-an-escape-position)). `--marker` would splice a marker inside the escape brackets and turn an illustration into a live citation; every later pass, the cross-reference wrap included, follows the marker and stops with it.

Where `check` withholds the demand for a rewrite in some of these zones is [§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite), which names its zones itself.

### zones.link-destination: A link destination is the URL

ID-shaped text inside a Markdown link destination is not rewritten, because rewriting it would change the URL rather than the visible citation. A destination is spelled four ways, and each is the URL in the same sense:

- the `(…)` of an inline link or an image, whose extent is the next paragraph's;
- the destination of a reference definition, `[label]: <destination> "title"`, from the first character after the `]:` and its spaces to the end of the definition's line, its optional title with it, because every reference that uses the definition takes the title along;
- an autolink, `<scheme:…>` or `<user@host>`, from its `<` to its `>`, both kept;
- the value of an `href` attribute of a raw HTML tag, quoted or unquoted, quotes kept.

No pass rewrites a byte in any of them, a trigger, `--marker` and a shorthand expansion no more than the cross-reference wrap. Off strict mode, a **bare** token in an inline destination is not recognized as a citation there at all ([§FS-citations.recognized](FS-citations.md#recognized-recognized-citations)), the same never-rewrite zone keeping `check` from demanding an edit `fmt` refuses to make. In the other three spellings a bare token is recognized as it always was, and only its bytes are protected: no grund release has skipped it there, so skipping it now would take an edge out of every repository that cites through a definition, an autolink or an `href`, and would stop a dangling one from failing, verdict moves in both directions that nothing licenses ([§REQ-backwards-compatibility.5](../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)); `check` withholds the mechanical demand there instead ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite)). The one exception is the destination of a cross-reference wrap `fmt` itself maintains, which every cross-reference pass re-derives ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)); a hand-authored destination keeps its bytes.

A destination runs from the `(` after an unescaped `](` to the `)` that balances it: each unescaped `(` inside it opens one more level and each unescaped `)` closes one, and a backslash-escaped parenthesis counts for neither. A destination whose `(` is never balanced on its line runs to the end of the line. Every pass of `fmt`, the cross-reference pass included, and every reading `check` makes of the zone take this one extent, so a citation after a `](` that never closes is inside a destination, and is left as written, wherever it stands on the rest of the line.

The destination `fmt` maintains is a **closed** wrap's: the bracket test of [§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets) passes and the extent above balances the wrap's `(`. After a `](` that never closes, every byte belongs to the author, and no pass rewrites or deletes one of them. Decided in [§DF-fmt-never-writes-into-a-link-destination](../decisions/functional/DF-fmt-never-writes-into-a-link-destination.md#df-fmt-never-writes-into-a-link-destination-fmt-never-writes-into-a-link-destination-it-does-not-maintain).

### zones.authored-markup: A reference key and an HTML tag keep their bytes

Two spans are the author's markup rather than prose, and no pass of `grund fmt`, nor the LSP on-type transform ([§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform)), rewrites a byte in them: a trigger, `--marker`, a shorthand expansion and the cross-reference wrap all stop at them.

- **A reference key**: the `[key]` of a full reference `[text][key]`; the text of a collapsed reference `[text][]`, which is its key; the text of a shortcut reference `[text]` that a definition matches, which is its key too; and the label of a definition line, `[label]: …`.
- **A raw HTML tag**, from its `<` to its `>`, outside an `href` value, which is a link destination ([§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url)). An element's content, between `<a …>` and `</a>`, is text a reader sees, not a tag.

A reference finds its definition by its key, so a collapsed or shortcut reference's text and its definition's label are one key spelled twice. A pass that rewrote one without rewriting the other identically would unlink the reference: `[the §<ID> record][]` above `[the §<ID> record]: adr/x.md` stops being a link when only one of them gains a wrap, or a marker, or an expanded shorthand. So the rule is not that the two must change together but that **both keep every byte**, in every pass, which no pass can split. The visible text of a full reference, `[text]` before `[key]`, is not a key, and a token there is rewritten as a token in an inline link's text is ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)).

A citation in either span is still a citation: it is recognized, resolved and checked as it is anywhere else ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)), and only the mechanical demand is withheld there ([§FS-citations.rewrite-exemption](FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite)). Which brackets are a key, and which definitions a file holds, is the document reading of [§FS-citations.wrap.enclosing.shape](FS-citations.md#wrapenclosingshape-which-links-enclose).

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

A shorthand inside inline code ([§FS-citations.zones.inline-code](FS-citations.md#zonesinline-code-an-inline-code-span-closes-on-a-run-of-its-own-length)), a Markdown link destination in any of its spellings ([§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url)), a reference key or an HTML tag ([§FS-citations.zones.authored-markup](FS-citations.md#zonesauthored-markup-a-reference-key-and-an-html-tag-keep-their-bytes)), or a source string literal is exempt from the resolving form of the shorthand error ([§FS-check.3.13](FS-check.md#313-number-only-shorthand-citation)), because [§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten) forbids the rewrite in those four contexts, where the text is legitimate as it stands, and an error whose only named fix the tool declines to perform there is one a repository can never clear. These four are the whole list: a zone added to what `fmt` never rewrites does not join it by being added there. A suppressed scope and an external file-symlink target are not among them: `fmt` will not rewrite there either, but the error still fires and the author clears it by hand ([§FS-fmt.2.5](FS-fmt.md#25-suppressed-scopes), [§FS-fmt.2.3.2](FS-fmt.md#232-a-link-that-leaves-the-config-root-is-not-written-through)). The exempt citation is untouched in every other respect — it resolves, `refs` lists it, and it keeps its declaration from being reported unused ([§FS-citations.shorthand.edge](FS-citations.md#shorthandedge-a-resolved-shorthand-is-a-real-edge)). The exemption is for the *mechanical* form only: a shorthand matching zero or several declarations is still reported in those contexts, because that is a dangling citation rather than a formatting nit.

## wrap: The wrap

A wrap is the form `grund fmt --cross-refs` writes ([§FS-fmt.6.2](FS-fmt.md#62-form)). Three questions about one are answered here once: what shape `check` asks of a kind's index entry, how `fmt` tells a citation it already wrapped from one it has not, and how both tell a wrap from a citation inside a link its author wrote.

So every citation in a Markdown file is read one of three ways: a **wrap** ([§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets)), a citation inside an **enclosing link** ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)), or **unwrapped**. The reading is of the document rather than of one line, because a reference finds its definition anywhere in the file and a link's text may run onto later lines ([§FS-citations.wrap.enclosing.shape](FS-citations.md#wrapenclosingshape-which-links-enclose)); a file with no citation near a `[`, a `<` or a definition line has nothing for it to find. One reading serves every caller: `fmt`'s link pass, `check`'s kind-index entry form and its external enrollment, `show`'s flatten, and the editor's hover linkification ([§FS-lsp.1.2.1](FS-lsp.md#121-citation-preview)) each ask it rather than writing their own, so none of them can take a link the others leave to its author ([§REQ-one-answer.contract](../requirements/REQ-one-answer.md#contract-one-question-one-tree-one-answer)).

### wrap.shape: The shape, never the target

`check` requires the **shape** — the citation wrapped as `[§<ID>…](<target>)` — and never the target. A wrap's URL is re-derived on every `grund fmt --cross-refs` pass ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)), so a heading rename that rots an anchor is a one-line `fmt` diff rather than a second finding here, and re-deriving it in `check` would put the anchor algorithm in a second command for no new coverage.

The shape is a link a reader can follow from the index to the declaration, and two shapes are one: the wrap, and a citation inside an enclosing link whose destination is not empty, such as `- [§<ID>: Use Postgres for all data](use-postgres.md)` ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)). Either is an ordinary entry, and neither has its target checked. What `fmt` owns is narrower than what `check` credits: only the wrap is `fmt`'s to write and re-derive, and only the canonical wrap enrolls an external source declaration ([§FS-check.3.18.3](FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link)).

### wrap.recognized: An existing wrap is recognized by its brackets

A citation is already wrapped when its immediately-preceding character is `[` and its immediately-following text is `](…)` closed by the `)` that balances its `(`, and that `[` does not follow an unescaped `!`. A `[` after an unescaped `!` opens an image's description, and an image is never a wrap ([§FS-citations.wrap.enclosing.images](FS-citations.md#wrapenclosingimages-an-image-is-not-a-link)). The one test decides both the re-derive of [§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive) and the no-double-wrap rule; a citation that fails it is wrapped fresh, unless it stands inside an enclosing link, which leaves it as written ([§FS-citations.wrap.enclosing](FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)). One function holds the test, beside the enclosing-link reading, and `fmt`, `show`'s flatten ([§FS-show.3.2.1](FS-show.md#321-the-wrap-shape-it-collapses)) and `check`'s kind-index entry form ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)) each call it rather than writing it again.

So a wrap's whole destination is re-derived: `[§<ID>](old(1).md)` keeps no `.md)`. And a `[§<ID>](` whose destination never closes is no wrap, whether or not a later `)` stands on its line: `[§<ID>](stale and an aside (keep me) end.` is wrapped fresh, the `](`, the later `)` and the text before it kept, exactly as `[§<ID>](stale and more` is.

Both answers are grund 0.19.0's, which ended the first-`)` reading; decided in [§DF-fmt-and-editor-answers-from-0-19](../decisions/functional/DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them).

### wrap.enclosing: A citation inside a link its author wrote is left to the author

A citation stands inside an **enclosing link** when it is in the text of a Markdown link its author wrote, in any form [§FS-citations.wrap.enclosing.scope](FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in) lists, and is not itself a wrap: `[the §<ID> record](adr/use-postgres.md)`, the index row `- [§<ID>: Use Postgres for all data](use-postgres.md)`, or `See [the §<ID> rationale][pg].` An inline link whose text is the citation alone is the wrap ([§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets)); every other form encloses whatever its text holds, so `[§<ID>][pg]` encloses too. Its author wrote that link, its words and its destination, so it is theirs and not a wrap `fmt` keeps:

- **`fmt` leaves the link as written.** The link pass neither wraps the citation nor re-derives the destination, in every run that reaches it: the default pass and the forced `--cross-refs`, the dry run and `--write`, and the kind-index carve-out under `enabled = false` or in a suppressed scope, which this outranks ([§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)). A wrap written there would be a link inside a link, which CommonMark does not allow: the inner one wins and the rest of the author's link renders as text. Only the link is protected, never the citation in it: outside a reference key and an HTML tag ([§FS-citations.zones.authored-markup](FS-citations.md#zonesauthored-markup-a-reference-key-and-an-html-tag-keep-their-bytes)), a trigger, `--marker` and a shorthand expansion rewrite its token as they would anywhere ([§FS-fmt.2](FS-fmt.md#2-behavior)), and the citation is recognized, resolved and checked like any other, so a dangling citation or a missing section in the text gets its usual finding ([§REQ-no-missed-citation](../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked)).
- **`check` takes it as a kind-index entry where the link goes somewhere**, as it takes a wrap ([§FS-citations.wrap.shape](FS-citations.md#wrapshape-the-shape-never-the-target)): an inline link whose destination is not empty, and a full, collapsed or shortcut reference whose key a definition with a destination that is not empty matches. It never compares that destination with the one `fmt` would derive. Each citation in the text counts for its own ID, so `[§<A> and §<B>](both.md)` is an entry for both. Every other enclosing form is protected and is no entry, because protecting a citation does not make it a link from the index to the declaration: an inline link with an empty destination, `[the §<ID> record]()`, goes nowhere, and so does a reference no definition matches; a definition's label is a key a reader never sees; an autolink holds its citation in its URL, not its text; a raw HTML `<a>` is not the Markdown link an entry is ([§DF-index-entry-form.2.1](../decisions/functional/DF-index-entry-form.md#21-the-entry-is-a-recognized-citation-of-the-id-written-as-a-full-markdown-link)); and an incomplete label never closed ([§FS-citations.wrap.enclosing.shape](FS-citations.md#wrapenclosingshape-which-links-enclose)). The ID then falls to [§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index). No enclosing form enrolls an external source declaration, which only the canonical wrap does ([§FS-check.3.18.3](FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link)).
- **`show` prints it as written**, because only a wrap collapses ([§FS-show.3.2.1](FS-show.md#321-the-wrap-shape-it-collapses)).

Decided for the same-line inline link in [§DF-fmt-preserves-authored-link-labels](../decisions/functional/DF-fmt-preserves-authored-link-labels.md#df-fmt-preserves-authored-link-labels-fmt-leaves-a-citation-in-a-link-its-author-wrote-and-check-takes-that-link-as-the-index-entry), and for every other form and the image in [§DF-fmt-preserves-authored-markup](../decisions/functional/DF-fmt-preserves-authored-markup.md#df-fmt-preserves-authored-markup-fmt-leaves-a-citation-in-every-link-form-its-author-wrote-and-in-an-image-and-check-credits-only-the-ones-that-go-somewhere).

#### wrap.enclosing.shape: Which links enclose

An enclosing link is read with the extents every other reading of the file uses, across the lines of its paragraph:

- its text opens at a `[` that is not backslash-escaped, not inside an inline code span ([§FS-citations.zones.inline-code](FS-citations.md#zonesinline-code-an-inline-code-span-closes-on-a-run-of-its-own-length)), and not an image's ([§FS-citations.wrap.enclosing.images](FS-citations.md#wrapenclosingimages-an-image-is-not-a-link));
- it closes at the `]` that balances that `[`: an unescaped `[` inside opens one more level and an unescaped `]` closes one, so `[the [draft] §<ID> record](…)` encloses, and a bracket that is escaped or inside an inline code span counts for neither. The search runs to the end of the `[`'s paragraph, which a blank line ends, and so does a line that opens a fenced code block, an ATX heading, a list item, a block quote or a thematic break. A block quote is read the same way behind its `>` markers: a quoted line continues the paragraph of its quote unless what follows its markers is blank or opens one of those blocks; a line with fewer markers than the paragraph began with, or none, continues it lazily on the same terms; and a line with more opens a quote, and a paragraph, of its own. A `[` that no `]` balances before then opens no link, and a citation after it is unwrapped, unless it stands in an incomplete label (below);
- what follows the `]` at once decides the form. A `(` makes an inline link, whose destination runs to the `)` that balances it on that line, a backslash-escaped parenthesis counting for neither, by the one extent of [§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url); a destination that never closes on its line makes no link, so a citation in the text is unwrapped and is wrapped fresh, as a `[§<ID>](` that never closes is ([§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets)). A `[` makes a full or a collapsed reference, whose key runs to the `]` that balances it on that line. A `:` after a `[` that opens its line makes a definition where three things hold: the `[` stands behind at most three spaces, or behind the container prefixes of that line, any block-quote `>` markers and list markers with their indent; the line starts its paragraph, or every line of the paragraph before it is a definition, because a definition never interrupts a paragraph; and after the `:` the line holds a destination, `<…>` or a run with no whitespace whose parentheses balance, and at most a title after it, set off by whitespace, in `"…"`, `'…'` or `(…)`, and closed on that line. So `> [label]: url` and `- [label]: url` that start their paragraph are definitions, and a collapsed reference in the same container finds its key there. A line that fails any of the three is not a definition, and what follows its `]` is read by the rest of this list: `[label]: url` under a line of text, in a quote or not, and `- [label]: §<ID> and more words` are prose, as they were before. A `[^` opens a footnote, not a label, so a footnote's text after its `]:` stays prose. Anything else is a shortcut reference where a definition matches the text, and plain brackets where none does.

A key is matched as CommonMark matches one: case folded, each run of whitespace read as one space, and leading and trailing whitespace dropped. The `>` markers of a quoted line are the quote's, not the key's, so text that runs across quoted lines, `> See [the §<ID>` above `> record][]`, spells the key that `> [the §<ID> record]: …` spells on one. A definition may stand before or after the references that use it, anywhere in the file outside a fenced code block, and where two define one key the first wins. Nothing is resolved outside the file and no destination is fetched ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

Only a citation in the text is enclosed. A citation in the destination is in a never-rewrite zone already, one in an inline code span is code, and one after the link is outside it. Surrounding words, punctuation, balanced brackets and other citations in the text are all allowed, and a file may hold the same citation inside an enclosing link and outside one: each occurrence is read by its own position, so the one outside is wrapped and the one inside is left.

**A link inside the text.** Brackets in the text balance like any others, a link's included, so `[a [b](c.md) §<ID> d](x.md)` encloses the citation and, in a kind's index, is the entry for it. CommonMark renders that row otherwise, because a link holds no link: `[b](c.md)` is the link and the outer brackets and `(x.md)` print as text. The reading is kept on purpose. `fmt` writes nothing into the line either way, and taking the credit away would turn a passing `check` into a failing one that no prohibition stands behind ([§REQ-backwards-compatibility.5](../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)).

**An incomplete label.** A backtick run in the text with no closing run on its line leaves the rest of that line code ([§FS-citations.zones.inline-code](FS-citations.md#zonesinline-code-an-inline-code-span-closes-on-a-run-of-its-own-length)), and so can hide the `]` that would close the text, as in ``[the §<ID> `record](use-tick.md)``. Where no `]` balances a `[` before its paragraph ends and such a run stands after the `[`, the text is an **incomplete label**, running from its `[` to its paragraph's end: the link pass leaves every citation in it as written, none of them is an entry, and an index that holds the ID only there is told to write one by hand ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)). The code span keeps its reading. CommonMark would read the lone backtick as literal and the link as whole, but writing that guess into the file is the wrong write the inline-code zone refuses, while leaving the label costs at most a wrap not made and an entry written by hand.

#### wrap.enclosing.wrap-wins: A wrap inside a longer text is a wrap

A citation whose own brackets pass the test of [§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets) is a wrap, whatever brackets stand around it. That decides the line an older `fmt` already nested, `- [[§<ID>](use-postgres.md#…): Use Postgres for all data](use-postgres.md)`, the way CommonMark reads it: the inner `[§<ID>](…)` is the link, a wrap `fmt` keeps re-deriving and `check` takes as the entry, and the outer brackets and `(use-postgres.md)` are text, not an enclosing link. Nothing is nested further and nothing is unwrapped, so such a line is a fixed point under `fmt --write` and `fmt --check`, and `show` collapses only the inner wrap. Taking the outer brackets back out is a hand edit, and the release note of [§DF-fmt-preserves-authored-link-labels](../decisions/functional/DF-fmt-preserves-authored-link-labels.md#df-fmt-preserves-authored-link-labels-fmt-leaves-a-citation-in-a-link-its-author-wrote-and-check-takes-that-link-as-the-index-entry) says how to find the lines.

#### wrap.enclosing.images: An image is not a link

An image, `![description](file)`, is neither a link nor a wrap: a `[` after an unescaped `!` opens a description, which never encloses and fails the wrap test ([§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets)). Its destination is the image's file, so it is the author's, and no pass rewrites a byte of it. Two images hold a citation, and each keeps a reading of its own:

- **A citation that is an image's whole description**, `![§<ID>](cache.png)`, is left as written by the link pass, which neither wraps the citation nor writes into the destination; the earlier passes still rewrite its token, as in a link's text. It is no link to the declaration, so it is no kind-index entry and enrolls no external source declaration. grund 0.19.0 still credited it with both for one release, and warned; grund 0.20.0 retired the credit ([§FS-citations.checks.image-index-entry](FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release)).
- **A citation in a description that holds more**, `![a diagram of §<ID>](diagram.png)`, is read as it always was: `fmt` wraps it as it wraps any unwrapped citation, writing `![a diagram of [§<ID>](…)](diagram.png)`, and `check` reads it as bare and never as an entry. An index row a `fmt` already wrapped that way, `- ![a diagram of [§<ID>](…)](d.png)`, keeps its credit through the inner wrap, which is a wrap by its own brackets ([§FS-citations.wrap.enclosing.wrap-wins](FS-citations.md#wrapenclosingwrap-wins-a-wrap-inside-a-longer-text-is-a-wrap)).

So the credit this point gives an enclosing link never reaches an image. Until this point the citation-only image passed the bracket test and was read as a wrap: `fmt` overwrote `cache.png` with the declaration's URL, and `check` took the image as an entry. Decided in [§DF-fmt-preserves-authored-markup](../decisions/functional/DF-fmt-preserves-authored-markup.md#df-fmt-preserves-authored-markup-fmt-leaves-a-citation-in-every-link-form-its-author-wrote-and-in-an-image-and-check-credits-only-the-ones-that-go-somewhere).

#### wrap.enclosing.scope: Every form a link is written in

A citation is enclosed in every form a Markdown link is written in, each read by the one reading of [§FS-citations.wrap.enclosing.shape](FS-citations.md#wrapenclosingshape-which-links-enclose):

- an inline link, `[text](destination)`, its text on one line or running onto later lines of its paragraph;
- a full reference, `[text][key]`, whether or not a definition matches its key;
- a collapsed reference, `[text][]`, whether or not a definition matches its text;
- a shortcut reference, `[text]`, only where a definition matches its text;
- a definition line's label, `[label]: destination`;
- an autolink, `<scheme:…>` or `<user@host>`, whose whole is a destination ([§FS-citations.zones.link-destination](FS-citations.md#zoneslink-destination-a-link-destination-is-the-url));
- the content of a raw HTML `<a>` element, its tag names matched without regard to case, from its opening tag to the `</a>` that closes it, or to its paragraph's end where none does. The opening tag ends at the first `>` outside a quoted attribute value, and an HTML comment, `<!-- … -->`, opens no element.

The link pass leaves each of them as written, and the editor's hover reads them the same way ([§FS-lsp.1.2.1](FS-lsp.md#121-citation-preview)). This is not a Markdown parser: the forms are recognized by the extents this point and the zones name, and malformed syntax neither names keeps the reading it had before.

## citing-kind: Which kind a citation site counts as

Every citation site has exactly one **citing kind**, the kind a `[citations.<kind>]` rule
names it by ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), and it is decided in this order:

1. **The enclosing declaration's kind.** The nearest preceding declaration whose body contains
   the site ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)) gives its kind; a site in front matter is enclosed by the file's first declaration
   ([§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)). A site later in the file than that body
   does not inherit it and falls through.
2. **The kind of the file's home.** Where no declaration encloses the site, the kind whose
   `folder` or `file` contains the site's file gives its kind — but only when exactly one
   configured home contains it.
3. **The homeless kind.** Otherwise — outside every home, or in a file two overlapping homes
   contain — the site is the homeless kind's ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)).

The second step asks nothing about declarations, which is why the files of a non-citable
kind, which declare none, count as that kind because of where they are
([§FS-config.3.4.1.1](FS-config.md#3411-what-a-non-citable-kind-keeps)). An `E2E` case's obligations are
judged per case, over its `spec.refs` lines and any citation sites the run scans inside it, and
a run over the project scans none there ([§FS-declarations.e2e-case.walk](FS-declarations.md#e2e-casewalk-the-walk-stops-at-a-case-directory),
[§FS-config.3.9.1.2](FS-config.md#3912-e2e-obligations-are-per-case)).

## checks: Checks

The checks a citation's wrap is held to. Each is a section named by its diagnostic code ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)).

### checks.image-index-entry: A citation-only image is an index entry for one more release

The check this heading names is spent. [§FS-citations.wrap.enclosing.images](FS-citations.md#wrapenclosingimages-an-image-is-not-a-link) made a citation that is an image's whole description, `![§<ID>](cache.png)`, no link to the declaration. grund 0.19.0, the first release that carried it, kept the image's credit in a kind's index for one release: the image was still taken as the entry for `<ID>` ([§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is)), and where its destination was the canonical link to an external source declaration it still enrolled it ([§FS-check.3.18.3](FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link)). Where that credit was all the index held for `<ID>`, `check` warned at the image's line with code `image-index-entry`, naming the link to write beside it.

grund 0.20.0 retired the credit ([§DF-image-index-entry-lands-in-0-20](../decisions/functional/DF-image-index-entry-lands-in-0-20.md#df-image-index-entry-lands-in-0-20-a-citation-only-image-is-no-index-entry-from-0200)), and `check` now reads the image as `fmt` always has:

- the image is no entry and enrolls nothing;
- an ID it was the only entry for falls to `missing-index-entry` ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)), an error at the declaration's heading. The index mentions the ID, so the finding says it **appears** in the index but not as an entry ([§FS-check.3.18.5](FS-check.md#3185-what-an-entry-is)), and never that it is not a link, because `fmt` leaves the image as written ([§FS-check.3.17.4](FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule));
- an external declaration it alone enrolled is no longer a member ([§FS-check.3.18.4](FS-check.md#3184-what-does-not-enroll)), and the image is an ordinary citation of it;
- an image beside another entry for the same ID changes nothing.

The repair is the one the warning named: write `[§<ID>](<link>)` beside the image, where `<link>` is the link `grund fmt --cross-refs` writes from the index to the declaration, the literal of [§FS-check.3.18.5.2](FS-check.md#31852-what-not-listed-says). The image may stay.

- **Code:** `image-index-entry` stays in the catalog of codes, so `--only image-index-entry` and `--ignore image-index-entry` are still accepted ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)) and select nothing; neither `check` nor the editor ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)) reports it. An old `--ignore image-index-entry` leaves the `missing-index-entry` error visible. An old `--only image-index-entry` selects no replacement error, keeps the selected report's exit, and says the check is spent with the off line of [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so), which names `missing-index-entry`, the replacement to select:

  ```text
  --only image-index-entry judged nothing — the check is spent from grund 0.20.0; select missing-index-entry instead
  ```
- **Release:** the route was [§DF-fmt-preserves-authored-markup](../decisions/functional/DF-fmt-preserves-authored-markup.md#df-fmt-preserves-authored-markup-fmt-leaves-a-citation-in-every-link-form-its-author-wrote-and-in-an-image-and-check-credits-only-the-ones-that-go-somewhere), and [§DF-image-index-entry-lands-in-0-20](../decisions/functional/DF-image-index-entry-lands-in-0-20.md#df-image-index-entry-lands-in-0-20-a-citation-only-image-is-no-index-entry-from-0200) carries the notice of the release that completed it. This address, heading text included, is kept so citations written while the credit lasted still resolve.

### checks.unclosed-wrapper: Unclosed wrapper read differently from grund 0.19.0

Retired in grund 0.19.0: a wrap is read as closed only by the `)` that balances its `(` ([§FS-citations.wrap.recognized](FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets)), so no line is read differently any more and the warning, its code and its catalog row are gone ([§DF-fmt-and-editor-answers-from-0-19](../decisions/functional/DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them)). An `--only unclosed-wrapper` or `--ignore unclosed-wrapper` left in a script then names an unknown code and is refused as any unknown code is ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)). This address is kept so citations written before the change still resolve.

### checks.mixed-path: A citation of a name under a number is checked, and warns until 0.19.0

The warning window shipped in 0.18.0 and closes in 0.19.0. With named
sections enabled, a marked mixed path ([§FS-config.3.3.1](FS-config.md#331-named-components)) is resolved whole like
any other citation. If its declaration exists but its section does not, it is
ordinary `missing-section` ([§FS-check.3.2](FS-check.md#32-missing-section)); if its ID is undeclared, it is
ordinary `dangling` ([§FS-check.3.1](FS-check.md#31-dangling-citation)). Both are errors with the ordinary messages,
codes and sites, with no mixed-path clause. Text and NDJSON use stdout with
empty stderr; NDJSON severity is `error`, and these errors make `check` exit 1
([§FS-check.2](FS-check.md#2-outputs)). Code selection remains [§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)'s.

A resolving mixed citation still counts for `refs`, `cover`, `unused` and the
editor, and a missing lock entry is immediately
[§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold). Rule and configured
citation verdicts are [§FS-rules.7.12](FS-rules.md#712-a-finding-only-a-citation-of-a-name-under-a-number-raises-warns-until-0190)'s. The completed transition is
[§DF-mixed-path-citation-errors.decision](../decisions/functional/DF-mixed-path-citation-errors.md#decision-decision); this heading and address remain so
citations written during the warning window keep resolving.

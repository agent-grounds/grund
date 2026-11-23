# FS-values: opted-in kinds bind authored components to one declared value

`grund` lets a repository make numbered components authoritative values, so that a value written in prose is checked against the one place it is declared. A kind opts whole declarations in, an author marks one citable numbered section, or a kind declares the chapter whose named children are values; a binding names one component or a whole value root beside a literal, and `grund check` compares the two in the current tree only ([§DA-explicit-value-bindings.2](../decisions/architectural/DA-explicit-value-bindings.md#2-decision)). Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work) and [§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms)
(source declaration, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, workspace, member, alias),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion, anchor), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction), and
[§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value, component, binding).

- **value root** — The declaration or section whose numbered children carry the authored values.
- **authority** — What makes a section a root — the `values = true` row, the embedded tag, or
  the declared chapter. A second authority over the same root is an error, not a confirmation.
- **value source** — A file at a whole-value kind's home that declares values as data rather
  than as Markdown: the `file` home itself, or a direct child of a `folder` home, whose extension
  is `.json`, `.yaml` or `.yml` and which the scan does not claim ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)). Its
  **format**, JSON or YAML, is what it is read as: `.json` and a version-2 `data` mapping are
  JSON, `.yaml` and `.yml` are YAML.

## requirements: What value authority holds to

Four requirements every command shares hold here as written: every surface reads one value ([§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)), no path reads two ways ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), answered in [§FS-values.5.1](FS-values.md#51-resolve-before-comparison)), the report's order and bytes are fixed ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)), and an upgrade never changes a verdict quietly ([§REQ-backwards-compatibility](../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly)).

### requirements.ordinary-coordinates: A value's coordinates are the ones its author wrote

Value authority adds checking, never identity. Every coordinate a value has is one its author wrote — an ordinary declaration, an existing numbered or named section, or a member key and element of a value source — and no authority synthesizes an ID, a section, a resolver or an identifier grammar beside it ([§FS-values.1](FS-values.md#1-per-kind-opt-in-and-identity), [§FS-values.2.4](FS-values.md#24-embedded-section-value-roots), [§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)). So every reader keeps its ordinary answer, with no root aggregate or row, no value-specific candidate, no invented title, Markdown or anchor, and no badge or substitution in an editor ([§FS-values.6.1](FS-values.md#61-marked-roots-in-the-catalog), [§FS-values.6.2](FS-values.md#62-value-source-declarations-in-the-catalog), [§FS-values.7](FS-values.md#7-workspaces-and-editor-consumers)). Serves [§GOAL-polyglot-citation](../goals.md#goal-polyglot-citation-ids-cite-cleanly-from-anywhere-they-are-useful).

### requirements.origin-blind: A root is used the same way whichever authority made it

Authority decides where a root comes from and nothing after that. A valid root is bound in one grammar ([§FS-values.3.1](FS-values.md#31-the-only-binding-grammar)), resolved in one order ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)), compared under one equality ([§FS-values.4](FS-values.md#4-exact-equality)), reported in one finding shape ([§FS-values.5.2](FS-values.md#52-fixed-value-errors)), refused for a spaced component in one wording ([§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)) and kept by the formatter alike ([§FS-values.8](FS-values.md#8-formatting-stability)), whether a `values = true` row, a tag or a declared chapter made it; and a marked and a chapter-declared root share one component run, reported at the same locations, and one catalog identity ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots), [§FS-values.6.1](FS-values.md#61-marked-roots-in-the-catalog)). An authority added to the three of [§FS-values.1](FS-values.md#1-per-kind-opt-in-and-identity) changes only where a root comes from, and meets every clause of this point in the same change rather than bringing a binding grammar, a resolution order or a finding of its own. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.agree-or-fail: A binding passes only by agreeing

A delimited form aimed at value authority never leaves `grund check` green unless its literal equals what is declared. It is compared ([§FS-values.4](FS-values.md#4-exact-equality)); or refused as an attempted binding ([§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts), [§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)); or suppressed behind a finding that is itself an error ([§FS-values.5.1](FS-values.md#51-resolve-before-comparison)). The three value findings are errors under every `strict` and `--suggestions` setting (their rows in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), and a near miss that is no attempt stays an ordinary citation, checked as one ([§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts)): this is [§REQ-no-missed-citation](../requirements/REQ-no-missed-citation.md#req-no-missed-citation-every-citation-the-run-reads-is-checked) read for bindings. A run narrowed to a path keeps only what is about that path ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)), so a binding held back there by an invalid root outside it passes, as its own site does in the whole run. One case is deferred: a root in a stub's unscanned target is read to answer the binding but not judged ([§FS-check.3.2.1](FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not), [§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)), so its invalidity suppresses the comparison and is reported nowhere, and the whole run passes. Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration).

## 1. Per-kind opt-in and identity

Value authority comes from three places and no other, each opted into explicitly, and none infers a value from ordinary prose:

- **A whole declaration** in the home of a `[[kinds]]` row that sets `values = true` ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)), or in version 2 a kind with `form = "value"` ([§FS-fields.forms](FS-fields.md#forms-form-lowers-to-the-version-1-contracts)). No whole declaration outside that home is authoritative.
- **A marked root**: one numbered section whose heading carries the tag of [§FS-values.2.4](FS-values.md#24-embedded-section-value-roots), in any scanned Markdown declaration or supported source doc-comment, whatever its kind's `values` setting and home. The tag adds no key, and its declaration and sibling sections gain nothing from it.
- **A chapter-declared root**: a named child of the chapter a row's `value_chapter` names ([§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values), [§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)).

None creates a kind or a second identifier grammar. A value declaration's ID is an ordinary full local ID of its kind, whose slug carries stable identity rather than the value: `CONST-field-price`, not `V-1200-USD`.

## 2. Value declarations

### 2.1 Markdown declarations

A whole-declaration Markdown value is an ordinary declaration in its kind's home whose immediate citable child headings are its components: one nonempty contiguous run `.1` through `.N`, each heading at the configured strict depth for its coordinate. A gap, a zero or leading-zero coordinate, a nested or named citable section, or a duplicate component makes the declaration invalid.

A component's text is the whole heading title after its coordinate, on that one physical line: nonempty, with no leading or trailing whitespace, no backtick and no control character. Lead prose, bodies below component headings and plain non-citable headings carry no value. A component is a number when its whole text matches JSON number grammar, and a string otherwise.

### 2.2 Declarations from value sources

A value source ([§FS-values.terms](FS-values.md#terms-terms)) declares values at its kind's own home, never in a separately configured place. A `file` home ending in `.json`, `.yaml` or `.yml` is its one source; a `folder` home's sources are its normalized direct children with those extensions, in one bytewise path order across both formats. A source's format picks its reader ([§FS-values.2.2.3](FS-values.md#223-how-a-yaml-source-is-read)), and nothing else about a declaration.

The scan's claim comes first, and only for YAML ([§DF-yaml-value-sources.2.2](../decisions/functional/DF-yaml-value-sources.md#22-the-scans-claim-wins)). In version 1, a `.yaml` or `.yml` file whose extension is in the effective `[scan] extensions`, the default list where the config writes none, is scan input: it keeps its citations and declares no value. A `.json` file is never claimed. In version 2 the file's language decides instead ([§FS-config-v2.schema.sources.values](FS-config-v2.md#schemasourcesvalues-a-value-home-reads-a-file-by-its-language)).

A value source is catalog input, not scan input: it is read once whatever `[scan]` include, exclude and ignore files, an explicit command path or `--full` say, and none of them hides one or finds another. It contributes no citations, so a `§` citation in a YAML source's comment neither resolves nor dangles, a declared blind spot ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded), [§DF-yaml-value-sources.5](../decisions/functional/DF-yaml-value-sources.md#5-what-it-does-not-cover)).

#### 2.2.1 Source shape

A source is one top-level JSON object or YAML mapping. Each member's key is a full, unaliased local ID of the owning kind, and its value a nonempty JSON array or YAML sequence whose element `i` is component `i + 1`. An element is a number or a string, a YAML scalar being one or the other as [§FS-values.2.2.3](FS-values.md#223-how-a-yaml-source-is-read) says, and a decoded string is held to the Markdown component's boundary: nonempty, with no edge whitespace, backtick or control character ([§FS-values.2.1](FS-values.md#21-markdown-declarations)). An empty or non-object root; a wrong-kind, invalid or qualified key; an empty array; and a null, boolean, object, nested-array, or non-finite or malformed number element make an invalid declaration, in either format.

An invalid member keeps no valid sibling from being declared. A member refused at its key or at its whole value declares nothing, so a citation of it is an unknown reference; a member of the wrong shape or with an invalid component keeps its ID, declared and invalid.

#### 2.2.2 Reader fidelity

Both readers keep member order, every repeated key ([§FS-values.2.3](FS-values.md#23-duplicates-and-ownership)), each number's and string's raw spelling, each string's decoded text, and the exact span of every key, member and element. The member slice a source gives an ID runs from the key's first byte to the last byte of its last element, a flow sequence's closing `]` included, and the element slice a component gives is that element's scalar, quotes included. In YAML, comments between the key and the last element are inside the member slice, and comments before the key or after the last element are not. A leading byte order mark belongs to no slice, and no slice ends in a carriage return.

#### 2.2.3 How a YAML source is read

A YAML source is one YAML stream holding one document, with an optional `---` start and `...` end, whose root is a mapping. Its keys are plain or quoted scalars, each an ID, and its values block or flow sequences of scalars. A block sequence may sit at its key's indent or deeper, and a flow sequence may span lines while each element stays on one line; a trailing comma adds no element. Comments may stand on any line.

A component's kind comes from how it is written, never from YAML's own type resolution:

- A **quoted** scalar is a string. A double-quoted one decodes JSON's escapes and only those, and in a single-quoted one `''` is one `'`.
- A **plain** scalar is a number exactly when its whole text matches JSON number grammar, as a Markdown component is ([§FS-values.2.1](FS-values.md#21-markdown-declarations)), and a string otherwise. So `1.10` is the number spelled `1.10`, and `0o17`, `1_200`, `yes` and `no` are strings.
- Plain `true`, `false`, `null` and `~`, and an empty element, are invalid components, as JSON's booleans and null are.

So that every component is the bytes at its own site ([§DF-yaml-value-sources.2.1](../decisions/functional/DF-yaml-value-sources.md#21-yaml-is-a-second-format-of-a-value-source)), a source refuses every construct that could make it otherwise, each an `invalid-value-declaration` at its own line with the reason `a YAML value source refuses <construct>`: `an anchor`, `an alias`, `a tag`, `a merge key` (`<<`), `a second document`, `a nested collection` (a mapping or sequence as an element), `a block scalar`, `a multi-line scalar`, and ``the escape `<escape>` `` for any escape in a double-quoted scalar but JSON's. An invalid component's reason is the JSON one with `YAML` for `JSON`.

Past its reading, a YAML source is a JSON one: each finding at it carries the code, severity and exit class it would carry at a JSON source ([§FS-values.5.3](FS-values.md#53-incomplete-input-and-deterministic-output)), so nothing about a finding depends on the format but the word for it ([§DF-yaml-value-window-closes-in-0-22](../decisions/functional/DF-yaml-value-window-closes-in-0-22.md#df-yaml-value-window-closes-in-0-22-the-yaml-value-source-warning-window-closes-in-0220)). A source of either format in a fetched kind's home is read and declares its IDs, so a citation of one resolves with no `missing-snapshot` or `dangling` answer.

### 2.3 Duplicates and ownership

A key repeated in one JSON object or YAML mapping, one ID declared across value sources of either format, a Markdown declaration and a value-source member of one ID, and one value source enrolled by two opted-in homes, through a link from one home into the other, are each an ambiguous duplicate declaration, even when the components agree; neither source nor kind takes precedence ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)). A duplicate holds no source back: every site is named and every declaration is in the catalog.

The home is what enrolls a value source, so two opted-in homes are two claims on one set of declarations: the duplicate names each spelling once, and each spelling is also judged at its own home, so the one under another kind's home is a misplaced declaration too ([§FS-declarations.checks.misplaced-declaration.2](FS-declarations.md#checksmisplaced-declaration2-another-kinds-home)). A value source with two names in one home, such as a file and a link to it there, has one owner and is no duplicate: it is read once, and its findings carry both names ([§FS-scan.identity.findings.content](FS-scan.md#identityfindingscontent-a-content-finding-is-one-line-naming-every-name)).

### 2.4 Embedded section value roots

An author makes one existing citable numbered section, at any numbered depth, a marked root by ending its heading content with one ASCII space and the byte-exact lowercase tag `<!-- grund:value -->`, followed only by optional trailing whitespace. A root keeps its ordinary coordinate and its components are its children: a marked root at `FS-pricing.2.3` owns `FS-pricing.2.3.1` through `.N`. A root or component written in the relative form ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)) is judged at its canonical path, so `### .1. 45` under `## 1. Regional floor <!-- grund:value -->` is the component `1.1`. A misspelled, recased, respaced, incomplete or otherwise lookalike tag is opaque prose and earns neither authority nor a value finding.

An exact tag where no marked root can stand grants no authority and is `invalid-value-declaration` at the tag: on a plain or named heading, on an empty or malformed numbered one, or on a heading outside a declaration; the same bytes in ordinary prose are inert. Several marked roots in one declaration are valid while neither root path is an ancestor of the other; nested or overlapping marks invalidate both roots and are reported at the inner tag, and a tag inside a whole-declaration Markdown value invalidates its marked root while the declaration's own authority stands. Section-depth findings own a malformed root heading and run before value comparison.

#### 2.4.1 Recognition in source doc-comments

For source declarations the scanner recognizes the tag after removing the same configured comment wrapper used for headings ([§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper)), and in a docstring reads its content as it stands ([§FS-languages.docstring](FS-languages.md#docstring-docstrings)). The language adds no alternate spelling.

#### 2.4.3 The component run

A valid marked root owns exactly one nonempty run of immediate numbered children in physical order, with relative coordinates `.1` through `.N` contiguous and in order. Each child is written exactly one heading depth below the root, with a component title as [§FS-values.2.1](FS-values.md#21-markdown-declarations) defines one; its section record and exact value span are the component. Blank lines and a source block comment's outer delimiters are harmless. Every other nonblank line within the root is `invalid-value-declaration` at that line: root lead prose, a component body, a named or plain child heading, a grandchild, and a child whose text is no component. A root with no components is reported at its tag, or, for a chapter root, at its heading. A gap or out-of-order coordinate is reported at each heading that differs from the next expected index. A duplicate component keeps [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) or the ordinary duplicate-section finding, and is also `invalid-value-declaration` at its duplicate heading.

### 2.5 Chapter-declared value roots

A `[[kinds]]` row's `value_chapter` ([§FS-config.3.4.13](FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values)), or in version 2 a field with `content = "values"` ([§FS-fields.content.values](FS-fields.md#contentvalues-a-values-field-is-the-value-chapter)), names one chapter whose named direct children are value roots, with no tag, in every declaration of that kind. The declared chapter is the declaration's **direct** named chapter whose section name is the configured one — `values` in `## values: Values`, never the displayed title, and never a same-named chapter nested deeper, so `subsystems.pump.values` is ordinary prose. A root keeps its ordinary coordinate: `AR-power.values.aux-voltage` owns `AR-power.values.aux-voltage.1` through `.N`. The chapter and its roots may be written in the relative form, as `## .values: Values` with `### .aux-voltage: Auxiliary supply voltage` under it. A named section outside every declared chapter stays ordinary prose. The kind's declarations are read in Markdown and in supported source doc-comments, after the same comment-wrapper removal as a tag ([§FS-values.2.4.1](FS-values.md#241-recognition-in-source-doc-comments)).

A chapter root's components are the run of [§FS-values.2.4.3](FS-values.md#243-the-component-run), reported at the same locations. The declared chapter holds only named direct children: chapter lead prose, a numbered child and a plain child heading are each `invalid-value-declaration` at that line, and a valid sibling root is unaffected. An exact tag inside a declared chapter is no second authority over the root the chapter made, and is `invalid-value-declaration` like any other unexpected content there. Chapter authority governs inside the declared chapter, and tag authority everywhere else.

## 3. Explicit value bindings

### 3.1 The only binding grammar

A binding is a nonempty single-backtick-delimited literal, one ASCII space, then a parenthesized marker-prefixed citation of a value root — a whole-value ID, a marked root ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots)) or a chapter-declared root ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)) — optionally followed by one positive numbered immediate-component coordinate, all on one physical line ([§DA-explicit-value-bindings.2](../decisions/architectural/DA-explicit-value-bindings.md#2-decision)). With the coordinate the binding is component-aimed and its literal is that one component; without it the binding is root-aimed and its literal is the root's whole component run, joined as [§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root) says:

```text
`1200` (§CONST-field-price.1)
`1200 USD` (§CONST-field-price)
```

The marker is required even when `[reference] strict = false`. The literal is never empty or multiline and has no edge whitespace, backtick or control character. A root path is a marked root's numbered path or a declared chapter's direct named child, never a path that puts a name under a number ([§FS-config.3.3.1](FS-config.md#331-named-components)), so `### 2.values: Values` and everything below it bind nothing.

#### 3.1.1 Invalid attempts and non-attempts

An attempt that breaks the grammar is `invalid-value-binding` at its literal, the line and the column of its opening backtick: a tab or a missing or extra space, absent parentheses, a missing marker, and a zero, leading-zero, named or otherwise non-numeric component coordinate. So is a delimited form aimed at a declared chapter's heading or below one of a root's components, whichever authority made the root, and a root-aimed form [§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root) refuses.

A binding split across two lines that are one run of text is an attempt too, and is never compared, because one physical line is the only grammar: a literal whose closing backtick is followed only by whitespace to the end of its line, then a next line whose first text after its continuation prefix is `(` immediately followed by a marker-prefixed citation aimed at value authority. One run is consecutive lines of one Markdown paragraph, the first not a heading, or of one comment block ([§FS-languages.comment-block.forms](FS-languages.md#comment-blockforms-comment-blocks)) whose literal sits in recognized comment content ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts)). The continuation prefix is the next line's indentation and, inside a blockquote, the same `>` markers the literal's line carries, or in a comment block the next line's comment wrapper and the whitespace around it ([§FS-values.2.4.1](FS-values.md#241-recognition-in-source-doc-comments)). The citation on the next line stays one ordinary citation, as a binding's does ([§FS-values.3.2](FS-values.md#32-recognized-text-contexts)), and the message, unlike every other attempt's, tells the author to join the two:

```text
docs/split.md:1: error: invalid value binding: value binding must be on one physical line; join the literal and its citation
```

Everything else stays ordinary prose and one ordinary citation, and is not compared: an unbackticked adjacent token; a bare value citation, one with no delimited literal before it on its line and none closing the line above onto it; two lines that are not one such run; and the same delimited shape, on one line or two, aimed at an ordinary unmarked section, a named section outside every declared chapter, or a whole declaration whose kind lacks `values = true`.

#### 3.1.2 A binding aimed at the root

A root-aimed literal is the root's components `.1` through `.N`, in declared order, joined by one ASCII space. `grund` splits the literal at every U+0020 and only there, and compares part `i` with component `i` under [§FS-values.4](FS-values.md#4-exact-equality), each part keeping its component's equality rule, so `` `1200.0 USD` `` agrees with a declared `[1200, "USD"]`. The split keeps every part, so two adjacent spaces leave an empty part, which no component equals ([§FS-values.2.1](FS-values.md#21-markdown-declarations)).

A value any of whose components contains an ASCII space may not be bound at its root, whatever its arity, because the join would no longer say how the literal splits ([§DA-root-aimed-value-bindings.3](../decisions/architectural/DA-root-aimed-value-bindings.md#3-why)). The root-aimed form over it is `invalid-value-binding` at the binding, naming the cited root as a mismatch renders it ([§FS-values.5.2](FS-values.md#52-fixed-value-errors)), the first such component in declared order, and that component's declaration site:

```text
docs/offer.md:3: error: invalid value binding: CONST-head-office cannot be bound at its root because component 1 contains an ASCII space at values/head-office.md:2
```

The refusal belongs to the root-aimed spelling, never to the value: the declaration stays valid, and binding that component by its coordinate compares it as usual. For a value of one component the two spellings give the same verdict wherever the root-aimed one is admitted, and only the verdict: each finding names what its own site wrote ([§DF-root-value-mismatch-wording.3](../decisions/functional/DF-root-value-mismatch-wording.md#3-why)).

### 3.2 Recognized text contexts

A binding is read in Markdown outside fenced code blocks, and in a scanned source file only when the whole form sits in one comment or doc-comment line the scanner recognizes, never in a host-language expression or string. Its citation stays one ordinary citation for resolution, `refs`, `cover`, citation directions, grounding and unused-declaration counting ([§FS-check.1.1](FS-check.md#11-recognized-citations)).

## 4. Exact equality

Each side of a comparison is a number or a string. A declared component takes its kind from its declaration: a Markdown component's from JSON number grammar over its text ([§FS-values.2.1](FS-values.md#21-markdown-declarations)), a JSON element's from its JSON type ([§FS-values.2.2.1](FS-values.md#221-source-shape)), and a YAML scalar's from how it is written ([§FS-values.2.2.3](FS-values.md#223-how-a-yaml-source-is-read)). A literal, or a part of a root-aimed one, is a number when its whole text matches JSON number grammar.

Two numbers are compared as arbitrary-precision decimals, never through binary floating point or exponent expansion, and with no size limit beyond input and memory, so `1200`, `1200.0` and `1.2e3` are equal and negative zero equals zero.

Two strings are equal when their decoded Unicode scalar sequences are exactly equal: case-sensitive, with no trimming, Unicode or locale normalization, separator stripping, unit conversion, or numeric/string coercion. So `1,200`, `+1200` and `1_200` are strings, and a number never equals a string. Ranges and units are only successive components by convention, with no schema, algebra, ordering or conversion.

## 5. Resolution, findings, and exit status

### 5.1 Resolve before comparison

A binding's citation resolves through the ordinary local and workspace resolver ([§FS-workspace.4](FS-workspace.md#4-resolution)), in one order whichever authority made the root. A path that resolves uniquely to one valid root — a whole declaration, a marked root or a chapter root — is root-aimed, and every component of that root takes part. A path whose parent resolves uniquely to one valid root, and whose final numbered segment names one of that root's immediate components, is component-aimed. No path reads both ways over valid authority, because every arrangement in which a root is also another root's component is an invalid declaration ([§FS-values.2.1](FS-values.md#21-markdown-declarations), [§FS-values.2.4](FS-values.md#24-embedded-section-value-roots), [§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)).

An unknown alias, a dangling or duplicate declaration, a duplicate or missing section, an invalid root and a noncanonical shorthand are reported first and suppress value comparison at that site. They suppress the space refusal of [§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root) too, so an invalid root bound at its own path reports its declaration error alone, while malformed binding syntax is reported regardless. A root-aimed binding the space refusal refuses is never also a mismatch. A mismatch exists only once one unique valid authority and its component resolve, or, for a root-aimed binding, its whole component run.

### 5.2 Fixed value errors

Three codes report values, with their rows in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog): `invalid-value-declaration` ([§FS-check.3.20](FS-check.md#320-invalid-value-declaration)), `invalid-value-binding` ([§FS-check.3.21](FS-check.md#321-invalid-value-binding)) and `value-mismatch` ([§FS-check.3.22](FS-check.md#322-value-mismatch)). Each is located, rendered and selected as every finding is ([§FS-check.2.1](FS-check.md#21-report-format)).

A component-aimed mismatch names the cited coordinate and carries that component's declaration site, in the text and in the structured `sites` ([§FS-output-shapes.1.1](FS-output-shapes.md#11-value-mismatch)):

```text
docs/offer.md:7: error: value mismatch for CONST-field-price.1: bound `1250`, declared `1200` at values/field-price.md:2; change the literal to the declared value, or the declaration to the bound one
```

A root-aimed mismatch names what its site wrote ([§DF-root-value-mismatch-wording](../decisions/functional/DF-root-value-mismatch-wording.md#df-root-value-mismatch-wording-a-root-aimed-value-mismatch-names-the-cited-root-and-the-literal-as-written-after-one-release-that-appends-it)): the cited root, the literal as authored, and the root's components as declared, joined by one ASCII space. When the literal has as many parts as the root has components, the line also names the first unequal component by its section path after the ID — `2` for `CONST-field-price.2`, `values.price.1` for the first component of the chapter root `COMP-zz-a.values.price` — and carries that component's site:

```text
value mismatch for <root> (component <section-path>): bound `<literal>`, declared `<joined>` at <site>; change the literal to the declared value, or the declaration to the bound one
docs/offer.md:1: error: value mismatch for CONST-field-price (component 2): bound `1200 EUR`, declared `1200 USD` at values/field-price.md:3; change the literal to the declared value, or the declaration to the bound one
```

When the counts differ no component is at fault: the line keeps the component-aimed shape with the root in place of the coordinate, and carries component 1's site. Either way the finding has exactly one declaration site, so `sites` keeps one entry and an editor one component span ([§FS-lsp.1.1.2](FS-lsp.md#112-findings-the-core-report-shapes)).

```text
docs/offer.md:3: error: value mismatch for CONST-field-price: bound `1200`, declared `1200 USD` at values/field-price.md:2; change the literal to the declared value, or the declaration to the bound one
```

The reported pair is the one component a component-aimed line names, or on a root-aimed line the first unequal component and the part bound to it. When that pair differs in kind ([§FS-values.4](FS-values.md#4-exact-equality)), the two sides can print the same bytes, so the message gains, before its fix, ` — bound is a <kind>, declared is a <kind>; a number and a string are never equal`, or on a root-aimed line ` — component <section-path>: bound is a <kind>, declared is a <kind>; a number and a string are never equal`, where each `<kind>` is `number` or `string`, bound side first, and `<section-path>` is the one the line already names. A pair of one kind, and a part-count mismatch, gain no clause. The clause names no side to change, because which one is wrong is the author's intent rather than a fact of the tree.

```text
docs/record.md:6: error: value mismatch for QTY-mach-001-year.1: bound `2022`, declared `2022` at values/machines.json:2 — bound is a number, declared is a string; a number and a string are never equal; change the literal to the declared value, or the declaration to the bound one
docs/offer.md:5: error: value mismatch for CONST-field-price (component 2): bound `1200 1`, declared `1200 USD` at values/field-price.md:3 — component 2: bound is a number, declared is a string; a number and a string are never equal; change the literal to the declared value, or the declaration to the bound one
```

### 5.3 Incomplete input and deterministic output

A value source that cannot be read or parsed makes the run incomplete ([§FS-check.2.4](FS-check.md#24-an-incomplete-run)): one scan-failure line on stderr with the shared reason of [§FS-check.2.4.1](FS-check.md#241-one-reason-for-one-scan-io-failure), in the source's context (`could not read value JSON source` or `could not read value YAML source`) or its value home's, and exit `2`. A readable source's semantic violation is an ordinary finding:

| A value source that is | Exit and code |
|---|---|
| missing, unreadable, or malformed UTF-8 | `2`, `io` |
| a JSON text or YAML stream the parser rejects, a tab in YAML indentation among them | `2`, `unparsable-file` ([§FS-check.2.4.3](FS-check.md#243-what-failed-and-where-decide-the-code-and-the-fix)) |
| readable, with a wrong shape, an invalid component, or a refused YAML construct or escape | `1`, `invalid-value-declaration` |
| readable, with a repeated key, or declaring an ID another source declares | `1`, `duplicate` |

A source that cannot be read stops every reader of the catalog: `check`, `list`, `show`, `refs`, `cover` and `id` exit `2`, so `show` of an ID only that file declares is that failure, not `ID not found`.

```text
error: values/limits.yaml: invalid YAML: 2:1: while parsing a flow sequence, expected ',' or ']'; make the file valid YAML
```

A YAML source whose collections nest more than 128 deep is a stream the reader rejects, at the line and column of the collection that crosses the bound, with the reason `collections are nested more than 128 deep`, so no source makes the reader recurse past that bound ([§DA-yaml-value-reader.2.2](../decisions/architectural/DA-yaml-value-reader.md#22-one-span-tree-with-no-value-rules-in-it), [§REQ-never-crashes.1](../requirements/REQ-never-crashes.md#1-every-failure-is-a-diagnostic)).

A direct child of a value home that would be a source by its name ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)), but whose file status cannot be read, as when its home can be listed but not searched, is incomplete input too, never a finding that it is no source ([§DF-value-candidate-metadata-failure.2](../decisions/functional/DF-value-candidate-metadata-failure.md#2-decision-and-compatibility)): that candidate or its home is reported on the scan-failure channel, and catalog commands, `list` and `check` among them, exit `2` whether or not anything cites its values. Whether a candidate is a file is read from the same view its content is read from, so an open buffer is a file, a buffer-only one and a physical alias included ([§FS-scan.text.buffer](FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write), [§FS-scan.identity.buffers](FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location)), and an entry identified as a directory, even one named `sub.json`, is silently no source. Selected-folder fetch is held only by such a failure in its own home ([§FS-fetch.5.1](FS-fetch.md#51-existing-value-source-declarations-prevent-installation)).

Value findings keep the shared streams and order ([§FS-output-shapes.3](FS-output-shapes.md#3-text-report-ordering)), and each finding's NDJSON object describes the same finding as its text line ([§FS-output-shapes.1](FS-output-shapes.md#1-finding-object)).

## 6. Shared catalog consumers

Value declarations of every form, and every value root, enter the one declaration and section catalog every command reads.

### 6.1 Marked roots in the catalog

A value root of either origin and its components keep their ordinary dotted section coordinates: `show` prints the queried verbatim section slice, shell completion offers the recorded section paths with no value-specific candidate, and `refs <ID.path>` and `--section <path>` keep their ordinary-section meanings, with no root aggregate. `list` keeps one row for the enclosing declaration and marks its roots on that row ([§FS-list.2](FS-list.md#2-behaviour)): the text suffix of [§FS-list.3.1.1](FS-list.md#311-row-notes) and the NDJSON `value_roots` member of [§FS-list.3.2.2](FS-list.md#322-value_roots), each only on a row with roots, and neither in `--summary`.

The tag is no part of a marked root's semantic heading: its section title, derived Markdown anchor, LSP title and hover token, and semantic title range leave out the tag and the space before it. Raw bytes keep it: `show` prints it on a verbatim root heading, `fmt` keeps it byte for byte ([§FS-values.8](FS-values.md#8-formatting-stability)), and a finding may point at the tag or at the offending source line.

### 6.2 Value-source declarations in the catalog

`list` reports a value source's IDs and locations with no invented title. `id` counts them in its collision checks and never writes a value source. `show` prints the member slice for an ID and the element slice for a section ([§FS-values.2.2.2](FS-values.md#222-reader-fidelity)); `--brief`, the default, `--toc` and `--full` all print that one slice, and no command synthesizes Markdown. A folder kind's generated index links a value source's IDs to their file, with no invented anchor.

## 7. Workspaces and editor consumers

An unqualified binding resolves in its own project. The qualified form `<§>alias/CONST-field-price.1` resolves through the same workspace resolver under the target project's grammar, authority, declarations and equality, including the target's own `value_chapter`, which a member may set where the root project does not. In an editor, a value diagnostic equals the CLI finding, a binding's hover is its exact `show --toc` slice, and go-to-definition lands on the component's Markdown or source heading, or on a value source's key or element. A root's own hover is its ordinary section title and usage count ([§FS-values.6.1](FS-values.md#61-marked-roots-in-the-catalog)). Citations, highlights and document links use the written dotted citation, with no badge, rendered substitution or value-specific completion ([§FS-lsp.1](FS-lsp.md#1-capabilities)).

## 8. Formatting stability

`fmt`'s link pass never wraps or otherwise rewrites the citation bytes of a delimited form that `check` reads as a binding ([§FS-values.3.1](FS-values.md#31-the-only-binding-grammar)) or refuses as an attempted one ([§FS-values.3.1.1](FS-values.md#311-invalid-attempts-and-non-attempts), [§FS-values.3.1.2](FS-values.md#312-a-binding-aimed-at-the-root)), component- or root-aimed, on one line or split across two, whichever authority made its root, a root that only a tag makes in a repository that sets neither key included. So after `--write` `check` reports the same refusal ([§DF-fmt-preserves-value-attempts](../decisions/functional/DF-fmt-preserves-value-attempts.md#df-fmt-preserves-value-attempts-formatting-preserves-every-refused-value-binding-attempt)). The pass judges each citation occurrence by `check`'s own classification, local or qualified, so an ordinary citation of the same target, even on the same line, is still linked, and dry runs, `--check` and `--write` agree. Trigger and safe shorthand rewrites apply there as anywhere. `fmt` never inserts, canonicalizes, moves or removes a tag, and keeps its authored bytes and trailing whitespace.

## 9. Compatibility and explicit exclusions

Without `values = true` (`form = "value"` in version 2) no file is opened as a value source, so a config file, a Markdown file's front matter or a CI workflow is never read as one; without `value_chapter` (`content = "values"` in version 2) no chapter declares roots; and without the exact tag of [§FS-values.2.4](FS-values.md#24-embedded-section-value-roots) no section is a marked root. A repository with none of the three has no value root, binding, value finding or value output field, and reads no extra file; unmarked sections, malformed lookalikes, citations and prose never gain authority ([§FS-non-goals.2](FS-non-goals.md#2-spelling-grammar-prose-quality)). An application may read an opted-in value source directly, and `grund` neither generates nor freshness-checks a language module from one ([§DA-explicit-value-bindings.2](../decisions/architectural/DA-explicit-value-bindings.md#2-decision)).

The feature has no rendering or interpolation, no lint of an adjacent value or a bare literal, no range or unit semantics, no derived arithmetic, generated artifact, target fingerprint or history, no `reconcile` or `stale` command, no excluded-path hint, no value-aware search, no generated-file policy, no JSON or YAML root embedded in a document, no nested marked root and no orphan relief. A named heading is a root only inside a configured chapter, never by its own text, an inline mark or inference ([§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)). The feature changes neither `[reference] strict`, scan scope or filters, custom citation markers or separators, nor the history, AST, documentation-generation, offline and deterministic-install non-goals ([§FS-non-goals](FS-non-goals.md#fs-non-goals-what-grund-will-deliberately-not-do)).

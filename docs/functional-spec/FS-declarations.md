# FS-declarations: a declaration is addressable once, from one allowed place, and holds nothing that is neither a coordinate nor a finding

A declaration is the line that introduces an ID, and every citation addresses one. An ID is
declared exactly once, where its kind's home allows it, and every ATX heading inside its body
in a Markdown file is either a coordinate a citation can reach or a finding `check` reports;
source doc-comment and setext text stay outside that rule
([§FS-declarations.checks.unmarked-heading.1](FS-declarations.md#checksunmarked-heading1-which-headings-participate)). The checks enforce that sentence, save three, which hold
what an address is worth once it resolves: that the declaration has the form its ID grammar
gives it ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)), that its leads stay inside the size
its project budgets where the project asks for one ([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)),
and that its text is not in the middle of a merge ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)). Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and [§GOAL-token-economy.1](../goals.md#1-what-this-requires).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms)
(source declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, place, complement place, workspace, member),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, caution), and [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (rule).

- **marker line** — A line git writes around each side of a conflict it could not merge: column 0, exactly seven `<`, `|`, `=` or `>`, then a space or the end of the line ([§FS-declarations.body.conflict.markers](FS-declarations.md#bodyconflictmarkers-a-marker-line)).
- **conflict block** — The marker lines of one unresolved conflict, from its opening `<<<<<<<` to its closing `>>>>>>>`, with every line between them ([§FS-declarations.body.conflict.block](FS-declarations.md#bodyconflictblock-a-conflict-block)).

## requirements: What every declaration holds to

The points below bind declarations as a whole; the rest of what a declaration holds to is
required elsewhere. That an ID has one declaration and a section path one heading is [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), enforced by the duplicate checks; that every
command reads one declaration set, one stub verdict and one section map, whatever the scope,
is [§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree); and that a check about a declaration is specified here under its code is
[§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code).

### requirements.mentions: Writing about an ID never declares it

A line that names an ID without introducing it never declares that ID, wherever it sits: an ID
followed by a section path, whose token with a declaration colon after it is the near miss and
never a declaration of the shorter ID ([§FS-declarations.line.section-suffix](FS-declarations.md#linesection-suffix-an-id-with-a-section-after-it-declares-nothing)); a heading inside
a fence or a raw-text HTML block, and a `#` line in front matter ([§FS-declarations.heading](FS-declarations.md#heading-what-a-heading-is),
[§FS-declarations.stubs.target](FS-declarations.md#stubstarget-what-a-stubs-target-gives-it), [§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)); and an ID in inline code or Markdown prose,
which is no near miss either ([§FS-declarations.checks.declaration-near-miss.3](FS-declarations.md#checksdeclaration-near-miss3-never-in-inline-code-prose-or-a-fenced-block)). One case is
deferred to [§RM-doc-comment-declarations](../roadmap.md#rm-doc-comment-declarations-declarations-only-in-classmethod-doc-comments): every comment-prefixed line of a source file is a
declaration position ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)), so `// FS-x: …` inside a function body, a
citation whose marker was dropped, declares `FS-x` and makes the real declaration a duplicate.
Serves [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) and [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration).

### requirements.actionable: A finding is located at the fault, and no command makes the fix

Every finding of [§FS-declarations.checks](FS-declarations.md#checks-checks) is located at the fault — the declaration or heading
in question, or the stub whose link is broken — never at a line with nothing to fix, so a
declaration reached through stubs is named at its target ([§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at)),
every other site listed in the message ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration),
[§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)). The message states what is true and the author's
choices, and no command applies one: no corrected ID is proposed
([§FS-declarations.checks.declaration-near-miss.4](FS-declarations.md#checksdeclaration-near-miss4-facts-not-a-guessed-rename), [§FS-declarations.checks.declaration-near-miss.6](FS-declarations.md#checksdeclaration-near-miss6-no-opt-out-no-rewrite)),
a suggested coordinate is guidance `fmt` never writes ([§FS-declarations.checks.unmarked-heading.3](FS-declarations.md#checksunmarked-heading3-the-suggested-coordinate),
[§FS-declarations.checks.unmarked-heading.6](FS-declarations.md#checksunmarked-heading6-no-command-numbers-the-heading)), a non-citable home is named as a place, not mapped
to a kind ([§FS-declarations.checks.misplaced-declaration.3](FS-declarations.md#checksmisplaced-declaration3-a-non-citable-home)), and an oversized lead gets remedies
that keep its citations ([§FS-declarations.checks.oversized-lead.1](FS-declarations.md#checksoversized-lead1-the-message)). It is a direction for every
check added here. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## line: What a line in declaration position declares

A line is in declaration position where a declaration could open: a Markdown heading, a
comment-prefixed line in a source file, behind a configured prefix or any form it stands for
([§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for)), a bare line inside a Python docstring, or a line inside a
`/* … */` block comment, with or without its ` * ` gutter ([§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment),
[§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)). The position is necessary, not sufficient: the token that opens the line must be
the shape a declaration takes, and an ID with a section after it is not
([§FS-declarations.line.section-suffix](FS-declarations.md#linesection-suffix-an-id-with-a-section-after-it-declares-nothing)).

That token is the whole token the effective ID grammar reads ([§FS-config.3.2](FS-config.md#32-id--id-grammar)), in every
declaration position, so a `format` literal such as `:` and every character the `slug_pattern`
admits, trailing punctuation included, belong to the ID, and every command reads that complete
ID ([§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set), [§DF-canonical-slug-declarations](../decisions/functional/DF-canonical-slug-declarations.md#df-canonical-slug-declarations-configured-slug-punctuation-retains-its-canonical-declaration-identity)). A token the grammar does
not read whole declares no ID it starts with ([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)).

A byte-order mark that starts a file is part of line 1, in Markdown and source alike, so line 1
declares nothing, and a block comment or docstring opened on line 1 is not opened, so nothing
inside it declares; a comment line after line 1 still declares.

### line.block-comment: A line inside a block comment

A block comment opens on a line whose first non-blank characters are `/*`, `/**` or `/*!`, and
closes at the first `*/` after its opener, on that line or a later one; it is read where `/*` is
among `[scan] comment_prefixes` ([§FS-languages.comment-block.forms](FS-languages.md#comment-blockforms-comment-blocks), [§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions)).
Once the wrapper of [§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper) is read off, three lines in it are in
declaration position:

- the opener line, as in `/** FS-b: Beta` or `/* FS-b: Beta */`, whose title stops before the
  block's first `*/` and the whitespace in front of it, so `/** FS-b: Beta */ tail */` is titled
  `Beta` ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line), [§DF-one-answer-rulings-parse](../decisions/functional/DF-one-answer-rulings-parse.md#df-one-answer-rulings-parse-one-parse-gives-every-command-the-same-extents-comment-blocks-anchors-and-title-citations) D17d);
- a continuation line behind ` * `: ` * FS-b: Beta`;
- a bare continuation line, indented or not, `FS-b: Beta`, only in a block whose `*/` is found:
  in a block that runs to the end of the file, a bare line declares nothing while the other two
  forms still do.

The bare line declares in any block comment, not only a `/**` doc block, and its body, sections
and lead are its comment block's ([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)), bare lines kept as written, so a
bare `## 1. Sub` under it is its section `.1` ([§DF-bare-block-comment-line-declares](../decisions/functional/DF-bare-block-comment-line-declares.md#df-bare-block-comment-line-declares-a-line-inside-a-block-comment-declares-with-or-without-its-gutter-behind-any-of-its-openers)). A line
after the block's `*/`, after a `/*` that does not start its line (`x = 1; /* note */`,
`let s = "/*";`), or after a `//` comment that contains `/*` (`// see /* here`) is not inside a
block. This is recognition, not parsing: a line that starts with `/*` inside a string literal
spanning several lines opens a block all the same.

### line.section-suffix: An ID with a section after it declares nothing

A line in declaration position whose first token is an ID followed directly by the configured
`[id] section_separator` ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)) and a section path declares nothing, not that ID and
no other: `FS-042-user-login.2` is the shape of a coordinate, never of a declaration. A
docstring line `FS-042-user-login.2 is named here`, a `// FS-042-user-login.2 / a note` and the
heading `# FS-042-user-login.2 and what a session must outlive` declare nothing; such a line is
prose, or a citation where it carries the marker ([§FS-check.1.1](FS-check.md#11-recognized-citations)). Without the declaration colon
it is no `declaration-near-miss` either ([§FS-declarations.checks.declaration-near-miss.2](FS-declarations.md#checksdeclaration-near-miss2-the-declaration-colon-is-the-discriminator)), and
`check` reports nothing about it. With the colon, `# FS-042-user-login.2: the session` has a
token that runs to the colon, section included, so the line is the off-grammar declaration
[§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable) keeps under its exact spelling and
[§FS-declarations.checks.declaration-near-miss.1](FS-declarations.md#checksdeclaration-near-miss1-what-counts) reports, never a declaration of
`FS-042-user-login`. Under `section_separator = ":"`, `FS-042-user-login:2 is named here`
declares nothing and is no near miss. The section is a numbered path, or a named one such as
`FS-042-user-login.goals` only where `named_sections = true` ([§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names)).

## stubs: What a stub points at

A stub is a pointer ([§FS-terms.terms.3](FS-terms.md#terms3-source-forms)): a heading whose title is one link
([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)) to a file, its target, pointing at the declarations of its ID
there. Which those are, if any, is the stub's verdict, and a stub paired with them is not
counted beside them as one more.

### stubs.verdict: One verdict per stub, reached once per scan

Each stub's verdict is reached once per scan, and is one of five: the target is missing; the target is a file the scan does not read ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)); the target, read as the scan reads it on the text a save would write ([§FS-declarations.stubs.target](FS-declarations.md#stubstarget-what-a-stubs-target-gives-it)), does not declare the ID; it declares the ID, on one line or more, and the stub points at each of those declarations ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)); or the target is the stub's own file, which declares the ID on another line, and the stub points at none of them but is one of its own beside them, so the ID is a duplicate ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)). The first three are a broken stub ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)), a stub linked to its own file that declares the ID nowhere else being the third. The ID's declarations are counted from those verdicts, once its stubs are paired.

### stubs.repair: A missing target beside another declaration explains the repair

When a stub's verdict is a missing target and its ID has a real home — a declaration that is
not itself a stub, a target healthy stubs represent included ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned))
— the messages connect the two. The guidance reads only the declarations and verdicts the scan
found: it never infers that a file moved, never searches the filesystem again, and never offers
another broken stub as a destination. Only a missing-target verdict gets it: a healthy,
excluded, unread or read-but-lacking target keeps its own explanation
([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)). With no real home the ordinary message stands.

- `duplicate` keeps its complete message as a prefix and appends each missing-target stub, by
  `path:line` and its authored target, then the real homes. With exactly one, it says the ID is
  declared there and, for a stub in another file, to point the stub's link there; for a stub in
  the home's own file it explains that a link to the same file will not pair it, under the
  own-file verdict ([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)); where several missing stubs share the ID,
  no single instruction covers one in the home's own file. Several homes are named in
  `path:line` order, and none is chosen or offered.
- The missing-target `broken-stub` keeps its complete message as a prefix and appends a
  connection that names no site: that the ID has one real home, and the safe action for this
  stub; that the sole home is in this stub's own file and a link to it will not pair; or that
  there are several homes, whose locations the `duplicate` finding gives. It reads the same whether
  `--only` keeps that finding or not, and never says it is above or below.
- An ID query — a bare ID, `--toc`, a section ID or `refs` — keeps exit `1`, its complete
  `ambiguous ID: <ID> (declared at <sites>)` prefix and its sites' order, and appends the
  `duplicate` explanation; the broken stub is still a declaration of its own.

The fix offered is the `duplicate` and `broken-stub` rows' *Fix* ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)). The guidance
is in the message alone, the same in text and JSON: `sites` lists the declaration sites alone ([§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output)),
`duplicate` stays at its first site even when a real home sorts before the stub
([§FS-errors.4.1](FS-errors.md#41-ordering)), and the missing-target finding stays at the stub with `sites` null
([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)).

### stubs.shape: What a stub looks like

A stub is a heading whose title is one link, `<ID>: [<text>](<path>)`, at any heading level,
in a Markdown file of some kind's home ([§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates)): a file under a kind's `folder`, or a
kind's `file`, citable or not, however many homes hold it. Whether a file is in a home is read
from its path below the config root, so where the checkout sits never moves the answer.
`## FS-x: [x](../../src/x.rs)` in `docs/fs/x.md` is a stub. The complement place is not a home
([§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure)), so a heading of that shape there is a declaration titled with a link, and so
is one in `docs/notes/y.md` where no kind's home holds `docs/notes/`
([§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190)).

### stubs.target: What a stub's target gives it

The link resolves as a Markdown link does, relative to the stub file's directory, and where
nothing is at that path, relative to the config root. The target declares the ID exactly as the
scanner's own pass reads that file, as if it were scanned, and only where the scan would read it
at all ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing); D3 of [§DF-one-answer-rulings-stubs](../decisions/functional/DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is)). It is read on the
text a save would write ([§FS-scan.text.buffer](FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write)), a buffer open under any spelling of the file
or with no file behind it included ([§FS-scan.identity.buffers](FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location), [§FS-scan.text.buffer-only](FS-scan.md#textbuffer-only-a-buffer-with-no-file-behind-it-is-a-file)), so an
unsaved edit breaks or repairs the stub as saving it would ([§REQ-one-answer.text.targets](../requirements/REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would)). A
heading inside a fence ([§FS-languages.fence](FS-languages.md#fence-fences)) or a raw-text HTML block ([§FS-check.1.1.5.1](FS-check.md#1151-a-raw-html-block-holds-no-heading)) of
a Markdown target declares nothing ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)), inside `[scan] include` or not, while in a
source target a doc-comment's fence hides only section and plain headings, never a declaration
line ([§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings), [§DF-stub-target-fenced-heading](../decisions/functional/DF-stub-target-fenced-heading.md#df-stub-target-fenced-heading-a-heading-inside-a-fence-of-a-stubs-target-does-not-declare-its-id)). A heading in the
target that is itself a stub, by the shape test on the target's own path, declares nothing
there, so a chain of stubs is broken at its first link (D13 of [§DF-one-answer-rulings-stubs](../decisions/functional/DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is)).

#### stubs.target.3: A target the scan does not read declares nothing

A target the scan never reads declares nothing, whatever it holds, so its stub is broken as if
the target never named the ID, inside `[scan] include` or not. Wherever it sits, that is a
target whose own name begins with `.`, skipped by that name before `extensions` is consulted
([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), and one whose extension `[scan] extensions` does not list, a name with
none included ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), or, in a version-2 file, one no listed language reads or a
listed `data` language reads, which only a value kind's value pass reads ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)). The
name judged is the one the stub wrote, not the one a symlink resolves to: `notes/.a.md -> a.md`
declares nothing even where the scanned `notes/a.md` declares the ID, so the stub pairs with
neither and is a duplicate beside it. Listing the extension under version 1, mapping it to a
listed language or listing a language that reads it under version 2
([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)), or renaming the target, lets the target be read.

A target is refused too by a directory on its path that `[scan] exclude` names
([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)), by a version-2 exclusion pattern
([§FS-config-v2.schema.sources.exclude.stubs](FS-config-v2.md#schemasourcesexcludestubs-a-pattern-is-the-exclude-reason-a-stubs-target-is-not-read)), and by an enabled ignore rule
([§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)), before the stub pairs or lends any of its content
([§DF-stub-target-exclusions](../decisions/functional/DF-stub-target-exclusions.md#df-stub-target-exclusions-excluded-and-ignored-stub-targets-are-broken-from-grund-0190)). They are asked as the walk asks them, from each scan root the run
walks and from the config root in place of `include`, on the target's lexical link spelling
([§FS-scan.scope.links.names](FS-scan.md#scopelinksnames-the-directory-rules-apply-under-the-link-name)). A scan root under an excluded or ignored name outruns them as in
the walk ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)). After the name and extension refusals, the first rule met
descending refuses, `exclude` before ignore at one step. `respect_gitignore = false` turns off
ignore rules only, `--full` still applies every one of these refusals, and a path narrows only
the report. Lying outside `include` is no reason, and neither is a hidden ancestor, another
project's boundary, an unwalked home or a target outside the config root.

The hidden-name refusal comes first and, like a target that is read and lacks the ID, reads
`stub link target lacks <ID>: <path>`. Every other refusal says the target is not scanned and
never that it lacks the ID, naming the link as the stub wrote it ([§FS-errors.3](FS-errors.md#3-message-text)). For the
extension the reason is the cell of the reason table in [§FS-scan.checks.unread-scan-root.message](FS-scan.md#checksunread-scan-rootmessage-the-message),
the data row for a listed `data` language, with no remedy clause, as the first line below
shows. For `exclude` and ignore it is `[scan] exclude names <entry>`,
`[schema.sources] exclude matches it by <pattern>` or `<file> ignores it by <pattern>`, entries
and patterns as written and an ignore file spelled from the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)),
absolute outside the loaded root, and it is joined to the `broken-stub` row's *Fix*
([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)) as the second line shows:

```text
docs/architecture/AR-c.md:1: error: stub link target src/c/router.h is not scanned: extension h is not in [scan] extensions
docs/fs/x.md:1: error: stub link target ../../notes/gen/a.md is not scanned: [scan] exclude names gen; this became an error in grund 0.19.0 — move the target out of gen, or take gen out of [scan] exclude
```

#### stubs.target.5: What a target outside the include gives, and what stays unjudged

A target outside `[scan] include` gives the stub what the same tree with the target scanned
gives it ([§REQ-one-answer.stubs.outside-include](../requirements/REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned)):

- its declarations of the ID, counted as [§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned) counts them, and their
  sections, so a citation into one reads as it would were the target scanned ([§FS-check.3.2.1](FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not)),
  and so does every reader that enumerates a declaration's sections
  ([§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)). No target of an ID with more than one declaration lends a
  section, a target that declares the ID twice included ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments), [§FS-show.2.2.1](FS-show.md#221-ambiguous-id));
- every section path one of those declarations claims twice, a `duplicate-section`
  ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)) at the target's headings, whether or not anything
  cites it ([§DF-stub-target-section-declared-twice](../decisions/functional/DF-stub-target-section-declared-twice.md#df-stub-target-section-declared-twice-a-section-a-stubs-target-declares-twice-is-reported-where-it-is-written-scanned-or-not));
- every conflict block in the target, a `conflict-marker`
  ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)) at the target's lines, with the owners the target's
  own pass gives it, on which `fmt` writes nothing ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model));
- each citation the scanner's pass over the target assigns to one of those declarations, in its
  body: once it resolves, through the shorthand and the declaration-local section citation as a
  scanned one does ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand), [§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)), it is an edge for a rule's obligation,
  prohibition and counts ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)) and for a `[citations]` obligation and prohibition
  ([§FS-check.3.11](FS-check.md#311-missing-required-citation), [§FS-check.3.12](FS-check.md#312-forbidden-citation)), `refs` lists it ([§FS-refs.2](FS-refs.md#2-behaviour)), and the ID it cites is not
  `unused` ([§FS-check.4.1](FS-check.md#41-unused-declaration));
- each of those declarations' chapters, cited or not, as a rule's subject and in a chapter
  rule's count ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path), [§FS-rules.3.1](FS-rules.md#31-chapter-presence)).

Stubs to one target lend its citations and chapters once between them, a target that declares
the ID twice lends each declaration's own, and a broken stub
([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)) lends nothing. A finding about a lent citation is located
at the citation's line in the target, and one about the declaration itself, a count, a chapter
count or `unused`, at the declaration's line there ([§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at)), each an
error or a warning as the same finding is in the scanned tree.

Nothing else of the target is judged, because it is read on the stub's behalf, not scanned (D8
of [§DF-one-answer-rulings-stubs](../decisions/functional/DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is)): its other declarations, its notes, formatting and leads,
whether it is grounded and whether its citations resolve earn no finding. A citation of it that
does not resolve counts for nothing, as in the scanned tree ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)), and is reported
nowhere; one the pass assigns to no declaration of the ID is not lent, nor is another
declaration's chapter. A target outside `include` still answers to
[§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing): one that `exclude` or an enabled ignore rule refuses is broken
and lends no body, sections, chapters or citation edges, even if its text declares the ID.
Decided in [§DF-stub-sections-from-unscanned-target](../decisions/functional/DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it) and
[§DF-stub-target-citations-and-chapters](../decisions/functional/DF-stub-target-citations-and-chapters.md#df-stub-target-citations-and-chapters-a-stubs-target-lends-the-stubbed-declaration-its-citations-and-chapters-and-what-that-newly-raises-warns-until-0190).

### stubs.pairing: How a stub pairs with what it points at

A stub whose target declares its ID is the pointer to those declarations, never one more beside
them ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), and a declaration reached through stubs is shown where
it is written, in its target ([§DF-stub-pairs-with-unscanned-target](../decisions/functional/DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it),
[§DF-stub-target-declared-twice](../decisions/functional/DF-stub-target-declared-twice.md#df-stub-target-declared-twice-a-stubs-target-that-declares-its-id-twice-is-two-homes-scanned-or-not), [§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190)).

Stubs of one ID whose links resolve to one target file stand for its declarations once between
them, and stubs whose links resolve to different files, each declaring the ID, make the ID a
duplicate. A surface that shows such a declaration at a stub rather than at its target — `list`,
where the target is outside the scan — shows it once, at the first of its stubs in `path:line`
order, and once for a target that declares the ID twice too, a row that carries the note every
line of a duplicated ID carries ([§FS-list.3.1.1](FS-list.md#311-row-notes)).

A surface that says where a paired declaration is names the target's declaration, its
`<path>:<line>` and its title, as the same tree with the target scanned names it: `show`
([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)), plain `list` ([§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical)), `list --size` ([§FS-list.3.4.6](FS-list.md#346-a-home-outside-the-scan)), and every finding
about the declaration itself — `unused` ([§FS-check.4.1](FS-check.md#41-unused-declaration)), and a count or chapter count it fails
([§FS-rules.7.2](FS-rules.md#72-chapter-cardinality), [§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality), [§FS-rules.7.4](FS-rules.md#74-inbound-citation-cardinality)). A duplicate error or an ambiguity refusal
([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)) names each of the target's declarations of the ID, never a stub's own line, which
has nothing to fix, its sites sorted and the error located per [§FS-check.2.1](FS-check.md#21-report-format) over those
names.

#### stubs.pairing.1: A stub pairs with its target whether or not the target is scanned

A stub whose target declares its ID — the stub [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) accepts — is the pointer to that declaration, not a second declaration of the ID, whether or not `[scan] include` reaches the target ([§REQ-one-answer.stubs.outside-include](../requirements/REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned)). The ID's declarations are counted off each stub's verdict ([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)), on the target's text as [§FS-declarations.stubs.target](FS-declarations.md#stubstarget-what-a-stubs-target-gives-it) reads it, so an unsaved edit to the target moves the count as saving it would ([§REQ-one-answer.text.targets](../requirements/REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would)). A lone stub to an unscanned target still stands for the declaration it points at, so its ID has one declaration, never none. It stands for every declaration of the ID in its target that is not itself a stub, each read as the scan would read it were the target scanned, so a target that declares the ID twice is two declarations and the ID is a duplicate, lone stub or not, named at both lines ([§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at)) as the same tree with the target scanned names them. A broken stub pairs with nothing and stays a declaration of its own, scanned target or not.

## heading: What a heading is

A heading is a Markdown ATX heading line (`#` through `######`) outside any fenced block,
raw-text HTML block or front matter ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)): a line inside one is never a heading,
by [§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example), and in a comment block the fence is the one read through the comment wrapper
([§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)). A heading either opens a declaration
([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)), is a citable section of the declaration whose body holds it
([§FS-declarations.section](FS-declarations.md#section-what-a-section-is)), or is plain prose. In a source file the comment-prefixed
declaration line plays a heading's part ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)).

### heading.rendered-text: A heading's rendered text

A heading's rendered text is its text as a Markdown renderer shows it. Where the heading
contains an inline link (including a citation that `--cross-refs` has already wrapped,
[§FS-fmt.6.4](FS-fmt.md#64-what-is-never-wrapped)) or an HTML-tag-shaped span, `## 4. Refining [§FS-<x>.1](FS-x.md#1-y)` reads
as if it read `## 4. Refining §FS-<x>.1` (the destination URL is not part of the text),
and `## RM-read: grund <ID>` reads as `## RM-read: grund ` (the `<ID>` is dropped) —
exactly as a Markdown renderer treats them. For an explicit named heading, the rendered
text includes the name and colon: `## goals: Scope` reads `goals: Scope`, and so does a
name under a number: `### 1.goals: Goals` reads `1.goals: Goals`.

A closing sequence of `#` characters, and the spaces or tabs before it, is not part of the
rendered text, exactly as CommonMark reads an ATX heading: `## 1. Steps ##` reads `1. Steps`,
so its anchor is `1-steps` ([§FS-fmt.6.2.2](FS-fmt.md#622-the-anchor-slugifies-the-headings-rendered-text)). A `#` run that is not preceded by a space or tab, or that
is followed by anything but spaces or tabs, is text: `## C#` reads `C#`. Every anchor `fmt`
writes and `show` reports is slugged from this text ([§DF-closing-hashes-are-not-the-heading](../decisions/functional/DF-closing-hashes-are-not-the-heading.md#df-closing-hashes-are-not-the-heading-a-headings-closing-hashes-are-not-part-of-its-anchor)).
A title ([§FS-list.3.1](FS-list.md#31---format-text-default)) is not this text: it keeps its links.

## body: Where a declaration's body ends

A declaration's body runs from its declaration line to the end of its body span. In
Markdown that is the start of the next same-or-higher heading, whether or not it declares
an ID (or end of file); in a source file it is the end of the comment block the
declaration line opens ([§FS-show.2.3.1](FS-show.md#231-what-counts-as-the-comment-block)). The span, not the most recently scanned
declaration, owns the coordinate: a same-or-higher Markdown heading, the end of an inline
source doc-comment or docstring, the next declaration in a shared comment block, and a
stub's single declaration line each end ownership. The file's front matter, which no span
reaches, still belongs to its first declaration without growing its body
([§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)), and in a kind that declares a `front-matter` field it is that
declaration's front-matter sections, outside the body ([§FS-fields.front-matter.coordinates](FS-fields.md#front-mattercoordinates-its-coordinates)).

### body.conflict: A body still in the middle of a merge

A declaration whose body holds a conflict block is not one text but two, and nothing in the tree says which is the declaration's: every citation of it, and every fingerprint the section lock takes of it ([§FS-section-lock.holds.fingerprint](FS-section-lock.md#holdsfingerprint-the-fingerprint)), would read one address as both ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)). So `check` reports each block ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)), and `fmt` writes nothing while one exists ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)). A block changes no recognition: every other command reads the file as it would without it, and `show` still prints the bytes it reads ([§FS-show.requirements.authored-bytes](FS-show.md#requirementsauthored-bytes-a-slice-is-the-authors-bytes-selected-and-never-rewritten)). Decided in [§DF-conflicted-declaration](../decisions/functional/DF-conflicted-declaration.md#df-conflicted-declaration-a-declaration-that-still-holds-an-unresolved-merge-conflict-fails-check-and-stops-fmt-from-writing).

#### body.conflict.markers: A marker line

A marker line starts at column 0 with exactly seven of one character, `<`, `|`, `=` or `>`, followed by a space or the end of the line, a CR before the LF being the end of the line: `<<<<<<< HEAD`, `=======` and `>>>>>>> other` are marker lines, and `<<<<<<<< HEAD` (eight), `<<<<<<<HEAD` (no space) and ` <<<<<<< HEAD` (indented) are not. The test is on the raw line, so a marker line inside a fence, an HTML comment, a docstring or a block comment is one all the same.

#### body.conflict.block: A conflict block

A conflict block opens at a `<<<<<<<` line and closes at the next `>>>>>>>` line, and is those two lines with every line between them. A `|||||||` or `=======` line is a marker only inside an open block, so a setext underline of seven `=` or a table row of seven `|` outside one is text, and a `<<<<<<<` inside an open block opens no second one. Two ill-formed blocks are blocks all the same: a `>>>>>>>` with no block open is a block of that one line, and a `<<<<<<<` that no `>>>>>>>` closes is a block that ends at its last marker line. Outside an open block a `>>>>>>> ` line is such a lone closer, not a seven-level blockquote, which is written `> > > > > > >`.

Blocks are read off a file's raw lines, top to bottom, in two places:

- **Every line of a Markdown file that declares an ID**: its preamble, its fences, and the lines after a heading that closed every body. Whether the file declares ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)) is read with fences ignored ([§DF-conflicted-declaration](../decisions/functional/DF-conflicted-declaration.md#df-conflicted-declaration-a-declaration-that-still-holds-an-unresolved-merge-conflict-fails-check-and-stops-fmt-from-writing)).
- **In a source file, the raw lines of each declaration's body** ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), a docstring or `/** */` comment as much as a line-style run, and a block that opens on the line right after a line-style run ends ([§FS-languages.comment-block.close](FS-languages.md#comment-blockclose-find-the-close-boundary)), through its last line: a marker line is not a comment line, so a conflict inside a `///` or `#` run cuts the run at its opening marker.

A marker line anywhere else is not looked at: in code outside those places, in a Markdown file that declares no ID, or in a file the scan does not read. A target a stub pairs with outside `[scan] include` is looked at as if it were scanned, and its blocks are reported at its own lines ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)).

A block names every declaration whose declaration line or body its lines overlap, each ID once, in the order its first such declaration opens, and a block counted after a cut line-style run names the declaration whose run it cut. A block that overlaps no body names none. A block that holds declaration lines names the declaration above it and each one inside it, so a merge that wrote `# FS-b: B` on one side and `# FS-c: C` on the other, below `# FS-a: A`, names all three. Owners are read from the body spans every command reads ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), so `check`'s finding and `fmt`'s refusal name the same declarations.

## section: What a section is

A section is a citable heading inside a declaration's body: a numbered heading, or a named
one where named sections are enabled, whose path is written in the grammar of
[§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth), or taken from its parent heading by a relative heading
([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)). A plain heading is not a section; it remains prose. A
section's extent is its heading and everything down to the next sibling-or-shallower heading;
nested deeper headings included. Which sections a declaration has is its section map
([§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span)). In a kind that declares a `front-matter` field, the front
matter is a section, `<ID>.front-matter`, and so is each top-level key whose name is a section
name, `<ID>.front-matter.<key>`, with no heading and outside the body
([§FS-fields.front-matter.coordinates](FS-fields.md#front-mattercoordinates-its-coordinates)).

### section.map: The section map is the declaration's body span

A coordinate enters the section map only when its heading lies inside that declaration's
existing body span ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)). A deeper numeric or enabled named heading
before that boundary remains a section; one after it is absent from the map and is
reported by [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration). Fenced pseudo-headings, in
Markdown or a comment block, remain content ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)). This boundary makes no otherwise legal
plain heading inside a body an error; whether a plain heading is one is
[§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)'s question. Every surface reads this map ([§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)),
so none answers for a heading it rejects. Its only entries without a heading are the
front-matter coordinates of a kind with a `front-matter` field, above the body span and marked
as lying in the front matter ([§FS-fields.front-matter.coordinates](FS-fields.md#front-mattercoordinates-its-coordinates)).

Bodies nest in Markdown: a declaration heading deeper than an enclosing one opens a body that
ends at the next heading at its level or shallower, while the enclosing body goes on. A
numbered, enabled named or relative heading after the nested body ends, and before the
enclosing one does, is a coordinate of the innermost declaration whose body holds it, never of
the nested declaration the scan read last, and never dropped from every map:

```markdown
## AR-ci: CI

### AR-matrix: Matrix

### 1. Pull-request workflows
```

`### 1. Pull-request workflows` is `AR-ci.1`, and `AR-matrix` has no section `1`. Such a heading
is a claimant like any other in that body for [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path) and
[§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch); the declaration whose body holds it is also the
owner [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) names for a plain heading in the same place
([§DF-section-after-nested-declaration](../decisions/functional/DF-section-after-nested-declaration.md#df-section-after-nested-declaration-a-section-heading-after-a-nested-declaration-is-the-enclosing-declarations-coordinate)).

### section.relative: A heading may write only the component it adds

Inside a declaration body a heading may write only the component it adds and take the rest of
its path from its parent: under `## format: Formatting`, `### .1: Username` declares
`format.1`. Its **canonical path** is its parent's path, a `.`, and its component, or the
component alone under the declaration heading, and it is the section's one coordinate: `.1`
alone is never a coordinate, and no citation form addresses a section relative to anything
([§FS-declarations.section.relative.shown](FS-declarations.md#sectionrelativeshown-how-a-relative-heading-is-shown)). The form needs no setting, and relative and
complete-path headings mix freely at any depth. The section lock guards a cited relative
section against silently holding other text after a move ([§FS-section-lock](FS-section-lock.md#fs-section-lock-a-cited-relative-or-mixed-section-keeps-its-text-or-check-says-where-the-text-went)). Decided in
[§DF-relative-section-headings](../decisions/functional/DF-relative-section-headings.md#df-relative-section-headings-a-heading-may-write-only-the-component-it-adds).

#### section.relative.form: The form

A relative heading writes one `.` and exactly one component, a number `[0-9]+` or a name in the grammar of [§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names), then the title syntax the complete-path heading for the same canonical path would use: the optional trailing period and space-separated title of a purely numeric path, or the colon form of a name-bearing one ([§FS-config.3.3.1](FS-config.md#331-named-components)).

| Parent | Relative heading | Canonical path | Complete-path heading it stands for |
|---|---|---|---|
| `## 3. Third` | `### .1 Username` or `### .1. Username` | `3.1` | `### 3.1 Username` or `### 3.1. Username` |
| `## format: Formatting` | `### .1: Username` | `format.1` | `### format.1: Username` |
| `## format: Formatting` | `### .user: Username` | `format.user` | `### format.user: Username` |
| `## 1. Overview` | `### .goals: Goals` | `1.goals` | `### 1.goals: Goals` |
| the declaration | `## .format: Formatting` | `format` | `## format: Formatting` |
| the declaration | `## .3 Third` | `3` | `## 3 Third` |

A relative heading is refused for everything the complete-path heading would be: a name with `named_sections` off, the colon form on a purely numeric path, and a numeric path without its space or period; only a name needs `named_sections`, so with it off `### .1 Username` under `## 3. Third` is still `3.1`. A dotted heading not of this form — two components (`#### .a.b: Deep`), an empty one (`### .: Title`), or any refusal — declares nothing and stays an [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) error, whose message also shows the relative form ([§FS-declarations.checks.unmarked-heading.2.1](FS-declarations.md#checksunmarked-heading21-a-dotted-heading-is-also-shown-the-relative-form)).

The form is read on the ATX headings of a Markdown declaration body that [§FS-declarations.checks.unmarked-heading.1](FS-declarations.md#checksunmarked-heading1-which-headings-participate) judges, and on the headings a source doc-comment's section reading sees ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)), under the same form, parent, lock and display rules, the citations below it belonging to its section. In a doc-comment the parent is read from the section map the record gives that body, and a mixed canonical path ([§FS-config.3.3.1](FS-config.md#331-named-components)) is declared in both spellings, a complete `/// ### 1.goals: Goals` and a relative `/// ### .goals: Goals` under `/// ## 1. Overview` ([§DF-number-name-sections.decision.doc-comments](../decisions/functional/DF-number-name-sections.md#decisiondoc-comments-doc-comments-wait-for-639), [§DF-doc-comment-relative-headings](../decisions/functional/DF-doc-comment-relative-headings.md#df-doc-comment-relative-headings-a-relative-heading-in-a-doc-comment-warns-in-0180-and-is-a-section-from-0190)). Everywhere else a dotted line keeps the meaning it has without this form: in a file title before the first declaration, after a heading that closed the body, in a file or comment block with no declaration, and in a fence or raw HTML block.

#### section.relative.parent: The parent

A relative heading at level N takes as its parent the nearest heading before it in the same body that is shallower than N, a heading inside a fence being no candidate. That heading must be exactly level N−1 and be the declaration heading or a **usable** section: one the scan recorded, whose path is not an orphan ([§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path)), and which is not any claimant of a path claimed twice ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)). A parent with its own depth finding ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)) is usable, and its child does not repeat the finding. Otherwise — a skipped level, a plain heading, an orphan, a duplicate, a failed relative heading, a would-be parent inside a fence — the relative heading declares nothing and is a [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent) error naming the parent it needed.

Relative headings chain: `#### .1:` under `### .user:` under `## format:` is `format.user.1`. A relative and a complete-path heading that resolve to one canonical path, or two relative ones that do, are a [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path) like any two claimants. A relative heading is one level below its parent by construction, so it never carries a depth finding of its own ([§FS-config.3.3.2](FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)). A relative heading always needs its actual parent, even where a complete-path heading's numeric prefix need not be recorded ([§FS-config.3.3.3](FS-config.md#333-every-prefix-of-a-name-bearing-path-is-recorded)).

#### section.relative.shown: How a relative heading is shown

Every surface that reports a coordinate reports the canonical path. Three surfaces print a heading line for the reader to copy a coordinate from, and print a relative heading with its canonical path in place of the relative one, at its written depth and with everything after the path kept:

- the section map ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)): `### .1: Username` under `## format: Formatting` maps as `### format.1: Username`, and `### .1. Username` under `## 3. Third` as `### 3.1. Username`;
- the title hover, `` `format.1: Username` `` ([§FS-lsp.1.2.4](FS-lsp.md#124-the-title-hovers-text));
- a read that selects a section, by its coordinate or by `--section`, on the slice's first line, the section's own heading, in all four modes — the default read, `--brief`, `--toc` and `--full` — for a Markdown and a doc-comment body alike: with `### .2: Ceiling` under `## retry: Retry schedule` in `FS-billing`, `grund FS-billing.retry.2` and `grund FS-billing --section retry.2` both open with `### retry.2: Ceiling`, the slice's self-label ([§FS-show.2.2](FS-show.md#22-section), [§DF-show-selected-relative-heading](../decisions/functional/DF-show-selected-relative-heading.md#df-show-selected-relative-heading-a-read-of-a-selected-section-names-the-section-it-read)). `md` and the JSON `body` carry the same line as `text`, and `list --size` measures it ([§FS-list.3.4.1](FS-list.md#341-measured-strings)).

Every other heading line a read prints, a fenced one among them, shows a relative heading as written, in the JSON `body` of those reads too. A `--toc` read's JSON carries the map's lines in `body`, and each `sections[]` entry's `path` is the canonical path, its `title` and `anchor` those of the heading as written. A link anchor follows the heading as written and rendered, so `### .2: Ceiling` keeps `2-ceiling` ([§FS-fmt.6.2.2](FS-fmt.md#622-the-anchor-slugifies-the-headings-rendered-text)), and a doc-comment section's anchor stays null. No command rewrites a relative heading into its complete form or back in the file ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)), and the expansion, from the recorded parent, is a structural slice, not a summary ([§FS-non-goals.14](FS-non-goals.md#14-generated-summaries-token-saving-inside-check)).

A relative heading's display name is what follows its component: `## .terms: Terms` is a `Terms` chapter to a rule ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)), `## .status: Accepted` holds the field value `Accepted` ([§FS-fields.content.one-of](FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)), and `list --selector` titles either one as its explicit form is titled.

## lead: What cuts a lead

A declaration's lead is the prose between the declaration heading and the first child
citable section heading (`## 1. ...`, or `## goals: ...` when named sections are enabled).
A named heading is a citable section and cuts its parent's lead exactly as a numbered
heading does; a plain heading remains prose and does not cut it. A deeper declaration's
heading cuts its parent's lead too, because the lead is the parent's own prose and the deeper
declaration's is not. A lead may be empty.

### lead.section: A section's lead

A section's lead is the same cut one level down: the prose between that heading and the
first *child* heading (`#### 3.1.1 ...` under `### 3.1 ...`). A section that opens
directly with a child heading has an empty lead.

## e2e-case: What an E2E case directory declares

The `E2E` kind is the one kind no heading line declares: a **case directory** does. Where a
project configures a citable kind named `E2E` with a `folder` ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), that folder
holds the cases; without one there are no case directories, and every directory this point
describes is ordinary tree. Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration): a `§E2E-<name>` citation resolves
exactly where its case exists.

### e2e-case.id: A case directory and the ID it declares

On a run over the project, where the `E2E` folder holds no `expected.exit` of its own, a
directory is a case directory when all three hold:

1. it sits directly in the `E2E` folder, never deeper, whatever a deeper one holds;
2. it contains a file named `expected.exit`, or a link of that name whose target does not
   resolve, which makes it a case that fails at the link ([§FS-declarations.e2e-case.failure](FS-declarations.md#e2e-casefailure-a-case-that-cannot-be-read-fails-alone));
3. its name, written after `E2E` and the literal that follows `{kind}` in `[id] format`
   ([§FS-config.3.2](FS-config.md#32-id--id-grammar)), reads as an `E2E` ID with no section.

That ID is what it declares: under the default `{kind}-{number}-{slug}`, `007-login` declares
`E2E-007-login`. A directory that fails any of the three declares nothing. What a run scoped to
a path inside the folder declares, or a folder that holds `expected.exit` itself, is not
specified here.

A case declaration is located at its directory, line 1. It is never a stub and has no sections,
so a citation of it with a section suffix names a section that does not exist ([§FS-show.2.4](FS-show.md#24-e2e-cases)),
and no title, which grund does not make up from the name: `grund list` leaves its title column
empty and its JSON title is `null` ([§FS-list.3.1.1](FS-list.md#311-row-notes), [§FS-output-shapes.5](FS-output-shapes.md#5-list---formatjson)). Its body is the
manifest `grund E2E-<name>` prints ([§FS-show.2.4](FS-show.md#24-e2e-cases)), and a case nothing cites is not reported as
unused ([§FS-check.4.1.3](FS-check.md#413-e2e-declarations-are-exempt)).

### e2e-case.walk: The walk stops at a case directory

A case directory is a boundary of the ordinary walk: the case is registered through its
manifest, and the walk does not descend into it, so nothing of the fixture repository it
carries — its own `grund.toml`, its specs, its nested cases — is read as a declaration or a
citation of this project. The boundary holds wherever the walk would enter the directory, under
its own name or through a symlink that resolves to it, and `--full` keeps it ([§FS-check.1.3.1](FS-check.md#131-the-scan-covers-the-whole-config-root)).
It stops a walk that would enter a case, not one that starts below it: a path argument naming a
file or directory inside a case directory is where that run's walk starts
([§FS-scan.path.discovery](FS-scan.md#pathdiscovery-a-path-is-where-discovery-starts)), and it is read.

### e2e-case.excluded: `exclude` and an ignore file keep a case out

A case is found by the ordinary walk and its read predicate, with no exception: below a scan
root, a case directory that `exclude` or an ignore file matches, a `[scan] exclude` name in a
version 1 configuration, is not read, `--full` or not
([§FS-config-v2.schema.sources.exclude.roots](FS-config-v2.md#schemasourcesexcluderoots-how-exclude-composes-with-the-other-filters), item 2), wherever the cases folder lies, listed by the
walk or a link out of the project the walk skips. An excluded case declares no ID, so a citation of it dangles,
`grund list` does not name it, and nothing in it can fail the run. No other reason the walk
has not to enter a directory, such as a link out of the project or another project's tree,
keeps a case out, since the walk never descends into a case ([§FS-declarations.e2e-case.walk](FS-declarations.md#e2e-casewalk-the-walk-stops-at-a-case-directory)):
a case directory that is such a link is still read.

A case is read whole with its manifests and fixture tree, whatever `exclude` or an ignore file
says of a path inside the case directory: a fixture's own `.gitignore` is the fixture
repository's content, and a pattern such as `repo/`, which keeps fixture trees out of the walk,
neither trims the fixture list `grund E2E-<name>` prints ([§FS-show.2.4](FS-show.md#24-e2e-cases)) nor hides a failure
reading the case ([§FS-declarations.e2e-case.failure](FS-declarations.md#e2e-casefailure-a-case-that-cannot-be-read-fails-alone)). A case directory has none of the exemption
[§FS-values.2.2](FS-values.md#22-declarations-from-value-sources) gives value sources. Decided in [§DF-e2e-case-on-the-one-walk](../decisions/functional/DF-e2e-case-on-the-one-walk.md#df-e2e-case-on-the-one-walk-an-e2e-case-is-read-by-the-one-walk-so-exclude-and-ignore-files-keep-it-out-and-a-scan-failure-is-one-line-at-the-path-that-failed).

### e2e-case.failure: A case that cannot be read fails alone

A failure reading one case directory belongs to that case alone: a `command.args`,
`expected.exit` or `spec.refs` that cannot be read or decoded, an `expected.exit` that does not
parse as an exit code, or a fixture directory that cannot be listed. A manifest whose name is a
link whose target does not resolve fails the case at the link's own path, with the reason the
walk gives the same link ([§FS-scan.scope.links.unresolvable](FS-scan.md#scopelinksunresolvable-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into)), and its entries are never read as
none; only a name that is not there at all is an absent manifest, or, for `expected.exit`, no case. The failure is a per-file failure
of [§FS-check.2.4](FS-check.md#24-an-incomplete-run), located at the file or directory that failed as the tree spells it under the
configured folder ([§DF-e2e-case-failure-at-the-file](../decisions/functional/DF-e2e-case-failure-at-the-file.md#df-e2e-case-failure-at-the-file-a-failure-reading-an-e2e-case-is-located-at-the-file-that-failed-and-path-scoped-exits-move-with-it-in-both-directions)). The walk over the cases goes on, wherever
the failing case sorts, and every other case declares its ID as usual: a citation of it
resolves, its findings print and `grund list` names it ([§DF-e2e-case-read-failure-fails-alone](../decisions/functional/DF-e2e-case-read-failure-fails-alone.md#df-e2e-case-read-failure-fails-alone-a-failure-reading-one-e2e-case-is-that-cases-alone-and-path-scoped-exits-move-with-it-in-both-directions)).
Each failing file or directory is its own report, in one case or two, and the run exits as
[§FS-check.2.4](FS-check.md#24-an-incomplete-run) says.

A run narrowed to a path holds the failure exactly when the path holds that file as the tree
spells it, through a link that resolves to that spelling included, and otherwise cautions, as
[§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) says. A file reported at several spellings is one failure and one line
([§FS-check.2.4.2](FS-check.md#242-one-failure-one-line)). A fixture directory reported at another spelling is one line only where the
report kept this spelling as a name of the failure, and a run holding this spelling still holds
that line; otherwise each spelling reported is a line of its own. What
a citation of the failing case itself does is not specified here.

## checks: Checks

Each section below is named by a finding code ([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)) and says what
`grund check` judges under it; `stub-shape` and `excluded-stub-target` name checks that never
fire, and `parse-moves` is a name `check` refuses. A code's row in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) holds its
severity, what turns it on and its fix. A finding is located, rendered and selected as
[§FS-check.2.1](FS-check.md#21-report-format) says and exits as [§FS-check.2](FS-check.md#2-outputs) says; a section says only what differs.

### checks.duplicate: Duplicate declaration

The same ID declared more than once by declarations that are not stubs ([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)), headings or inline doc-comment declarations, in one file or several; how a stub counts is [§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at). It is one error at the first site in the order of [§FS-errors.4.1.1](FS-errors.md#411-the-order-of-a-duplicates-sites), naming the others. Where no site is a stub the fix is the row's `rename` case; where one is, the stub's repair ([§FS-declarations.stubs.repair](FS-declarations.md#stubsrepair-a-missing-target-beside-another-declaration-explains-the-repair)) or its `broken-stub` finding names it. The value-source duplicates [§FS-values.2.3](FS-values.md#23-duplicates-and-ownership) lists, a value source two opted-in homes claim among them, are this finding too, even when their components agree, and it names each spelling once.

#### checks.duplicate.1: A stub pairs with its target whether or not the target is scanned

Moved to [§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned).

#### checks.duplicate.2: Stubs to one target stand for its declarations once

Moved to [§FS-declarations.stubs.pairing.2](FS-declarations.md#stubspairing2-stubs-to-one-target-stand-for-its-declarations-once).

#### checks.duplicate.3: A declaration reached through stubs is named at its target

Moved to [§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target).

### checks.declaration-near-miss: Declaration near miss

A near miss is a line in declaration position ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares); in a source file, read through its comment block, [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)) whose first token begins unambiguously with a configured citable kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), which the effective ID grammar rejects, and which is followed by the declaration colon. The colon is required because a line that opens with an ID-shaped token and no colon is prose far more often than a declaration. This also covers a per-kind format whose first literal after `{kind}` differs from the persisted token. It stays a declaration for compatibility ([§FS-config.3.2](FS-config.md#32-id--id-grammar)): `list`, `show`, `refs`, `cover`, `fmt` and LSP navigation read it, and its heading's citations count ([§FS-citations.recognized.title-lines](FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names)). It is one error at the declaration line:

```text
docs/FS-billing.md:1: error: `FS-billing` resolves for compatibility but does not match [id] format = "{kind}-{number}-{slug}" — rename it or change the effective format; this became an error in grund 0.16.0
```

The message names the key that sets the template as the `invalid ID` hint of [§FS-show.3.5](FS-show.md#35-the-hint-for-each-failure) does. It proposes no corrected ID and nothing rewrites the declaration, because only the author can choose between renaming it and changing the format ([§DF-off-grammar-declaration-compatibility](../decisions/functional/DF-off-grammar-declaration-compatibility.md#df-off-grammar-declaration-compatibility-persisted-declarations-remain-readable-without-relaxing-the-authoring-grammar)); no line-level opt-out exists. The token stops at a backtick, and a near miss is read only where a declaration would be, so inline code, Markdown prose and fenced blocks never hold one ([§DF-code-declarations-drop-hash](../decisions/functional/DF-code-declarations-drop-hash.md#df-code-declarations-drop-hash-code-resident-declarations-may-drop-the--prefix)).

### checks.broken-stub: Broken inline-spec stub

A stub ([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)) whose verdict holds no declaration of its ID
([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)) is an error at the stub. That is so when nothing is at the path its
link resolves to, when the target is a file the scan does not read, one an exclude or an
enabled ignore file keeps out among them ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)), and when the target, read
as [§FS-declarations.stubs.target](FS-declarations.md#stubstarget-what-a-stubs-target-gives-it) reads it, does not declare the ID. A heading of the ID inside a
fence of the target ([§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)) or a stub in it
([§FS-declarations.stubs.target.4](FS-declarations.md#stubstarget4-a-stub-in-the-target-declares-nothing)) declares nothing, so a chain of stubs is broken at its first
link.

#### checks.broken-stub.1: The target is read as a save would write it

Moved to [§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it).

#### checks.broken-stub.2: A heading inside a fence of the target declares nothing

Moved to [§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing).

#### checks.broken-stub.3: A target the scan does not read declares nothing

Moved to [§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing).

#### checks.broken-stub.4: Every command reads the one verdict

Every command reads each stub's verdict and the declaration count it gives ([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)). What they then agree on is [§REQ-one-answer.stubs.verdict](../requirements/REQ-one-answer.md#stubsverdict-every-command-reads-one-verdict-per-stub), [§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set), [§REQ-one-answer.stubs.outside-include](../requirements/REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned) and [§REQ-one-answer.text.targets](../requirements/REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would).

### checks.stub-shape: Stub shape read two ways (a spent check)

This check never fires, because a heading whose title is one link has one reading
([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like), [§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190)). `--only stub-shape` and
`--ignore stub-shape` are accepted and select nothing, and `--only stub-shape` prints the off
line of [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so).

### checks.excluded-stub-target: Stub to a target the walk excludes (a spent check)

This check never fires: a stub whose target the scan excludes, or an enabled ignore file
refuses, is a `broken-stub` error ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing),
[§DF-stub-target-exclusions](../decisions/functional/DF-stub-target-exclusions.md#df-stub-target-exclusions-excluded-and-ignored-stub-targets-are-broken-from-grund-0190)). The code is accepted by `--only` and `--ignore` and selects
nothing, so `--ignore excluded-stub-target` leaves that error standing, and
`--only excluded-stub-target` selects none of it and prints the off line of [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so),
which names `broken-stub`.

### checks.misplaced-declaration: Misplaced declaration (configured kind home)

A declaration line or stub line, never a citation or a prose mention, that stands where the configured kind homes do not allow it is one error at that line. Three placements are refused:

- a single-file kind's declaration outside its files: a kind whose places are all files, a v1 `file` ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) or a v2 row's `files` alone, is declared only in those, which the message lists comma-separated in written order. A stub is exempt, and this rule alone compares physical files, so a link to one of them from elsewhere is that file unless it sits in another kind's home;
- a declaration in a file that belongs to another kind's home, a `file` home holding that one path and a `folder` home the files below it;
- any declaration in a non-citable home ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), whose message names the place and says why, because no kind could be declared there instead.

```text
docs/notes.md:3: error: GOAL-draft must be declared in docs/goals.md (single-file kind)
docs/fs/FS-login.md:5: error: AR-router declares kind AR inside FS home docs/fs; move it to a home of its own kind
skills/review/SKILL.md:1: error: FS-002-review must not be declared in skills/ (not a citable home)
```

Each path that reaches a file is judged at its own path, a value source or Markdown file linked into another kind's home included ([§FS-scan.identity.findings.place](FS-scan.md#identityfindingsplace-a-place-finding-is-judged-at-each-spelling), [§FS-values.2.3](FS-values.md#23-duplicates-and-ownership)). In version 1, a file that no configured home holds, or that several hold because homes overlap or nest, belongs to the complement ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), so the other-kind and non-citable placements do not judge it, while a single-file kind's declaration there is refused as anywhere else. In version 2 every scanned file has one owner ([§FS-config-v2.schema.places.ownership](FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder), [§DF-v2-place-ownership](../decisions/functional/DF-v2-place-ownership.md#df-v2-place-ownership-in-v2-the-row-named-code-is-the-complement-and-a-file-belongs-to-its-listed-row-or-else-its-deepest-folder)): a declaration of `K` stands in a file `K` owns, or in one the complement owns unless `K` is a single-file kind, and the message names the owning place.

Beneath a nested project config that no `[workspace] members` entry lists, and that is therefore not read, the message says why the enclosing project's homes apply, and its repair is the row's `unlisted` case, with `roots` where the directory is itself a scan root, because an exclusion does not prune a root ([§FS-check.1.3.2](FS-check.md#132-the-wider-scan-reads-a-superset-each-file-once)):

```text
packages/notify/requirements.md:3: error: FS-email must be declared in requirements.md (single-file kind) — packages/notify/grund.toml (project notify) is not read: no [workspace] members entry lists packages/notify; kind homes from grund.toml (project shop) apply; add "packages/notify" to [workspace] members in grund.toml, or exclude "packages/notify/" with [schema.sources] exclude in grund.toml
```

The nested config is named as discovery finds it, `.agents/grund.toml` included ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), with its project name where that can be read. Its directory stays ordinary tree to the enclosing scan ([§FS-check.1.3.1](FS-check.md#131-the-scan-covers-the-whole-config-root)). A plain nested config, one with no `[workspace]` table, gets no finding of its own ([§FS-check.3.29](FS-check.md#329-unlisted-workspace-block)), and an unlisted nested `[workspace]` block is reported as [§FS-check.3.29](FS-check.md#329-unlisted-workspace-block) says. Where an ancestor's `members` or `optional_members` value cannot be read, the context is left out ([§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)).

### checks.duplicate-section: Duplicate section path

Two or more citable section headings in one declaration's body ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)) that claim the same section path, compared on canonical paths, so a relative heading and a complete-path heading, or two relative headings, collide too ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)). It is [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) one level down and in its shape, one error at the first heading in file order naming the others, because picking one of two destinations is the guess [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) forbids ([§DF-duplicate-section-path](../decisions/functional/DF-duplicate-section-path.md#df-duplicate-section-path-a-section-coordinate-names-one-heading-or-the-run-says-so)).

```text
docs/functional-spec/FS-001-login.md:5: error: duplicate section FS-001-login.1 (also declared at docs/functional-spec/FS-001-login.md:9); renumber or rename all but one of the headings
```

The same path under two declarations is two coordinates, and a heading past the body's end is no section of it, so neither is a finding. A front-matter key that is a section, written twice in a kind that declares a `front-matter` field, is this finding too, compared on its decoded name ([§FS-fields.front-matter.keys](FS-fields.md#front-matterkeys-its-keys-are-its-sections)). A stub declares no sections: the headings that count for its ID are those of the declaration it pairs with, judged and reported at their own lines in the target whether or not `[scan] include` reaches it ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)), once however many stubs point there. Two claimants are a duplicate in every `section_heading_levels` mode, and [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch) judges only the first ([§DF-duplicate-section-path.2.4](../decisions/functional/DF-duplicate-section-path.md#24-the-heading-level-rule-judges-only-the-heading-the-path-resolves-to)).

#### checks.duplicate-section.4: The same record `show` reads

That this rule and `show` read one section record is [§REQ-one-answer.sections.ambiguity](../requirements/REQ-one-answer.md#sectionsambiguity-check-and-show-judge-a-section-path-from-one-record).

### checks.orphan-section: Orphan name-bearing section path

A name-bearing section heading is an orphan when some proper prefix of its path is not recorded in the same declaration ([§FS-config.3.3.3](FS-config.md#333-every-prefix-of-a-name-bearing-path-is-recorded)), even where another heading visually contains it, and it is one error at its line; purely numeric paths are never judged:

```text
docs/functional-spec/FS-parents.md:13: error: orphan section FS-orphan.missing.x: missing prefix FS-orphan.missing; declare the prefix as a heading above it
```

It is independent of depth, so a heading with a missing prefix and a wrong depth has both findings, and of citations, so it is reported though nothing cites the orphan, and a citation of it resolves to it. A relative heading is never an orphan, because it is recorded only under a usable parent; one beneath an orphan is [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent) alone.

### checks.relative-section-parent: A relative heading without its parent

A well-formed relative heading ([§FS-declarations.section.relative.form](FS-declarations.md#sectionrelativeform-the-form)) whose parent is neither the declaration heading nor a usable section one level up ([§FS-declarations.section.relative.parent](FS-declarations.md#sectionrelativeparent-the-parent)) declares nothing, and is one error at its line:

```text
docs/functional-spec/FS-parents.md:5: error: relative heading #### .1 needs an H3 parent under FS-skip.format, and there is none; give the heading the parent it needs, or write its complete path
```

The heading is shown as its `#` run and relative component, `H` is followed by the level it needed, one less than its `#` count, and the coordinate is the nearest usable section or declaration heading above it that is shallower than that level, which always exists. Every reason a parent is not usable gives this one line, and a relative heading beneath a failed one fails the same way at its own line. A citation of the address the heading meant is a [§FS-check.3.2](FS-check.md#32-missing-section) finding.

### checks.section-heading-level: Section heading level mismatch

A citable section heading whose Markdown depth does not match its path, as `[id] section_heading_levels` ([§FS-config.3.3.2](FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)) or, in version 2, `[schema] heading_depth` ([§FS-config-v2.schema.1](FS-config-v2.md#schema1-schema)) requires, is one finding at the heading line:

```text
docs/functional-spec/FS-parents.md:79: error: section FS-mismatch.format.1 heading level mismatch: expected ### (level 3), found #### (level 4); re-level the heading, or change its path to match its depth
```

It judges only headings that carry a coordinate; a heading inside a body that carries none is [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) in every mode.

### checks.section-outside-declaration: Section outside a declaration

A numeric or enabled named section heading deeper than the last declaration heading the scan read, on a line no declaration's body holds ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), is one error at the heading, the first message for a numeric heading and the second for a named one:

```text
numbered section outside any declaration; move it inside a declaration's body, or drop its section path
named section outside any declaration; move it inside a declaration's body, or drop its section path
```

A plain same-or-higher heading ends a body as a declaration does, and a stub's body is its one line, so a numbered heading below a stub is outside. A heading inside a fence, in Markdown or in a comment block, is content and never this finding ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example), [§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)). The heading is not in the section map ([§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span)), so no command resolves it, and `show` answers a missing section rather than this message. `--full` reports it in the files it adds too ([§DF-check-full-scope](../decisions/functional/DF-check-full-scope.md#df-check-full-scope-check---full-walks-past-scan-include-and-reports-unresolved-references-plus-orphaned-section-headings-out-there)).

### checks.unmarked-heading: Unmarked Markdown heading

An ATX heading in a scanned Markdown file that lies inside a declaration's body, deeper than its heading, and is neither a declaration nor a numeric or enabled named section, written with its complete path or as a relative heading ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)), is one error at its line. It names the innermost declaration whose body holds it ([§FS-declarations.section.map.nested](FS-declarations.md#sectionmapnested-after-a-nested-declaration-the-enclosing-body-owns-the-heading)). A heading inside a fence is content and outside the rule ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)). A well-formed relative heading without a usable parent is [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent) instead, while a dotted heading not in the relative form ([§FS-declarations.section.relative.form](FS-declarations.md#sectionrelativeform-the-form)) is judged here. `--full` does not take the rule past the configured scope ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)). No command numbers the heading ([§DF-unmarked-markdown-headings](../decisions/functional/DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph)), and in the editor the finding's range is the complete authored ATX heading ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).

```text
docs/FS-heading-policy.md:5: error: unmarked heading inside FS-heading-policy; number it (### 1. Origin of the fails queues) as FS-heading-policy.1, declare an ID, or use a bold label; this became an error in grund 0.16.0
```

The suggested heading is the authored `#` run, the coordinate in its numeric or mixed heading form, and the authored title, or `Untitled` where there is none, so applying it declares a section and clears the error; a dotted heading is suggested this complete-path heading too. The coordinate has as many components as the heading sits levels below its declaration. The components before the last are its parent's: at each shallower level, the nearest preceding citable or already-suggested heading, and `.1` for a level the heading skips. The last is one above the largest numeric sibling that exists or was already suggested under that parent, the declaration where the heading sits one level below it, or `1` where there is none, so a named parent can get a numeric child such as `goals.1`; no hole is filled and no coordinate reused.

Clauses follow that text in this order, each where it applies, and at most one of the five path clauses:

```text
; under <parent coordinate> a relative heading is written <relative heading>
; kind <KIND> declares the field <name>, written <field heading>
; the section path <path> needs a title after it, written <shown heading>
; the section path <path> has a _, which a section name ([a-z][a-z0-9-]*) never takes, written <shown heading>
; the section path <path> has a _, which a section name ([a-z][a-z0-9-]*) never takes, and needs a title after it, written <shown heading>
; the section path <path> is numeric, so it takes . rather than :, written <shown heading>
; the section path <path> is numeric, so it takes . rather than :, and needs a title after it, written <shown heading>
; the chapter <NAME> that <RULE-ID> counts is written <named heading>
```

The relative and path clauses appear only where the heading they show would declare a recognized section, never a repair that trades this error for another; in a value declaration ([§FS-values.2.1](FS-values.md#21-markdown-declarations)) or under a value root ([§FS-values.2.4](FS-values.md#24-embedded-section-value-roots), [§FS-values.2.5](FS-values.md#25-chapter-declared-value-roots)), that is only the next numeric component the declaration or root expects at that line ([§FS-values.2.4.3](FS-values.md#243-the-component-run)).

- **Relative.** A heading whose text begins with `.` is shown the relative heading for the suggested coordinate, under that coordinate less its last component, which must be the declaration or a usable section ([§FS-declarations.section.relative.parent](FS-declarations.md#sectionrelativeparent-the-parent)). The relative heading is the authored `#` run, a space, `.` and the last component, the title syntax the suggested heading uses, and the authored title less its leading dotted token, its first run of non-space characters, or `Untitled` where nothing is left.
- **Field.** In a version-2 project, a heading one ATX level below its declaration's heading whose text equals a field's configured `title` or name, byte for byte, is shown the first such field in configuration order ([§FS-fields.declare.field](FS-fields.md#declarefield-a-field-is-a-table-under-its-kind)): the authored `#` run, a space, the field's name, `: `, and its title, or the heading text where it has none. Numbering that heading leaves the field missing; only the named form declares it ([§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name)).
- **Path.** A heading not beginning with `.` whose path is written in place but lacks a title (`### requirements.1`, `### requirements.3:`, `## 1.`), has a `_` in a named component, or puts a `:` after a purely numeric path (`## 1: Embedded`) gets the path clause naming its faults. `<path>` is the authored path without a trailing `:` or `.`, and `<shown heading>` is the authored `#` run, a space, that path with each `_` made `-`, then `: ` on a name-bearing path or `. ` on a numeric one, then the authored title, or `Untitled`. A lone component with nothing after it (`## background`) reads as a title, and a corrected name outside `[a-z][a-z0-9-]*` gets no clause. The shown path must have as many components as the heading sits levels below its declaration, a usable parent section where it has two or more, and named sections enabled where it is name-bearing, and be neither declared nor already shown in that body.
- **Chapter.** Where a chapter rule counts the heading in its named form, the clause shows that form ([§FS-rules.7.2.1](FS-rules.md#721-display-name-comparison-context)).

### checks.oversized-lead: Oversized lead *(opt-in)*

Where a project sets the key the row names ([§FS-config.3.1.2](FS-config.md#312-lead_size_warning--the-oversized-lead-opt-in); in version 2, [§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)), `check` measures each declaration and citable section lead by [§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements), and a lead strictly greater than the maximum is one finding at its heading line; equality passes:

```text
docs/local.md:1: warning: FS-local lead is 2 words, over the configured maximum of 1; move detail into citable child sections, or promote a child section to its own ID after running grund refs FS-local --summary
```

The message gives the lead's measure in the configured unit, `2 words` above, and the configured maximum, and its coordinate is local in a member's check and workspace-qualified in a workspace-root check. A broken stub has no lead and is not judged, and each claimant of a duplicate declaration or section is judged on its own slice. Selecting the code never turns it on, and `--only oversized-lead` without the key prints the off line of [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so). A check of a path judges only the sites at that path, `--full` judges no lead outside the configured scope, and in a workspace each member's own key governs its sites.

### checks.conflict-marker: Unresolved merge conflict

Each conflict block found where [§FS-declarations.body.conflict.where](FS-declarations.md#bodyconflictwhere-where-blocks-are-looked-for) looks is one error at the block's first line:

```text
docs/login.md:7: error: unresolved merge conflict in FS-login, lines 7–11: resolve it by hand
```

The owners are the IDs the block names ([§FS-declarations.body.conflict.owners](FS-declarations.md#bodyconflictowners-which-declarations-a-block-names)), each spelled as its own project spells it, in a workspace-root run too, joined by `, ` and a final ` and `, or `outside any declaration` where it names none; a block of one line reads `line 7` in place of `lines 7–11`. In JSON, `sites` holds the block's first and last lines, in that order, or its one line. The message names no side and no command, because which text is the declaration's is the author's call ([§FS-declarations.requirements.actionable](FS-declarations.md#requirementsactionable-a-finding-is-located-at-the-fault-and-no-command-makes-the-fix--realized-directional-for-the-next-check)). A declaration line or heading inside the block raises its own findings too, beside this one. `--ignore conflict-marker` unlocks nothing, because `fmt` reads the block from the model ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model), [§DF-conflicted-declaration](../decisions/functional/DF-conflicted-declaration.md#df-conflicted-declaration-a-declaration-that-still-holds-an-unresolved-merge-conflict-fails-check-and-stops-fmt-from-writing)).

### checks.parse-moves: Parse moves (not a code)

No check reports `parse-moves`, because every command reads one parse of each file
([§DF-one-parse-answers-from-0-19](../decisions/functional/DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves)), and no catalog row names it, so `--only parse-moves`
and `--ignore parse-moves` are refused as unknown codes ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).

## why: Why this exists

A declaration's address is what a repository stores about it in a thousand places at once,
so what may go wrong with a declaration is what may go wrong with an address. Gathering the
eleven checks here rather than under the command that reports them puts each beside the
invariant it defends, under the code a reader holds from the finding they have just read
([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)). `check` is one reader of this contract, and not the only one:
`show` refuses the coordinate this spec calls ambiguous, `fmt` rewrites toward the canonical
form it names, and the LSP transports every finding here unchanged, which is the other reason
the checks do not live under the command.

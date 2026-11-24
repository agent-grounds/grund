# FS-declarations: a declaration is addressable once, from one allowed place, and holds nothing that is neither a coordinate nor a finding

A declaration is the line that introduces an ID, and it is what every citation in a
grund repository addresses. One invariant makes that address worth storing: an ID is
declared exactly once, it is declared where its kind's home allows it, and every ATX heading
inside its body in a Markdown file is either a coordinate a citation can reach or a finding
`check` reports; source doc-comment and setext text stay outside that rule
([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)).
Which line introduces an ID at all is [§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares),
and which declarations a stub stands for is [§FS-declarations.stubs](FS-declarations.md#stubs-what-a-stub-points-at).
What a heading is and what its rendered text reads, where a body ends, what a section and
the section map are, and what cuts a lead are [§FS-declarations.heading](FS-declarations.md#heading-what-a-heading-is), [§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends),
[§FS-declarations.section](FS-declarations.md#section-what-a-section-is) and [§FS-declarations.lead](FS-declarations.md#lead-what-cuts-a-lead).
The checks below are that sentence enforced, save three, which hold what an address is worth
once it resolves: that the declaration is in the form its ID grammar gives it
([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)), that its lead, and every citable section
lead in its body, stays inside the size its project budgets where that project asks for
one ([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)), and that the text it addresses is not still in the
middle of a merge ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)). Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and
[§GOAL-token-economy.1](../goals.md#1-what-this-requires).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms)
(source declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, place, complement place, workspace, member),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, caution), and [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (rule).

- **marker line** — A line git writes around each side of a conflict it could not merge: column 0, exactly seven `<`, `|`, `=` or `>`, then a space or the end of the line ([§FS-declarations.body.conflict.markers](FS-declarations.md#bodyconflictmarkers-a-marker-line)).
- **conflict block** — The marker lines of one unresolved conflict, from its opening `<<<<<<<` to its closing `>>>>>>>`, with every line between them ([§FS-declarations.body.conflict.block](FS-declarations.md#bodyconflictblock-a-conflict-block)).

## requirements: What every declaration holds to

Two requirements on declarations as a whole, each carrying a mark of
[§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). The rest of what a declaration holds to is required elsewhere and
is not repeated: that an ID has one declaration and a section path one heading is
[§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which names the duplicate checks as its enforcement; that every
command reads one declaration set, one stub verdict and one section map, whatever the scope,
is [§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree); and that a check about a declaration is specified here under its code is
[§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code).

### requirements.mentions: Writing about an ID never declares it — realized, one case deferred

A line that names an ID without introducing it never declares that ID, wherever it sits. An
ID followed by a section path is a coordinate in every declaration position, and with the
declaration colon after it the whole token is the near miss, never a declaration of the
shorter ID ([§FS-declarations.line.section-suffix](FS-declarations.md#linesection-suffix-an-id-with-a-section-after-it-declares-nothing), [§FS-declarations.line.section-suffix.2](FS-declarations.md#linesection-suffix2-with-the-colon-the-whole-token-is-the-near-miss)); a
heading inside a fence or a raw-text HTML block is an example, and a `#` line in front matter is
a YAML or TOML comment ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)) ([§FS-declarations.heading](FS-declarations.md#heading-what-a-heading-is),
[§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)); and an ID in inline code or in Markdown prose
declares nothing and is no near miss ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)). One
case is owed. Every comment-prefixed line of a source file is a declaration position
([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)), so `// FS-x: …` inside a function body, a citation whose marker was
dropped, declares `FS-x` and makes the real declaration a duplicate. Seeing a source
declaration only in a doc-comment that documents the definition after it is **deferred** to
[§RM-doc-comment-declarations](../roadmap.md#rm-doc-comment-declarations-declarations-only-in-classmethod-doc-comments). Serves [§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) and [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration).

### requirements.actionable: A finding is located at the fault, and no command makes the fix — realized, directional for the next check

Every finding of [§FS-declarations.checks](FS-declarations.md#checks-checks) is located at the fault — the
declaration or heading in question, or the stub whose link is broken — never at a line with
nothing to fix: a declaration reached through stubs is named at its target, not at a stub
([§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target)), with every other site listed in the message
([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration), [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)). The message
states what is true and the choices the author has, and no command applies one: no corrected
ID is proposed ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)), a suggested coordinate is guidance `fmt`
never writes ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)), a non-citable home is named as a place rather
than mapped to a kind ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)), and an oversized
lead is answered with remedies that keep its citations
([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)). This binds every check added here from now on.
Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible), beyond the `path:line` that [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements)
already requires.

## line: What a line in declaration position declares

A line is in declaration position where a declaration could open: a Markdown heading, a
comment-prefixed line in a source file — behind a configured prefix or any form it stands for,
so `--- ID:`, `-- | ID:` and `;; ID:` as well as `-- ID:` and `; ID:`
([§FS-languages.prefixes.forms](FS-languages.md#prefixesforms-the-forms-a-prefix-stands-for)) — a bare line inside a Python docstring, or a line
inside a `/* … */` block comment, with or without its ` * ` gutter
([§FS-declarations.line.block-comment](FS-declarations.md#lineblock-comment-a-line-inside-a-block-comment), [§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)). The position is necessary and not sufficient. The token that opens the
line has to be the shape a declaration takes, and one shape that starts out like it is
not. A byte-order mark that starts a file is part of line 1, in Markdown and in a source
file alike, so line 1 declares nothing, and a block comment or docstring whose opener is on
line 1 is not opened, so nothing inside it declares. A comment line that follows line 1
still declares.

### line.block-comment: A line inside a block comment

A block comment opens on a line whose first non-blank characters are `/*`, `/**` or `/*!`,
and closes at the first `*/` after its opener, on the opener's line or a later one. The
block is the one [§FS-languages.comment-block.forms](FS-languages.md#comment-blockforms-comment-blocks) names, read where `/*` is among
`[scan] comment_prefixes` ([§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions)). Inside it, three forms of line are in
declaration position, once the wrapper of [§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper) is read off:

- the opener line, behind any of the three openers: `/* FS-b: Beta`, `/** FS-b: Beta`,
  `/*! FS-b: Beta`, and the one-line `/** FS-b: Beta */` or `/* FS-b: Beta */`, titled `Beta`: the title stops before the block's first `*/` and the whitespace in front of it, which are the comment's envelope ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line), [§DF-one-answer-rulings-parse](../decisions/functional/DF-one-answer-rulings-parse.md#df-one-answer-rulings-parse-one-parse-gives-every-command-the-same-extents-comment-blocks-anchors-and-title-citations) D17d),
  so `/** FS-b: Beta */ tail */` is titled `Beta` too;
- a continuation line behind ` * `: ` * FS-b: Beta`;
- a bare continuation line, with no ` * `, indented or not: `FS-b: Beta`. Only a block whose
  `*/` is found has bare lines in declaration position. In a block that runs to the end of
  the file without one, a bare line declares nothing, while the opener and starred forms
  read as they do in a closed block.

So the bare line declares exactly where its starred twin does, in any block comment and not
only a `/**` doc block, and the declaration is found by every command that reads the
catalog: `check`, `show`, `list`, `refs`, `cover` and the editor. A newly found declaration
draws every finding any other does, including `declaration-near-miss` (whose position is
this point's, [§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss), at the severity [§DF-bare-block-comment-line-declares.4](../decisions/functional/DF-bare-block-comment-line-declares.md#4-consequences) settles), duplicates, homes, indexes
and rules. Its body, sections and lead are its comment block's
([§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)), with bare lines kept as written, so a bare `## 1. Sub` under
it is its section `.1`. Decided in [§DF-bare-block-comment-line-declares](../decisions/functional/DF-bare-block-comment-line-declares.md#df-bare-block-comment-line-declares-a-line-inside-a-block-comment-declares-with-or-without-its-gutter-behind-any-of-its-openers).

Nothing else is inside a block, so none of these lines is in declaration position by this
point:

- a line after the block's `*/`;
- a line after a `/*` that is not at the start of its line, such as `x = 1; /* note */` or
  `let s = "/*";`;
- a line after a `//` comment that contains `/*`, such as `// see /* here`.

This is recognition, not parsing: a line that starts with `/*` inside a
string literal spanning several lines opens a block all the same.

### line.configured-literals: Configured literals belong to the canonical ID

A literal in the effective ID format belongs to the ID, including `:`
([§FS-config.3.2](FS-config.md#32-id--id-grammar)). Under
`format = "{kind}:{slug}"`, `# FS:login: Login` declares the canonical ID
`FS:login` with title `Login`: the first colon is part of the ID, and the colon
after the complete ID separates its title. This holds in every declaration
position, including source comments and Python docstrings. A shorter token
ending at an internal colon must not turn a conforming declaration into an
off-grammar declaration or a `declaration-near-miss` finding.

Discovery and body rereading retain that complete canonical identity, so the
ID printed by `list` resolves through bare and explicit `show`, both for its
lead and for its sections
([§FS-show.1.1](FS-show.md#11-id), [§FS-list.2](FS-list.md#2-behaviour)). For
`# FS:login: Login` followed by a blank line and `Lead.`, both `grund FS:login`
and `grund show FS:login` print `Lead.` and exit `0`. This does not accept a
canonical prefix in place of a longer off-grammar token
([§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable)),
or turn a section-suffixed coordinate into a canonical declaration
([§FS-declarations.line.section-suffix](FS-declarations.md#linesection-suffix-an-id-with-a-section-after-it-declares-nothing)).

### line.configured-slug: Characters admitted by the slug pattern belong to the canonical ID

The effective `slug_pattern` determines the slug's characters
([§FS-config.3.2](FS-config.md#32-id--id-grammar)). A complete token that
matches that grammar is a canonical declaration even when its final character
is non-word punctuation. Under `format = "{kind}-{slug}"` and
`slug_pattern = "[a-z*][a-z0-9*-]*"`, `# FS-*: Literal star` and
`# FS-tail*: Trailing star` declare `FS-*` and `FS-tail*`, respectively;
the `*` is part of the slug. This holds in Markdown headings, source comments
and bare declaration lines inside Python docstrings.

Discovery and body rereading retain the same complete canonical identity.
`list` prints the declaration and `cover` records its marked citations; bare
and explicit `show` print its actual lead
([§FS-show.1.1](FS-show.md#11-id)). `refs` lists its citation sites; with no
citers it exits `0` without the note that the operand is neither declared nor
cited ([§FS-refs.2.1](FS-refs.md#21-an-id-with-no-citations)). With valid Markdown-link
index entries and no other findings, `check` prints `success` and exits `0`:
the canonical declaration is no `declaration-near-miss`, its citations are
known references, and its index links resolve
([§FS-check.3.1](FS-check.md#31-dangling-citation),
[§REQ-no-wrong-citation.2](../requirements/REQ-no-wrong-citation.md#2-no-false-alarms)).

The whole token must match. An invalid extension such as `FS-tail*!` is not
a canonical declaration of `FS-tail*`; a colon-terminated off-grammar token
keeps its complete spelling under
[§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable).
A section-coordinate heading such as `# FS-tail*.2 names a section` declares
nothing under
[§FS-declarations.line.section-suffix](FS-declarations.md#linesection-suffix-an-id-with-a-section-after-it-declares-nothing).

### line.section-suffix: An ID with a section after it declares nothing

A line in declaration position whose first token is an ID followed directly by the
project's section separator and a section path declares nothing: not that ID, and no
other. `FS-042-user-login.2` names section 2 of `FS-042-user-login`. That is the shape of
a coordinate ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)), never of a declaration, so none of these three lines
declares `FS-042-user-login` titled `.2 …`:

```python
def login():
    """A note about the login spec.

    FS-042-user-login.2 is named here with a section suffix and no colon.
    """
```

```rust
fn inside_a_function_body() {
    // FS-042-user-login.2 / a citation note whose marker was dropped
}
```

```markdown
# FS-042-user-login.2 and what a session must outlive
```

Each of them used to, and the false declaration stayed silent until something collided
with it. The real declaration then failed as a duplicate, and every query of it as an
ambiguous ID, naming a file whose author had only written *about* the point. Where
nothing collided, an ID that no specification declares resolved, because a sentence about
one of its sections had declared it. Such a line is prose, or a citation where it carries
the marker ([§FS-check.1.1](FS-check.md#11-recognized-citations)). A line whose first token is an ID with nothing directly after
it but the colon, whitespace or the end of the line is outside this point.

#### line.section-suffix.1: Colon-less, it is no near miss either

Without the declaration colon the line raises no `declaration-near-miss`: that rule reads
only a token followed by the colon ([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)), and
this line has none. `check` reports nothing about it.

#### line.section-suffix.2: With the colon, the whole token is the near miss

`# FS-042-user-login.2: the session` opens the way a declaration does, and its token ends
at the declaration colon, so the token is all of `FS-042-user-login.2`, section included.
The ID grammar rejects that token, which makes the line the off-grammar declaration
[§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable) retains under its exact spelling and
[§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss) reports. It is never a declaration of
`FS-042-user-login`.

#### line.section-suffix.3: The separator and the sections are the project's

The separator is the configured `[id] section_separator` ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)), not a literal
`.`. Under `section_separator = ":"`, `FS-042-user-login:2 is named here` declares
nothing, and it is no near miss either: its `:` is the separator in front of a section,
not a declaration colon in front of a title. The section is whatever the project's
section grammar admits: a numbered path always, and a named path such as
`FS-042-user-login.goals` only where `named_sections = true` ([§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names)).

## stubs: What a stub points at

A stub is a pointer ([§FS-terms.terms.3](FS-terms.md#terms3-source-forms)): a heading whose title is one link
([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)) to a file, its target, and it points at the declarations of its
ID there. What the target gives the stub is [§FS-declarations.stubs.target](FS-declarations.md#stubstarget-what-a-stubs-target-gives-it), how the stub pairs
with what it points at is [§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at), and which declarations those are, if
any, is the stub's verdict ([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)). A stub paired with the
declarations it points at is not counted beside them as one more
([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)).

### stubs.verdict: One verdict per stub, reached once per scan

Each stub's verdict is reached once per scan, and it is one of five: the target is missing; the target is a file the scan does not read ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)); the target, read as the scan reads it on the text a save would write ([§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it), [§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)), does not declare the ID; it declares the ID, on one line or more, and the stub points at each of those declarations ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)); or the target is the file the stub sits in, which declares the ID on another line, and the stub points at none of those declarations but is one of its own beside them, so the ID is a duplicate ([§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)). The first three are a broken stub ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)); a stub linked to its own file that declares the ID nowhere else is the third. The ID's declarations are counted from those verdicts, once its stubs are paired.

### stubs.repair: A missing target beside another declaration explains the repair

When a stub's recorded verdict is missing and the ID has other independent declaration
homes, diagnostics connect those facts. This is guidance from the declarations and verdicts
already found by the scan, never an inference that a file moved or a second filesystem search
([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)).

`check` retains both the duplicate finding and the missing-target `broken-stub` finding,
with each complete existing message as a contiguous prefix. The duplicate finding appends
an explanation identifying each missing-target broken stub by `path:line` and its missing
authored target, followed by the available real declaration sites. With exactly one real
home, it says that the ID is declared there. It tells the reader to point a missing stub's
link there only when that home is in a different file from that stub. A home in the stub's
own file remains a declaration fact, but the guidance explains that linking to the same
file will not pair the stub, under the existing own-file verdict
([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)).
When several missing stubs share the ID, each instruction must apply only to stubs in a
different file from the home; no blanket instruction may include a same-file stub.
A real home is a declaration that is not itself a stub, including a target
represented by healthy stubs
([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned));
other broken stubs are never repair destinations. Several real homes are named in deterministic
`path:line` order without selecting one or telling the reader to point the stub at one.
The missing-target finding appends a site-free connection: it states that the ID has one
real declaration home and the safe action for this stub, or explains that the sole home is
in this stub's own file and linking to the same file will not pair it, or states that there
are several real homes without selecting one,
and identifies the duplicate declaration finding as the source of their locations. It
names no additional candidate path or line. The missing authored target is the existing
missing-target fact, not a declaration site. With no real home, the ordinary missing-target
check message remains.

Each finding's message, location and sites are independent of check selection:
`--only duplicate` explains the located repair itself; `--only broken-stub` identifies the companion
duplicate declaration finding without assuming it is visible or saying it is above or below.

ID queries, including bare IDs, `--toc`, section IDs, and `refs`, retain exit `1` and their
complete `ambiguous ID: <ID> (declared at <sites>)` prefix. They append an explanation
identifying each missing-target broken stub and its missing link target, followed by the
same located explanation and safe repair guidance as the duplicate finding. The original
declaration-site list and its order are unchanged, even when a real home sorts before the stub. A broken stub
remains its own declaration: guidance neither pairs it nor makes an ambiguous query succeed.

Text and JSON carry the same message. Duplicate and query `sites` remain exactly their
original ordered declaration-site lists, without annotations or extra entries
([§FS-errors.5.2](FS-errors.md#52-on-stderr--what-is-not-output)). The duplicate finding
stays at its original first site, even when a real home sorts before the broken stub
([§FS-errors.4.1](FS-errors.md#41-ordering)). The missing-target finding stays located at
the faulty stub and retains `sites:null` ([§FS-errors.5.1](FS-errors.md#51-on-stdout--the-commands-output)).
Query path and line remain null. Codes, severities, channels, exits, authority, finding
counts, and all field names and types remain unchanged. Only a recorded missing-target
verdict gets this guidance: healthy, excluded, unread, and read-but-lacking-ID targets keep
their own explanations and remedies
([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)).

### stubs.shape: What a stub looks like

A stub is a heading whose title is one link, `<ID>: [<text>](<path>)`, at any heading level,
in a Markdown file of some kind's home ([§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates)): a file under a kind's `folder`,
or a kind's `file`, citable or not, however many homes hold it. Whether a file is in a home is
read from its path below the config root, so where the checkout sits never moves the answer.
The complement place is not a home ([§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure)), so a heading of that shape there is a
declaration titled with a link, and so is one in `docs/notes/y.md` where no kind's home holds
`docs/notes/`. No release has required a stub to be an H1: `## FS-x: [x](../../src/x.rs)` in
`docs/fs/x.md` is a stub.

Before grund 0.19.0 a heading of that shape was a stub wherever a directory named `docs` was on
its whole path; 0.18.0 warned where the readings differ ([§FS-declarations.checks.stub-shape](FS-declarations.md#checksstub-shape-stub-shape-read-two-ways-a-spent-check)),
and [§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190) removed that test.

### stubs.target: What a stub's target gives it

The link resolves as a Markdown link does, relative to the stub file's directory, so `lychee`
and rendered docs see the same target. Where nothing is at that path, `grund` falls back to
the path relative to the config root, for older stubs that wrote repo-root paths. The target
declares the ID exactly as the scanner's own pass reads that file, as if it were scanned
(D3 of [§DF-one-answer-rulings-stubs](../decisions/functional/DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is)): on the text a save would write
([§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it)), with a fence's headings declaring nothing
([§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)), only where the scan would read the file at all
([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)), and never through a stub of its own
([§FS-declarations.stubs.target.4](FS-declarations.md#stubstarget4-a-stub-in-the-target-declares-nothing)). What a target outside `[scan] include` gives the stub, and
what of it stays unjudged, is [§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged).

#### stubs.target.1: The target is read as a save would write it

Whether the file at `<path>` contains a source declaration of the ID is judged on the target's text as [§FS-scan.text.buffer](FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write) defines it, the text a save would write. That is the text the scan reads any file from and the text `show` slices the stub's body out of ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments), [§REQ-one-answer.text.targets](../requirements/REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would)). The path the stub wrote reads a buffer the editor opened under any spelling of the same file ([§FS-scan.identity.buffers](FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location)), and a target open only in a buffer, with no file saved behind it, is not missing: it is judged on the buffer, so it is healthy where the buffer declares the ID and `stub link target lacks <ID>: <path>` where it does not, as the saved file would be ([§FS-scan.text.buffer-only](FS-scan.md#textbuffer-only-a-buffer-with-no-file-behind-it-is-a-file)).

So an unsaved edit that removes the target's declaration breaks the stub at once. `check` reports `broken-stub` at the stub line, and the editor shows it as the user edits ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)). `show` refuses with the second line of [§FS-show.2.3.4](FS-show.md#234-broken-stub), never `ID not found`. An unsaved edit that puts the declaration back repairs the stub just as soon: `show` reads the body from the edited text, and the error goes. Scan scope plays no part, as it plays none in finding the body ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)).

#### stubs.target.2: A heading inside a fence of the target declares nothing

The target's text, the one [§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it) names, is read for the ID the way the scan reads that file. In a Markdown target a heading inside a fenced code block is an example, not a declaration ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)), and the fence is the one [§FS-languages.fence](FS-languages.md#fence-fences) defines: a fence delimiter line, and every line while a fence is open, declares nothing, however much it looks like `# <ID>: …`. A line inside a raw-text HTML block such as `<pre>` ([§FS-check.1.1.5.1](FS-check.md#1151-a-raw-html-block-holds-no-heading)) declares nothing in the same way. A target whose only heading of the ID sits inside a fence or such a block therefore contains no declaration of it, and the stub is broken exactly as if the target never named the ID, whether or not the target is inside `[scan] include`: `check` reports `stub link target lacks <ID>: <path>` at the stub's line, and `show` refuses with the second line of [§FS-show.2.3.4](FS-show.md#234-broken-stub) rather than answering `ID not found`. A target that holds a fenced example of the heading and the real declaration outside every fence declares the ID once, at the real heading, and the stub is healthy. In a source target a declaration line inside a doc-comment's fence is read as it would be outside it: such a fence hides only section and plain headings ([§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)), and this test reads the target the same way. Why a stub that used to pass now fails is [§DF-stub-target-fenced-heading](../decisions/functional/DF-stub-target-fenced-heading.md#df-stub-target-fenced-heading-a-heading-inside-a-fence-of-a-stubs-target-does-not-declare-its-id).

#### stubs.target.3: A target the scan does not read declares nothing

The target is read for the ID the way the scan reads that file ([§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)), and some files the scan never reads, wherever they sit: one whose own name begins with `.`, skipped by that name before `extensions` is consulted ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), and one whose extension `[scan] extensions` does not list ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)), or, in a version-2 file, one no listed language reads or one a listed `data` language reads, which only a value kind's value pass reads ([§FS-values.2.2](FS-values.md#22-declarations-from-value-sources)). Such a target declares nothing, whatever it holds. A `# <ID>: …` heading in `notes/.a.md`, or in `notes/a.zz` where `zz` is not listed, is text the scan does not read, so the stub is broken exactly as if the target never named the ID.

Name and extension refusals apply whether or not the target lies inside `[scan] include`. The name judged is the one the stub wrote, not the one a symlink resolves to: `notes/.a.md -> a.md` declares nothing even where the `notes/a.md` it resolves to is scanned and declares the ID, so the stub pairs with neither and is a declaration of its own beside that one, the duplicate a regular hidden copy of `notes/a.md` makes. The same holds for `notes/a.zz -> a.md` where `zz` is not listed. That a target outside `include` reads as if it were scanned ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)) reaches only a file the scan would read were it in scope.

When the target is refused because the configuration reads no file by its name's extension,
`check` reports `stub link target <path> is not scanned: <reason>`, whose `<reason>` is the
cell of the reason table in [§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read), with no remedy clause;
a target a listed `data` language reads takes that table's data row, so `notes/a.json` under `languages = ["markdown", "json"]` says
``stub link target notes/a.json is not scanned: the scan reads `.json` as `json`, a data language only a value kind reads``.
The diagnostic names the target the stub wrote; it never claims that this unread target lacks the ID.
It remains an error coded `broken-stub` at the stub's line, and `check` exits `1`.
This explanation applies inside and outside `[scan] include` alike, with the serialized
finding's fields unchanged. The hidden-name refusal still takes precedence over the
extension refusal and keeps `stub link target lacks <ID>: <path>`, as does a target
that is read and actually lacks the ID. Listing the extension in `[scan] extensions` under version 1, mapping it to a listed language or listing a language that reads it under version 2 ([§FS-config-v2.schema.sources.extensions](FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)), or renaming the target to an extension the configuration reads, the one repair for a target with none,
lets the target be read, and a target declaring the ID then makes the stub healthy.

`show` still refuses with the second line of [§FS-show.2.3.4](FS-show.md#234-broken-stub) rather than answering a body sliced out of a file the scan does not read. A stub broken this way pairs with nothing ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so `refs` refuses nothing from its target, however many headings of the ID the target holds ([§FS-refs.4.1](FS-refs.md#41-a-stubs-homes-are-found-in-its-target)). The remedy is the target's: a name that does not begin with `.`, and an extension the configuration reads.

From grund 0.19.0 two more rules refuse a target: a directory on its path that `[scan] exclude` names ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)), and an enabled ignore rule ([§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)). A v2 exclusion pattern has the same effect ([§FS-config-v2.schema.sources.exclude.stubs](FS-config-v2.md#schemasourcesexcludestubs-a-pattern-is-the-exclude-reason-a-stubs-target-is-not-read)). Such a target declares nothing: its stub is `broken-stub`, an error at the stub's line, and `check` exits `1`. The refusal is recorded before pairing or lending any target content. This activates D5 of [§DF-one-answer-rulings-run.decision](../decisions/functional/DF-one-answer-rulings-run.md#decision-decision) after the warning release ([§DF-stub-target-exclusions](../decisions/functional/DF-stub-target-exclusions.md#df-stub-target-exclusions-excluded-and-ignored-stub-targets-are-broken-from-grund-0190)).

Both rules are asked as the walk asks them, from each scan root the run walks and from the config root in place of `include`. They match the target's lexical link spelling, as the walk's directory rules do ([§FS-scan.scope.links.names](FS-scan.md#scopelinksnames-the-directory-rules-apply-under-the-link-name)); two aliases of one physical file can therefore have different eligibility verdicts, while its content is still read once. A scan root under an excluded or ignored name outruns the descent rule here as in the walk ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)). Name and extension refusals take precedence; otherwise the first rule met descending refuses, with exclude before ignore at the same step. `respect_gitignore = false` disables ignore only. Nested ignore files, negations, git-info and global ignore rules keep the walk's semantics. `--full` still honors these filters; path narrowing changes reporting only. Lying outside `include` alone is no reason. This activation adds no refusal for hidden ancestors, another project's boundary, an unwalked home or a target outside the config root. Formatter exclusions keep their separate role.

The error uses `stub link target <path> is not scanned: <reason>; this became an error in grund 0.19.0 — <remedy>`. The path is the destination as written. For v1 exclude the reason is `[scan] exclude names <entry>` and the remedy is `move the target out of <entry>, or take <entry> out of [scan] exclude`. For v2 the reason is `[schema.sources] exclude matches it by <pattern>` and the remedy is `move the target out of what <pattern> matches, or take <pattern> out of [schema.sources] exclude`. For ignore rules the reason is `<file> ignores it by <pattern>` and the remedy is `move the target out of what <file> ignores, or stop ignoring it there`. Entries and patterns are as written; an ignore file is spelled as the report spells a path, from the report base ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)), and absolute where it lies outside the loaded root. Existing JSON fields stay unchanged. These errors replace the spent warning ([§FS-declarations.checks.excluded-stub-target](FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-a-spent-check)).

#### stubs.target.4: A stub in the target declares nothing

A heading in the target that is itself a stub ([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)) declares
nothing there: it is a pointer, not a declaration. A stub whose target holds only a stub of
its ID is therefore broken, `stub link target lacks <ID>: <path>` at its line, exactly as if
the target never named the ID, so a chain of stubs never reaches the declaration at its end.
`docs/fs/a.md` holding `# FS-x: [x](b.md)` beside `docs/fs/b.md` holding
`# FS-x: [x](../../src/x.rs)` is broken at `docs/fs/a.md:1`, and, being broken, is a
declaration of its own beside `src/x.rs:1`, so `FS-x` is a duplicate. Which headings in the
target are stubs is the shape test of the release, read on the target's own path. Every
command gives this answer (D13 of [§DF-one-answer-rulings-stubs](../decisions/functional/DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is)).

#### stubs.target.5: What a target outside the include gives, and what stays unjudged

A target outside `[scan] include` gives the stub what every command resolves through: its
declarations of the ID, counted as [§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned) counts them; their
sections, so a citation into one reads as it would were the target scanned
([§FS-check.3.2.1](FS-check.md#321-a-stubs-sections-are-its-targets-scanned-or-not)); every section path one of those declarations claims twice, which
`check` reports as `duplicate-section` ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)) at the
target's headings, `<path>:<line>` exactly as the same tree with the target scanned reports
them, whether or not anything cites the section
([§DF-stub-target-section-declared-twice](../decisions/functional/DF-stub-target-section-declared-twice.md#df-stub-target-section-declared-twice-a-section-a-stubs-target-declares-twice-is-reported-where-it-is-written-scanned-or-not)); and every conflict block in the target, which `check` reports
as `conflict-marker` ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)) at the target's lines,
with the owners the target's own pass gives it, as the same tree with the target scanned
reports it, and on which `fmt` writes nothing ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)). A section the target does not declare is reported
as before, in the same words, remedy included ([§FS-errors.3.11](FS-errors.md#311-the-missing-section-remedy)), and under
the same `missing-section` code. A broken stub ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)) pairs
with nothing and gains no section: it keeps its broken-stub finding, and a citation of one of
its sections is `missing section` as before. A target that declares the ID more than once
pairs as `show` refuses it, ambiguously ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)), and lends no section, and so does
every target of an ID with more than one declaration ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)).

Every reader that enumerates a declaration's sections reads those sections from the paired
declaration, as the same tree's readers do with the target scanned: `show --batch --all`
generates a query for each ([§FS-show.2.6](FS-show.md#26-batch-resolution)), shell completion offers each
([§FS-completions.2.1](FS-completions.md#21-arguments-and-candidate-modes)), the editor's snapshot holds each for completion and navigation
([§FS-lsp.1.6](FS-lsp.md#16-declared-id-completion), [§FS-lsp.1.3](FS-lsp.md#13-go-to-definition)), and `list --selector` selects a chapter among them
([§FS-rules.8](FS-rules.md#8-command-surfaces)). A section path the target declares twice is one generated query, which
`show --batch --all` answers with the failed record of an ambiguous section
([§FS-show.2.6](FS-show.md#26-batch-resolution)), so the batch exits `1` on the tree `check` already fails. Before grund
0.19.0 those four readers saw none of an outside target's sections
([§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190)).

It also gives every verdict those declarations' content feeds, exactly as the same tree with
the target scanned gives it. Each citation the scanner's pass over the target assigns to one
of those declarations, the citations in its body, is that declaration's citation: once it
resolves, through the shorthand and the declaration-local section citation as a scanned
citation does ([§FS-check.1.2](FS-check.md#12-the-number-only-shorthand), [§FS-check.1.1.8](FS-check.md#118-declaration-local-numeric-section-candidates)), it is an edge that counts toward a rule's
obligation, prohibition and counts ([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)) and toward a `[citations]` obligation and
prohibition ([§FS-check.3.11](FS-check.md#311-missing-required-citation), [§FS-check.3.12](FS-check.md#312-forbidden-citation)), `refs` lists it ([§FS-refs.2](FS-refs.md#2-behaviour)), and the ID it
cites is not `unused` ([§FS-check.4.1](FS-check.md#41-unused-declaration)). Each of those declarations' chapters, cited or not, is
a rule's subject and is counted by a chapter rule ([§FS-rules.2.1](FS-rules.md#21-a-chapters-name-is-its-whole-path), [§FS-rules.3.1](FS-rules.md#31-chapter-presence)). Stubs to one
target lend its citations and chapters once between them
([§FS-declarations.stubs.pairing.2](FS-declarations.md#stubspairing2-stubs-to-one-target-stand-for-its-declarations-once)), and a target that declares the ID twice lends each
declaration's own, as scanned. A broken stub pairs with nothing and lends nothing.

A finding about one of those citations is reported at the citation's line in the target, where
the scanned tree reports it: `src/x.rs:3: … FS must not cite GOAL …`. A finding about the
declaration itself, a count it fails, a chapter count or `unused`, is reported at the
declaration's line in the target ([§FS-declarations.stubs.pairing.4](FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target)). Each is an error or a
warning exactly as the same finding is in the scanned tree: from grund 0.19.0 nothing the
target lends is held on the warnings channel ([§FS-rules.7.11](FS-rules.md#711-a-citation-or-chapter-read-through-a-stub-warns-until-0190)).

Nothing else of the target is judged. Findings about the target file itself — its other
declarations, its notes and formatting, its leads, whether it is grounded, and whether its
citations resolve — are not reported: the target is read on the stub's behalf, not scanned (D8
of [§DF-one-answer-rulings-stubs](../decisions/functional/DF-one-answer-rulings-stubs.md#df-one-answer-rulings-stubs-a-stub-is-found-by-its-home-and-shape-read-as-the-scan-reads-and-shown-where-its-target-is)). A citation of the target that does not resolve — an unknown
ID, a section the cited declaration does not have, a shorthand that matches no declaration or
several — counts for nothing, as in the scanned tree, where it is never an edge
([§FS-rules.5.1](FS-rules.md#51-facts-and-identity)), and it is reported nowhere. A citation the pass assigns to no declaration of
the ID, in a note outside its body or in another declaration, is the target file's and is not
lent, and neither is another declaration's chapter. Decided in
[§DF-stub-sections-from-unscanned-target](../decisions/functional/DF-stub-sections-from-unscanned-target.md#df-stub-sections-from-unscanned-target-a-stubs-sections-are-its-targets-whether-or-not-the-scan-reaches-it), for a section the target declares twice in
[§DF-stub-target-section-declared-twice](../decisions/functional/DF-stub-target-section-declared-twice.md#df-stub-target-section-declared-twice-a-section-a-stubs-target-declares-twice-is-reported-where-it-is-written-scanned-or-not), and for the citations and chapters in
[§DF-stub-target-citations-and-chapters](../decisions/functional/DF-stub-target-citations-and-chapters.md#df-stub-target-citations-and-chapters-a-stubs-target-lends-the-stubbed-declaration-its-citations-and-chapters-and-what-that-newly-raises-warns-until-0190).

A target outside `include` is still asked the rules of [§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing):
`other/gen/a.md`, outside `include = ["docs"]` and under a `[scan] exclude`
entry `gen`, is broken and lends no body, sections, chapters or citation edges,
even if its text declares the ID. The same holds for an enabled ignore refusal.
Show refuses the body, list/size marks the broken stub, fmt links nothing through
it, and editor/completion gains none of the target's sections. `refs` can still
list citations to the stub's ID ([§FS-refs.4.1](FS-refs.md#41-a-stubs-homes-are-found-in-its-target)). Only `include` itself is never a reason.

### stubs.pairing: How a stub pairs with what it points at

A stub whose target declares its ID is the pointer to those declarations, never one more
beside them ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)); stubs to one target stand for its
declarations once between them ([§FS-declarations.stubs.pairing.2](FS-declarations.md#stubspairing2-stubs-to-one-target-stand-for-its-declarations-once)); a declaration reached
through stubs is named at its target ([§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target)); and a surface that
shows where it is shows it there ([§FS-declarations.stubs.pairing.4](FS-declarations.md#stubspairing4-every-surface-shows-a-paired-declaration-at-its-target)). Decided in
[§DF-stub-pairs-with-unscanned-target](../decisions/functional/DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it), and for a target that declares the ID twice in
[§DF-stub-target-declared-twice](../decisions/functional/DF-stub-target-declared-twice.md#df-stub-target-declared-twice-a-stubs-target-that-declares-its-id-twice-is-two-homes-scanned-or-not).

#### stubs.pairing.1: A stub pairs with its target whether or not the target is scanned

A stub whose target declares its ID — the stub [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) accepts — is the pointer to that declaration, not a second declaration of the ID, whether or not `[scan] include` reaches the target ([§REQ-one-answer.stubs.outside-include](../requirements/REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned)). The ID's declarations are counted off each stub's verdict ([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)), on the target's text as [§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it) reads it, so an unsaved edit to the target moves the count as saving it would ([§REQ-one-answer.text.targets](../requirements/REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would)). A lone stub to an unscanned target still stands for the declaration it points at, so its ID has one declaration, never none. It stands for every declaration of the ID in its target that is not itself a stub, each read as the scan would read it were the target scanned, so a target that declares the ID twice is two declarations and the ID is a duplicate, lone stub or not, named at both lines ([§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target)) as the same tree with the target scanned names them. A broken stub pairs with nothing and stays a declaration of its own, scanned target or not.

#### stubs.pairing.2: Stubs to one target stand for its declarations once

Stubs of one ID whose links resolve to the same target file stand for its declaration once between them, not once each: two pointers to one declaration do not make it two. Where that file declares the ID twice they stand for both declarations, once, as a lone stub does ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), never once per stub. Stubs whose links resolve to different files, each declaring the ID, stand for at least as many declarations as there are files, and the ID is a duplicate. A surface that shows such a declaration at a stub rather than at its target — `list`, where the target is outside the scan — shows it once, at the first of its stubs in `path:line` order, and once for a target that declares the ID twice too, a row that carries the note every line of a duplicated ID carries ([§FS-list.3.1.1](FS-list.md#311-row-notes)).

#### stubs.pairing.3: A declaration reached through stubs is named at its target

Where a duplicate error or an ambiguity refusal ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)) lists a declaration that stubs point at, the site it names is the target's declaration of the ID, `<path>:<line>`, exactly as it would be were the target scanned — never a stub's own line, which has nothing to fix. A target that declares the ID twice is named at each of its declarations, never at its first alone. The sites sort, and the error is located, per [§FS-check.2.1](FS-check.md#21-report-format) over those names, so the first site may be in the target file. The same sites travel in the finding's JSON `sites` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### stubs.pairing.4: Every surface shows a paired declaration at its target

A surface that says where a paired declaration is names the target's declaration, its
`<path>:<line>` and its title, as the same tree with the target scanned names it: `show`
([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)), plain `list` ([§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical)), `list --size` ([§FS-list.3.4.6](FS-list.md#346-a-home-outside-the-scan)), the duplicate
error and the ambiguity refusal ([§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target)), and every finding about the
declaration itself — the `unused` warning ([§FS-check.4.1](FS-check.md#41-unused-declaration)), and a count or chapter count it
fails ([§FS-rules.7.2](FS-rules.md#72-chapter-cardinality), [§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality), [§FS-rules.7.4](FS-rules.md#74-inbound-citation-cardinality)), whatever of the target's citations and
chapters the count reads ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)).

Plain `list` prints one row for each declaration of the ID the target holds, with the
target's title, once for all its stubs as [§FS-declarations.stubs.pairing.2](FS-declarations.md#stubspairing2-stubs-to-one-target-stand-for-its-declarations-once) says:
`FS-x  src/x.rs:1  X`. A target that declares the ID twice prints two rows, each marked as
[§FS-list.2.6](FS-list.md#26-duplicate-declarations) marks a duplicate. `check` reports a finding about the declaration at the
declaration's line in the target: `src/x.rs:1: warning: declared but never cited: FS-x; cite it`.

Before grund 0.19.0 plain `list` printed `FS-x  docs/fs/x.md:1  → ../../src/x.rs` and `check`
placed those findings at the stub; [§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190) moved both.

## heading: What a heading is

A heading is a Markdown ATX heading line (`#` through `######`) outside any fenced block,
raw-text HTML block or front matter ([§FS-check.1.1.5.2](FS-check.md#1152-front-matter-holds-no-heading)): a line inside one is never a heading, by [§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example), and in
a comment block the fence is the one read through the comment wrapper
([§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)). A heading
either opens a declaration ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares)), is a citable section of the
declaration whose body holds it ([§FS-declarations.section](FS-declarations.md#section-what-a-section-is)), or is plain prose. In a source
file the comment-prefixed declaration line plays a heading's part ([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)).

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
stub's single declaration line each end ownership. The one line the span does not reach that
a declaration still owns is the file's front matter, which belongs to its first declaration
without growing its body ([§FS-check.1.1.5.2.1](FS-check.md#11521-its-citations-belong-to-the-files-first-declaration)). In a kind that declares a `front-matter`
field, that block is also the declaration's front-matter sections, which lie outside the body
([§FS-fields.front-matter.coordinates](FS-fields.md#front-mattercoordinates-its-coordinates)); the body still starts at the heading.

### body.conflict: A body still in the middle of a merge

A declaration whose body holds a conflict block is not one text but two, and nothing in the tree says which of them is the declaration's: every citation of it, and every fingerprint the section lock takes of it ([§FS-section-lock.holds.fingerprint](FS-section-lock.md#holdsfingerprint-the-fingerprint)), would read one address as both ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)). This is the one definition of such a block: which lines are markers ([§FS-declarations.body.conflict.markers](FS-declarations.md#bodyconflictmarkers-a-marker-line)), how they make blocks ([§FS-declarations.body.conflict.block](FS-declarations.md#bodyconflictblock-a-conflict-block)), where blocks are looked for ([§FS-declarations.body.conflict.where](FS-declarations.md#bodyconflictwhere-where-blocks-are-looked-for)), and which declarations a block names ([§FS-declarations.body.conflict.owners](FS-declarations.md#bodyconflictowners-which-declarations-a-block-names)). `check` reports each block ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)), and `fmt` writes nothing while one exists ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)). A block changes no recognition: every other command reads the file as it did, and `show` still prints the bytes it reads ([§FS-show.requirements.authored-bytes](FS-show.md#requirementsauthored-bytes-a-slice-is-the-authors-bytes-selected-and-never-rewritten)). Decided in [§DF-conflicted-declaration](../decisions/functional/DF-conflicted-declaration.md#df-conflicted-declaration-a-declaration-that-still-holds-an-unresolved-merge-conflict-fails-check-and-stops-fmt-from-writing).

#### body.conflict.markers: A marker line

A marker line starts at column 0 with exactly seven of one character, `<`, `|`, `=` or `>`, followed by a space or by the end of the line; a CR before the LF is the end of the line. So `<<<<<<< HEAD`, `=======` and `>>>>>>> other` are marker lines, and `<<<<<<<< HEAD` (eight), `<<<<<<<HEAD` (no space) and ` <<<<<<< HEAD` (indented) are not. These are the lines git writes around a conflict, `|||||||` being the base marker of its `diff3` and `zdiff3` styles. The test is on the raw line, so a marker line inside a fence, an HTML comment, a docstring or a block comment is one all the same.

#### body.conflict.block: A conflict block

A conflict block opens at a `<<<<<<<` line and closes at the next `>>>>>>>` line, and it is those two lines with every line between them. A `|||||||` or `=======` line is a marker only inside an open block, so a setext underline of seven `=` or a table row of seven `|` outside one is text. A `<<<<<<<` inside an open block opens no second one: the block still closes at the next `>>>>>>>`. Two blocks are not well formed and are blocks all the same:

- a `>>>>>>>` with no block open is a block of that one line, which is what deleting one side of a conflict with its opening marker, and keeping the closing one, leaves behind;
- a `<<<<<<<` that no `>>>>>>>` closes is a block that ends at its last marker line.

A `>>>>>>> ` line is also a seven-level blockquote in Markdown. Outside an open block it is a lone closer, so a quote that deep is written `> > > > > > >`.

#### body.conflict.where: Where blocks are looked for

Blocks are read off a file's raw lines, top to bottom ([§FS-declarations.body.conflict.block](FS-declarations.md#bodyconflictblock-a-conflict-block)), and looked for in two places:

- **Every line of a Markdown file that declares an ID**: its preamble, its fences, and the lines after a heading that closed every body. Whether the file declares an ID is read off its raw lines, fences ignored: a line that would declare an ID by [§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares) were no fence open makes the file one that declares. A conflict can open a fence it never closes, and the file's one declaration below it would then read as fenced text; deciding on the fence-aware reading would leave that file's markers unread.
- **In a source file, the raw lines of each declaration's body** ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), in a docstring or a `/** */` comment as much as in a line-style run, and a block that opens on the line right after a declaration's line-style run ends ([§FS-languages.comment-block.close](FS-languages.md#comment-blockclose-find-the-close-boundary)), through its last line. A marker line is not a comment line, so a conflict inside a `///` or `#` run cuts the run at its opening marker and the body ends one line above the block. Counting the block there is what keeps `fmt --write`, which runs before any compiler, usually in a pre-commit hook, from fingerprinting the cut-short body into the section lock.

A marker line anywhere else is not looked at: in code outside those places the compiler or interpreter of that file refuses it, and a Markdown file that declares no ID, or a file the scan does not read, is no declaration's text. A target a stub pairs with outside `[scan] include` is read all the same, and is looked at as if it were scanned: it is the paired declaration's text, which `show` prints and the section lock fingerprints, so its blocks are reported at its own lines ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)).

#### body.conflict.owners: Which declarations a block names

A block names every declaration whose declaration line or body its lines overlap, each ID once, in the order its first such declaration opens, and a block counted after a cut line-style run names the declaration whose run it cut. A block that overlaps no body names none: one after a heading that closed the body, in a Markdown file's preamble, or above a declaration a fence the conflict left open hides. A block that holds declaration lines names the declaration above it and each one inside it, so a merge that wrote `# FS-b: B` on one side and `# FS-c: C` on the other, below `# FS-a: A`, names all three. Owners are read from the body spans every other command reads ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)), so `check`'s finding and `fmt`'s refusal name the same declarations.

## section: What a section is

A section is a citable heading inside a declaration's body: a numbered heading, or a named
one where named sections are enabled, whose path is written in the grammar of
[§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth), or, in a Markdown body, taken from its parent heading by a relative heading
([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)). A plain heading is not a section; it remains prose. A section's extent is
its heading and everything down to the next sibling-or-shallower heading; nested deeper
headings included. Which sections a declaration has is its section map
([§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span)). In a kind that declares a `front-matter` field, a
declaration's front matter is a section of it, `<ID>.front-matter`, and so is each top-level
key whose name is a section name, `<ID>.front-matter.<key>`. These sections have no heading
and lie outside the body ([§FS-fields.front-matter.coordinates](FS-fields.md#front-mattercoordinates-its-coordinates)).

### section.map: The section map is the declaration's body span

A coordinate enters the section map only when its heading lies inside that declaration's
existing body span ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)). A deeper numeric or enabled named heading
before that boundary remains a section; one after it is absent from the map and is
reported by [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration). Fenced
pseudo-headings, in Markdown or in a comment block, remain content under [§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example). This boundary does not make an
otherwise legal plain heading inside a body an error; that separate policy is outside this
contract. Every surface reads this map ([§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)), so none answers for
a heading it rejects. The map holds one more kind of entry, besides those headings: the
front-matter coordinates of a declaration whose kind declares a `front-matter` field. Those
entries lie above the body span, have no heading, and are marked as lying in the front matter
([§FS-fields.front-matter.coordinates](FS-fields.md#front-mattercoordinates-its-coordinates)). No heading outside the body enters the map that way.

#### section.map.nested: After a nested declaration, the enclosing body owns the heading

Bodies nest in Markdown. A declaration heading deeper than an enclosing one opens a nested
body, and that body ends at the next heading at its own level or shallower, while the
enclosing body goes on ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)). A numbered, enabled named or relative
heading that comes after the nested body has ended, and before the enclosing body ends, is a
coordinate of the enclosing declaration. It is never a coordinate of the nested declaration
the scan read last, and it is never dropped from every map. Where bodies nest several deep,
the owner is the innermost declaration whose body holds the heading. This is the owner
[§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) names for a plain heading in the same place, so
the coordinate that check suggests is the one the numbered heading then has.

```markdown
## AR-ci: CI

The CI lead.

### AR-matrix: Matrix

The matrix body.

### 1. Pull-request workflows

The pull-request workflows body.
```

`### 1. Pull-request workflows` is `AR-ci.1`. `grund AR-ci.1` prints it, `grund AR-ci --toc`
lists it, and a citation of it checks clean. `AR-matrix` has no section `1`. The same
holds in three other cases:

- an enabled named heading in that place, `### pr: Pull-request workflows`, which is `AR-ci.pr`;
- a heading under such a section, which takes its path from it, so `#### .1 Push` under a
  `### 2. Workflows` there is `AR-ci.2.1`;
- a nested declaration one level further down, so `#### 1.2 Second` after a `#### AR-x` inside
  `### 1. Overview` is `AR-ci.1.2`.

A heading admitted this way is a claimant like any other in that body. A `### 1.` before the
nested declaration and another after it are a [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path) of
`AR-ci.1`. A depth that does not match its path is a
[§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch) finding. And an unmarked heading after it is
suggested the next unused sibling ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)). grund 0.22
and earlier offered such a heading only to the nested declaration, which refused it, so it
resolved nowhere and was judged by nothing. The verdicts this moves are
[§DF-section-after-nested-declaration](../decisions/functional/DF-section-after-nested-declaration.md#df-section-after-nested-declaration-a-section-heading-after-a-nested-declaration-is-the-enclosing-declarations-coordinate).

### section.relative: A heading may write only the component it adds

Inside a Markdown declaration body a heading may write only the component it adds and take the rest of its path from its parent: under `## format: Formatting`, the heading `### .1: Username` declares `format.1`. The heading's **canonical path** is its parent's path, a `.`, and its component; under the declaration heading itself it is the component alone. The canonical path is the section's one coordinate. Citations, `refs`, `list`, completion, query results, the section map and every diagnostic name `FS-login.format.1`; `.1` alone is never a coordinate, and no citation form addresses a section relative to anything.

The form needs no setting and serves every outline, numeric or named, in every repository; relative and complete-path headings mix freely at any depth. It is unambiguous because no repository that passes `grund check` today contains it: before this form, every dotted heading inside a Markdown declaration body was an [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) error. Decided in [§DF-relative-section-headings](../decisions/functional/DF-relative-section-headings.md#df-relative-section-headings-a-heading-may-write-only-the-component-it-adds).

A cited relative section is guarded against silently holding other text after a move by the section lock ([§FS-section-lock](FS-section-lock.md#fs-section-lock-a-cited-relative-or-mixed-section-keeps-its-text-or-check-says-where-the-text-went)).

#### section.relative.form: The form

A relative heading writes one `.` and exactly one component, a number `[0-9]+` or a name in the grammar of [§FS-config.3.2.7](FS-config.md#327-named_sections--the-gate-for-explicit-section-names). The rest of the line uses the title syntax the complete-path heading for the same canonical path would use: the optional trailing period and space-separated title of a purely numeric path, or the colon form of a name-bearing one ([§FS-config.3.3.1](FS-config.md#331-named-components)).

| Parent | Relative heading | Canonical path | Complete-path heading it stands for |
|---|---|---|---|
| `## 3. Third` | `### .1 Username` or `### .1. Username` | `3.1` | `### 3.1 Username` or `### 3.1. Username` |
| `## format: Formatting` | `### .1: Username` | `format.1` | `### format.1: Username` |
| `## format: Formatting` | `### .user: Username` | `format.user` | `### format.user: Username` |
| `## 1. Overview` | `### .goals: Goals` | `1.goals` | `### 1.goals: Goals` |
| the declaration | `## .format: Formatting` | `format` | `## format: Formatting` |
| the declaration | `## .3 Third` | `3` | `## 3 Third` |

Expanding a relative heading is inserting its parent's path before the `.`. Everything the complete-path heading would be refused for, the relative heading is refused for too: a name with `named_sections` off, the colon form on a purely numeric path, and a numeric path written without its space or period. A dotted heading that is not this form — two components (`#### .a.b: Deep`), an empty component (`### .: Title`), or any of the refusals above — declares nothing and stays an [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) error, whose message then also shows the relative form ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)). Only a name needs `named_sections`: with it off, `### .1 Username` under `## 3. Third` is still section `3.1`.

#### section.relative.parent: The parent

A relative heading at level N takes as its parent the nearest heading before it in the same declaration body that is shallower than N; a heading inside a fence is content and is not a candidate. That heading must be exactly level N−1, and it must be either the declaration heading itself or a **usable** section: one the scan recorded, whose path is not an orphan ([§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path)), and which is not one of two or more claimants of one path ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)), whichever claimant it is. A parent that carries its own depth finding ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)) is still usable, and its child does not repeat that finding.

In every other case — a skipped level, a parent that is a plain heading, an orphan, a duplicate, or a failed relative heading, a would-be parent inside a fence — the relative heading declares nothing and is a [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent) error that names the parent it needed. Relative headings chain: `#### .1:` under `### .user:` under `## format:` is `format.user.1`. A relative heading and a complete-path heading that resolve to one canonical path, or two relative headings that do, are a [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path) like any two claimants.

A relative heading sits one level below its parent by construction, so it never carries a depth finding of its own ([§FS-config.3.3.2](FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)). Markdown's six levels therefore bound how many relative steps nest, five under an H1 declaration, and not how long a path is: under `"loose"` a parent that writes a seven-component path hands its relative child an eight-component one. The rule that a numeric prefix need not be recorded ([§FS-config.3.3.3](FS-config.md#333-every-prefix-of-a-name-bearing-path-is-recorded)) still holds for complete-path headings; a relative heading always needs its actual parent, because the parent is where its path comes from.

#### section.relative.where: Markdown declaration bodies, and doc-comments from 0.19.0

The form is recognised in an ATX heading inside a Markdown declaration body, the headings [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) judges. In a source doc-comment declaration body, the headings the doc-comment's section reading sees ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)), a line of the form declares its section under the same form, parent, lock and display rules as in Markdown, and citations below it belong to that section. The parent it reads is the section map the record gives that body, so a relative child of a `/** … */` line without ` * ` that reads `## 1. Sub` resolves under `.1` ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)). grund 0.18.0 kept the line as prose and warned `parse-moves` ([§FS-declarations.checks.parse-moves](FS-declarations.md#checksparse-moves-parse-moves-not-a-code), shape 4); 0.19.0 gave it this meaning. Giving it that meaning at once would have moved verdicts that passed, so it took the warning route of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), decided in [§DF-doc-comment-relative-headings](../decisions/functional/DF-doc-comment-relative-headings.md#df-doc-comment-relative-headings-a-relative-heading-in-a-doc-comment-warns-in-0180-and-is-a-section-from-0190).

A heading whose canonical path is mixed ([§FS-config.3.3.1](FS-config.md#331-named-components)) follows the same rule in a doc-comment, in both spellings: a complete `/// ### 1.goals: Goals` and a relative `/// ### .goals: Goals` under `/// ## 1. Overview` both declare their section, as they do in Markdown; 0.18.0 kept both as prose and warned `parse-moves` (shapes 5 and 4) ([§DF-number-name-sections.decision.doc-comments](../decisions/functional/DF-number-name-sections.md#decisiondoc-comments-doc-comments-wait-for-639)).

The same dotted line keeps exactly the meaning it has without this form everywhere else, in both releases: in a file title before the first declaration, after a heading that closed the body, in a file or comment block with no declaration, and in a fence or raw HTML block.

#### section.relative.shown: How a relative heading is shown

Every surface that reports a coordinate reports the canonical path. Three surfaces print a heading line for the reader to copy a coordinate from, and they print a relative heading with its canonical path in place of the relative one, at its written depth and with everything after the path kept. The section map ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)) maps `### .1: Username` under `## format: Formatting` as `### format.1: Username`, and `### .1. Username` under `## 3. Third` as `### 3.1. Username`, for a whole declaration, for a selected section, and for the selected section's own first line. The title hover shows `` `format.1: Username` `` ([§FS-lsp.1.2.4](FS-lsp.md#124-the-title-hovers-text)). A read of a selected section opens with its own heading in that form, in every mode ([§FS-declarations.section.relative.shown.selected](FS-declarations.md#sectionrelativeshownselected-a-selected-sections-own-heading-names-its-path)). A relative heading's display name is what follows its component, as a complete-path heading's is what follows its path: `## .terms: Terms` is a `Terms` chapter to a rule ([§FS-rules.5.1.1](FS-rules.md#511-a-chapters-display-name-is-the-label-its-author-wrote)), `## .status: Accepted` holds the field value `Accepted` ([§FS-fields.content.one-of](FS-fields.md#contentone-of-a-scalars-value-is-its-display-name)), and `list --selector` titles either one as its explicit form is titled. The expansion is deterministic, from the recorded parent, so it is a structural slice and not a summary ([§FS-non-goals.14](FS-non-goals.md#14-generated-summaries-token-saving-inside-check)).

Every other heading line a read of the file's text prints — the default read, `--brief` and `--full`, of a whole declaration or below a selected section's first line — prints a relative heading as written, and so does the JSON `body` of those reads. The JSON form of a `--toc` read carries the map's lines in `body`, and each `sections[]` entry's `path` is the canonical path, its `title` the title as written, and its `anchor` the anchor of the heading as written. A link anchor follows the heading as written and rendered, so `### .1: Username` is `#1-username` ([§FS-fmt.6.2.2](FS-fmt.md#622-the-anchor-slugifies-the-headings-rendered-text)). No command rewrites a relative heading into its complete form or back in the file ([§FS-fmt.2.3](FS-fmt.md#23-what-is-never-rewritten)).

##### section.relative.shown.selected: A selected section's own heading names its path

A read that selects a section, by its coordinate or by `--section`, prints that section's own heading line, the slice's first line, with its canonical path in place of the relative one, at its written depth and with its punctuation and title kept. That holds in all four modes — the default read, `--brief`, `--toc` and `--full` — and for a Markdown body and a source doc-comment body alike. With `### .2: Ceiling` under `## retry: Retry schedule` in `FS-billing`, `grund FS-billing.retry.2` and `grund FS-billing --section retry.2` both open with `### retry.2: Ceiling`, never `### .2: Ceiling`: the line is the slice's self-label ([§FS-show.2.2](FS-show.md#22-section)), and a relative heading labels nothing on its own, so two declarations that share a section shape would otherwise print the same first line.

Only that line changes. Every heading inside the slice below it, a heading inside a fence, and every heading of a whole-declaration read print as written; a heading written with its complete path is already canonical and prints unchanged. The section's anchor stays the anchor of the heading as written, so `### .2: Ceiling` keeps `2-ceiling` ([§FS-fmt.6.2.2](FS-fmt.md#622-the-anchor-slugifies-the-headings-rendered-text)), and a doc-comment section's stays null. `md` and the JSON `body` carry the same line as `text`, so a JSON body is still the text body ([§FS-show.3.1.3](FS-show.md#313-json)), and `list --size` measures it ([§FS-list.3.4.1](FS-list.md#341-measured-strings)). The expansion is the map's, from the recorded parent, and nothing is written back to the file. Decided in [§DF-show-selected-relative-heading](../decisions/functional/DF-show-selected-relative-heading.md#df-show-selected-relative-heading-a-read-of-a-selected-section-names-the-section-it-read), which replaces #643's "as written" for this one line ([§DF-relative-section-headings.decision.shown](../decisions/functional/DF-relative-section-headings.md#decisionshown-canonical-where-a-coordinate-is-copied-bytes-everywhere-else)).

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
holds the cases. A project with no citable `E2E` kind, or one whose `E2E` row names no folder,
has no case directories, and every directory this point describes is ordinary tree there.
Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration): a `§E2E-<name>` citation resolves exactly where its case exists.

### e2e-case.id: A case directory and the ID it declares

On a run over the project, where the `E2E` folder holds no `expected.exit` of its own, a
directory is a case directory when all three hold:

1. it sits directly in the `E2E` folder — a directory deeper down is not a case, whatever it holds;
2. it contains a file named `expected.exit`, or a link of that name whose target does not resolve,
   which makes it a case that fails at the link
   ([§FS-declarations.e2e-case.failure](FS-declarations.md#e2e-casefailure-a-case-that-cannot-be-read-fails-alone));
3. its name, written after `E2E` and the literal that follows `{kind}` in `[id] format`
   ([§FS-config.3.2](FS-config.md#32-id--id-grammar)), reads as an `E2E` ID with no section.

That ID is what the directory declares: under the default `{kind}-{number}-{slug}` the
directory `007-login` declares `E2E-007-login`, and under `{kind}-{slug}` the directory
`login` declares `E2E-login`. A directory that fails any of the three — `.gitkeep`, a folder
with no `expected.exit`, a name the grammar does not read — declares nothing. What a run
scoped to a path inside the folder declares, or a folder that holds `expected.exit` itself,
is not specified here.

A case declaration is located at its directory, line 1. It is never a stub and has no
sections, so a citation of it with a section suffix names a section that does not exist
([§FS-show.2.4](FS-show.md#24-e2e-cases)). It has no title either: no
heading declares it, so there is no title tail to read, and grund makes none up from the
directory's name. Where a surface reports a declaration's title, a case's is absent: an
empty title column in `grund list`, and `null` in its JSON ([§FS-list.3.1.1](FS-list.md#311-row-notes), [§FS-output-shapes.5](FS-output-shapes.md#5-list---formatjson)). Its body is the manifest `grund E2E-<name>` prints ([§FS-show.2.4](FS-show.md#24-e2e-cases)), and
a case that nothing cites is not reported as unused ([§FS-check.4.1.3](FS-check.md#413-e2e-declarations-are-exempt)).

### e2e-case.walk: The walk stops at a case directory

A case directory is a boundary of the ordinary walk. The case is registered through its
manifest, and the walk does not descend into the directory: the fixture repository a case
carries — its own `grund.toml`, its specs, its own nested cases — is not this project's
content, and nothing in it is read as a declaration or a citation of this project. The
boundary holds wherever the walk would enter the directory, under its own name or through a
symlink that resolves to it, and `--full` keeps it ([§FS-check.1.3.1](FS-check.md#131-the-scan-covers-the-whole-config-root)). It stops a walk that
would enter a case, not one that starts below it: a path argument naming a file or directory
inside a case directory is where that run's walk starts ([§FS-scan.path.discovery](FS-scan.md#pathdiscovery-a-path-is-where-discovery-starts)), and it
is read.

### e2e-case.excluded: `exclude` and an ignore file keep a case out

A case directory is found by the ordinary walk and its read predicate, and a case is no
exception to them: below a scan root, a case directory that `exclude` or an ignore file matches
is not read ([§FS-config-v2.schema.sources.exclude.roots](FS-config-v2.md#schemasourcesexcluderoots-how-exclude-composes-with-the-other-filters), item 2). In a version 1 configuration that is
a `[scan] exclude` name as well as an ignore file. It holds wherever the cases folder lies,
whether the walk lists it or the folder is a link out of the project that the walk skips. An
excluded case declares no ID, so a citation of it dangles and `grund list` does not name it,
and nothing in it can fail the run: a manifest that cannot be read or decoded there is not
reported. `--full` applies `exclude` and the ignore files as it always does. Only these keep a
case out. The walk's other reasons not to enter a directory, such as a link out of the project
or another project's tree, are about descending, and the walk never descends into a case
([§FS-declarations.e2e-case.walk](FS-declarations.md#e2e-casewalk-the-walk-stops-at-a-case-directory)). So a case directory that is such a link is still read.

What a case holds is its data and is read whole with the case: its manifests and its fixture
tree, whatever `exclude` or an ignore file says of a path inside the case directory. A
fixture's own `.gitignore` is content of the fixture repository. A pattern such as `repo/`,
which keeps fixture trees out of the walk, matches inside the case. Neither trims the fixture
list `grund E2E-<name>` prints ([§FS-show.2.4](FS-show.md#24-e2e-cases)), and neither hides a failure reading the case
([§FS-declarations.e2e-case.failure](FS-declarations.md#e2e-casefailure-a-case-that-cannot-be-read-fails-alone)). Value sources keep the exemption [§FS-values.2.2](FS-values.md#22-declarations-from-value-sources) gives them; a case
directory has none. Decided in [§DF-e2e-case-on-the-one-walk](../decisions/functional/DF-e2e-case-on-the-one-walk.md#df-e2e-case-on-the-one-walk-an-e2e-case-is-read-by-the-one-walk-so-exclude-and-ignore-files-keep-it-out-and-a-scan-failure-is-one-line-at-the-path-that-failed).

### e2e-case.failure: A case that cannot be read fails alone

A failure reading one case directory belongs to that case alone: a `command.args`,
`expected.exit` or `spec.refs` that cannot be read or decoded, an `expected.exit` that does not
parse as an exit code, or a fixture directory that cannot be listed. A `command.args`, `expected.exit` or `spec.refs` whose name is a link whose target does not resolve is such a file, not an absent one: it fails the case at the link's own path, with the reason the walk gives the same link ([§FS-scan.scope.links.unresolvable](FS-scan.md#scopelinksunresolvable-a-link-the-scan-cannot-resolve-is-reported-and-not-scanned-into)) — a broken link where the target does not exist, an unreadable link where it cannot be resolved for any other reason, a loop among them — and a manifest's entries are never read as none; only a name that is not there at all is an absent manifest, or, for `expected.exit`, no case. It is a per-file failure of [§FS-check.2.4](FS-check.md#24-an-incomplete-run), located at the file or directory that failed, as the tree spells it under the configured folder; decided in [§DF-e2e-case-failure-at-the-file](../decisions/functional/DF-e2e-case-failure-at-the-file.md#df-e2e-case-failure-at-the-file-a-failure-reading-an-e2e-case-is-located-at-the-file-that-failed-and-path-scoped-exits-move-with-it-in-both-directions). A run narrowed to a path holds it exactly when the path holds that file as the tree spells it there ([§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)), and a path through a link that resolves to that spelling does: with `c` a link to `b`, `grund check e2e/cases/c` holds a failure at `e2e/cases/b/spec.refs`. A path that does not resolve to that spelling does not hold it and cautions, and a link's target is such a path, a working directory inside a linked case or cases folder among them. The same file reported at another spelling, whichever of a manifest, a case and the cases folder is the link, or met by the walk as well as by the case, is the same failure and one line ([§FS-check.2.4.2](FS-check.md#242-one-failure-one-line)), named by the spelling the run's own path reached it under, so a run holding this spelling keeps its exit. A fixture directory reported at another spelling is one line only where the report kept this spelling as a name of the failure, and a run holding this spelling still holds that line; where the report kept no such name, each spelling reported is a line of its own.
So the walk over the cases continues, wherever the failing case sorts; decided in [§DF-e2e-case-read-failure-fails-alone](../decisions/functional/DF-e2e-case-read-failure-fails-alone.md#df-e2e-case-read-failure-fails-alone-a-failure-reading-one-e2e-case-is-that-cases-alone-and-path-scoped-exits-move-with-it-in-both-directions). Every other case declares
its ID as usual: a citation of it resolves, its findings print and `grund list` names it. Each
failing file or directory is reported, so two are two reports in one run, in one case or in two, and the run exits as [§FS-check.2.4](FS-check.md#24-an-incomplete-run) says, and as [§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud) says when it is narrowed to a path. What a citation
of the failing case itself does is not specified here.

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

A declaration's address is the one thing a repository stores about it in a thousand
places at once, so what may go wrong with a declaration is the same question as what may
go wrong with an address. Gathering the eleven checks here rather than under the command
that happens to report them puts each one beside the invariant it defends, and names each
one what a reader already holds when they need it: the code out of a finding they have
just read ([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)). `check` is one reader of this contract, and not the
only one — `show` refuses the coordinate this spec calls ambiguous, `fmt` rewrites toward
the canonical form it names, and the LSP transports every finding here unchanged — which
is the other reason the checks do not live inside the command.

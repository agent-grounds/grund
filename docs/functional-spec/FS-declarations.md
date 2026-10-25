# FS-declarations: a declaration is addressable once, from one allowed place, and holds nothing that is neither a coordinate nor a finding

A declaration is the line that introduces an ID, and it is what every citation in a
grund repository addresses. One invariant makes that address worth storing: an ID is
declared exactly once, it is declared where its kind's home allows it, and every ATX heading
inside its body in a Markdown file is either a coordinate a citation can reach or a finding
`check` reports; source doc-comment and setext text stay outside that rule
([§FS-declarations.checks.unmarked-heading.1](FS-declarations.md#checksunmarked-heading1-which-headings-participate)).
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
(source declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, workspace, member),
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
heading inside a fence or a raw-text HTML block is an example ([§FS-declarations.heading](FS-declarations.md#heading-what-a-heading-is),
[§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)); and an ID in inline code or in Markdown prose
declares nothing and is no near miss ([§FS-declarations.checks.declaration-near-miss.3](FS-declarations.md#checksdeclaration-near-miss3-never-in-inline-code-prose-or-a-fenced-block)). One
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
ID is proposed ([§FS-declarations.checks.declaration-near-miss.4](FS-declarations.md#checksdeclaration-near-miss4-facts-not-a-guessed-rename),
[§FS-declarations.checks.declaration-near-miss.6](FS-declarations.md#checksdeclaration-near-miss6-no-opt-out-no-rewrite)), a suggested coordinate is guidance `fmt`
never writes ([§FS-declarations.checks.unmarked-heading.3](FS-declarations.md#checksunmarked-heading3-the-suggested-coordinate),
[§FS-declarations.checks.unmarked-heading.6](FS-declarations.md#checksunmarked-heading6-no-command-numbers-the-heading)), a non-citable home is named as a place rather
than mapped to a kind ([§FS-declarations.checks.misplaced-declaration.3](FS-declarations.md#checksmisplaced-declaration3-a-non-citable-home)), and an oversized
lead is answered with remedies that keep its citations
([§FS-declarations.checks.oversized-lead.1](FS-declarations.md#checksoversized-lead1-the-message)). This binds every check added here from now on.
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
not.

### line.block-comment: A line inside a block comment

A block comment opens on a line whose first non-blank characters are `/*`, `/**` or `/*!`,
and closes at the first `*/` after its opener, on the opener's line or a later one. The
block is the one [§FS-languages.comment-block.forms](FS-languages.md#comment-blockforms-comment-blocks) names, read where `/*` is among
`[scan] comment_prefixes` ([§FS-languages.prefixes](FS-languages.md#prefixes-comment_prefixes-compose-with-extensions)). Inside it, three forms of line are in
declaration position, once the wrapper of [§FS-languages.comment-wrapper](FS-languages.md#comment-wrapper-the-comment-wrapper) is read off:

- the opener line, behind any of the three openers: `/* FS-b: Beta`, `/** FS-b: Beta`,
  `/*! FS-b: Beta`, and the one-line `/** FS-b: Beta */`, whose title is read as the
  one-line `/* FS-b: Beta */` form's title is;
- a continuation line behind ` * `: ` * FS-b: Beta`;
- a bare continuation line, with no ` * `, indented or not: `FS-b: Beta`. Only a block whose
  `*/` is found has bare lines in declaration position. In a block that runs to the end of
  the file without one, a bare line declares nothing, while the opener and starred forms
  read as they do in a closed block.

So the bare line declares exactly where its starred twin does, in any block comment and not
only a `/**` doc block, and the declaration is found by every command that reads the
catalog: `check`, `show`, `list`, `refs`, `cover` and the editor. A newly found declaration
draws every finding any other does, including `declaration-near-miss` (whose position is
this point's, [§FS-declarations.checks.declaration-near-miss.1](FS-declarations.md#checksdeclaration-near-miss1-what-counts), at the severity [§DF-bare-block-comment-line-declares.4](../decisions/functional/DF-bare-block-comment-line-declares.md#4-consequences) settles), duplicates, homes, indexes
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
only a token followed by the colon ([§FS-declarations.checks.declaration-near-miss.2](FS-declarations.md#checksdeclaration-near-miss2-the-declaration-colon-is-the-discriminator)), and
this line has none. `check` reports nothing about it.

#### line.section-suffix.2: With the colon, the whole token is the near miss

`# FS-042-user-login.2: the session` opens the way a declaration does, and its token ends
at the declaration colon, so the token is all of `FS-042-user-login.2`, section included.
The ID grammar rejects that token, which makes the line the off-grammar declaration
[§FS-config.3.2.5](FS-config.md#325-off-grammar-declarations-stay-readable) retains under its exact spelling and
[§FS-declarations.checks.declaration-near-miss.1](FS-declarations.md#checksdeclaration-near-miss1-what-counts) reports. It is never a declaration of
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
([§FS-errors.5.2.1](FS-errors.md#521-sites-on-an-ambiguity-refusal)). The duplicate finding
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
its whole path; 0.18.0 warned where the readings differ ([§FS-declarations.checks.stub-shape](FS-declarations.md#checksstub-shape-stub-shape-read-differently-from-grund-0190)),
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

Whether the file at `<path>` contains a source declaration of the ID is judged on the target's text as [§FS-scan.text.buffer](FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write) defines it, the text a save would write. That is the text the scan reads any file from and the text `show` slices the stub's body out of ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id), [§REQ-one-answer.text.targets](../requirements/REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would)). The path the stub wrote reads a buffer the editor opened under any spelling of the same file ([§FS-scan.identity.buffers](FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location)), and a target open only in a buffer, with no file saved behind it, is not missing: it is judged on the buffer, so it is healthy where the buffer declares the ID and `stub link target lacks <ID>: <path>` where it does not, as the saved file would be ([§FS-scan.text.buffer-only](FS-scan.md#textbuffer-only-a-buffer-with-no-file-behind-it-is-a-file)).

So an unsaved edit that removes the target's declaration breaks the stub at once. `check` reports `broken-stub` at the stub line, and the editor shows it as the user edits ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)). `show` refuses with the second line of [§FS-show.2.3.4](FS-show.md#234-broken-stub), never `ID not found`. An unsaved edit that puts the declaration back repairs the stub just as soon: `show` reads the body from the edited text, and the error goes. Scan scope plays no part, as it plays none in finding the body ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)).

#### stubs.target.2: A heading inside a fence of the target declares nothing

The target's text, the one [§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it) names, is read for the ID the way the scan reads that file. In a Markdown target a heading inside a fenced code block is an example, not a declaration ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)), and the fence is the one [§FS-languages.fence](FS-languages.md#fence-fences) defines: a fence delimiter line, and every line while a fence is open, declares nothing, however much it looks like `# <ID>: …`. A line inside a raw-text HTML block such as `<pre>` ([§FS-check.1.1.5.1](FS-check.md#1151-a-raw-html-block-holds-no-heading)) declares nothing in the same way. A target whose only heading of the ID sits inside a fence or such a block therefore contains no declaration of it, and the stub is broken exactly as if the target never named the ID, whether or not the target is inside `[scan] include`: `check` reports `stub link target lacks <ID>: <path>` at the stub's line, and `show` refuses with the second line of [§FS-show.2.3.4](FS-show.md#234-broken-stub) rather than answering `ID not found`. A target that holds a fenced example of the heading and the real declaration outside every fence declares the ID once, at the real heading, and the stub is healthy. In a source target a declaration line inside a doc-comment's fence is read as it would be outside it: such a fence hides only section and plain headings ([§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)), and this test reads the target the same way. Why a stub that used to pass now fails is [§DF-stub-target-fenced-heading](../decisions/functional/DF-stub-target-fenced-heading.md#df-stub-target-fenced-heading-a-heading-inside-a-fence-of-a-stubs-target-does-not-declare-its-id).

#### stubs.target.3: A target the scan does not read declares nothing

The target is read for the ID the way the scan reads that file ([§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)), and some files the scan never reads, wherever they sit: one whose own name begins with `.`, skipped by that name before `extensions` is consulted ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), and one whose extension `[scan] extensions` does not list ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)). Such a target declares nothing, whatever it holds. A `# <ID>: …` heading in `notes/.a.md`, or in `notes/a.zz` where `zz` is not listed, is text the scan does not read, so the stub is broken exactly as if the target never named the ID.

Name and extension refusals apply whether or not the target lies inside `[scan] include`. The name judged is the one the stub wrote, not the one a symlink resolves to: `notes/.a.md -> a.md` declares nothing even where the `notes/a.md` it resolves to is scanned and declares the ID, so the stub pairs with neither and is a declaration of its own beside that one, the duplicate a regular hidden copy of `notes/a.md` makes. The same holds for `notes/a.zz -> a.md` where `zz` is not listed. That a target outside `include` reads as if it were scanned ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)) reaches only a file the scan would read were it in scope.

When the target is refused because its extension is absent from `[scan] extensions`,
`check` reports `stub link target <path> is not scanned: extension <extension> is not in [scan] extensions`.
The diagnostic names the target the stub wrote, the refused extension and the configuration
key that prevents scanning; it never claims that this unread target lacks the ID.
It remains an error coded `broken-stub` at the stub's line, and `check` exits `1`.
This explanation applies inside and outside `[scan] include` alike, with the serialized
finding's fields unchanged. The hidden-name refusal still takes precedence over the
extension refusal and keeps `stub link target lacks <ID>: <path>`, as does a target
that is read and actually lacks the ID. Adding the extension to `[scan] extensions`
lets the target be read, and a target declaring the ID then makes the stub healthy.

`show` still refuses with the second line of [§FS-show.2.3.4](FS-show.md#234-broken-stub) rather than answering a body sliced out of a file the scan does not read. A stub broken this way pairs with nothing ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so `refs` refuses nothing from its target, however many headings of the ID the target holds ([§FS-refs.4.1](FS-refs.md#41-a-stubs-homes-are-found-in-its-target)). The remedy is the target's: a name that does not begin with `.`, and an extension `extensions` lists.

From grund 0.19.0 two more rules refuse a target: a directory on its path that `[scan] exclude` names ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)), and an enabled ignore rule ([§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)). A v2 exclusion pattern has the same effect ([§FS-config-v2.schema.sources.exclude.stubs](FS-config-v2.md#schemasourcesexcludestubs-a-pattern-is-the-exclude-reason-a-stubs-target-is-not-read)). Such a target declares nothing: its stub is `broken-stub`, an error at the stub's line, and `check` exits `1`. The refusal is recorded before pairing or lending any target content. This activates D5 of [§DF-one-answer-rulings-run.decision](../decisions/functional/DF-one-answer-rulings-run.md#decision-decision) after the warning release ([§DF-stub-target-exclusions](../decisions/functional/DF-stub-target-exclusions.md#df-stub-target-exclusions-excluded-and-ignored-stub-targets-are-broken-from-grund-0190)).

Both rules are asked as the walk asks them, from each scan root the run walks and from the config root in place of `include`. They match the target's lexical link spelling, as the walk's directory rules do ([§FS-scan.scope.links.names](FS-scan.md#scopelinksnames-the-directory-rules-apply-under-the-link-name)); two aliases of one physical file can therefore have different eligibility verdicts, while its content is still read once. A scan root under an excluded or ignored name outruns the descent rule here as in the walk ([§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent)). Name and extension refusals take precedence; otherwise the first rule met descending refuses, with exclude before ignore at the same step. `respect_gitignore = false` disables ignore only. Nested ignore files, negations, git-info and global ignore rules keep the walk's semantics. `--full` still honors these filters; path narrowing changes reporting only. Lying outside `include` alone is no reason. This activation adds no refusal for hidden ancestors, another project's boundary, an unwalked home or a target outside the config root. Formatter exclusions keep their separate role.

The error uses `stub link target <path> is not scanned: <reason>; this became an error in grund 0.19.0 — <remedy>`. The path is the destination as written. For v1 exclude the reason is `[scan] exclude names <entry>` and the remedy is `move the target out of <entry>, or take <entry> out of [scan] exclude`. For v2 the reason is `[schema.sources] exclude matches it by <pattern>` and the remedy is `move the target out of what <pattern> matches, or take <pattern> out of [schema.sources] exclude`. For ignore rules the reason is `<file> ignores it by <pattern>` and the remedy is `move the target out of what <file> ignores, or stop ignoring it there`. Entries and patterns are as written; ignore files are relative to the config root or absolute where outside it. Existing JSON fields stay unchanged. These errors replace the spent warning ([§FS-declarations.checks.excluded-stub-target](FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-broken-from-grund-0190)).

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
as before, in the same words — `missing section <ID>.<path>` for a numeric path — and under
the same `missing-section` code. A broken stub ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)) pairs
with nothing and gains no section: it keeps its broken-stub finding, and a citation of one of
its sections is `missing section` as before. A target that declares the ID more than once
pairs as `show` refuses it, ambiguously ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)), and lends no section, and so does
every target of an ID with more than one declaration ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)).

Every reader that enumerates a declaration's sections reads those sections from the paired
declaration, as the same tree's readers do with the target scanned: `show --batch --all`
generates a query for each ([§FS-show.2.6.2](FS-show.md#262-exhaustive-generation)), shell completion offers each
([§FS-completions.2.1](FS-completions.md#21-arguments-and-candidate-modes)), the editor's snapshot holds each for completion and navigation
([§FS-lsp.1.6](FS-lsp.md#16-declared-id-completion), [§FS-lsp.1.3](FS-lsp.md#13-go-to-definition)), and `list --selector` selects a chapter among them
([§FS-rules.8](FS-rules.md#8-command-surfaces)). A section path the target declares twice is one generated query, which
`show --batch --all` answers with the failed envelope of an ambiguous section
([§FS-show.2.6.3](FS-show.md#263-query-failures-and-run-level-failures)), so the batch exits `1` on the tree `check` already fails. Before grund
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
([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)), plain `list` ([§FS-list.2.5](FS-list.md#25-inline-homes-stay-canonical)), `list --size` ([§FS-list.3.4.6](FS-list.md#346-a-home-outside-the-scan)), the duplicate
error and the ambiguity refusal ([§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target)), and every finding about the
declaration itself — the `unused` warning ([§FS-check.4.1](FS-check.md#41-unused-declaration)), and a count or chapter count it
fails ([§FS-rules.7.2](FS-rules.md#72-chapter-cardinality), [§FS-rules.7.3](FS-rules.md#73-outbound-citation-cardinality), [§FS-rules.7.4](FS-rules.md#74-inbound-citation-cardinality)), whatever of the target's citations and
chapters the count reads ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)).

Plain `list` prints one row for each declaration of the ID the target holds, with the
target's title, once for all its stubs as [§FS-declarations.stubs.pairing.2](FS-declarations.md#stubspairing2-stubs-to-one-target-stand-for-its-declarations-once) says:
`FS-x  src/x.rs:1  X`. A target that declares the ID twice prints two rows, each marked as
[§FS-list.2.6](FS-list.md#26-duplicate-declarations) marks a duplicate. `check` reports a finding about the declaration at the
declaration's line in the target: `src/x.rs:1: warning: declared but never cited: FS-x`.

Before grund 0.19.0 plain `list` printed `FS-x  docs/fs/x.md:1  → ../../src/x.rs` and `check`
placed those findings at the stub; [§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190) moved both.

## heading: What a heading is

A heading is a Markdown ATX heading line (`#` through `######`) outside any fenced block
or raw-text HTML block: a line inside one is never a heading, by [§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example), and in
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
The `--toc` `title` is this text too: `## 1. Steps ##` is listed with the title `Steps`. grund 0.18.0
still printed the closing run there; 0.19.0 dropped it
([§DF-one-parse-answers-from-0-19](../decisions/functional/DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves)).

## body: Where a declaration's body ends

A declaration's body runs from its declaration line to the end of its body span. In
Markdown that is the start of the next same-or-higher heading, whether or not it declares
an ID (or end of file); in a source file it is the end of the comment block the
declaration line opens ([§FS-show.2.3.1](FS-show.md#231-what-counts-as-the-comment-block)). The span, not the most recently scanned
declaration, owns the coordinate: a same-or-higher Markdown heading, the end of an inline
source doc-comment or docstring, the next declaration in a shared comment block, and a
stub's single declaration line each end ownership.

### body.conflict: A body still in the middle of a merge

A declaration whose body holds a conflict block is not one text but two, and nothing in the tree says which of them is the declaration's: every citation of it, and every fingerprint the section lock takes of it ([§FS-section-lock.holds.fingerprint](FS-section-lock.md#holdsfingerprint-the-fingerprint)), would read one address as both ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution)). This is the one definition of such a block: which lines are markers ([§FS-declarations.body.conflict.markers](FS-declarations.md#bodyconflictmarkers-a-marker-line)), how they make blocks ([§FS-declarations.body.conflict.block](FS-declarations.md#bodyconflictblock-a-conflict-block)), where blocks are looked for ([§FS-declarations.body.conflict.where](FS-declarations.md#bodyconflictwhere-where-blocks-are-looked-for)), and which declarations a block names ([§FS-declarations.body.conflict.owners](FS-declarations.md#bodyconflictowners-which-declarations-a-block-names)). `check` reports each block ([§FS-declarations.checks.conflict-marker](FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)), and `fmt` writes nothing while one exists ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)). A block changes no recognition: every other command reads the file as it did, and `show` still prints the bytes it reads ([§FS-show.requirements.authored-bytes](FS-show.md#requirementsauthored-bytes-a-slice-is-the-authors-bytes-selected-and-never-rewritten--realized)). Decided in [§DF-conflicted-declaration](../decisions/functional/DF-conflicted-declaration.md#df-conflicted-declaration-a-declaration-that-still-holds-an-unresolved-merge-conflict-fails-check-and-stops-fmt-from-writing).

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
([§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span)).

### section.map: The section map is the declaration's body span

A coordinate enters the section map only when its heading lies inside that declaration's
existing body span ([§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends)). A deeper numeric or enabled named heading
before that boundary remains a section; one after it is absent from the map and is
reported by [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration). Fenced
pseudo-headings, in Markdown or in a comment block, remain content under [§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example). This boundary does not make an
otherwise legal plain heading inside a body an error; that separate policy is outside this
contract. Every surface reads this map ([§REQ-one-answer.sections.map](../requirements/REQ-one-answer.md#sectionsmap-every-surface-reads-one-section-map)), so none answers for
a heading it rejects.

### section.relative: A heading may write only the component it adds

Inside a Markdown declaration body a heading may write only the component it adds and take the rest of its path from its parent: under `## format: Formatting`, the heading `### .1: Username` declares `format.1`. The heading's **canonical path** is its parent's path, a `.`, and its component; under the declaration heading itself it is the component alone. The canonical path is the section's one coordinate. Citations, `refs`, `list`, completion, query results, the section map and every diagnostic name `FS-login.format.1`; `.1` alone is never a coordinate, and no citation form addresses a section relative to anything.

The form needs no setting and serves every outline, numeric or named, in every repository; relative and complete-path headings mix freely at any depth. It is unambiguous because no repository that passes `grund check` today contains it: before this form, every dotted heading inside a Markdown declaration body was an [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) error. Decided in [§DF-relative-section-headings](../decisions/functional/DF-relative-section-headings.md#df-relative-section-headings-a-heading-may-write-only-the-component-it-adds).

A cited relative section is guarded against silently holding other text after a move by the section lock ([§FS-section-lock](FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)).

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

Expanding a relative heading is inserting its parent's path before the `.`. Everything the complete-path heading would be refused for, the relative heading is refused for too: a name with `named_sections` off, the colon form on a purely numeric path, and a numeric path written without its space or period. A dotted heading that is not this form — two components (`#### .a.b: Deep`), an empty component (`### .: Title`), or any of the refusals above — declares nothing and stays an [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading) error, whose message then also shows the relative form ([§FS-declarations.checks.unmarked-heading.2.1](FS-declarations.md#checksunmarked-heading21-a-dotted-heading-is-also-shown-the-relative-form)). Only a name needs `named_sections`: with it off, `### .1 Username` under `## 3. Third` is still section `3.1`.

#### section.relative.parent: The parent

A relative heading at level N takes as its parent the nearest heading before it in the same declaration body that is shallower than N; a heading inside a fence is content and is not a candidate. That heading must be exactly level N−1, and it must be either the declaration heading itself or a **usable** section: one the scan recorded, whose path is not an orphan ([§FS-declarations.checks.orphan-section](FS-declarations.md#checksorphan-section-orphan-name-bearing-section-path)), and which is not one of two or more claimants of one path ([§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path)), whichever claimant it is. A parent that carries its own depth finding ([§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch)) is still usable, and its child does not repeat that finding.

In every other case — a skipped level, a parent that is a plain heading, an orphan, a duplicate, or a failed relative heading, a would-be parent inside a fence — the relative heading declares nothing and is a [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent) error that names the parent it needed. Relative headings chain: `#### .1:` under `### .user:` under `## format:` is `format.user.1`. A relative heading and a complete-path heading that resolve to one canonical path, or two relative headings that do, are a [§FS-declarations.checks.duplicate-section](FS-declarations.md#checksduplicate-section-duplicate-section-path) like any two claimants.

A relative heading sits one level below its parent by construction, so it never carries a depth finding of its own ([§FS-config.3.3.2](FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)). Markdown's six levels therefore bound how many relative steps nest, five under an H1 declaration, and not how long a path is: under `"loose"` a parent that writes a seven-component path hands its relative child an eight-component one. The rule that a numeric prefix need not be recorded ([§FS-config.3.3.3](FS-config.md#333-every-prefix-of-a-name-bearing-path-is-recorded)) still holds for complete-path headings; a relative heading always needs its actual parent, because the parent is where its path comes from.

#### section.relative.where: Markdown declaration bodies, and doc-comments from 0.19.0

The form is recognised in an ATX heading inside a Markdown declaration body, the headings [§FS-declarations.checks.unmarked-heading.1](FS-declarations.md#checksunmarked-heading1-which-headings-participate) judges. In a source doc-comment declaration body, the headings the doc-comment's section reading sees ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)), a line of the form declares its section under the same form, parent, lock and display rules as in Markdown, and citations below it belong to that section. The parent it reads is the section map the record gives that body, so a relative child of a `/** … */` line without ` * ` that reads `## 1. Sub` resolves under `.1` ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)). grund 0.18.0 kept the line as prose and warned `parse-moves` ([§FS-declarations.checks.parse-moves.2](FS-declarations.md#checksparse-moves2-the-five-shapes), shape 4); 0.19.0 gave it this meaning. Giving it that meaning at once would have moved verdicts that passed, so it took the warning route of [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), decided in [§DF-doc-comment-relative-headings](../decisions/functional/DF-doc-comment-relative-headings.md#df-doc-comment-relative-headings-a-relative-heading-in-a-doc-comment-warns-in-0180-and-is-a-section-from-0190).

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
2. it contains a file named `expected.exit`;
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
([§FS-show.2.4.1](FS-show.md#241-the-slices-over-a-manifest)). It has no title either: no
heading declares it, so there is no title tail to read, and grund makes none up from the
directory's name. Where a surface reports a declaration's title, a case's is absent: an
empty title column in `grund list`, and `null` in its JSON ([§FS-list.3.1.1](FS-list.md#311-row-notes), [§FS-list.3.2](FS-list.md#32---format-json)). Its body is the manifest `grund E2E-<name>` prints ([§FS-show.2.4](FS-show.md#24-e2e-cases)), and
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

## checks: Checks

Each section below is one check `grund check` enforces about a declaration, and its name
is the finding code verbatim — the token `--only` and `--ignore` take, as
[§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) publishes it and [§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code) requires. Severity is not part of
the address: each code's row in that catalog carries it, so a promotion edits a cell and
moves no coordinate. How a finding is rendered, selected and exited on is the command's
business, in [§FS-check.2](FS-check.md#2-outputs) and [§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore).

### checks.duplicate: Duplicate declaration

The same ID declared more than once: any two declarations that are not stubs ([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)), whether headings or inline doc-comment declarations and whether in one file or several. Reported per [§FS-check.2.1](FS-check.md#21-report-format): one error located at the lexicographically-first site, with the remaining sites listed in the message.

Duplicate JSON keys, cross-file JSON IDs, Markdown/JSON collisions, and overlapping opted-in ownership feed this same ambiguity rule even when their components agree. A duplicate target cannot be value-compared ([§FS-values.2.3](FS-values.md#23-duplicates-and-ownership)).

How a stub counts here is [§FS-declarations.stubs.pairing](FS-declarations.md#stubspairing-how-a-stub-pairs-with-what-it-points-at): a stub whose link holds is not a declaration of its own, whether or not the scan reaches what it points at ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)); stubs that point at one target stand for its declarations once between them ([§FS-declarations.stubs.pairing.2](FS-declarations.md#stubspairing2-stubs-to-one-target-stand-for-its-declarations-once)); and a declaration reached through stubs is named at its target ([§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target)).

#### checks.duplicate.1: A stub pairs with its target whether or not the target is scanned

Moved to [§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned).

#### checks.duplicate.2: Stubs to one target stand for its declarations once

Moved to [§FS-declarations.stubs.pairing.2](FS-declarations.md#stubspairing2-stubs-to-one-target-stand-for-its-declarations-once).

#### checks.duplicate.3: A declaration reached through stubs is named at its target

Moved to [§FS-declarations.stubs.pairing.3](FS-declarations.md#stubspairing3-a-declaration-reached-through-stubs-is-named-at-its-target).

### checks.declaration-near-miss: Declaration near miss

A heading that opens the way a declaration does and does not match its effective ID format remains a declaration for read compatibility ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). The classic stumble is `# FS-login: …` under the default `{kind}-{number}-{slug}` — the `-NNN-` left out. `check` emits one **error** per such declaration, at the line a contributor has to edit:

```
docs/spec.md:1: error: `FS-login` resolves for compatibility but does not match [id] format = "{kind}-{number}-{slug}" — rename it or change the effective format; this became an error in grund 0.16.0
```

What counts is [§FS-declarations.checks.declaration-near-miss.1](FS-declarations.md#checksdeclaration-near-miss1-what-counts): the declaration colon is its discriminator ([§FS-declarations.checks.declaration-near-miss.2](FS-declarations.md#checksdeclaration-near-miss2-the-declaration-colon-is-the-discriminator)), and inline code, prose and fenced blocks never count ([§FS-declarations.checks.declaration-near-miss.3](FS-declarations.md#checksdeclaration-near-miss3-never-in-inline-code-prose-or-a-fenced-block)). The message states facts rather than a guessed rename ([§FS-declarations.checks.declaration-near-miss.4](FS-declarations.md#checksdeclaration-near-miss4-facts-not-a-guessed-rename)), it became an error in 0.16.0 ([§FS-declarations.checks.declaration-near-miss.5](FS-declarations.md#checksdeclaration-near-miss5-a-warning-before-0160-an-error-in-it)), and there is no opt-out ([§FS-declarations.checks.declaration-near-miss.6](FS-declarations.md#checksdeclaration-near-miss6-no-opt-out-no-rewrite)).

- **Code:** `declaration-near-miss` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### checks.declaration-near-miss.1: What counts

A line in declaration position ([§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares), in a source file read through its comment block, [§FS-languages.comment-block](FS-languages.md#comment-block-the-comment-block)) whose first token unambiguously begins with a configured citable kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), which the effective ID grammar rejects, and which is **followed by the declaration colon**. This also covers a per-kind format whose first literal after `{kind}` differs from the persisted token.

#### checks.declaration-near-miss.2: The declaration colon is the discriminator

A line opening with an ID-shaped token and no colon is prose far more often than it is a declaration attempt — a comment wrapped across lines whose continuation begins with one is the case that proved it, in this repository's own source. So the rule reads exactly the shape a declaration attempt has, `<KIND>-…: <title>`, and says nothing about the rest. A near miss written without a title is not reported; that is the cost, and it buys a rule that stays quiet on prose.

#### checks.declaration-near-miss.3: Never in inline code, prose, or a fenced block

The token stops at a backtick, so an inline-code mention is not a near miss. The position rules are the declaration rules exactly, so a near miss is only ever read where a declaration would have been: a bare `FS-login: …` in Markdown prose is not one ([§DF-code-declarations-drop-hash](../decisions/functional/DF-code-declarations-drop-hash.md#df-code-declarations-drop-hash-code-resident-declarations-may-drop-the--prefix)), and neither is anything inside a fenced block.

#### checks.declaration-near-miss.4: Facts, not a guessed rename

The message names the token as written and the effective template, states that lookup remains compatible, and offers the two real migration choices: rename the declaration and its citations, or change the effective format. The template's key location follows the loaded-version and kind-override vocabulary of [§FS-show.3.5.1](FS-show.md#351-an-id-the-grammar-rejects). It does **not** propose a corrected ID; assembling one from component patterns would guess what the author meant.

#### checks.declaration-near-miss.5: A warning before 0.16.0, an error in it

Before 0.16.0 it was a warning, which left the exit code alone and ended `this warning becomes an error in grund 0.16.0`. In grund 0.16.0 the same code and location became an error: a retained finding contributes exit `1` ([§FS-check.2](FS-check.md#2-outputs)), and the deadline clause became the past-tense release report [§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) requires, `this became an error in grund 0.16.0`. This also covers an ID that resolves for compatibility but does not match `[id] format`. The declaration still appears in `list` and resolves through every reader — `show`, `refs`, `list`, `cover`, `fmt`, and LSP navigation read it exactly as before, and its heading's citations count as any declaration's do ([§FS-citations.recognized.title-lines](FS-citations.md#recognizedtitle-lines-a-declarations-heading-cites-what-it-names)); severity never changes recognition. The flip's release record is [§DF-off-grammar-declaration-compatibility.release-note](../decisions/functional/DF-off-grammar-declaration-compatibility.md#release-note-release-note).

#### checks.declaration-near-miss.6: No opt-out, no rewrite

There is no line-oriented opt-out or automatic rewrite: the position and colon rules bound recognition, and migration remains the repository author's choice.

### checks.broken-stub: Broken inline-spec stub

A stub ([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)) whose verdict is one of the three that hold no
declaration of the ID ([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)): nothing is at the path its link
resolves to, the target is a file the scan does not read ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)),
or the target, read as [§FS-declarations.stubs.target](FS-declarations.md#stubstarget-what-a-stubs-target-gives-it) reads it, does not declare the ID. A
target whose only heading of the ID is inside a fence ([§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing)) or is
itself a stub ([§FS-declarations.stubs.target.4](FS-declarations.md#stubstarget4-a-stub-in-the-target-declares-nothing)) does not declare it, so a chain of stubs is
broken at its first link.
An excluded or enabled-ignore-refused target is a file the scan does not read too:
the reason and remedy are defined by [§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing). It is an error
at the stub, replacing the warning of [§FS-declarations.checks.excluded-stub-target](FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-broken-from-grund-0190).

#### checks.broken-stub.1: The target is read as a save would write it

Moved to [§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it).

#### checks.broken-stub.2: A heading inside a fence of the target declares nothing

Moved to [§FS-declarations.stubs.target.2](FS-declarations.md#stubstarget2-a-heading-inside-a-fence-of-the-target-declares-nothing).

#### checks.broken-stub.3: A target the scan does not read declares nothing

Moved to [§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing).

#### checks.broken-stub.4: Every command reads the one verdict

Every command reads each stub's verdict and the declaration count it gives ([§FS-declarations.stubs.verdict](FS-declarations.md#stubsverdict-one-verdict-per-stub-reached-once-per-scan)). What they then agree on is [§REQ-one-answer.stubs.verdict](../requirements/REQ-one-answer.md#stubsverdict-every-command-reads-one-verdict-per-stub), [§REQ-one-answer.identity.declarations](../requirements/REQ-one-answer.md#identitydeclarations-every-command-reads-one-declaration-set), [§REQ-one-answer.stubs.outside-include](../requirements/REQ-one-answer.md#stubsoutside-include-a-target-outside-the-include-answers-as-if-scanned) and [§REQ-one-answer.text.targets](../requirements/REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would).

### checks.stub-shape: Stub shape read differently from grund 0.19.0

The check this heading names is spent: grund 0.19.0 removed the `docs` test of
[§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like), so no heading is read differently by two tests and the check
never fires. A heading whose title is one link is a stub in a kind's home and a declaration
titled with a link anywhere else, and nothing warns about either.

- **Code:** `stub-shape` stays in the catalog of codes, so `--only stub-shape` and
  `--ignore stub-shape` are still accepted ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)) and select nothing; neither
  `check` nor the editor ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)) reports it, and `--only stub-shape`
  says the check is spent with the off line of [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so).
- **Release:** the route was [§DF-stubs-by-home-and-at-the-target](../decisions/functional/DF-stubs-by-home-and-at-the-target.md#df-stubs-by-home-and-at-the-target-a-rule-declared-through-a-stub-applies-now-and-a-stub-is-read-by-its-home-and-shown-at-its-target-from-0190). This address, heading text
  included, is kept only so citations written during the window still resolve.

### checks.excluded-stub-target: Stub to a target the walk excludes, broken from grund 0.19.0

This check is spent: from grund 0.19.0 an excluded or enabled-ignore-refused target
makes the stub `broken-stub`, an error under [§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing),
instead of warning while keeping the stub healthy. Neither check nor the editor
emits `excluded-stub-target`.

The code remains accepted by `--only` and `--ignore` ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)) and selects
no findings. An old `--ignore excluded-stub-target` leaves the replacement error
visible; an old `--only excluded-stub-target` selects no replacement error and
keeps normal selected-report exit semantics, and says the check is spent with the
off line of [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so), which names `broken-stub`, the replacement to select.
JSON field shapes stay unchanged. This heading and coordinate remain for old citations.
The release route and compatibility notice are [§DF-stub-target-exclusions.release-note](../decisions/functional/DF-stub-target-exclusions.md#release-note-release-note).

### checks.misplaced-declaration: Misplaced declaration (configured kind home)

A declaration that sits where a configured kind home does not allow it is a misplaced-declaration error, located at the declaration line. Three placements are refused: a single-file kind's declaration outside its file ([§FS-declarations.checks.misplaced-declaration.1](FS-declarations.md#checksmisplaced-declaration1-a-single-file-kind)), a declaration inside another kind's home ([§FS-declarations.checks.misplaced-declaration.2](FS-declarations.md#checksmisplaced-declaration2-another-kinds-home)), and any declaration in a non-citable home ([§FS-declarations.checks.misplaced-declaration.3](FS-declarations.md#checksmisplaced-declaration3-a-non-citable-home)). The home rules ([§FS-declarations.checks.misplaced-declaration.2](FS-declarations.md#checksmisplaced-declaration2-another-kinds-home), [§FS-declarations.checks.misplaced-declaration.3](FS-declarations.md#checksmisplaced-declaration3-a-non-citable-home)) apply to declaration lines and stub lines, not citations or prose mentions. A file that belongs to no configured home, or that matches several because configured homes overlap or nest, is not checked by them, because its expected kind is ambiguous.

That exemption is v1's. In a version-2 config every scanned file has exactly one owner ([§FS-config-v2.schema.places.ownership](FS-config-v2.md#schemaplacesownership-a-listed-file-then-the-deepest-folder)), so a file in a nested place is checked against its owner: a declaration of `K` is allowed in a file owned by `K` or by the complement, and refused anywhere else. The home a message names is the place that owns the file, such as `AR-router declares kind AR inside FS home docs/spec/core`. A row whose places are all files is a single-file kind across them, and its message lists them, comma-separated, in written order.

#### checks.misplaced-declaration.1: A single-file kind

A kind configured with `file = "<path>"` in [[kinds]] ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)) is a *single-file kind*: every declaration of that kind must live in that exact document, and one whose H1/H2 is found in any other scanned file is reported:

```
docs/notes.md:42: GOAL-foo must be declared in docs/goals.md (single-file kind)
```

Stubs (`# <ID>: [<text>](<path>)`) are exempt from this exact-file requirement: a stub points from a kind's home folder to a source declaration elsewhere, a multi-file-kind feature, and a single-file kind has no folder to redirect from. This rule is the canonical mechanism that keeps `GRUND`, `GOAL`, and `RM` declarations in their documents, and what makes "one file, all goals inline" a checked invariant rather than a convention.

#### checks.misplaced-declaration.2: Another kind's home

Every configured `file` and `folder` is also a declaration-home boundary: a declaration line in a file that belongs to exactly one configured kind home must declare that home's kind. A `file` home matches only that exact path; a `folder` home matches files below that directory. The error names the declared kind, the expected home kind, and the configured home:

```
docs/functional-spec/FS-lsp.md:42: AR-router declares kind AR inside FS home docs/functional-spec
```

#### checks.misplaced-declaration.3: A non-citable home

A **non-citable home** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) admits no declaration of any kind. It has no kind an author could have declared instead, so the message names the place and says why rather than pointing at a kind that does not exist:

```
skills/review/SKILL.md:1: FS-review must not be declared in skills/ (not a citable home)
```

That is the rule working as designed, not a gap in it: `citable = false` says the directory is a place, and a place with a declaration in it is one of the two facts in conflict.

#### checks.misplaced-declaration.unlisted-project: Explain the enclosing homes beneath an unlisted project

When an existing misplaced-declaration error lies beneath a nested project config that is not active because no workspace member entry lists its directory, the message explains why the enclosing project's homes apply. This is context on each of the three placement forms above, not another finding: the nested directory remains ordinary tree to its enclosing scan ([§FS-check.1.3.1](FS-check.md#131-the-scan-covers-the-whole-config-root)). The explanation identifies the nested config's report-relative path, including the `.agents/grund.toml` discovery form ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), and its project name when reliably available; says that config is not read because its directory is unlisted; and names the enclosing config and project whose kind homes are being enforced. A declaration deeper below that directory gets the same context.

The message gives both concrete repairs in the enclosing config: add the nested project directory to `[workspace] members` if it belongs to the workspace, or exclude it from the enclosing scan if it does not. The exclusion advice uses that config version's setting and value: a path pattern with `[schema.sources] exclude` in v2 ([§FS-config-v2.schema.sources.exclude](FS-config-v2.md#schemasourcesexclude-exclude-is-a-gitignore-style-glob-list)), a directory component with `[scan] exclude` in v1, explaining that it excludes all directories of that name at every depth ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)). Exclusions prune descendants, not scan roots ([§FS-check.1.3.2](FS-check.md#132-the-wider-scan-reads-a-superset-each-file-once)); when the directory is itself a scan root, the advice must also explain removing it from the configured roots rather than promising that exclusion alone removes it. Changing a declaration's heading depth does not change which config owns the scan.

For the v2 root `shop` listing `packages/billing` but not `packages/notify`, an explanation can read:

```
packages/notify/requirements.md:3: error: FS-email must be declared in requirements.md (single-file kind) — packages/notify/grund.toml (project notify) is not read: no [workspace] members entry lists packages/notify; kind homes from grund.toml (project shop) apply; add "packages/notify" to [workspace] members in grund.toml, or exclude "packages/notify/" with [schema.sources] exclude in grund.toml
```

Text and JSON carry the same enriched message. The declaration path and line, `misplaced-declaration` code, severity, structured fields, finding count and exit verdict stay the same. Placement errors without this nested-config context retain their ordinary wording. A plain nested config with no placement violation gains no standalone finding, and the standalone unlisted-workspace finding and its message remain unchanged ([§FS-check.3.29.3](FS-check.md#3293-three-neighbouring-shapes-are-not-this-finding)). This explanation uses established config discovery and workspace ownership; it does not introduce another project boundary or new refusals for malformed configs or undecidable membership. An unobtainable ancestor `members` or `optional_members` value cannot establish missing membership, so the quiet claim check suppresses this context as specified by [§FS-workspace.6.1.7.5](FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable).

### checks.duplicate-section: Duplicate section path

Two or more citable section headings inside one declaration claiming the same dotted path ([§FS-declarations.section](FS-declarations.md#section-what-a-section-is)) — either two `## 1. …` headings or two `## goals: …` headings under one `# FS-001-login`. Reported per [§FS-check.2.1](FS-check.md#21-report-format) in [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)'s shape: one error located at the first heading in file order, with every other heading line named in the message. Named and numeric coordinates use the same `duplicate-section` code ([§FS-errors.5](FS-errors.md#5-json-format)), the same multi-site `sites` record [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) carries, and the same no-ranking rule.

```
docs/functional-spec/FS-001-login.md:5: duplicate section FS-001-login.1 (also declared at docs/functional-spec/FS-001-login.md:9)
```

This is [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) one level down. A section path is a citation target, so two headings claiming it give `§FS-001-login.1` two destinations, and picking one silently is the guess [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) forbids by name. Decided in [§DF-duplicate-section-path](../decisions/functional/DF-duplicate-section-path.md#df-duplicate-section-path-a-section-coordinate-names-one-heading-or-the-run-says-so). The collision is scoped to one declaration ([§FS-declarations.checks.duplicate-section.1](FS-declarations.md#checksduplicate-section1-scoped-to-one-declaration)) and its body ([§FS-declarations.checks.duplicate-section.2](FS-declarations.md#checksduplicate-section2-scoped-to-that-declarations-body)), independent of the heading-level mode ([§FS-declarations.checks.duplicate-section.3](FS-declarations.md#checksduplicate-section3-independent-of-id-section_heading_levels)), and read from the record `show` reads ([§FS-declarations.checks.duplicate-section.4](FS-declarations.md#checksduplicate-section4-the-same-record-show-reads)).

#### checks.duplicate-section.1: Scoped to one declaration

Section paths are addressed as `<ID>.<path>`, so the same `1.` under two different declarations is two distinct coordinates and not a finding. Only headings sharing a declaration collide.

#### checks.duplicate-section.2: Scoped to that declaration's body

The headings judged are the ones inside the body [§FS-declarations.body](FS-declarations.md#body-where-a-declarations-body-ends) delimits. A `## 1.` further down the file — in the *next* item's doc-comment, or under a later unrelated heading — is not one of this declaration's sections: `grund <ID>.1` never reaches it, and reporting it would ask for a renumbering that changes what nothing points at. A stub ([§FS-declarations.stubs.shape](FS-declarations.md#stubsshape-what-a-stub-looks-like)) is one link line whose tail is a path rather than a body, so it declares no sections of its own and is never reported here. The headings that count for its ID are those of each declaration it pairs with ([§FS-declarations.stubs.pairing.1](FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), which is also the file `grund <ID>.<path>` reads: they are judged in that declaration's body and reported at their own lines in the target, whether or not `[scan] include` reaches it ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)), exactly as the same tree with the target scanned reports them. A scanned target's declaration is one record however many stubs point at it, so its duplicate is reported once.

#### checks.duplicate-section.3: Independent of `[id] section_heading_levels`

The mode ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)) governs how deep a heading must sit for the path it writes, which is a different fact; `## 1.` and `### 1.` under an H1 declaration both claim path `1` and are a duplicate in every mode, `"loose"` included. [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch) judges only the first heading, the one the path resolves to; a later claimant is no section target and is not additionally judged for depth, so it yields this finding alone ([§DF-duplicate-section-path.2.4](../decisions/functional/DF-duplicate-section-path.md#24-the-heading-level-rule-judges-only-the-heading-the-path-resolves-to)).

#### checks.duplicate-section.4: The same record `show` reads

That this rule and `show` read one section record is [§REQ-one-answer.sections.ambiguity](../requirements/REQ-one-answer.md#sectionsambiguity-check-and-show-judge-a-section-path-from-one-record).

#### checks.duplicate-section.5: Judged over canonical paths

A relative heading claims its canonical path ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)), so `### format.1: Username` and `### .1: Username` under `## format:`, or two `### .1:` headings under it, are two claimants of `format.1` and this one finding, in the message above: anchored at the first line, naming the other. Neither claimant is a usable parent, so a relative heading beneath either is [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent).

### checks.orphan-section: Orphan name-bearing section path

With `[id] named_sections = true`, every proper prefix of a name-bearing section path must be recorded in the same declaration before the descendant can be addressed. `### goals.performance: Performance` therefore requires a recorded `goals`; `### missing.performance: Performance` is one error at that heading's line even when another Markdown heading visually contains it. The check uses the complete recorded path set and is independent of heading-depth validation, so a heading with both a missing prefix and a wrong depth produces both findings. Purely numeric paths are unchanged and are never judged by this rule.

The message names the orphan coordinate and its first absent prefix. Its code is `orphan-section`. This is a declaration-side structural error, not a missing-citation error: it is reported even when nobody cites the orphan, and a citation to it may independently be present and resolve to the recorded coordinate.

The rule is judged over canonical paths ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)). A relative heading is never an orphan, because it is recorded only under a usable parent; a relative heading beneath an orphan is [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent), and no second orphan finding.

### checks.relative-section-parent: A relative heading without its parent

A well-formed relative heading ([§FS-declarations.section.relative.form](FS-declarations.md#sectionrelativeform-the-form)) whose parent is not a usable section exactly one level shallower, nor the declaration heading ([§FS-declarations.section.relative.parent](FS-declarations.md#sectionrelativeparent-the-parent)), declares nothing and is one error at its line with code `relative-section-parent`:

```text
relative heading <written> needs an H<N-1> parent under <coordinate>, and there is none
```

`<written>` is the heading's `#` run, a space, and its relative component with its `.` (`#### .1`), and `<N-1>` is one less than that `#` count. `<coordinate>` is where the parent was needed: the nearest usable section or declaration heading before the heading that is shallower than level N−1, as `<ID>` or `<ID>.<path>`. The declaration heading always qualifies, so there is always one. Under `## format: Formatting` in `FS-login`, a `#### .1: Username` with no `###` between reads `relative heading #### .1 needs an H3 parent under FS-login.format, and there is none`. The same line covers every reason a parent is not usable — a skipped level, a plain heading, an orphan, a duplicate before or after the child, a failed relative heading, a would-be parent inside a fence — because the fix is the same: give the heading the parent it needs, or write its complete path.

A relative heading beneath a failed one fails the same way, at its own line. A citation of the address the heading meant is a [§FS-check.3.2](FS-check.md#32-missing-section) finding as it would be for any absent section. Text, JSON (`"sites":null`), LSP and `--only`/`--ignore` treatment are those of every located error ([§FS-check.2.1](FS-check.md#21-report-format)).

### checks.section-heading-level: Section heading level mismatch

`[id] section_heading_levels` ([§FS-config.3.3.2](FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)) sets how a citable section heading's Markdown depth must match its dotted path ([§FS-declarations.section](FS-declarations.md#section-what-a-section-is)), and whether a mismatch is an error, a warning, or not reported. A mismatch the mode reports is located at the heading line. This section judges the depth of headings that already carry coordinates; the project-wide in-body Markdown ATX rule for a heading that carries none is [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading), independent of this mode. Bold labels are not headings and remain unchecked.

Under v2 the key is `[schema] heading_depth`, a strength ([§FS-config-v2.schema.1](FS-config-v2.md#schema1-schema)). `must`, its default, reports a mismatch as an error, `warn` as a warning and `should` as a suggestion, and `may` recognizes sections without judging their depth, which is v1's `loose`.


### checks.section-outside-declaration: Section outside a declaration

When the scanner encounters a numeric section heading, or an enabled named section heading, that is deeper than its stale declaration context but whose line lies inside no declaration body, `check` emits one located error at the heading. A numeric heading's exact message is `numbered section outside any declaration`; an enabled named heading's is `named section outside any declaration`. Both use the public code `section-outside-declaration`. Ownership is the declaration's body span ([§FS-declarations.checks.section-outside-declaration.1](FS-declarations.md#checkssection-outside-declaration1-ownership-is-the-body-span)), the rejected heading leaves the section map every consumer reads ([§FS-declarations.checks.section-outside-declaration.2](FS-declarations.md#checkssection-outside-declaration2-the-rejected-heading-leaves-the-section-map)), and the finding is reported like any other hard error ([§FS-declarations.checks.section-outside-declaration.3](FS-declarations.md#checkssection-outside-declaration3-an-ordinary-hard-finding)).

#### checks.section-outside-declaration.1: Ownership is the body span

Ownership is the body span already used for extraction and citing-side classification, not a second section-only approximation. In Markdown, a same-or-higher heading ends a declaration body even when that boundary heading is plain; a later deeper section-like heading is outside. In source, the end of a doc-comment or docstring ends ownership. A later declaration in the same comment block ends the earlier body and begins its own. An inline-spec stub owns only its single heading line, so numbered prose below the stub belongs to no stubbed declaration. A deeper section-like heading before any of those boundaries stays valid. A heading inside a fence, in Markdown or in a comment block, is content and emits nothing ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example), [§FS-languages.fence.comment-block](FS-languages.md#fencecomment-block-a-fence-inside-a-comment-block-hides-its-headings)). Legal unmarked or plain headings are not errors under this section; adopting a general unmarked-heading policy is a separate change ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)).

#### checks.section-outside-declaration.2: The rejected heading leaves the section map

The rejected heading is excluded from the shared body-local section map before any consumer runs ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)). It cannot resolve a citation or query, enter completion or list/size output, become an embedded-value root, participate in duplicate-section detection, or acquire an LSP navigation target. `show` and other failed queries retain their ordinary missing-section semantics rather than printing this check-only message.

#### checks.section-outside-declaration.3: An ordinary hard finding

This is an ordinary hard finding under §[§FS-check.2](FS-check.md#2-outputs)–3. Text uses the located `<path>:<line>: error: <message>` form. JSON emits `{"severity":"error","path":<path>,"line":<line>,"code":"section-outside-declaration","message":<message>,"sites":null}`. `--only section-outside-declaration` retains it and `--ignore section-outside-declaration` removes it; a retained finding contributes exit `1`, while selecting it away restores the ordinary selected-report result. Parallel and workspace scans merge the record once under the workspace-relative path, never once per stale declaration. A narrowed scan judges the complete selected file, and `--full` applies the same code and message to otherwise out-of-scope files it adds. The LSP transports the same error severity, code, message, and heading range through its shared snapshot ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).

### checks.unmarked-heading: Unmarked Markdown heading

A Markdown ATX heading that is deeper than a declaration heading and whose line is still inside that declaration's body is an error when it is neither another declaration nor a recognized numeric or enabled named section, written with its complete path or as a relative heading ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)). The containing declaration is the nearest enclosing body, so a plain heading beneath a deeper child declaration names that child, not an overlapping ancestor. This is a project-wide rule with no configuration, severity selector, or permanent opt-out ([§DF-unmarked-markdown-headings](../decisions/functional/DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph)).

Which headings participate is [§FS-declarations.checks.unmarked-heading.1](FS-declarations.md#checksunmarked-heading1-which-headings-participate). The message is [§FS-declarations.checks.unmarked-heading.2](FS-declarations.md#checksunmarked-heading2-the-message) and its suggested coordinate [§FS-declarations.checks.unmarked-heading.3](FS-declarations.md#checksunmarked-heading3-the-suggested-coordinate); its rendering is [§FS-declarations.checks.unmarked-heading.4](FS-declarations.md#checksunmarked-heading4-rendering-exit-and-selection), its flip to an error [§FS-declarations.checks.unmarked-heading.5](FS-declarations.md#checksunmarked-heading5-an-error-in-grund-0160), and what it leaves unchanged [§FS-declarations.checks.unmarked-heading.6](FS-declarations.md#checksunmarked-heading6-no-command-numbers-the-heading).

#### checks.unmarked-heading.1: Which headings participate

Only ATX headings in scanned Markdown files participate. A heading inside a backtick or tilde fence is content. A file title before the first declaration, a same-or-shallower heading that closes a declaration body, source doc-comment text, setext text, and a bold label are outside the rule. A deeper declaration and a valid numeric or enabled named section already participate in the graph and are not unmarked, and so does a well-formed relative heading: one that has no usable parent is [§FS-declarations.checks.relative-section-parent](FS-declarations.md#checksrelative-section-parent-a-relative-heading-without-its-parent) and never this finding. A dotted heading that is not the relative form ([§FS-declarations.section.relative.form](FS-declarations.md#sectionrelativeform-the-form)) is judged here like any other heading. `--full` keeps this convention-scoped finding narrowed to the configured scan scope, as it does other convention findings ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)).

#### checks.unmarked-heading.2: The message

The error is located at the heading line, uses code `unmarked-heading`, and has this text:

```text
unmarked heading inside <ID>; number it (<suggested heading>) as <ID>.<path>, declare an ID, or use a bold label; this became an error in grund 0.16.0
```

##### checks.unmarked-heading.2.1: A dotted heading is also shown the relative form

When the heading's text begins with `.` — a dotted heading that is not a well-formed relative heading, such as `#### .a.b: Deep`, `### .1: Title` under a numeric parent, or a name with `named_sections` off — the message keeps every byte above and appends one clause showing the relative heading that would declare the same suggested coordinate:

```text
; under <parent coordinate> a relative heading is written <relative heading>
```

`<parent coordinate>` is `<ID>`, or `<ID>.<path>` without its last component, taken from the suggested coordinate. `<relative heading>` is the authored `#` run, a space, `.` and the suggested coordinate's last component, then the title syntax the suggested heading uses — `: ` on a name-bearing path, and on a purely numeric path the suggested heading's own `.` and space — then the authored title with its leading dotted token (the first run of non-space characters) removed, or `Untitled` when nothing is left. So `### .1: Username` under `## 3. Third` in `FS-001-login` reads `…; this became an error in grund 0.16.0; under FS-001-login.3 a relative heading is written ### .1. Username`. The clause is appended only where the suggested coordinate's parent is the declaration or a usable section ([§FS-declarations.section.relative.parent](FS-declarations.md#sectionrelativeparent-the-parent)), because a relative heading under any other parent would only trade this error for another. It is an append, so a consumer matching the existing text as a prefix, and every `code` consumer, reads the finding as before ([§FS-errors.3](FS-errors.md#3-message-text)).

##### checks.unmarked-heading.2.2: A direct field title or name is also shown the field form

In a version-2 project, an unmarked Markdown heading exactly one ATX level deeper than its actual containing declaration's heading is also shown the field form when its parsed heading text equals a field's configured `title` or its name, byte for byte. Use that declaration site's kind and depth, even when another site declares the same ID. Nested headings, headings outside a declaration, and text differing from both title and name receive no field clause; there is no case folding, fuzzy matching, or trimming beyond the heading parser. If several fields match, use only the first in configuration header order ([§FS-fields.declare.field](FS-fields.md#declarefield-a-field-is-a-table-under-its-kind)), whether it matched by title or by name.

Keep the entire existing message, including any relative-heading clause from [§FS-declarations.checks.unmarked-heading.2.1](FS-declarations.md#checksunmarked-heading21-a-dotted-heading-is-also-shown-the-relative-form), as a contiguous prefix, then append:

```text
; kind <KIND> declares the field <name>, written <field heading>
```

`<field heading>` is the authored `#` run, a space, the matched field's name, `: `, and its configured title if present, otherwise the parsed heading text. Thus both `## Schema` and `## schema` for a field named `schema` with `title = "Schema"` are shown `## schema: Schema`. This is repair guidance only: field identity stays [§FS-fields.identity.name](FS-fields.md#identityname-a-field-is-the-direct-section-of-its-name). The bare heading still produces `unmarked-heading` and an absent required field still produces `missing-field` at the declaration line; numbering the heading leaves the field missing, while the named edit declares it. Codes, severity, locations, exit status and structured output shape stay unchanged. Appending follows [§FS-errors.3](FS-errors.md#3-message-text); version-1 output, including a synthesized value-chapter field, stays byte-identical under [§FS-fields.v1](FS-fields.md#v1-version-1-is-unchanged).

#### checks.unmarked-heading.3: The suggested coordinate

The suggested coordinate is guidance, not a rewrite. Its path depth follows the written ATX depth relative to the containing declaration. At each depth, grund uses the nearest preceding citable or already-suggested parent and appends one above the largest existing or earlier-suggested numeric sibling; it never fills a hole or reuses a coordinate. With no parent it starts one above the largest root numeric coordinate, or at `1` when none exists. If the authored heading skips a depth, missing parents are filled with `.1`. A named parent may therefore receive a numeric child such as `goals.1`. The suggested heading preserves the authored `#` depth and title and inserts the complete coordinate in that valid numeric or mixed form. The suggestion is always this complete-path heading, for a plain heading and a dotted one alike; the relative form is offered only beside it ([§FS-declarations.checks.unmarked-heading.2.1](FS-declarations.md#checksunmarked-heading21-a-dotted-heading-is-also-shown-the-relative-form)). A titleless ATX heading has no authored title to preserve, so its otherwise-identical suggestion uses the literal title `Untitled`; applying that complete suggested heading produces a recognized section and clears the error.

#### checks.unmarked-heading.4: Rendering, exit, and selection

Text output uses the `<path>:<line>: error: <message>` form, and a retained finding contributes exit `1`. JSON emits the same path, line, code, and message with `"severity":"error"` and `"sites":null`. `--only unmarked-heading` retains it and `--ignore unmarked-heading` removes it, and with it the exit, through the ordinary exact-code selection rules. The LSP carries the same core finding with error severity and the complete ATX heading as its range ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).

#### checks.unmarked-heading.5: An error in grund 0.16.0

Grund 0.14.0 and 0.15.x reported the same finding as a warning at exit `0`, ending `this warning becomes an error in grund 0.16.0`; that window served [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). In grund 0.16.0 the site, code, suggestion, and every other byte stayed stable: only the severity moved to error, contributing exit `1`, and the deadline clause became `this became an error in grund 0.16.0`. The flip's release record is [§DF-unmarked-markdown-headings.release-note](../decisions/functional/DF-unmarked-markdown-headings.md#release-note-release-note).

#### checks.unmarked-heading.6: No command numbers the heading

No command numbers the heading: `grund fmt` remains unchanged. `show`, `list`, `refs`, `cover`, formatting, section resolution, and source scanning otherwise keep their current behavior, including the body boundary and rejected section behavior of [§FS-declarations.checks.section-outside-declaration](FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration). `grund_config_version` stays `1`; the managed agent block is the existing mechanical repair surface and moves to v10 under `grund init` ([§FS-init.2.3.4.5](FS-init.md#2345-declaration-forms)).

### checks.oversized-lead: Oversized lead *(opt-in)*

When the effective project config contains the opt-in key from [§FS-config.3.1](FS-config.md#31-reference--citation-form):

```toml
[reference]
lead_size_warning = { max = <N>, unit = "<unit>" }
```

`check` measures each declaration and citable section lead in that project by [§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements). A lead whose selected measurement is strictly greater than `max` produces one warning at that site's heading line. Equality passes. A broken stub has no measurable lead and produces no size warning; duplicate declaration homes and duplicate section claimants are judged separately from their own site-local slices.

The message is [§FS-declarations.checks.oversized-lead.1](FS-declarations.md#checksoversized-lead1-the-message) and its exit and rendering [§FS-declarations.checks.oversized-lead.2](FS-declarations.md#checksoversized-lead2-exit-code-and-rendering). Only the key activates it ([§FS-declarations.checks.oversized-lead.3](FS-declarations.md#checksoversized-lead3-only-the-key-activates-it)), and which sites it judges is [§FS-declarations.checks.oversized-lead.4](FS-declarations.md#checksoversized-lead4-which-sites-it-judges).

Under v2 the opt-in is the measure table `[schema.leads.words]` ([§FS-config-v2.schema.measures](FS-config-v2.md#schemameasures-measures-are-tables-strengths-are-keys)). Its strength picks the channel of the same finding: `must` an error, `warn` this warning, `should` a suggestion, `may` none.


#### checks.oversized-lead.1: The message

The fixed code is `oversized-lead`, severity is `warning`, and the exact text after `<path>:<line>: ` is:

```text
<coordinate> lead is <actual> <unit>, over the configured maximum of <max>; move detail into citable child sections, or promote a child section to its own ID after running grund refs <coordinate> --summary
```

The coordinate is local for a member-local check and workspace-qualified for a workspace-root check. The two remedies preserve grounding and citation stability; the message never suggests shortening or deleting it.

#### checks.oversized-lead.2: Exit code and rendering

A warning-only run exits `0` and replaces the text `success` line; JSON uses the ordinary located finding object ([§FS-errors.5](FS-errors.md#5-json-format)). The LSP publishes the same message, line, code, and warning severity as the CLI ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)). Any simultaneous error, including `duplicate` or `duplicate-section`, still decides exit `1`; the warning neither suppresses it nor changes its priority.

#### checks.oversized-lead.3: Only the key activates it

The absent key activates no measurement or finding, and a run that does not select the code leaves the existing text and JSON check output byte-identical. `--only oversized-lead` and `--ignore oversized-lead` select the finding after the complete check and never activate it; `--only oversized-lead` without the key prints the off line of [§FS-check.1.4.1](FS-check.md#141-a-selected-check-the-run-leaves-off-says-so), naming the key, in place of `success`.

#### checks.oversized-lead.4: Which sites it judges

An explicit-path check judges only declaration and section sites scanned at that path. `--full` adds its existing out-of-scope citation findings but does not extend this repository policy beyond declarations in the configured scan scope. In a workspace, each member's effective key governs only that member's sites; a member without the key remains silent even when another member opts in.

### checks.conflict-marker: Unresolved merge conflict

Each conflict block found where [§FS-declarations.body.conflict.where](FS-declarations.md#bodyconflictwhere-where-blocks-are-looked-for) looks is one error with code `conflict-marker`, located at the block's first line:

```text
unresolved merge conflict in <owners>, lines <first>–<last>: resolve it by hand
```

`<owners>` is the IDs the block names ([§FS-declarations.body.conflict.owners](FS-declarations.md#bodyconflictowners-which-declarations-a-block-names)), joined by `, ` and a final ` and `, and the phrase reads `outside any declaration` when it names none. Each ID is spelled as its own project spells it, as a `duplicate` finding spells it, in a workspace-root run too. `<first>` and `<last>` are the block's first and last lines, joined by an en dash, and a block of one line reads `line <first>`. A merge that left two versions of `FS-login`'s `### .1: Username` reads `docs/login.md:7: error: unresolved merge conflict in FS-login, lines 7–11: resolve it by hand`. In JSON, `sites` holds the block's first and last lines, in that order, and its one line for a block of one line. Text, LSP and `--only`/`--ignore` treatment are those of every located error ([§FS-check.2.1](FS-check.md#21-report-format)).

The message names no side and no command, because none can choose one: which text is the declaration's is the author's call ([§FS-declarations.requirements.actionable](FS-declarations.md#requirementsactionable-a-finding-is-located-at-the-fault-and-no-command-makes-the-fix--realized-directional-for-the-next-check)). A block that holds declaration lines or headings still raises what they raise, such as `duplicate`, `declaration-near-miss` or `duplicate-section`, beside this finding, which names their cause. `--ignore conflict-marker` hides the finding and unlocks nothing, because `fmt` reads the block from the model and not from `check`'s report ([§FS-fmt.7.4](FS-fmt.md#74-no-write-without-a-complete-model)). It is an error from the release that adds it, with no warning first ([§DF-conflicted-declaration](../decisions/functional/DF-conflicted-declaration.md#df-conflicted-declaration-a-declaration-that-still-holds-an-unresolved-merge-conflict-fails-check-and-stops-fmt-from-writing)).

### checks.parse-moves: A verdict that moves in 0.19.0

Retired in grund 0.19.0. From 0.19.0 every command reads one parse of each file
([§DF-one-parse-answers-from-0-19](../decisions/functional/DF-one-parse-answers-from-0-19.md#df-one-parse-answers-from-0-19-one-parses-other-answers-ship-in-0190-after-a-release-that-announces-them-and-warns-where-a-verdict-moves)), and a relative or mixed heading in a doc-comment
declaration body is a section ([§FS-declarations.section.relative.where](FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190)). grund 0.18.0
warned `parse-moves` at each line whose `check` verdict that release moved; 0.19.0 gives
those answers and the check, its code and its catalog row are gone ([#639](https://github.com/agent-grounds/grund/issues/639)). No tree gets
a `parse-moves` finding, and an `--only parse-moves` or `--ignore parse-moves` left in a
script names an unknown code and is refused as any unknown code is ([§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore)).
This address and its four below are kept so citations written before the change still
resolve.

#### checks.parse-moves.1: The message

Retired in grund 0.19.0 with the check ([§FS-declarations.checks.parse-moves](FS-declarations.md#checksparse-moves-a-verdict-that-moves-in-0190)).

#### checks.parse-moves.2: The five shapes

Retired in grund 0.19.0 with the check. Each of the five shapes is now the answer it
announced: a heading of the open section's depth or shallower, plain or numbered, or one
that repeats an open path, ends a doc-comment section for ownership too
([§FS-declarations.section](FS-declarations.md#section-what-a-section-is)); a `/** … */` line without ` * ` that reads as a heading
is that section ([§FS-languages.comment-wrapper.headings](FS-languages.md#comment-wrapperheadings-the-wrapper-of-a-heading-line)); `/// # 1. Sub` cuts no lead
([§FS-show.2.3.3](FS-show.md#233-section-selection-inside-a-doc-comment)); and a relative heading, or a complete heading whose path puts a
name under a number, in a doc-comment declaration body declares its section
([§FS-declarations.section.relative.where](FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190)).

#### checks.parse-moves.3: Rendering, exit, and selection

Retired in grund 0.19.0 with the check ([§FS-declarations.checks.parse-moves](FS-declarations.md#checksparse-moves-a-verdict-that-moves-in-0190)).

#### checks.parse-moves.4: Removed in 0.19.0

The check and its code were removed in 0.19.0, as [§FS-declarations.checks.parse-moves](FS-declarations.md#checksparse-moves-a-verdict-that-moves-in-0190)
says.

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

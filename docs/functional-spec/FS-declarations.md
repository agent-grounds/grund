# FS-declarations: a declaration is addressable once, from one allowed place, and holds nothing that is neither a coordinate nor a finding

A declaration is the line that introduces an ID, and it is what every citation in a
grund repository addresses. One invariant makes that address worth storing: an ID is
declared exactly once, it is declared where its kind's home allows it, and every heading
inside its body is either a coordinate a citation can reach or a finding `check` reports.
Which line introduces an ID at all is [§FS-declarations.line](FS-declarations.md#line-what-a-line-in-declaration-position-declares).
The checks below are that sentence enforced, save two that hold what an address is worth
once it resolves: that the declaration is in the form its ID grammar gives it
([§FS-declarations.checks.declaration-near-miss](FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss)), and that its lead, and every citable section
lead in its body, stays inside the size its project budgets where that project asks for
one ([§FS-declarations.checks.oversized-lead](FS-declarations.md#checksoversized-lead-oversized-lead-opt-in)). Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) and
[§GOAL-token-economy.1](../goals.md#1-what-this-requires).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, coordinate,
lead, index, catalog), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms)
(source declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, workspace, member),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, caution), and [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (rule).

## line: What a line in declaration position declares

A line is in declaration position where a declaration could open: a Markdown heading, a
comment-prefixed line in a source file, or a bare line inside a Python docstring
([§FS-show.2.3](FS-show.md#23-source-declarations-in-code-and-doc-comments)). The position is necessary and not sufficient. The token that opens the
line has to be the shape a declaration takes, and one shape that starts out like it is
not.

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

## checks: Checks

Each section below is one check `grund check` enforces about a declaration, and its name
is the finding code verbatim — the token `--only` and `--ignore` take, as
[§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog) publishes it and [§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code) requires. Severity is not part of
the address: each code's row in that catalog carries it, so a promotion edits a cell and
moves no coordinate. How a finding is rendered, selected and exited on is the command's
business, in [§FS-check.2](FS-check.md#2-outputs) and [§FS-check.1.4](FS-check.md#14-selecting-findings-with---only-and---ignore).

### checks.duplicate: Duplicate declaration

The same ID declared more than once: any two declarations that are not stubs ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)), whether headings or inline doc-comment declarations and whether in one file or several. Reported per [§FS-check.2.1](FS-check.md#21-report-format): one error located at the lexicographically-first site, with the remaining sites listed in the message.

Duplicate JSON keys, cross-file JSON IDs, Markdown/JSON collisions, and overlapping opted-in ownership feed this same ambiguity rule even when their components agree. A duplicate target cannot be value-compared ([§FS-values.2.3](FS-values.md#23-duplicates-and-ownership)).

A stub whose link holds is not a home of its own, whether or not the scan reaches what it points at ([§FS-declarations.checks.duplicate.1](FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)); stubs that point at one target are one home ([§FS-declarations.checks.duplicate.2](FS-declarations.md#checksduplicate2-stubs-to-one-target-are-one-home)); and a home reached through stubs is named at its target's declaration ([§FS-declarations.checks.duplicate.3](FS-declarations.md#checksduplicate3-a-home-reached-through-stubs-is-named-at-its-target)). Decided in [§DF-stub-pairs-with-unscanned-target](../decisions/functional/DF-stub-pairs-with-unscanned-target.md#df-stub-pairs-with-unscanned-target-a-stub-pairs-with-its-target-whether-or-not-the-scan-reaches-it), and for a target that declares the ID twice in [§DF-stub-target-declared-twice](../decisions/functional/DF-stub-target-declared-twice.md#df-stub-target-declared-twice-a-stubs-target-that-declares-its-id-twice-is-two-homes-scanned-or-not).

#### checks.duplicate.1: A stub pairs with its target whether or not the target is scanned

A stub whose target declares its ID — the stub [§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub) accepts — is the pointer to that declaration, not a second home of the ID, whether or not `[scan] include` reaches the target. The count of homes is read off the verdict the broken-stub check reports ([§FS-declarations.checks.broken-stub.4](FS-declarations.md#checksbroken-stub4-one-verdict-per-stub-read-by-every-command)), so it reads the target as a save would write it, the editor's unsaved text where the target is open in one and the disk only where it is not ([§FS-declarations.checks.broken-stub.1](FS-declarations.md#checksbroken-stub1-the-target-is-read-as-a-save-would-write-it)): the scan scope, which decides nothing about the stub's health, decides nothing about how many homes its ID has. So an unsaved edit to the target adds or removes a home of the ID exactly as saving it would, and the editor reports the duplicate, or its absence, that `check` reports once the edit is saved. A lone stub to an unscanned target still stands for its home, so its ID has one home, never none. The home it stands for is every declaration of the ID in its target that is not itself a stub, each read as the scan would read it were the target scanned, so a target that declares the ID twice is two homes and the ID is a duplicate, lone stub or not, named at both lines ([§FS-declarations.checks.duplicate.3](FS-declarations.md#checksduplicate3-a-home-reached-through-stubs-is-named-at-its-target)) as the same tree with the target scanned names them. A broken stub pairs with nothing and stays a home of its own, scanned target or not.

#### checks.duplicate.2: Stubs to one target are one home

Stubs of one ID whose links resolve to the same target file are one home between them, not one each: two pointers to one declaration do not make it two. Where that file declares the ID twice they stand for both declarations, once, as a lone stub does ([§FS-declarations.checks.duplicate.1](FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), never once per stub. Stubs whose links resolve to different files, each declaring the ID, are at least as many homes as there are files, and the ID is a duplicate. A surface that shows such a home at a stub rather than at its target — `list`, where the target is outside the scan — shows it once, at the first of its stubs in `path:line` order, and once for a target that declares the ID twice too, a row that carries the note every line of a duplicated ID carries ([§FS-list.3.1.1](FS-list.md#311-row-notes)).

#### checks.duplicate.3: A home reached through stubs is named at its target

Where a duplicate error or an ambiguity refusal ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)) lists a home that stubs point at, the site it names is the target's declaration of the ID, `<path>:<line>`, exactly as it would be were the target scanned — never a stub's own line, which has nothing to fix. A target that declares the ID twice is named at each of its declarations, never at its first alone. The sites sort, and the error is located, per [§FS-check.2.1](FS-check.md#21-report-format) over those names, so the first site may be in the target file. The same sites travel in the finding's JSON `sites` ([§FS-errors.5](FS-errors.md#5-json-format)).

### checks.declaration-near-miss: Declaration near miss

A heading that opens the way a declaration does and does not match its effective ID format remains a declaration for read compatibility ([§FS-config.3.2](FS-config.md#32-id--id-grammar)). The classic stumble is `# FS-login: …` under the default `{kind}-{number}-{slug}` — the `-NNN-` left out. `check` emits one **error** per such declaration, at the line a contributor has to edit:

```
docs/spec.md:1: error: `FS-login` resolves for compatibility but does not match [id] format = "{kind}-{number}-{slug}" — rename it or change the effective format; this became an error in grund 0.16.0
```

What counts is [§FS-declarations.checks.declaration-near-miss.1](FS-declarations.md#checksdeclaration-near-miss1-what-counts): the declaration colon is its discriminator ([§FS-declarations.checks.declaration-near-miss.2](FS-declarations.md#checksdeclaration-near-miss2-the-declaration-colon-is-the-discriminator)), and inline code, prose and fenced blocks never count ([§FS-declarations.checks.declaration-near-miss.3](FS-declarations.md#checksdeclaration-near-miss3-never-in-inline-code-prose-or-a-fenced-block)). The message states facts rather than a guessed rename ([§FS-declarations.checks.declaration-near-miss.4](FS-declarations.md#checksdeclaration-near-miss4-facts-not-a-guessed-rename)), it became an error in 0.16.0 ([§FS-declarations.checks.declaration-near-miss.5](FS-declarations.md#checksdeclaration-near-miss5-a-warning-before-0160-an-error-in-it)), and there is no opt-out ([§FS-declarations.checks.declaration-near-miss.6](FS-declarations.md#checksdeclaration-near-miss6-no-opt-out-no-rewrite)).

- **Code:** `declaration-near-miss` ([§FS-errors.5](FS-errors.md#5-json-format)).

#### checks.declaration-near-miss.1: What counts

A line in declaration position — a Markdown heading, or a comment-prefixed line in a source file under the rules of [AR-scanner.4](../architecture/AR-scanner.md#4-inline-declarations-in-language-doc-comments) — whose first token unambiguously begins with a configured citable kind ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)), which the effective ID grammar rejects, and which is **followed by the declaration colon**. This also covers a per-kind format whose first literal after `{kind}` differs from the persisted token.

#### checks.declaration-near-miss.2: The declaration colon is the discriminator

A line opening with an ID-shaped token and no colon is prose far more often than it is a declaration attempt — a comment wrapped across lines whose continuation begins with one is the case that proved it, in this repository's own source. So the rule reads exactly the shape a declaration attempt has, `<KIND>-…: <title>`, and says nothing about the rest. A near miss written without a title is not reported; that is the cost, and it buys a rule that stays quiet on prose.

#### checks.declaration-near-miss.3: Never in inline code, prose, or a fenced block

The token stops at a backtick, so an inline-code mention is not a near miss. The position rules are the declaration rules exactly, so a near miss is only ever read where a declaration would have been: a bare `FS-login: …` in Markdown prose is not one ([§DF-code-declarations-drop-hash](../decisions/functional/DF-code-declarations-drop-hash.md#df-code-declarations-drop-hash-code-resident-declarations-may-drop-the--prefix)), and neither is anything inside a fenced block.

#### checks.declaration-near-miss.4: Facts, not a guessed rename

The message names the token as written and the effective template, states that lookup remains compatible, and offers the two real migration choices: rename the declaration and its citations, or change the effective format. It does **not** propose a corrected ID; assembling one from component patterns would guess what the author meant.

#### checks.declaration-near-miss.5: A warning before 0.16.0, an error in it

Before 0.16.0 it was a warning, which left the exit code alone and ended `this warning becomes an error in grund 0.16.0`. In grund 0.16.0 the same code and location became an error: a retained finding contributes exit `1` ([§FS-check.2](FS-check.md#2-outputs)), and the deadline clause became the past-tense release report [§FS-distribution.4.2](FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) requires, `this became an error in grund 0.16.0`. This also covers an ID that resolves for compatibility but does not match `[id] format`. The declaration still appears in `list` and resolves through every reader — `show`, `refs`, `list`, `cover`, `fmt`, and LSP navigation read it exactly as before; severity never changes recognition. The flip's release record is [§DF-off-grammar-declaration-compatibility.release-note](../decisions/functional/DF-off-grammar-declaration-compatibility.md#release-note-release-note).

#### checks.declaration-near-miss.6: No opt-out, no rewrite

There is no line-oriented opt-out or automatic rewrite: the position and colon rules bound recognition, and migration remains the repository author's choice.

### checks.broken-stub: Broken inline-spec stub

A `docs/` file whose H1 has the stub shape `# <ID>: [<text>](<path>)` where either the path does not exist, or the file at that path contains no source declaration of the same ID. Relative stub links resolve as normal Markdown links first — relative to the stub file's directory — so `lychee` and rendered docs see the same target. If that path does not exist, `grund` falls back to resolving the path relative to the config root for compatibility with older stubs that wrote repo-root paths. Which lines of the target can hold that declaration is [§FS-declarations.checks.broken-stub.2](FS-declarations.md#checksbroken-stub2-a-heading-inside-a-fence-of-the-target-declares-nothing), and which targets can hold one at all is [§FS-declarations.checks.broken-stub.3](FS-declarations.md#checksbroken-stub3-a-target-the-scan-does-not-read-declares-nothing). The scan reaches this verdict once per stub, and every command reads it rather than one of its own ([§FS-declarations.checks.broken-stub.4](FS-declarations.md#checksbroken-stub4-one-verdict-per-stub-read-by-every-command)).

#### checks.broken-stub.1: The target is read as a save would write it

Whether the file at `<path>` contains a source declaration of the ID is judged on the text a save would write: the editor's unsaved text where the file is open in one, and the text on disk only where it is not. That is the text the scan reads any file from and the text `show` slices the stub's body out of ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)), so the verdict and the body never come from two versions of one file.

So an unsaved edit that removes the target's declaration breaks the stub at once. `check` reports `broken-stub` at the stub line, and the editor shows it as the user edits ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)). `show` refuses with the second line of [§FS-show.2.3.4](FS-show.md#234-broken-stub), never `ID not found`. An unsaved edit that puts the declaration back repairs the stub just as soon: `show` reads the body from the edited text, and the error goes. Scan scope plays no part, as it plays none in finding the body ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)). Every answer is the one the same text gives once it is saved, which is what keeps the editor's diagnostics, its hover and the command line in agreement ([§FS-lsp.4](FS-lsp.md#4-determinism-and-parity-with-the-cli), [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)).

#### checks.broken-stub.2: A heading inside a fence of the target declares nothing

The target's text, the one [§FS-declarations.checks.broken-stub.1](FS-declarations.md#checksbroken-stub1-the-target-is-read-as-a-save-would-write-it) names, is read for the ID the way the scan reads that file. In a Markdown target a heading inside a fenced code block is an example, not a declaration ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)), and the fence is the one [§FS-check.1.1.5](FS-check.md#115-contexts-read-as-neither-prose-nor-code) defines: a fence delimiter line, and every line while a fence is open, declares nothing, however much it looks like `# <ID>: …`. A target whose only heading of the ID sits inside a fence therefore contains no declaration of it, and the stub is broken exactly as if the target never named the ID, whether or not the target is inside `[scan] include`: `check` reports `stub link target lacks <ID>: <path>` at the stub's line, and `show` refuses with the second line of [§FS-show.2.3.4](FS-show.md#234-broken-stub) rather than answering `ID not found`. A target that holds a fenced example of the heading and the real declaration outside every fence declares the ID once, at the real heading, and the stub is healthy. Fences are tracked in a Markdown target only: inside a source target's doc-comment the scan does not track them, so neither does this test ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)). Why a stub that used to pass now fails is [§DF-stub-target-fenced-heading](../decisions/functional/DF-stub-target-fenced-heading.md#df-stub-target-fenced-heading-a-heading-inside-a-fence-of-a-stubs-target-does-not-declare-its-id).

#### checks.broken-stub.3: A target the scan does not read declares nothing

The target is read for the ID the way the scan reads that file ([§FS-declarations.checks.broken-stub.2](FS-declarations.md#checksbroken-stub2-a-heading-inside-a-fence-of-the-target-declares-nothing)), and some files the scan never reads, wherever they sit: one whose own name begins with `.`, skipped by that name before `extensions` is consulted ([§FS-config.3.5.12](FS-config.md#3512-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), and one whose extension `[scan] extensions` does not list ([§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)). Such a target declares nothing, whatever it holds. A `# <ID>: …` heading in `notes/.a.md`, or in `notes/a.zz` where `zz` is not listed, is text the scan does not read, so the stub is broken exactly as if the target never named the ID.

What decides is the target's own name and extension, not the scan's scope, so the stub is broken whether or not the target lies inside `[scan] include`. The name judged is the one the stub wrote, not the one a symlink resolves to: `notes/.a.md -> a.md` declares nothing even where the `notes/a.md` it resolves to is scanned and declares the ID, so the stub pairs with neither and is a home of its own beside that declaration, the duplicate a regular hidden copy of `notes/a.md` makes. The same holds for `notes/a.zz -> a.md` where `zz` is not listed. That a target outside `include` reads as if it were scanned ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)) reaches only a file the scan would read were it in scope. `check` reports `stub link target lacks <ID>: <path>` at the stub's line, and `show` refuses with the second line of [§FS-show.2.3.4](FS-show.md#234-broken-stub) rather than answering a body sliced out of a file the scan does not read. A stub broken this way pairs with nothing ([§FS-declarations.checks.duplicate.1](FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)), so `refs` refuses nothing from its target, however many headings of the ID the target holds ([§FS-refs.4.1](FS-refs.md#41-a-stubs-homes-are-found-in-its-target)). The remedy is the target's: a name that does not begin with `.`, and an extension `extensions` lists.

#### checks.broken-stub.4: One verdict per stub, read by every command

Each stub's verdict is reached once per scan, and it is one of four: the target is missing; the target is a file the scan does not read ([§FS-declarations.checks.broken-stub.3](FS-declarations.md#checksbroken-stub3-a-target-the-scan-does-not-read-declares-nothing)); the target, read as the scan reads it on the text a save would write ([§FS-declarations.checks.broken-stub.1](FS-declarations.md#checksbroken-stub1-the-target-is-read-as-a-save-would-write-it), [§FS-declarations.checks.broken-stub.2](FS-declarations.md#checksbroken-stub2-a-heading-inside-a-fence-of-the-target-declares-nothing)), declares no home of the ID; or it declares the ID, on one line or more, and each of those declarations is a home the stub points at ([§FS-declarations.checks.duplicate.1](FS-declarations.md#checksduplicate1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)). The first three are a broken stub. The homes of the ID are counted from those verdicts, and `check`, `show`, `refs`, `list`, `list --size`, `fmt --cross-refs` and the editor all read the verdict and the count; none reaches an answer of its own. So on one tree they cannot disagree:

- Where `check` reports a stub broken, `show` refuses it as a broken stub ([§FS-show.2.3.4](FS-show.md#234-broken-stub)), `list --size` labels its row `broken stub` ([§FS-list.3.4.3](FS-list.md#343-text-rows)), and `fmt` wraps no citation through it ([§FS-fmt.6.4.2](FS-fmt.md#642-a-citation-of-a-broken-stub-or-of-an-ambiguous-id-is-not-wrapped)).
- Where `check` reports the ID a duplicate, `show` and `refs` refuse it as ambiguous at the same sites ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id), [§FS-refs.4.1](FS-refs.md#41-a-stubs-homes-are-found-in-its-target)), `list` and `list --size` mark its rows duplicate, and `fmt` wraps none of its citations ([§FS-fmt.6.4.2](FS-fmt.md#642-a-citation-of-a-broken-stub-or-of-an-ambiguous-id-is-not-wrapped)).
- A target outside `[scan] include` is counted, measured and linked exactly as it is with the target scanned, a target that declares the ID twice included ([§FS-list.3.4.6](FS-list.md#346-a-home-outside-the-scan)).
- An unsaved edit to the target moves every answer at once, the editor's diagnostics and its navigation included, exactly as saving it would.

### checks.misplaced-declaration: Misplaced declaration (configured kind home)

A declaration that sits where a configured kind home does not allow it is a misplaced-declaration error, located at the declaration line. Three placements are refused: a single-file kind's declaration outside its file ([§FS-declarations.checks.misplaced-declaration.1](FS-declarations.md#checksmisplaced-declaration1-a-single-file-kind)), a declaration inside another kind's home ([§FS-declarations.checks.misplaced-declaration.2](FS-declarations.md#checksmisplaced-declaration2-another-kinds-home)), and any declaration in a non-citable home ([§FS-declarations.checks.misplaced-declaration.3](FS-declarations.md#checksmisplaced-declaration3-a-non-citable-home)). The home rules ([§FS-declarations.checks.misplaced-declaration.2](FS-declarations.md#checksmisplaced-declaration2-another-kinds-home), [§FS-declarations.checks.misplaced-declaration.3](FS-declarations.md#checksmisplaced-declaration3-a-non-citable-home)) apply to declaration lines and stub lines, not citations or prose mentions. A file that belongs to no configured home, or that matches several because configured homes overlap or nest, is not checked by them, because its expected kind is ambiguous.

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

### checks.duplicate-section: Duplicate section path

Two or more citable section headings inside one declaration claiming the same dotted path ([AR-scanner.2.2](../architecture/AR-scanner.md#22-section-detection)) — either two `## 1. …` headings or two `## goals: …` headings under one `# FS-001-login`. Reported per [§FS-check.2.1](FS-check.md#21-report-format) in [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration)'s shape: one error located at the first heading in file order, with every other heading line named in the message. Named and numeric coordinates use the same `duplicate-section` code ([§FS-errors.5](FS-errors.md#5-json-format)), the same multi-site `sites` record [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) carries, and the same no-ranking rule.

```
docs/functional-spec/FS-001-login.md:5: duplicate section FS-001-login.1 (also declared at docs/functional-spec/FS-001-login.md:9)
```

This is [§FS-declarations.checks.duplicate](FS-declarations.md#checksduplicate-duplicate-declaration) one level down. A section path is a citation target, so two headings claiming it give `§FS-001-login.1` two destinations, and picking one silently is the guess [§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution) forbids by name. Decided in [§DF-duplicate-section-path](../decisions/functional/DF-duplicate-section-path.md#df-duplicate-section-path-a-section-coordinate-names-one-heading-or-the-run-says-so). The collision is scoped to one declaration ([§FS-declarations.checks.duplicate-section.1](FS-declarations.md#checksduplicate-section1-scoped-to-one-declaration)) and its body ([§FS-declarations.checks.duplicate-section.2](FS-declarations.md#checksduplicate-section2-scoped-to-that-declarations-body)), independent of the heading-level mode ([§FS-declarations.checks.duplicate-section.3](FS-declarations.md#checksduplicate-section3-independent-of-id-section_heading_levels)), and read from the record `show` reads ([§FS-declarations.checks.duplicate-section.4](FS-declarations.md#checksduplicate-section4-the-same-record-show-reads)).

#### checks.duplicate-section.1: Scoped to one declaration

Section paths are addressed as `<ID>.<path>`, so the same `1.` under two different declarations is two distinct coordinates and not a finding. Only headings sharing a declaration collide.

#### checks.duplicate-section.2: Scoped to that declaration's body

The headings judged are the ones inside the body [§FS-show.2.1](FS-show.md#21-whole-declaration-default) and [§FS-show.2.3.1](FS-show.md#231-what-counts-as-the-comment-block) delimit — in Markdown down to the next same-or-shallower heading, in a source file to the end of the comment block the declaration line opens. A `## 1.` further down the file — in the *next* item's doc-comment, or under a later unrelated heading — is not one of this declaration's sections: `grund <ID>.1` never reaches it, and reporting it would ask for a renumbering that changes what nothing points at. A stub ([§FS-declarations.checks.broken-stub](FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)) is one link line whose tail is a path rather than a body, so it declares no sections at all and is never reported here; the headings that count are the inline home's, which is also the file `grund <ID>.<path>` reads.

#### checks.duplicate-section.3: Independent of `[id] section_heading_levels`

The mode ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth)) governs how deep a heading must sit for the path it writes, which is a different fact; `## 1.` and `### 1.` under an H1 declaration both claim path `1` and are a duplicate in every mode, `"loose"` included. [§FS-declarations.checks.section-heading-level](FS-declarations.md#checkssection-heading-level-section-heading-level-mismatch) judges only the first heading, the one the path resolves to; a later claimant is no section target and is not additionally judged for depth, so it yields this finding alone ([§DF-duplicate-section-path.2.4](../decisions/functional/DF-duplicate-section-path.md#24-the-heading-level-rule-judges-only-the-heading-the-path-resolves-to)).

#### checks.duplicate-section.4: The same record `show` reads

This rule and [§FS-show.2.2.2](FS-show.md#222-ambiguous-section) answer from one recorded section set, so `grund <ID>.<path>` refuses exactly when this rule reports `<ID>.<path>` and returns a body exactly when it does not. Two readers that each decided for themselves would disagree — a fenced example, a heading past the end of the body — and a coordinate `check` calls clean but `show` will not resolve is [§REQ-no-wrong-citation](../requirements/REQ-no-wrong-citation.md#req-no-wrong-citation-a-citation-never-resolves-to-a-guess) failing quietly in the other direction.

### checks.orphan-section: Orphan name-bearing section path

With `[id] named_sections = true`, every proper prefix of a name-bearing section path must be recorded in the same declaration before the descendant can be addressed. `### goals.performance: Performance` therefore requires a recorded `goals`; `### missing.performance: Performance` is one error at that heading's line even when another Markdown heading visually contains it. The check uses the complete recorded path set and is independent of heading-depth validation, so a heading with both a missing prefix and a wrong depth produces both findings. Purely numeric paths are unchanged and are never judged by this rule.

The message names the orphan coordinate and its first absent prefix. Its code is `orphan-section`. This is a declaration-side structural error, not a missing-citation error: it is reported even when nobody cites the orphan, and a citation to it may independently be present and resolve to the recorded coordinate.

### checks.section-heading-level: Section heading level mismatch

`[id] section_heading_levels` ([§FS-config.3.3.2](FS-config.md#332-section_heading_levels--heading-depth-against-path-depth)) sets how a citable section heading's Markdown depth must match its dotted path ([AR-scanner.2.2](../architecture/AR-scanner.md#22-section-detection)), and whether a mismatch is an error, a warning, or not reported. A mismatch the mode reports is located at the heading line. This section judges the depth of headings that already carry coordinates; the project-wide in-body Markdown ATX rule for a heading that carries none is [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading), independent of this mode. Bold labels are not headings and remain unchecked.

### checks.section-outside-declaration: Section outside a declaration

When the scanner encounters a numeric section heading, or an enabled named section heading, that is deeper than its stale declaration context but whose line lies inside no declaration body, `check` emits one located error at the heading. A numeric heading's exact message is `numbered section outside any declaration`; an enabled named heading's is `named section outside any declaration`. Both use the public code `section-outside-declaration`. Ownership is the declaration's body span ([§FS-declarations.checks.section-outside-declaration.1](FS-declarations.md#checkssection-outside-declaration1-ownership-is-the-body-span)), the rejected heading leaves the section map every consumer reads ([§FS-declarations.checks.section-outside-declaration.2](FS-declarations.md#checkssection-outside-declaration2-the-rejected-heading-leaves-the-section-map)), and the finding is reported like any other hard error ([§FS-declarations.checks.section-outside-declaration.3](FS-declarations.md#checkssection-outside-declaration3-an-ordinary-hard-finding)).

#### checks.section-outside-declaration.1: Ownership is the body span

Ownership is the body span already used for extraction and citing-side classification, not a second section-only approximation. In Markdown, a same-or-higher heading ends a declaration body even when that boundary heading is plain; a later deeper section-like heading is outside. In source, the end of a doc-comment or docstring ends ownership. A later declaration in the same comment block ends the earlier body and begins its own. An inline-spec stub owns only its single heading line, so numbered prose below the stub belongs to no stubbed declaration. A deeper section-like heading before any of those boundaries stays valid. A heading inside a Markdown fence is content and emits nothing ([§FS-show.2.5](FS-show.md#25-a-heading-inside-a-fenced-code-block-is-an-example)). Legal unmarked or plain headings are not errors under this section; adopting a general unmarked-heading policy is a separate change ([§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)).

#### checks.section-outside-declaration.2: The rejected heading leaves the section map

The rejected heading is excluded from the shared body-local section map before any consumer runs ([§FS-show.2.1.2](FS-show.md#212-section-map---toc)). It cannot resolve a citation or query, enter completion or list/size output, become an embedded-value root, participate in duplicate-section detection, or acquire an LSP navigation target. `show` and other failed queries retain their ordinary missing-section semantics rather than printing this check-only message.

#### checks.section-outside-declaration.3: An ordinary hard finding

This is an ordinary hard finding under §[§FS-check.2](FS-check.md#2-outputs)–3. Text uses the located `<path>:<line>: error: <message>` form. JSON emits `{"severity":"error","path":<path>,"line":<line>,"code":"section-outside-declaration","message":<message>,"sites":null}`. `--only section-outside-declaration` retains it and `--ignore section-outside-declaration` removes it; a retained finding contributes exit `1`, while selecting it away restores the ordinary selected-report result. Parallel and workspace scans merge the record once under the workspace-relative path, never once per stale declaration. A narrowed scan judges the complete selected file, and `--full` applies the same code and message to otherwise out-of-scope files it adds. The LSP transports the same error severity, code, message, and heading range through its shared snapshot ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)).

### checks.unmarked-heading: Unmarked Markdown heading

A Markdown ATX heading that is deeper than a declaration heading and whose line is still inside that declaration's body is an error when it is neither another declaration nor a recognized numeric or enabled named section. The containing declaration is the nearest enclosing body, so a plain heading beneath a deeper child declaration names that child, not an overlapping ancestor. This is a project-wide rule with no configuration, severity selector, or permanent opt-out ([§DF-unmarked-markdown-headings](../decisions/functional/DF-unmarked-markdown-headings.md#df-unmarked-markdown-headings-in-body-markdown-atx-headings-participate-in-the-knowledge-graph)).

Which headings participate is [§FS-declarations.checks.unmarked-heading.1](FS-declarations.md#checksunmarked-heading1-which-headings-participate). The message is [§FS-declarations.checks.unmarked-heading.2](FS-declarations.md#checksunmarked-heading2-the-message) and its suggested coordinate [§FS-declarations.checks.unmarked-heading.3](FS-declarations.md#checksunmarked-heading3-the-suggested-coordinate); its rendering is [§FS-declarations.checks.unmarked-heading.4](FS-declarations.md#checksunmarked-heading4-rendering-exit-and-selection), its flip to an error [§FS-declarations.checks.unmarked-heading.5](FS-declarations.md#checksunmarked-heading5-an-error-in-grund-0160), and what it leaves unchanged [§FS-declarations.checks.unmarked-heading.6](FS-declarations.md#checksunmarked-heading6-no-command-numbers-the-heading).

#### checks.unmarked-heading.1: Which headings participate

Only ATX headings in scanned Markdown files participate. A heading inside a backtick or tilde fence is content. A file title before the first declaration, a same-or-shallower heading that closes a declaration body, source doc-comment text, setext text, and a bold label are outside the rule. A deeper declaration and a valid numeric or enabled named section already participate in the graph and are not unmarked. `--full` keeps this convention-scoped finding narrowed to the configured scan scope, as it does other convention findings ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)).

#### checks.unmarked-heading.2: The message

The error is located at the heading line, uses code `unmarked-heading`, and has this text:

```text
unmarked heading inside <ID>; number it (<suggested heading>) as <ID>.<path>, declare an ID, or use a bold label; this became an error in grund 0.16.0
```

#### checks.unmarked-heading.3: The suggested coordinate

The suggested coordinate is guidance, not a rewrite. Its path depth follows the written ATX depth relative to the containing declaration. At each depth, grund uses the nearest preceding citable or already-suggested parent and appends one above the largest existing or earlier-suggested numeric sibling; it never fills a hole or reuses a coordinate. With no parent it starts one above the largest root numeric coordinate, or at `1` when none exists. If the authored heading skips a depth, missing parents are filled with `.1`. A named parent may therefore receive a numeric child such as `goals.1`. The suggested heading preserves the authored `#` depth and title and inserts the complete coordinate in that valid numeric or mixed form. A titleless ATX heading has no authored title to preserve, so its otherwise-identical suggestion uses the literal title `Untitled`; applying that complete suggested heading produces a recognized section and clears the error.

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

#### checks.oversized-lead.1: The message

The fixed code is `oversized-lead`, severity is `warning`, and the exact text after `<path>:<line>: ` is:

```text
<coordinate> lead is <actual> <unit>, over the configured maximum of <max>; move detail into citable child sections, or promote a child section to its own ID after running grund refs <coordinate> --summary
```

The coordinate is local for a member-local check and workspace-qualified for a workspace-root check. The two remedies preserve grounding and citation stability; the message never suggests shortening or deleting it.

#### checks.oversized-lead.2: Exit code and rendering

A warning-only run exits `0` and replaces the text `success` line; JSON uses the ordinary located finding object ([§FS-errors.5](FS-errors.md#5-json-format)). The LSP publishes the same message, line, code, and warning severity as the CLI ([§FS-lsp.1.1](FS-lsp.md#11-diagnostics)). Any simultaneous error, including `duplicate` or `duplicate-section`, still decides exit `1`; the warning neither suppresses it nor changes its priority.

#### checks.oversized-lead.3: Only the key activates it

The absent key activates no measurement or finding and leaves the existing text and JSON check output byte-identical. `--only oversized-lead` and `--ignore oversized-lead` select the finding after the complete check and never activate it.

#### checks.oversized-lead.4: Which sites it judges

An explicit-path check judges only declaration and section sites scanned at that path. `--full` adds its existing out-of-scope citation findings but does not extend this repository policy beyond declarations in the configured scan scope. In a workspace, each member's effective key governs only that member's sites; a member without the key remains silent even when another member opts in.

## why: Why this exists

A declaration's address is the one thing a repository stores about it in a thousand
places at once, so what may go wrong with a declaration is the same question as what may
go wrong with an address. Gathering the ten checks here rather than under the command
that happens to report them puts each one beside the invariant it defends, and names each
one what a reader already holds when they need it: the code out of a finding they have
just read ([§REQ-spec-section-names.code](../requirements/REQ-spec-section-names.md#code-a-check-is-named-by-its-diagnostic-code)). `check` is one reader of this contract, and not the
only one — `show` refuses the coordinate this spec calls ambiguous, `fmt` rewrites toward
the canonical form it names, and the LSP transports every finding here unchanged — which
is the other reason the checks do not live inside the command.

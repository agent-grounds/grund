# REQ-spec-content: the functional spec says what grund does now, once, at the grain of a promise

A person reads a page of the functional spec to learn what one part of `grund` does and what
they can build on; a test proves it. The person needs the promise and what they would
otherwise get wrong, the test every case and every byte. A page written at the test's grain
is too long to read whole ([§GOAL-token-economy.1](../goals.md#1-what-this-requires), [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)), and its prose is
the copy nothing checks. This binds every `FS-` declaration and every change to one; how a
section is named is [§REQ-spec-section-names](REQ-spec-section-names.md#req-spec-section-names-a-section-name-says-what-the-thing-is-and-never-moves).

## keeps: What a page says

- **Purpose.** The lead says what the thing is for, in a paragraph a reader can stop after.
- **The contract.** What it reads, does, prints and refuses, and how it exits. Where a
  program or another install reads the bytes, they are exact: exit statuses, streams, JSON
  fields, config keys and values, the citation grammar, the managed block, the section lock,
  the message shapes of [§FS-errors.2](../functional-spec/FS-errors.md#2-the-fixed-shapes) and the order output is printed in ([§FS-errors.4](../functional-spec/FS-errors.md#4-determinism)).
- **Important notes.** A limit, a surprise or a trap that a reader who knows the rest of the
  section would get wrong, in one sentence citing the decision that argues it.
- **One example**, where it shows the promise faster than words. A message in a fence is a
  line a test's golden holds; `<slot>` stands for what varies.

## homes: Where everything else lives

| Fact | Its home | A page |
|---|---|---|
| A code's meaning, severity, ramp, what turns it on, fix, note | its row in [§FS-errors.5.5](../functional-spec/FS-errors.md#55-the-check-code-catalog) | cites the row and never retypes a cell |
| How a finding is located, rendered, selected and exits | [§FS-check.2.1](../functional-spec/FS-check.md#21-report-format) and [§FS-errors](../functional-spec/FS-errors.md#fs-errors-grund-emits-messages-in-fixed-shapes) | cites them |
| A term | [§FS-terms](../functional-spec/FS-terms.md#fs-terms-the-shared-vocabulary-of-the-functional-spec-and-the-architecture), or the definition chapter that owns it | cites it |
| A fact another point states | that point | cites it |
| One input and what `grund` does with it | the test that pins it | states the promise it is an instance of |
| Why a choice was made, and what was rejected | its `DF-` or `DA-` record | cites it beside the choice |
| What `grund` did before or will do | the changelog, a decision's release note, the roadmap, an issue | nothing |
| How it is built | the architecture | nothing |
| Which sections a declaration has | `grund <ID> --toc` | nothing |

## sentence: Keep, cite or cut

Ask of each sentence, in order:

1. Does it say what `grund` did or will do, rather than what it does? Cut it. A version that
   is part of the contract, such as config `version = 2`, is not history, and a pending ramp
   is current behavior that its catalog row holds.
2. Does another point say it? Cite that point.
3. Would a reader who knows the rest of the section predict it? Then it is a detail, and its
   test is its home. If not, and guessing wrong costs a wrong verdict, lost work or a broken
   integration, it is an important note: say it once.

A sentence the binary contradicts stays, and the contradiction is reported.

## grain: When a point is its own section

A section is one promise about a class of inputs, of which several tests are instances. A
point is its own section only when it can break while its siblings hold and a reader would
look it up by its own name: a command, a flag, a config key, a finding code, an output
shape, an exit status, a term, a limit. Anything else is a sentence or list item of its
parent; a case, an argument, history and a restatement are never sections.

No section but a pointer sits more than three components below its ID. Where a section's
children are cases, all of them fold, so the parent states the promise whole. A lead says
what its children decide, never how, and promises nothing they do not, because the
exact-leaf coverage gate (`tests/integration/functional_spec_coverage.rs`) asks evidence of
leaves alone.

## writing: How it is written

Plain sentences in the present tense, saying what `grund` does; a table where the facts have
columns. A fix rewrites the paragraph so that the promise holds again, rather than appending
an exception to it. A pointer left by a move keeps its one line and gains nothing.

## fold: Folding a section

A folded section's citations move to its nearest surviving ancestor in the same change,
wherever they are written, test manifests included; a citation moves anywhere else only when
its fact moved there. A released changelog escapes the old coordinate instead, so it keeps
the one it shipped with. A fold is not a move: the ancestor still says what the old address
named, so no pointer is left ([§REQ-spec-section-names.permanence](REQ-spec-section-names.md#permanence-a-name-is-permanent)).

## examples: Before and after

History, cut (<§>FS-show.2.1.3):

> It is also the way to recover today's pre-<§>DF-show-default-token-cheap behavior: use
> `grund <ID> --full`.

A catalog cell retyped, cut, because the row says it
([§FS-section-lock.checks.section-address-changed](../functional-spec/FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address)):

> The row's *Note* says what the check compares and what writes the lock, …

A lead that lists its children, cut ([§FS-check.4.10](../functional-spec/FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)):

> What counts is <§>FS-check.4.10.2, and <§>FS-check.4.10.3 and <§>FS-check.4.10.4 are what
> does not. The message is <§>FS-check.4.10.5, …

Cases, cut to the promise (<§>FS-show.1.4.1, about 430 words):

> No hint is appended for a missing `--path`, which was named as a path on purpose; for a
> missing positional operand that is not coordinate-shaped, …; or for any `--batch` run; …

becomes "Only a positional path that does not exist and is shaped like a coordinate gets the
hint." Each exclusion stays a case: `show-batch-missing-path-no-hint`,
`show-missing-path-not-coordinate-no-hint`.

An argument, cut to its citation (<§>FS-check.4.10.12, 283 words): "It never becomes an
error, because neither remedy records that the author meant it
([§DF-unread-opted-out-block.2.3](../decisions/functional/DF-unread-opted-out-block.md#23-what-makes-a-workspace-finding-ramp-and-what-makes-one-permanent))."

An important note, kept ([§FS-section-lock.limits.edit-and-move](../functional-spec/FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change)):

> When a cited section moves and its text is edited in the same change, its recorded
> fingerprint is found nowhere, so the lock reads it as edited in place: `check` passes, and
> `fmt --write` refreshes the entry.

## why: Why this exists

A section per case makes every edge three copies: a spec sentence, a test, and the citation
between them. The sentence is the copy nothing checks, so it is the one that drifts. A page a
person reads whole is a page a person can check, and a fact said once drifts once at most.
Like [§REQ-spec-section-names](REQ-spec-section-names.md#req-spec-section-names-a-section-name-says-what-the-thing-is-and-never-moves), this is a contract on the repository's own surfaces, held by
tests rather than by argument.

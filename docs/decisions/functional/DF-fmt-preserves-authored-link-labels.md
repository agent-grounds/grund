# DF-fmt-preserves-authored-link-labels: `fmt` leaves a citation in a link its author wrote, and `check` takes that link as the index entry

**Date:** 2026-10-19

## status: Accepted

## 1. Context

Reported as [issue #825](https://github.com/agent-grounds/grund/issues/825). On a version-2
repository whose `ADR` kind keeps its records under `docs/adr`, `docs/adr/use-postgres.md`
declares `ADR-use-postgres: Use Postgres for all data`, and two files cite it from inside a link
their author wrote:

```markdown
<!-- docs/adr/README.md, the kind's index -->
- [§ADR-use-postgres: Use Postgres for all data](use-postgres.md)

<!-- docs/notes.md -->
See [the §ADR-use-postgres record](adr/use-postgres.md).
```

Both links render, and both take a reader to the record. `grund check` fails the index row anyway:

```text
docs/adr/README.md:1: error: index entry §ADR-use-postgres is not a link; unchecked in grund 0.11.0, an error in 0.12.0 — run `grund fmt --write`
```

and `grund fmt --write`, the command it names, rewrites both lines:

```markdown
- [[§ADR-use-postgres](use-postgres.md#adr-use-postgres-use-postgres-for-all-data): Use Postgres for all data](use-postgres.md)

See [the [§ADR-use-postgres](adr/use-postgres.md#adr-use-postgres-use-postgres-for-all-data) record](adr/use-postgres.md).
```

A link cannot contain a link in CommonMark, so the inner one wins: the author's link is gone
from the rendered page, its brackets and destination are printed as text around the new link,
and its text is split in two. A second `grund check` and `grund fmt --check` then pass, so
nothing reports the damage again.

Both halves ask one question. A citation is in a wrap when a `[` stands immediately before it
and `](` immediately after ([§FS-citations.wrap.recognized](../../functional-spec/FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets)); a citation that fails that test was
wrapped fresh by `fmt` and read as bare by `check`'s entry form, whatever link stood around it.
The citation inside longer link text fails the test, so `fmt` wrote a second link into the first,
and `check` asked for the command that does it.

## 2. Decision

### 2.1 `fmt` leaves the author's link as written

A citation inside a link its author wrote, the **enclosing link** of
[§FS-citations.wrap.enclosing](../../functional-spec/FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author), is neither wrapped nor re-derived, and the link keeps every byte.
That holds in a kind's index too, where the carve-out of [§FS-fmt.6.1.1](../../functional-spec/FS-fmt.md#611-a-kinds-index-entries-are-always-linkified) would otherwise
wrap the citation under `enabled = false` or in a suppressed scope: the carve-out exists so that
`check` never asks for an entry `fmt` will not write, and the author's link is already one
([§FS-fmt.6.4](../../functional-spec/FS-fmt.md#64-what-is-never-wrapped)). The citation is still a citation: the earlier passes still rewrite its token, and
`check` still resolves it, so a dangling citation or a missing section inside the text keeps its
usual finding.

### 2.2 `check` takes that link as the index entry

In a kind's index, a citation inside an enclosing link whose destination is not empty is an
entry ([§FS-check.3.18.5](../../functional-spec/FS-check.md#3185-what-an-entry-is)), so the row above satisfies [§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index) and is never
reported under [§FS-check.3.17](../../functional-spec/FS-check.md#317-index-entry-is-not-a-link) ([§FS-check.3.17.4](../../functional-spec/FS-check.md#3174-only-a-citation-fmt-would-wrap-reaches-this-rule)). The destination is the author's and is
never compared, as a wrap's is not ([§DF-index-entry-form.2.2](DF-index-entry-form.md#22-full-link-is-the-link-fmt-would-write-here-not-has-an-anchor)). Each citation in the text counts
for its own ID. An empty destination, `[the <§>ADR-use-postgres record]()`, takes a reader nowhere
and is no entry. And an enclosing link never enrolls an external source declaration: that is
the canonical link alone ([§FS-check.3.18.3](../../functional-spec/FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link)), and a link the author aimed by hand is not a
statement that the declaration belongs to the kind.

This extends [§DF-index-entry-form.2.3](DF-index-entry-form.md#23-one-link-per-id-not-every-mention): a citation is a bare entry exactly when the next
`grund fmt --write` would wrap it, and from now on it would not wrap this one. The same section
already ran the predicate one way only — it withholds the error, not credit for an entry that is
already a link — and the author's link is one.

### 2.3 On a line an older `fmt` nested, the wrap wins

`- [[§<ID>](…): Use Postgres for all data](use-postgres.md)` is read as CommonMark
reads it ([§FS-citations.wrap.enclosing.wrap-wins](../../functional-spec/FS-citations.md#wrapenclosingwrap-wins-a-wrap-inside-a-longer-text-is-a-wrap)): the inner `[§<ID>](…)` is a wrap, so `fmt`
re-derives its destination and `check` takes it as the entry, and the outer brackets are text.
The line is a fixed point of `fmt --write` and passes `fmt --check`, exactly as today. Unwrapping
it is a hand edit, and the release note says how to find the lines.

### 2.4 An image is not a link

`![a diagram of <§>ADR-use-postgres](diagram.png)` holds no enclosing link
([§FS-citations.wrap.enclosing.images](../../functional-spec/FS-citations.md#wrapenclosingimages-an-image-is-not-a-link)): `fmt` still wraps the citation in the alt text and `check`
still reads it as bare, and an image is never an index entry.

### 2.5 Only a same-line inline link, for now

Reference links, collapsed `[…][]` and definition labels, autolinks, raw HTML `<a>`, link text
that spans lines, the citation-only image, and the editor's hover are read as before
([§FS-citations.wrap.enclosing.scope](../../functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in)). Each is a separate reading with its own verdicts, and a
later change owes them.

## 3. Rejected alternative: report the row as a missing entry

The corrected verdict for the index row could have been [§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)'s error instead of a
pass: the row would stop naming a command that breaks it, and the author would be told to write
`[§<ID>](use-postgres.md): Use Postgres for all data`. That is the same false alarm under a
second name. The row is a link to the declaration a reader can follow, which is the whole job of
an index entry ([§DF-index-entry-form.2.1](DF-index-entry-form.md#21-the-entry-is-a-recognized-citation-of-the-id-written-as-a-full-markdown-link)), and the condition that keeps an entry out of
[§FS-check.3.17](../../functional-spec/FS-check.md#317-index-entry-is-not-a-link) has always credited a link that is already there
([§FS-check.3.17.5](../../functional-spec/FS-check.md#3175-anything-else-is-not-an-entry)). Demanding a shorter label would take the pen from the author of the index,
which is the one thing the entry rule was written not to do ([§DF-index-entry-form.1](DF-index-entry-form.md#1-context),
[§DF-index-entry-form.2.4](DF-index-entry-form.md#24-layout-is-free)).

## 4. Rejected alternative: unwrap the lines an older `fmt` nested

Those lines pass `fmt --check` today. Rewriting them would fail it on lines it passes, which is a
pass-to-fail move no prohibition licenses, and which bytes were the author's cannot always be
recovered from the line: the outer destination may be the author's or a later hand edit. They
stay a fixed point, and the release note points at them.

## 5. Consequences

Every verdict that moves, for `fmt --check` and for `check`, moves one way, from fail to pass; one
finding changes its rule and keeps its verdict.

- **A citation in a link its author wrote, anywhere.** `fmt --check` stops listing its line as a
  `markdown link`, and `--write` leaves its bytes.
- **The same in a kind's index.** `check` stops reporting the row as `index entry … is not a link`,
  and the ID has its entry.
- **An empty destination in a kind's index.** `[the <§>ADR-use-postgres record]()` was reported as
  not a link; it is now no entry, so the ID is reported missing from the index
  ([§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)). The verdict stays a failure, and the finding names the declaration, the index
  and the form of an entry, in the bytes that finding has shipped in.
- **A source declaration an authored link pointed at.** `fmt --write` no longer turns that link
  into one that holds the canonical link, so the declaration is not enrolled behind the author's
  back ([§FS-check.3.18.3](../../functional-spec/FS-check.md#3183-an-external-source-declaration-enrolls-by-its-canonical-link)).
- **Lines an older `grund` nested.** Unchanged: their inner wrap is re-derived, they pass both
  `check` and `fmt --check`, and only a hand edit unwraps them.
- **Nothing else.** `show` already printed the author's link as written and flattens only the
  inner wrap of a nested line ([§FS-show.3.2.1](../../functional-spec/FS-show.md#321-the-wrap-shape-it-collapses)), the missing-entry message keeps its bytes,
  and the editor's hover is the later change's.

## 6. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses each fail-to-pass move. The two halves rest on
different requirements and shipped in different releases, so each is held to the five conditions
on its own.

### 6.1 The `fmt` half

1. *Prior prohibition.* The old verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), which bounds `fmt --write` to "the citation tokens it names, and the cross-reference wrapper it owns around such a token", "and no other byte".
   A link its author wrote is no wrapper `fmt` owns, and a wrap written into its text takes the link away from every byte the author wrote around the citation: the rendered page prints them as text. The pass's own form has promised since v0.1.0 that "a reader scanning the rendered Markdown sees the citation exactly as before; only now it is clickable" ([§FS-fmt.6.2](../../functional-spec/FS-fmt.md#62-form)), and a nested line shows the reader stray brackets and a URL instead.
   The old verdict first shipped in v0.1.0, whose cross-reference pass already wrapped every citation without a `[` immediately before it. [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) was first released in v0.10.1, and every release from v0.10.1 to v0.18.0 shipped the old verdict under it. The prohibition already applied when each of those releases shipped.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form a user chose and could keep beside a new one: the old verdict is a write that breaks the author's link, and keeping it one more release would break more.
   The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at: the move is from fail to pass, so there is nothing for the user to run, and no command could tell which bytes of an already-nested line were the author's.
3. *Named release.* The release note below names the verdict move and the lines it reaches.
4. *Actionable findings.* No finding replaces the old verdict: `fmt --check` passes where it failed, and there is nothing to act on.
5. *No new licence.* This is not a licence for anything else: it reaches only a citation inside a same-line inline link its author wrote. Unwrapping the lines an older `fmt` nested, and the readings [§FS-citations.wrap.enclosing.scope](../../functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in) leaves to a later change, each owe their own route.

### 6.2 The `check` half

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): the row is a citation that resolves, in a link to its declaration, and was reported as not a link. It is none of the three classes that section exempts — not a non-canonical form, not a policy, and not a finding the spec knowingly scopes, because the spec bounded this one to where its command would act: the licence of [§FS-check.3.17.3](../../functional-spec/FS-check.md#3173-an-error-on-arrival) "is only honest while the named command is one that would actually act here", and the command it names breaks the link instead of making one.
   The old verdict first shipped in v0.12.0, the release that made the unlinked entry an error, and that sentence of [§FS-check.3.17.3](../../functional-spec/FS-check.md#3173-an-error-on-arrival) shipped with it. [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) was first released in v0.10.1, so the prohibition already applied when the old verdict first shipped, and in every release since, through v0.18.0.
2. *Accepted proof.* This record. The deprecation path does not fit, because the old verdict asks the user to run a command that damages the file it was asked to fix; one more release of it is one more release of that advice. A mechanical migration does not fit either, because the move is from fail to pass and the user has nothing to run.
   Why the corrected verdict is a pass rather than [§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)'s missing-entry error is section 3.
3. *Named release.* The release note below names the verdict move.
4. *Actionable findings.* For a link with a destination, no finding replaces the old verdict. For an empty destination, the [§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index) finding that replaces it names the declaration's heading, the index file, and the form an entry takes.
5. *No new licence.* This is not a licence for anything else. The entry form already credited a link that is there ([§FS-check.3.17.5](../../functional-spec/FS-check.md#3175-anything-else-is-not-an-entry)), so no prohibition is new, and the corrected reading creates no new obligation: an index that passed keeps passing.

## release-note: Release note

- [§DF-fmt-preserves-authored-link-labels](DF-fmt-preserves-authored-link-labels.md#df-fmt-preserves-authored-link-labels-fmt-leaves-a-citation-in-a-link-its-author-wrote-and-check-takes-that-link-as-the-index-entry), [§FS-citations.wrap.enclosing](../../functional-spec/FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author), [§FS-check.3.18.5](../../functional-spec/FS-check.md#3185-what-an-entry-is), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): **`grund fmt --cross-refs` no longer writes a link inside a link you wrote, and `grund check` takes your link as the index entry.** A citation in the text of a Markdown link, as in `See [the <§>ADR-x record](adr/x.md).` or the index row `- [<§>ADR-x: Use Postgres](x.md)`, was wrapped in a second link, which broke yours, and `check` reported the index row as `index entry … is not a link` until it was. Both lines are now left as written, and the row is the entry. **Who this affects:** a repository with a citation inside link text. The verdict moves from fail to pass: `fmt --check` stops listing those lines and `check` stops reporting those rows. Every finding that replaces the old verdict names its location and the action to take: an index row whose link has an empty destination, `[<§>ADR-x: Use Postgres]()`, is reported as a missing entry instead, at the declaration's heading, naming the index file and the form an entry takes. Lines an older `grund` already nested are left as they are; to find them, search for `[[§`, which catches a nested index row, and for the pattern `\[[^]\n]*\[§`, which catches a citation linked inside longer link text, and unwrap them by hand.

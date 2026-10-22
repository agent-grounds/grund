# DF-fmt-preserves-authored-markup: `fmt` leaves a citation in every link form its author wrote, and in an image, and `check` credits only the ones that go somewhere

**Date:** 2026-10-21

## status: Accepted

## 1. Context

Reported as [issue #839](https://github.com/agent-grounds/grund/issues/839), the follow-up
[§DF-fmt-preserves-authored-link-labels.2.5](DF-fmt-preserves-authored-link-labels.md#25-only-a-same-line-inline-link-for-now) left owed. That record kept a citation inside a link its
author wrote only where the link was a same-line inline link. In a version-2 repository whose `ADR`
kind keeps `ADR-use-postgres` ("Use Postgres for all data") and `ADR-use-redis` under `docs/adr`,
`grund fmt --write` still rewrote every other form, and `#…` below stands for
`#adr-use-postgres-use-postgres-for-all-data`:

```markdown
See [the §ADR-use-postgres rationale][pg].          → See [the [§ADR-use-postgres](adr/use-postgres.md#…) rationale][pg].
See [the §ADR-use-postgres record][].               → See [the [§ADR-use-postgres](adr/use-postgres.md#…) record][].
[the §ADR-use-postgres record]: adr/use-postgres.md → [the [§ADR-use-postgres](adr/use-postgres.md#…) record]: adr/use-postgres.md
See <https://example.com/§ADR-use-postgres>.        → See <https://example.com/[§ADR-use-postgres](adr/use-postgres.md#…)>.
<a href="adr/use-postgres.md">the §ADR-use-postgres record</a>
                                                    → <a href="adr/use-postgres.md">the [§ADR-use-postgres](adr/use-postgres.md#…) record</a>
![§ADR-use-redis](cache.png)                        → ![§ADR-use-redis](adr/use-redis.md#adr-use-redis-use-redis)
```

and link text that runs onto a second line gained a wrap on whichever line held the citation. Each
rewrite breaks the author's markup: the reference stops matching its definition, the autolink's
URL changes, the anchor holds a link, and the image loses its file. In a kind's index, `check`
reported the reference rows as `index entry … is not a link` and named the `fmt --write` that did
this, and it took the citation-only image as an entry. The editor's hover had its own copy of the
bracket test and nested an authored link in the same way.

## 2. Decision

### 2.1 One reading of the document, every link form

[§FS-citations.wrap.enclosing.scope](../../functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in) lists the forms and [§FS-citations.wrap.enclosing.shape](../../functional-spec/FS-citations.md#wrapenclosingshape-which-links-enclose) reads them:
inline links on one line or several, full, collapsed and shortcut references, definition labels,
autolinks and raw HTML `<a>` content. The reading is of the file, because a reference finds its
definition anywhere in it, and it serves `fmt`, `check`, `show` and the editor's hover alike. It is
no Markdown parser: malformed syntax it does not name keeps the reading it had.

### 2.2 Authored bytes keep every byte

The link pass leaves each form as written. A definition's destination, an autolink and an `href`
value are link destinations in everything but syntax, so [§FS-citations.zones.link-destination](../../functional-spec/FS-citations.md#zoneslink-destination-a-link-destination-is-the-url) covers
them, and no pass rewrites them. A reference key and an HTML tag are
[§FS-citations.zones.authored-markup](../../functional-spec/FS-citations.md#zonesauthored-markup-a-reference-key-and-an-html-tag-keep-their-bytes): a collapsed or shortcut reference's text and its definition's
label are one key, so both keep every byte in every pass, and no pass can split them. A bare token in
the three new destination spellings is still recognized, as it always was; only its bytes are new.

### 2.3 Credit only where the link goes somewhere

In a kind's index, a reference whose key a definition matches is an entry, as an inline link is, and
so is a complete multi-line link ([§FS-citations.wrap.enclosing](../../functional-spec/FS-citations.md#wrapenclosing-a-citation-inside-a-link-its-author-wrote-is-left-to-the-author)). An unresolved reference, a
definition's label, an autolink, an HTML anchor and an incomplete label are protected and are no
entry, because protecting a citation does not make it a link from the index to the declaration.

### 2.4 Two shapes from the review of #838, decided

A link inside the link's text, `[a [b](c.md) §<ID> d](x.md)`, keeps the balanced-bracket reading
#838 gave it: it encloses and is an entry. A backtick that never closes on its line can hide the
`]` that ends the text; such an incomplete label is left as written and is no entry, and the
inline-code zone keeps its reading ([§FS-citations.wrap.enclosing.shape](../../functional-spec/FS-citations.md#wrapenclosingshape-which-links-enclose)).

### 2.5 An image's destination is the author's

Only the citation-only image changes ([§FS-citations.wrap.enclosing.images](../../functional-spec/FS-citations.md#wrapenclosingimages-an-image-is-not-a-link)). `fmt` leaves
`![§<ID>](cache.png)` as written, and it is no index entry and no enrollment: grund 0.19.0 warned
for one release, and grund 0.20.0 retired the credit ([§FS-citations.checks.image-index-entry](../../functional-spec/FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release), [§DF-image-index-entry-lands-in-0-20](DF-image-index-entry-lands-in-0-20.md#df-image-index-entry-lands-in-0-20-a-citation-only-image-is-no-index-entry-from-0200)). `![a diagram of §<ID>](diagram.png)` keeps its
reading, and a row an older `fmt` already wrapped that way keeps its credit through the inner wrap.

## 3. Rejected alternative: retire the image's credit in the correcting release

Every other move here is from fail to pass, or from one failure to another that names its fix. The
image's is from pass to fail: an index whose only entry for an ID is `![§<ID>](…)` passes today and
would fail. No requirement forbade that verdict when it shipped. The writer's data loss
([§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)) is a prohibition on `fmt`, not on `check`, and the correction route refuses, by its
no-new-licence condition, refuses to stretch one into the other. So the image's credit takes the deprecation path
([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)): one release warns at the image's line with the literal entry to
write, and the next minor retires it. The route is complete: grund 0.19.0 warned, and grund 0.20.0
retired the credit ([§DF-image-index-entry-lands-in-0-20](DF-image-index-entry-lands-in-0-20.md#df-image-index-entry-lands-in-0-20-a-citation-only-image-is-no-index-entry-from-0200)).

## 4. Rejected alternative: read Markdown as CommonMark does

A conforming parser would decide the two review shapes differently: the inner link of a nested label
wins, and a lone backtick is literal. Adopting one would move verdicts this change has no route for,
in both directions, and would let `fmt` act on a guess where a line's code span may continue on the
next ([§FS-citations.zones.inline-code](../../functional-spec/FS-citations.md#zonesinline-code-an-inline-code-span-closes-on-a-run-of-its-own-length)). The extents grund already has decide instead, and the
forms the issue lists are added to them.

## 5. Consequences

- **A citation in a reference, a definition, an autolink, an HTML anchor or multi-line link text,
  anywhere.** `fmt --check` stops listing its line as a `markdown link`, and `--write` leaves its
  bytes. A trigger, an unmarked token under `--marker` and a shorthand inside a reference key or an
  HTML tag are left too, and a shorthand there is exempt from the mechanical demand
  ([§FS-citations.rewrite-exemption](../../functional-spec/FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite)).
- **A citation-only image.** `fmt --write` stops replacing its destination.
- **The same in a kind's index.** A reference whose key a definition matches, and a complete
  multi-line link, stop being reported as `index entry … is not a link` and are the entry. An
  unresolved reference, a definition's label, an autolink, an HTML anchor and an incomplete label
  stop being reported that way too, and the ID is reported missing from the index
  ([§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index)) at the declaration's heading, naming the index and the form an entry takes.
- **A citation-only image in a kind's index.** It kept its credit for one release, 0.19.0, and
  warned `image-index-entry` where that credit was all the index held for the ID; from 0.20.0 it
  is no entry ([§DF-image-index-entry-lands-in-0-20](DF-image-index-entry-lands-in-0-20.md#df-image-index-entry-lands-in-0-20-a-citation-only-image-is-no-index-entry-from-0200)).
- **Nothing else.** A bare citation is still wrapped, a wrap is still re-derived, a line an older
  `fmt` nested is still a fixed point, and `show` still collapses only a wrap.

## 6. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses each fail-to-pass move below; the image's pass-to-fail move takes the deprecation path instead (section 3). The two halves rest on different requirements, so each is held to the five conditions on its own.

### 6.1 The `fmt` half

1. *Prior prohibition.* The old verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), which bounds `fmt --write` to "the citation tokens it names, and the cross-reference wrapper it owns around such a token", "and no other byte". A wrap written into a reference's text, a definition's label, an autolink, an anchor or a second line of link text is no wrapper around a token `fmt` owns: it takes the author's link away. A destination written over an image's file is no such wrapper either. A token rewritten in one spelling of a reference key and not the other, wherever the two stand in different scopes, cuts the reference from its definition, and the key is the reference's, not the token's.
   The old verdict first shipped in v0.1.0, whose wrap pass already wrote into every one of these forms and read the citation-only image as a wrap. [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) was first released in v0.10.1, and every release from v0.10.1 to v0.18.0 shipped the old verdict under it. The prohibition already applied when each of those releases shipped.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form a user chose, only a write that breaks markup, and one more release of it breaks more. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at: the move is from fail to pass, so there is nothing to run.
3. *Named release.* The release note below names the verdict move and the lines it reaches.
4. *Actionable findings.* No finding replaces the old verdict: `fmt --check` passes where it failed.
5. *No new licence.* This is not a licence for anything else: it reaches only the forms [§FS-citations.wrap.enclosing.scope](../../functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in) lists and the citation-only image's destination.

### 6.2 The `check` half

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): a reference row whose key a definition matches, and a complete multi-line link, are links to the declaration a reader can follow, and they were reported as not a link, with a command that broke them. [§FS-check.3.17.3](../../functional-spec/FS-check.md#3173-an-error-on-arrival) licenses that error "only" while "the named command is one that would actually act here". A shorthand demand inside a key is the trap [§FS-citations.rewrite-exemption](../../functional-spec/FS-citations.md#rewrite-exemption-where-the-text-forbids-the-rewrite) names, an error whose only fix the tool now declines.
   The old verdict first shipped in v0.12.0, with that sentence of [§FS-check.3.17.3](../../functional-spec/FS-check.md#3173-an-error-on-arrival). [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms) was first released in v0.10.1, so the prohibition already applied when the old verdict first shipped, and in every release since, through v0.18.0.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because one more release of the old verdict is one more release of advice that damages the file. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration does not fit either, because the move is from fail to pass and there is no command to run.
3. *Named release.* The release note below names the verdict move.
4. *Actionable findings.* Where the form goes nowhere, the [§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index) finding that replaces the old verdict names the declaration's heading, the index and the form an entry takes.
5. *No new licence.* This is not a licence for anything else: an index that passed keeps passing, the image included, until the release that warned of it.

## release-note: Release note

- [§DF-fmt-preserves-authored-markup](DF-fmt-preserves-authored-markup.md#df-fmt-preserves-authored-markup-fmt-leaves-a-citation-in-every-link-form-its-author-wrote-and-in-an-image-and-check-credits-only-the-ones-that-go-somewhere), [§FS-citations.wrap.enclosing.scope](../../functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in), [§FS-citations.zones.authored-markup](../../functional-spec/FS-citations.md#zonesauthored-markup-a-reference-key-and-an-html-tag-keep-their-bytes), [§FS-citations.checks.image-index-entry](../../functional-spec/FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): **`grund fmt` no longer writes into a reference link, a definition, an autolink, an HTML anchor, link text that spans lines, or a citation-only image's destination.** `See [the <§>ADR-x rationale][x].`, `[the <§>ADR-x record]: x.md`, `<https://example.com/<§>ADR-x>`, `<a href="x.md">the <§>ADR-x record</a>` and `![<§>ADR-x](cache.png)` are left as written, and a collapsed or shortcut reference's text keeps every byte with its definition's label. **Who this affects:** a repository with a citation in one of those forms. The verdict moves from fail to pass: `fmt --check` stops listing those lines, and in a kind's index a reference to a matching definition and a multi-line link are entries. Every finding that replaces the old verdict names its location and the action to take: an unresolved reference, a definition's label, an autolink, an HTML anchor, or a label a lone backtick leaves unclosed is reported as a missing entry instead, at the declaration's heading, naming the index and the form an entry takes. A citation-only image that is an ID's only index entry warns `image-index-entry` in grund 0.19.0, naming the link to write beside it, and stops counting in grund 0.20.0. Lines an older `grund` already nested are left; to find them, search for `[[§`, for the pattern `\[[^]\n]*\[§`, and for `](` inside `<…>` or `<a …>…</a>`, and unwrap them by hand.

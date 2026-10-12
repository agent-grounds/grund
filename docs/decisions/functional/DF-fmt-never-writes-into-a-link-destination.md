# DF-fmt-never-writes-into-a-link-destination: `fmt` never writes into a link destination it does not maintain

**Status:** Accepted
**Date:** 2026-10-12

## 1. Context

[§DF-one-answer-rulings-fmt-editor](DF-one-answer-rulings-fmt-editor.md#df-one-answer-rulings-fmt-editor-fmt-writes-only-into-a-complete-link-wrapper-never-into-a-link-destination) ruled D17g of #576 in two parts. The destination part is a
candidate correction: `fmt --cross-refs --write` writes into a hand-authored link destination,
though [§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten) lists link destinations among what is never rewritten. The wrapper part,
which `[§<ID>](…` counts as a wrap and where a wrap ends, takes the warning release, and
[§DF-fmt-and-editor-answers-from-0-19](DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them) holds it. Reported as step 5 of #576,
[issue #598](https://github.com/agent-grounds/grund/issues/598).

Every pass of `fmt` but one reads a destination the same way: from the `(` after `](` to the `)`
that balances it, to the end of the line where none does
([§FS-citations.zones.link-destination](../../functional-spec/FS-citations.md#zoneslink-destination-a-link-destination-is-the-url)). The cross-reference pass checks only inline code, and
ends a wrap at the first `)` after its `](`. On this `docs/notes.md`, where `docs/fs/FS-x.md`
declares `FS-x`,

```markdown
See [the target](§FS-x) for detail.

Then [§FS-x](stale-dest and an aside (keep me) end.

Unclosed [the target](notes and §FS-x after.
```

`grund fmt --cross-refs --check` lists all three lines, and `--write` turns them into:

```markdown
See [the target]([§FS-x](fs/FS-x.md#fs-x-the-x-spec)) for detail.

Then [§FS-x](fs/FS-x.md#fs-x-the-x-spec) end.

Unclosed [the target](notes and [§FS-x](fs/FS-x.md#fs-x-the-x-spec) after.
```

The first line has a link written into a URL. The second lost `stale-dest and an aside (keep
me`: the `)` the pass stopped at closes the author's `(keep me`, not the wrap's `(`, so by the
reading every other pass uses that wrap never closes. The third has a link written into a
destination that runs to the end of the line. A second `fmt --check` then passes, so the
damage is permanent.

## 2. Decision

The cross-reference pass reads a destination as every other pass does, through the scanner's
own zone ([§FS-citations.zones.link-destination](../../functional-spec/FS-citations.md#zoneslink-destination-a-link-destination-is-the-url)), and owns only a **closed** wrap's destination.

- A citation inside a destination is left as written: the first and the third line keep their
  bytes, wherever on the line the citation stands.
- A `[§<ID>](` whose destination never closes, with a later `)` on the line, is left as written:
  neither re-derived nor wrapped fresh ([§FS-citations.wrap.recognized](../../functional-spec/FS-citations.md#wraprecognized-an-existing-wrap-is-recognized-by-its-brackets)). `check` warns at its
  line, `unclosed-wrapper`, naming the `)` to add ([§FS-citations.checks.unclosed-wrapper](../../functional-spec/FS-citations.md#checksunclosed-wrapper-unclosed-wrapper-read-differently-from-grund-0190)).
- Nothing else moves in this release. A closed wrap is still re-derived up to its first `)`, and
  a `[§<ID>](` with no `)` after it is still wrapped fresh; both change in 0.19.0 under
  [§DF-fmt-and-editor-answers-from-0-19](DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them).

## 3. Rejected alternative: wrap the second line fresh now

That is the wrapper rule of 0.19.0 itself, made without the warning release
[§DF-one-answer-rulings-fmt-editor](DF-one-answer-rulings-fmt-editor.md#df-one-answer-rulings-fmt-editor-fmt-writes-only-into-a-complete-link-wrapper-never-into-a-link-destination) requires for it: `fmt --check` would fail on a line it passes
today, with no release that says so first.

## 4. Rejected alternative: re-derive the second line up to the end of the line

The destination by the one extent runs to the end of the line, so re-deriving it would delete
more of the author's bytes, not fewer.

## 5. Consequences

The verdict moves one way, from fail to pass, for `fmt --cross-refs --check`, and `--write`
writes nothing where it wrote before.

- **The three lines.** `fmt --check` stops listing them and `--write` leaves their bytes. The
  second line now draws a warning from `check`, which leaves its exit status where it was:

  ```text
  docs/notes.md:3: warning: the link after FS-x never closes, so grund fmt --cross-refs leaves it as written, and from grund 0.19.0 it is not read as FS-x's link and FS-x is linked on its own — add the ')' that ends the link to keep it
  ```

- **Nested links an older `grund` wrote.** A `](` then `[§` that an older `fmt --write` left
  behind holds its inner citation in the outer destination, so it is left as written too, and
  its URL is no longer re-derived. Search for `]([§` to find them and unwrap them by hand.
- **The kind-index form.** `check`'s entry form already reads the destination zone, so `check` and
  `fmt` now agree on the first line there too.
- **Nothing else.** `show`, the editor, and `check`'s verdicts keep today's bytes.

## 6. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the fail-to-pass move, and each of its five conditions
holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), which already applied when that verdict shipped: it arrived in `fb12d5b85f`, was first released in v0.10.1, and is present in v0.17.0, whose `fmt` writes into each of the three lines.
   That section bounds `fmt --write` to "the citation tokens it names, and the cross-reference wrapper it owns around such a token", "and no other byte".
   A hand-authored destination is no wrapper `fmt` owns, and [§FS-fmt.2.3](../../functional-spec/FS-fmt.md#23-what-is-never-rewritten) has listed link destinations as never rewritten since v0.1.0. A wrap whose `(` never closes owns no byte after its `](`, because the extent every other pass has read since that release runs its destination to the end of the line.
   This reaches only those writes. A closed wrap, and a `[§<ID>](` with no `)` after it, keep every verdict.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form a user chose and could keep beside a new one: the old verdict is a write that destroys text, and keeping it one more release would destroy more.
   The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because no command can bring the deleted bytes back.
3. *Named release.* The release note below names the verdict move and the three lines it reaches.
4. *Actionable findings.* For the first and the third line, no finding replaces the old verdict: `fmt --check` passes where it failed, and there is nothing to act on. For the second, the `unclosed-wrapper` warning names the file, the line and the `)` to add ([§FS-citations.checks.unclosed-wrapper](../../functional-spec/FS-citations.md#checksunclosed-wrapper-unclosed-wrapper-read-differently-from-grund-0190)).
5. *No new licence.* This is not a licence for anything else: the wrapper rule and the balanced-parenthesis wrap take the deprecation path under [§DF-fmt-and-editor-answers-from-0-19](DF-fmt-and-editor-answers-from-0-19.md#df-fmt-and-editor-answers-from-0-19-fmts-link-wrapper-and-the-editors-title-span-change-in-0190-after-a-release-that-announces-them), and the destination extent written into [§FS-citations.zones.link-destination](../../functional-spec/FS-citations.md#zoneslink-destination-a-link-destination-is-the-url) is the one every other pass of `fmt` has used since v0.1.0 and `check` uses today, so no prohibition is new.

## release-note: Release note

- [§DF-fmt-never-writes-into-a-link-destination](DF-fmt-never-writes-into-a-link-destination.md#df-fmt-never-writes-into-a-link-destination-fmt-never-writes-into-a-link-destination-it-does-not-maintain), [§FS-citations.zones.link-destination](../../functional-spec/FS-citations.md#zoneslink-destination-a-link-destination-is-the-url), [§FS-citations.checks.unclosed-wrapper](../../functional-spec/FS-citations.md#checksunclosed-wrapper-unclosed-wrapper-read-differently-from-grund-0190), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own): **`grund fmt --cross-refs` no longer writes into a link it does not own.** It turned `[the target](§<ID>)` into a link nested inside that link's URL. Where a `[§<ID>](` never closed, it deleted the text up to a later `)` on the line. And after a `](` that never closes, it linked a later citation on the same line, as in `Unclosed [the target](notes and §<ID> after.`, inside a destination that runs to the end of the line. All three are now left as written. **Who this affects:** a repository that holds any of these lines. The verdict moves from fail to pass: `fmt --check` stops reporting the first and the third, and the second, whose `fmt --check` passes too, now draws the `check` warning `unclosed-wrapper` at its line, which leaves the exit status where it was. Every finding that replaces the old verdict names its location and the action to take: the `)` that ends the link. Nested links an older `grund` already wrote are left alone too; search for `]([§` to find them and unwrap them by hand.

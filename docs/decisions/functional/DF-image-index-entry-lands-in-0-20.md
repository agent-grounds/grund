# DF-image-index-entry-lands-in-0-20: a citation-only image is no index entry from 0.20.0

**Date:** 2026-10-22

## status: Accepted

## 1. Context

[§DF-fmt-preserves-authored-markup](DF-fmt-preserves-authored-markup.md#df-fmt-preserves-authored-markup-fmt-leaves-a-citation-in-every-link-form-its-author-wrote-and-in-an-image-and-check-credits-only-the-ones-that-go-somewhere) made a citation that is an image's whole description,
`![§<ID>](cache.png)`, no link to the declaration ([§FS-citations.wrap.enclosing.images](../../functional-spec/FS-citations.md#wrapenclosingimages-an-image-is-not-a-link)). `fmt`
stopped writing into it at once, but `check` took the image's credit away by the deprecation path
([§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)), because that move is from pass to fail: 0.19.0 kept the image as
an index entry and an enrollment, and warned `image-index-entry` where that credit was all the
index held for the ID, naming 0.20.0 and the link to write beside it. That record's notice shipped
with 0.19.0, and a notice already released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so
the release that retires the credit needs a notice of its own, and this record carries it.

## 2. Decision

In 0.20.0 (agent-grounds/grund#930), `check` reads a citation-only image as `fmt` already does
([§FS-citations.checks.image-index-entry](../../functional-spec/FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release)):

- in a kind's index the image is no entry and enrolls nothing;
- an ID it was the only entry for is reported `missing-index-entry`, an error at the
  declaration's heading, saying the ID appears in the index but not as an entry
  ([§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index), [§FS-check.3.18.5](../../functional-spec/FS-check.md#3185-what-an-entry-is)), and never `unlinked-index-entry`, because `fmt` leaves the
  image as written;
- an external source declaration it alone enrolled is no longer a member ([§FS-check.3.18.4](../../functional-spec/FS-check.md#3184-what-does-not-enroll)), and
  the image is an ordinary citation of it;
- the code `image-index-entry` is spent: it never fires, `--only` and `--ignore` still accept it,
  and `--only image-index-entry` says so and names `missing-index-entry` ([§FS-check.1.4.1](../../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so),
  [§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog)).

## 3. Rejected alternatives

- **Keep the credit for another release.** 0.19.0's warning named 0.20.0, and the release guard
  refuses a version that reaches a promise it does not keep ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). Moving the
  date would make the warning 0.19.0 printed false.
- **Retire the code with its catalog row.** An `--ignore image-index-entry` left in a CI line would
  then name an unknown code and be refused with exit `2` ([§FS-check.1.4](../../functional-spec/FS-check.md#14-selecting-findings-with---only-and---ignore)), a verdict move no 0.19.0
  message announced, and `grund config explain image-index-entry` could no longer say what the
  code was. Spending it, as `excluded-stub-target` and `stub-shape` were spent, keeps every
  selector parsing and names the finding that replaced it.

## release-note: Release note

- [§DF-image-index-entry-lands-in-0-20](DF-image-index-entry-lands-in-0-20.md#df-image-index-entry-lands-in-0-20-a-citation-only-image-is-no-index-entry-from-0200), [§FS-citations.checks.image-index-entry](../../functional-spec/FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release), [§FS-check.3.18](../../functional-spec/FS-check.md#318-declaration-missing-from-its-kinds-index), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **A citation-only image in a kind's index is no longer an entry.** What grund 0.19.0 announced with the `image-index-entry` warning lands: an index row such as `![§<ID>](cache.png)` neither lists `<ID>` nor enrolls an external source declaration. An ID whose only entry was such an image is reported as `missing-index-entry`, an error at the declaration's heading that says the ID appears in the index but not as an entry, so `grund check` exits `1`, and an external declaration that only such an image enrolled is no longer a member of the index. The `image-index-entry` code is spent: it never fires, and `--only` and `--ignore` still accept it. **Who this affects:** an index that drew the `image-index-entry` warning in 0.19.0: write the link that warning named, `[§<ID>](<link>)`, beside the image, which may stay. Closes [issue #930](https://github.com/agent-grounds/grund/issues/930).

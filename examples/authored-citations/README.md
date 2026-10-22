# Authored citations: every link you write is yours

This runnable mini-repository demonstrates [§FS-citations.wrap.enclosing.scope](../../docs/functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in). It is for authors who write their own links around a citation, in any form Markdown has, and want `grund fmt` to leave those links alone while still checking the citation inside. Open `repo/docs/notes.md` first, then `repo/docs/adr/README.md`, the `ADR` kind's index.

Run it from the repository root:

```bash
grund check examples/authored-citations/repo
echo $?    # 1
```

stdout is the golden below; stderr is empty:

```text
docs/adr/use-redis.md:1: error: ADR-use-redis appears in docs/adr/README.md but not as an entry: an entry is a `§`-marked Markdown link to the declaration — became an error in grund 0.13.0
docs/adr/consider-memory.md:1: warning: declared but never cited: ADR-consider-memory (its index entry at docs/adr/README.md:5 does not count as a use)
```

`docs/notes.md` cites `ADR-use-postgres` once in each form a link is written in: an inline link, an inline link whose text runs onto the next line, a reference link, a collapsed and a shortcut reference, an autolink and a raw HTML `<a>`. `grund fmt --write --marker --cross-refs examples/authored-citations/repo` writes nothing to it. A wrap there would put a link inside your link, which Markdown does not allow, and `fmt` does not touch your destination either. Each of those citations is still resolved and checked: rename the declaration and every one of them is reported dangling. Only the first, a citation alone in its link, is `fmt`'s own wrap, which it keeps pointing at the declaration's heading.

A reference key is your markup too ([§FS-citations.zones.authored-markup](../../docs/functional-spec/FS-citations.md#zonesauthored-markup-a-reference-key-and-an-html-tag-keep-their-bytes)). `[the §ADR-use-postgres record][]` is found by its definition `[the §ADR-use-postgres record]: …`, so both spellings are one key. No pass writes into either one: not the cross-reference wrap, not `--marker`, not a shorthand expansion, and not the editor's live `$$` conversion. A pass that changed one spelling and not the other would unlink the reference.

The index's second row is a citation that is an image's whole description. An image is no link to the declaration ([§FS-citations.wrap.enclosing.images](../../docs/functional-spec/FS-citations.md#wrapenclosingimages-an-image-is-not-a-link)). So it is no index entry either ([§FS-citations.checks.image-index-entry](../../docs/functional-spec/FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release)), and it is all the index holds for `ADR-use-redis`: `check` reports the ID missing at its declaration, saying the index mentions it but not as an entry. That error is the exit code 1. Write `[§ADR-use-redis](use-redis.md#adr-use-redis-use-redis-for-the-cache)` beside the image to clear it; the image may stay, and the separate unused warning below remains.

## When only the index cites a record

`ADR-consider-memory` is listed in the index, but no other document or code uses
it yet. The warning above names the entry that does not count as use
([§FS-check.4.1.2](../../docs/functional-spec/FS-check.md#412-an-index-entry-does-not-count)).
Refs still lists that citation and labels it, so the two answers explain each
other ([§FS-refs.3.1.1](../../docs/functional-spec/FS-refs.md#311-label-the-excluded-index-sites),
[§FS-refs.3.3.1](../../docs/functional-spec/FS-refs.md#331-identify-the-excluded-sites-within-a-summary)):

```text
$ grund refs ADR-consider-memory examples/authored-citations/repo
docs/adr/README.md:5: §ADR-consider-memory (index entry)
$ grund refs ADR-consider-memory examples/authored-citations/repo --summary
docs/adr/README.md: 1 (line 5) (index entries at line 5)
```

The summary still counts one citation. Its annotation identifies the excluded
line, rather than removing the entry. Keep the record if it remains useful;
when something depends on it, cite that dependency where it is made. Do not add
an unrelated citation merely to clear the warning.

## Authored-link trade-offs

- A link you wrote is never re-derived. If its target file moves, `fmt` does not follow it; only `fmt`'s own wrap is kept current.
- In a kind's index, your link is the entry only where it goes somewhere: an inline link with a destination, or a reference whose key a definition matches. An unresolved reference, a definition's label, an autolink and an HTML anchor are left as written but are not entries, so the ID is reported missing.
- A citation-only image is no index entry. grund 0.19.0 still took it as one, with a warning; from grund 0.20.0 the ID it alone listed is reported missing.

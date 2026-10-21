# Authored citations: every link you write is yours

This runnable mini-repository demonstrates [§FS-citations.wrap.enclosing.scope](../../docs/functional-spec/FS-citations.md#wrapenclosingscope-every-form-a-link-is-written-in). It is for authors who write their own links around a citation, in any form Markdown has, and want `grund fmt` to leave those links alone while still checking the citation inside. Open `repo/docs/notes.md` first, then `repo/docs/adr/README.md`, the `ADR` kind's index.

Run it from the repository root:

```bash
grund check examples/authored-citations/repo
echo $?    # 0
```

stdout is the golden below; stderr is empty:

```text
docs/adr/README.md:4: warning: index entry §ADR-use-redis is an image, not a link; write `[§ADR-use-redis](use-redis.md#adr-use-redis-use-redis-for-the-cache)` in docs/adr/README.md — this warning becomes an error in grund 0.20.0
```

`docs/notes.md` cites `ADR-use-postgres` once in each form a link is written in: an inline link, an inline link whose text runs onto the next line, a reference link, a collapsed and a shortcut reference, an autolink and a raw HTML `<a>`. `grund fmt --write --marker --cross-refs examples/authored-citations/repo` writes nothing to it. A wrap there would put a link inside your link, which Markdown does not allow, and `fmt` does not touch your destination either. Each of those citations is still resolved and checked: rename the declaration and every one of them is reported dangling. Only the first, a citation alone in its link, is `fmt`'s own wrap, which it keeps pointing at the declaration's heading.

A reference key is your markup too ([§FS-citations.zones.authored-markup](../../docs/functional-spec/FS-citations.md#zonesauthored-markup-a-reference-key-and-an-html-tag-keep-their-bytes)). `[the §ADR-use-postgres record][]` is found by its definition `[the §ADR-use-postgres record]: …`, so both spellings are one key. No pass writes into either one: not the cross-reference wrap, not `--marker`, not a shorthand expansion, and not the editor's live `$$` conversion. A pass that changed one spelling and not the other would unlink the reference.

The index's second row is a citation that is an image's whole description. An image is no link to the declaration ([§FS-citations.wrap.enclosing.images](../../docs/functional-spec/FS-citations.md#wrapenclosingimages-an-image-is-not-a-link)). It is the only entry for `ADR-use-redis`, so `check` warns and names the line to write beside it ([§FS-citations.checks.image-index-entry](../../docs/functional-spec/FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release)). The warning leaves the exit code at 0. Add that line and the run prints `success`.

The trade-offs:

- A link you wrote is never re-derived. If its target file moves, `fmt` does not follow it; only `fmt`'s own wrap is kept current.
- In a kind's index, your link is the entry only where it goes somewhere: an inline link with a destination, or a reference whose key a definition matches. An unresolved reference, a definition's label, an autolink and an HTML anchor are left as written but are not entries, so the ID is reported missing.
- The image keeps its credit as an index entry for grund 0.19.0 only. From grund 0.20.0 it is no entry, and the ID it alone listed is reported missing.

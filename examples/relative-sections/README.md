# Relative sections: write only the component a heading adds

This runnable mini-repository demonstrates [§FS-declarations.section.relative](../../docs/functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds) and the section lock that guards it, [§FS-section-lock](../../docs/functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went). `FS-login` writes its subsections as `### .1: Username` under `## format: Formatting`, and `docs/use.md` cites two of them. The lock records one citation and misses the other.

Run it from the repository root:

```bash
grund check examples/relative-sections/repo
echo $?    # 1: FS-login.format.2 is cited but not in the lock
```

A relative heading takes its path from the heading one level up. `### .1:` under `## format:` is `FS-login.format.1`, and `### .1:` under `## parse:` is `FS-login.parse.1`. Every place that shows a coordinate prints the full path. `grund show FS-login --toc` lists `### format.1: Username`, while the default read and `--full` print the heading as it was written.

A relative address has a cost: move the block under another parent and the same words answer to a different address. So a cited relative section is locked to its text. `.grund/sections/FS-login.lock` holds one line per cited address with a fingerprint of the section's text. Only `grund fmt --write`, run from the root, writes that file, and the file is committed. Here the lock records `FS-login.format.1`. Nothing records `FS-login.format.2`, so `check` prints the golden `unrecorded-section-citation` error below.

Run `grund fmt --write` on a copy of the repository: it adds the missing line, and `check` then prints `success`. Later, if `## format:` and `## parse:` swap their `### .1` blocks, `check` says where the cited text went. Accepting that move takes a pointer in the old section or an edit of the citations.

The trade-offs:

- Source doc-comments still take complete paths. The relative form exists only in Markdown declaration bodies.
- Moving and editing a cited section in one change goes unnoticed: the lock treats it as an in-place edit ([§FS-section-lock.limits.edit-and-move](../../docs/functional-spec/FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change)). The only trace is the refreshed line in the lock diff, so review it.
- Complete-path headings need no lock. If a section's address must never move, write its full path.

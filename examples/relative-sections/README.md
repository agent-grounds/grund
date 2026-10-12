# Relative sections: write only the component a heading adds

This runnable mini-repository demonstrates [§FS-declarations.section.relative](../../docs/functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds). It is for authors of deep outlines who are tired of repeating the parent's path in every heading. `FS-login` writes its subsections as `### .1: Username` under `## format: Formatting`, and `docs/use.md` cites two of them. Open `repo/docs/functional-spec/FS-login.md` first, then `repo/docs/use.md`.

Run it from the repository root:

```bash
grund FS-login --toc examples/relative-sections/repo
echo $?    # 0
```

The section map on stdout is the golden below; stderr is empty:

```text
## format: Formatting
### format.1: Username
### format.2: Password
## parse: Parsing
### parse.1: Tokens
```

A relative heading takes its path from the heading one level up. `### .1:` under `## format:` is `FS-login.format.1`, and `### .1:` under `## parse:` is `FS-login.parse.1`. So the citation `<§>FS-login.format.1` in `docs/use.md` resolves to the Username section, and the map prints that full path for you to copy. Every place that shows a coordinate prints the full path. The default read and `--full` print the heading as it was written.

A relative address has a cost: move the block under another parent and the same words answer to a different address. So a cited relative section is locked to its text ([§FS-section-lock](../../docs/functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)). `repo/.grund/sections/FS-login.lock` holds one line per cited address with a fingerprint of the section's text, and `grund check examples/relative-sections/repo` prints `success`. Only `grund fmt --write`, run from the root, writes that file, and the file is committed. The sibling example [`relative-sections-lock/`](../relative-sections-lock/) shows the lock catching a citation it does not record.

The trade-offs:

- Source doc-comments still take complete paths. The relative form exists only in Markdown declaration bodies.
- Moving and editing a cited section in one change goes unnoticed: the lock treats it as an in-place edit ([§FS-section-lock.limits.edit-and-move](../../docs/functional-spec/FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change)). The only trace is the refreshed line in the lock diff, so review it.
- Complete-path headings need no lock. If a section's address must never move, write its full path.

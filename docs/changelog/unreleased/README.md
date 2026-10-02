# Unreleased changelog entries

Every change that has merged and is not yet released is one file in this directory. Two pull requests that each record a change add two files, so they merge in either order without touching a line in common. The release collects the files into the changelog and deletes them. This README stays.

## Part one: the format

This part is shared. A repository that takes the format copies everything above part two byte for byte and writes its own part two.

1. **One file per pending change**, in `docs/changelog/unreleased/`. This `README.md` describes the format and is not an entry.
2. **The file name** is `<slug>.<category>.md`, or `<slug>.md` in a changelog without sections.
   - The slug is lowercase letters, digits and hyphens. It is unique in the directory and stays with the entry for life. The branch name is a good default.
   - The category is one lowercase word. It is published under the same word capitalized, so `fixed` becomes `### Fixed`.
3. **One bullet per file**, exactly as published. It starts with `- `. It may wrap, with continuation lines indented two spaces.
4. **Links** are relative to the file, as a formatter writes them. The release rewrites them for where the entry lands.
5. **The number.** An entry may end with `(PR #N)` naming its own pull request, with `(PR #TBD)`, or with neither. A release writes the number at the end of the entry and nowhere else.
6. **The release** groups entries by category and puts the oldest-landed first, breaking ties by file name. It then deletes the files.

## Part two: how this repository uses it

The rules are [§FS-distribution.4.12](../../functional-spec/FS-distribution.md#412-pending-changelog-entries-are-one-file-each), and a pull request that adds no entry is refused before the push and again in CI ([§FS-distribution.4.6](../../functional-spec/FS-distribution.md#46-the-changelog-gate-keeps-the-release-section-mappable-to-its-pull-requests)).

- The category is required and is one of `added`, `changed`, `deprecated`, `removed`, `fixed` or `security`. Sections are released in that order.
- A `**Schema:**` callout is a `changed` entry whose text begins with `**Schema:**`.
- Bullets stay on one line by convention.

For a branch named `fix/issue-379` that changes behavior, the entry is `docs/changelog/unreleased/fix-issue-379.changed.md`.

# Unreleased changelog entries

Each changelog entry waiting for a release is one file in this directory. No two entries share a line, so entries written apart merge in either order. The release collects the files into the changelog and deletes them. This README stays.

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

The rules are [§FS-distribution.4.12](../../functional-spec/FS-distribution.md#412-pending-changelog-entries-are-one-file-each).

- The category is required and is one of `added`, `changed`, `deprecated`, `removed`, `fixed` or `security`. Sections are released in that order.
- A `**Schema:**` callout is a `changed` entry whose text begins with `**Schema:**`.
- Bullets stay on one line by convention.

A pull request adds no entry. Whoever cuts a release writes the entries first, in one pull request, the write-up ([§FS-distribution.4.6](../../functional-spec/FS-distribution.md#46-the-release-lists-the-pull-requests-merged-since-the-previous-tag)):

- **The list** is every pull request merged since the latest `vX.Y.Z` tag that changed more than docs and CI. This prints it:

  ```sh
  tag="$(git describe --tags --abbrev=0 --match 'v*.*.*' origin/main)"
  gh pr list --state merged --base main --limit 200 \
    --search "merged:>$(git log -1 --format=%cI "$tag")" \
    --json number,title,files \
    --jq '.[] | select(any(.files[].path; test("^(docs/|\\.github/)|\\.md$|^(LICENSE|lychee\\.toml)$") | not)) | "#\(.number) \(.title)"'
  ```

  Each gets an entry, or is named in one. Skip any whose entry is already in this directory: it ships as written.
- **The words** come from the issues the pull request closed, which its description names (`Closes #N`), or from its own description where it closed none.
- **The number** is the pull request's own: every entry ends in `(PR #N)`. The release leaves such an entry as it stands, and cannot supply a forgotten one: a number it would give several entries goes into none of them, with a warning for each, but a lone entry without one takes the write-up's own number.
- **The slug** is the pull request's branch name, its `/` written as `-`.
- **A verdict correction** is the one exception: it brings its own entry, in the pull request that makes it ([§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)).

For pull request #395, merged from `fix/issue-295`, the write-up adds `docs/changelog/unreleased/fix-issue-295.fixed.md`:

```markdown
- [§FS-examples.5.3](../../functional-spec/FS-examples.md#53-commandargs-is-read-as-shell-words-or-refused): the e2e runner reads `command.args` as shell words, and refuses what it cannot honour. (PR #395)
```

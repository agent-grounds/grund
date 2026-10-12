# DF-four-editor-checks-read-saved-text: four editor checks read the saved file for one more release

**Status:** Accepted
**Date:** 2026-10-12

## 1. Context

A file's text is what a save would write, for every reader a run has ([§FS-scan.text.buffer](../../functional-spec/FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write)).
In the editor, four readers still take the saved file instead of the buffer. They are rows
R06a to R06d of [§REQ-one-answer.differences.text](../../requirements/REQ-one-answer.md#differencestext-text), and each was reproduced on `main` at
f9e72785c8:

- **R06a, the index check.** An index entry edited to a bare citation is clean in the
  editor and `unlinked-index-entry` once saved.
- **R06b, the inline-code hint.** A dangling citation edited into inline code keeps the
  `(or write <§>… if this is an illustration)` hint in the editor, read off the saved line.
- **R06c, the `AGENTS.md` block check.** An edit to the managed block is judged on the old
  disk text until saved.
- **R06d, the end-to-end manifest reader.** An edit to a case's `expected.exit` is read from
  disk until saved, so `show` of the case answers the old exit code.

#595 makes `sources` the only reader of a file's text ([§AR-system.2.13](../../architecture/README.md#213-sources)). These four
readers move into it with everything else, so whether each reads the buffer becomes one
choice in one place.

## 2. Decision

For one release the four readers read `sources`' `saved` view: the disk past the buffers,
which is the answer they give today. In the release after it, 0.19.0, each one reads the
run's text, as every other reader does, and rows R06a to R06d leave the list
([#624](https://github.com/agent-grounds/grund/issues/624)).

No requirement forbade these four answers. None of them is a citation reported broken or
resolved wrongly. R06a is a form finding on an index. R06b is message text, and message text
always changes through a warning release. R06c and R06d are not about citations at all. So
under [§DF-one-answer-rulings-run](DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held) (D7, every other part) they take the deprecation path of
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path).

## 3. The warning is a notice, not a diagnostic

The warning for this release is the compatibility notice below, published in the release
notes. The editor gets no diagnostic for it. No saved repository's verdict moves, so there
is nothing for a repository to migrate. A diagnostic in the editor would cost a second pass
of each check on every keystroke, only to tell the user to save.

## 4. Rejected alternatives

- **Flip them now.** Without a requirement the old answer broke, a verdict moves only after
  a warning release, as the ticket's author answered on #576.
- **Leave their reads outside `sources`.** That would keep four of the very copies #595
  removes, and the flip would again mean finding them.

## release-note: Release note

- [§DF-four-editor-checks-read-saved-text](DF-four-editor-checks-read-saved-text.md#df-four-editor-checks-read-saved-text-four-editor-checks-read-the-saved-file-for-one-more-release), [§FS-scan.text.buffer](../../functional-spec/FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **from 0.19.0 the editor judges four more checks on unsaved text.** Until then, as now, the editor reads the saved file for the index check (`unlinked-index-entry`), the inline-code hint on a dangling citation, the `AGENTS.md` managed-block check and the end-to-end manifest reader. From 0.19.0 each reads the open buffer, as every other check already does, so an unsaved edit shows the finding a save would. **Who this affects:** only editor users, and only before a save. The command line, CI and the bindings read no buffers, and no saved repository's verdict changes, so there is nothing to migrate. Tracked in [issue #624](https://github.com/agent-grounds/grund/issues/624).

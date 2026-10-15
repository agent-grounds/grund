# DF-a-buffer-is-the-file: an editor buffer is the file it names, under every spelling and with no file saved behind it

**Date:** 2026-10-12

## status: Accepted

## 1. Context

The editor hands a run the buffers it holds open, and [§FS-declarations.checks.broken-stub.1](../../functional-spec/FS-declarations.md#checksbroken-stub1-the-target-is-read-as-a-save-would-write-it)
already said a stub's target is judged on the text a save would write. Three trees gave
the editor an answer that saving the same bytes clears. Each was reproduced on `main` at
f9e72785c8 through `lsp_snapshot` and `show_with_overlays`, with the edit held as a buffer
and then saved.

- **A target that exists only as a buffer.** `docs/a.md` stubs `../src/a.rs`, and
  `src/a.rs` is open in the editor, never saved, declaring `FS-a`. The editor reports
  `stub link target missing: ../src/a.rs` and `duplicate declaration of FS-a`, and `show`
  answers `ambiguous ID`. The stub gate asked the disk whether the target is a file. This
  was row R05 of [§REQ-one-answer.differences.text](../../requirements/REQ-one-answer.md#differencestext-text).
- **A target behind a symlink.** `docs/a.md` stubs `../alias/a.md`, where `alias -> notes`.
  A buffer of `notes/a.md` that removes the declaration leaves the stub healthy until the
  save. A buffer that adds it, opened as `notes/a.md` or as the stub's own spelling
  `alias/a.md`, leaves `stub link target lacks FS-a: ../alias/a.md` until the save. Buffers
  were looked up by the spelling the walk met, never by the file. This was row R04.
- **A buffer-only file the walk would not read.** An unsaved `docs/gen/b.md` declaring
  `FS-a`, in a directory `.git/info/exclude` ignores, makes `FS-a` a duplicate in the
  editor and an `ambiguous ID` to `show`. Saved, the walk skips it and the tree passes. The
  editor's unsaved files had a hand-written copy of the walk's filter that skipped global
  ignore files, `.git/info/exclude` and unwalked homes. #576 listed this case, and planning
  #595 reproduced it.

[§DF-one-answer-rulings-run](DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held) records the rulings these trees fall under. D7 says a
buffer with no file behind it exists, and names its reproduced false answers as a candidate
for the correction route. D6 says file identity is physical, for buffers too.

## 2. Decision

A buffer is the file it names. It stands for that file under every spelling that reaches the
file ([§FS-scan.identity.buffers](../../functional-spec/FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location)). A buffer with no file behind it is a file of the run:
it exists for a stub, and the walk reads it or skips it by the rules that would apply to the
saved file ([§FS-scan.text.buffer-only](../../functional-spec/FS-scan.md#textbuffer-only-a-buffer-with-no-file-behind-it-is-a-file)). In every tree above, the editor now gives the
answer the saved tree gives, before the save ([§FS-scan.text.buffer](../../functional-spec/FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write)). Rows R04 and R05
leave [§REQ-one-answer.differences.text](../../requirements/REQ-one-answer.md#differencestext-text).

D6 is still a warning-release ruling in general. R04 takes the correction route because of
the answer it gave, not because of how identity works. Its false answer is D7's: a stub
reported broken until the same bytes are saved. It reaches that answer through a buffer the
lookup misses, not through a buffer with no file behind it. The ticket's author ruled that
R04 goes by the correction route, in this record, when approving the proposal on
[#595](https://github.com/agent-grounds/grund/issues/595).

## 3. Rejected alternative: keep the lookup by spelling for one release

R04 could keep the lookup by spelling for one release, ship a warning, and flip in the next.
That would keep a lookup known to be wrong alive inside `sources`, the one component that
exists to be the only reader of a file's text ([§AR-system.2.13](../../architecture/README.md#213-sources)). It would also keep a
false `broken-stub` that only a save clears. That is the alarm the correction route below
says no release may keep.

## 4. Consequences

The verdict moves only in the editor, and only on these trees. No finding's wording changes.

- **Fail to pass.** The editor no longer reports `stub link target missing` or
  `duplicate declaration` for a stub to an unsaved new target that declares the ID, and
  `show` answers its body. A buffer that adds the declaration to a target behind a symlink
  repairs the stub at once, under either spelling. An unsaved file in an ignored or
  excluded directory no longer makes an ID a duplicate.
- **Pass to fail.** A buffer that removes the declaration from a target behind a symlink
  breaks the stub at once. Before, the break appeared only after the save. A citation that
  resolved only through an unsaved file in an ignored directory now dangles in the editor,
  just as it does after the save.
- **Finding moved.** An unsaved new target that lacks the ID now reports
  `stub link target lacks <ID>: <path>` instead of `stub link target missing: <path>`.
  That is the finding the saved file gets.

The command line, CI, pre-commit hooks and the Node and Python bindings hold no buffers, so
none of their answers moves. A Rust caller of `show_with_overlays` or `lsp_snapshot` sees the
editor's new answers.

## 5. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the move in one release, and each of its five conditions holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms), read through [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which already applied when that verdict shipped: both arrived in e98a2cc9e9, first released in 0.10.1, and both are present in 0.17.0, whose stub gate asks the disk whether a target is a file and whose editor looks buffers up by spelling.
   A stub whose target, read as a save would write it, declares the ID is healthy ([§FS-declarations.checks.broken-stub.1](../../functional-spec/FS-declarations.md#checksbroken-stub1-the-target-is-read-as-a-save-would-write-it)), so `stub link target missing` and `stub link target lacks` on it are false alarms, and so is the `duplicate declaration` its pairing failure makes. An `ambiguous ID` answered for an ID with one declaration resolves its citations to a conflict that does not exist, which is the wrong resolution [§DF-stub-pairs-with-unscanned-target.5](DF-stub-pairs-with-unscanned-target.md#5-the-correction-route) corrected.
   The same holds for a stub reported healthy over a buffer that no longer declares its ID, and for a duplicate made by a file the walk does not read: each is an answer the same bytes saved do not give.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form to keep beside a new one, only a wrong answer that saving the same bytes clears, and nothing a user could write selects it. [§DF-one-answer-rulings-run](DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held) refused the warning release for D7 for the same reason.
   The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because there is nothing in the repository to rewrite: the stubs, targets and ignore files are right as written.
3. *Named release.* The release note below names the verdict move in both directions, and says who sees it.
4. *Actionable findings.* Every finding that replaces the old verdict is one the saved tree already gets: `broken-stub` at the stub line, naming the target and what it lacks ([§FS-declarations.checks.broken-stub](../../functional-spec/FS-declarations.md#checksbroken-stub-broken-inline-spec-stub)), or a dangling citation at its line. Each names its location and the edit to make.
5. *No new licence.* This is not a licence for anything else. No rule, finding or prohibition is new, and no other overlay policy moves. The four editor checks that still read the saved file keep it for one release and take the deprecation path ([§DF-four-editor-checks-read-saved-text](DF-four-editor-checks-read-saved-text.md#df-four-editor-checks-read-saved-text-four-editor-checks-read-the-saved-file-for-one-more-release)). A stub to a target the walk excludes or ignores stays healthy until its own warning release ([#622](https://github.com/agent-grounds/grund/issues/622)).

## release-note: Release note

- [§DF-a-buffer-is-the-file](DF-a-buffer-is-the-file.md#df-a-buffer-is-the-file-an-editor-buffer-is-the-file-it-names-under-every-spelling-and-with-no-file-saved-behind-it), [§FS-scan.text.buffer-only](../../functional-spec/FS-scan.md#textbuffer-only-a-buffer-with-no-file-behind-it-is-a-file), [§FS-scan.identity.buffers](../../functional-spec/FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.2](../../requirements/REQ-no-wrong-citation.md#2-no-false-alarms): **the editor judges an unsaved file as the saved file, before the save.** A file that exists only as an editor buffer exists, so a stub to it is no longer `stub link target missing` and its ID no longer a `duplicate declaration`. A buffer is the file under every spelling, so an unsaved edit to a stub's target behind a symlink moves the stub at once. A buffer-only file in a directory the walk skips is not read, as the saved file would not be. **Who this breaks:** only editor users, and only before a save. The verdict moves both ways: a stub whose symlinked target loses its declaration in a buffer is `broken-stub` at once, and a citation that resolved only through an unsaved file in an ignored directory dangles. Every finding that replaces the old verdict is one the saved file already gets, naming its location and the fix. The command line, CI and the bindings see no change. Closes [issue #595](https://github.com/agent-grounds/grund/issues/595).

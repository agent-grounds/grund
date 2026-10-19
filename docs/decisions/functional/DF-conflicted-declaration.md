# DF-conflicted-declaration: a declaration that still holds an unresolved merge conflict fails `check` and stops `fmt` from writing

**Date:** 2026-10-19

## status: Accepted

## 1. Context

A merge that stops on a conflict writes both sides of it into the file, between marker lines, and
leaves the resolving to the author. Until this record, grund read those lines as ordinary text. A
declaration body that held both a `2 to 32 characters` and a `3 to 64 characters` rule for one
username passed `grund check` with `success`, and `grund fmt --write` refreshed the section lock
entry of its cited `.1` to a fingerprint of the conflicted text, so the half-merged body was
recorded as the section's settled text. The lock's own conflict markers were already handled
([§FS-section-lock.write.rebuild](../../functional-spec/FS-section-lock.md#writerebuild-a-lock-grund-cannot-read-is-rebuilt)); a declaration's were not, because no point said what a conflict
in a body is. Reported as [issue #837](https://github.com/agent-grounds/grund/issues/837).

Reproducing it showed a second fault in `fmt`. A run that needs the whole declaration set refuses
on an unreadable path and says `nothing was rewritten`, but with cross-references off, the lock
pass and a shorthand's deferred scan both take their proof after the walk has already rewritten
earlier files, so the message was false on those runs.

## 2. Decision

A conflict block is defined once, at [§FS-declarations.body.conflict](../../functional-spec/FS-declarations.md#bodyconflict-a-body-still-in-the-middle-of-a-merge): column-0 marker lines of
exactly seven `<`, `|`, `=` or `>` ([§FS-declarations.body.conflict.markers](../../functional-spec/FS-declarations.md#bodyconflictmarkers-a-marker-line)), cut into blocks with
the middle markers read only inside an open block ([§FS-declarations.body.conflict.block](../../functional-spec/FS-declarations.md#bodyconflictblock-a-conflict-block)), looked
for in every line of a Markdown file that declares an ID, fences included, and in the body of
every source declaration, the line right after a cut line-style run included
([§FS-declarations.body.conflict.where](../../functional-spec/FS-declarations.md#bodyconflictwhere-where-blocks-are-looked-for)), and naming every declaration it overlaps
([§FS-declarations.body.conflict.owners](../../functional-spec/FS-declarations.md#bodyconflictowners-which-declarations-a-block-names)).

`check` reports each block as the error `conflict-marker` at its first line
([§FS-declarations.checks.conflict-marker](../../functional-spec/FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)), from the release that carries this record. `fmt`
takes the same block from the model as a refusal: no run that needs the whole declaration set
writes anything while one exists ([§FS-fmt.7.4](../../functional-spec/FS-fmt.md#74-no-write-without-a-complete-model)), and each block is named in a
`nothing was rewritten` line ([§FS-fmt.3.3](../../functional-spec/FS-fmt.md#33-a-refused-run-still-names-every-unreadable-path), [§FS-fmt.3.4](../../functional-spec/FS-fmt.md#34-a-refusal-says-that-nothing-was-rewritten)). And `fmt` writes nothing before the last
refusal it can meet ([§FS-fmt.7.4.3](../../functional-spec/FS-fmt.md#743-nothing-is-written-before-the-last-refusal)), which makes the unreadable-path message true as well.

Two readings were settled while the plan was reviewed. A conflict inside a `///` or `#` run cuts
the run, because a marker line is not a comment line, so the block that opens on the line right
after the run is counted as the declaration's: otherwise `fmt --write`, which runs before any
compiler, would fingerprint the cut-short body. And whether a Markdown file declares an ID is
read off its raw lines, fences ignored, because a conflict can leave the file's one declaration
inside a fence it never closes.

## 3. Rejected alternative: a warning first

[§FS-declarations.checks.declaration-near-miss](../../functional-spec/FS-declarations.md#checksdeclaration-near-miss-declaration-near-miss) warned for a release before it became an error,
and that window let a repository move to the new form. A conflict block has no new form to move
to, and during a window `fmt --write` would keep fingerprinting conflicted text into the lock,
which is the harm this record stops.

## 4. Rejected alternative: look only in declaration bodies, outside fences

The first plan exempted fences and source declarations. A conflict does not respect either: the
merge writes its markers wherever the two sides differ, and a fence or a doc-comment is as much
the declaration's text as a paragraph. A fence the conflict opens and never closes would also
hide the markers that caused it. So every line of a declaring Markdown file is read, and a spec
that has to show a marker line literally indents it by one space, which still renders in a fence.

## 5. Rejected alternative: guard each write site

The lock pass and the shorthand's deferred scan each wrote after the walk had written. A guard
beside each was the shape [§FS-fmt.7.4.1](../../functional-spec/FS-fmt.md#741-why-this-rule-is-structural) records failing twice. Planning every write before making
any puts each later pass before the first write by construction.

## 6. Consequences

Only a tree that holds a block where [§FS-declarations.body.conflict.where](../../functional-spec/FS-declarations.md#bodyconflictwhere-where-blocks-are-looked-for) looks moves:

- **`check` goes from `0` to `1`.**

  ```console
  $ grund check
  docs/login.md:7: error: unresolved merge conflict in FS-login, lines 7–11: resolve it by hand
  ```

- **`fmt --write` goes from `0` to `2`, and `fmt --check` from `0` or `1` to `2`,** on every run
  that needs the whole declaration set, with the tree left byte for byte as it was:

  ```console
  $ grund fmt --write
  error: nothing was rewritten: docs/login.md:7: unresolved merge conflict in FS-login, lines 7–11: resolve it by hand
  ```

- **Embedders** get one new public type, `FmtConflictAbort`, and the bindings one new failure
  code, `conflict-marker`, under the failure kind `operation` ([§FS-distribution.3.3.2](../../functional-spec/FS-distribution.md#332-operational-failures-and-invalid-arguments)).

A run that refuses on an unreadable path with cross-references off, a lock to write or a
shorthand to expand no longer rewrites the files before it. Its exit code stays `2` and its
message was already saying that nothing was rewritten, so no verdict moves there. A spec that
shows a column-0 marker line literally, and a seven-level blockquote written `>>>>>>> `, now
reads as a conflict and is written indented or as `> > > > > > >`.

Four things do not move. A tree with no block keeps every verdict. A lock in conflict alone is
rebuilt exactly as before. A run that needs no declaration set, such as a source-only path, reads
no block and makes its `$$` and `--marker` rewrites as before ([§FS-fmt.3.2](../../functional-spec/FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)). And `show`, `refs`,
`list` and `cover` read the file as they did, `show` still printing the bytes it reads.

## 7. The correction route

[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses the pass-to-fail move, and each of its five conditions
holds.

1. *Prior prohibition.* The old verdict violated [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which already applied when that verdict shipped: it is present in every released tag since v0.16.0, and in v0.18.0, the first to ship the section lock.
   A section coordinate "resolves to the declaration's recorded heading or fails", and a lookup that can be satisfied two ways must have its rule written down. A conflicted body answers one address with two texts and no rule picks one, yet `check` reported `success` on it.
   The lock makes that a persisted wrong answer, which [§REQ-no-wrong-citation.3](../../requirements/REQ-no-wrong-citation.md#3-no-wrong-write) forbids of the writer in the same tags. Suppose the other branch moved the username rule from `.1` to `.2`. The old `fmt --write` refreshed `.1`'s entry to the conflicted text, so after the resolution that fingerprint is found at no address and the entry reads as edited in place. `section-address-changed`, which exists to report exactly that move, never fires, and every citation of `FS-login.format.1` silently reads other text.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because a conflict block is no old form with a new one to keep beside it, and a warning window would let `fmt --write` go on fingerprinting conflicted text for its whole length.
   The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration has no command to point at, because no command can choose a side of a conflict: only the author knows which text is the declaration's.
3. *Named release.* The release note below names the verdict move for `check` and for `fmt`.
4. *Actionable findings.* The finding that replaces the old verdict is located at the block's first line, names the declarations it overlaps and the lines it spans, and says to resolve it by hand ([§FS-declarations.checks.conflict-marker](../../functional-spec/FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)); `fmt`'s refusal carries the same location and message.
5. *No new licence.* This is not a licence for anything else. The issue called the old verdict correct under the contract of its day, and of the functional specification that is true: no point defined a conflict block. The prohibition is [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), which predates the verdict; this change writes the specification point that applies it and invents none. A marker line where [§FS-declarations.body.conflict.where](../../functional-spec/FS-declarations.md#bodyconflictwhere-where-blocks-are-looked-for) does not look keeps its verdict.

## release-note: Release note

- [§FS-declarations.checks.conflict-marker](../../functional-spec/FS-declarations.md#checksconflict-marker-unresolved-merge-conflict), [§FS-fmt.7.4.3](../../functional-spec/FS-fmt.md#743-nothing-is-written-before-the-last-refusal), [§DF-conflicted-declaration](DF-conflicted-declaration.md#df-conflicted-declaration-a-declaration-that-still-holds-an-unresolved-merge-conflict-fails-check-and-stops-fmt-from-writing),
  [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-wrong-citation.1](../../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution): **a declaration that still holds an
  unresolved merge conflict fails `check`, and `fmt` writes nothing while one exists.** A
  column-0 conflict block in a Markdown file that declares an ID, fences included, or in a source
  declaration's body is the new error `conflict-marker`, at the block's first line and naming
  every declaration it overlaps. **Who this breaks:** the verdict moves from pass to fail for
  `grund check` on such a tree (exit `0` becomes `1`), and every `fmt` run that needs the whole
  declaration set refuses it and rewrites nothing, so `fmt --write` exits `2` where it exited `0`
  and `fmt --check` exits `2` where it exited `0` or `1`; Python raises `OperationError` and Node
  rejects with kind `operation`, both with code `conflict-marker`, and Rust returns the new
  `FmtConflictAbort`. A spec that shows a marker line literally indents it by one space, and a
  seven-level blockquote is written `> > > > > > >`. A run refused for an unreadable path no
  longer rewrites files before refusing. Every finding that replaces the old verdict names its
  location and the action to take: resolve the conflict by hand. Closes
  [issue #837](https://github.com/agent-grounds/grund/issues/837).

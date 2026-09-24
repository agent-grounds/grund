# DF-undeclared-blind-spots: two skips no section named become located findings

**Status:** Accepted
**Date:** 2026-09-24

## 1. Context

Two regions of text `grund check` did not read were nowhere written down, and the run
said nothing about either.

- **A qualified citation whose tail misses the fallback grammar.** A run that loads no
  workspace recognises the `<§><alias>/` prefix before applying any ID grammar, and reads
  the tail with a fixed `KIND[-NUM]-SLUG` shape because the target's `[id] format` is
  unreachable there ([§FS-workspace.5.2](../../functional-spec/FS-workspace.md#52-recognition-in-a-run-that-loads-no-workspace)).
  The scanner used that shape to decide *recognition* as well: a tail it declined —
  a lowercase kind, a slug-only grammar that does not split on `-`, a kind carrying
  characters outside `[A-Z0-9]` — produced no citation at all, so a citation into a
  namespace that does not exist passed the check.
- **A kind home reached through a linked parent directory.** The scan-root gate walked
  every component below the project root and refused the root when *any* of them was a
  directory symlink. [§FS-config.3.5.1](../../functional-spec/FS-config.md#351-a-symlink-in-the-tree-is-followed)
  gates link *traversal* and says "the same gate applies when a configured or explicit
  scan root is itself a directory symlink" — the root itself, not an ancestor. A
  repository whose `docs/` sits behind a link and whose `[scan] include` names
  `docs/spec` therefore had its whole documentation tree skipped.

```console
$ grund check .                     # docs -> ../shared-docs, include = ["docs/spec"]
warning: nothing to scan — grund looked under [scan] include = ["docs/spec"] …
$ echo $?
0                                   # every citation under docs/spec unread
```

Both are the shape [§FS-workspace.5.2](../../functional-spec/FS-workspace.md#52-recognition-in-a-run-that-loads-no-workspace)
and [§FS-config.3.5.1](../../functional-spec/FS-config.md#351-a-symlink-in-the-tree-is-followed)
already describe; neither is the shape the code had.

## 2. Decision

### 2.1 The tail grammar reads a tail and never decides recognition

[§FS-workspace.5.2](../../functional-spec/FS-workspace.md#52-recognition-in-a-run-that-loads-no-workspace)
states it in as many words: "the unknown thing is the alias, which needs no tail grammar,
so a tail that does not match it … is an `unknown project alias` error all the same". The
loose parse stays exactly where it was — it is how a recognised tail is *read* — and a
tail outside it now yields a citation carrying the token's own bytes. Nothing resolves
through that ID: the run has no workspace catalogue, so the alias is unknown or unverified
and the citation ends at the alias. The workspace-root run, which parses each tail with
the target project's grammar, remains the one place the tail itself is judged.

### 2.2 The root gate asks about the root, not its ancestors

A scan root written below a linked directory traverses no link — the walk starts under it
— so it is ordinary scope, and the canonical-root fence still refuses a root that *is* an
outward link. That keeps the gate's purpose (a link may not carry a walk out of the
project) while ending the skip it was also performing.

## 3. Rejected alternative: narrow the two specification points to the code

Write the fallback grammar into [§FS-workspace.5.2](../../functional-spec/FS-workspace.md#52-recognition-in-a-run-that-loads-no-workspace)
as a recognition condition, and widen [§FS-config.3.5.1](../../functional-spec/FS-config.md#351-a-symlink-in-the-tree-is-followed)
to every linked ancestor.

It fails on what the reader would then be owed. A declared blind spot has to be
*bounded and answerable on demand*, and neither of these can be: "citations whose tail
your target project's ID format happens not to split on `-`" is not a region anybody can
enumerate, and "anything behind a directory link anywhere above the root" silently grows
every time a repository is checked out through one. Writing them down would satisfy the
letter of the requirement and none of its purpose.

## 4. Consequences

- A repository that today passes while writing `<§>unknown/anything` starts failing, with
  a located `unknown project alias` finding naming the alias. A repository whose kind home
  sits behind a directory link starts being scanned, and the findings under it are new.
  Both verdicts move from `0` to `1`.

  **Which clause licenses the move.** [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) does, because the old silence violated the separately declared prohibition in [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded): "a skip that no section names is the bug", and no section named either of these two. That requirement already applied when the verdict shipped; this decision records the conflict rather than inventing a prohibition for the correction.

  Neither ordinary route fits. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path cannot apply, because there is no old form carried beside a new one — the citation and the file were never read, so there is nothing to warn about first. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical-migration route has no command the tool ships that completes the change: what a repository must do is edit a citation or move a home, and no writer owns either. Nor does [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) exempt them: the old runs produced output and exit `0` had a defined meaning.

  The correcting release names the verdict change, and every finding that replaces the old verdict names its location and the action the maintainer can take — the alias finding anchors at the citation's own `path:line` and names the alias to correct, and a finding under a newly read home is an ordinary located `check` finding. This is the bounded correction route and not a licence for ordinary policy tightening, feature removal, or a newly invented prohibition.
- A repository with no qualified citations and no directory links is byte-identical. The
  fixed `KIND[-NUM]-SLUG` shape still governs how a recognised tail is read, so no
  citation that resolves today resolves differently.
- Neither change touches the workspace-root run, which already used each target project's
  own grammar, nor the ownership boundary a loaded workspace adds
  ([§FS-workspace.6](../../functional-spec/FS-workspace.md#6-nested-project-boundary)).

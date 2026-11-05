# DF-fetch-declaration-outside-home: a folder-home fetch refuses an ID declared outside the home

**Date:** 2026-11-05

## status: Accepted

## 1. Context

`grund fetch <ID>` looked for the existing declaration in a `folder` home by walking only
that home. A declaration anywhere else was not seen, so the fetch took the create branch of
[§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write), exited 0 and wrote `<home>/<ID>.md` beside it. Where a stub in the home pointed at
that declaration, the walk read the stub's heading as a declaration and replaced it. Both
leave the same tree: a green one whose next `grund check` fails on a duplicate, with a message
that does not say fetch caused it, and a second fetch that exits 0 and changes nothing.

One shape stays green. Under version 1 with `[scan] include = ["fs"]`, a stub in `fs/` whose
target `src/a.py` is outside the include is the one declaration of its ID
([§FS-declarations.stubs.pairing.1](../../functional-spec/FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)). The fetch overwrote the stub, nothing was left to collide
with, `grund check` stayed green, and `grund show` changed from `Local body.` to `Fetched
body.` with nothing to say so. Reported as
[issue #1214](https://github.com/agent-grounds/grund/issues/1214).

## 2. Decision

[§FS-fetch.5.3](../../functional-spec/FS-fetch.md#53-a-declaration-outside-the-home-prevents-installation) refuses: exit 2, naming the ID and the declaring file, with empty stdout and the
tree byte-identical. Two answers lose.

- **Saying that a declaration outside the home is not looked for.** That would make the
  duplicate the documented result and fetch's exit 0 a promise of a red check, and it would
  bless overwriting a stub, a line fetch does not own ([§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership)).
- **Replacing or rewriting the declaring file.** Its bytes around the declaration are not the
  requested declaration's, and a Markdown file outside the configured home is no snapshot.

What declares is the scan's answer, read through its one pairing of an ID's declarations, so a
stub is judged by the verdict every command reads and fetch keeps no reader of its own. That
is what reaches the green shape: for a target outside the include the scan holds only the
stub, and only the stub's verdict says what it stands for. A healthy stub in the home points at
a declaration outside it, so it refuses and stays. A healthy stub outside the home points at
the home's own declaration, so that declaration is replaced under it. A broken stub stays a
declaration of its own, wherever it sits: outside the home it refuses, and in the home, with
nothing declared elsewhere, it is replaced. That last rule is what makes the refusal's way
through hold on a stub, because a target with the declaration taken out of it breaks the stub.

This refusal comes ahead of [§FS-fetch.5.2](../../functional-spec/FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation)'s, which asks only the home's own files, and the two
read alike. Where both apply the file to name is the one outside the home: taking a declaration
out of the home leaves the one outside it standing, and a stub under another name points at that
same file, so naming the stub would send the maintainer to a line that holds no fact.

The scan asked is the selected project's ([§FS-fetch.1](../../functional-spec/FS-fetch.md#1-input-and-project-selection)), so another workspace project declaring
the same local ID does not count: one local ID in two projects is two namespaces, not a
duplicate.

A path outside the home that the scan cannot read does not stop the fetch. Such a path may hold
the declaration, but refusing on that would make an unrelated unreadable file gate every fetch
in the project, which nothing reported, and the tree it leaves was never green: `grund check`
already exits 2 on it, so an exit 0 turns nothing red. The line between this and a discovery
failure is the one [§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run) already draws. A file, a directory or a link the walk steps
past is a per-file failure of the run, and that includes a config root it cannot list, which
[§FS-check.2.4](../../functional-spec/FS-check.md#24-an-incomplete-run) reports the same way and fetch already let through. A scan that could not run at
all has no answer to read and is a discovery failure under [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write).

## 3. Compatibility

The verdict of `grund fetch <ID>` moves from exit 0 to exit 2 for a tree in which a file outside
the home declares the ID, on the route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids).
The prior verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) by installing a declaration of an ID another file already held, or by writing over a stub.
Fetch may replace or insert only its validated named declaration. A second declaration of an ID
another file holds replaces nothing and is no insertion of that declaration, and a stub whose
target declares its ID is a pointer to that declaration rather than that declaration
([§FS-declarations.stubs.pairing.1](../../functional-spec/FS-declarations.md#stubspairing1-a-stub-pairs-with-its-target-whether-or-not-the-target-is-scanned)).
That prohibition already applied when the old verdict shipped, and [§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership) extends it to fetch.
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because a warning window would keep installing the duplicate and keep overwriting the stub, and the line a stub's author wrote is not recoverable from the tree that follows.
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit, because no one command can choose between keeping the fact where it is written and moving it to a fetched snapshot.
The maintainer can, and the refusal names the file and both ways through.

The trees that lose a successful fetch are of two sorts, and only the first is answered by
what `check` would have said. In a tree whose target or declaration the scan reads, the next
check was already going to fail on the duplicate, so exit 2 says earlier what the tool already
refuses later. A declaration in the home beside one outside it was already red before the fetch.

The version 1 stub is not answered that way, because that tree was green before the fetch and
green after it. What answers it is what the fetch did to it. The maintainer wrote a stub, and
the prior success replaced that line with a copy of the external fact, so the project's one
declaration of the ID moved from `src/a.py` to `fs/` and `grund show` began answering from
there, while `src/a.py` went on holding text nothing reads any more. That is the data loss
[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) exists to prevent, with no finding left to catch it, so it is the shape
that most needs the refusal, not the one that can do without it. A maintainer who wants the
fetched text takes the declaration out of the target, or removes the stub, and fetches again.

Checking and querying an unchanged tree keep their verdicts.
This record is not a licence for refusing a file the scan does not read, for letting an unrelated finding gate fetch, or for treating another project's declaration as this project's: each of those still gets its `<ID>.md`.

## release-note: Release note

- In grund 0.22.0, `grund fetch <ID>` on a `folder` home refuses with exit 2 when a file outside
  that home already declares the ID, in a source comment or in Markdown, and where a stub in the
  home points at such a file it is the file that is named. It used to exit 0 and write
  `<home>/<ID>.md` beside that file, or overwrite the stub, so a green tree failed its next
  `grund check` on a duplicate; under version 1, with the stub's target outside `[scan]
  include`, it replaced the stub with nothing going red and `grund show` quietly answered with
  the fetched text. The refusal leaves every file as it was. Keep the fact in that file, or take
  the declaration out of it and fetch again, which creates or replaces the snapshot as before. A
  file the scan does not read, or cannot read, outside the home does not stop the fetch. This
  verdict changes under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids): the prior success violated
  [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), and every finding names its location and the action to take
  ([§FS-fetch.5.3](../../functional-spec/FS-fetch.md#53-a-declaration-outside-the-home-prevents-installation), [§DF-fetch-declaration-outside-home](DF-fetch-declaration-outside-home.md#df-fetch-declaration-outside-home-a-folder-home-fetch-refuses-an-id-declared-outside-the-home)).

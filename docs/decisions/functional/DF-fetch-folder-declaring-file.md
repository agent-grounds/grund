# DF-fetch-folder-declaring-file: a folder-home fetch refuses an ID a file it does not replace already declares

**Date:** 2026-11-05

## status: Accepted

## 1. Context

`grund fetch <ID>` looked for the existing declaration in a `folder` home by
walking the home as the scan does and then setting aside every file not named
`.md`. A comment declaration in a file the scan reads as a source language, or
as Markdown under another name, was not seen, so the fetch took the create branch
of [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write), exited 0 and wrote `<home>/<ID>.md` beside it. The tree had been
green, and the next `grund check` failed on a duplicate whose message does not
say fetch caused it; a second fetch exited 0 and changed nothing. Python, Rust,
Go and Ruby comment forms all went this way, under both config versions, wherever
in the home the file sat. Reported as
[issue #1213](https://github.com/agent-grounds/grund/issues/1213).

## 2. Decision

[§FS-fetch.5.2](../../functional-spec/FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation) refuses: exit 2, naming the ID and the declaring file, with empty
stdout and the tree byte-identical. Two answers lose.

- **Replacing the declaring file.** Read literally, the replace branch of
  [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write) reaches this file. Its bytes around the declaration are code,
  which the requested declaration does not own ([§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership)), so the first
  sentence of [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write) now says "the Markdown file".
- **Rewriting the comment in place and keeping the code.** That is a new writer
  for every comment form the scan reads, with its own rules for indentation,
  markers and the shape of a fetched body. Nothing asks for it, and the refusal
  leaves it open.

Which comment declares stays the scan's answer: the home's files go through the
scan's own pass, so a file that only cites the ID, or holds its text where nothing
declares, is no reason to refuse. A file that pass cannot read or decode cannot be
said not to declare the ID, so it is a discovery failure, which [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write)
already refuses, and `grund check` already exits 2 on that tree.

The refusal comes ahead of everything about the home's Markdown files, so a
Markdown declaration of the same ID cannot hide it. Replacing that Markdown file
would leave the duplicate standing and exit 0. A stub in the home whose target is
the declaring file is the same case: replacing it would turn a green tree red.

A declaration outside the home is a separate question. Discovery still asks only
the home's own files, and this record does not move that.

## 3. Compatibility

The verdict of `grund fetch <ID>` moves from exit 0 to exit 2 for a home holding
such a file, on the route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids).
The prior verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) by installing a declaration of an ID another file already held.
Fetch may replace or insert only its validated named declaration. A second
declaration of an ID another file holds replaces nothing and is no insertion of
that declaration, and replacing the declaring file would erase what it holds around
the declaration.
That prohibition already applied when the old verdict shipped, and [§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership) extends it to fetch.
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because a warning window would keep installing the duplicate and leave the same red tree behind it.
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit, because no one command can choose between keeping the fact in the authored file and moving it to a fetched snapshot.
The maintainer can, and the refusal names the file and both ways through. The
trees that lose a successful fetch are those whose next check was already going
to fail on the duplicate, and a home with a file the scan cannot read, where check
already exits 2. Checking and querying an unchanged tree keep their verdicts.
This record is not a licence for refusing a file the scan does not read, or for
letting an unrelated finding gate fetch: a file that does not declare the ID still
gets its `<ID>.md`.

## release-note: Release note

- In grund 0.22.0, `grund fetch <ID>` on a `folder` home refuses with exit 2 when a
  file in that home it does not replace already declares the ID: a comment
  declaration in a source language, or a file read as Markdown under a name other
  than `.md`. It used to exit 0 and write `<home>/<ID>.md` beside that file, so a
  green tree failed its next `grund check` on a duplicate. The refusal leaves every
  file as it was. Keep the fact in that file, or take the declaration out of it and
  fetch again, which creates or replaces the snapshot as before. A file in the home
  the scan cannot read or decode is now refused the same way, where `grund check`
  already exits 2. This verdict changes under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids): the
  prior success violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), and every finding names its location
  and the action to take ([§FS-fetch.5.2](../../functional-spec/FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation), [§DF-fetch-folder-declaring-file](DF-fetch-folder-declaring-file.md#df-fetch-folder-declaring-file-a-folder-home-fetch-refuses-an-id-a-file-it-does-not-replace-already-declares)).

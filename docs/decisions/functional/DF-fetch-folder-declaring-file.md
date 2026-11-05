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
already refuses. Section 3 says what that does to a tree's verdict.

The refusal comes ahead of everything about the home's Markdown files, so a
Markdown declaration of the same ID cannot hide it. Replacing that Markdown file
would leave the duplicate standing and exit 0. A stub in the home whose target is
the declaring file is the same case: replacing it would turn a green tree red.

A declaration outside the home is a separate question. Discovery still asks only
the home's own files, and this record does not move that.

## 3. Compatibility

The verdict of `grund fetch <ID>` moves from exit 0 to exit 2 for a home holding
such a file. Checking and querying an unchanged tree keep their verdicts. Four
kinds of tree lose a successful fetch, as far as the runs below and a reading of the
code reach, each on its own ground, and only the first is a correction of a prohibition.

**The next check was going to fail on the duplicate.** This is what the issue
reports, and it takes the route of [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids).
On these trees the prior verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) by installing a declaration of an ID another file already held.
Fetch may replace or insert only its validated named declaration. A second
declaration of an ID another file holds replaces nothing and is no insertion of
that declaration, and replacing the declaring file would erase what it holds around
the declaration. That is a reading of "only its validated named declaration": the old
fetch wrote one new file in the home and changed no other byte, and what this record
holds against it is that another file already held the ID.
That prohibition already applied when the old verdict shipped, and [§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership) extends it to fetch.
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because a warning window would keep installing the duplicate and leave the same red tree behind it.
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit, because no one command can choose between keeping the fact in the authored file and moving it to a fetched snapshot.
The maintainer can, and the refusal names the file and both ways through.

**A tree `grund check` already fails on for another reason.** A file in the home the
scan cannot read or decode makes `grund check` exit 2, and a stub under another name
whose target does not exist makes it exit 1. Neither was the passing repository the
lead of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) speaks of, so fetch's verdict moves on a tree that
was already failing. No prohibition is claimed for these, and they do not rest on
condition 1 of the route above. One of them loses something real: with `fs/a.txt` read
as Markdown and holding a stub whose target is `FS-a.md`, not there yet, check fails on
`stub link target missing`, 0.21.0 exited 0 creating the target and check then passed,
and this change refuses, with check still failing until the stub is removed, after which
fetch exits 0 and check exits 0.

**The scan does not read `<ID>.md` as Markdown, and a file in the home declares
the ID:** version 2 `languages = ["python"]`, version 1 `extensions = ["py"]`,
`md = "python"`, or a snapshot path that `.gitignore` or `[schema.sources] exclude` keeps
out. With a snapshot like the one below these trees pass before the fetch and after it,
on 0.21.0 as on this change, so `grund check` does not move on them, only fetch's exit.
[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) is not what they violated, so condition 1 of the route above does
not carry them, and this record does not claim it does. Nor does the first case of
[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise), whose words are
"A construct that had **no defined meaning** and produced no output was not a working feature, so giving it one is not a break".
Neither half holds as written: [§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write) said to replace the declaring file,
the literal reading section 2 turns down, and the old fetch exited 0 and wrote a file.
The nearest they come is that nothing takes that file for the declaration: `grund list`
and `grund <ID>` stay with the source file. The path the requirement leaves them is the
default one, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): a release that warns and keeps the old
success, then a refusal no earlier than the next. Refusing them now instead is a
judgement this record makes rather than something the requirement grants, named so that
a person can overrule it. What can be said for it: with such a snapshot `grund check`
does not move on them, and where no language reads `<ID>.md` as Markdown,
[issue #1206](https://github.com/agent-grounds/grund/issues/1206) already takes
[issue #1197](https://github.com/agent-grounds/grund/issues/1197), that fetch exits 0 on
a snapshot no language reads as Markdown, as one of its acceptance cases. With `fs/a.py`
declaring `FS-a` under `languages = ["python"]`, 0.21.0 ran:

```text
$ grund fetch FS-a           # exit 0, writes fs/FS-a.md
$ grund list                 # exit 0, the declaration is still the source file's
FS-a  fs/a.py:1  Alpha
$ grund FS-a --full          # exit 0, the source file's body, not the fetched one
Local body.
$ grund check                # exit 0, fs/FS-a.md goes unmentioned
fs/a.py:1: warning: declared but never cited: FS-a; cite it
$ grund fetch FS-a           # exit 2, nothing having changed
error: snapshot file <project>/fs/FS-a.md already exists for a different ID
```

`extensions = ["py"]` and a snapshot path kept out of the scan ran the same. Under those
no command reads the file's bytes, and a later fetch stops on the file being there.
`md = "python"` differs: the scan reads the snapshot as Python from the first fetch on, so
a comment in it cites (`grund refs` lists it, and one naming an unknown ID fails the next
check), and fetch reads it as Markdown, so the second fetch exits 0 and rewrites it over a
hand edit, with `grund list` and `grund FS-a` still reading the source file. On these
trees the second way the refusal names does not help by itself. With the docstring taken
out, fetch exits 0 and writes the same unread snapshot, or exits 2 with
`already exists for a different ID` where an earlier fetch left one and the scan does not
read it at all;
`grund FS-a` says `ID not found: FS-a`, and a file citing the ID fails check with
`unknown reference FS-a; no snapshot in fs — run grund fetch FS-a`. Both versions do this,
and whether fetch may write there at all is
[issue #1206](https://github.com/agent-grounds/grund/issues/1206)'s. There are two ways
through here. Keep the fact in the file. Or get the fetched fact: in one change, take the
docstring out and have the scan read the snapshot as Markdown, by adding `"markdown"` to
`languages` (version 1: `"md"` to `extensions`; with `md = "python"`, dropping that
mapping too; with a snapshot path kept out, removing that entry), then fetch. `grund list`
then names `fs/FS-a.md`, `grund FS-a --full` prints the fetched body and check exits 0, on
0.21.0 as on this change. Adding Markdown first, with the docstring still there and an
earlier snapshot in place, fails check on the duplicate until the docstring goes.

**A stub read as Markdown under another name, whose target is a `.md` file in the
same home:** `txt = "markdown"`, with `fs/a.txt` holding `# FS-a: [t](t.md)` and
`fs/t.md` declaring `FS-a`. These trees also pass before and after, so `grund check`
does not move on them, only fetch's exit. Here the old success was real: 0.21.0 exits 0
and replaces `fs/t.md`, and `grund FS-a --full` then prints the fetched body.
[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) is not what it violated, and it had a meaning, so neither condition 1
of the route above nor [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) carries it. The path the
requirement leaves it is the default one, [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): a release that
warns and keeps the old success, then a refusal no earlier than the next. Refusing it now
instead is a judgement this record makes rather than something the requirement grants,
named so that a person can overrule it. What can be said for it: the same tree spelled
with the stub in `fs/a.md` was already refused on 0.21.0 as ambiguous ([§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write),
[§FS-fetch.7](../../functional-spec/FS-fetch.md#7-output-and-exits)), exit 2 with `snapshot folder has multiple declarations for requested ID`, so
refusing a stub beside its target is not new in kind, though it is new on this tree, and
the old success existed only because the walk set the stub aside by its name. It is named
here and in the release note, and the refusal names the file and the way through: with
`fs/a.txt` removed, fetch exits 0 and replaces `fs/t.md`, and check exits 0. Answering it
differently would put a third behaviour in [§FS-fetch.5.2](../../functional-spec/FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation) that stops short of the same stub with
its target outside the home, which on 0.21.0 exits 0 beside it and fails the next
check on the duplicate.

This record is not a licence for refusing a file the scan does not read, or for
letting an unrelated finding gate fetch: a file that does not declare the ID still
gets its `<ID>.md`.

## release-note: Release note

- In grund 0.22.0, `grund fetch <ID>` on a `folder` home refuses with exit 2 when a
  file in that home it does not replace already declares the ID: a comment
  declaration in a source language, or a file read as Markdown under a name other
  than `.md`. It used to exit 0 and write `<home>/<ID>.md` beside that file, and
  where that left a duplicate, a green tree failed its next `grund check`. The
  refusal leaves every file as it was. Keep the fact in that file, or take the
  declaration out of it and fetch again, which creates or replaces the snapshot as
  before. A file in the home the scan cannot read or decode is now refused the same
  way, where `grund check` already exits 2. Two more kinds of tree lose the old success
  with no duplicate left behind. Where the scan does not read `<ID>.md` as Markdown
  (`languages = ["python"]`, `extensions = ["py"]`, `md = "python"`, or a snapshot path
  `.gitignore` or `exclude` keeps out), the old fetch wrote a snapshot `grund list` and
  `grund <ID>` never take for the declaration, so keep the fact in the source file, or,
  in one change, take the declaration out of it, have the scan read `<ID>.md` as Markdown
  and fetch. For a stub in a file read as Markdown under another name, whose target is a
  `.md` file in the same home, the old fetch replaced the target, or created it where the
  target was `<ID>.md`, so remove the stub and fetch again. This verdict changes under [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)
  where the fetch left a duplicate: that prior success violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own),
  and every finding names its location and the action to take ([§FS-fetch.5.2](../../functional-spec/FS-fetch.md#52-a-file-it-does-not-replace-that-declares-the-id-prevents-installation),
  [§DF-fetch-folder-declaring-file](DF-fetch-folder-declaring-file.md#df-fetch-folder-declaring-file-a-folder-home-fetch-refuses-an-id-a-file-it-does-not-replace-already-declares)).

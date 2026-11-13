# DF-fetch-unread-home: fetch refuses a home the scan does not read, before the integration runs

**Date:** 2026-11-12

## status: Accepted

## 1. Context

[[§FS-fetch.4.2](../../functional-spec/FS-fetch.md#42-a-home-read-as-source-is-refused)] refused a `file` home the text scan reads as source, and said that a home the
text scan does not read "goes on being written": a name no language or `[scan] extensions` list
claims, such as `fs.dat`, or a hidden name, such as `.fs.py`. Such a file was not read two ways,
and the refusal said nothing about it.

It is read no way at all, and that is the defect. `fetch` wrote the snapshot, exited 0, and
`grund FS-a` then said `ID not found`, while `check` warned that the file is not read. On the next
fetch nothing said where the declaration ended, so `fetch`'s own line loop took it to run to the
next H2 or the end of the file, and lines someone wrote below it were replaced, with exit 0 and
nothing printed (grund.246, the third comment on
[#1205](https://github.com/agent-grounds/grund/issues/1205)). A data file such as `fs.json`
stopped parsing on the first write ([#1206](https://github.com/agent-grounds/grund/issues/1206)).
A `folder` home whose `<ID>.md` the scan does not read, because no admitted language reads
Markdown, is the same tree: the snapshot is written, nothing reads it, and the ID is declared
nowhere after it, which is the tree [[§DF-fetch-folder-declaring-file.3](DF-fetch-folder-declaring-file.md#3-compatibility)] describes and left to
#1206.

## 2. Decision

[[§FS-fetch.4.2.1](../../functional-spec/FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)] refuses, first write included, with exit 2, empty stdout and the tree
byte-identical, by the home's path and kind and before the integration runs, in the one function
that already refuses a value source and a source home, so the refused fetch runs nothing and
`check` stops offering `run grund fetch` there ([[§FS-check.3.1.3.1](../../functional-spec/FS-check.md#3131-a-home-fetch-refuses)]). For a `folder` home the
refusal holds for the `<ID>.md` `fetch` would create, when the one scan run before the
integration finds no file in the home declaring the ID ([[§FS-fetch.6.1](../../functional-spec/FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)]). The line's shape is
[[§FS-fetch.4.2](../../functional-spec/FS-fetch.md#42-a-home-read-as-source-is-refused)]'s, and its reason is the one `check`'s unread warning gives, so the two say the
same thing and a correction to that reason moves both.

A folder home's `<ID>.md` that an ignore file ignores or an `exclude` rule matches is the same
tree: the walk does not read it, so the snapshot is written and nothing reads it. It is refused the
same way, asked of the walk's own read predicate rather than of a second reading of the ignore
files, and its `<reason>` is the rule in the broken-stub line's words, with the rule's repair in
place of a name, since the name is the ID's ([[§FS-fetch.4.2.1](../../functional-spec/FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)]). Admission's reading after
the integration asks the same predicate, so a target the walk would not read is not there.

Three answers lose.

- **Keep writing an unread home.** Admission's "the requested declaration is there once"
  ([[§FS-fetch.admission.1](../../functional-spec/FS-fetch.md#admission1-the-requested-declaration-is-there-once-and-its-body-takes-in-all-of-the-output)]) is the only clause standing between a refresh and the deleted lines,
  and a home nothing reads has no declaration to find, so an exception for it would keep the
  deletion.
- **Refuse only the refresh.** What the first fetch writes is, on the next fetch, the home that
  loses lines, so letting it through only moves the refusal one fetch later with nothing
  changed, as [[§DF-fetch-source-home.3.1](DF-fetch-source-home.md#31-the-prior-prohibition)] argued for an absent source home. The first write also
  does damage of its own, since a data file stops parsing.
- **Refuse when the configuration loads.** `check`, `list` and `show` read such a project
  correctly, and [[§DF-fetch-source-home.2](DF-fetch-source-home.md#2-decision)] rejected the same move.

This record is shared with [#1206](https://github.com/agent-grounds/grund/issues/1206), which
proposed the same refusal under its one answer to "read as Markdown": one refusal, one message
and one record, written by whichever pull request moves the verdict first and cited by the other.

## 3. Compatibility

The verdict of `grund fetch <ID>` moves from exit 0 to exit 2 for a home the text scan does not
read, and for a folder home's `<ID>.md` the walk does not read when it is absent or a link to
nothing, which 0.22.0 wrote and then never read, on the route of [[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)]. It is the licence
[[§DF-fetch-source-home.3.5](DF-fetch-source-home.md#35-what-this-licenses)] and [[§DF-fetch-folder-value-source](DF-fetch-folder-value-source.md#df-fetch-folder-value-source-folder-fetch-refuses-an-id-already-declared-by-a-value-source)] withheld and left to another
ticket.

The prior prohibition is [[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)] and [[§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write)]'s "all bytes outside the owned
declaration remain unchanged", which a refresh into a home nothing reads cannot keep, because no
read says where the declaration ends.
Both already applied when the old verdict shipped.
[[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)] does not fit, because a warning window would keep writing a
snapshot nothing reads and keep cutting lines on refresh. [[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)] does not fit, because no one command can choose the file's new name: the maintainer can, and the
refusal names the home and the way through.

The tree that worked is one where another tool reads the home as Markdown, and `grund` was only
asked to write it. Its way through is a name a listed document language reads — `.md`, or a
suffix mapped to `markdown` — with no new key, after which `fetch` writes it and every command
reads what it wrote.

`check`'s tail at such a home moves from `— run grund fetch <ID>` to
`— give the file a name the scan reads as Markdown`, or, at an `<ID>.md` an ignore file or an
`exclude` rule keeps out of the walk, to `— the scan does not read <path>: <reason>; <repair>`
([[§FS-check.3.1.3.1](../../functional-spec/FS-check.md#3131-a-home-fetch-refuses)]), in the same release, under the third
exception of [[§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change)], the one for `check`'s fetch tail, and for
its reason: a window would keep recommending on purpose a command that can only refuse. The base
of the message, the code, the severity and the exit stay what they are.

Checking and querying an unchanged tree keep their verdicts.
This record is not a licence for refusing a home the scan reads, whatever its name, or for refusing at load time a configuration `check`, `list` and `show` read correctly.

## release-note: Release note

- In grund 0.23.0, `grund fetch <ID>` refuses with exit 2, before running the integration, a
  `file` home the scan does not read — a name no listed language or `[scan] extensions` entry
  claims, such as `fs.dat`, or a hidden name such as `.fs.py` — and a `folder` home's `<ID>.md`
  the scan does not read, when no file in the home declares the ID, an `<ID>.md` an ignore file
  or an `exclude` rule keeps out of the walk included. It used to write the snapshot
  and exit 0, though nothing then read it, and a later fetch could cut lines written below it. The
  refusal names the home and the reason `check` already gives for it, and leaves every file as it
  was. Give the file a name a listed document language reads, such as `.md`, or lift the rule
  that keeps the `<ID>.md` out of the walk, and fetch again.
  `grund check` at such a home now says `give the file a name the scan reads as Markdown`, or
  names the rule and its repair, in place of `run grund fetch <ID>`. This verdict changes under
  [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids): the prior success violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), and every finding
  names its location and the action to take ([[§FS-fetch.4.2.1](../../functional-spec/FS-fetch.md#421-a-home-the-scan-does-not-read-is-refused)], [[§DF-fetch-unread-home](DF-fetch-unread-home.md#df-fetch-unread-home-fetch-refuses-a-home-the-scan-does-not-read-before-the-integration-runs)]).

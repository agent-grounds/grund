# DF-fetch-admission: fetch installs only what the next scan reads back as exactly what was asked for

**Date:** 2026-11-12

## status: Accepted

## 1. Context

`grund fetch` had its own idea of what a Markdown declaration is. One line loop validated the
integration's output and another read the home, and neither was the scanner. So a fetch could
write what the next scan read differently: content deleted with exit 0, a green tree turned red,
or a snapshot nothing reads. Thirteen issues reported a shape each, and each fix closed one shape
(#1155, #1159), while the next one opened another. Issue
[#1205](https://github.com/agent-grounds/grund/issues/1205) fused them under one ruling: the same
behaviour lives in one place, under one name, and every caller uses it.

## 2. Decision

[[§FS-fetch.admission](../../functional-spec/FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)] admits a candidate through the selected project's own scan, read once
before the integration runs ([[§FS-fetch.6.1](../../functional-spec/FS-fetch.md#61-one-scan-read-before-the-integration-runs-and-installed-against)]): the requested declaration is there once and holds
all of the output, every other declaration in the home reads as before, `check`'s node-local
rules report nothing new, the output closes what it opens, and fetching it again leaves the same
bytes. What `fetch` replaces is what the scan says the declaration owns ([[§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write)],
[[§FS-fetch.5.5](../../functional-spec/FS-fetch.md#55-which-file-declares-and-what-idmd-may-hold)]). Four answers lose.

- **Patch both line loops again.** That is how #1155 and #1159 closed, and the next shape opened
  the next issue.
- **Read the output with the scan and keep `fetch`'s own home reader.** #1176, #1177, #1182,
  #1204 and the `# T-1 x: y` heading are defects in how `fetch` read the home, not the output.
- **Run all of `check` on the candidate.** The relational half would refuse a fetched body for
  citing what the reader has not fetched yet ([[§FS-fetch.3](../../functional-spec/FS-fetch.md#3-accepted-declaration)]).
- **Re-level or trim the output to fit.** Accepted bytes stay verbatim ([[§FS-fetch.3](../../functional-spec/FS-fetch.md#3-accepted-declaration)]); output
  admission would have to rewrite is refused.

## 3. Compatibility

Each row is one input shape, the verdict before and after, and the route the move takes.

| Input shape | Before | After | Route |
|---|---|---|---|
| Output `check` then faults: a bad section path (#1162), a wrong value shape (#1148), a duplicate section | exit 0, `check` red | exit 2, `check`'s line | restores [[§FS-fetch.3](../../functional-spec/FS-fetch.md#3-accepted-declaration)]'s "refused rather than accepted and turned red by the next `grund check`" |
| Output whose only new finding is a warning — a field or section warning, an oversized lead under an opted-in budget | exit 0, `check` exits 0 | exit 2 | 3.2 |
| Output a bare `#` or `##` cuts short (#1163) | exit 0, the text after it belongs to nothing | exit 2 | restores [[§FS-fetch.3](../../functional-spec/FS-fetch.md#3-accepted-declaration)]'s "exactly one Markdown declaration" |
| Output with an open fence or `<pre>` (#1179) | exit 0, later declarations stop being read | exit 2 | restores [[§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write)]'s "all bytes outside the owned declaration remain unchanged" |
| Output that will not settle (#1180) | exit 0, bytes drift per fetch, or refused | exit 2, or written alike twice | restores [[§FS-fetch.6](../../functional-spec/FS-fetch.md#6-stability-and-ownership)]'s idempotence |
| A hand-written declaration inside the replaced range (#1182, #1177) | exit 0, deleted | exit 2, named | 3.1 |
| A heading shallower than the declaration after it (#1176), an indented sibling the scan reads (#1177) | exit 0, deleted or de-indented | exit 0, kept | restores [[§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write)] |
| The ID read under a heading `fetch` could not parse, or deeper than it looked (#1204, `# T-1 x: y`) | a duplicate written, or exit 2 "malformed" | replaced in place, or exit 2 "beside other content" | restores [[§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write)] and [[§FS-fetch.5](../../functional-spec/FS-fetch.md#5-folder-home-write)]'s "when no declaration exists" |
| A `file`-home fetch of an ID declared in another file | exit 0, a duplicate | exit 2, [[§FS-fetch.5.3](../../functional-spec/FS-fetch.md#53-a-declaration-outside-the-home-prevents-installation)]'s line | 3.1 |
| A link inside the config root at the target (#1198) | exit 0, the link replaced by a copy | exit 0, written through, the link kept | restores [[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)] |
| A link outward or dangling at the target (#1198) | exit 0 with the link cut, or exit 2 for a false reason | exit 2, named | restores [[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)] |
| A home edited while the integration runs | exit 0, overwritten | exit 2, changed | restores [[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)] |
| Subsections deeper than one level that `check` accepts, a value chapter (#1148) | exit 2 | exit 0 | 3.3 |
| An untitled or indented declaration heading (`## T-1`, `## T-1:`, `  ## T-1: x`) the scan reads | exit 2 | exit 0 | 3.3 |
| A near-miss sibling heading in the home (`## TICKET-bad: …`) | exit 2 "malformed declaration heading" | exit 0, the sibling kept | 3.3 |
| A refusal naming a path | absolute | from the report base | none: the verdict does not move ([[§FS-errors.4](../../functional-spec/FS-errors.md#4-determinism)]) |

The unread home and the `<ID>.md` with no ID are recorded on their own
([[§DF-fetch-unread-home](DF-fetch-unread-home.md#df-fetch-unread-home-fetch-refuses-a-home-the-scan-does-not-read-before-the-integration-runs)], [[§DF-fetch-refusals-follow-the-scan](DF-fetch-refusals-follow-the-scan.md#df-fetch-refusals-follow-the-scan-four-fetch-refusals-say-what-the-scan-reads-corrected-in-place-and-an-empty-idmd-is-filled)]).

### 3.1. The rows that correct a verdict

Two rows move exit 0 to exit 2 on the route of [[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)], and are not
called restorations, because no sentence of [[§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write)] said them.

- **A hand-written declaration inside the replaced range.** The prior verdict deleted a
  declaration `fetch` does not own, which [[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)] already forbade when it shipped,
  and [[§FS-fetch.3.1](../../functional-spec/FS-fetch.md#31-a-nested-declaration-is-refused)] already said such a declaration stays legal. A warning window
  ([[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)]) would keep deleting it, and the text is not recoverable from
  the tree that follows. No one command can choose between keeping it and moving it out, so
  [[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)] does not fit; the refusal names the file, the line and both
  ways through.
- **A `file`-home fetch of an ID declared in another file.** The argument is
  [[§DF-fetch-declaration-outside-home](DF-fetch-declaration-outside-home.md#df-fetch-declaration-outside-home-a-folder-home-fetch-refuses-an-id-declared-outside-the-home)]'s, unchanged: a second declaration of an ID another file
  holds replaces nothing and inserts no declaration of the home's, so the prior success
  violated [[§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)], and the next `check` was already going to fail on the
  duplicate.

For both rows the prior verdict violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), and that prohibition already applied when the old verdict shipped.
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) does not fit, because a warning window would keep deleting the hand-written declaration and keep writing the duplicate.
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) does not fit, because no one command can choose between keeping a declaration where it is written and moving it out.

### 3.2. Warnings count

"Report nothing new" counts a warning. Output whose only new finding is a warning moves from
exit 0 to exit 2 while `check` on the same tree exits 0. That takes away a fetch that worked, on
the route of [[§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)]: the prior success wrote a tree the project's own
rules faulted, the fault was the fetched text's, and a warning that is never refused at the one
write that causes it is a finding nobody acts on. The refusal is `check`'s own message at the
output's line, and the way through is the upstream text or the rule's strength.

### 3.3. The rows that become a write

Refusals that become writes break no tree. Each was refused by `fetch`'s own heading test or
depth limit, never by anything the scan or `check` says, and each now writes what the next scan
reads as asked.

### 3.4. Who breaks, and who does not

An integration whose output the next `check` faults, warnings included, or which leaves a fence
open, and a project fetching through a link that leaves the project, are refused with exit 2,
each refusal naming the file, the line and the repair. Nothing else that `fetch` writes today and
`check` then reads as asked, with no new finding of either channel, is refused.
This record is not a licence for refusing output the next `check` reads as asked with no new finding, or for running `check`'s relational rules on a fetch.

## release-note: Release note

- In grund 0.23.0, `grund fetch <ID>` builds the file it would write, has the project's own scan
  read it, and writes it only if that reading is exactly what was asked for
  ([[§FS-fetch.admission](../../functional-spec/FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)]). Output the next `grund check` would fault, a warning included, output
  that leaves a fence or `<pre>` open, output a bare `#` or `##` cuts short, output whose next
  fetch would change it again, and a fetch that would delete a declaration written by hand inside
  the one replaced, now exit 2 with every file as it was and the line that says why; each used to
  exit 0. A `file`-home fetch of an ID another file already declares is refused as a folder-home
  one has been since 0.22.0. A link at the target inside the project is written through and kept;
  one that leaves the project or points nowhere is refused. Fetch now replaces any declaration
  the scan reads, at any depth and under any heading the scan reads, and keeps what the scan
  says lies outside it. Value chapters, deeper subsections and untitled headings are written.
  Every refusal names its path from the report base. The verdicts that move to exit 2 change under
  [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids): the prior success violated [§REQ-no-data-loss.2](../../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own), and every finding names its location and
  the action to take ([[§DF-fetch-admission](DF-fetch-admission.md#df-fetch-admission-fetch-installs-only-what-the-next-scan-reads-back-as-exactly-what-was-asked-for)]).

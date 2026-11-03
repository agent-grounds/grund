# DF-yaml-value-window-closes-in-0-22: The YAML value-source warning window closes in 0.22.0

**Date:** 2026-11-03

## status: Accepted

## 1. Context

[§DF-yaml-value-sources.2.3](DF-yaml-value-sources.md#23-one-release-of-warnings) gave a YAML value source one release of warnings on the deprecation
path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): every error located at one was a
`yaml-value-source-deferred` warning, a source with such a finding declared nothing, the other
commands said nothing about it, and a fetched kind's home skipped it. grund 0.21.0 told its users
that this ends in 0.22.0. That notice shipped with 0.21.0, and a notice already released is not
published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that ends the window needs a notice of
its own, and this record carries it.

Nothing else ended it. The check for the window read no version, and the release named in its
messages was only text, so a 0.22.0 cut from `main` kept the window open while saying the
warnings lasted until 0.22.0 ([issue #1133](https://github.com/agent-grounds/grund/issues/1133)).

## 2. Decision

In 0.22.0 (agent-grounds/grund#1133) a YAML value source takes the path a JSON one takes
([§FS-values.2.2.4](../../functional-spec/FS-values.md#224-from-grund-0220-a-yaml-source-is-reported-as-a-json-one-is-after-one-release-of-warnings)):

- a finding at a YAML source carries its own code and severity, in the exit classes of
  [§FS-values.5.3](../../functional-spec/FS-values.md#53-incomplete-input-and-deterministic-output), so a missing index entry is a `missing-index-entry` error at exit `1`;
- an unreadable or unparsable YAML source is an incomplete scan, so `check`, `list`, `show`,
  `refs`, `cover` and `id` exit `2` for it, as they do for a JSON one;
- a readable source declares what it can: an invalid member keeps no valid sibling from being
  declared, and a duplicate holds no source back, where 0.21.x let the JSON value win
  ([§FS-values.2.3](../../functional-spec/FS-values.md#23-duplicates-and-ownership));
- a fetched kind's YAML source is read, so a citation of an ID it declares resolves;
- `grund fetch` into a `file` home that is a value source is refused at exit `2` with the file
  byte-identical, in either format ([§FS-fetch.4](../../functional-spec/FS-fetch.md#4-file-home-write)), and `check` stops naming `grund fetch` for such a home
  ([§FS-check.3.1.3.1](../../functional-spec/FS-check.md#3131-a-home-fetch-refuses)). Now that the source is read, the Markdown a fetch
  appended would be comments to YAML, so the fetch would exit `0`, the fetched value would be
  dropped and `check` would stay green on the old one. JSON takes the same refusal because the two
  formats keep one path: its half refuses earlier a tree the next `check` already refuses at exit
  `2`, and moves `fetch`'s own exit from `0` to `2`;
- `yaml-value-source-deferred` is spent: it stays selectable, as a shipped code does, never fires,
  and `--only` says so ([§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog), [§FS-check.1.4.1](../../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so));
- [§FS-values.2.2.4](../../functional-spec/FS-values.md#224-from-grund-0220-a-yaml-source-is-reported-as-a-json-one-is-after-one-release-of-warnings) keeps its coordinate, because a section name that has shipped is kept
  ([§REQ-spec-section-names.permanence](../../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent)) and [§DF-yaml-value-sources](DF-yaml-value-sources.md#df-yaml-value-sources-a-value-home-reads-yaml-beside-json-and-a-yaml-sources-findings-are-warnings-for-one-release) cites it: it says what is true
  now, beside a past-tense account of what 0.21.x did.

The messages carry no release clause of their own. [§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name) describes a landed error
naming the release that made it one, and the other closings of this release do that, because each
turned one warning into one new refusal. This closing changes where existing findings apply, not
what they are. A `missing-index-entry` is the error it has been since 0.13.0 and says so at a YAML
source as at a JSON one, and an unparsable source prints the JSON line with the format's word
swapped. A clause at the YAML sites alone would make one finding read two ways by format, which is
what [§FS-values.requirements.origin-blind](../../functional-spec/FS-values.md#requirementsorigin-blind-a-root-is-used-the-same-way-whichever-authority-made-it--realized-directional-for-the-next-authority) and this closing are against. The release guard loses
nothing, because the landed messages of this release's other closings already refuse a cut below
0.22.0 ([§FS-distribution.4.2.2](../../functional-spec/FS-distribution.md#422-the-release-guard-reads-the-releases-the-tree-names)), and the news is carried by the notice below, where a person reads
the release.

## 3. Rejected alternatives

- **Re-date the window to a later release.** It is the one other honest remedy, because a named
  release is what the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) buys a repository time
  with, so the code and [§FS-values](../../functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value) would have to move together and on the record. Nothing asks
  for it: 0.21.0 gave the release, the warnings named the fix, and 1.0 is held behind 0.22.0.
- **A switch on the running version.** The window would close by itself at the cut, but the code
  and its goldens would then pin both readings on `main`, and the tree would ship a path no
  release can reach. Main is 0.22.0 work, so the closing lands outright, as this release's other
  closings do.
- **Promote only the findings' severity.** The findings would fail the run while a source with one
  still declared nothing, the other commands stayed silent and a fetched kind's source stayed
  unread, so the source would still be held apart from a JSON one.
- **Refuse a fetch into a YAML home alone, or let the Markdown become YAML comments.** Refusing
  YAML alone would put the two formats back on two paths and leave a JSON home taking an append
  that breaks it. Letting the append stand, with the loss named in the spec, would keep a fetch
  that exits `0` while nothing reads the value it fetched.

## release-note: Release note

- [§DF-yaml-value-window-closes-in-0-22](DF-yaml-value-window-closes-in-0-22.md#df-yaml-value-window-closes-in-0-22-the-yaml-value-source-warning-window-closes-in-0220), [§FS-values.2.2.4](../../functional-spec/FS-values.md#224-from-grund-0220-a-yaml-source-is-reported-as-a-json-one-is-after-one-release-of-warnings), [§FS-values.5.3](../../functional-spec/FS-values.md#53-incomplete-input-and-deterministic-output),
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a YAML value source is now reported as a JSON one is.** What
  grund 0.21.0 announced lands. A finding at a YAML source carries its own code and severity,
  where 0.21.x printed a `yaml-value-source-deferred` warning and exited `0`. A file that is not a
  value table, such as a stray `ci.yml`, is now `invalid-value-declaration` at exit `1`, as is a
  refused construct, a wrong shape or an invalid component; a repeated key or an ID two sources
  declare is `duplicate` at exit `1`; a missing index entry is `missing-index-entry` at exit `1`;
  and a source that cannot be read or parsed is an incomplete scan, so `check`, `list`, `show`,
  `refs`, `cover` and `id` exit `2`. A source with an invalid member now keeps its valid members,
  and in a duplicate across a JSON and a YAML source both are declared, where the JSON value used
  to win. A kind that sets `fetch` now reads the YAML sources in its home, so a citation of one of
  their IDs resolves instead of drawing `missing-snapshot` or `dangling`. A snapshot file a
  `folder` home gained under 0.21.x, for an ID a YAML source in that folder declares, is now a
  `duplicate` at exit `1`, so delete one of the two. `grund fetch` into a `file` home that is a
  value source is now refused at exit `2` and leaves the file byte-identical, in either format,
  where it used to exit `0` and append Markdown. That broke a JSON source at the next `check`. A
  YAML source was not read then; Markdown already appended to one now reads as comments, so
  `check` passes on the source's own value: delete the block and write the value in the source.
  The `yaml-value-source-deferred` code is spent: it never fires, and `--only` naming it says so.
  **Who this affects:** a repository with a YAML file in a value home that drew that warning in
  0.21.0, which should do what the warning named: correct the file, or move it out of the value
  home; and a kind that sets `fetch` on a `file` home that is a value source, in either format,
  which should do what the refusal names: move to a Markdown file or a folder without the source,
  or remove its `fetch`. Closes [issue #1133](https://github.com/agent-grounds/grund/issues/1133).

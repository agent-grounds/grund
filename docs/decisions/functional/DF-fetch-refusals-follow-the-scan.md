# DF-fetch-refusals-follow-the-scan: four fetch refusals say what the scan reads, corrected in place, and an empty `<ID>.md` is filled

**Date:** 2026-11-12

## status: Accepted

## 1. Context

Once `fetch` reads the home through the scan ([[§FS-fetch.admission](../../functional-spec/FS-fetch.md#admission-admission-the-next-scan-reads-back-exactly-what-was-asked-for)]), four refusals it printed
before name what the scan does not read there.

- An `<ID>.md` in a `folder` home that holds prose, a heading that declares nothing or a lone
  citation was refused as `snapshot file <path> already exists for a different ID`, although no
  ID is in it ([#1230](https://github.com/agent-grounds/grund/issues/1230)). An empty one was
  refused the same way, with nothing in it to lose.
- A home whose requested ID the scan reads beside other content, under a heading `fetch`'s own
  loop could not parse, was refused as `snapshot home has malformed declaration heading`,
  although `check` reads that heading.
- A section at the wrong level in the output was refused as `fetch output heading depth <n> is
  invalid for a depth-<d> declaration`, a depth limit `fetch` no longer keeps; the next `check`
  holds a section's level ([[§FS-fetch.3.2](../../functional-spec/FS-fetch.md#32-a-subsection-is-a-heading-the-next-scan-reads-as-a-section)]).
- A dangling link at the target was refused by the walk as `cannot read snapshot folder …`,
  naming a folder that reads fine.

## 2. Decision

Each line says what the scan reads there, in place, with no window.

| Shape | Before | After |
|---|---|---|
| `<ID>.md` with text and no ID | `snapshot file <path> already exists for a different ID` | `snapshot file <path> already exists and the scan reads no ID in it; move or rename it, and fetch again` ([[§FS-fetch.5.5](../../functional-spec/FS-fetch.md#55-which-file-declares-and-what-idmd-may-hold)]) |
| The ID read beside other content | `snapshot home has malformed declaration heading `<heading>`` | `snapshot file <path> contains the requested ID beside other content` ([§FS-fetch.5.5]) |
| A section at the wrong level | `fetch output heading depth <n> is invalid for a depth-<d> declaration` | `fetch output line <n>: <check's message>` ([[§FS-fetch.admission.3](../../functional-spec/FS-fetch.md#admission3-the-structural-rules-check-applies-report-nothing-new)]) |
| A dangling link at the target | `cannot read snapshot folder <folder> at <path>: …` | `cannot fetch <ID> into <path>: the symlink target does not exist; …` ([[§FS-fetch.links](../../functional-spec/FS-fetch.md#links-a-link-at-the-target-is-written-through-when-it-stays-inside-the-config-root)]) |

The first keeps `snapshot file <path> already exists` as its prefix and corrects only the tail.
The exit stays 2 in every row, and the tree stays byte-identical. Two lines keep their words:
`fetch output heading depth <n> is invalid` where the declaration's own heading is at the wrong
level for the home, and `fetch output must contain exactly one declaration and no leading
content`, which `fetch-sibling-declaration-refused` pins ([§FS-fetch.3]).

An `<ID>.md` that holds no bytes, or only whitespace and line endings, is filled with exit 0
([[§FS-fetch.5.5](../../functional-spec/FS-fetch.md#55-which-file-declares-and-what-idmd-may-hold)]). After the write the scan reads the requested declaration there once, which is
the whole admission rule, and it is what [[§FS-fetch.4.1](../../functional-spec/FS-fetch.md#41-an-absent-value-home-is-refused-with-the-way-through)] already tells a value kind to do: create
the empty file, then fetch.

## 3. Compatibility

The four lines change in place under the pre-1.0 licence of [[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise)], as
a named exception of [[§FS-errors.requirements.prefix-windows](../../functional-spec/FS-errors.md#requirementsprefix-windows-a-wording-change-keeps-the-old-message-whole-until-its-window-closes--realized-directional-for-the-next-change)], because a window would keep
naming, on purpose, what is not there: an ID that is not in the file, a heading `check` reads, a
limit `fetch` no longer has, a folder that reads fine. Consumers match `fetch`'s exit 2, which
does not move. The change lands in 0.23.0, the last release before 1.0
([[§DF-scan-failure-code-and-fix](DF-scan-failure-code-and-fix.md#df-scan-failure-code-and-fix-a-scan-failure-carries-its-own-code-and-a-fix-that-works-at-its-path)]).

The empty-file fill moves exit 2 to exit 0 and breaks no tree: the refused file held nothing.

## release-note: Release note

- In grund 0.23.0, four `grund fetch` refusals say what the scan reads, with no window: an
  `<ID>.md` holding text but no ID is refused as `snapshot file <path> already exists and the scan
  reads no ID in it`, no longer `for a different ID`; a home holding the ID beside other content
  says so, no longer `malformed declaration heading`; a section at the wrong level is refused with
  `check`'s own line, no longer `heading depth <n> is invalid`; and a dangling link at the target is
  named as one, no longer `cannot read snapshot folder`. Each still exits 2 with the tree as it
  was. An empty `<ID>.md` is now filled rather than refused ([[§DF-fetch-refusals-follow-the-scan](DF-fetch-refusals-follow-the-scan.md#df-fetch-refusals-follow-the-scan-four-fetch-refusals-say-what-the-scan-reads-corrected-in-place-and-an-empty-idmd-is-filled)]).

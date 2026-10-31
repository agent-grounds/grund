# DF-v2-open-line-closes-in-0-22: The version-2 open-line window closes in 0.22.0

**Date:** 2026-10-31

## status: Accepted

## 1. Context

[§DF-v2-string-closes-on-its-line](DF-v2-string-closes-on-its-line.md#df-v2-string-closes-on-its-line-a-string-in-a-version-2-array-closes-on-its-own-line-warned-for-in-0210-and-refused-from-0220) held a string in a version-2 array to the line it opens on, and
took the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) to get there. grund 0.21.0 still read
an array with an open line that a later line closes, `exclude = [` / `"a` / `b",` / `]` as
`["a b"]`, and warned with code `config-not-toml`, or `config-multiline-string` where the string
opened with `"""`, ending `this warning becomes an error in grund 0.22.0`
([§FS-config-v2.reader.5.strings.window](../../functional-spec/FS-config-v2.md#reader5stringswindow-through-grund-021x-an-array-a-later-line-closes-is-read-as-before-and-warns)). That record's notice ships with 0.21.0, and a notice
already released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that refuses
these arrays needs a notice of its own, and this record carries it.

This is the last of the three version-2 windows 0.21.0 opened. The other two close in the same
release ([§DF-v2-string-closes-early-lands-in-0-22](DF-v2-string-closes-early-lands-in-0-22.md#df-v2-string-closes-early-lands-in-0-22-a-version-2-string-that-closes-early-is-expected-string-from-0220), [§DF-v2-array-comma-window-closes-in-0-22](DF-v2-array-comma-window-closes-in-0-22.md#df-v2-array-comma-window-closes-in-0-22-the-version-2-trailing-comma-window-closes-in-0220)), so
once this one closes nothing fires either code.

## 2. Decision

In 0.22.0 (agent-grounds/grund#1118):

- no line after an open line is joined to the array: the open line is its last, and the array is
  refused at its key's line as `unclosed array` with the parenthetical naming the open line
  ([§FS-config-v2.reader.5.strings.refusal](../../functional-spec/FS-config-v2.md#reader5stringsrefusal-from-grund-0220-the-open-line-ends-the-array)), so `check` and `config show` exit `2` and
  `config validate` exits `1`;
- where the readers before 0.22.0 would have closed the array on a later line, so that its text
  read through the window, the refusal ends `; this became an error in grund 0.22.0`; an array
  they did not close either, such as `["a,b]` on the last line of the file, keeps the bare
  refusal;
- a `"""` string open at the end of its line is refused with the parenthetical
  `(a multi-line string opens on line <n>, which version 2 does not read)` and the same suffix
  rule ([§FS-config-v2.reader.5.strings.multi-line](../../functional-spec/FS-config-v2.md#reader5stringsmulti-line-a-string-opened-with--is-one-version-2-does-not-read));
- an array with an open line is refused for it before any element is read, so a string in it that
  closes early, or a comma with no element before it, earns no refusal of its own
  ([§FS-config-v2.reader.6.refusal](../../functional-spec/FS-config-v2.md#reader6refusal-from-grund-0220-a-string-that-closes-early-is-expected-string), [§FS-config-v2.reader.5.commas.refusal](../../functional-spec/FS-config-v2.md#reader5commasrefusal-from-grund-0220-a-comma-with-no-element-before-it-is-expected-string));
- three quotes in a row that open a string open a multi-line string, read as TOML reads one, so
  a line that ends `""""a` is open, and the refusal names it, where 0.21.x read the quotes one at
  a time and named a later line ([§FS-config-v2.reader.5.strings](../../functional-spec/FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)). The verdict does not move, and
  neither does the suffix, which the reading of the readers before decides;
- the warning goes, and `config-not-toml` and `config-multiline-string`, with nothing left to
  fire them, are spent from 0.22.0: each stays in the catalog, so `--only` and `--ignore` still
  accept it, and `--only` says the check is spent ([§FS-config-v2.checks.config-not-toml](../../functional-spec/FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses),
  [§FS-config-v2.checks.config-multiline-string](../../functional-spec/FS-config-v2.md#checksconfig-multiline-string-a-version-2-config-value-holds-a-multi-line-string), [§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog));
- the ancestor climb reads such a `[workspace] members` value as not a list, as it reads any
  value the full load refuses ([§FS-workspace.6.1.7.5](../../functional-spec/FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable));
- version 1, which reads no array across lines, does not move;
- the window's point, [§FS-config-v2.reader.5.strings.window](../../functional-spec/FS-config-v2.md#reader5stringswindow-through-grund-021x-an-array-a-later-line-closes-is-read-as-before-and-warns), stays as a past-tense account of
  what 0.21.x did, because a section name that has shipped is kept
  ([§REQ-spec-section-names.permanence](../../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent)) and the accepted record above cites it.

## 3. Rejected alternatives

- **Keep the window for another release.** 0.21.0 printed 0.22.0 as the release these arrays
  are refused in, and the release guard refuses a version that reaches a promise it does not
  keep ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)).
- **Keep naming the line 0.21.x named for a `"""` run.** The line slot is there so the author
  edits the right line, and one that names a later line than the one the string opens on is a
  defect in the message, not phrasing a script can rely on.
- **Leave the two codes firing-capable but silent.** A code that nothing can fire and that
  still reads as an always-on warning tells `--only` nothing; the catalog's spent severity is how
  a code says it never fires ([§FS-errors.5.5.1](../../functional-spec/FS-errors.md#551-how-a-row-is-read)), as it says for `image-index-entry`.

## release-note: Release note

- [§DF-v2-open-line-closes-in-0-22](DF-v2-open-line-closes-in-0-22.md#df-v2-open-line-closes-in-0-22-the-version-2-open-line-window-closes-in-0220), [§FS-config-v2.reader.5.strings.refusal](../../functional-spec/FS-config-v2.md#reader5stringsrefusal-from-grund-0220-the-open-line-ends-the-array),
  [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path):
  **a version-2 array whose string runs over a line break is now refused.** What grund 0.21.0
  announced lands. `exclude = [` / `"a` / `b",` / `]`, which 0.21.0 read as `["a b"]` and warned
  for, is refused as `error: grund.toml:9: unclosed array (a string is still open at the end of
  line 10); this became an error in grund 0.22.0`, and a string opened with `"""` there as
  `unclosed array (a multi-line string opens on line 10, which version 2 does not read)` with
  the same suffix; `check` and `config show` exit `2`, `config validate` exits `1`. An array that
  never closed keeps its bare `unclosed array`, and where a string opens with four or more quotes
  in a row the refusal now names the line it opens on. `config-not-toml` and
  `config-multiline-string` no longer fire at all: `--only` and `--ignore` still accept them,
  and `--only` says the check is spent. Version-1 configs load exactly as before. **Who this
  affects:** a repository whose version-2 `grund.toml` drew the `close the string on line …` or
  `write the string on line … as one basic string` warning in 0.21.0, which should close each
  string on the line it opens on, and a script that selects either code. Closes
  [issue #1118](https://github.com/agent-grounds/grund/issues/1118).

# DF-v2-string-closes-on-its-line: a string in a version-2 array closes on its own line, warned for in 0.21.0 and refused from 0.22.0

**Date:** 2026-10-29

## status: Accepted

## 1. Context

A version-2 `grund.toml` may write an array across lines ([§FS-config-v2.reader.5](../../functional-spec/FS-config-v2.md#reader5-an-array-may-span-lines)). The reader joined the lines for as long as a `[` stayed open, and it counted brackets with an "inside a string" flag that it carried from one line into the next. So a string left open at the end of a line ran on into the next, and the break between them was read as one space:

```toml
[schema.sources]
exclude = [
  "a
b",
]
```

loaded as `exclude = ["a b"]`, and `"a,` / `b",` as `["a, b"]`, with `check` green. TOML does not allow a line break inside a basic string, so every other TOML tool refuses the file, and the value grund read is one nobody wrote: `a b` matches no directory, so whatever the author meant to exclude was still scanned. The near-miss is a quote dropped at the end of one element of a long array ([issue #1089](https://github.com/agent-grounds/grund/issues/1089)).

The reading was not an accident of code that disagreed with its spec. [§FS-config-v2.reader.5](../../functional-spec/FS-config-v2.md#reader5-an-array-may-span-lines) said an array's lines belong to it "until its `[` closes outside a string", which is exactly the carried flag, so changing it changes what the point means. Every release with the version-2 reader, 0.18.0 through 0.20.x, loads these files, and [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) allows a release to turn a file that loads into one that is refused only by one of its routes.

## 2. Decision

### 2.1 A string closes on the line it opens on

A string in an array closes on its own line, as a TOML basic string does, and a line break is never part of a value ([§FS-config-v2.reader.5.strings](../../functional-spec/FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on)). The first line of an array that ends inside a string is its open line, and every message names it, because that is the line the author has to edit; the array is still reported at its key's line, where every config value is located ([§FS-config-v2.reader.4](../../functional-spec/FS-config-v2.md#reader4-order-and-location-are-kept)).

### 2.2 The route is the deprecation window

The route is the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): grund 0.21.0 reads these files as before and warns at the key's line, naming the open line and grund 0.22.0, and grund 0.22.0 refuses them ([§FS-config-v2.reader.5.strings.window](../../functional-spec/FS-config-v2.md#reader5stringswindow-through-grund-021x-an-array-a-later-line-closes-is-read-as-before-and-warns), [§FS-config-v2.reader.5.strings.refusal](../../functional-spec/FS-config-v2.md#reader5stringsrefusal-from-grund-0220-the-open-line-ends-the-array)).

The correction route, section 5 of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly), would refuse them in this release, and it needs a hard requirement, cited by number, that the old reading already broke. None does. [§REQ-never-crashes](../../requirements/REQ-never-crashes.md#req-never-crashes-garbage-in-diagnostic-out) is about crashes and the exit-code mapping, and the reader did not fail: it read a value, and `0` was the right exit for what it read. [§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) held, because the config reader and the ancestor climb both read `a b`. The nearest rule, that a mistake in the config fails loudly ([§FS-config.requirements.5](../../functional-spec/FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly--realized-one-case-deferred)), is a specification point, not a hard requirement, and its text is about unknown keys and invalid values falling back to a default. Reading any of them as a ban on this case would invent the ban. The issue's author ruled the same on the plan: an FS point does not carry section 5.

The mechanical migration of section 3 does not apply either: no command can make the fix, because only the author knows whether `"a` / `b",` meant `"a b"`, `"a", "b"`, or something else. That is the same reason [§DF-absorbed-scan-warning](DF-absorbed-scan-warning.md#df-absorbed-scan-warning-a-scan-its-own-members-swallowed-is-a-warning-with-a-named-release-not-an-error) gave for taking the window on another config mistake no command can repair.

### 2.3 The refusal keeps its message and gains a parenthetical

The finding is the existing `unclosed array`, with ` (a string is still open at the end of line <n>)` appended where a string caused it. [§FS-config.3.11](../../functional-spec/FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string) already said that version 2 refuses an unclosed string "as an array that does not close", and `["a,b]` already printed `unclosed array`; a second message, such as `unclosed string`, would split one cause in two depending on whether some later quote happened to balance it. Appending keeps `unclosed array` a verbatim prefix ([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text)), so a script that matches it still matches.

From 0.22.0 the refusal ends `; this became an error in grund 0.22.0` only where the readers before it would have closed the array, so that the file loaded through the window ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). An array that never closed, such as `["a,b]` on the last line of the file, was refused before, and its refusal claims no release.

### 2.4 Two codes, because a `"""` string is TOML

The warning's code is `config-not-toml` ([§FS-config-v2.checks.config-not-toml](../../functional-spec/FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses)). It names the class rather than this one construct: text TOML refuses that a version-2 reader loads. #1087 (`["ab",,]`) and #1088 (`key = "one" "two"`) are constructs of the same class, and each can join the code under its own record in the release it lands in.

A string opened with `"""` is the exception. TOML admits a multi-line basic string, so `"""a` / `b"""` is valid TOML, and neither "not TOML" nor "close the string on line 8" is true of it. What is true is that version 2 does not read multi-line strings: through 0.20.x it read that value as `""a b""`, quotes included. So the reader tells the two apart by how the open string opened, and the `"""` case says so, under its own code, `config-multiline-string` ([§FS-config-v2.reader.5.strings.multi-line](../../functional-spec/FS-config-v2.md#reader5stringsmulti-line-a-string-opened-with--is-one-version-2-does-not-read), [§FS-config-v2.checks.config-multiline-string](../../functional-spec/FS-config-v2.md#checksconfig-multiline-string-a-version-2-config-value-holds-a-multi-line-string)), with the remedy of writing the string as one basic string. The window and the refusal are the same for both. Reading multi-line strings is not part of this decision.

### 2.5 `config show` escapes a line break on the version-2 path

The writer escaped only `\` and `"`, so `config show` wrote an element written `"a\nb"` back across a line break, as `"a` / `b"`. That output already reloaded as `a b`, which broke [§FS-config.4.2](../../functional-spec/FS-config.md#42-grund-config-show-path)'s promise that what `config show` prints loads back to the same values, and from 0.22.0 grund would refuse its own output. The version-2 writer, `config show` on a version-2 config, now writes a line feed, a tab and a carriage return as `\n`, `\t` and `\r`, the escapes the reader already reads. The shared escaper does not move: the version-1 `config show`, whose bytes [§FS-config.4.2](../../functional-spec/FS-config.md#42-grund-config-show-path) keeps, the `grund init` scaffold and the invalid-rule remedy print what they printed before. Version 1 reads no array across lines ([§FS-config.3](../../functional-spec/FS-config.md#3-keys-version-1)), so its output cannot fall into this window.

### 2.6 The ancestor climb moves with the reader

The climb that reads an ancestor's `[workspace] members` reads a version-2 array through the reader's own join ([§FS-workspace.6.1.7.5](../../functional-spec/FS-workspace.md#6175-an-unobtainable-members-value-leaves-the-claim-undecidable)), so it moves with it and the two cannot disagree: through 0.21.x it reads what the config loads, and from 0.22.0 a `members` value with an open line is not a list, exactly where the reader refuses it.

## 3. Alternatives considered

**Refuse in this release.** Rejected on [§DF-v2-string-closes-on-its-line.2.2](DF-v2-string-closes-on-its-line.md#22-the-route-is-the-deprecation-window): it is the correction route with no hard requirement behind it, and it would fail every command on a repository that loads today, with no release to fix it in.

**A new `unclosed string` refusal.** Rejected on [§DF-v2-string-closes-on-its-line.2.3](DF-v2-string-closes-on-its-line.md#23-the-refusal-keeps-its-message-and-gains-a-parenthetical): one cause would print one of two messages depending on a later quote, and the existing message would be reworded rather than appended to.

**One code for both cases.** Rejected on [§DF-v2-string-closes-on-its-line.2.4](DF-v2-string-closes-on-its-line.md#24-two-codes-because-a--string-is-toml): no name is true of both a file that is not TOML and one that is, unless it names only the line break, and then the sibling constructs of #1087 and #1088, which hold no line break, could not join it.

**Escaping in the shared escaper.** Rejected on [§DF-v2-string-closes-on-its-line.2.5](DF-v2-string-closes-on-its-line.md#25-config-show-escapes-a-line-break-on-the-version-2-path): it would move the bytes of the version-1 `config show`, the init scaffold and a rule's remedy, none of which this window concerns.

## 4. Consequences

A repository with such an array keeps its exit code and its values in 0.21.0, and `check` prints the warning where it printed `success`; JSON output carries one more warning. From 0.22.0 every command that loads the config exits `2` until the string is closed on its line, and that includes a `"""` string written across lines. A script that matches the whole line `unclosed array` sees the parenthetical; a prefix match still matches. `config show` on a version-2 config prints a control character inside a value as its escape rather than as the raw byte. The issue that closes the window in 0.22.0 is opened when this decision ships.

## release-note: Release note

- [§FS-config-v2.reader.5.strings](../../functional-spec/FS-config-v2.md#reader5strings-a-string-closes-on-the-line-it-opens-on), [§FS-config-v2.checks.config-not-toml](../../functional-spec/FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses), [§FS-config-v2.checks.config-multiline-string](../../functional-spec/FS-config-v2.md#checksconfig-multiline-string-a-version-2-config-value-holds-a-multi-line-string), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a string in a version-2 array that runs over a line break now warns, and becomes an error in grund 0.22.0.** `exclude = [` / `"a` / `b",` / `]` loaded silently as `["a b"]`; it still loads, and `check`, `config validate` and `config show` print `warning: grund.toml:7: unclosed array (a string is still open at the end of line 8), read for now with the line break as a space; close the string on line 8; this warning becomes an error in grund 0.22.0`, code `config-not-toml`. A string opened with `"""` earns `config-multiline-string` instead, saying version 2 does not read multi-line strings. An array that never closes keeps its `unclosed array` refusal and now names the open line, as in `unclosed array (a string is still open at the end of line 7)` for `exclude = ["a,b]`. `config show` on a version-2 config writes a line break, a tab and a carriage return inside a value as `\n`, `\t` and `\r`, so its output loads back to the same values. **Who this affects:** a repository whose version-2 `grund.toml` lets a string run over a line break, which gets a warning now and a load error in 0.22.0, and a script that matches the whole `unclosed array` line. Closes [issue #1089](https://github.com/agent-grounds/grund/issues/1089).

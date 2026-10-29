# DF-v2-string-closes-at-its-quote: a version-2 string ends at its first unescaped quote, warned for in 0.21.0 and refused from 0.22.0, and version 1 keeps its reading

**Date:** 2026-10-29

## status: Accepted

## 1. Context

The readers before grund 0.21.0 checked a string only at the first and the last byte of its value or its element, and copied every `"` between them into the string. So a version-2 config such as

```toml
[schema.kinds.SPEC]
fetch = "one" "two"
folders = ["docs" "specs"]
```

loaded, as the one command `one" "two` and the one folder `docs" "specs`, and so did a quoted key, `"md" "x" = "markdown"`, as the suffix `md" "x`. TOML refuses each of these lines, so every other tool that reads the file as TOML refuses the file grund calls valid. And the second line is a comma forgotten: grund looks for a folder that does not exist, reads neither `docs/` nor `specs/`, and nothing points back at the line ([issue #1088](https://github.com/agent-grounds/grund/issues/1088)). A green `check` over a tree it never read is the outcome a mistake in the config failing loudly exists to prevent ([§FS-config.requirements.5](../../functional-spec/FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly--realized-one-case-deferred)).

The reading was not an accident of code that disagreed with its spec. [§FS-config.3.11](../../functional-spec/FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string) promised that no list read before it is read differently, and named the reading it kept: a piece between two commas that is not one basic string is cut at each comma and each part read as before. Every release with the version-2 reader, 0.18.0 through 0.20.x, loads these files, and [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly) allows a release to turn a file that loads into one that is refused only by one of its routes.

## 2. Decision

### 2.1 A version-2 string ends at its first unescaped quote

In version 2 a basic string, a scalar value, an element or a quoted key, closes at the first `"` that is not part of a `\"` escape, and only whitespace, a comment, a separating `,` or the closing `]` may follow it ([§FS-config-v2.reader.6](../../functional-spec/FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote)). That is TOML's rule and the one [§FS-config.3.11](../../functional-spec/FS-config.md#311-a-list-element-ends-at-a-comma-outside-a-string) already reads strings by when it cuts a list at its commas; only the check of a value was lax.

### 2.2 The route is the deprecation window

The route is the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): grund 0.21.0 reads these files as before and warns at the key's line, naming the string as it is read, the line to edit and grund 0.22.0, and grund 0.22.0 refuses them as `expected string` ([§FS-config-v2.reader.6.window](../../functional-spec/FS-config-v2.md#reader6window-through-grund-021x-a-string-that-closes-early-is-read-as-before-and-warns), [§FS-config-v2.reader.6.refusal](../../functional-spec/FS-config-v2.md#reader6refusal-from-grund-0220-a-string-that-closes-early-is-expected-string)).

The correction route, section 5 of [§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly), would refuse them in this release, and it needs a hard requirement, cited by number, that the old reading already broke. None does. The nearest rule, that a mistake in the config fails loudly, is a specification point and not a hard requirement, which is the finding [§DF-v2-string-closes-on-its-line.2.2](DF-v2-string-closes-on-its-line.md#22-the-route-is-the-deprecation-window) made for the string left open at a line's end. The mechanical migration of section 3 does not apply either: no command can make the repair, because only the author knows whether `"docs" "specs"` meant two folders or one folder whose name holds two quotes.

### 2.3 One code, with a remedy for where the string stands

The warning's code is `config-not-toml` ([§FS-config-v2.checks.config-not-toml](../../functional-spec/FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses)), which was named for the class, text TOML refuses that a version-2 reader still loads, so that this construct could join it ([§DF-v2-string-closes-on-its-line.2.4](DF-v2-string-closes-on-its-line.md#24-two-codes-because-a--string-is-toml)). It joins as a second construct with remedies of its own. An element's remedy has two halves, write `\"` for a quote inside the string or a `,` between two strings, because either may be what was meant. A scalar's or a key's has the first alone: followed there, the second turns `fetch = "one" "two"` into `fetch = "one", "two"`, which closes early again.

The line the remedy names is the line the string closes early on, which in an array written across lines is not the key's line, so that the line named is the line the author edits; the warning is still located at the key's line, as every value is. One value earns one warning, naming the first string in it that closes early, as an array earns one for its open lines. An array that also has a line ending inside a string earns both warnings, because they are two constructs, and each report orders them as it orders every finding: the one for the open line comes first in `config validate` and `config show`, and the early close first in `check` ([§FS-config-v2.reader.6.window](../../functional-spec/FS-config-v2.md#reader6window-through-grund-021x-a-string-that-closes-early-is-read-as-before-and-warns)).

### 2.4 A `"""` string on one line is a multi-line string

`title = """one"""` is valid TOML that version 2 does not read: it loaded as `""one""`. So it is not `config-not-toml` but `config-multiline-string`, the code a `"""` string written across lines already earns ([§FS-config-v2.reader.6.multi-line](../../functional-spec/FS-config-v2.md#reader6multi-line-a--string-on-one-line-is-one-version-2-does-not-read)), with the same window and the same remedy, to write the string as one basic string. A `"""` string written across lines is one construct, already warned for, and earns no second warning once its lines are joined.

### 2.5 Version 1 keeps its reading for good

A version-1 file keeps its meaning and its validation in every release, and the stricter rules of version 2 never refuse a file version 1 accepts ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)). So version 1 reads these strings as it always has, with no refusal and no warning of its own: the `config-version-1` caution already names the command that moves it to version 2. The rule meets a version-1 file at that command instead. `config migrate --to 2` refuses a string that closes early at its key's line and gives the version-1 repair ([§FS-config-migrate.3.1.8](../../functional-spec/FS-config-migrate.md#318-a-string-that-closes-early)), because writing the value out escaped would carry the typo into version 2 as valid TOML.

### 2.6 The quick readers record nothing

The readers that read one key of a config without loading it, the name of a config no run loads and the ancestor climb's `members`, have no run to report into. Through 0.21.x they read what the full load reads, and from 0.22.0 they pass over a value the full load refuses, as they pass over any value they cannot read ([§FS-config-v2.reader.6.refusal](../../functional-spec/FS-config-v2.md#reader6refusal-from-grund-0220-a-string-that-closes-early-is-expected-string)).

## 3. Alternatives considered

**Refuse in this release.** Rejected on [§DF-v2-string-closes-at-its-quote.2.2](DF-v2-string-closes-at-its-quote.md#22-the-route-is-the-deprecation-window): it is the correction route with no hard requirement behind it, and it would fail every command on a repository that loads today, with no release to fix it in.

**Refuse in version 1 too, through a window of its own.** Rejected on [§DF-v2-string-closes-at-its-quote.2.5](DF-v2-string-closes-at-its-quote.md#25-version-1-keeps-its-reading-for-good): it would break a version-1 file that version 1 accepts, which [§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning) rules out for every older version.

**A code of its own.** Rejected on [§DF-v2-string-closes-at-its-quote.2.3](DF-v2-string-closes-at-its-quote.md#23-one-code-with-a-remedy-for-where-the-string-stands): the class it would name is `config-not-toml`'s, and a second code for it would let `--ignore config-not-toml` keep one TOML refusal and hide another.

**One remedy for every string.** Rejected on [§DF-v2-string-closes-at-its-quote.2.3](DF-v2-string-closes-at-its-quote.md#23-one-code-with-a-remedy-for-where-the-string-stands): the `,` half is a repair only where two strings can stand, and followed on a scalar it earns the same warning again.

## 4. Consequences

A repository whose version-2 `grund.toml` holds such a string keeps its exit code and its values in 0.21.0, and `check`, `config validate` and `config show` print the warning where `check` printed `success`; JSON output carries one more warning. From 0.22.0 every command that loads the config exits `2` until the string ends at its closing quote. A version-1 config never breaks, and meets the rule only when its author runs `config migrate`. `config explain config-not-toml` gives the remedy for each construct. The issue that closes the window in 0.22.0 is opened when this decision ships.

## release-note: Release note

- [§FS-config-v2.reader.6](../../functional-spec/FS-config-v2.md#reader6-a-string-ends-at-its-first-unescaped-quote), [§FS-config-v2.checks.config-not-toml](../../functional-spec/FS-config-v2.md#checksconfig-not-toml-a-version-2-config-holds-text-toml-refuses), [§FS-config-v2.checks.config-multiline-string](../../functional-spec/FS-config-v2.md#checksconfig-multiline-string-a-version-2-config-value-holds-a-multi-line-string), [§FS-config-migrate.3.1.8](../../functional-spec/FS-config-migrate.md#318-a-string-that-closes-early), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a version-2 string followed by more text after its closing quote now warns, and becomes an error in grund 0.22.0.** `fetch = "one" "two"` loaded silently as the one string `one" "two`, and `folders = ["docs" "specs"]`, a comma forgotten, as one folder named `docs" "specs`; they still load, and `check`, `config validate` and `config show` print `warning: grund.toml:5: expected string, read for now as the one string `docs" "specs`; end the string on line 5 at its closing quote: write `\"` for a quote inside it, or a `,` between two strings; this warning becomes an error in grund 0.22.0`, code `config-not-toml`; for a scalar or a quoted key the remedy stops at `write `\"` for a quote inside it`. A string opened with `"""` on one line, `title = """one"""`, earns `config-multiline-string` instead. Version-1 configs load exactly as before, in every release; `config migrate --to 2` now refuses a version-1 string that closes early, naming its line and the version-1 repair, rather than writing it out as valid TOML. **Who this affects:** a repository whose version-2 `grund.toml` writes two strings side by side or a quote unescaped inside one, which gets a warning now and a load error in 0.22.0, and a repository migrating such a version-1 file. Closes [issue #1088](https://github.com/agent-grounds/grund/issues/1088).

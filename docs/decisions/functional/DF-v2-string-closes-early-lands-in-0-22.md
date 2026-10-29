# DF-v2-string-closes-early-lands-in-0-22: a version-2 string that closes early is `expected string` from 0.22.0

**Date:** 2026-10-29

## status: Accepted

## 1. Context

[§DF-v2-string-closes-at-its-quote](DF-v2-string-closes-at-its-quote.md#df-v2-string-closes-at-its-quote-a-version-2-string-ends-at-its-first-unescaped-quote-warned-for-in-0210-and-refused-from-0220-and-version-1-keeps-its-reading) made a version-2 string end at its first unescaped quote, and took the deprecation path of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) to get there, because the move is from load to refusal: grund 0.21.0 read a string that closes early as the readers before it read it and warned `config-not-toml` at its key's line, or `config-multiline-string` for a `"""` string on one line, naming grund 0.22.0 ([§FS-config-v2.reader.6.window](../../functional-spec/FS-config-v2.md#reader6window-through-grund-021x-a-string-that-closes-early-is-read-as-before-and-warns)). That record's notice shipped with 0.21.0, and a notice already released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that closes the window needs a notice of its own, and this record carries it.

## 2. Decision

In grund 0.22.0 (agent-grounds/grund#1094), version 2 refuses what 0.21.x warned for ([§FS-config-v2.reader.6.refusal](../../functional-spec/FS-config-v2.md#reader6refusal-from-grund-0220-a-string-that-closes-early-is-expected-string)):

- a scalar, an element or a quoted key whose string closes early is refused at its key's line as `expected string; this became an error in grund 0.22.0`, exit `2` on every command that loads the config and `1` on `config validate` ([§FS-config.4.3](../../functional-spec/FS-config.md#43-invalid-config-behavior));
- a `"""` string on one line is refused the same way, with the parenthetical `(a multi-line string, which version 2 does not read)` before the suffix ([§FS-config-v2.reader.6.multi-line](../../functional-spec/FS-config-v2.md#reader6multi-line-a--string-on-one-line-is-one-version-2-does-not-read));
- a value the readers before refused keeps the bare `expected string`, so `exclude = ["target" "a,b"]` reads as it did;
- the quick readers pass over such a value, as they pass over any value they cannot read;
- `config-not-toml` and `config-multiline-string` no longer fire for it, and stay for the constructs whose windows are still open ([§FS-config-v2.checks.config-not-toml.1](../../functional-spec/FS-config-v2.md#checksconfig-not-toml1-a-warning-in-021x-the-unclosed-array-or-expected-string-refusal-from-0220)).

Version 1 is unchanged: it keeps its reading for good ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)), and `config migrate --to 2` still refuses such a string ([§FS-config-migrate.3.1.8](../../functional-spec/FS-config-migrate.md#318-a-string-that-closes-early)).

## 3. Rejected alternatives

- **Keep the reading for another release.** 0.21.0's warning named 0.22.0, and the release guard refuses a version that reaches a promise it does not keep ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). Moving the date would make the warning 0.21.0 printed false.
- **Refuse every such value with the bare message.** A value that loaded in 0.21.x and fails in 0.22.0 is an upgrade's doing, and the suffix says which release did it; a value refused before is not, and a suffix on it would blame a release that changed nothing about it.

## release-note: Release note

- [§DF-v2-string-closes-early-lands-in-0-22](DF-v2-string-closes-early-lands-in-0-22.md#df-v2-string-closes-early-lands-in-0-22-a-version-2-string-that-closes-early-is-expected-string-from-0220), [§FS-config-v2.reader.6.refusal](../../functional-spec/FS-config-v2.md#reader6refusal-from-grund-0220-a-string-that-closes-early-is-expected-string), [§FS-config-v2.reader.6.multi-line](../../functional-spec/FS-config-v2.md#reader6multi-line-a--string-on-one-line-is-one-version-2-does-not-read), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **a version-2 string followed by more text after its closing quote is now an error.** What grund 0.21.0 announced with its `config-not-toml` warning lands: `fetch = "one" "two"` and `folders = ["docs" "specs"]`, a comma forgotten, no longer load; every command that loads the config prints `error: grund.toml:5: expected string; this became an error in grund 0.22.0` and exits `2`, and `config validate` exits `1`. A `"""` string on one line, `title = """one"""`, is refused the same way with `(a multi-line string, which version 2 does not read)` before the suffix. Version-1 configs load exactly as before. **Who this affects:** a version-2 `grund.toml` that drew that warning, or the `config-multiline-string` warning for a string on one line, in 0.21.x: end the string at its closing quote, writing `\"` for a quote inside it or a `,` between two strings. Closes [issue #1094](https://github.com/agent-grounds/grund/issues/1094).

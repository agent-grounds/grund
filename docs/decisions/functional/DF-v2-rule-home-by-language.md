# DF-v2-rule-home-by-language: a version-2 rule kind's `file` home is Markdown by the language `[schema.sources]` reads it as

**Date:** 2026-11-04

## status: Accepted

## 1. Context

`rules = true`, and its version-2 spelling `form = "rule"`, requires a home
"whose declarations are Markdown" ([§FS-config.3.4.12](../../functional-spec/FS-config.md#3412-rules--rule-declaration-kinds)).
The load-time check answered that from the file's name alone: a `file` home
was Markdown when its extension was `md`. That is version 1's own reading rule,
so in version 1 the two agree. Version 2 reads a file by the language
`[schema.sources]` maps its extension to
([§FS-config-v2.schema.sources.extensions](../../functional-spec/FS-config-v2.md#schemasourcesextensions-schemasourcesextensions-maps-a-suffix-to-a-language)),
and the version-2 loader reused the name test unchanged, so the check and the
reader asked different questions. Over

```toml
[schema.sources]
languages = ["markdown", "json"]

[schema.sources.extensions]
md = "json"

[schema.kinds.RULE]
files = ["rules/rules.md"]
form = "rule"
```

the row loaded, the rules file was never read, and `grund check` exited `0`
with one warning, while the same row spelled `rules/rules.json` was refused at
`form`. The converse was refused for the opposite reason: under
`txt = "markdown"` a `rules/rules.txt` home, which the scan reads as Markdown,
failed the name test. Reported as
[issue #1149](https://github.com/agent-grounds/grund/issues/1149).

## 2. Decision

### 2.1 The check asks the reader's question

In version 2 a `file` home is Markdown exactly when the language its extension
maps to is of class `document`, the class the Markdown reader reads
([§FS-config-v2.schema.sources.rules](../../functional-spec/FS-config-v2.md#schemasourcesrules-a-rule-kinds-file-home-is-markdown-by-its-language)).
That is one answer with the scan, from the one place version 2 resolves a path
to its language, not a second copy of it. A test for the built-in `markdown`
alone would not be: the scan reads a `document` definition the same way, and
`md = "v1_markdown"`, what `grund config migrate --to 2` writes, must stay a
rule kind. Every entry of `files` is asked, because each is read on its own: a
row is refused when any entry is not Markdown, not only its first. A home no
admitted language reads is not Markdown either: nothing reads its declarations.
The refusal keeps its
message and its location, the `form` key. Version 1 keeps the name test, which
is already its reader's answer.

### 2.2 A `folder` home is left as it is

A `folder` home is accepted on its presence, in both versions. Which of its
files are read is a question for the scan, which already warns about a home it
reads nothing from; deciding at load that a folder holds no Markdown would be a
new rule this report does not ask for.

## 3. Consequences

- **Three verdicts move from `0` to `2`.** A version-2 rule kind whose `file`
  home is read as another language, or read by no admitted language, is now
  refused at load where it loaded and passed. So is one with several `files`
  entries of which a later one is either of those, such as
  `files = ["rules/rules.md", "rules/other.json"]`. The rules those homes named
  were never checked, so no rule that was in force stops applying.
- **One verdict moves from `2` to `0` or `1`.** A version-2 rule kind whose
  `file` home has another name but is read by a `document` language, the
  built-in `markdown` under `txt = "markdown"` or a `document` definition, now
  loads, and its rules apply, so the run reports what they find.

  **Which clause licenses the move.** [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)
  does, and each of its conditions is met:

  1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips),
     which already applied when it shipped: a run whose configured rule catalog
     is never read is not complete and "never passes as if it were". It also
     broke [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded):
     no section named a rule kind whose home the reader skips, and the warning
     that was printed gave a wrong reason, that no listed language reads `.md`.
  2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit:
     there is no old form beside a new one to warn about first, only a
     configuration that already names the same intent the `.json` spelling is
     refused for. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration does not fit
     either, because it has nothing mechanical to write: whether to drop the
     mapping or rename the home is the author's call.
  3. *Named release.* The release note below names the move.
  4. *Actionable findings.* The replacing finding is the existing refusal,
     located at the row's `form` key, which is where the edit goes.
  5. *No new licence.* Nothing is newly prohibited: the prerequisite is the one
     [§FS-config.3.4.12](../../functional-spec/FS-config.md#3412-rules--rule-declaration-kinds)
     already stated, now asked of the file the way the reader asks it. This
     correction is not a licence for any other tightening of a rule kind's row.

  **The weak point.** The old run did print a warning, so it was not wholly
  silent. It still exited `0` over rules it never read, which is the pass the
  requirement forbids. Were the route refused, the fallback is the deprecation
  path: for one release the refusal is a warning beside the existing one, and a
  note names the release in which it becomes the error.

## release-note: Release note

- [§FS-config-v2.schema.sources.rules](../../functional-spec/FS-config-v2.md#schemasourcesrules-a-rule-kinds-file-home-is-markdown-by-its-language), [§DF-v2-rule-home-by-language](DF-v2-rule-home-by-language.md#df-v2-rule-home-by-language-a-version-2-rule-kinds-file-home-is-markdown-by-the-language-schemasources-reads-it-as), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): **a version-2 rule kind's `file` home is Markdown by the language `[schema.sources]` reads it as, not by its name.** **Who this breaks:** a version-2 project whose `form = "rule"` kind names a `.md` home that `[schema.sources]` maps to another language (`md = "json"`) or that no admitted language reads (`languages = ["json"]`), or whose `files` lists such a home after a Markdown one (`files = ["rules/rules.md", "rules/other.json"]`), loaded and exited `0` with those rules unread; it is now refused at the row's `form` key with ``kind `RULE` sets `rules = true` but rule kinds must be citable, scanned Markdown kinds with a `file` or `folder` home and without `values` or `fetch` ``, exit `2`, the refusal the same row already drew under a `.json` name. The verdict moves from `0` to `2`, and every finding that replaces it names its location, the row's `form` key, and the fix. Fix it by mapping `.md` back to a `document` language or by giving the rules file a name a `document` language reads. Conversely, a home with another name that `[schema.sources]` reads with a `document` language (`txt = "markdown"`, or a `document` definition) was refused and now loads, so its rules apply. Version 1 and `folder` homes are unchanged. Closes [issue #1149](https://github.com/agent-grounds/grund/issues/1149).

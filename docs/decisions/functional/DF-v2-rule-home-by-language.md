# DF-v2-rule-home-by-language: a version-2 rule kind's `file` home is Markdown by the language `[schema.sources]` reads it as

**Date:** 2026-11-04
**Superseded in part by:** [§DF-markdown-homes-warn-before-1-0](DF-markdown-homes-warn-before-1-0.md#df-markdown-homes-warn-before-1-0-an-md-mapped-away-from-markdown-a-named-index-or-rule-home-not-read-as-markdown-and-an-absent-later-value-home-warn-from-0230-and-are-refused-from-100), for [§DF-v2-rule-home-by-language.2.2](DF-v2-rule-home-by-language.md#22-a-folder-home-is-left-as-it-is) and [§DF-v2-rule-home-by-language.2.3](DF-v2-rule-home-by-language.md#23-a-home-a-source-language-reads-keeps-the-answer-it-had): a rule kind's folder that can hold no Markdown, and a `files` entry a `source` language reads in any position, warn from grund 0.23.0 and are refused from 1.0.0.

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

In version 2 a `file` home is Markdown when the language its extension maps to
is of class `document`, the class the Markdown reader reads
([§FS-config-v2.schema.sources.rules](../../functional-spec/FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading)).
That is one answer with the scan, from the one place version 2 resolves a path
to its language, not a second copy of it. A test for the built-in `markdown`
alone would not be: the scan reads a `document` definition the same way, and
`md = "v1_markdown"`, what `grund config migrate --to 2` writes, must stay a
rule kind. Every entry of `files` is asked, because each is read on its own: a
row is refused when any entry is read by a `data` language or by no admitted
language, not only when its first is. A home no admitted language reads is not
Markdown either: nothing reads its declarations. An entry a `source` language
reads is the one this question does not settle
([§DF-v2-rule-home-by-language.2.3](DF-v2-rule-home-by-language.md#23-a-home-a-source-language-reads-keeps-the-answer-it-had)). The refusal keeps its message and its
location, the `form` key. Version 1 keeps the name test, which is already its
reader's answer.

### 2.2 A `folder` home is left as it is

A `folder` home is accepted on its presence, in both versions, because this
report does not ask about one: deciding at load that a folder holds no Markdown
would be a new rule. That leaves a gap this record does not close. Which of a
folder's files are read is up to the scan, and the scan does not always say when
it reads none of them: a folder rule home it reads nothing from draws no warning
when anything else in the project is read.

### 2.3 A home a `source` language reads keeps the answer it had

An entry a `source` language reads is not refused for its class. It keeps the
answer the row had before this change, the name test on the row's first `files`
entry. So `md = "rust"` over `rules/rules.md` loads, and so does
`files = ["rules/rules.md", "rules/more.rs"]`, with `[schema.sources]` written or
left out. `rules/rules.rs` alone stays refused, and so does
`files = ["rules/rules.txt", "rules/more.rs"]` under `txt = "markdown"`.

The reason is what was reported: a rules file that loads and is never read. A
`source` language does read its file, through its comment hosts, and a rule
declared in a doc-comment there is in force and fires. This change leaves the
row's verdict where it was, so it is no verdict move and owes no route under
[§REQ-backwards-compatibility](../../requirements/REQ-backwards-compatibility.md#req-backwards-compatibility-an-upgrade-never-changes-a-verdict-quietly). Refusing the row would take a project whose
doc-comment rules are in force from `0` to `2`, which this report does not ask
for.

What that leaves standing is the name test for this one class of language:
`rules/rules.rs` is refused where `md = "rust"` loads. That is the same intent
with two verdicts, though here a doc-comment rule is enforced rather than
skipped. It also leaves a gap this record names and does not settle: a rule
written in a form the language does not declare, under `md = "rust"` a Markdown
`# RULE-x:` heading or, under the default `doc-comments`, a `//` comment, is
dropped on a green run, with no warning. That gap belongs to the same ticket as
the question it turns on: whether a rule may live outside Markdown at all, which
a `folder` home already lets through.

## 3. Consequences

- **Three verdicts move from `0` to `2`.** A version-2 rule kind whose `file`
  home is read by a `data` language, or by no admitted language, is now refused
  at load where it loaded and passed. So is one with several `files` entries of
  which a later one is either of those, such as
  `files = ["rules/rules.md", "rules/other.json"]`. The home each refusal is
  about was never read, so its rules were never checked and none of them stops
  applying. The row's other entries are a different matter: `rules/rules.md`
  above was read, its rules were in force, and they stop applying until the row
  is corrected. Where one of them fired, the run exited `1` and now exits `2`.
- **One verdict moves from `2` to `0` or `1`.** A version-2 rule kind whose first
  `file` home has another name, and whose every entry a `document` language
  reads, the built-in `markdown` under `txt = "markdown"` or a `document`
  definition, now loads, and its rules apply, so the run reports what they find.
  No tree that passed stops passing: such a row was refused before anything was
  read.
- **A home a `source` language reads does not move.** A rule in a form that
  language declares was in force and still is, and one in any other form declared
  nothing and still does not ([§DF-v2-rule-home-by-language.2.3](DF-v2-rule-home-by-language.md#23-a-home-a-source-language-reads-keeps-the-answer-it-had)).

**Which clause licenses the `0` to `2` moves.** [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids)
does, and each of its conditions is met:

1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips),
   which already applied when it shipped: a run with a configured rule home it
   never reads is not complete and "never passes as if it were". It also broke
   [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded):
   no section named a rule kind whose home the reader skips, and under a listed
   `data` language the warning that was printed gave a wrong reason, that no
   listed language reads the file.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit:
   there is no old form beside a new one to warn about first, only a
   configuration that already names the same intent the `.json` spelling is
   refused for. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration does not fit
   either, because it has nothing mechanical to write: whether to drop the
   mapping or rename the home is the author's call.
3. *Named release.* The release note below names the move.
4. *Actionable findings.* The replacing finding is the existing refusal, located
   at the row's `form` key, which states what the row must be, a Markdown kind.
   It names neither the entry nor the mapping, and spells `rules = true`,
   version 1's key, so the release note gives two examples of an edit that clears it.
5. *No new licence.* Nothing is newly prohibited: the prerequisite is the one
   [§FS-config.3.4.12](../../functional-spec/FS-config.md#3412-rules--rule-declaration-kinds)
   already stated, now asked of the file the way the reader asks it. This
   correction is not a licence for any other tightening of a rule kind's row.

**The weak points.** The old run did print a warning, so it was not wholly
silent. It still exited `0` over rules it never read, which is the pass the
requirement forbids. Were the route refused, the fallback is the deprecation
path: for one release the refusal is a warning beside the existing one, and a
note names the release in which it becomes the error.

Condition 4 holds at the level of the prerequisite and no finer. Neither edit
the release note gives is at `form`, and the message alone does not tell a
reader which entry is at fault or that a mapping is why. It stays as it is
because the report asks for this case to be refused exactly like the `.json`
control, and [§FS-fields.forms](../../functional-spec/FS-fields.md#forms-form-lowers-to-the-version-1-contracts)
promises version 1's messages at the version-2 key.

## release-note: Release note

- [§FS-config-v2.schema.sources.rules](../../functional-spec/FS-config-v2.md#schemasourcesrules-every-home-of-a-rule-kind-is-judged-by-its-reading), [§DF-v2-rule-home-by-language](DF-v2-rule-home-by-language.md#df-v2-rule-home-by-language-a-version-2-rule-kinds-file-home-is-markdown-by-the-language-schemasources-reads-it-as), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): **a version-2 rule kind's `file` home is Markdown by the language `[schema.sources]` reads it as, not by its name.** **Who this breaks:** a version-2 project whose `form = "rule"` kind names a home that a `data` language reads (`md = "json"`) or that no admitted language reads (`languages = ["json"]`), or whose `files` lists such a home after a Markdown one (`files = ["rules/rules.md", "rules/other.json"]`), loaded with that home's rules unread; it is now refused at the row's `form` key with ``kind `RULE` sets `rules = true` but rule kinds must be citable, scanned Markdown kinds with a `file` or `folder` home and without `values` or `fetch` ``, exit `2`, the refusal the same row already drew under a `.json` name. The verdict moves to `2` from whatever the run gave, `0`, or `1` where it had any finding, and every finding that replaces it gives its location, the row's `form` key, and the action it states, that the row be a Markdown kind. The message names neither the entry nor the mapping, so the edit is the author's; for example, map `.md` back to a `document` language, or give the rules file a name a `document` language reads. Conversely, a home with another name that `[schema.sources]` reads with a `document` language (`txt = "markdown"`, or a `document` definition) was refused and now loads, so its rules apply. A home a `source` language reads is unchanged: `md = "rust"`, or a later `rules/more.rs` after a `.md` first entry, still loads, and `rules/rules.rs` alone is still refused; a rule in a doc-comment there still fires, and a rule in a form that language does not declare, such as a Markdown `# RULE-x:` heading under `md = "rust"`, still declares nothing, with no warning. Version 1 and `folder` homes are unchanged too. Closes [issue #1149](https://github.com/agent-grounds/grund/issues/1149).

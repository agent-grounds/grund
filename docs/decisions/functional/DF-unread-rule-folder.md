# DF-unread-rule-folder: a rule kind's folder home the run reads no file of is named, not refused

**Date:** 2026-11-04

## status: Accepted

## 1. Context

A rule kind's `folder` home is accepted on its presence, in both versions ([§DF-v2-rule-home-by-language.2.2](DF-v2-rule-home-by-language.md#22-a-folder-home-is-left-as-it-is)). Which of its files are read is left to the scan, and the scan did not say when it read none of them:

```toml
[schema.sources]
languages = ["markdown", "json", "rust"]

[schema.sources.extensions]
md = "json"

[schema.kinds.RULE]
folders = ["rules"]
form = "rule"
```

With `rules/rules.md` holding `# RULE-terms: Each FS must cite at least one RULE.` and a spec that cites no rule, `grund check` exited `0`, and printed `success` once the spec was otherwise cited. The same tree with `md` left to Markdown exits `1` with the rule firing. So one line of configuration takes every rule in the folder out of force on a green run, and nothing in the report names the folder or the kind. `languages = ["json", "rust"]` does the same, and so does a version-1 `[scan] extensions` that leaves out `md`. The empty-scan caution speaks only when the whole run read nothing. Reported as [issue #1183](https://github.com/agent-grounds/grund/issues/1183).

## 2. Decision

The folder stays unread, the row keeps loading, and `check` names it. A rule kind's folder home earns the located `unread-scan-root` warning when the run read no file it holds and refused at least one by its extension alone ([§FS-scan.checks.unread-scan-root.rule-folder](../../functional-spec/FS-scan.md#checksunread-scan-rootrule-folder-a-rule-kinds-folder-the-run-reads-no-file-of)). The exit code does not move and `success` is withheld.

### 2.1 A warning, not a refusal at load

A `file` home's own name answers whether it is Markdown, and the configuration alone settles that ([§DF-v2-rule-home-by-language.2.1](DF-v2-rule-home-by-language.md#21-the-check-asks-the-readers-question)). A folder's answer depends on which files are on disk. A refusal would fail every subcommand over a stray file, and would turn when one `.md` is added or removed. The configuration cannot answer it either: "no `document` language is admitted" catches `languages = ["json", "rust"]` and misses `md = "json"` with `markdown` still listed, and widening that test to `source` languages would refuse folders whose doc-comment rules are in force. This is a statement about how much a run covers, which is what the warning already carries ([§FS-scan.checks.unread-scan-root.channel](../../functional-spec/FS-scan.md#checksunread-scan-rootchannel-a-located-finding-on-stdout-and-the-exit-code-does-not-move)).

### 2.2 Counted in the walk's own loop

There is no second listing. The loop over a root's entries already asks the one read predicate of every regular file the walker yields, after hidden directories, `exclude` and the ignore files have been applied ([§AR-scanner.1.13](../../architecture/AR-scanner.md#113-one-predicate-says-whether-a-file-is-read-and-why-not)). Counting there is exactly "read none and refused one by extension alone": hidden, excluded and ignored files stay silent with no code of their own, and no second traversal can drift from the walk's rules. A file is credited to the home that holds it whichever root reached it, and the verdict is taken after every root ([§AR-scanner.1.6](../../architecture/AR-scanner.md#16-the-roots-the-walk-starts-from)), so a folder reached twice gets one warning.

### 2.3 Rule folders only, and one message

Which rows ask is a filter over the one tally. A spec folder that loses its files usually shows up anyway, as dangling citations; a lost rule shows up as nothing, which is why this record singles out rule homes. Nothing in the row, the reason or the fix names a rule, so [issue #1153](https://github.com/agent-grounds/grund/issues/1153) can reuse this record to widen the filter to other folder homes without a second message.

### 2.4 The gap it leaves: a folder a `source` language reads part of

A folder holding any file the run read earns nothing, so `rules/rules.md` under `md = "json"` beside a `rules/helper.rs` rust reads stays silent while every Markdown rule in the folder is out of force. The issue does not rule that out: it excludes only a folder holding a file a `document` language reads. The silence is accepted because a doc-comment rule in the `.rs` file may be in force, and "is not read" would then be false. So this is a gap this record names and does not close. It sits beside the question [§DF-v2-rule-home-by-language.2.3](DF-v2-rule-home-by-language.md#23-a-home-a-source-language-reads-keeps-the-answer-it-had) leaves open, whether a rule may live outside Markdown at all, and belongs with it. A folder `md = "rust"` reads whole is the same question.

## 3. Consequences

- A project, in either version, with a rule kind's folder home the run reads no file of sees one warning per folder in place of `success`, exit still `0`. A script that greps for `success` notices.
- Nothing new is read and no rule starts firing, so no repository gains a finding inside a file.
- An empty folder, an absent one, one whose files are all hidden, excluded or ignored, a mixed one and a non-rule folder home are unchanged.
- `list`, `show`, `fmt` and the other commands that walk stay silent, by [§DF-unread-scan-root.2.1](DF-unread-scan-root.md#21-check-alone-says-it).
- `--ignore unread-scan-root` turns it off like any code.

**This record corrects two sentences.** Both declared the silence it ends, so both gain the exception:

- [§FS-scan.scope.extensions.root](../../functional-spec/FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so): "A directory root is filtered file by file, which is what `extensions` means, so it earns nothing either."
- [§FS-scan.checks.unread-scan-root.silent](../../functional-spec/FS-scan.md#checksunread-scan-rootsilent-what-stays-silent): "a directory whose files are all unlisted earns nothing here".

**Which clause licenses the move.** A new warning moves a verdict ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)). [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) licenses it, on these conditions:

1. *Prior prohibition.* The old verdict violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), which already applied when the verdict shipped. That requirement says a walk that matches nothing "is reported too, as a warning naming what was searched: an empty tree is not a failure, but it must never look like a clean one." Applied to a configured rule home, a folder the row names as where rules live and of which the run read nothing, the old `success` looked exactly like a clean run.
2. *Accepted proof.* This record. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path does not fit, because there is no old form beside a new one to warn about first: the warning is itself the gentlest form a finding has. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical migration does not fit either, because there is nothing mechanical to write: whether to list the extension, undo a mapping or rename the files is the author's call.
3. *Named release.* The release note below names the move.
4. *Actionable findings.* Every warning gives its location, the `folder` or `folders` key of the row, and its fix, the edit the message names.
5. *No new licence.* Nothing is newly prohibited, and the run reads what it read. This correction is not a licence for any other tightening of a rule kind's row or of what a directory root earns.

**The weak point.** The old text did declare this silence, in the two sentences above, so this was a stated behaviour and not an oversight. The case for the route rests on [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips) reaching a configured rule home, not only a whole empty tree, and that sentence speaks of a walk. It is an honest reading rather than a literal one: the declared silence was written about what the extension list means for a directory, and neither sentence weighed a folder whose rules go out of force with it. Were the route refused, the fallback is the deprecation path: the same warning, with a note naming the release from which it counts.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| Refuse the row at load | The folder's answer is on disk rather than in the configuration ([§DF-unread-rule-folder.2.1](DF-unread-rule-folder.md#21-a-warning-not-a-refusal-at-load)). |
| List the folder a second time when a root yields nothing | A second traversal is a second set of rules to drift, and a rule folder is reached by more than one root, so no single walk's result could answer it ([§DF-unread-rule-folder.2.2](DF-unread-rule-folder.md#22-counted-in-the-walks-own-loop)). |
| Warn on every folder home that reads nothing | A folder of another kind can rightly hold only unread files, and its loss usually shows as dangling citations. That is the follow-up, [issue #1153](https://github.com/agent-grounds/grund/issues/1153) ([§DF-unread-rule-folder.2.3](DF-unread-rule-folder.md#23-rule-folders-only-and-one-message)). |
| Say "so no rule in it is in force" | True here, but a rule-specific ending would force a second message on that widening, and the template already says "nothing in it is checked". |

## release-note: Release note

- [§FS-scan.checks.unread-scan-root.rule-folder](../../functional-spec/FS-scan.md#checksunread-scan-rootrule-folder-a-rule-kinds-folder-the-run-reads-no-file-of), [§DF-unread-rule-folder](DF-unread-rule-folder.md#df-unread-rule-folder-a-rule-kinds-folder-home-the-run-reads-no-file-of-is-named-not-refused), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips): **`check` now warns, exit unchanged, where a rule kind's `folder` home holds files the run reads none of.** **Who this breaks:** a project whose rule folder's files have an extension no language or list reads, such as `md = "json"`, `languages = ["json", "rust"]`, or a version-1 `[scan] extensions` that leaves out `md`, and a script that greps for `success` on such a tree. Each such folder earns one `unread-scan-root` warning at its row's `folder` or `folders` key in place of `success`, so the verdict moves from `success` to a warning while the exit stays `0`, and every finding gives its location and the fix the message names: list the extension in version 1, or give the files an extension a listed language reads. Nothing new is read, and no rule starts firing until the row is fixed. An empty folder, one holding only hidden, excluded or ignored files, one holding any file the run reads, and any other kind's folder home are unchanged. Closes [issue #1183](https://github.com/agent-grounds/grund/issues/1183).

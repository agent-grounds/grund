# DF-unmarked-markdown-headings: in-body Markdown ATX headings participate in the knowledge graph

**Status:** Accepted
**Date:** 2026-09-10

Making omitted section coordinates visible serves
[§GOAL-agent-grounding.1](../../goals.md#1-the-three-layers) and
[§GOAL-friendliness-first](../../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

## 1. Context

Grund already validates declarations, section coordinates, and the depth of a
heading that writes a coordinate. It nevertheless accepted an ordinary ATX
heading inside a Markdown declaration body without saying that the heading was
not addressable. A forgotten number and deliberately non-citable structure
therefore produced the same clean result.

The first report also exposed a separate scanner defect: a section-like heading
beyond a declaration body could remain in the prior declaration's section map.
Issue #225 and PR #226 corrected that invariant under
[§FS-declarations.checks.section-outside-declaration](../../functional-spec/FS-declarations.md#checkssection-outside-declaration-section-outside-a-declaration).
This decision begins at the body-local map that predecessor established and does
not reopen its `show --full` boundary.

## 2. Decision

Every non-declaration ATX heading deeper than a Markdown declaration heading and
still inside its body must carry a recognized numeric or enabled named section
coordinate. Before grund 0.16.0, [§FS-declarations.checks.unmarked-heading](../../functional-spec/FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)
reports an unmarked heading as a fixed warning at the heading, names the nearest
enclosing declaration, and suggests a deterministic unused coordinate. In
0.16.0 the same project-wide finding becomes an error.

The body span is the policy boundary. Pre-declaration titles and
same-or-shallower headings that close a body remain legal. Fenced headings are
examples, and source doc-comments, setext text, and bold labels are not Markdown
ATX structure governed by this rule. A nested declaration owns headings in its
own overlapping body because the nearest enclosing declaration is the fact an
author is editing.

Severity is fixed for this rule, for its own reason: there is no
`allow | warn | error` selector and no permanent opt-out because grund ships no
command that can choose an author's intended hierarchy, so there is no standing
an advisory mode could report from — a project holding this finding at advisory
standing would be holding it there forever. That is a judgement about this rule
rather than about the verdict vocabulary, which freezes the severity *set* and
not which rules a project holds in force
([§DF-verdict-vocabulary-freeze.2.4](DF-verdict-vocabulary-freeze.md#24-what-the-test-refuses)).
The warning window follows
[§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)
because the migration cannot be mechanized: with no command able to choose the
hierarchy, every affected heading is hand work.

## 3. Rejected alternatives

**Reject every ordinary heading in a grounded file.** This would make file
titles and body-closing chapter structure part of a declaration they do not
belong to. **Apply the rule to source doc-comments.** Language-native headings
such as Rustdoc `# Examples`, `# Errors`, and `# Panics` are documentation
structure rather than Markdown-file section declarations. **Make severity a
repository setting.** That would turn one project-wide graph invariant into a
per-repository verdict choice and preserve an indefinite escape from the
endpoint. **Number headings in `grund fmt`.** Choosing the next unused number is
mechanical; deciding the intended parent and whether the heading should instead
be a declaration or bold label is not.

## 4. Consequences

Repositories receive one release window to number or reclassify affected
headings before the 0.16.0 error. A warning suppresses the bare `success` line
but leaves exit `0`; text, JSON, selection, and LSP carry the same core finding.
The suggestion is stable guidance and never a write path. `show`, `list`,
`refs`, `cover`, formatting, section resolution, and source scanning keep their
existing behavior.

Agent guidance must teach the narrowed rule. Because the managed block is byte
compared, its existing v9 moves to v10 and `grund init` remains the one-command
block repair required by
[§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations).
No configuration key changes meaning, so `grund_config_version` stays 1. The
scheduled verdict change landed in
[§FS-declarations.checks.unmarked-heading.5](../../functional-spec/FS-declarations.md#checksunmarked-heading5-an-error-in-grund-0160).

The window closed in grund 0.16.0, on the schedule this record named:
`unmarked-heading` is an error, a retained finding exits `1`, and the message's
last clause reads `this became an error in grund 0.16.0`. The site, the
containing declaration, the suggestion, and the text, JSON, and LSP transport
did not move, and `--ignore unmarked-heading` now clears the exit with the
finding, as it does for every other error. The reasoning above stands as
written and is not revised.

## release-note: Release note

- [§FS-declarations.checks.unmarked-heading](../../functional-spec/FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading), [§FS-declarations.checks.unmarked-heading.5](../../functional-spec/FS-declarations.md#checksunmarked-heading5-an-error-in-grund-0160), [§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **an unmarked heading inside a declaration body now fails `grund check`.** The ramp every `0.14.x` and `0.15.x` binary announced in its own output lands on the release it named: `docs/FS-heading-policy.md:5: warning: unmarked heading inside FS-heading-policy; number it (### 1. Origin of the fails queues) as FS-heading-policy.1, declare an ID, or use a bold label; this warning becomes an error in grund 0.16.0` at exit `0` becomes the same line at `error:` ending `; this became an error in grund 0.16.0` at exit `1`, and JSON and the LSP report `"severity":"error"`. [§RM-unmarked-heading-error](../../roadmap.md#rm-unmarked-heading-error-make-unmarked-markdown-headings-errors-in-0160) loses its plan but keeps its address and heading text as a pointer, because released changelogs cite it. Seven e2e cases move from exit `0` to exit `1` — `check-unmarked-heading`, `-json`, `-boundaries`, `-suggestions`, `-only`, `-level-loose` and `-level-warn` — and three that already exited `1` regroup their lines. **What does not move:** the `code`, the location at the heading line, the nearest enclosing declaration, the deterministic suggested coordinate, the exempt headings, `--full`'s narrowing to the configured scan scope, `[id] section_heading_levels` (which governs only `section-heading-level` and never softened this rule), and every command other than `check` — no command numbers the heading. **Who this breaks:** a repository with an unnumbered ATX heading inside a Markdown declaration body — its `check` exits `1` where it exited `0`. The fixes are the ones the message names: number the heading as suggested, declare an ID, or use a bold label; `--ignore unmarked-heading` removes the finding and the exit together. A consumer matching the message's last clause exactly sees it change; a `code` consumer reads the same code. Closes [issue #443](https://github.com/agent-grounds/grund/issues/443).

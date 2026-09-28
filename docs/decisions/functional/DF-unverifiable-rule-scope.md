# DF-unverifiable-rule-scope: a rule the scope cannot judge is reported, rendered, and written

**Status:** Accepted
**Date:** 2026-09-28

## 1. Context

A chapter rule whose object kind is pinned at a workspace member — `workshop/OP`,
the form [§FS-rules.3](../../functional-spec/FS-rules.md#3-the-five-sentence-families) accepts and `docs/user-facing/rules.md` documents without a
scope caveat — was refused by `grund init` in every scope, and the refusal
withheld the managed-block write.

`configured_rule_sentences`, the one function `init` and the managed-block drift
pass share, built its vocabulary with an empty namespace map. Only the
`check --rule` path ever filled one. So a pinned object kind resolved nowhere
`init` ran: not in a member, and not at the workspace root, where `grund check`
in the same directory parses the same sentence and fires it on a real violation.

```console
$ grund check .            # the rule parses and fires
success
$ grund init . --no-vcs
docs/rules/RULE-ops.md:1: error: RULE-ops is not a valid rule: unknown kind "OP" in namespace "workshop"; accepted form: Each FS must cite at least one GOAL.
$ echo $?
1
```

Inside a member the same refusal closes a loop with no exit. A grund upgrade
ages every managed block, so `grund check <member>` reports `outdated grund init
block v1 (run grund init to update to v11)` and names `grund init` as the
remedy; `grund init <member>` then refuses over the rule and writes nothing. The
member's `AGENTS.md` cannot be brought current by any command, and the only
escape is to move the rule file aside, run `init`, and put it back — an edit of
the repository around the tool. Reported as
[issue #277](https://github.com/agent-grounds/grund/issues/277).

Underneath both sat a third thing nobody had seen, because it was silent.
`check_agents_block_version` called the same function as `.ok().flatten()`, so
whenever *any* configured rule failed to resolve the whole `### Chapter rules`
section was dropped from the byte comparison and no finding was emitted. A
member's managed block could be missing a bullet, or hold arbitrary text in
place of one, and a run at the workspace root exited `0`.

## 2. Decision

### 2.1 The vocabulary follows the run's own workspace, and only its own

`init` and the drift pass build the namespace vocabulary `grund check` already
builds, from the workspace the run's **own effective config** declares. At a
workspace root that resolves `workshop/OP` exactly as `check` does, which is the
plain defect: [§FS-rules.4](../../functional-spec/FS-rules.md#4-validation-lifecycle) says a rule is *resolved* before managed-block
rendering, and one command in one directory resolved it while the other did not.

The workspace is never climbed to. `init` does walk up to render
`### Workspace members`, and that walk stays where it is, because it is
*teaching* — it tells an agent which aliases exist. Resolving a rule is
*judging*, and judging off a climbed tree would make one sentence valid or
invalid according to what happens to sit on disk beside the checkout: a member
cloned alone would get a different verdict from the same bytes.
[§FS-workspace.5.1](../../functional-spec/FS-workspace.md#51-a-member-run) already settles the analogous case for a citation by reporting
at the site rather than resolving across the boundary, and the relaxation of
that boundary is a deferred design of its own
([§DF-subproject-namespaces.3.6](DF-subproject-namespaces.md#36-standalone-members-fail-loud-not-silent)). Nothing here pre-empts it.

### 2.2 Unverifiable is a verdict the parser reaches, not a judgement

A member-scoped run holds no namespace vocabulary at all, so a pinned object
kind there is neither resolvable nor refutable. [§FS-rules.4.1](../../functional-spec/FS-rules.md#41-a-rule-this-scope-cannot-verify) gives that case
its own name and one mechanical test: the vocabulary holds no namespace
whatever. `typo/OP` at a workspace root, where the vocabulary holds namespaces
and none is named `typo`, stays an invalid rule with every consequence it has
today. The exception is exactly the case where the *scope itself* is the reason
the alias cannot be judged, and never the case where the scope could judge it
and the answer was no.

The finding keeps its `invalid-rule` code and its nonzero exit; only the reason
text changes, and it changes through the wording migration [§FS-errors.3.7](../../functional-spec/FS-errors.md#37-the-rule-site-unknown-alias-wording-migration) fixes.
The exit code was deliberately left alone: it is the write that stops being
withheld, not the failure that is forgiven, so nothing that gates on `grund
init` succeeding starts passing.

### 2.3 The unverifiable bullet is rendered as authored, not omitted

`init` writes the managed block when every unresolved rule is unverifiable here,
and renders each such sentence into `### Chapter rules` exactly as its heading
spells it ([§FS-rules.9.1](../../functional-spec/FS-rules.md#91-one-tree-renders-one-block)).

Omitting the bullet was the tidier option and is rejected. The rendered section
is config-derived content `check` re-renders and byte-compares, and the two
scopes would then render two different sections for one file: the member writes
a block with no bullet, and the workspace root — which *can* resolve the alias —
re-renders one with the bullet and reports the member's own file as drifted. A
member could not run `init` and then pass CI. Rendering as authored keeps
[§REQ-deterministic-output](../../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) over the pair: same tree, same bytes, whichever scope
the command ran in.

### 2.4 The drift pass stops dropping a section it cannot fully resolve

`check` renders `### Chapter rules` from the valid rules together with the
unverifiable ones and byte-compares it, in every scope. Only a genuinely invalid
rule leaves the section unrenderable, and such a run already carries that rule's
own located error at its heading, so the omission is named where a reader can
act on it ([§FS-check.3.5.4](../../functional-spec/FS-check.md#354-no-config-derived-section-is-exempt-from-the-comparison)). The unverifiable rule's diagnostic is not repeated
by this pass: `check_chapter_rules` reports it at the rule's own site, and one
fact is one finding.

## 3. Rejected alternative: let a member-scoped run climb to its parent workspace

The ticket offered this as its larger option: resolve `workshop/OP` in member
scope by loading the enclosing workspace, so `grund init <member>` exits `0`.

It is rejected for the reason [§DF-unverifiable-rule-scope.2.1](DF-unverifiable-rule-scope.md#21-the-vocabulary-follows-the-runs-own-workspace-and-only-its-own) gives, and the ticket's own reproducer shows
the cost. In that same member-scoped run the citation `<§>workshop/OP-weld` is
reported as `unknown project alias workshop` at its own line, deliberately and
by specification. Resolving the rule while refusing the citation would leave one
run answering two ways about one alias; resolving both would be the relaxed
standalone mode [§DF-subproject-namespaces.3.6](DF-subproject-namespaces.md#36-standalone-members-fail-loud-not-silent) holds as a design of its own,
decided against a goal — [§GOAL-no-dangling-refs](../../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration) — rather than in passing while
fixing an `init` refusal.

The narrow exception delivers what the ticket actually needs, which is an exit
from the upgrade loop, and leaves that question open rather than answering it
by accident.

## 4. Consequences

- A member holding a cross-boundary rule can complete a grund upgrade from
  inside itself. [§REQ-agents-md.2](../../requirements/REQ-agents-md.md#2-the-managed-block-stays-current) — "the managed block stays current … a
  missing, older, or unsupported block fails the build" — becomes satisfiable
  there, which it was not: `check` demanded the refresh and `init` refused it.
- `grund init` at a workspace root over such a rule moves from exit `1` to exit
  `0` and writes a block carrying the bullet. A refusal becoming a success
  breaks nothing that is sensible to gate on, and the release names it.
- A member's `AGENTS.md` gains a bullet on the next `grund init`, which is a
  tracked-file change and the point of [§DF-unverifiable-rule-scope.2.3](DF-unverifiable-rule-scope.md#23-the-unverifiable-bullet-is-rendered-as-authored-not-omitted).
- **One verdict moves from `0` to `1`.** With a sibling-pinned rule present the
  `### Chapter rules` comparison was skipped in silence, so a managed block
  missing a bullet, or holding arbitrary text in its place, passed `grund check`
  at the workspace root. That comparison now runs, and a repository that worked
  around the refusal gets a new `agents-init` error whose remedy is the one
  command that used to refuse.

  **Which clause licenses the move.** [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) does, because the old silence violated the separately declared prohibition in [§REQ-no-missed-citation.2](../../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded) — "a skip that no section names is the bug, because it is the one a reader cannot plan around" — and no section named this one: [§FS-rules.9](../../functional-spec/FS-rules.md#9-managed-guidance-and-editor-parity) required the comparison in as many words while the code omitted it. That requirement already applied when the verdict shipped, so this record states the conflict rather than inventing a prohibition for the correction.

  Neither ordinary route fits. The [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) deprecation path cannot apply, because no old form is carried beside a new one — the section was never compared, so there is nothing to warn about first and no window in which a repository could see the finding coming. The [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations) mechanical-migration route has no command to point at either, or rather it has exactly the one that did not work: the migration is `grund init`, and the refusal this change removes is why a repository could not run it.

  The correcting release names the verdict change, and every finding that replaces the old verdict names its location and the action to take — the drift error anchors at the block's own `path:line` and names `grund init` as the refresh. This is the bounded correction route and not a licence for ordinary policy tightening, feature removal, or a newly invented prohibition; each of those still owes [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) or [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations).
- A project with no `rules = true` kind is byte-identical: no vocabulary is
  built, the v10 block keeps its bytes, and no section is added to compare.
- A repository that greps the rule-site message text sees a longer line for one
  release window, with the legacy reason still a contiguous prefix;
  `code == "invalid-rule"` is the stable thing to match, and
  `--only invalid-rule` still selects the finding.

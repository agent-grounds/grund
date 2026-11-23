# FS-cochange-recipe: an opt-in Git recipe reports related declaration and test edits

The maintained `examples/cochange/` recipe serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers) and
[§FS-examples.2](FS-examples.md#2-canonical-use-cases). From a commit-msg hook or in CI, it checks, file by file, that each changed
implementation file cites an eligible declaration, and that the same change edits both a file
declaring it and a configured test citing it. It is copyable Python 3.11+ standard-library
code requiring Git 2.32+ and supporting released Grund 0.16.1. It uses existing queries and
changes no standing check; `grund` ships no command, history or range API or rule-language
extension for it.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, section, coordinate, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation, qualified citation),
[§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace, member, alias), and
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity).

- **comparison** — The resolved base tree and candidate tree whose Git changes
  provide evidence. Git trees here are distinct from fetcher snapshots.
- **obligation** — An eligible grounding requirement or a required related edit
  for one changed source path.
- **waiver** — A commit trailer that excuses named missing edit obligations for
  exact paths changed by that commit, with a reason.

## inputs: Explicit policy and entry points

The entry point is `python3 examples/cochange/cochange.py --repo <Git-root> --policy
<JSON-file> --grund <executable>` followed by a mode. `--policy` and `--grund` are resolved
from the invoking directory, a bare `--grund` name through `PATH`. The policy is exactly
`{"config_root":".","source_paths":["src/"],"test_paths":["tests/e2e/"],"eligible_kinds":["FS"]}`
with caller-selected values. Its paths are Git-root relative exact files or directory
prefixes ending `/`, and `.` is allowed only as the config root. Globs, escapes, absolute
paths, empty or overlapping source and test selections, unknown keys and wrong types refuse,
and `eligible_kinds` must be `["FS"]`.

`commit-msg --base <commit-or-empty> --message <file>` evaluates the index, with that
message, against the base the caller supplies: `HEAD` for a new commit, its first parent
for an amend, or `empty`, Git's empty tree, for a root commit; an amend is never guessed.
`ci --target <commit> --head <commit>` compares the unique merge base of the two with
`--head`, over every commit reachable from `--head` but not from that base. The caller
supplies the actual PR head and the local history needed: the recipe never fetches.

## snapshots: Exact trees and truthful refusals

The recipe never changes the contributor's index bytes or working files; `commit-msg` reads
its candidate tree from a copy of the index, writing that tree's objects to the repository's
object store. Git's comparison and every query see the same contents of each tree: exactly
its tracked blobs and modes, copied in isolation, never the working tree. A tracked symlink
keeps its mode, and one that leads outside the isolated copy or into a cycle refuses.

In each tree it needs, even for an empty comparison, the recipe runs `check`, `cover`,
`list` and batch `show` independently at the configured root, under that tree's own
configuration ([§FS-cover.4](FS-cover.md#4-exit-codes), [§FS-list.4](FS-list.md#4-exit-codes),
[§FS-show.1.8](FS-show.md#18---batch-an-explicit-query-stream)). A query that fails, or whose
output is not complete and well-formed with each batch result answering its own query in
order, refuses, so partial output never produces success. A changed source that Grund scans
but that cites nothing does not refuse; it fails eligible grounding. Every ordinary `check`
finding is kept, whatever its severity, and a `check` error refuses with its explanation
visible.

The recipe also refuses missing objects, shallow history, no unique merge base, changed
project names, aliases or member wiring, roots or members outside the Git tree, external
fact roots outside it, submodules, unmerged, intent-to-add, split or sparse indexes,
non-UTF-8 paths, and tracked names or modes the host filesystem cannot represent, which it
names by exact Git path and never renames or omits. A refusal of an unsupported input or a
failing query names it and a corrective action.

Renames are detected at `--find-renames=50%`. Added, edited and renamed paths are judged by
their new names and candidate facts, deleted ones by their old names and base facts. A
source file keeps its obligations when it is renamed, a pure move included, or only changes
mode; a declaration or test file supplies evidence only by a content addition, edit or
deletion.

### snapshots.scope: Every classified file is one Grund scans

Every tracked file that `source_paths` or `test_paths` selects must be one the scan reads,
whether or not the comparison touches it. One with no `cover` row refuses every run and is
never skipped: exit 2, code `scan-scope`, its Git path and query `cover`. The message names
the selection key or keys that classified it, says Grund does not scan it, and offers
narrowing that selection to leave it out or removing it from Git. A source moved to a
destination with no `cover` row refuses the same way at the destination; as a move is
classified by its old path, that message names no selection and offers moving it to a path
Grund scans whose own name is not hidden. Either message offers adding the file to the scan
configuration only when its own name is not hidden, since no configuration reads a hidden
file ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)).

The shipped `examples/cochange/policy.json`, which the guide shows, selects only files that
a repository laid by `grund init --scaffold` scans, so such a repository's root commit
passes with it.

## evidence: Both edits for one resolved direct target

Each changed classified source file must directly cite at least one resolved target of an
eligible kind, and for at least one such target the comparison must change the content of
both a file declaring it and a configured test file directly citing it; evidence split
between two targets does not pass. Every citation's full coordinate, section included, must
positively resolve, or the run refuses with `resolution`; catalog presence is not enough. A
resolved citation is reduced to `(project, declaration-root)`, and a qualified citation is
read through the alias its citing project sees ([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)). The same bare ID in another workspace
member, or another declaration, is unrelated. No transitive citation is followed, and no
coverage is inferred from a nearest preceding citation.

The recipe prints the limitation `file-level related edits; no changed-line coverage,
semantic correctness or test execution proof`. An unrelated edit within a matched
multi-declaration file may pass. A valid implementation correction, a refactor or a test
classified elsewhere may fail until waived or configured. Running tests remains CI's job.

## waivers: JSON commit trailers and commit-local scopes

A waiver is a trailer of the final message under the exact key `Grund-Cochange`, read with
Git's trailer parsing; there is no committed waiver file or source pragma. Its value is one
JSON object with exactly `paths`, `missing` and `reason`:

```text
Grund-Cochange: {"paths":["src/lib.rs"],"missing":["spec"],"reason":"Fix implementation to the existing contract"}
```

Both arrays are nonempty and hold unique strings: `paths` exact normalized Git-root paths,
`missing` a subset of `spec` and `test`. `reason` is a string, nonempty after trimming. Globs,
duplicate JSON keys, unknown keys or values, multiline JSON, absolute or escaping paths and
duplicate or conflicting `(commit, path, obligation)` entries refuse even when no evidence is
missing; several trailers may cover disjoint paths and obligations. Each path must be a
classified source that its own commit changes, against the first parent for a merge, the
supplied base for `commit-msg` and the empty tree for a root commit.

Evidence counts across the whole comparison, but waivers are commit-local: every selected
commit that changes a source path must itself waive each obligation still missing there,
and a commit that does not fails the obligation and is named. So a merge whose first-parent
diff brings in a source change needs its own trailer, and a newly squashed head inherits
none from the commits it replaces. A scope never passes to another name across a rename,
and valid unused trailers are reported as unused. A waiver excuses only a missing evidence
class on a selected eligible target, never missing grounding or a check, Git, config, scan,
query or unsupported-input failure.

## output: Deterministic JSON report

Stdout is one UTF-8 JSON object and a newline, containing `mode`, `policy` (validated, else
the parsed input on a config refusal, else null), `base` and `candidate` objects, each with
the full `commit` and `tree` identities (`commit` null for the empty base or an index
candidate), `limitation` ([§FS-cochange-recipe.evidence](FS-cochange-recipe.md#evidence-both-edits-for-one-resolved-direct-target)), `sources` and `errors`.

A source, in Git-root path order, has `path`, `status` (`added`, `modified`, `renamed`,
`deleted`), `targets` (every eligible target), `missing` and `waivers`. A target has
`project` (null for standalone), `id` (the bare declaration root), `spec_paths`,
`test_paths` and `snapshot` (`base` or `candidate`). Paths and targets are sorted, and
missing classes are ordered `grounding`, `spec`, `test`. `missing` holds the obligations
unsatisfied after waivers: `grounding` alone without a target, else those of the target
with the fewest unsatisfied classes, ties broken by `(project, id)`. A waiver has `commit`
(null for the index message), `path`, `missing`, `reason` and `used`, and keeps the scope
it actually excuses.

An error has `code`, `message` and nullable `path`, `commit` and `query`. The co-change
codes are `missing-grounding`, `missing-evidence` and `missing-waiver`, whose message names
the source and, for missing evidence or a missing waiver, the selected target and the
missing class. The refusal codes are `git-input`, `config`, `unsupported-input`, `check`,
`query`, `scan-scope`, `resolution` and `waiver`, an unresolved target being `resolution`
and a malformed envelope `query`. A refusal may report an unavailable identity as null and
keeps its partial facts explicit, never looking like successful evidence. Errors sort by
`(path, code, commit, query, message)`, null as empty.

Paths keep their spaces and newlines, and no absolute scratch path appears. Stderr is empty
for an evaluated report; argument-parser errors may use it. Identical inputs produce
identical bytes, apart from the explicit entry mode.

## exit: Recipe results are independent of Grund verdicts

Exit 0 requires complete input validation and every obligation satisfied or validly
waived; exit 1 is missing eligible grounding or an unwaived related edit; exit 2 is a
refusal, under any refusal code of [§FS-cochange-recipe.output](FS-cochange-recipe.md#output-deterministic-json-report), and takes precedence over
obligations. These are the recipe's exits: Grund's own exits and standing errors are
preserved as facts and never converted into success. With identical trees, policy and
applicable messages, `commit-msg` and `ci` produce the same obligations, evidence and waiver
reasons.

## examples: Maintained walkthrough, tests and opt-in guidance

`examples/cochange/` holds the recipe, its policy and a self-contained walkthrough of its
verdicts and waivers, whose shared-runner goldens Grund CI checks ([§FS-examples.5.4](FS-examples.md#54-commandexternal-invokes-an-explicit-json-argv)). A
compatibility run evaluates fresh checkouts with both the built binary and released 0.16.1,
and fails rather than skips when its setup fails. `docs/user-facing/cochange.md` is the hook
and CI guide. Grund's own contributions are not gated by the recipe.

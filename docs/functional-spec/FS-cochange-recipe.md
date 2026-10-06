# FS-cochange-recipe: an opt-in Git recipe reports related declaration and test edits

The maintained `examples/cochange/` recipe serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers) and
[§FS-examples.2](FS-examples.md#2-canonical-use-cases). It reports file-level evidence for changed implementation files.
It is copyable Python 3.11+ standard-library code requiring Git 2.32+ and supporting
released Grund 0.16.1. It uses existing queries; it introduces no shipped command,
history/range API, rule-language extension or changed standing check.

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

The entry point is `python3 examples/cochange/cochange.py`. Common arguments are
`--repo <Git-root> --policy <JSON-file> --grund <executable>`; paths to the policy
and executable are resolved before creating isolated trees. Policy is exactly
`{"config_root":".","source_paths":["src/"],"test_paths":["tests/"],"eligible_kinds":["FS"]}`
with caller-selected values. Paths are Git-root relative exact files or directory
prefixes ending `/`; `.` is allowed only for the config root. No globs, escapes,
absolute paths, empty selections or overlapping source/test selections are
accepted. Unknown keys and types refuse. Eligible kinds initially support `FS`.
The eligible kind is `FS`. The policy fixes both evidence classes, direct matching and at least one shared
target; it has no implicit alternative policy.

`commit-msg --base <commit-or-empty> --message <file>` evaluates the index with
the supplied message. `empty` denotes Git's empty tree. Callers supply HEAD for
a new commit, its first parent for an amend, and `empty` for a root commit.
Amendment is never guessed. `ci --target <commit> --head <commit>` selects the
unique merge base and all commits reachable from head but not that base.
Callers supply the actual PR head, including a newly squashed commit, and arrange
adequate local history; the recipe never fetches.

## snapshots: Exact trees and truthful refusals

Resolve and report full commit/tree object identities. Index mode reports no
candidate commit and uses a copied index with Git object storage to obtain its
tree. Never change contributor index bytes or working files. Every Git comparison
and Grund query uses the same isolated tracked blob/mode contents, without checkout
filters, archive substitution, inherited parent ignores or live-tree fallback.
Keep symlink modes without following them outside the isolated root.

Run independent `check`, `cover`, `list` and batch `show` in each required tree at
the configured root ([§FS-cover.4](FS-cover.md#4-exit-codes), [§FS-list.4](FS-list.md#4-exit-codes), [§FS-show.1.8](FS-show.md#18---batch-an-explicit-query-stream)). Ordinary config changes
are interpreted in their own trees. Preserve project identity and citing-project
alias resolution ([§FS-workspace.8.6](FS-workspace.md#86-grund-cover)). Compare normalized project identities, never
bare IDs across members. Catalog presence is insufficient: every used citation
must positively resolve, including its section, before reduction to its root.
A qualified citation is normalized using the alias as seen by its citing project.
No independent citation parser or one full scan per ID is permitted.

Require complete query results: success exits, valid NDJSON records, unique cover
rows for each classified existing path, catalog identities, and one ordered
successful batch envelope for every submitted query with matching query/result.
Missing, duplicate, truncated, malformed or failed records refuse; a scanned row
with no citations instead fails eligible grounding. Preserve every ordinary check
finding; errors refuse and their explanations remain visible. Empty comparisons
still validate required config/queries.

Refuse missing objects, shallow history, no unique merge base, changed project
names/aliases/member wiring, roots or members outside the Git tree, external fact
roots outside it, submodules, unmerged/intent-to-add/split/sparse index forms and
non-UTF-8 paths. Name the unsupported input or failing query and a corrective
action. Partial query output can never produce success. Query formats, ordinary
severity/exit mappings and offline behavior stay unchanged.

Use NUL-delimited Git paths and fixed rename detection (`--find-renames=50%`).
Compare blob identities and modes to distinguish pure moves and mode-only edits.
Additions/content edits use candidate facts; deletions use base facts. Renames use
new paths/candidate facts; deleted paths use their old names/base facts. Source
renames, including pure moves, and source mode changes retain obligations. Pure
declaration/test moves and mode-only changes supply no evidence. Content additions,
edits and deletions may supply evidence using facts from their corresponding tree.

## evidence: Both edits for one resolved direct target

Each changed classified source file must directly cite at least one eligible
resolved target. Normalize each citation to `(project, declaration-root)` only
after resolution of the full coordinate. Follow no transitive citation edges.
For at least one such target require both a content-changed declaring file and a
content-changed configured test file directly citing that same target. Test paths
are classification, not an `E2E-` assumption. Report all eligible targets and
matched paths. Evidence split between two different source targets cannot pass.
An unrelated edit cannot count merely because it has the same bare ID in another
member or cites another declaration. Deletion evidence retains base provenance.

Waivers excuse only missing evidence classes on a selected eligible target;
they cannot supply grounding. An unchanged-contract fix has a test edit plus a
spec waiver; a refactor may waive both edits explicitly. Unwaived spec-only and
test-only comparisons fail. A declaring file outside the comparison cannot count
as edited local evidence; only a bounded evidence waiver can excuse its absence.

Print the limitation `file-level related edits; no changed-line coverage,
semantic correctness or test execution proof`. An unrelated edit within a matched
multi-declaration file may pass. Valid implementation corrections, refactors and
tests classified elsewhere may fail until waived or configured. Never infer
coverage from a nearest preceding citation. Running tests remains CI's job.

## waivers: JSON commit trailers and commit-local scopes

Use Git's trailer parsing on the final message, with exact key `Grund-Cochange`.
Each value is one JSON object with exactly `paths`, `missing`, `reason`:

```text
Grund-Cochange: {"paths":["src/lib.rs"],"missing":["spec"],"reason":"Fix implementation to the existing contract"}
```

Both arrays are nonempty, contain unique strings, and name exact normalized
Git-root paths and a subset of `spec`, `test`. Reason is a nonempty string after
trimming. No globs, duplicate JSON keys, unknown keys or values, multiline JSON,
absolute/escaping paths or duplicate/conflicting `(commit,path,obligation)` entries
are accepted. Multiple trailers may cover disjoint obligations/paths. Malformed
trailers refuse even when no evidence is missing. Scope paths must be classified
sources actually changed by that message's commit, using first-parent comparison
for merges, the supplied base for commit-msg and the empty tree for root commits.
No committed waiver file or source pragma is supported.

For each obligation still missing in the whole comparison, every selected commit
changing that source path must waive it itself. Earlier trailers do not waive
later repeated edits. Importing a change through a merge can require a merge
trailer because its first-parent diff changes the path. Whole-PR evidence can
satisfy obligations across commits; valid unused trailers are reported as unused.
A squash is a new comparison with its new message, without inherited waivers.
Path renames do not implicitly transfer trailer scopes to another name: report
each commit's changed path explicitly. Missing commit-local coverage fails the
co-change obligation and names the offending commit. A waiver never excuses
grounding, ordinary check, Git/config/scan/query or unsupported-input failures.

## output: Deterministic JSON report

Stdout is one UTF-8 JSON object followed by a newline. It contains `mode`,
`policy` (the validated policy, or the parsed input on config refusal, null if
unreadable), `base` and `candidate` objects with `commit`
(null for empty base/index candidate) and `tree`, `limitation` (the text above),
`sources` and `errors`. Sources are ordered by Git-root path; each has `path`,
`status` (`added`, `modified`, `renamed`, `deleted`), `targets`, `missing` and
`waivers`. A target has `project` (null for standalone), `id` (bare declaration
root), `spec_paths`, `test_paths`, and `snapshot` (`base` or `candidate`). Paths
and targets are sorted; missing classes order is `grounding`, `spec`, `test`.
Source `missing` describes obligations still unsatisfied after waivers. For
missing evidence, select the target with the fewest unsatisfied classes, breaking
ties by `(project, id)`; no target yields only `grounding`. Each
waiver has `commit` (null for the supplied index message), `path`, `missing`,
`reason`, `used` and preserves the scope it actually excuses.

Errors have `code`, `message` and nullable `path`, `commit`, `query`. Co-change
codes are `missing-grounding`, `missing-evidence`, `missing-waiver`; refusal
codes are `git-input`, `config`, `unsupported-input`, `check`, `query`,
`scan-scope`, `resolution`, `waiver`. Messages name the source/target, missing
class or failed input/query as applicable. Unresolved targets are resolution
refusals; malformed envelopes are query refusals. Used waiver reasons and scopes
remain visible on success. Refusals may report unavailable identities as null and
retain partial facts explicitly; they cannot look like successful evidence.
JSON escaping preserves spaces/newlines and there are no absolute scratch paths.
Errors sort by `(path, code, commit, query, message)`, with null treated as empty.
Stderr is empty for evaluated reports; argument-parser errors may use stderr.
Identical inputs produce identical bytes, apart from the explicit entry mode.

## exit: Recipe results are independent of Grund verdicts

Exit 0 requires complete input validation and all obligations satisfied or
validly waived. Exit 1 is missing eligible grounding or unwaived related edits.
Exit 2 refuses Git/config/check/query/resolution/trailer/unsupported inputs and
takes precedence over obligations. These are recipe exits; Grund's own exits and
standing errors are preserved as facts and never converted into success.
With identical trees, policy and applicable messages, commit-msg and CI produce
the same obligations, evidence and waiver reasons. CI additionally records commit
identities while index mode records null for its prospective message.

## examples: Maintained walkthrough, tests and opt-in guidance

`examples/cochange/` includes the recipe, policy, self-contained walkthrough and
shared-runner goldens ([§FS-examples.5.4](FS-examples.md#54-commandexternal-invokes-an-explicit-json-argv)). Demonstrate missing grounding/evidence,
spec-only/test-only failures, unrelated/split targets, complete evidence,
unchanged-contract test-plus-spec-waiver and explicit refactor waivers. Synthetic
Git tests cover partial staging with unstaged distractors, missing history,
additions/edits, pure/edited source moves, deletions, pure evidence moves, modes,
spaces/newlines, empty comparisons and failures on empty comparisons.

Keep workspace duplicate IDs, qualified citations, alias normalization, unresolved
sections, missing cover rows, query failure/partial output and batch completeness
pinned. Pin unsupported inputs, malformed/conflicting/out-of-range/stale waivers,
repeated edits, multi-commit/merge first-parent scopes, amend/root bases, squashes,
and exact-tree local/CI parity. Run a fresh checkout against both the built binary
and released 0.16.1; setup failures must not silently skip this compatibility run.

README hook/CI guidance links `docs/user-facing/cochange.md`; the guide index,
examples index and `tests/e2e/README.md` link/document this workflow and manifest.
Document explicit bases, amend/root/merge/squash handling, CI history setup,
trailer grammar, limitations and captured real outputs. Demonstrations run in
Grund CI through the existing harness; adoption does not enable the contribution
gate for Grund itself. No changelog entry or release work is part of this recipe.

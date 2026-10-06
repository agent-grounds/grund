# Opt-in Git co-change evidence

[§FS-cochange-recipe.evidence](../functional-spec/FS-cochange-recipe.md#evidence-both-edits-for-one-resolved-direct-target)
defines the copyable [recipe](../../examples/cochange/cochange.py). It reports
whether each changed source file has a related spec edit **and** a related test
edit for at least one directly cited declaration. Project identity and full
coordinate resolution precede declaration-root matching. This is an optional
repository policy. Grund's ordinary checks remain a separate requirement.

**Install and configure**

Use Python 3.11+, Git 2.32+ and Grund 0.16.1 or a compatible newer build. One
portable installation of the supported release is:

```bash
cargo install grund --version 0.16.1 --locked
```

Copy the entire `examples/cochange/` directory into your repository, keeping
its Python modules together. Edit `policy.json` to select the Git-relative
config root, source and executable-test locations. The four keys are required:

```json
{"config_root":".","source_paths":["src/"],"test_paths":["tests/"],"eligible_kinds":["FS"]}
```

Selections are exact files or directory prefixes ending `/`. Source and test
selections must be disjoint. There are no globs or policy switches for weaker
evidence. Test classification should name your executable tests; the recipe
only reports edits and CI must execute them. Paths in results are Git-relative,
even when `config_root` names a nested app. Configuration edits are read from
their own snapshots. Namespace migrations and roots outside the Git tree are
explicitly unsupported.
Snapshots also refuse symlink cycles and tracked names or modes the host cannot
represent, naming the exact Git path. For example, a newline in a filename is
supported on POSIX but requires another host for evaluation on Windows. The
recipe never renames or drops unsupported paths to obtain a result.

**Local commit-msg hook**

[§FS-cochange-recipe.inputs](../functional-spec/FS-cochange-recipe.md#inputs-explicit-policy-and-entry-points)
requires an explicit base. For a new commit use `HEAD`, for an amend use the
commit's first parent, and for a root commit use the literal `empty`. The recipe
cannot infer an amendment from the staged index. This hook requires the caller
to supply that choice, so an amend is never silently checked against itself:

```sh
#!/bin/sh
set -eu
: "${GRUND_COCHANGE_BASE:?Set HEAD for new commits, HEAD^ for amends, or empty for a root}"
repo=$(git rev-parse --show-toplevel)
python3 "$repo/examples/cochange/cochange.py" \
  --repo "$repo" --policy "$repo/examples/cochange/policy.json" \
  --grund "$(command -v grund)" \
  commit-msg --base "$GRUND_COCHANGE_BASE" --message "$1"
```

Install this as `.git/hooks/commit-msg` (or integrate it into your existing
commit-msg hook) and make it executable. Examples:

```bash
GRUND_COCHANGE_BASE=HEAD git commit
GRUND_COCHANGE_BASE='HEAD^' git commit --amend
GRUND_COCHANGE_BASE=empty git commit             # first commit or root amendment
```

The hook copies the index and evaluates staged blob contents. Unstaged spec or
test edits supply no evidence. It changes neither index bytes nor working files.
It refuses unmerged, intent-to-add, split and sparse indexes: resolve conflicts,
stage full contents, use `git update-index --no-split-index`, or disable sparse
checkout before evaluating. Waivers need the final message, so this recipe is
enforced at commit-msg rather than pre-commit.

**CI and PR history**

Fetch complete local history during CI setup. For GitHub Actions, use
`actions/checkout` with `fetch-depth: 0`, explicitly check out the actual PR head,
and ensure the target commit exists locally. Run:

```bash
python3 examples/cochange/cochange.py \
  --repo "$PWD" --policy examples/cochange/policy.json --grund "$(command -v grund)" \
  ci --target "$PR_TARGET_SHA" --head "$PR_HEAD_SHA"
```

The comparison starts at the unique merge base and ends at the actual head;
it spans the whole PR, allowing evidence in a later commit. Missing/shallow
history or multiple merge bases refuse with setup guidance. The recipe never
fetches. Each selected commit's waiver scope uses its first-parent diff; a merge
importing a source change can need its own trailer. A newly squashed commit is
evaluated with its new message and cannot inherit discarded commit trailers.
Run the same independent Grund gate and your tests in CI. Grund's own CI runs
the recipe's demonstrations; it does not enforce this policy on contributions.

**Reason-bearing exceptions**

[§FS-cochange-recipe.waivers](../functional-spec/FS-cochange-recipe.md#waivers-json-commit-trailers-and-commit-local-scopes)
permits a final Git trailer block such as:

```text
Fix alpha to its existing contract

Grund-Cochange: {"paths":["src/lib.py"],"missing":["spec"],"reason":"Fix implementation to the existing contract"}
```

That fix still needs a related test edit. A refactor may explicitly name both
`spec` and `test`. Each path must be an exact classified source changed by that
commit. Every commit changing it must waive any class still absent in the whole
comparison; an earlier reason never excuses a later edit. Renames do not transfer
waivers to another name. Multiple trailers in the same final block may cover
disjoint paths/classes. Arrays are nonempty and unique, JSON keys are exact and
unique, and reasons are nonblank. No multiline JSON, globs, stale scopes or
conflicting entries are accepted, even if no waiver would be needed. Newlines
in path names use JSON `\n` escapes. Valid unused trailers remain visible.
Waiver `missing` preserves the authored scope; `used_missing` identifies the
classes it actually excuses for the selected target.

Waivers never excuse missing eligible grounding, unresolved sections, standing
Grund errors, incomplete queries or unsupported Git/configuration inputs.

**Interpret results**

[§FS-cochange-recipe.output](../functional-spec/FS-cochange-recipe.md#output-deterministic-json-report)
requires one deterministic JSON object on stdout, with exact base/candidate
commit and tree identities, target project/root identities, matched paths,
missing obligations and waiver reasons/scopes. Index candidates have no commit
yet. `check_findings` retains ordinary findings with snapshot provenance, including
warnings that do not cause refusal. Recipe exits are 0 for satisfied/waived evidence, 1 for missing obligations,
and 2 for refused input. These exits do not change Grund's own verdicts. Deletions
use base facts; additions and edits use candidate facts. Source moves and mode
changes retain obligations. Pure spec/test moves and mode changes supply no edit.
Moving a source outside its configured prefix retains its source obligation
under the new name; the destination must still be scanned. Waivers name that
new path exactly.
`unused_waivers` retains valid commit-local scopes for paths absent from the net
comparison, such as edits reverted later in the PR. Those scopes transfer to no
other path and excuse no obligation.

The printed limitation is: `file-level related edits; no changed-line coverage,
semantic correctness or test execution proof`. An unrelated change inside a
matched multi-declaration file can pass. A valid correction or refactor can fail
until explicitly waived; tests outside configured paths can fail until configured.
There is no inference from a nearest preceding citation and no transitive graph
matching. See the [captured walkthrough](../../examples/cochange/README.md).

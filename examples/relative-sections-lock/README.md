# The section lock: a cited relative section the lock does not record

This runnable mini-repository demonstrates [§FS-section-lock](../../docs/functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went), the committed file that keeps a cited relative section at its text. It is for anyone citing a relative heading ([§FS-declarations.section.relative](../../docs/functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)) for the first time. The tree is the one in [`relative-sections/`](../relative-sections/), which shows how those headings resolve, except that its lock records one of the two citations. Open `repo/docs/use.md`, then `repo/.grund/sections/FS-login.lock`.

Run it from the repository root:

```bash
grund check examples/relative-sections-lock/repo
echo $?    # 1: FS-login.format.2 is cited but not in the lock
```

stdout carries the one error below; stderr carries only the caution that the config is on version 1 ([§FS-check.checks.config-version-1](../../docs/functional-spec/FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)):

```text
docs/use.md:4: error: FS-login.format.2 is cited but has no entry in .grund/sections/FS-login.lock; run grund fmt --write with no path at the workspace root, or at the project root outside a workspace
```

`docs/use.md` cites `FS-login.format.1` and `FS-login.format.2`. The lock holds one line per cited address with a fingerprint of the section's text, and here it records only `FS-login.format.1`, so `check` reports the `unrecorded-section-citation` error at the citation it cannot vouch for.

Run `grund fmt --write` on a copy of the repository: it adds the missing line, and `check` then passes, with that caution standing where `success` would. Only `fmt --write` run from the project root writes the lock, and the file is committed with the citation. Later, if `## format:` and `## parse:` swap their `### .1` blocks, `check` says where the cited text went and names every line that still cites the old address. Accepting that move takes a pointer in the old section or an edit of the citations. `grund config explain section-address-changed` prints those ways out and why the lock is not edited by hand; [`config-explain/`](../config-explain/README.md#step-5-what-clears-a-finding) walks through that answer.

## Try a declaration rename on a copy

This exercise enables link emission in the copy, records the baseline, then
renames the declaration and authored citation IDs. Leave generated URLs and lock
files to fmt ([§FS-init.2.3.4.9](../../docs/functional-spec/FS-init.md#2349-declaration-blast-radius),
[§FS-fmt.6.3](../../docs/functional-spec/FS-fmt.md#63-idempotency-and-re-derive)).
Start at the grund repository root with `grund` on PATH:

```bash
mkdir -p "$HOME/ag/tmp"
rename_copy=$(mktemp -d "$HOME/ag/tmp/grund-824-rename.XXXXXX")
cp -R examples/relative-sections-lock/repo/. "$rename_copy/"
(
  set -e
  cd "$rename_copy" || exit 1
  git init -q
  git config user.name 'Example'
  git config user.email 'example@example.invalid'
  # Enable generated links so both maintenance effects are visible.
  sed -i 's/enabled = false/enabled = true/' grund.toml
  grund fmt --write
  git add .
  git commit -qm 'Record baseline'
  grund refs FS-login --summary
  git mv docs/functional-spec/FS-login.md docs/functional-spec/FS-sign-in.md
  # Edit the declaration ID and authored citation IDs, leaving generated URLs stale.
  sed -i 's/# FS-login:/# FS-sign-in:/' docs/functional-spec/FS-sign-in.md
  sed -i 's/§FS-login/§FS-sign-in/g' docs/use.md
  grund fmt --write
  grund check
  grund fmt --check
  git diff -- docs/use.md
  git status --short
  git add .
  git commit -qm 'Rename login declaration and maintain generated links and locks'
)
```

The second write reports three changed files: the old lock, the new
`.grund/sections/FS-sign-in.lock`, and `docs/use.md`. The link destinations now
use `FS-sign-in.md`; section anchors remain `#1-username` and
`#2-password`. `check` and `fmt --check` exit 0 (the version-1 caution remains).
Inspect the copy, then remove it with `rm -rf -- "$rename_copy"`.

Run maintenance from the config/project root or workspace root so eligible
section locks are written ([§FS-section-lock.write.rules](../../docs/functional-spec/FS-section-lock.md#writerules-add-refresh-drop)).
A file move without an ID rename leaves the ID-keyed lock name alone. fmt does
not replace authored IDs, repair arbitrary URLs, or silently approve a protected
section move; fix citations or leave a pointer at the old address first
([§FS-section-lock.write.never-a-move](../../docs/functional-spec/FS-section-lock.md#writenever-a-move-fmt-never-approves-a-move)).

The trade-off: the lock is one more file to review. Its benefit is that a relative address cannot silently start answering with another section's words. Its cost is a line per cited relative address, and one gap: a section moved and edited in one change reads as an edit in place, so the refreshed line in the lock diff is the only trace ([§FS-section-lock.limits.edit-and-move](../../docs/functional-spec/FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change)).

# The section lock: a cited relative section the lock does not record

This runnable mini-repository demonstrates [§FS-section-lock](../../docs/functional-spec/FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went), the committed file that keeps a cited relative section at its text. It is for anyone citing a relative heading ([§FS-declarations.section.relative](../../docs/functional-spec/FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)) for the first time. The tree is the one in [`relative-sections/`](../relative-sections/), which shows how those headings resolve, except that its lock records one of the two citations. Open `repo/docs/use.md`, then `repo/.grund/sections/FS-login.lock`.

Run it from the repository root:

```bash
grund check examples/relative-sections-lock/repo
echo $?    # 1: FS-login.format.2 is cited but not in the lock
```

stdout carries the one error below; stderr carries only the caution that the config is on version 1 ([§FS-check.checks.config-version-1](../../docs/functional-spec/FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)):

```text
docs/use.md:4: error: FS-login.format.2 is cited but has no entry in .grund/sections/FS-login.lock; run grund fmt --write
```

`docs/use.md` cites `FS-login.format.1` and `FS-login.format.2`. The lock holds one line per cited address with a fingerprint of the section's text, and here it records only `FS-login.format.1`, so `check` reports the `unrecorded-section-citation` error at the citation it cannot vouch for.

Run `grund fmt --write` on a copy of the repository: it adds the missing line, and `check` then passes, with that caution standing where `success` would. Only `fmt --write` run from the project root writes the lock, and the file is committed with the citation. Later, if `## format:` and `## parse:` swap their `### .1` blocks, `check` says where the cited text went. Accepting that move takes a pointer in the old section or an edit of the citations.

The trade-off: the lock is one more file to review. Its benefit is that a relative address cannot silently start answering with another section's words. Its cost is a line per cited relative address, and one gap: a section moved and edited in one change reads as an edit in place, so the refreshed line in the lock diff is the only trace ([§FS-section-lock.limits.edit-and-move](../../docs/functional-spec/FS-section-lock.md#limitsedit-and-move-a-section-moved-and-edited-in-one-change)).

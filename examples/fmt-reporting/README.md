# Reading what `grund fmt` will do, and what it did

This runnable mini-repository shows how `grund fmt` reports a change: a preview
row names every kind of rewrite its line gets
([§FS-fmt.3.5.1](../../docs/functional-spec/FS-fmt.md#351-a-line-with-several-rewrites-carries-the-first-label)), the `--write` summary totals them
([§FS-fmt.3.7](../../docs/functional-spec/FS-fmt.md#37-the---write-report)), and a lock file a rename deletes or creates is said to be
deleted or created ([§FS-section-lock.write.report](../../docs/functional-spec/FS-section-lock.md#writereport-what-fmt-reports)). It is for anyone who
reads the preview to decide whether to write, or who describes a write from its
report.

The tree is caught halfway through a rename. `FS-login` became `FS-sign-in`: the
file was moved and the declaration and its authored citations were edited, but
`fmt --write` has not run yet. So the link in `docs/notes.md` still points at the
old file, and `.grund/sections/FS-login.lock` still records the cited section
under its old name. `docs/runbook.md` cites `FS-sign-in` without the marker.

Run it from the repository root:

```bash
grund fmt examples/fmt-reporting/repo --marker
echo $?    # 1: rewrites are pending
```

```text
docs/notes.md:1: markdown link
docs/runbook.md:1: bare → marker, markdown link
.grund/sections/FS-login.lock:1: section lock: drop FS-login.code.1; deletes the file
.grund/sections/FS-sign-in.lock:1: section lock: add FS-sign-in.code.1; creates the file
```

The runbook row leads with `bare → marker`, the first change its line gets, and
names `markdown link` after it, because `--marker` marks the citation and the
link pass then wraps what it marked
([§FS-fmt.6.5](../../docs/functional-spec/FS-fmt.md#65-interaction-with---marker)). The lock rows say that the run moves the
entry from one lock file to another: the old file is left with no entry, so the
write deletes it, and the new one does not exist yet, so the write creates it.

Write it on a copy:

```bash
mkdir -p "$HOME/ag/tmp"
copy=$(mktemp -d "$HOME/ag/tmp/grund-fmt-reporting.XXXXXX")
cp -R examples/fmt-reporting/repo/. "$copy/"
grund fmt "$copy" --marker --write
```

```text
rewrote 4 lines: 1 citation marked, 2 links written, 2 section-lock entries
  .grund/sections/FS-login.lock (1) deleted
  .grund/sections/FS-sign-in.lock (1) created
  docs/notes.md (1)
  docs/runbook.md (1)
```

The totals count occurrences, not lines. The runbook's citation is counted once
as marked and once as linked, and the re-pointed link in `docs/notes.md` is the
second link. A link that was already right would count nothing. Lock entries
are counted too, so `rewrote 0 lines` still means nothing was written.

Trade-off: the report says that a lock file was deleted and another created,
but it does not pair them as a move. Each fact is certain, while which
deletion goes with which creation is a guess. Commit both files with the rename.

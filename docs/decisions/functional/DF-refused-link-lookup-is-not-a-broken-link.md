# DF-refused-link-lookup-is-not-a-broken-link: a directory link whose target lookup is refused on the way is unopenable, not broken, and exits move with it

**Date:** 2026-11-10

## status: Accepted

## 1. Context

The walk took a directory link it could not open for a broken link whenever asking whether its target is a directory came back no, and that answer is no for a lookup refused by permission as much as for a target that is not there. With `docs/x -> ../.r/sub` and `.r` at mode `000`, the link took the broken-link tests, and the name test silences a broken link with no extension ([§FS-scan.scope.links.blind-spot](../../functional-spec/FS-scan.md#scopelinksblind-spot-a-broken-link-with-no-extension-is-a-declared-blind-spot)). `grund check` printed `success` and exited `0` over a tree it never read, while the same link named `docs/x.md`, or with `.r/sub` barred instead of `.r`, exited `2` at the link. `grund check docs/x` said the path does not exist. Reported as [issue #1325](https://github.com/agent-grounds/grund/issues/1325).

## 2. Decision

A link is broken only when the lookup of its target says nothing can stand there. A lookup refused for lack of permission, on the target or on any directory on the way to it, makes the link unopenable ([§FS-scan.scope.links.owed.refused](../../functional-spec/FS-scan.md#scopelinksowedrefused-a-target-the-lookup-is-refused-on-the-way-to-is-not-a-broken-link)), reported at the link as [§FS-scan.scope.links.unresolvable.unopenable](../../functional-spec/FS-scan.md#scopelinksunresolvableunopenable-a-directory-link-whose-target-cannot-be-opened-fails-at-the-link-once) says, whatever its name, and whether or not anything stands behind the refusal. A link given as the path is that link too.

### 2.1 Exits that move

On one tree: `docs/a.md` declaring and citing `FS-a`, `.r/sub/x.md` citing an unknown `FS-nope`, the link `docs/x -> ../.r/sub`, and `.r` at mode `000`, as a user that mode stops. "Before" is `grund 0.22.1` and main at `0cc6275878`.

| Tree | Run | Before | Now |
|---|---|---|---|
| As above | `grund check`, `grund check docs`, `grund check --full` | `success`, exit `0` | `error: docs/x: unreadable symlink: permission denied; make the path readable, or keep it out of the scan`, exit `2` |
| As above | `grund check --format json` | nothing, exit `0` | that failure as an `io` error at `docs/x`, exit `2` |
| As above | `grund check docs/x` | `error: path does not exist: docs/x`, exit `2` | the failure at `docs/x`, exit `2` |
| As above | `grund cover`, `grund list` | the answer, exit `0` | the answer and the same line, exit `2` |
| As above | `grund fmt --check` | the rewrites it would make, exit `1` | `error: nothing was rewritten: docs/x: unreadable symlink: …`, exit `2` |
| The link to `../.r/nothing`, `.r` at mode `000` | `grund check` | `success`, exit `0` | the same line, exit `2`: the run cannot tell this target from the one above |
| The link `docs/logo.png -> ../.r/logo.png`, `.r` at mode `000` | `grund check` | `success`, exit `0` | `error: docs/logo.png: unreadable symlink: permission denied; make the path readable, or keep it out of the scan`, exit `2`: a name the scan does not read is no broken link either |
| A version-1 project whose default root `src` is the link `src -> data/real`, `data` at mode `000` | `grund check` | `error: src: permission denied; make the path readable, or keep it out of the scan by removing it from the [scan] include or [[kinds]] row that lists it, …`, exit `2` | `error: src: unreadable symlink: permission denied; …`, the same remedy, exit `2`: the line a root link to a barred target already prints |
| A value home `values` holding the link `values/a.json -> ../data/x.txt`, `data` barred | `grund check`, `grund list` | `error: values/a.json: could not read value JSON source: permission denied; …`, exit `2` | unchanged, and still the one line: the walk now meets the link too, and the value pass's line stands in place of the walk's ([§FS-check.2.4.2](../../functional-spec/FS-check.md#242-one-failure-one-line), [§FS-values.5.3.1](../../functional-spec/FS-values.md#531-a-listed-candidate-whose-file-status-cannot-be-read)) |
| A workspace with `include_root = false` and the member `m`, the link `docs/x -> ../rr/sub` or `m/docs/mx -> ../../rr/sub`, `rr` at mode `000` and in no project | `grund check <link>`, its `--format json`, `grund list <link>`, `grund list --size=words <link>`, `grund cover <link>`, `grund fmt --check <link>`, `grund show --batch --glob 'FS-*' <link>` | `error: path does not exist: <link>`, exit `2` | the failure at the link, exit `2` |
| That workspace, `include_root` either way, with `m` at mode `000` and the link `docs/ml -> ../m/docs` | the same commands on `docs/ml` | `error: path does not exist: docs/ml`, exit `2`, but for `show --batch --glob` with `include_root = false`, which named the member's home `m/docs` | the failure at `docs/ml`, exit `2` |
| A workspace that loads no project, `members = []`, `optional_members = ["gone"]` and `include_root = false`, the link `docs/x -> ../rr/sub`, `rr` at mode `000` | the same commands on `docs/x` | `error: path does not exist: docs/x`, exit `2` | the failure at `docs/x`, exit `2` |

The three workspace rows keep their exit against `grund 0.22.1`, and only the line moves. Main between the releases, from #1348 until the fix for #1354, exited `0` with no line on most of them: `check`, `list` and `show --batch --glob` on the first and third rows, `check` on the second with `include_root = false`, and `cover` and `fmt --check` on the second either way. A workspace run walks each project whole, so it never asked the link, and a walk that started from the link dropped it at the member boundary. That exit never shipped, so it is no verdict this record corrects.

### 2.2 What does not move

- With `.r` readable, `docs/x` reads through the link and reports `docs/x/x.md:3: … unknown reference FS-nope`, exit `1`.
- With `.r/sub` barred, or the link named `docs/x.md`, the line at the link and exit `2` are unchanged.
- A link to `../.r/nothing` with `.r` readable is broken, and with no extension it is the declared blind spot: `success`, exit `0`.
- A link going outside the project with a refused directory on the way is pruned by the fence, exit `0` ([§FS-scan.scope.links.owed.fence](../../functional-spec/FS-scan.md#scopelinksowedfence-the-project-root-fence-is-judged-on-the-target-opened-or-not)).
- A link that loops is a different failure, outside this record. Its lookup fails with a loop, not a refusal, so it keeps its route: an extensionless cycle `docs/x -> y`, `docs/y -> x` is `success`, exit `0`, and `grund check docs/x` says the path does not exist.

## 3. The correction route

These exits flip a shipped verdict, which [§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered) allows only by a named route. [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids) is the route, and all five conditions hold.

1. *Prior prohibition.* The old exit `0` violated [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), a separately declared hard requirement that already applied when the old verdict shipped, as it has in every release whose walk follows directory links: a file the scan cannot read is reported at its path, the run exits `2`, and an incomplete report "never passes as if it were". The barred tree holds `FS-nope`, which the run never read and never mentioned, on the same build that reports `docs/x.md`.
2. *Accepted proof.* This record, which cites that section and proves the conflict. [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path)'s deprecation path does not fit, because nothing a user writes selects the old answer. [§REQ-backwards-compatibility.3](../../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)'s mechanical migration does not fit, because no input is at fault and no command can rewrite a directory's mode on anyone's behalf.
3. *Named release.* The release note below names these exits, who notices and what to do.
4. *Actionable findings.* What replaces the old verdict is one failure at the link, whose text names the action: `docs/x: unreadable symlink: permission denied; make the path readable, or keep it out of the scan`.
5. *No new licence.* This route cannot justify ordinary policy tightening, feature removal, or a prohibition invented by this change, and it is used for none of them. The failure and its line are [§FS-scan.scope.links.unresolvable.unopenable](../../functional-spec/FS-scan.md#scopelinksunresolvableunopenable-a-directory-link-whose-target-cannot-be-opened-fails-at-the-link-once)'s already. This change adds no check and no diagnostic code; it stops reading a refusal as an absence.

## release-note: Release note

- [§FS-scan.scope.links.owed.refused](../../functional-spec/FS-scan.md#scopelinksowedrefused-a-target-the-lookup-is-refused-on-the-way-to-is-not-a-broken-link), [§DF-refused-link-lookup-is-not-a-broken-link](DF-refused-link-lookup-is-not-a-broken-link.md#df-refused-link-lookup-is-not-a-broken-link-a-directory-link-whose-target-lookup-is-refused-on-the-way-is-unopenable-not-broken-and-exits-move-with-it), [§REQ-no-missed-citation.1](../../requirements/REQ-no-missed-citation.md#1-no-silent-skips), [§REQ-backwards-compatibility.5](../../requirements/REQ-backwards-compatibility.md#5-correcting-a-verdict-another-requirement-forbids): the verdict changes: **a directory link whose target sits behind a directory this user cannot search is reported at the link, whatever its name.** `grund check`, `cover`, `list` and `fmt --check` over such a link without an extension exited as if the tree were clean, and now exit `2` with `error: <link>: unreadable symlink: permission denied; …`; `grund check <link>` no longer says the path does not exist. A link whose name carries an extension the scan does not read, such as `docs/logo.png`, moves the same way, from `success` to exit `2`. In a workspace, a link given as the path to `check`, `list`, `cover`, `fmt --check` or `show --batch --glob` keeps its exit `2` and is reported at the link the same way, whether or not the root is a project of the run, where it said the path does not exist. A scan root that is such a link keeps its exit `2` and its remedy, and its line now reads `error: <root>: unreadable symlink: permission denied; …` where it read `error: <root>: permission denied; …`. Every finding that replaces the old verdict names its location, the link, and the action to take. Who notices: a run over a tree with such a link. What to do: make the directory searchable, or keep the link out of the scan with an ignore file or `[scan] exclude`.

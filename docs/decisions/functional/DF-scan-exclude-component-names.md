# DF-scan-exclude-component-names: a `[scan] exclude` entry containing `/` is a config error

**Status:** Accepted
**Date:** 2026-10-06

## 1. Context

`[scan] exclude` has been a list of directory **names** since `0.1.0`: each entry is compared against one directory's own name, at every depth ([§FS-config.3.5](../../functional-spec/FS-config.md#35-scan--what-gets-scanned)). The reader nonetheless accepted any string, so `exclude = ["docs/plans"]` loaded cleanly and excluded nothing — no directory's name contains `/`. `config validate` exited `0` on it, and `check` went on to scan `docs/plans/` and report what it found there. The sibling key `[fmt] exclude` takes gitignore-style globs against the config root ([§FS-config.3.10.1](../../functional-spec/FS-config.md#3101-entries-are-gitignore-style-globs)), so the path-shaped spelling is exactly what a reader of the other key would write.

That is a counterexample to the realized half of [§FS-config.requirements.5](../../functional-spec/FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly--realized-one-case-deferred) — an invalid value fails loudly, at its line — and not to its deferred wrong-scope case ([§DISC-core-concerns.8.5](../../discussions/proposals/2026-09-30-core-concerns.md#85-one-correction-to-the-plans-own-citation)). The published direction already chose the remedy: the path-shaped entry "is fixed separately, as an immediate located config error" ([§DISC-core-concerns.8.3](../../discussions/proposals/2026-09-30-core-concerns.md#83-the-two-exclude-keys-mean-opposite-things)).

## 2. Decision

### 2.1 The entry is refused at its line

The first `[scan] exclude` entry containing `/`, in list order, fails the config load with the located error of [§FS-config.4.3](../../functional-spec/FS-config.md#43-invalid-config-behavior), naming the entry and both repairs ([§FS-config.3.5.16](../../functional-spec/FS-config.md#3516-an-exclude-entry-containing--is-refused)). Only `/` is refused; no other spelling of an entry changes status, and a well-formed name keeps matching at every depth.

### 2.2 It is not a break: the entry was never a promise

[§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise) sets aside a construct that had no defined meaning and produced no output, provided the argument is made in a decision record. This is that argument.

- **No defined meaning.** Every published description of the key, from `0.1.0` on, calls an entry a directory name. A string containing `/` is not one, and no specification point, example or release note ever gave it a second reading.
- **No effect.** Both scanner walks compare one path component at a time, so the entry matched nothing on any tree. Removing it from a config leaves that config's scan byte-for-byte what it was.
- **The same shape as its precedent.** [§DF-number-only-citation-shorthand](DF-number-only-citation-shorthand.md#df-number-only-citation-shorthand-the-number-only-shorthand-is-authoring-sugar-and-a-persisted-one-is-a-check-error) turned a silently ignored construct into an error on the same ground: what the user meant was never done, and the silence was the defect.

What does move is a verdict: a config holding such an entry stopped loading, so `check` on it exits `2` where it exited `0` or `1`. That is disclosed by this record's release note rather than ramped.

### 2.3 No deprecation window and no migration command

A warning would leave the meaningless value in place for a window and fix nothing, and the repair is not mechanical: the two available fixes mean different things, so the user must choose. Removing the entry keeps the previous scan scope. Writing its last component (`plans`) excludes that name at every depth, which is a new policy, not a translation. A path-specific exclusion is not expressible in v1 at all; it is the v2 sources contract's, whose exclusions are globs.

### 2.4 What this leaves to the v2 migration

The v1 reader refuses these entries from this release on, but files that carry them exist. A future `config migrate` must be able to read such a historical v1 file without first passing the corrected validation, and report how each such entry's reach changes when it becomes the v2 glob it looks like ([§DISC-core-concerns.8.3](../../discussions/proposals/2026-09-30-core-concerns.md#83-the-two-exclude-keys-mean-opposite-things)). This change builds neither the v2 globs nor the migrator.

## 3. Rejected alternatives

### 3.1 Read the entry as a path or a glob

That would honor what the user probably meant, and change what gets scanned in every repository that carries such an entry, silently: files that have been checked would stop being checked. v1 keeps its meaning ([§FS-config.5.2](../../functional-spec/FS-config.md#52-every-older-version-keeps-its-meaning)); the path reading is v2's.

### 3.2 Strip everything before the last `/`

Mechanically turning `docs/plans` into `plans` silently widens the exclusion to every directory of that name — the exact undisclosed policy change the error exists to prevent.

## release-note: Release note

- [§FS-config.3.5.16](../../functional-spec/FS-config.md#3516-an-exclude-entry-containing--is-refused), [§FS-config.requirements.5](../../functional-spec/FS-config.md#requirements5-a-mistake-in-the-config-fails-loudly--realized-one-case-deferred), [§REQ-backwards-compatibility.4](../../requirements/REQ-backwards-compatibility.md#4-what-was-never-a-promise): **a `[scan] exclude` entry containing `/` is now a config error.** `exclude = ["docs/plans"]` was accepted and excluded nothing, because an entry is one directory name; it now fails the config load at its line — ``error: grund.toml:4: [scan] exclude entry `docs/plans` contains `/`, not a directory name; remove it to keep the scan unchanged, or use `plans` to exclude that name at any depth`` — with `check` exiting `2` and `config validate` exiting `1`, nothing on stdout. **Who this breaks:** a config holding such an entry, on every command that loads it. **The two repairs are not the same:** removing the entry keeps exactly the scan you had, while writing `plans` excludes every directory of that name at every depth. A path-specific exclusion has no v1 spelling. Well-formed names and `[fmt] exclude` globs are unchanged.

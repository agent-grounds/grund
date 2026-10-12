# FS-section-lock: a cited relative section keeps its text, or check says where the text went

A relative heading takes its address from its parent ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds)), so a section can change address without a byte of its heading changing. Swap the `### .1` blocks under `## format:` and `## parse:` and every citation of `FS-login.format.1` silently reads the other section's text, while `grund check` stays green. A complete-path heading cannot do this, because its address is written on it and moves with it. The section lock is what catches it: a committed file that records, for every relative address the tree cites, a fingerprint of the text it held, so `check` can tell an edit in place from text that left for another address ([§REQ-no-wrong-citation.1](../requirements/REQ-no-wrong-citation.md#1-no-wrong-resolution), [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration)). Decided in [§DF-relative-section-headings.lock](../decisions/functional/DF-relative-section-headings.md#lock-a-cited-relative-address-is-locked-to-its-text).

The lock is part of the tree. `check` reads it and no history, so its verdict stays a function of the tree ([§FS-check.3.6.4](FS-check.md#364-a-pure-function-of-the-tree)); only `grund fmt --write` writes it ([§FS-section-lock.write](FS-section-lock.md#write-who-writes-the-lock)). A repository that cites no relative heading has no lock and reads none, and complete-path headings are never enrolled.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, body, section, coordinate, relative heading, canonical path, section lock), [§FS-terms.terms.2](FS-terms.md#terms2-citations) (citation), and [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, config root, workspace, member, alias).

- **cited relative address** — A coordinate some citation in the project resolves to, whose section is a relative heading.
- **entry** — One line of a lock file: a cited relative address and its fingerprint.
- **fingerprint** — Sixteen lowercase hex digits that stand for the text a section held ([§FS-section-lock.holds.fingerprint](FS-section-lock.md#holdsfingerprint-the-fingerprint)).
- **recorded address** — The coordinate an entry names.

## holds: What the lock holds

### holds.entry: One line per cited relative address

A lock file holds one entry per cited relative address of one declaration, `<coordinate> <fingerprint>`, one space between them, one entry per line, sorted bytewise by coordinate, each line ending in LF:

```text
FS-login.format.1 8b1a5e0c2f9d7a64
FS-login.parse.user 0d3c9e51b7f2a816
```

Nothing else is recorded: no title, no line number, no history. A coordinate of a complete-path heading is not an entry, cited or not, and neither is an address nothing cites. A relative heading in a stub's target is entered like one in a scanned file, wherever the target lies: one outside `[scan] include` still lends the stub its sections ([§FS-declarations.stubs.target.5](FS-declarations.md#stubstarget5-what-a-target-outside-the-include-gives-and-what-stays-unjudged)), so a citation of one is a cited relative address.

### holds.fingerprint: The fingerprint

The fingerprint is the first 16 hex digits, lowercase, of the SHA-256 of the section's text: what `grund <coordinate> --full` prints in text format below the heading line, which is the section and everything nested in it. Before hashing, line endings are made LF, trailing spaces and tabs are removed from each line, leading and trailing empty lines are dropped, and the lines are joined with LF with no final LF; a section with no text hashes the empty string. Because the text is `show`'s, cross-reference links are already flattened ([§FS-show.3.2](FS-show.md#32-cross-reference-links-are-flattened-in-text-and-json)): `fmt` re-deriving an anchor, a wrapped against a bare citation, a CRLF checkout and trailing whitespace all give one fingerprint, so two installs agree on it ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). The digest is grund's one SHA-256, the algorithm [§FS-remote-projects.bytes.digest](FS-remote-projects.md#bytesdigest-the-digest-is-over-exact-bytes) names.

### holds.repeated: A text the declaration repeats

When another section of the same declaration has the same fingerprint, the two cannot be told apart, so the entry is written with `-` in place of the fingerprint (`FS-login.format.1 -`). Such an entry guards nothing — it is never a [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) finding — until the texts differ, when `fmt --write` records the fingerprint.

## file: Where the lock lives

### file.shard: One file per cited declaration

The entries for the cited relative addresses of declaration `<ID>` live in `.grund/sections/<ID>.lock` under the config root of the project whose files hold the citations. Two changes that cite sections of different declarations never touch the same file, and two changes that cite different sections of one declaration touch different lines of a sorted file, so the lock is no file every change edits. The file is named by the ID, not the declaring file, so moving or renaming a spec file leaves its lock alone. A file with no entry does not exist, and neither does an empty `.grund/sections/`. The scan never enters `.grund/` ([§FS-scan](FS-scan.md#fs-scan-what-a-path-argument-means-which-projects-a-run-loads-and-which-files-it-reads)), so a lock file is never read as a source.

### file.members: Citations into another project

Each project keeps the entries for the citations its own files hold. A citation into another workspace member or a remote, `§<alias>/<ID>.<path>`, is entered in `.grund/sections/<alias>/<ID>.lock` of the citing project, so a member's own run and a run from the workspace root agree on what it cites and where that is recorded.

Such an entry's coordinate is the one the citation names, `<ID>` and its path as the target project spells them, without the alias, and it is judged against the target's sections and texts like any other entry. A finding about it names the coordinate with the alias, `api/FS-login.format.1`, and [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) sits at the target's heading. A run that does not load the alias — a member's own run, where a sibling's alias is unknown ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)) — leaves `.grund/sections/<alias>/` alone: it reports nothing from it and writes nothing to it.

## write: Who writes the lock

### write.rules: Add, refresh, drop

Only `grund fmt --write` writes lock files, and only when its scope is a project's config root or the workspace root; `fmt` narrowed to a file or directory leaves every lock alone, and neither `check`, the editor, nor any other command writes one. A run brings each lock file to what the tree needs:

- **add** an entry for each cited relative address that has none;
- **refresh** an entry whose section was edited in place — its recorded fingerprint is found at no address of the declaration, so its text changed where it stands — to the section's current fingerprint;
- **drop** an entry whose address nothing cites any more, or whose address is no longer held by a relative heading.

An entry that already matches is left alone, so a second run writes nothing. A stale entry is never a `check` finding: it is something `fmt --write` drops.

### write.never-a-move: `fmt` never approves a move

`fmt` never refreshes an entry whose recorded fingerprint now sits at another address of the declaration, because that is text that moved, and recording the new text at the old address would turn the report of [§FS-section-lock.checks.section-address-changed](FS-section-lock.md#checkssection-address-changed-a-cited-sections-text-sits-at-another-address) into silence. The one exception is a move already acknowledged by a pointer ([§FS-section-lock.move.pointer](FS-section-lock.md#movepointer-leave-a-pointer)), whose pointer text it records.

### write.report: What `fmt` reports

The dry run, `fmt --check`, lists one row per entry `--write` would add, refresh or drop, in the [§FS-fmt.3.5](FS-fmt.md#35-the-dry-run-report) shape, ordered by path, then line, then the row's text:

```text
.grund/sections/FS-login.lock:1: section lock: add FS-login.format.1
.grund/sections/FS-login.lock:2: section lock: refresh FS-login.parse.1
.grund/sections/FS-login.lock:3: section lock: drop FS-login.parse.2
```

An add is reported at the line the entry will occupy after the write, a refresh or drop at the line it occupies now. A pending row makes `fmt --check` exit `1` as any pending rewrite does ([§FS-fmt.3](FS-fmt.md#3-outputs)). `--write` lists each lock file it changed in its report like any other file ([§FS-fmt.3.7](FS-fmt.md#37-the---write-report)), its count being the number of rows the dry run lists for it, so a move that drops one entry and adds another is `(2)`. Like the cross-reference pass, the lock pass needs the whole declaration set, so an unreadable path refuses it ([§FS-fmt.3.2](FS-fmt.md#32-a-run-that-needs-the-whole-declaration-set-rewrites-nothing)).

### write.rebuild: A lock grund cannot read is rebuilt

A lock line that is not `<coordinate> <fingerprint>` — a hand edit, or the conflict markers of a merge — is a `check` error ([§FS-section-lock.checks.invalid-section-lock](FS-section-lock.md#checksinvalid-section-lock-a-lock-line-grund-cannot-read)), and `fmt --check` lists it as `section lock: rebuild` at its line. `fmt --write` rebuilds the file: it keeps every entry it can read, from either side of a conflict, takes for a coordinate entered twice the entry whose fingerprint the section holds now, or else the first, and then adds, refreshes and drops as above. Two changes that each cite a new section of one declaration therefore merge with one `fmt --write`.

## move: Moving a cited section on purpose

A cited section that moves changes its address, and [§REQ-spec-section-names.permanence](../requirements/REQ-spec-section-names.md#permanence-a-name-is-permanent) asks that the old address not silently become something else. There are two ways to move one, and both pass `check`.

### move.citations: Update every citation

Change every citation of the old address to the new one and run `fmt --write`. The old entry is no longer cited, so it is dropped; the new address is added. Until `fmt --write` runs, `check` reports each citation of the new address as [§FS-section-lock.checks.unrecorded-section-citation](FS-section-lock.md#checksunrecorded-section-citation-a-cited-relative-address-the-lock-does-not-hold).

### move.pointer: Leave a pointer

Keep the old address cited and leave a pointer there: a section at the recorded address whose text cites the address where the recorded text now sits (`See <§>FS-login.parse.2.`). That acknowledges the move: `check` does not report it, and `fmt --write` records the pointer's text as the entry's new fingerprint. Retiring the pointer once nothing cites the old address is a change of its own, and it drops the entry.

### move.copy: Copy, then edit the original

Copying a cited section to a new address and then editing the original is the one case the lock reports though nothing moved. After the edit the recorded text is found only at the copy, which is exactly what a move looks like. The way out is the one the finding names: edit the copy, so its text is no longer the recorded one and the original reads as edited in place, or cite the copy from the original, which acknowledges it as a pointer does.

## limits: What the lock does not catch

### limits.edit-and-move: A section moved and edited in one change

When a cited section moves and its text is edited in the same change, its recorded fingerprint is found nowhere, so the lock reads it as edited in place: `check` passes, and `fmt --write` refreshes the entry. Take `FS-login` with `## format:` holding `### .1: Username` and `## parse:` holding `### .1: Tokens`, `FS-login.format.1` cited and locked. Swap the two `### .1` blocks and edit both texts: `grund check` succeeds, `grund fmt --check` lists `section lock: refresh FS-login.format.1`, and after `fmt --write` the citation of `FS-login.format.1` reads the edited Tokens text. The only trace is that refreshed line in the lock diff `fmt --write` produces. This limit is accepted: a second key beside the fingerprint, such as the title, would report a section retitled and edited in place whenever another section carries its old title.

### limits.identical: Identical texts

Sections with byte-identical text cannot be told apart, so an entry whose text the declaration repeats guards nothing until they differ ([§FS-section-lock.holds.repeated](FS-section-lock.md#holdsrepeated-a-text-the-declaration-repeats)).

### limits.across: Across declarations

The lock compares a section only with the other sections of its own declaration. A section moved into another declaration is a risk complete-path headings already carry, and it is not covered here.

### limits.gate: A lock nobody refreshes

An edit that `fmt --write` never recorded leaves the old fingerprint in place, which weakens the guard for that entry until the next `fmt --write`. A repository that runs `fmt --check` in its gate never carries one.

## checks: Checks

Each is a located `grund check` error with its code in [§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog), selected by `--only` and `--ignore` like any other, and rendered in text, JSON (`"sites":null`) and the LSP as [§FS-check.2.1](FS-check.md#21-report-format) says. None fires in a repository with no lock file and no citation of a relative address. A message names the lock file as the report names every path, so from the workspace root a member's lock is `packages/api/.grund/sections/FS-login.lock`.

### checks.section-address-changed: A cited section's text sits at another address

When an entry's recorded address is cited and held by a relative heading, the section there no longer has the recorded fingerprint, and another address of the same declaration does, the text that was cited has moved and the address now holds something else. The error is located at the heading that now holds the recorded address:

```text
FS-login.format.1 is cited, and what .grund/sections/FS-login.lock recorded for it now sits at FS-login.parse.1; move it back, cite FS-login.parse.1 instead, or leave a pointer to it at FS-login.format.1; if FS-login.parse.1 is a copy, edit the copy or cite it from FS-login.format.1
```

The three fixes are the move undone, [§FS-section-lock.move.citations](FS-section-lock.md#movecitations-update-every-citation) and [§FS-section-lock.move.pointer](FS-section-lock.md#movepointer-leave-a-pointer); the last clause is the way out of [§FS-section-lock.move.copy](FS-section-lock.md#movecopy-copy-then-edit-the-original). When the fingerprint sits at more than one other address, the first in document order is named. It is reported for a swap of two sections with or without the same title, for a parent rename after which a sibling takes the old address, and for a section moved under another parent whose old address another section took. It is not reported when the section at the recorded address cites the address it names, nor for an entry written `-`.

### checks.unrecorded-section-citation: A cited relative address the lock does not hold

A citation that resolves to a relative heading's address with no entry in the lock is an error at the citation's site:

```text
FS-login.format.1 is cited but has no entry in .grund/sections/FS-login.lock; run grund fmt --write
```

Without an entry the address is unguarded, so the citation is not taken as checked until `fmt --write` records it. A citation that resolves to no section at all is [§FS-check.3.2](FS-check.md#32-missing-section) and never this.

### checks.invalid-section-lock: A lock line grund cannot read

A line of a lock file that is not an entry, as [§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-cited-relative-address) defines one, is an error at that line:

```text
section lock line is not <coordinate> <fingerprint>; run grund fmt --write to rebuild the file
```

The entries `check` can read are still judged. `fmt --write` rebuilds the file ([§FS-section-lock.write.rebuild](FS-section-lock.md#writerebuild-a-lock-grund-cannot-read-is-rebuilt)).

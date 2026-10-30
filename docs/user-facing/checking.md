# Checking a repository

From grund 0.19.0, an excluded or ignored stub target produces `broken-stub`
with error severity and exit 1 ([§FS-declarations.stubs.target.3](../functional-spec/FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)). For example, a
stub to `notes/gen/a.md` under `exclude = ["gen"]` warned in 0.18.0; now it
errors and show refuses. Move the target or change the rule named in the error.
The old `excluded-stub-target` code is spent: `--ignore excluded-stub-target`
leaves the new error visible; `--only excluded-stub-target` selects no findings.
Use `broken-stub` for the replacement ([§FS-declarations.checks.excluded-stub-target](../functional-spec/FS-declarations.md#checksexcluded-stub-target-stub-to-a-target-the-walk-excludes-broken-from-grund-0190)).

`grund check` is the gate: it scans the tree, resolves every citation against every
declaration, and exits non-zero when one dangles or breaks a rule the repository
declared ([§FS-check](../functional-spec/FS-check.md#fs-check-grund-validates-every-citation-in-a-repo)). Run it in pre-commit and in CI.

## When only an index cites a declaration

A kind's index lists declarations for navigation. Its entry is a citation that
`refs` lists, but it does not establish use, so `check` still warns
([§FS-check.4.1.2](../functional-spec/FS-check.md#412-an-index-entry-does-not-count)).
The warning locates the discounted entry; text refs labels it, including within
a file summary ([§FS-refs.3.1.1](../functional-spec/FS-refs.md#311-label-the-excluded-index-sites),
[§FS-refs.3.3.1](../functional-spec/FS-refs.md#331-identify-the-excluded-sites-within-a-summary)).

For example, from the [authored-citations mini-repository](../../examples/authored-citations/README.md):

```text
$ grund check examples/authored-citations/repo --only unused
docs/adr/consider-memory.md:1: warning: declared but never cited: ADR-consider-memory (its index entry at docs/adr/README.md:5 does not count as a use); cite it
$ grund refs ADR-consider-memory examples/authored-citations/repo
docs/adr/README.md:5: §ADR-consider-memory (index entry)
$ grund refs ADR-consider-memory examples/authored-citations/repo --summary
docs/adr/README.md: 1 (line 5) (index entries at line 5)
```

These commands agree: the entry exists, and the declaration has no genuine use
yet. Decide whether the declaration is still useful, or connect it to the
behavior that actually depends on it. Add a citation when that dependency exists;
an unused warning alone is no reason to invent one. Ordinary use rows stay
unannotated; when a summary mixes entry and use sites, its annotation identifies
only the excluded lines (and columns when they share a line).

When the use lives in a repository this check cannot read — a closed downstream
that cites your points through a workspace alias — commit a generated page of its
citations instead, as [Citers in a repository you cannot read](repository-options.md#citers-in-a-repository-you-cannot-read)
describes, so your own gate guards them. Never write such a page by hand to quiet the warning.

## A misplaced declaration beneath another project's config

A nested `grund.toml` does not make its directory a workspace member. Until an
enclosing `[workspace] members` entry lists it, that directory is ordinary tree
to the enclosing scan and its declarations follow the enclosing kind homes.
When one is misplaced, the error identifies the unread nested config and the
enclosing config/project whose homes apply, in text and JSON
([§FS-declarations.checks.misplaced-declaration.unlisted-project](../functional-spec/FS-declarations.md#checksmisplaced-declarationunlisted-project-explain-the-enclosing-homes-beneath-an-unlisted-project)).

For example, `packages/notify/requirements.md` can be misplaced when the enclosing
project's home is its own root-level `requirements.md`. If notify belongs to the
workspace, add `"packages/notify"` to `[workspace] members` in the enclosing
config. Membership alone fixes which homes apply; promoting `## FS-email` to
`# FS-email` does not change ownership. If notify should stay outside the scan,
use `[schema.sources] exclude = ["packages/notify/"]` in v2. In v1 use
`[scan] exclude = ["notify"]`, which excludes every directory named `notify` at
every depth. If configured source includes or kind homes start a scan at or
beneath that directory, remove those roots too: exclusions only prune
descendants. The diagnostic names report-relative locations, but suggested edit
paths and v2 exclusion patterns are relative to the enclosing config root.

Run the [workspace repair walkthrough](../../examples/workspace/README.md#repairing-an-unlisted-project)
to see the failing check and both repairs. A plain nested config with no misplaced
declaration adds no standalone finding.

## What it enforces

Renumber the heading `### 3.2 Missing section` in [`FS-check.md`](../functional-spec/FS-check.md) to `3.99` and `grund check` flags every site that leaned on it — code, tests, decision docs and the README alike, in one resolver:

```text
$ grund check
README.md:34: error: missing section FS-check.3.2
crates/grund-cli/tests/index_entry_round_trip.rs:232: error: missing section FS-check.3.2
crates/grund-core/src/checker/index.rs:247: error: missing section FS-check.3.2
crates/grund-core/src/checker/index.rs:361: error: missing section FS-check.3.2
crates/grund-core/src/checker/judge.rs:69: error: missing section FS-check.3.2
crates/grund-core/src/checker/references.rs:2: error: missing section FS-check.3.2
…
docs/requirements/REQ-no-wrong-citation.md:7: error: missing section FS-check.3.2
```

`grund check <path>` reports on `<path>` while resolving citations against the whole project; with no path it scans the canonical layout (`requirements.md`, `docs/`, `e2e/`, `src/`). In the scanned tree it enforces:

1. Every cited ID resolves to a declaration. *(dangling references)*
2. Every section coordinate (`.3.1`) resolves to a heading inside the declaration. *(missing sections)*
3. No ID is declared in two places. *(duplicates)*
4. Every deeper ATX heading inside a scanned Markdown declaration body is another declaration or carries a numeric or enabled named section coordinate. Fences, file titles, body-closing headings, source doc-comments, setext text, and bold labels are exempt. *(unmarked headings)*
5. Every stub heading `# <ID>: [<text>](<path>)` points at a file containing the inline declaration. *(broken stubs)*
6. The `AGENTS.md` / `CLAUDE.md` entry-point block is up to date. *(stale init)* `check` holds the whole block to what `grund init` would write there, so a config edit, a migration or a renamed kind reports the part it changed until `grund init` runs ([§FS-check.3.5.4](../functional-spec/FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)). The two workspace sections, `Project namespaces` and `Workspace members`, are judged only by a run at the root of a full workspace checkout: a run inside a member, at a nested group's root, or in a checkout without an optional member leaves them out ([§FS-check.3.5.5](../functional-spec/FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)). In a version-2 config that includes each scoped `AGENTS.md` the root block lists: one that is missing, stale, or no longer placed by the config is reported at that file ([§FS-check.3.5](../functional-spec/FS-check.md#35-invalid-agent-entrypoint-init-block)). A scoped `AGENTS.md` is guidance about its directory, never one of its units, so it is not held to that directory's citation rules.
7. Declared-but-uncited IDs are flagged. *(unused — warning, not error; a configured `E2E` kind's cases are exempt)*
8. *(opt-in)* With `require_grounding = true`: every source file — or every file of one configured place, down to every `##` section or doc-comment block of it — carries at least one citation. *(ungrounded source file)*
9. *(workspace)* Alias-qualified citations resolve across configured sub-projects. *(cross-project references — see [§FS-workspace](../functional-spec/FS-workspace.md#fs-workspace-grund-validates-cross-project-citations-in-a-workspace))*
10. No declaration is left in the middle of a merge. *(conflict markers)* Each block of the lines git writes around a conflict, `<<<<<<<`, `|||||||`, `=======` and `>>>>>>>` at column 0, is one error at its first line, naming every declaration it overlaps ([§FS-declarations.checks.conflict-marker](../functional-spec/FS-declarations.md#checksconflict-marker-unresolved-merge-conflict)).

A conflict block counts anywhere in a Markdown file that declares an ID, inside a fence too, and in a source file inside a declaration's doc-comment, including the lines right after a `///` or `#` run the conflict cut short ([§FS-declarations.body.conflict.where](../functional-spec/FS-declarations.md#bodyconflictwhere-where-blocks-are-looked-for)):

```text
docs/login.md:7: error: unresolved merge conflict in FS-login, lines 7–11: resolve it by hand
```

Resolve it by hand: grund cannot tell which side is the declaration's text, so the message names neither. While a block remains, `grund fmt` rewrites nothing, because it would otherwise fingerprint one side into the section lock; `--ignore conflict-marker` hides the finding and does not change that ([§FS-fmt.7.4](../functional-spec/FS-fmt.md#74-no-write-without-a-complete-model)). A `>>>>>>> ` line is also how Markdown would spell a quote seven levels deep, and outside a conflict it reads as a lone closer, so write such a quote `> > > > > > >`. To show a marker line as text, in a fence for example, indent it by one space: ` <<<<<<< HEAD` is not a marker line.

Text inside a source file's string literal is data, so a qualified citation there is not a cross-project citation and `grund fmt` never rewrites it; a marked unqualified citation there is still checked. A literal that spans lines — a Rust raw string, a Java text block, a JavaScript template, a shell heredoc and the other forms in [§FS-languages.multi-line-string.forms](../functional-spec/FS-languages.md#multi-line-stringforms-the-recognized-forms) — counts as a string on every line it spans. The forms grund does not carry, such as Ruby heredocs and shell's ordinary quoted strings, are listed in [§FS-languages.multi-line-string.unrecognized](../functional-spec/FS-languages.md#multi-line-stringunrecognized-the-forms-that-are-not-carried); on their later lines a qualified citation is still read as one.

That is the core. Every rule, including citation directions, chapter rules, fields and
values, with what it means underneath and the finding code it reports, is in
[Schema and links](structure.md#schema-and-links-the-rules-a-repository-is-held-to).
To ask what one finding code means, how severe it is and what turns it on, run
`grund config explain <code>`: it answers from the grund you run, in your
config's version ([config explain](explain.md)).

`grund check` reads the configured source scope: version 1 uses `[scan] include`; version 2 uses configured places, including the `code` complement, with `[schema.sources] exclude` pruning files ([sources guide](sources.md)). Citations outside that scope are invisible to the ordinary run. `grund check --full` ([§FS-check.1.3](../functional-spec/FS-check.md#13-the-full-tree-scope---full)) widens the walk past configured scope, including places with `scan = false`, and reports unresolved references and section headings outside declaration bodies there. Source exclusions still apply. The wider walk does not apply the ordinary convention rules outside scope; it is additive, so it can only turn a green run red.

`grund` doesn't invent citations — that's the contributor's call. What `grund` does is make sure the ones you wrote *resolve*. With `require_grounding = true` — in `[reference]` for every place at once, or on one `[[kinds]]` row for that place alone, at a `grounding_level` from the whole file down to every `##` of it ([§FS-config.3.4.8](../functional-spec/FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) — it also fails what carries no resolving citation; the stronger file-level "implementation changed with its spec and test, or bounded waivers" recipe is tracked separately in [§RM-cochange-gate](../roadmap.md#rm-cochange-gate-an-opt-in-commit-msg--ci-recipe-for-spec-and-test-edits).

## Fixing every wide line in a note

A column finding lists every line over its selected limit, in source order, before its remedy
([§FS-inline-citation-style.4.1.1.1](../functional-spec/FS-inline-citation-style.md#4111-every-over-limit-line)). For example:

```text
src/ingest/parse.py:5: error: inline note is 105 columns, over the 100-column maximum: lines 5-7 cite §AR-ingest.1.1; over-limit lines: line 5: 103 columns, line 6: 105 columns; shorten or rewrap its widest line
```

Wrap both line 5 and line 6 before checking again. Shortening only line 6 leaves
`over-limit lines: line 5: 103 columns`; shortening both to at most 100 characters
clears this column finding. Line 7 belongs to the note's span but needs no width
edit. Width includes indentation and comment delimiters and counts Unicode
characters, so an accented letter counts once. JSON messages and v2 softer
column-budget findings carry the same list, measured against their own limit.

## Finding a stale workspace entry

A stale workspace section shows the first differing nonblank body line as
`expected …; found …` ([§FS-check.3.5.3.1](../functional-spec/FS-check.md#3531-first-differing-workspace-entry)).
For example, the root block still carries the *(not yet initialized)* mark an
earlier grund wrote for billing, where the current render links its directory:

```text
AGENTS.md:3: error: repo maintenance: stale grund init block: the workspace members section differs from the workspace (expected - [`billing`](packages/billing/); found - [`billing`](packages/billing/) *(not yet initialized)*) — run `grund init` to refresh (does not affect citation validity)
```

`expected` is what the current render would write; `found` is the entry on disk.
`<missing>` means that side has no corresponding entry. The comparison follows
the entries' order, so a reordered list shows the first unequal pair. If the
entries agree but section text or order differs, the detail says that instead.
Run `grund init` in the directory the finding names; with no directory hint,
run it in the project where you ran `check`. A member's finding from the root
names that member's directory, such as `packages/billing`.

Try the [workspace walkthrough](../../examples/workspace/README.md#finding-and-refreshing-a-stale-entry)
to make one entry stale and refresh it.

## The report

With `[id] named_sections = true`, a marked citation of a name under a number
is checked as a whole. From grund 0.19.0, an absent section or declaration is
an ordinary error; the warning and its temporary release clause from 0.18.0
are gone ([§FS-citations.checks.mixed-path](../functional-spec/FS-citations.md#checksmixed-path-a-citation-of-a-name-under-a-number-is-checked-and-warns-until-0190)). For example, if
`FS-login` exists but has no `1.goals` section:

```text
docs/use.md:5: error: section not found: FS-login.1.goals; write <§> before it to show the shape without citing it
```

The finding goes to stdout and makes the check exit `1`; JSON reports
`missing-section` with severity `error`. Repair the heading or citation, or
escape an illustrative shape with `<§>`.

When the section was renamed after its section lock recorded it, the finding
also names where the recorded text sits now
([§FS-section-lock.checks.vanished](../functional-spec/FS-section-lock.md#checksvanished-a-cited-address-that-is-gone-and-where-its-text-went)):

```text
src/loans.py:5: error: section not found: SPEC-loans.requirements.borrow-a-book; write <§> before it to show the shape without citing it; what .grund/sections/SPEC-loans.lock recorded for it now sits at SPEC-loans.requirements.check-out-a-book
```

That citation is live, so do not escape it. Cite the address the finding names,
or rename the heading back, then run `grund fmt --write` and commit the lock
([§FS-section-lock.move.citations](../functional-spec/FS-section-lock.md#movecitations-update-every-citation)).
A write before the citations follow drops the old entry, and from then on the
finding no longer names the new address.

When the section resolves but its
lock entry is missing
([§FS-section-lock.checks.unrecorded-section-citation](../functional-spec/FS-section-lock.md#checksunrecorded-section-citation-a-locked-address-the-lock-does-not-hold)),
run `grund fmt --write` with no path at the workspace root, or at the project
root outside a workspace, and commit the lock: `fmt` given a file, or a
directory that is no project's root, writes no lock, and a member's own run
writes none for a citation into a sibling
([§FS-section-lock.checks.remedy](../functional-spec/FS-section-lock.md#checksremedy-the-run-that-writes-the-lock)).

In a kind's index, an entry is a link that goes somewhere: the one `fmt`
writes, an inline link whose text holds the citation, or a reference whose key a
definition matches. A citation that is an image's whole description,
`- ![§ADR-use-redis](cache.png)`, is no link to the declaration, so it is no
entry either. grund 0.19.0 still took it as one and warned; grund 0.20.0 retired
that credit, so an ID the image alone listed is reported where it is declared
([§FS-citations.checks.image-index-entry](../functional-spec/FS-citations.md#checksimage-index-entry-a-citation-only-image-is-an-index-entry-for-one-more-release)):

```text
docs/adr/use-redis.md:1: error: ADR-use-redis appears in docs/adr/README.md but not as an entry: an entry is a `§`-marked Markdown link to the declaration — became an error in grund 0.13.0
```

Write `[§ADR-use-redis](use-redis.md#adr-use-redis-use-redis-for-the-cache)`
beside the image to clear it; whether the index keeps the image as well is yours
to choose. `--ignore image-index-entry` no longer hides anything: the code is
spent, and the error above is `missing-index-entry`.

A passing text check prints `success` and exits 0. Findings go to stdout as
`<path>:<line>: error: <message>`, `warning:`, or opt-in `suggestion:` lines:
errors come first, then warnings and suggestions, while the location remains the
jump-friendly prefix. `grund check | …` / `grund check --format=json | jq` work
without redirection (the linter convention — only run-level `error:` lines, like
an unreadable path, go to stderr). JSON output remains diagnostics-only and in
global location order, so a clean `grund check --format=json` prints nothing.

Some warnings are about what the run did not read rather than what it found.
A file the configuration names as a scan root — a `[[kinds]]` `file` home, a
version-2 `files` entry, or an `[scan] include` entry that is a file — is read
only if `[scan] extensions` lists its extension and its name does not begin with
`.`. When either rule keeps it out, the file stays unread and `check` says so at
the line of the key that named it, keeping the exit code at 0 and withholding
`success`:

```text
grund.toml:10: warning: [[kinds]] TXT home `notes.txt` is not read — `txt` is not in [scan] extensions, so nothing in it is checked. Add "txt" to [scan] extensions, or give the file an extension the list holds.
grund.toml:10: warning: [[kinds]] NOTE home `.notes.md` is not read — its name begins with `.`, and grund reads no hidden file, so nothing in it is checked. Rename the file to a name that is not hidden.
```

List the extension, or give the file one the list holds; grund reads no hidden
file, so a dotfile is renamed rather than listed, and a file refused by both
rules gets one warning naming both. A row that names the file only as a place in
the Project map says so with `scan = false`. `--ignore
unread-scan-root` drops the warning
([§FS-scan.checks.unread-scan-root](../functional-spec/FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read)).

`grund check --watch` checks immediately and updates the terminal after saves;
Ctrl-C restores an owned alternate screen and returns the last completed status.
Redirected output appends ordinary reports, and clean JSON runs have no visible
boundary. See the [watch guide](watch.md)
([§FS-check.6](../functional-spec/FS-check.md#6-watch-mode---watch)).
Exact-text consumers migrating from the former unmarked, global-location report
should use `--format=json`, whose bytes, object shape, and order are unchanged.

From this checkout, the initial report is:

```text
$ cargo run --quiet -- check --watch --format=text
success
```

Watch stays resident until Ctrl-C; this capture has empty stderr and exits 0 after that initial report.

## Narrowing the report

When you need a narrower answer without weakening the repository's default check,
select its stable finding codes: `grund check --ignore agents-init` asks whether
the remaining content report has errors, while repeatable `--only <code>` and
`--ignore <code>` compose as sets, and `--only-rule` narrows the same report by
rule authority instead — only what a `--rule` trial sentence authored, which is
what lets you try a sentence out for the cost of its own findings
([§FS-check.1](../functional-spec/FS-check.md#1-inputs), [§FS-rules.8](../functional-spec/FS-rules.md#8-command-surfaces)).
A check narrowed to a code rules produce still says when a rule behind it was
skipped as invalid ([§FS-rules.7.6](../functional-spec/FS-rules.md#76-selection-json-ordering-and-exits)),
or when one applies to nothing ([§FS-check.2.2.1.4](../functional-spec/FS-check.md#2214-a-rules-obligation)).
A selected code whose check this run leaves off judges nothing, so instead of a
bare `success` the run says so and names what turns it on, spelled in the config
version it read:

```text
warning: --only oversized-lead judged nothing — the check is off until [schema.leads.words] warn = <N> is set
```

The line is the `selected-check-off` warning: it withholds `success`, never
moves the exit code, and only `--ignore selected-check-off` removes it
([§FS-check.1.4.1](../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)).
Selection happens only after the complete scan, and operational failures remain
visible; selected `success` describes only that view, not an all-findings
repository verdict ([§FS-check.2](../functional-spec/FS-check.md#2-outputs)).

### Imports are always reported

A config that declares `[workspace.imports]` reads each neighbour live from the tree at its
path, which no commit pins, and `check` says so on every run: one `import-live` warning per
import, at its table line ([§FS-imports.checks.import-live](../functional-spec/FS-imports.md#checksimport-live-an-import-is-read-live-unpinned)).

```text
grund.toml:4: warning: import `native-image` is read live from `../graal/substratevm`, unpinned
```

A neighbour whose path names no directory is one `import-missing` error there instead, the
run exits `1`, and every citation into it is left unchecked rather than passed
([§FS-imports.checks.import-missing](../functional-spec/FS-imports.md#checksimport-missing-an-import-names-no-directory)). Neither `--only` nor `--ignore` hides either line,
as neither hides an `io` finding, so a narrowed run still says what it read
([§FS-imports.checks](../functional-spec/FS-imports.md#checks-checks)).

## What it does not check

`grund` does **not** check Markdown links, URLs, spelling, or grammar. Use [`lychee`](https://github.com/lycheeverse/lychee), `vale`, etc. for those.

What a repository can add to the check:

- [Workspaces and sub-projects](repository-options.md#workspaces-and-sub-projects)
- [Keep shared values consistent](repository-options.md#keep-shared-values-consistent)
- [Cite external facts offline](repository-options.md#cite-external-facts-without-making-checks-depend-on-the-network)

## In pre-commit and CI

This repo ships a ready-to-install [.pre-commit-config.yaml](../../.pre-commit-config.yaml) — `grund check` for citations and the managed blocks it verifies, `grund init --check` for the files `init` would still write, a block `check` leaves alone among them (a member's workspace sections, from a run inside it), `lychee` for Markdown links. For optional staged commit-msg/whole-PR evidence of related spec and test edits with reason-bearing trailers, see the [co-change recipe](cochange.md); Grund runs its demonstrations without enabling that contribution gate ([§FS-cochange-recipe.examples](../functional-spec/FS-cochange-recipe.md#examples-maintained-walkthrough-tests-and-opt-in-guidance)):

```bash
pip install pre-commit && cargo install lychee && pre-commit install
```

# AR-scanner: how grund discovers declarations and citations

The scanner is the single tree-walk that produces grund's input data, the `Findings` of section 3, and it holds the one probe over the tree that is no part of that walk: which agent entrypoint files a repository has ([§AR-system.2.5](README.md#25-scanner)). Every check in [§FS-check](../functional-spec/FS-check.md#fs-check-grund-validates-every-reference-in-a-repo) and every retrieval in [§FS-show](../functional-spec/FS-show.md#fs-show-grund-reads-a-single-declaration-body-by-id) derives from what the scanner finds. Speed ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)) is set here.

## placement: Where the scanner sits

```text
config ────┐
workspace ─┼─► [ scanner ] ─► Findings ─┬─► checker
grammar ───┘                            ├─► queries
                                        └─► writers
```

The fifth box of the pipeline ([§AR-system.2.5](README.md#25-scanner)). It takes the scope from workspace and config ([§AR-system.2.4](README.md#24-workspace), [§AR-system.2.3](README.md#23-config)) and the lexical facts from grammar ([§AR-system.2.1](README.md#21-grammar)), and gives one `Findings` (section 3) to the checker, the queries and the writers ([§AR-system.2.6](README.md#26-checker), [§AR-system.2.7](README.md#27-queries), [§AR-system.2.8](README.md#28-writers)). It knows no rule and no frontend, and never asks whether it is in a workspace: the resolver above it answers that ([§AR-workspace.3.2](AR-workspace.md#32-the-scanner-never-branches-on-workspace)).

## 1. Tree walk

A recursive walk from the scan roots ([§AR-scanner.1.6](AR-scanner.md#16-the-roots-the-walk-starts-from)) using the `ignore` crate, the same walker that powers `ripgrep`, chosen because it gives `.gitignore` support for free ([§AR-scanner.1.1](AR-scanner.md#11-respecting-gitignore-and-friends)). It hands the per-file scan one sorted list holding each physical file once ([§AR-scanner.1.8](AR-scanner.md#18-one-physical-file-is-read-once), [§AR-scanner.1.12](AR-scanner.md#112-the-extension-filter-and-the-scan-order)), beside the directories it descended into ([§AR-scanner.1.10](AR-scanner.md#110-the-walk-carries-out-the-directories-it-descended-into)).

### 1.1 Respecting `.gitignore` and friends

By default the walker honors every form of ignore file the `ignore` crate recognizes, the four that [§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-walked) lists under `respect_gitignore`: `.gitignore` files apply at any depth with nearest-wins precedence, as `git` itself does, and `.ignore` files (the ripgrep convention) hold `grund`-specific exclusions that are not appropriate for `git`. So `grund` does not scan files that `git` would not commit: generated artefacts, secrets, and vendored dependencies are skipped without any `grund.toml` configuration, and a repo's existing `.gitignore` is the source of truth.

Setting `[scan] respect_gitignore = false` turns this off, and is only for a repo that genuinely needs ignored paths scanned. The directory skip rules of [§AR-scanner.1.2](AR-scanner.md#12-directory-skip-rules) apply **in addition** to the ignore files, never instead of them ([§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-walked)).

### 1.2 Directory skip rules

The walker's directory filter prunes, at any depth:

- a hidden directory, any name starting with `.`, which already covers `.next`, `.venv`, and friends; a hidden **file** is [§AR-scanner.1.5](AR-scanner.md#15-a-hidden-file-is-not-read)'s rule, not this one;
- a build or output directory named in `[scan] exclude`, whose default is [§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-walked)'s and which is configurable per [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable).

A symlinked file is **followed** wherever it resolves. A symlinked directory is descended into only while its canonical target stays inside the walker's canonical project root, and the filter prunes an outward target ([§FS-config.3.5.1](../functional-spec/FS-config.md#351-a-symlink-in-the-tree-is-followed)). The filter pays that prefix comparison only after `resolved_link_dir` has already canonicalized a link-reached directory ([§AR-scanner.1.3](AR-scanner.md#13-the-boundary-rules-ask-the-canonical-path)), so a link-free tree pays nothing. Loaded-workspace ownership is checked beside that fence and remains stronger for another project physically inside the root ([§AR-workspace.6](AR-workspace.md#6-the-workspace-boundary)).

A followed entry keeps its in-tree path, so the two name rules above apply to a followed directory under its **link** name ([§FS-config.3.5.3](../functional-spec/FS-config.md#353-the-directory-rules-apply-under-the-link-name)) and every finding is reported there rather than at the target ([§FS-config.3.5.2](../functional-spec/FS-config.md#352-a-finding-names-the-in-tree-link-path)).

### 1.3 The boundary rules ask the canonical path

The two **boundary** rules, a workspace member root ([§AR-workspace.6](AR-workspace.md#6-the-workspace-boundary)) and an E2E case directory ([§AR-scanner.6](AR-scanner.md#6-e2e-case-declarations)), are not name rules: each is a property of the directory itself, not of the name it is reached under. So for a directory reached through a link, the link or anything below it, both are asked of its **canonical** path as well as its in-tree one, and it is pruned if either says so. Without that, a link walks a root scan straight into a member namespace [§AR-workspace.6](AR-workspace.md#6-the-workspace-boundary) forbids, and a link onto the E2E cases root scans the fixture repos the manifest pass owns. Only a link-reached directory pays the `canonicalize`, and only directories are asked at all, so the ordinary walk is untouched ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).

### 1.4 An unwalked home is pruned by its path

The filter also skips a home marked `scan = false` ([§FS-config.3.4.7](../functional-spec/FS-config.md#347-scan--a-place-that-is-listed-not-walked)), home and contents alike, and asks this before the entry is known to be a directory. Leaving the home out of the roots ([§AR-scanner.1.6](AR-scanner.md#16-the-roots-the-walk-starts-from)) is only half of it: the walk meets the same home again as a descendant of any root above it, a **single-file** home is never a directory to prune, and pruning its parent, an ordinary scanned directory, is not on offer. Like the boundary rules it is not a name rule: it strips the entry's in-tree path to the config root and compares it against the home, the way [§AR-scanner.2.4](AR-scanner.md#24-citing-side-classification) decides which home a file is in, so it prunes that home and nothing that merely shares its last component. It is not asked at all under `--full`, which reaches the home like any unconfigured directory, nor in a tree that configures no unwalked kind, which is where the cost of asking it per entry would otherwise fall ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).

### 1.5 A hidden file is not read

A file whose name begins with `.` is not read: the only skip rule that speaks about a **file's own name**. It is not in the filter. The walker is built with `hidden(false)` and the filter's own hidden test is asked of directories alone, so this rule is applied per entry as the file list is assembled, ahead of the extension filter ([§AR-scanner.1.12](AR-scanner.md#112-the-extension-filter-and-the-scan-order)), and a hidden `.md` is dropped by a rule `[scan] extensions` never gets to answer for ([§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-walked)).

Being asked per *entry* rather than per *descent*, it is the one skip rule that reaches a walk **root** ([§AR-scanner.1.7](AR-scanner.md#17---full-walks-the-configured-roots-too)). A root that is a single file is tested by it and contributes nothing, so an `include` entry naming a hidden file, and a `[[kinds]]` `file` home whose name is hidden, are read by nobody, while a hidden *directory* root is walked. `--full` cancels only `include` and the `scan = false` skip ([§AR-scanner.1.6](AR-scanner.md#16-the-roots-the-walk-starts-from)), so a hidden file is invisible to `grund check` and to `grund check --full` alike. That is a declared, bounded blind spot ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)): the rule is the file's own name, so the way to have a document checked is to give it a name that is not hidden.

### 1.6 The roots the walk starts from

The walk starts from `[scan] include` resolved against the config root **plus every configured `[[kinds]]` home** ([§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-walked)), less any home marked `scan = false`, a place in the Project map and not a root here, and less any `include` entry at or inside such a home, the one way it would still be a root and so out of reach of [§AR-scanner.1.4](AR-scanner.md#14-an-unwalked-home-is-pruned-by-its-path)'s skip ([§FS-config.3.4.7](../functional-spec/FS-config.md#347-scan--a-place-that-is-listed-not-walked)). An explicit path argument replaces that list. Under `grund check --full` ([§FS-check.1.3](../functional-spec/FS-check.md#13-the-full-tree-scope---full)) the walk starts from that list *together with* the config root itself, which cancels `include` and the `scan = false` skip and nothing more ([§AR-scanner.1.7](AR-scanner.md#17---full-walks-the-configured-roots-too)). The homes come after the `include` roots, so the first-seen spelling of a file reached two ways is the one `include` gives it ([§AR-scanner.1.8](AR-scanner.md#18-one-physical-file-is-read-once)); a home that does not exist contributes no files and no finding. The skip rules, the ignore files, and the extension filter apply identically to every root: only where the descent begins varies.

The reporting walk and the probe ([§AR-scanner.1.11](AR-scanner.md#111-the-unread-block-probe-runs-the-walk-and-scans-nothing)) gate each scan root before walking it; this cannot be left to the filter, which never prunes a walk root ([§AR-scanner.1.7](AR-scanner.md#17---full-walks-the-configured-roots-too)). A canonical directory root outside the canonical project root is skipped when the written root is a symlink; a plain parent-relative root remains intentional scan scope, and a config root whose own spelling is aliased remains the project. A root inside a member boundary or owned by another loaded project is skipped whole. A file root is not subject to the directory fence and retains [§FS-config.3.5.1](../functional-spec/FS-config.md#351-a-symlink-in-the-tree-is-followed)'s file-link behavior.

### 1.7 `--full` walks the configured roots too

The `--full` root list keeps the configured roots rather than starting at the config root alone because none of the three directory-level rules (hidden names, `[scan] exclude`, ignore files) can prune a walk root: the `ignore` walker never applies an ignore file or a hidden-name test at depth 0, and the `[scan] exclude` filter skips it too. A gitignored, excluded, or hidden `include` root is therefore read by the plain run, where it *is* a root, and a wider walk started only at the config root would prune it as a descendant and read fewer files than the plain run ([§FS-check.1.3](../functional-spec/FS-check.md#13-the-full-tree-scope---full)). A hidden-**file** root is read by neither list ([§AR-scanner.1.5](AR-scanner.md#15-a-hidden-file-is-not-read)), so additivity is unharmed: the file is missing from both.

### 1.8 One physical file is read once

The roots can overlap, so the one sorted file list is deduplicated by path before scanning: a file reached from two roots is read once ([§FS-config.3.5.4](../functional-spec/FS-config.md#354-one-physical-file-is-read-once)), and an overlapping `include` pair produces no duplicate-declaration report. Path is enough while every root spells its descendants the same way. It is not enough for a second **spelling**, which reaches one file under two names: a root that is a symlink to a directory inside the config root or a case variant of one, a file that is itself a link, or anything below a directory link. So the roots are canonicalized once before the walk, and while it walks the scanner records the files that can wear a second name (every file of such a root, a file that is itself a link, anything below a directory link) and resolves only those with `canonicalize`. Their resolved targets are the only paths another file can collide with, so the first-seen-wins pass keyed on file identity tests every other file against that small target set by path and never resolves it. `include` roots come first so that first-seen is the spelling the plain run reports, which keeps `--full` purely additive ([§FS-check.1.3](../functional-spec/FS-check.md#13-the-full-tree-scope---full)); each root's own file list is sorted before it joins the accumulated one, so "first seen" is deterministic within a root as well as across them ([§FS-errors.4](../functional-spec/FS-errors.md#4-determinism)).

What turns the pass on is that **list**, not a flag. A tree with no symlink and no root spelled other than its canonical path does not run it; a tree with one symlink pays one `realpath` rather than one per file. A flag would make a single link anywhere charge the entire repository, and real repositories have a link; the list is what keeps the ordinary walk at the cost [§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible) sets.

### 1.9 A link the walker cannot resolve is a scan failure

`follow_links` also turns a link the walker cannot resolve into an *error* entry in place of the file: a broken target, or a loop, which the `ignore` crate detects and reports rather than recursing into. A directory link whose target is at or above the walk root itself (`docs/up -> ..`) is the one loop the walker cannot see, so the directory filter prunes it where it is met and it is reported after the walk ([§FS-config.3.5.5](../functional-spec/FS-config.md#355-a-link-the-walk-cannot-resolve-is-reported-and-not-walked-into)). Those are collected as the per-file scan failures of [§FS-check.2](../functional-spec/FS-check.md#2-outputs) instead of aborting the walk. The walker applies neither its directory filter nor its ignore files to an error entry, so both are re-applied by hand: the hidden-name and `[scan] exclude` tests before a loop is reported, and, for a broken link, the extension filter ([§AR-scanner.1.12](AR-scanner.md#112-the-extension-filter-and-the-scan-order)) plus a one-level re-walk of the link's own directory, where the link is an ordinary entry and the ignore rules do reach it. Only a positive "the walker filtered this out" suppresses the report; an unreadable parent reports, because the wrong way to be wrong here is silently.

### 1.10 The walk carries out the directories it descended into

Beside the file list, the walk carries out **the directories it descended into**, scan roots included, sorted and deduplicated. It asks nothing of them: the one caller is the unlisted-`[workspace]` rule ([§FS-check.4.8](../functional-spec/FS-check.md#48-unlisted-workspace-block)), which probes each for a config and answers the claim above this layer, because the scanner never asks "am I in a workspace?" ([§AR-workspace.1](AR-workspace.md#1-layering)). Collecting them here keeps that rule from needing a second traversal: the entries are already being enumerated, and a directory is exactly what falls out of the extension filter.

### 1.11 The unread-block probe runs the walk and scans nothing

The walker itself, the builder and its directory filter, has a **second caller that scans nothing**. The unread-opted-out-block rule ([§FS-check.4.10](../functional-spec/FS-check.md#410-include_root--false-leaves-the-blocks-own-files-unread)) has to answer "would this block have read anything, had it been a project?", and answers it by running the walk rather than by restating its rules: a probe that pruned differently from the scan would caution a repository about a tree the scan would have skipped anyway, which is the one outcome that finding cannot afford. So the two share one builder and the same canonical physical-root fence ([§AR-scanner.1.2](AR-scanner.md#12-directory-skip-rules)). The probe's member boundary is the block's own members rather than the run's, and `--full` is off, because the question is about the configuration rather than about one walk ([§FS-check.1.3](../functional-spec/FS-check.md#13-the-full-tree-scope---full)).

The probe drops everything downstream of the walk: `Walk` is lazy, so it stops at the first file that passes the extension filter ([§AR-scanner.1.12](AR-scanner.md#112-the-extension-filter-and-the-scan-order)), keeps no file list, and raises no error for a path it cannot read. No run is scanning this tree, so there is no report for one to land in.

### 1.12 The extension filter and the scan order

Files are filtered by extension to those that can plausibly contain specs or inline declarations: `.md` and a curated list of source-file extensions, the `[scan] extensions` of [§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-walked). The walk produces one sorted file list before scanning starts. Small lists are scanned sequentially; large lists may scan files in parallel, but each file writes into a private result and the merge always happens in sorted path order. That preserves the byte-for-byte report ordering [§FS-errors.4](../functional-spec/FS-errors.md#4-determinism) requires while letting the hot full-tree commands use multiple cores once the thread-pool overhead is worth paying.

## 2. Per-file scan

A single linear pass over each file's lines performs three jobs together: declaration detection ([§AR-scanner.2.1](AR-scanner.md#21-declaration-detection)), section detection ([§AR-scanner.2.2](AR-scanner.md#22-section-detection)), and citation detection ([§AR-scanner.2.3](AR-scanner.md#23-citation-detection)).

### 2.1 Declaration detection

A regex matches declaration lines in one of two context-specific shapes:

1. **Markdown-form** — `#{1,N} <ID>[:…]`: a `#`-prefixed heading at any markdown level. This is how `.md` files declare (`# FS-foo:`).
2. **Code-form** — `<comment-prefix> <ID>[:…]`, or bare `<ID>[:…]` inside a Python docstring: a doc-comment line with the ID directly after the marker, no markdown `#` prefix. Decided in [§DF-code-declarations-drop-hash](../decisions/functional/DF-code-declarations-drop-hash.md#df-code-declarations-drop-hash-code-resident-declarations-may-drop-the--prefix). The comment prefix is **required** outside Python docstrings — without a `#` heading in markdown or a doc-comment marker in source, a line `FS-foo: anything` in prose is not a declaration.

Both forms record the same `Declaration` struct downstream; consumers (`grund <ID>`, `grund check`, `grund refs`) do not care which shape the source used. `E2E` declarations are the exception: they are directories, not heading lines ([§AR-scanner.6](AR-scanner.md#6-e2e-case-declarations)).

#### 2.1.1 The ID a declaration line carries

`<ID>` is the kind's effective grammar — the configured `[id]` grammar, or a citable `[[kinds]]` row's own `format` where it sets one ([§FS-config.3.2](../functional-spec/FS-config.md#32-id--id-grammar), [§FS-config.3.4.10](../functional-spec/FS-config.md#3410-format-resolve-and-fetch--external-snapshot-kinds)) — with `{kind}` drawn from a configured `[[kinds]]` prefix. A token in either shape that begins with a configured citable kind and ends at the declaration colon but misses that format still opens a declaration: it is retained in the catalog under its exact spelling and reported as a near miss ([§FS-check.4.6](../functional-spec/FS-check.md#46-declaration-near-miss)).

A markdown-form heading may sit at any level; where each default kind declares, and at which level, is [§FS-config.3.4.4.1](../functional-spec/FS-config.md#3441-where-each-citable-default-declares).

#### 2.1.2 The declaration heading level

When the regex matches, the line opens a new "current declaration" context and the **declaration heading level** `L` is recorded:

- Markdown-form: `L` is the count of `#` on the line (`#` -> 1, `##` -> 2, ...).
- Code-form: `L` defaults to `1`: the declaration is treated as a "level-1" heading *within* the comment block, so its sections are still numbered `## 1. …`, `### 1.1 …`, etc., one or more `#` deeper than the declaration line.

#### 2.1.3 Value declarations

For a `values = true` kind, this pass also validates the immediate numbered Markdown component run and records each component's raw text and exact span. A focused span-preserving JSON reader contributes equivalent declarations from only that kind's home-derived source set; it preserves member order and duplicate keys before the shared catalog is built ([§FS-values.2](../functional-spec/FS-values.md#2-value-declarations)).

### 2.2 Section detection

Within a declaration context whose heading is at level `L`, a numbered subsection heading is a line of the form `#{L+1,} <n₁.n₂.….n_d>[.] <title>` — at least one `#` more than the declaration heading, then a dotted number of one or more components, an **optional** trailing `.`, whitespace, and the heading text. The line is recorded on the current declaration as the section path `n₁.n₂.….n_d` together with its `<title>` text, source line, and Markdown heading level: the heading text is needed by [§FS-fmt.6](../functional-spec/FS-fmt.md#6-cross-reference-emission) / [§DF-md-link-anchor-strategy](../decisions/functional/DF-md-link-anchor-strategy.md#df-md-link-anchor-strategy-heading-text-slugs-re-derived-on-every-fmt-pass) and by `grund <ID> --format=md`, the source line and level by [§FS-check.3.9](../functional-spec/FS-check.md#39-section-heading-level-mismatch). Plain, unnumbered headings and bold labels are Markdown prose structure and are not recorded as sections. Nesting depth is unbounded ([§FS-config.3.3](../functional-spec/FS-config.md#33-section-paths--arbitrary-nesting-depth)); the recorded set is what [§AR-checker.2.3](../../crates/grund-core/src/checker/report.rs) validates citations against.

#### 2.2.1 How deep a section heading sits

The dotted number fixes the section's tree position, and the configured `[id] section_heading_levels` mode fixes how strictly the written `#` depth must match it ([§FS-config.3.3](../functional-spec/FS-config.md#33-section-paths--arbitrary-nesting-depth)). In `"strict"` mode (the default), the heading level must be exactly `L + d`, where `d` is the number of dotted path components: under an H1 declaration, `## 1.1 Recognized citations` is recorded but later reported as a check error. `"warn"` records the same mismatch as a warning. In `"loose"` mode, the historical rule applies: the `#` count only has to be deeper than the declaration heading, so `## 1.1` and `### 1.1` both declare section `1.1`.

#### 2.2.2 Named section paths

Config loading selects one heading grammar for the project. With `[id] named_sections = true`, that grammar adds explicit colon-form paths containing `[a-z][a-z0-9-]*` components, all-name at any depth and with numeric components only after a named prefix. It records them in the same first-wins section map and duplicate list as numeric headings ([§AR-scanner.2.2.3](AR-scanner.md#223-the-first-heading-claims-a-path)), carrying the complete path, complete rendered heading text, source line, and level. No consumer re-parses named headings: checking, `show` and JSON, `refs`, formatting anchors, completion, and the core LSP snapshot all answer from this mixed-component record. The map also supplies the proper-prefix set for [§AR-checker.2.17](../../crates/grund-core/src/checker/report.rs).

#### 2.2.3 The first heading claims a path

A path is recorded **once**, by the first heading that claims it. A later heading claiming a path already on the declaration does not overwrite it; it is appended to a parallel `duplicate_sections` list, in file order, carrying the same `SectionInfo`. Recording first-wins rather than last-wins is what makes the map agree with the file: the first heading is the one a reader scrolling to section `1` meets, the one `show`'s body extraction starts at, and the one the error anchors at. Both collections are provisional until the body-span post-pass retains only headings the declaration owns ([§AR-scanner.2.2.5](AR-scanner.md#225-coordinates-are-body-local), [§FS-show.2.1.2](../functional-spec/FS-show.md#212-section-map---toc)).

#### 2.2.4 Nothing resolves through the duplicate list

The map alone answers a `§<ID>.<path>` citation, the completion candidates, and the heading-level rule ([§FS-check.3.9](../functional-spec/FS-check.md#39-section-heading-level-mismatch)). Two commands do read the list, and read nothing else for the question they ask — [§AR-checker.2.15](../../crates/grund-core/src/checker/report.rs) names every colliding line in the duplicate-section error ([§FS-check.3.16](../functional-spec/FS-check.md#316-duplicate-section-path)), and `show` refuses a coordinate the list holds ([§FS-show.2.2.2](../functional-spec/FS-show.md#222-ambiguous-section)) before it re-reads the file for the body. `--toc` over a whole declaration is *not* one of them: it builds its map by re-scanning the source rather than by reading either structure, so it still prints both heading lines, which is what [§FS-show.2.2.2](../functional-spec/FS-show.md#222-ambiguous-section) exempts it for.

#### 2.2.5 Coordinates are body-local

**Every coordinate is recorded only inside the declaration's own body.** The scan's "current declaration" runs to the next declaration line or end of file, which is wider than the body span [§AR-scanner.2.4.1](AR-scanner.md#241-declaration-body-range) computes — a `## 1.` in the *next* function's doc-comment, or under a later unrelated Markdown heading, first lands on the previous declaration. The same post-pass that assigns body spans removes those headings from both `sections` and `duplicate_sections`, so resolution, checking, values, completion, list, refs, and LSP navigation all consume one body-local map. A stub spans its single link line ([§AR-scanner.2.4.1](AR-scanner.md#241-declaration-body-range)), so its prose contributes no coordinate at all.

#### 2.2.6 Headings outside every body

Each numeric or enabled named heading [§AR-scanner.2.2.5](AR-scanner.md#225-coordinates-are-body-local) removes is retained once in `section_headings_outside_declarations`, with its path and line, for the hard [§FS-check.3.23](../functional-spec/FS-check.md#323-section-outside-a-declaration) finding. That record is separate from the coordinate map: it can be selected, merged across parallel and workspace scans, carried through full scope, and transported to the LSP without making the rejected heading resolvable again. Legal plain headings are untouched; they only bound Markdown bodies under [§AR-scanner.2.4.1](AR-scanner.md#241-declaration-body-range).

#### 2.2.7 Unmarked Markdown headings

The plain-heading permission of [§AR-scanner.2.2](AR-scanner.md#22-section-detection) is narrowed by the Markdown body policy: for a Markdown file only, the same fence-aware ATX pass records a heading that matched neither a declaration nor the configured numeric/named section grammar as an unmarked candidate. The body-span post-pass retains it only when its level is deeper than a declaration and its line belongs to that body, assigning the nearest enclosing declaration where nested bodies overlap ([§AR-scanner.2.4.3](AR-scanner.md#243-the-enclosing-declaration)). The record carries the complete heading text, level, line, owner, and the deterministic unused path derived from preceding recorded or suggested siblings for [§AR-checker.2.20](../../crates/grund-core/src/checker/report.rs). Source comment envelopes never enter this candidate path, and full-scope narrowing drops out-of-policy candidates before checking ([§FS-check.4.14](../functional-spec/FS-check.md#414-unmarked-markdown-heading)).

#### 2.2.8 Value components

Value components reuse `SectionInfo`, annotated with the exact component span. For an embedded root, that same record carries `EmbeddedValueRoot` validity, its origin, and — for a marked root — the marker column; its existing dotted path is the authority, with no synthetic declaration or resolver. A section earns that record by either of two enrollment routes: the exact marker on an all-numeric path, or a path of `<chapter>.<name>` under the declaration's direct named chapter that the kind's `value_chapter` names ([§FS-values.2.5](../functional-spec/FS-values.md#25-chapter-declared-value-roots)). Downstream resolution, subtree validation and comparison read the record rather than the route. The scanner strips configured source-comment wrappers before counting authored heading depth, so `# ## 1` and `/// ## 1` are both authored H2 roots while enabled Python docstring content stays on the Markdown heading path ([§FS-values.2.4](../functional-spec/FS-values.md#24-embedded-section-value-roots)).

Whole Markdown values and embedded roots of either origin admit only contiguous immediate numeric children, while JSON array index `i` creates the same coordinate `i + 1`; downstream consumers therefore never maintain a second section table ([§FS-values.2](../functional-spec/FS-values.md#2-value-declarations)). Embedded shape validation walks physical source order: every immediate numeric candidate advances the coordinate position even when its title is invalid, so one content error neither cascades onto later components nor hides later out-of-order sites.

### 2.3 Citation detection

The full-ID and number-only-ID passes keep precedence. Beside them, the per-line scan records a
deferred candidate for each configured-marker-plus-numeric-path token and for each unsupported
digit-starting tail that must receive a whole-token verdict
([§FS-check.1.1.8](../functional-spec/FS-check.md#118-declaration-local-numeric-section-candidates)).
The candidate uses literal dots between numeric components under both strict modes, shares the
existing fence, escape, comment/docstring, and walk exclusions, and does not infer an ID during
lexing.

The citation regex matches the configured marker ([§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger); default `§`) immediately followed by an `<ID>` token, with an optional `<sep><section-path>` suffix, anywhere in the file. Which of its matches are citations is [§FS-check.1.1](../functional-spec/FS-check.md#11-recognized-citations)'s rule: the strict default, and under `strict = false` the bare-token carve-outs for a source-file string literal and a Markdown link destination ([§AR-scanner.2.3.1](AR-scanner.md#231-the-carve-outs-share-one-predicate)). A declaration's own heading line is never counted as a citation of the ID it declares.

#### 2.3.1 The carve-outs share one predicate

The two carve-outs share one predicate, `bare_token_in_never_rewrite_zone`, whose string half applies the deterministic left-to-right quote-tracking rule of [§FS-fmt.2.3.1](../functional-spec/FS-fmt.md#231-string-literal-exclusion-rule) — both are the never-rewrite zones [§FS-fmt.2.3](../functional-spec/FS-fmt.md#23-what-is-never-rewritten) forbids `fmt` from touching, so `check` never demands an edit there either.

#### 2.3.2 A qualified citation in source code

The source-only skip of a marker-prefixed **qualified** `§<alias>/<ID>` inside an inline-code span or a string literal is the same path-collision caution that already rules out an *unmarked* `alias/ID` ([§AR-workspace.3.1](AR-workspace.md#31-the-rule)). That collision is a source-language hazard; in Markdown the typed marker is the author's deliberate act, so a marker-prefixed qualified citation in a `.md` file — including one wrapped in backticks — is always a citation, exactly like the unqualified form ([§FS-workspace.1](../functional-spec/FS-workspace.md#1-citation-syntax)).

#### 2.3.3 Fence state is decided first

Markdown fence state is decided before this pass, by the opener and closer rules of [§FS-check.1.1](../functional-spec/FS-check.md#11-recognized-citations): the scanner remembers the opening character and run length, so a tilde run cannot close a backtick fence, a shorter run cannot close a longer one, and an indented code sample that merely contains three delimiters cannot hide the remainder of the file. Delimiter lines and every line while that state is open bypass declaration, section, citation, and escaped-citation detection together.

#### 2.3.4 Named section tails

Under `[id] named_sections = true`, citation scanning tokenizes the complete ID plus letter-bearing dot tail before classification. A marked legal named path becomes one citation record even when its section is absent; an unmarked letter-tail candidate and a reserved `number.name` candidate are suppressed whole rather than falling back to the bare ID or a numeric prefix. Full configured IDs are attempted before number-only shorthand ([§AR-scanner.2.6.1](AR-scanner.md#261-it-runs-after-the-full-pass-and-skips-its-markers)), and shorthand canonicalization does not consume section validity: the checker can therefore report the approved independent findings from one site. With the gate absent or false, the compiled grammar and emitted records are the existing numeric-only ones.

#### 2.3.5 Value bindings

After emitting the ordinary citation, the same line pass recognizes the exact authored binding form and emits a `ValueBinding` containing its literal and source span. Its section path remains complete: the checker can split an embedded root from its immediate component after ordinary local/workspace resolution. In source files recognition occurs only inside the comment or doc-comment line already classified by this scanner; Markdown fences and host expressions or strings remain excluded ([§FS-values.3](../functional-spec/FS-values.md#3-explicit-value-bindings)).

### 2.4 Citing-side classification

After body spans are assigned, local numeric candidates use the same enclosing-declaration result
as ordinary citing-side classification. A unique owner promotes the candidate to an ordinary
`Citation` carrying the canonical owner ID and numeric section plus its authored local spelling;
the nearest preceding declaration rule handles a multi-declaration source comment. No owner, or
an ownership state that is not unique, remains a diagnostic-only candidate. Adjacent citations
and declarations in other files never participate. This single promotion point makes checker,
queries, coverage, grounding, citation directions, and LSP snapshots consume one graph edge
([§DF-declaration-local-section-shorthand.2.2](../decisions/functional/DF-declaration-local-section-shorthand.md#22-existing-body-ownership-is-the-only-owner)).

The scanner knows a citation's *cited* kind from its ID, but the citation-direction rules ([§FS-config.3.9](../functional-spec/FS-config.md#39-citations--citation-direction-rules), [§DF-citation-directions](../decisions/functional/DF-citation-directions.md#df-citation-directions-encode-citation-directions-as-checked-config-with-rfc-2119-levels)) also need the *citing* kind — what kind of place the citation sits in. This is resolved once, at scan time, because the checker cannot reconstruct it cheaply: doc-comment declaration bodies are narrower than the file, and the same data is what [§FS-cover](../functional-spec/FS-cover.md#fs-cover-grund-groups-citations-by-scanned-file) and [§RM-gap-report](../roadmap.md#rm-gap-report-orphan-and-uncovered-id-reports) need. Three scan-time additions carry it: a body range on every declaration ([§AR-scanner.2.4.1](AR-scanner.md#241-declaration-body-range)), a source kind on every citation ([§AR-scanner.2.4.2](AR-scanner.md#242-citation-source-kind)), and the accepted chapter that immediately encloses the site ([§AR-scanner.2.4.4](AR-scanner.md#244-the-enclosing-accepted-chapter)).

#### 2.4.1 Declaration body range

Every `Declaration` records the line span of its body. In a Markdown file the body runs from the declaration heading until the next heading at the **same or higher** level (or end of file) — numbered subsections ([§AR-scanner.2.2](AR-scanner.md#22-section-detection)), which are deeper, stay inside it. In a source file the body is bounded by the **comment / docstring block** the declaration line opens ([§AR-scanner.4](AR-scanner.md#4-inline-declarations-in-language-doc-comments)); within a multi-ID block the nearest preceding declaration line wins, so an `AR-` and an `FS-` on one class partition the comment between them. A stub heading ([§AR-scanner.4](AR-scanner.md#4-inline-declarations-in-language-doc-comments)) and an `E2E` case directory ([§AR-scanner.6](AR-scanner.md#6-e2e-case-declarations)) span their single declaration line only.

#### 2.4.2 Citation source kind

Every `Citation` records the citing kind, resolved by three-step fallback with the bounds of [§AR-scanner.2.4.1](AR-scanner.md#241-declaration-body-range): (1) the kind of the **enclosing declaration** — the nearest preceding declaration whose body range contains the site; else (2) the **kind home of the file** — the reverse lookup from `[[kinds]]` `folder` / `file` ([§FS-config.3.4](../functional-spec/FS-config.md#34-kinds--recognized-kinds)), used only when exactly one home contains the file; else (3) the **homeless kind** — the complement of every home, named `code` unless the project declared it under another name ([§FS-config.3.9.2](../functional-spec/FS-config.md#392-the-homeless-kind)). Step 2 asks nothing about declarations, which is what makes a **non-citable** kind ([§FS-config.3.4.1](../functional-spec/FS-config.md#341-citable--kinds-that-declare-no-ids)) work through this code unchanged: its files classify as that kind because of where they are, and `[citations.<kind>]` then governs them. A citation later in the file than a declaration's body does **not** inherit that declaration's kind — it falls through to step 2 or 3.

#### 2.4.3 The enclosing declaration

The enclosing declaration is also recorded on the citation, so the obligation pass ([§AR-checker.2.9](../../crates/grund-core/src/checker/report.rs)) can ask "does this declaration's body cite the target?" as a lookup rather than a re-scan.

Unmarked Markdown candidates ([§AR-scanner.2.2.7](AR-scanner.md#227-unmarked-markdown-headings)) use the same body ranges and nearest-enclosing lookup. That shared ownership is why a candidate under a nested declaration names the nested ID, while a pre-declaration title or a same-or-shallower ATX heading that closed the body has no owner and remains legal ([§FS-check.4.14](../functional-spec/FS-check.md#414-unmarked-markdown-heading)).

#### 2.4.4 The enclosing accepted chapter

Every citation site also records its immediate enclosing accepted chapter, if
one exists. The scanner answers this while it holds the heading stack: the
nearest preceding accepted section whose subtree contains the site wins, and a
same-or-shallower heading closes it. A duplicate or rejected section path is
not an accepted chapter and cannot own a site. This is structural attribution,
not rule evaluation ([§FS-rules.5.1](../functional-spec/FS-rules.md#51-facts-and-identity)).

The scanner neither imports the rules component nor constructs `RuleFacts`.
The Markdown fact adapter reads this field from the resolved structural model
and decides which `cites` and `site_in` relations it implies
([§FS-rules.11](../functional-spec/FS-rules.md#11-functional-architecture-constraint),
[§AR-rules.3](AR-rules.md#3-rulefacts)). That keeps a scan reusable by every
command and lets a non-Markdown producer supply the same fact boundary without
imitating scanner records.

### 2.5 Escaped-citation illustrations

The `<§>`-escape ([§AR-workspace.3.1](AR-workspace.md#31-the-rule)) writes a citation's *shape* without it being live: the literal `<§>alias/ID` puts a `>` between the marker and the ID, so the citation pass ([§AR-scanner.2.3](AR-scanner.md#23-citation-detection)) never matches it. That inertness also hides a live citation whose marker was bracketed by accident, so a dedicated pass records these escapes into a separate, check-inert `escaped_citations` list for the checker to flag one that resolves ([§FS-check.2.3.1](../functional-spec/FS-check.md#231-escaped-citation-resolves)); nothing else reads the list, so it never affects an existing check.

#### 2.5.1 What the escape pass collects

The pass is cheap — a line that lacks the literal `<§>` needle short-circuits before any parsing — and runs uniformly in Markdown and source, since the escaped form is inert in both. Both unqualified `<§>ID` and qualified `<§>alias/ID` shapes are collected; the trailing ID is parsed with the citing project's grammar ([§FS-workspace.5](../functional-spec/FS-workspace.md#5-command-scope)'s loose parser is the cross-namespace fallback), so an exotic target grammar can miss a match — only ever costing a suggestion, never a false error. Escapes inside a fenced code block are skipped along with everything else there ([§AR-scanner.2.3.3](AR-scanner.md#233-fence-state-is-decided-first)).

### 2.6 Number-only shorthand citations

Where a kind's effective format ([§AR-scanner.2.1.1](AR-scanner.md#211-the-id-a-declaration-line-carries)) carries both `{number}` and `{slug}`, a second citation pattern is compiled beside the full one, one for the kinds that use `[id] format` and one for each `[[kinds]]` row whose own `format` qualifies: the format with the `{slug}` placeholder and one adjacent literal separator removed, so `{kind}-{number}-{slug}` yields `{kind}-{number}` and `§FS-042` is recognized ([§FS-check.1.2](../functional-spec/FS-check.md#12-the-number-only-shorthand)). A kind whose effective format misses either placeholder compiles no such pattern, and a project where no kind has one pays nothing anywhere below.

Five properties, [§AR-scanner.2.6.1](AR-scanner.md#261-it-runs-after-the-full-pass-and-skips-its-markers) to [§AR-scanner.2.6.5](AR-scanner.md#265-it-emits-an-ordinary-citation), make the pass safe to add to a grammar that already matches; a whole-file post-pass then resolves what it flagged ([§AR-scanner.2.6.6](AR-scanner.md#266-resolving-a-flagged-shorthand)).

#### 2.6.1 It runs after the full pass and skips its markers

The full-ID pass records the marker offset of every citation it matched on the line, whether or not it went on to emit one; the shorthand pass only considers markers outside that set. A configured grammar under which some full ID is also shorthand-shaped therefore resolves as the full ID ([§DF-number-only-citation-shorthand.2.6](../decisions/functional/DF-number-only-citation-shorthand.md#26-the-full-id-always-wins-and-only-a-whole-token-is-a-shorthand)). **A qualified marker is skipped when a qualified pass claimed it** — the same rule, extended to the other two producers ([§AR-scanner.2.6.1.1](AR-scanner.md#2611-qualified-markers)). **The record is scoped to one line**, like the full pass's: an offset means nothing across lines, so a record that outlived its own would suppress every shorthand sharing that column below it.

##### 2.6.1.1 Qualified markers

`§<alias>/<ID>` belongs to the qualified pass — the workspace one in workspace mode, the loose fallback outside it ([§FS-workspace.5](../functional-spec/FS-workspace.md#5-command-scope)) — and each records the markers it emitted at, so one marker is one citation. The workspace pass claims every qualified marker on the line, which makes its half of the rule unconditional in practice; the fallback claims only what its loose `KIND[-NUM]-SLUG` parser could read, and a qualified marker it declined is left to the shorthand pass rather than dropped. Skipping every qualified marker regardless would delete the citation under any `[id] format` the loose parser cannot read ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)).

#### 2.6.2 It requires a trailing boundary

The pattern is `\A`-anchored at the start only, so it matches the `FS-042` inside any longer ID-shaped token — including a full ID whose slug this grammar rejects, like `FS-042-User-Login`. Which characters after a match can continue an ID, and why `/` is not one of them, is [§FS-check.1.2](../functional-spec/FS-check.md#12-the-number-only-shorthand)'s rule. The `regex` crate has no lookahead, so the rule is a post-match character test rather than part of the pattern. Skipping it is not a missed citation but a *wrong* one — the pass would report a token the file does not contain, and `fmt` would splice the canonical slug into the middle of the author's text ([§FS-fmt.2.4](../functional-spec/FS-fmt.md#24-shorthand-to-canonical)).

#### 2.6.3 It records whether the token sits in a numeric run

A trailing boundary says the token *ended*; it says nothing about whether the token is a citation. `§SPEC-001→SPEC-003` clears the boundary test and is a renumbering table, so the pass also reads forward past the boundary for a delimiter run carrying a second number and flags the site ([§FS-fmt.2.4.1](../functional-spec/FS-fmt.md#241-a-shorthand-in-a-numeric-run-is-not-rewritten)). Two character classes decide it and no punctuation is enumerated, so the test is one forward walk of a few bytes on a token the earlier gates have already accepted — off the hot path of every canonical citation ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)).

#### 2.6.4 It is marker-gated unconditionally

Unlike the full-ID pass it has no bare branch, so `[reference] strict = false` does not widen it ([§AR-scanner.2.3](AR-scanner.md#23-citation-detection)). `KIND-NNN` is too common in the wild to recognize unmarked ([§DF-number-only-citation-shorthand.2.4](../decisions/functional/DF-number-only-citation-shorthand.md#24-the-marker-is-required-a-bare-shorthand-is-text)).

#### 2.6.5 It emits an ordinary `Citation`

The citation is flagged `shorthand` and carries an `Id` whose `slug` is `None` and the written token as its text.

#### 2.6.6 Resolving a flagged shorthand

A whole-file post-pass resolves each flagged citation against the declaration set: the declarations sharing its kind and number. Exactly one match rewrites the citation's `Id` to that declaration's, so every downstream consumer — checker, `refs`, `cover`, the unused-declaration warning, the LSP snapshot — reads a canonical `Id` and needs no shorthand awareness at all. Zero or several matches leave the slug `None`, which is the state [§AR-checker.2.12](../../crates/grund-core/src/checker/report.rs) reports on. The escaped list ([§AR-scanner.2.5](AR-scanner.md#25-escaped-citation-illustrations)) is resolved with it, so an escape whose shorthand would be live is still caught by [§FS-check.2.3.1](../functional-spec/FS-check.md#231-escaped-citation-resolves).

The resolution runs against the *project's* declarations, so it lands in the same merge step as the other whole-file post-passes ([§AR-scanner.2.4](AR-scanner.md#24-citing-side-classification)) rather than in the per-line loop, and stays in this component for that reason. A shorthand whose target lives in another namespace has no local grammar to expand it with: it is resolved later, against the target's declarations, by the resolver — the one component that holds every loaded project at once ([§AR-resolver.4](AR-resolver.md#4-the-shorthand-a-whole-runs-catalog-resolves), [§FS-workspace.1](../functional-spec/FS-workspace.md#1-citation-syntax)).

#### 2.6.7 One `(kind, number)` index per pass

Every pass that resolves more than one site builds a `(kind, number)` index of the declarations first and looks candidates up in it. Asking the question per site instead — a filter over every declaration in the project — is O(sites × declarations), and the tree where that bites is a repository full of shorthands being migrated to canonical form, which is precisely the tree this rule exists to be run over ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)). The index is built lazily, so a tree without shorthands builds none: `check` keys one per target namespace on first use, and `fmt` builds one per walk rather than one per line.

#### 2.6.8 Rewritability is decided here

A shorthand inside inline code, a Markdown link destination, or a source string literal is recorded as a citation like any other but flagged unrewritable, because [§FS-fmt.2.3](../functional-spec/FS-fmt.md#23-what-is-never-rewritten) forbids `fmt` from touching it. The scanner, not the checker, owns that call because it is the only pass holding the line text, and one predicate serving both keeps `check` from naming a fix `fmt` declines to make ([§FS-check.3.13](../functional-spec/FS-check.md#313-number-only-shorthand-citation)). The numeric-run flag ([§AR-scanner.2.6.3](AR-scanner.md#263-it-records-whether-the-token-sits-in-a-numeric-run)) rides along for the same reason and reaches a different verdict: unrewritable-here means say nothing, in-a-run means say something else ([§FS-check.3.15](../functional-spec/FS-check.md#315-shorthand-citation-in-a-numeric-run)).

#### 2.6.9 A docstring line is judged on its content

The question is "what would `fmt` do", so it has to be asked of the bytes `fmt` reads — and inside a Python docstring ([§AR-scanner.4](AR-scanner.md#4-inline-declarations-in-language-doc-comments)) `fmt` reads the interior, with the delimiters stripped, exactly as this scan does. That is the one place the raw line and the scanned line differ, and judging it on the raw line gave one docstring three verdicts by line: the `"""` opened a literal on the opening line and in a one-line docstring, opened nothing on an interior line, and on the closing line decided the site by whether it came before the citation or after it. [§FS-fmt.2.3.1](../functional-spec/FS-fmt.md#231-string-literal-exclusion-rule) settles it on the content — a docstring's delimiters are doc-comment syntax, not quotes — so a docstring line is judged like a `#` comment line and one predicate still serves both sides.

#### 2.6.10 The shared docstring view

`DocstringContent` is that shared view: the content slice plus its byte offset in the raw line, which is all `source_scan_line` ([§AR-scanner.4](AR-scanner.md#4-inline-declarations-in-language-doc-comments)) already computes. The scanner builds it from the line it just scanned; `fmt` re-derives it per rewrite stage rather than carrying it, because a stage that inserts a marker moves every offset after it; the LSP on-type path walks the document to the edited line for it ([§FS-lsp.1.4](../functional-spec/FS-lsp.md#14-live-trigger-transform)). Positions stay **raw-line** positions on every side — `never_rewrite_context_in` translates one into the content when it falls inside it and judges it on the raw line when it does not, which is what leaves a string literal on a code line, and the code after a one-line docstring closes, on the unchanged rule. A file that is not `.py`, and a project with `docstring_python = false`, build an empty view and pay two boolean tests.

#### 2.6.11 A partial `Id` renders as the shorthand

An `Id` with a `None` component whose placeholder appears in the format renders as the shorthand — `render_id` drops the placeholder and one adjacent literal, the same reduction the pattern is built from — so the one code path prints both forms and no caller has to special-case a partial ID.

### 2.7 Grounding units per file

When the row a file belongs to asks for a grounding unit finer than the file — an effective `grounding_level` above `1` ([§FS-config.3.4.8](../functional-spec/FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) — the per-file scan also records the structure the checker cuts those units out of: for a Markdown file every heading outside a fence, with its level and its text; for a source file every doc-comment block, with its line span and whether its first line is indented. The classifier that decides which comment blocks are documentation is the one [§AR-scanner.4.2](AR-scanner.md#42-doc-comment-or-inline-comment) already applies, read once per file from the extension.

#### 2.7.1 The file's own row asks

Which row a file belongs to is [§AR-scanner.2.4.2](AR-scanner.md#242-citation-source-kind)'s own home lookup — its home kind's row, or the homeless kind's where no single home claims it — which is what [§FS-check.3.6.1](../functional-spec/FS-check.md#361-which-files-a-row-governs) defers to for which row *governs* a file. Asking it here rather than asking "does any row in the table ask for one" is what keeps one place's level off every other place's files. The question is only reached at all where some row is above `1`: a project at level `1` answers it once, from a field the config resolved on load.

#### 2.7.2 Structure, not units

The record is **structure, not units**: the level that turns it into units belongs to a row, and one file can be reached by only one row, but keeping the cut in the checker leaves the level rule in one place ([§AR-checker.2.8](../../crates/grund-core/src/checker/report.rs)) and keeps this pass a pure description of the file. It is taken only for the files whose own row asks for it, which is the same shape [§AR-scanner.2.4](AR-scanner.md#24-citing-side-classification)'s citing-side classification uses: a project at level `1` — every configuration written before the key existed — records nothing and pays nothing ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)). The alternative, letting the checker re-read the governed files, would make the grounding rule the second one in the system to re-read the tree, and unlike the stub rule ([§AR-checker.2.5](../../crates/grund-core/src/checker/report.rs)) it would do so for every file in a home rather than for a handful of error sites.

## 3. Output

The walk's only structured output is a `Findings` struct, and everything downstream (checking, showing, IDE diagnostics) operates on it; which agent entrypoint files a repository has is the scanner's one answer outside it, from a probe that is no part of the walk ([§AR-system.2.5](README.md#25-scanner)). `Findings` contains:

- `declarations: BTreeMap<Id, Vec<Declaration>>` — keyed by ID, with file/line, stub-info, the recorded sections (each section path paired with its heading text — [§AR-scanner.2.2](AR-scanner.md#22-section-detection)) per declaration, and the body line range ([§AR-scanner.2.4.1](AR-scanner.md#241-declaration-body-range)). An `E2E` declaration ([§AR-scanner.6](AR-scanner.md#6-e2e-case-declarations)) carries its case-directory path, fixture list, invocation, and expected exit code instead.
- `citations: Vec<Citation>` — each with the referenced ID, optional section, file, line, and start column, whether it was written marker-prefixed or bare, whether it was written in the number-only shorthand ([§AR-scanner.2.6](AR-scanner.md#26-number-only-shorthand-citations)), the resolved source kind plus enclosing declaration ([§AR-scanner.2.4.2](AR-scanner.md#242-citation-source-kind), [§AR-scanner.2.4.3](AR-scanner.md#243-the-enclosing-declaration)), and, in a source inline comment, its inline citation site ([§AR-scanner.3.1](AR-scanner.md#31-inline-citation-sites)).
- `section_headings_outside_declarations: Vec<SectionHeadingOutsideDeclaration>` — the headings [§AR-scanner.2.2.6](AR-scanner.md#226-headings-outside-every-body) retains, only for [§FS-check.3.23](../functional-spec/FS-check.md#323-section-outside-a-declaration).
- `value_bindings: Vec<ValueBinding>` — exact authored components and citation sites recognized beside their ordinary `Citation` ([§AR-scanner.2.3.5](AR-scanner.md#235-value-bindings)); whole declarations carry value validity, and an ordinary `SectionInfo` the embedded-root validity, marker position or component span of [§AR-scanner.2.2.8](AR-scanner.md#228-value-components) ([§FS-values.2](../functional-spec/FS-values.md#2-value-declarations), [§FS-values.3](../functional-spec/FS-values.md#3-explicit-value-bindings)).
- `file_structure: BTreeMap<PathBuf, FileStructure>` — the headings and doc-comment blocks of the files whose row asks for a grounding unit finer than the file ([§AR-scanner.2.7](AR-scanner.md#27-grounding-units-per-file)), empty where no row does.

### 3.1 Inline citation sites

A citation inside a source **inline comment** block also carries that block as an inline citation site ([§FS-inline-citation-style.1](../functional-spec/FS-inline-citation-style.md#1-scope)): its first and last line, the character width of its widest line ([§FS-inline-citation-style.2.3](../functional-spec/FS-inline-citation-style.md#23-counting-lines-and-columns)), whether it carries a note, and the ascending list of the lines this run will *report* as failing the configured `[reference] inline_note_layout` ([§FS-inline-citation-style.3.3](../functional-spec/FS-inline-citation-style.md#33-inline_note_layout--where-the-citations-sit)). Every citation in one block carries the same site. A **doc comment** block carries no site at all — it is not one ([§FS-inline-citation-style.1.1](../functional-spec/FS-inline-citation-style.md#11-doc-comments-are-not-sites)) — so its citations are recorded the way a declaring block's are, with no site for the checker's style, budget, and layout rules to reach; [§AR-scanner.4.2](AR-scanner.md#42-doc-comment-or-inline-comment) is the classifier that decides which kind a block is.

### 3.2 The reported layout lines

That line list is what lets the checker report a per-line deviation without re-reading the file, and it is a record of the configured verdict rather than a survey of the tree: it holds the lines rule 1 judges, not every line carrying a citation, and it is empty — with no line classified — at the default `inline_note_layout = "any"`, under `inline_style = "citation-only"`, and at `inline_note_layout_check = "off"`, where the verdicts would reach no channel ([§FS-inline-citation-style.4.4](../functional-spec/FS-inline-citation-style.md#44-warnings-and-errors--opt-in-layout-deviations)). So a project that configures no layout, or configures one without gating it, tokenizes no line and classifies no line on the field's account, and pays not even a per-block memo — that allocation belongs to the second reader and is made only where one exists. A future consumer wanting deviations from an ungated tree is asking a question this field does not answer.

### 3.3 Merge order

When file scanning runs in parallel, each per-file result is merged as though the sorted file list had been scanned sequentially: declarations for duplicate IDs keep path order, citations keep path/line/column order, and `scanned_files` keeps the sorted file order. Workspace scans use the same per-file rule with the workspace target list already loaded, so `§<alias>/<ID>` citations are still parsed during that one read of the citing file rather than by a second pass.

## 4. Inline declarations in language doc-comments

The scanner is designed so that an inline declaration — most commonly an `AR-NNN-<slug>` for an architectural spec — can live inside the **class, method, module, or package doc-comment** of any major language — *inline* meaning in source, in a doc-comment, which is never an *inline comment* in [§FS-inline-citation-style.1.1](../functional-spec/FS-inline-citation-style.md#11-doc-comments-are-not-sites)'s sense. This makes class-level documentation a first-class place to put architecture specs: the spec body sits with the code it describes, and a stub under `docs/architecture/` points at it through a single-line H1 of the form `# <ID>: [<path>](<path>)`. The scanner recognizes the doc-comment forms of [§AR-scanner.4.3](AR-scanner.md#43-the-recognized-doc-comment-forms), strips each comment line to the content the author meant ([§AR-scanner.4.4](AR-scanner.md#44-comment-lines-are-normalized-before-detection)), and then runs its ordinary detection on that content.

### 4.1 Ruby and Python edge cases

- **Ruby** uses `#` as the comment marker. The declaration itself starts after that marker ([§AR-scanner.4.4](AR-scanner.md#44-comment-lines-are-normalized-before-detection)), so the canonical Ruby form is `# AR-<event-bus>`, not a markdown heading inside the comment.
- **Python** docstrings are not comments but string literals (`""" … """`). The scanner has a small docstring mode for `.py` ([§AR-scanner.4.4](AR-scanner.md#44-comment-lines-are-normalized-before-detection) gives its delimiter rules): when a triple-quoted string opens, lines inside it are scanned the same way as comment continuation lines until the matching close. This lets a Python class or module docstring be a fully-featured spec home.

### 4.2 Doc comment or inline comment

The block classifiers of [§AR-scanner.4.3](AR-scanner.md#43-the-recognized-doc-comment-forms) and [§AR-scanner.4.4](AR-scanner.md#44-comment-lines-are-normalized-before-detection) answer a second question, for the inline citation sites of [§AR-scanner.3.1](AR-scanner.md#31-inline-citation-sites): is this block a **doc comment** — documentation of the definition below it, or of the file — or an **inline comment**? Only the second is a site ([§FS-inline-citation-style.1.1](../functional-spec/FS-inline-citation-style.md#11-doc-comments-are-not-sites)). The classifier lives in `crates/grund-core/src/grammar/comment_block.rs`, beside `CommentBlockKind` and the block-boundary helpers the declaration pass already shares, and it is asked once per block — and only for a block that carries a citation, which is where `inline_citation_sites` already has the block in hand.

#### 4.2.1 One rule per extension

Which rule applies is keyed on the **file extension** and resolved once per file. There are three, and [§FS-inline-citation-style.1.1](../functional-spec/FS-inline-citation-style.md#11-doc-comments-are-not-sites) is their contract: the table of which extension takes which rule and what its test looks for, and the definition-starter matching rule.

- **Marker.** The language spells documentation with a marker of its own, so the marker is the answer. The test reads the run's marker for a line comment and the opening line for a block comment.
- **Position.** The language spells documentation like any other comment, so position is the answer. The block is a doc comment when the line immediately below it — no blank line between — is a **definition-starter** for that language, or when it is the file's **leading block**, which is how a position language spells a module doc.
- **None.** Every other extension has no doc-comment notion, so every block in it is inline. That is the behavior of every release before the rule, so no tree gains a finding from this classifier.

#### 4.2.2 Recognition, not parsing

The contract also names the corners the recognizer accepts rather than repairs. Nothing here parses the host language: the marker test is a prefix comparison and the position test is one look at one line ([§FS-non-goals.3](../functional-spec/FS-non-goals.md#3-code-ast-parsing)), the same discipline [§AR-scanner.5](AR-scanner.md#5-why-regex-not-a-parser) states for the recognizer as a whole, and a starter set widens without a `grund_config_version` bump.

#### 4.2.3 The planned declaration recognizer reuses it

[§RM-doc-comment-declarations](../roadmap.md#rm-doc-comment-declarations-declarations-only-in-classmethod-doc-comments) plans the same marker/position split for the **declaration** recognizer — a code declaration only inside a doc comment that documents the following definition. It reuses this classifier rather than growing a second one, so the two gates cannot come to disagree about what documentation is.

### 4.3 The recognized doc-comment forms

The comment forms a declaration is recognized in (matched as comment prefixes preceding the heading line); which blocks of them are doc comments is [§FS-inline-citation-style.1.1.2](../functional-spec/FS-inline-citation-style.md#112-languages)'s question, not this table's:

| Language(s)              | Declaration-hosting comment form                  | How the regex sees it                |
|--------------------------|---------------------------------------------------|--------------------------------------|
| Java, Kotlin, Scala      | `/** … */` (Javadoc / KDoc / Scaladoc)            | `/*` opens; ` * ` on continuation    |
| C, C++                   | `/** … */` (Doxygen) or `/// …`                   | `/*` or `//` (covers `///`)          |
| C#                       | `/// <summary>…</summary>` (XML doc)              | `//` (covers `///`)                  |
| Rust                     | `/// …` outer, `//! …` inner, `/** … */` block    | `//` covers `///` and `//!`; `/*` for block |
| TypeScript, JavaScript   | `/** … */` (JSDoc / TSDoc)                        | `/*` opens; ` * ` on continuation    |
| Go                       | `// …` block immediately above the declaration    | `//`                                 |
| Swift                    | `/// …` or `/** … */`                             | `//` or `/*`                          |
| PHP                      | `/** … */` (PHPDoc)                               | `/*` opens; ` * ` on continuation    |
| Ruby                     | `# …` lines (RDoc / YARD)                         | `#` (see note 4.1)                    |
| Python                   | `""" … """` or `''' … '''` docstring                 | special-cased (see note 4.1)         |
| Lisp, Scheme, Clojure    | `; …` line comments                               | `;`                                  |
| SQL, Haskell, Lua, Ada   | `-- …` line comments                              | `--`                                 |

This table documents the comment *conventions* for the languages `grund` is built to serve. It is not the only gate: the file extension must be in `[scan] extensions` and the marker must be in `[scan] comment_prefixes` ([§FS-config.3.5](../functional-spec/FS-config.md#35-scan--what-gets-walked)). The defaults contain both halves for every row above and also recognize bare `*` / `/*` block-comment lines. A language not in the table still works when the repository configures both its extension and its comment marker.

### 4.4 Comment lines are normalized before detection

Before declaration, section, or citation detection runs on a source file, the scanner normalizes each eligible comment/docstring line to the content the author meant:

- `//`, `///`, and `//!` line comments strip the full leading comment marker and one following space when present. Therefore `/// AR-001-router: Router`, `//! AR-001-router: Router`, and `// AR-001-router: Router` all expose the same declaration content: `AR-001-router: Router`.
- `#`, `;`, and `--` line comments strip that marker and one following space when present. Therefore Python/Ruby `# AR-001-router: Router` exposes `AR-001-router: Router`; a bare source line `AR-001-router: Router` is not a declaration outside a Python docstring, because it has no comment marker.
- Block comments strip the opener (`/*` or `/**`) and closer (`*/`) when they appear on their own content lines. Continuation lines strip one optional leading `*` plus one following space when present. Therefore ` * AR-001-router: Router` exposes `AR-001-router: Router`.
- Python triple-quoted docstrings in `.py` files enter docstring mode for both `"""` and `'''`. Delimiter-only opening and closing lines are not content; delimiter lines that also contain prose are scanned after stripping the delimiter on that side, so `"""Uses §FS-001-router."""` and an indented multi-line docstring body are both scanned as docstring content.
- With Python docstring scanning enabled, the same reader records the assigned-data spans of [§FS-check.1.1.3.1](../functional-spec/FS-check.md#1131-assigned-python-triple-quoted-data). It carries the delimiter across lines, excludes only raw columns inside the span, resumes at a same-line tail, and returns to neutral state before a later docstring. The comment-block classifier consumes that classification rather than treating the assigned opener or closer as a docstring boundary.
- The normalization is line-local and deterministic, with only delimiter and lexical-mode state carried between lines. It does not parse the host language beyond recognizing the comment/docstring and assigned-data forms above; after normalization, the same declaration, section, citation, value, and inline-site readers apply. The scanner, body and link resolvers, embedded-value pass, formatter, and editor transforms replay this one classification rather than maintaining consumer-specific Python parsers. Recorded source positions and data spans still point at original-file byte columns, not stripped content columns, so LSP ranges and diagnostics cover the token the user sees and writers can preserve assigned data byte-for-byte.

### 4.5 Forms the default settings must recognize

The following inline declarations are all required to be recognized under the default scan settings:

```rust
/// AR-001-router: Router
/// Routes requests by path.

//! AR-002-module: Module architecture

/**
 * AR-003-block: Block comment spec
 * ## 1. Contract
 */
```

```go
// AR-004-handler: Handler
// Handles HTTP requests.
```

```python
"""
AR-005-service: Service
## 1. Contract
"""
class Service:
    pass
```

```ruby
# AR-006-job: Job
# Runs background work.
```

## 5. Why regex, not a parser

Specs live in markdown *and* in source-file doc-comments across half a dozen languages. A real parser per language would be far more code and far slower than a single line-oriented regex pass. The scheme is deliberately designed to be regex-recognizable: the heading shape is unambiguous and the citation shape is anchored on word boundaries.

The trade-off: we cannot reason about the surrounding code structure. We do not need to — IDs are syntactic, not semantic. The link in the stub heading is the only structural pointer between a stub and the code that hosts the inline spec, and it is verified by [§AR-checker.2.5](../../crates/grund-core/src/checker/report.rs).

The marker character recognized in citations follows [§DF-reference-marker](../decisions/functional/DF-reference-marker.md#df-reference-marker-use--as-the-reference-marker-with--as-the-typing-trigger); the regex shape changes when the marker is reconfigured per [§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable).

## 6. E2E case declarations

`E2E` is a **configured** kind — it left the default `[[kinds]]` set in grund 0.12.0 ([§FS-config.3.4.4](../functional-spec/FS-config.md#344-the-default-kinds)) — and everything below follows the configured `E2E` home. A config with no citable `E2E` kind runs none of this pass: no case declarations, and no fixture-tree pruning, so a nested case repo under such a tree is ordinary content the walk reads. The `E2E` kind is the one kind not declared by a heading line: a case directory declares it ([§AR-scanner.6.1](AR-scanner.md#61-a-case-directory-declares-the-id)).

### 6.1 A case directory declares the ID

An `E2E` declaration is a **case directory** directly under the `E2E` kind's `[[kinds]] folder` (conventionally `e2e/cases`) that contains an `expected.exit` file, the minimal marker of a real case. The directory's name is the declared ID with the leading `{kind}` placeholder and its following literal stripped — under the default `[id] format = "{kind}-{number}-{slug}"`, a directory `007-login` declares `E2E-007-login`; under `{kind}-{slug}`, `login` declares `E2E-<login>`; under `{kind}-{number}`, `007` declares `E2E-007`. The directory name must match the format with the kind portion removed; directories that do not (e.g. `.gitkeep`, or a folder with no `expected.exit`) are skipped, so `e2e/cases/` itself never becomes a declaration.

### 6.2 What a case declaration records

The `Declaration` recorded for a case carries the directory path with `line = 1`, an empty section set (the fixture file set is not a numbered-heading tree, so any section-bearing citation of an `E2E` ID — a `.2` suffix and so on — is a missing-section error per [§AR-checker.2.4](../../crates/grund-core/src/checker/report.rs)), and the deterministic, sorted list of the case's fixture files plus the invocation (`command.args` contents, or the implicit `grund check` when absent) and the expected exit code — this is the "body" [§FS-show.2.4](../functional-spec/FS-show.md#24-e2e-cases) prints. E2E declarations are never stubs, are never hosted in code, and are not reported as unused when no spec cites them.

### 6.3 `spec.refs` is direction evidence, not citations

The case manifest also records non-empty `spec.refs` lines as cited-kind evidence for E2E citation-direction obligations ([§FS-config.3.9.1](../functional-spec/FS-config.md#391-levels)); these manifest references do not enter the ordinary citation stream.

### 6.4 The walk stops at a case directory

The ordinary file walk treats each direct case directory as an E2E manifest boundary, not as repo content to scan. A root scan over `e2e/` or `e2e/cases/` still registers the case declaration through the E2E manifest pass, but it does not read the nested fixture repo under that case; an explicit path inside the fixture repo remains scannable.

### 6.5 Citations of a case resolve like any other

Citations of an `E2E` ID resolve like any other: an `E2E-<name>` cite from a spec ("proven by …") is a dangling-ref error ([§AR-checker.2.3](../../crates/grund-core/src/checker/report.rs)) when the case directory under the configured `E2E` home does not exist; `e2e/cases/<name>` is the example produced by the conventional configuration that selects that folder.

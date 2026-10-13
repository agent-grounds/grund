# FS-scan: what a path argument means, which projects a run loads, and which files it reads

What a run is handed and what it reads, defined once for every command that scans a tree: what each command's `<path>` argument means, which projects a run loads and so whose kinds and declarations it reads, which files a walk reads, which path names a file it read, and which text a file has. The `[scan]` keys and their defaults are [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)'s, and which project owns a directory is [§FS-workspace.6.2](FS-workspace.md#62-the-boundary-is-mutual)'s; this page cites both rather than restating them. Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration), whose verdicts are only as good as the files a run read, and [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), because one point answers what a run reads.

## terms: Terms

Leans on [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace, member).

- **kind home** — The `folder` or `file` a `[[kinds]]` row names ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)): the place its declarations live.
- **scan root** — Where a walk starts: an `include` entry, a kind home, or an explicit path. Which rules it outruns is [§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent).
- **resolution scope**, **report scope** — The two scopes `check` tells apart for an explicit path, defined at [§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports).
- **narrowed scan** — One scan of the enclosing project below its config root, with no workspace loaded ([§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-no-workspace)).
- **buffer** — The text an editor holds open for a file, saved or not, which a run reads as that file ([§FS-scan.text.buffer](FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write)).

## path: What a path argument means

Every command that takes a `<path>` discovers its configuration from it ([§FS-scan.path.discovery](FS-scan.md#pathdiscovery-a-path-is-where-discovery-starts)). What the path then bounds is each command's own today: `check` reads its project's ordinary scope as well as the path ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)), and `cover`, `list`, `show`, `refs` and `id` each give it the meaning their own leaf below states. The commands do not yet agree, and the disagreement is listed, not settled, at [§REQ-one-answer.differences.scope](../requirements/REQ-one-answer.md#differencesscope-scope). Until grund 0.19.0 settles it for `list`, `show` and `id`, each of them says so wherever the narrowing changes its answer ([§FS-scan.path.warning](FS-scan.md#pathwarning-where-the-path-changes-an-answer-the-run-says-so)).

### path.discovery: A path is where discovery starts

An explicit `<path>` argument (e.g. `grund list apps/api`, `grund refs FS-x apps/api`, `grund complete ids --path apps/api`) behaves as if the command were invoked from that path: a `<path>` inside a member is member-scoped, not workspace-aggregate, even when a workspace exists above it ([§FS-scan.projects.discovery](FS-scan.md#projectsdiscovery-the-nearest-config-decides)). `cover`, whose `<path>` bounds a scan, runs one narrowed scan below the config root instead ([§FS-scan.path.cover](FS-scan.md#pathcover-cover-runs-one-narrowed-scan)).

### path.check: `check` reads more than it reports

A run given an explicit path reads the declarations and the citations its project's ordinary run reads, and reports findings only for the files at or under the path — or, where a finding spans several sites, for a finding naming one of them ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)). Two scopes, and telling them apart is the whole rule. The **resolution scope** is the set of files the run reads so that citations resolve and citation edges count: the enclosing project's ordinary scope — `[scan] include` plus every walked kind home ([§FS-scan.scope.include](FS-scan.md#scopeinclude-include-is-scanned-from-the-config-root), [§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)) — **union the path itself**, because a path may lie outside `include` and must still be read. The **report scope** is exactly the path. Every rule runs over the resolution scope, and a diagnostic anchored at a file outside the report scope is dropped before the report is written, unless it also names a site inside it ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)). So for `check` a path narrows the report and not what the run resolves against; which verdicts that moves is [§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)'s to say. Decided in [§DF-path-scope-resolves-project-wide](../decisions/functional/DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path).

### path.cover: `cover` runs one narrowed scan

Of the commands [§FS-workspace.8](FS-workspace.md#8-other-commands) covers, `cover` is the one whose `<path>` bounds a **scan** rather than choosing which config answers ([§FS-cover.1](FS-cover.md#1-inputs)), so it draws the aggregate/narrow line where `grund check` draws it and not where `list` does: a scope narrower than the config root is one narrowed scan of the enclosing project, no workspace loaded and no `project` field, the way `grund check <dir>` already behaves ([§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-no-workspace)). Widening `grund cover src/` back to every project would answer a question the caller did not ask, and an explicit path deliberately bypasses `[scan] include`, so the narrowing is the only thing that put those files in scope at all.

### path.warning: Where the path changes an answer, the run says so

`list`, `show` and `id` resolve within the path's tree. From grund 0.19.0 they resolve against the owning project — the one the path discovers ([§FS-scan.path.discovery](FS-scan.md#pathdiscovery-a-path-is-where-discovery-starts)), read as `check`'s resolution scope is ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)) — and narrow only what they report ([§DF-one-answer-rulings-run](../decisions/functional/DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held), ruling D1). Until then each keeps today's answer, stdout and exit code byte for byte under every `--format`, and adds one `warning:` line to stderr for each place where the answer computed against the owning project differs from it. Each command's leaf below says which places those are and names the site outside the path that moves each one. Every such line ends in the same clause:

```text
; from grund 0.19.0 a path narrows what is reported, not what resolves
```

#### path.warning.shape: A text line after everything else on stderr

The line is a CLI-level warning ([§FS-errors.2.2](FS-errors.md#22-cli-level-message)): plain text on stderr under every `--format`, written after every line the run already writes there, never on stdout and never moving the exit code. `<path>` in it is the path operand as the caller wrote it; every `<file>:<line>` is rendered against the base the command's own output uses ([§FS-errors.3.1](FS-errors.md#31-one-base-for-every-path-in-the-line)). Where several sites outside the path move the same answer, the line names the first in path, then line order.

#### path.warning.cost: Only a narrowed path reads more

Only a path narrower than the owning project's config root can change an answer, so only such a run reads beyond the path, and it reads the owning project once ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)). A run with no path, or with one at or above its config root, reads exactly what it read before and prints no such line.

#### path.warning.release: One named release, held by a test

The release the clause names is one constant shared by every such line and by [§FS-scan.projects.narrowed.alias](FS-scan.md#projectsnarrowedalias-a-refused-alias-the-workspace-root-resolves-says-so), and a unit test holds it above the running version ([§FS-distribution.4.2.1](FS-distribution.md#421-a-test-can-hold-only-the-pending-half)): the release guard's closed vocabulary does not read it ([§FS-distribution.4.2.3](FS-distribution.md#423-the-vocabulary-is-closed)). The decision and its release note are [§DF-path-narrowing-warns](../decisions/functional/DF-path-narrowing-warns.md#df-path-narrowing-warns-a-path-that-narrows-what-show-list-and-id-resolve-or-what-check--resolves-an-alias-against-says-so-for-one-release).

### path.list: `list` scans the path's tree

For `list`, `<path>` is the directory or file whose tree is scanned, `.` by default. Discovery is the same as every other subcommand (walk up to a `grund.toml`, else defaults — [§FS-config.1](FS-config.md#1-file-location-and-discovery)).

The places it warns about are the catalog entries this run prints, or counts under `--summary`, that would print differently, appear or vanish when the owning project is read: an entry under the path cited from outside it changes its JSON `refs` and its `--unused` selection, and one with another home outside the path gains the duplicate note ([§FS-list.2.6](FS-list.md#26-duplicate-declarations)). The line is one per such entry, in output order, and names the site outside the path that moves it — a citation of the entry, or its other home. For an entry `--unused` would drop, the citation is one that counts as use, never a kind's own index entry ([§FS-list.1.3](FS-list.md#13---unused)). The `--size` rows are such entries too ([§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements)): a row `--unused` would drop, or whose duplicate flag the other home sets, gives its entry the line, once however many of its coordinates print:

```text
warning: <ID> entry depends on <file>:<line>, outside <path>; from grund 0.19.0 a path narrows what is reported, not what resolves
```

A declaration that merely lies outside the path moves nothing: from grund 0.19.0 the path still bounds which entries are reported. So `list sub` over a `sub/` that declares nothing prints nothing and warns nothing, and a text `list` row whose citations lie outside the path warns nothing either, because the text row does not show them ([§FS-list.3.1](FS-list.md#31---format-text-default)).

### path.show: `show` resolves the ID in the path's tree

For `show`, `<path>` is the directory or file whose tree is scanned to resolve the ID. Defaults to `.`. Discovery is the same as every other subcommand (walk up to a `grund.toml`, else defaults — [§FS-config.1](FS-config.md#1-file-location-and-discovery)).

Two answers change when the owning project is read, and each gains its line after today's stderr. An ID not found within the path that the project declares outside it (today `ID not found`, stdout empty, exit `1`):

```text
warning: <ID> is declared at <file>:<line>, outside <path>; from grund 0.19.0 a path narrows what is reported, not what resolves
```

And an ID found within the path that the project also declares outside it (today the body, exit `0`; then an ambiguous ID, [§FS-show.2.2.1](FS-show.md#221-ambiguous-id)):

```text
warning: <ID> is also declared at <file>:<line>, outside <path>; from grund 0.19.0 a path narrows what is reported, not what resolves
```

A short form ([§FS-show.1.2](FS-show.md#12-the-number-only-shorthand)) that resolves to one ID within the path and to several in the project is ambiguous then too ([§FS-show.2.2.1.1](FS-show.md#2211-an-ambiguous-shorthand-names-its-candidates)), so it gains the second line, naming the first home of another match outside the path. One already ambiguous within the path lists more candidates in the project when the project adds a match the path does not hold, so it gains the second line too, naming the first outside home of such a match; one the project adds no match to gets no line. `<ID>` is the ID as the query wrote it, without its section. A miss the project does not resolve either moves nothing and gets no line. Under `--batch` each query gets its own line, in input order.

### path.refs: `refs` scans the path's tree

For `refs`, `<path>` is the directory or file whose tree is scanned, default `.`, discovered as by every other subcommand ([§FS-config.1](FS-config.md#1-file-location-and-discovery)).

### path.id: `id` numbers from the path's tree

For `id`, `<path>` is the directory whose tree is scanned for the next free number; it defaults to the current directory and is resolved as by every `grund` command ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)): config discovery walks up to `grund.toml`, else takes the defaults.

The answer changes when the owning project holds a higher number of the kind outside the path, so that `id` would propose another ID, or, under a number-less format ([§FS-id.4.1](FS-id.md#41-number-less-id-formats)), declares the proposed ID outside it, so that `id` would refuse it ([§FS-id.5](FS-id.md#5-collision-check)). Each case gains its line, naming the declaration that holds the number or the ID:

```text
warning: <ID> at <file>:<line>, outside <path>, holds the highest <KIND> number; from grund 0.19.0 a path narrows what is reported, not what resolves, and id proposes <proposal>
warning: <proposal> is already declared at <file>:<line>, outside <path>; from grund 0.19.0 a path narrows what is reported, not what resolves, and id refuses it
```

`<proposal>` is the ID `id` would print with the owning project read. Today's ID still goes to stdout and the exit stays `0`.

## projects: Which projects a run loads

Which projects a run loads decides whose kinds and declarations it reads. Which project owns a directory is [§FS-workspace.6.2](FS-workspace.md#62-the-boundary-is-mutual)'s, and this chapter does not restate it.

### projects.discovery: The nearest config decides

Discovery follows the same walk-up rule as `grund check` ([§FS-config.1](FS-config.md#1-file-location-and-discovery)): from the CWD (or from an explicit `<path>` argument, [§FS-scan.path.discovery](FS-scan.md#pathdiscovery-a-path-is-where-discovery-starts)), walk up to the nearest `grund.toml` in either discovery form. If the nearest config is a member's own config, the command runs member-local — qualified `<alias>/<ID>` cannot resolve, the same way `check` errors at the member scope ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)). A scope inside a member with no config of its own is member-local all the same, on [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)'s canonical defaults with the member directory as the config root; only outside every member does a nearest config at the workspace root run the command workspace-wide.

### projects.workspace-root: A run at the workspace root loads every project

`grund check` run at a workspace root checks the root project (unless `include_root = false`, [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)) and all configured members, aggregates findings, and prints the same explicit-channel `<path>:<line>: <channel>: <message>` shape as a normal check ([§FS-check.2.1](FS-check.md#21-report-format)). Paths are rendered relative to the workspace root when `[output] relative_paths = true`.

### projects.member: A member run loads the member

`grund check <member>` (or `grund check` invoked from inside a member tree) discovers the member's own config first and, for a member that declares no `[workspace]` block of its own, validates it as an independent project; one that does runs its own subtree ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)). What a qualified citation earns there is [§FS-workspace.5.1](FS-workspace.md#51-a-member-run)'s.

### projects.narrowed: A path below the config root loads no workspace

`grund check <dir>` with a `<dir>` narrower than the config root is one narrowed scan of the enclosing project — no workspace loaded, no `project` field.

#### projects.narrowed.alias: A refused alias the workspace root resolves says so

A narrowed scan loads no workspace, so a citation qualified by a project alias is refused with `unknown project alias <alias>` ([§FS-check.3.8](FS-check.md#38-cross-project-citation-failure)) even where the config root declares a `[workspace]` that resolves it. From grund 0.19.0 `check <dir>` resolves each project a run from `<dir>` would see ([§DF-one-answer-rulings-run](../decisions/functional/DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held), ruling D14). Until then, where the workspace the config root declares resolves the alias, the finding keeps its whole message as a verbatim prefix in every output shape and appends one clause ([§FS-errors.3](FS-errors.md#3-message-text)):

```text
unknown project alias <alias>; grund check from the workspace root resolves this alias, and from grund 0.19.0 grund check <path> will too
```

`<path>` is the path operand as the caller wrote it. Code, severity, location, the other findings and the exit code do not move. An alias the workspace root does not resolve either keeps today's message unchanged, and so does a member run ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)): a member directory still means a member run, from grund 0.19.0 as today.

### projects.kinds: Whose kinds a run reads

A run reads the kinds of the projects its configuration declares: its own config's `[[kinds]]` and, where that config declares a `[workspace]`, each member's ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)). It reads them from the configs alone, before any file is read, so a narrowed scan ([§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-no-workspace)), which loads no member's files, still reads the kinds its declared workspace configures and judges its `[citations]` targets against them ([§FS-config.3.9.5.2](FS-config.md#3952-a-pinned-target-is-judged-against-its-projects-kinds)). A rule object there is judged in the projects the run loads, so a namespaced one stays unverifiable, as [§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify) says. Every question a command asks about a kind name is answered from that one reading, never from a set the command builds for itself.

The question is always asked within a **scope**, a set of the projects the run read. Each command keeps the scope it documents, so two commands may accept different names; within one scope, a name has exactly one verdict, whichever command asks, except `code` where no row names the homeless kind, as [§REQ-one-answer.differences.identity](../requirements/REQ-one-answer.md#differencesidentity-identity) lists:

| Who asks | Scope |
|---|---|
| `list --kind`, `list --selector`, `list --size --selector` | the selected projects: every loaded project, or those `--project` names ([§FS-list.1.2](FS-list.md#12---project)) |
| a rule subject, on `check`, `check --rule` and in the editor | the project whose configuration reads the rule ([§FS-rule-refusals.sentences.named-sections](FS-rule-refusals.md#sentencesnamed-sections-a-subject-that-needs-named-sections-is-answered-with-one-they-make-valid)) |
| a rule object, or a `[citations]` target | the local project for a bare `KIND`, the aliased project for `alias/KIND`, every project for `*/KIND` ([§FS-config.3.9.5](FS-config.md#395-validation)) |
| `grund id <KIND>` | the project the run loads ([§FS-id.1.1](FS-id.md#11-unknown-and-non-citable-kinds)) |

The verdict is one of four:

1. **citable** — a project in scope configures the name as a citable kind;
2. **configured but not citable** — a project in scope configures it with `citable = false` ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)), and none in scope has it citable; the verdict carries that project's kind home, and its refusal is [§FS-errors.2.5.1](FS-errors.md#251-a-kind-that-declares-no-ids);
3. **unknown** — no project in scope configures it;
4. **unverifiable here** — the scope names a project alias and the run holds no workspace that could say what that project configures, as [§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify) defines it for a rule object.

Where two projects in one scope configure a name differently, citable wins, as [§FS-list.1.1](FS-list.md#11---kind) already says for `--kind`. The `known kinds:` line a refusal ends in names the citable kinds of the same scope, once each, in configuration order.

A kind's named sections are read under the project that configures it. A named chapter subject of `KIND` ([§FS-rules.2](FS-rules.md#2-subject-selectors)) is accepted where that project sets `[id] named_sections = true`, whatever the root or another loaded project sets, so `list --selector API.terms` at a workspace root answers whatever `grund api/API-a.terms` answers ([§FS-declarations.section.map](FS-declarations.md#sectionmap-the-section-map-is-the-declarations-body-span)). Where two projects in scope configure `KIND`, the subject is accepted when either has named sections on, and it matches only the declarations of a project that has them on.

`--unused` leaving out end-to-end cases, the kind titles a project lists, and where a non-citable kind's own files must cite are other questions, and stay their own points' ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)). The rulings this point carries out are [§DF-one-answer-rulings-grammar](../decisions/functional/DF-one-answer-rulings-grammar.md#df-one-answer-rulings-grammar-one-grammar-keeps-the-documented-refusal-bytes-and-order-and-one-kind-universe-keeps-each-commands-vocabulary)'s D18, and the routes it took [§DF-one-kind-verdict](../decisions/functional/DF-one-kind-verdict.md#df-one-kind-verdict-every-command-reads-one-kind-verdict-in-its-own-scope-and-refuses-a-rule-sentence-for-the-first-stage-it-fails).

## scope: Which files a run reads

`include` is the set of paths scanned from the config root ([§FS-scan.scope.include](FS-scan.md#scopeinclude-include-is-scanned-from-the-config-root)), beside every configured kind home ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)). `exclude` is the set of directory names skipped at any depth — one path component each, so an entry containing `/` is refused rather than accepted as a path ([§FS-config.3.5.16](FS-config.md#3516-an-exclude-entry-containing--is-refused)) — `extensions` filters which files are read ([§FS-scan.scope.extensions](FS-scan.md#scopeextensions-an-extension-makes-a-file-readable-not-declarable)), and `respect_gitignore` has the scan honor the ignore files as well ([§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files)). A hidden file is not read ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), and `include` is a scope, not a fence ([§FS-scan.scope.not-a-fence](FS-scan.md#scopenot-a-fence-include-is-a-scan-scope-not-a-fence)). A scan that ends up reading no files at all is reported, not silently passed ([§FS-check.2.2](FS-check.md#22-empty-scan)). In a loaded workspace a scan also stops at every directory another project owns ([§FS-workspace.6.2](FS-workspace.md#62-the-boundary-is-mutual)).

### scope.include: `include` is scanned from the config root

`include` is scanned **from the config root** — the directory the discovered `grund.toml` was found at ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), or, when no config was discovered, the current working directory (never a subdirectory that merely happened to be passed as `grund`'s path argument). So in a config-less repo `grund` (no path) and `grund check .` both scan `requirements.md`, `docs/`, `e2e/`, `src/` relative to the cwd; what an explicit path adds is each command's own ([§FS-scan.path](FS-scan.md#path-what-a-path-argument-means)). A plain parent-relative entry such as `../shared` intentionally names external content and is still scanned; [§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)'s project-root boundary applies only when a **directory symlink** carries traversal outside the project.

### scope.homes: Every configured kind home is scanned

**Every configured kind home is scanned, whether or not `include` names it** ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)). A home is the repository saying "declarations and citations live here", so `include` names the *extra* roots — `src`, `crates`, a `README.md` — rather than having to repeat the homes the `[[kinds]]` table already spelled. A home that does not exist scans as nothing and earns no finding, so a fresh repository whose default homes are not scaffolded yet stays silent. The one home that is not a scan root is the one the config says so about: `scan = false` ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) lists a place without scanning it.

#### scope.homes.why: Why every home is scanned

Scanning every home closes a trap that had nothing to do with non-citable kinds and everything to do with why one would be configured: a `folder` or `file` outside `include` was never scanned, so its declarations did not exist and its citations were **invisible rather than dangling** — no resolution, no finding, nothing to notice. A kind whose entire content is "this directory matters" would have fallen into it on its first line of config. **Upgrade note:** a repository that had a home outside `include` starts seeing that home's findings; they were always true of the tree, and the run was simply not reading it.

### scope.roots: A scan root outruns every rule about descent

A home is a **scan root**, and no scan root is pruned by `exclude`, an ignore file, or the hidden-directory rule ([AR-scanner.1](../architecture/AR-scanner.md#1-tree-walk)) — so a home the repository also excludes is read, while everything *below* it is filtered as usual. That is the honest reading of a config that says both: the `[[kinds]]` entry names the directory, and `exclude` was written about descendants. All three are rules about a descent, which is why a root outruns them. Two rules are about a file's own name, and a root does not outrun either: the hidden-file rule ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)) and the `extensions` list ([§FS-scan.scope.extensions.root](FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so)). A file root the list refuses is not read, and `check` names it rather than passing over it.

### scope.not-a-fence: `include` is a scan scope, not a fence

A citation in a file that neither `include` nor a kind home brings into the scan is invisible rather than merely unchecked ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), so `grund check --full` scans the whole config root past this key. The flag cancels `include` and, with it, the `scan = false` prune of [§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned), and nothing else: every other rule of this chapter applies to that scan unchanged ([§FS-check.1.3.1](FS-check.md#131-the-scan-covers-the-whole-config-root)), and what it reports outside the default scope is [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)'s to say.

### scope.hidden: A hidden file is not read, and the rule is not about descent

The scan skips a hidden directory by not descending into it; it skips a file whose own name begins with `.` by the name that file wears, before `extensions` is consulted at all ([AR-scanner.1](../architecture/AR-scanner.md#1-tree-walk)). So `docs/.notes.md` is not scanned though `md` is listed, and a citation inside it neither resolves nor dangles — it is invisible the way one outside `include` is, and `grund check --full` does not reach it either ([§FS-scan.scope.not-a-fence](FS-scan.md#scopenot-a-fence-include-is-a-scan-scope-not-a-fence)). Being a rule about a name rather than about a descent, it also reaches a scan **root**, the one exception to [§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent): a `file` home whose name is hidden, like an `include` entry naming one, is not read as a root. It is a blind spot the repository can see and plan around ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)): what decides is the file's own name, so a document that must be checked is one not named as a dotfile, and a run handed such a file whose extension `extensions` *does* allow says so rather than blaming the list ([§FS-check.2.2](FS-check.md#22-empty-scan)).

### scope.extensions: An extension makes a file readable, not declarable

Listing an extension makes a file *readable*, not declarable. A prose markup format other than Markdown — AsciiDoc, reStructuredText, LaTeX — has its citations checked as soon as its extension appears in `extensions`, because the citation grammar is format-agnostic; its native heading syntax still declares nothing, since a declaration is a `#`-prefixed heading or a comment-prefixed line. Whether those formats should be declaration homes of their own is an open discussion ([§DISC-markup-format-declarations](../discussions/proposals/2026-05-25-markup-format-declarations.md#disc-markup-format-declarations-declarations-in-asciidoc-restructuredtext-latex-and-similar-markup-document-formats)), not a configured behavior.

#### scope.extensions.root: A file root whose extension is unlisted is not read, and `check` says so

A scan root that is a single file — a `[[kinds]]` `file` home, a version-2 `files` entry, or an `[scan] include` entry naming a file — is read only if its extension is one `extensions` lists, or in a version-2 file one a language of `[schema.sources] languages` reads. A file with no extension at all, such as `NOTES`, has none the list can hold, so it is refused by the same rule. The list is not widened for a root: an unlisted extension has no comment prefixes to read the file with, and a row may name a file only as a place in the Project map. What the root earns instead is a report: every `grund check` that has the root in its configured scope prints the warning of [§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read), located at the row that made the file a root, so a configuration that names a file to read and a list that keeps it out never passes as `success` ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)).

Four such files earn nothing. A row marked `scan = false` is not a root ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)), a home that does not exist scans as nothing, and a hidden-named file is the declared blind spot of [§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent), whose rule is asked first. A value row's JSON home is read too, by the value pass rather than the walk ([§FS-values.2.2](FS-values.md#22-json-declarations-from-the-kind-home)). A directory root is filtered file by file, which is what `extensions` means, so it earns nothing either. The other commands that scan — `list`, `refs`, `show`, `cover`, `fmt` — read the same files and say nothing about the root: the fact is about the coverage of `check`'s verdict ([§DF-unread-scan-root](../decisions/functional/DF-unread-scan-root.md#df-unread-scan-root-a-configured-file-root-whose-extension-is-unlisted-is-named-not-read)).

### scope.ignore-files: `respect_gitignore` — the ignore files

`respect_gitignore` (default `true`) makes the scanner honor every form of ignore file the `ignore` crate recognizes — `.gitignore` at any depth, `.git/info/exclude`, the global `core.excludesFile`, and `.ignore` files. Set to `false` only when you genuinely need to scan ignored paths. The directory-level `exclude` list is applied **in addition** to ignore-file rules, never instead of them. See [AR-scanner.1.1](../architecture/AR-scanner.md#11-respecting-gitignore-and-friends).

### scope.links: A symlink in the tree is followed

A **file symlink** inside a scanned tree is followed wherever its target resolves. A **directory symlink** is followed only while its canonical target remains inside the independently checked project's canonical root; when the target is outside that root, the directory is pruned and no file below it is scanned. The same gate applies when a configured or explicit scan root is itself a directory symlink. Decided in [§DF-symlink-scan](../decisions/functional/DF-symlink-scan.md#df-symlink-scan-a-symlink-in-the-scanned-tree-is-followed-and-the-report-names-the-link).

This is a boundary on link traversal, not on scan paths generally: a non-symlink parent-relative `include` may still name external content. Nor does resolving the project's own config root make it an outward link: when the repository itself was reached through a symlink, its canonical root is the fence and its tree remains readable. In-root directory links keep their in-tree spelling and are followed as before. A loaded workspace adds the stronger ownership boundary of [§FS-workspace.6](FS-workspace.md#6-nested-project-boundary), so a link into another loaded project is pruned even when that project is physically inside the current project's root.

#### scope.links.names: The directory rules apply under the link name

The directory rules of [§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads) still apply to a followed directory under its **link** name, so `docs/node_modules -> ../../node_modules` is excluded exactly as a real directory of that name would be. The two boundaries that are *not* name rules are another project's root, which a link may not carry a scan across in any direction ([§FS-workspace.6](FS-workspace.md#6-nested-project-boundary)), and an E2E case directory, whose fixture tree stays out of the host scan however a link reaches it ([§FS-config.3.4.4.3](FS-config.md#3443-e2e-is-configured-not-a-default)).

#### scope.links.unresolvable: A link the scan cannot resolve is reported, and not scanned into

A link the scan cannot resolve is not a silent skip: a broken target, or a loop such as `docs/self -> .`, is the per-file scan failure of [§FS-check.2](FS-check.md#2-outputs) — reported at the link's own path, the scan continuing past it, the run exiting `2` ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)). Past it, and never *into* it: a loop is pruned where it is met, including the kind whose target reaches back over the scan root itself (`docs/up -> ..`), so the run never reports findings out of a tree it has just called unreadable and `include` still bounds what was read.

#### scope.links.owed: Which unresolvable links are owed a report

That report is owed only where the scan would otherwise have read through the link, judged by the same rules as any other entry: the ignore files for both kinds, and `extensions` as well for a broken link, which names a file where a loop names a directory. So a dangling `docs/logo.png -> nowhere`, and a link of either kind that `.gitignore` covers, stay silent — the scan was never going to read them.

#### scope.links.blind-spot: A broken link with no extension is a declared blind spot

A broken link with **no extension at all** is silent for the same reason and is worth naming, because it is the one case where that answer can be wrong: `docs/shared -> ../nonexistent-dir` would have been a directory to descend into had it resolved. Nothing on disk distinguishes it from `bin/tool -> nowhere`, since the target does not exist and only the target could have said which it was, and reporting every extensionless dangling link is the noise this gate exists to prevent. That is a declared, bounded blind spot ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)) and not a silent one: a link you need scanned is one you need to resolve.

## identity: Which path names a file a run read

A file a run reaches through a link is named by the path the run met, and a physical file is read once.

### identity.link-path: A finding names the in-tree link path

Every finding from a link met **inside a scanned tree** is reported at the in-tree link path, never the target's: that is the path a reader can act on, and it is what keeps `relative_paths` output ([§FS-config.3.6](FS-config.md#36-output--report-format)) and the additivity rule of [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full) meaningful. An explicit path argument follows the same reporting rule: resolving `grund check docs/beta.md` identifies and reads the target, but every text and JSON report names `docs/beta.md` as reached through the configured CLI base. This remains true when the target resolves outside the config root — the in-tree link is the bounded, actionable spelling and the external physical path never becomes report output.

### identity.root-path: The scan root keeps the path the run was handed

The same rule reaches the **scan root itself**: a repository whose own path is reached through a link — a symlinked `~/work`, macOS resolving `/var` to `/private/var` — is scanned and reported under the path the run was handed, never the physical one it resolves to. Resolving a root is how a run recognizes that a scope *is* the config root; it is not a decision about what the report calls it, and a finding spelled physically is one `relative_paths` ([§FS-config.3.6](FS-config.md#36-output--report-format)) cannot render and no reader of that repository ever wrote.

### identity.read-once: One physical file is read once

One **physical** file is read once however many spellings reach it; when two do, the surviving spelling is the earlier root's, and within a single root the lexicographically first one ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full), [§FS-errors.4](FS-errors.md#4-determinism)).

### identity.buffers: A buffer is the file at its physical location

An editor buffer is the file at the physical location its path resolves to, under whatever spelling the editor opened it and whatever spelling a stub or the walk reaches that file by. Where `alias -> notes`, a buffer the editor opened as `notes/a.md` is the text of `alias/a.md`, and one opened as `alias/a.md` is the text of `notes/a.md`, so a stub that writes `../alias/a.md` reads the buffer either way, as it would read the saved file ([§FS-scan.text.buffer](FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write)). Identity decides which text a spelling reads, and nothing else: the name a stub wrote is still judged as written ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)), a finding still names the in-tree link path ([§FS-scan.identity.link-path](FS-scan.md#identitylink-path-a-finding-names-the-in-tree-link-path)), and the file is read once ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once)). Decided in [§DF-a-buffer-is-the-file](../decisions/functional/DF-a-buffer-is-the-file.md#df-a-buffer-is-the-file-an-editor-buffer-is-the-file-it-names-under-every-spelling-and-with-no-file-saved-behind-it).

## text: Which text a file has

Every reader in a run reads one text per file, the one a save would write. The command line holds no buffers, so for it that text is the disk; an editor hands its run the buffers it holds open, and each one is the file it names ([§FS-scan.identity.buffers](FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location)).

### text.buffer: A file's text is what a save would write

A file's text is what saving it would write: the editor's open buffer where there is one, and the disk only where there is none. That holds for every reader a run has, the walk's scan of each file, a stub's target ([§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it)) and the body `show` slices out of it ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)) among them, so an unsaved edit moves every answer at once, exactly as saving it would ([§REQ-one-answer.text.targets](../requirements/REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would)). The readers that do not read the buffer yet are the rows [§REQ-one-answer.differences.text](../requirements/REQ-one-answer.md#differencestext-text) lists; until each row closes, that reader reads the saved file in the editor too ([§DF-four-editor-checks-read-saved-text](../decisions/functional/DF-four-editor-checks-read-saved-text.md#df-four-editor-checks-read-saved-text-four-editor-checks-read-the-saved-file-for-one-more-release)).

### text.buffer-only: A buffer with no file behind it is a file

A buffer with no file on disk behind it, a new file the editor has not saved, is a file of the run. It exists for a stub, so a stub to it is never `stub link target missing` and is judged on the buffer's text ([§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it)). The walk reads it or skips it by the rules of [§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads) exactly as it would the saved file: `include` and the kind homes bring it in, and a hidden name, an unlisted extension, `exclude`, an ignore file of any form [§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files) names, a `scan = false` home and another project's root keep it out. Where the editor holds a buffer for a file the walk skips, the walk's rule wins: the buffer is the file, and the file is not read. A stub's target outside `include` is read as if it were scanned ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)), a buffer as much as a saved file, and only by the name and extension rules of [§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing).

## checks: Checks

### checks.unread-scan-root: A configured file root is not read

A scan root that is a single file — a `[[kinds]]` `file` home, a version-2 `files` entry, or an `[scan] include` entry naming a file — whose extension the extension list leaves out ([§FS-scan.scope.extensions.root](FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so)). Nothing in the file was read: no declaration in it reached a catalog and no citation in it was resolved or reported, so the configuration names a file to check and the run checks none of it. Saying so is this finding's whole job ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)).

It is one located warning per unread root, at the line of the row that made the file a root ([§FS-scan.checks.unread-scan-root.location](FS-scan.md#checksunread-scan-rootlocation-one-warning-per-unread-root-at-its-rows-line)), in the exact bytes of [§FS-scan.checks.unread-scan-root.message](FS-scan.md#checksunread-scan-rootmessage-the-message). Some roots earn nothing ([§FS-scan.checks.unread-scan-root.silent](FS-scan.md#checksunread-scan-rootsilent-what-stays-silent)). It is printed on stdout and moves no exit code ([§FS-scan.checks.unread-scan-root.channel](FS-scan.md#checksunread-scan-rootchannel-a-located-finding-on-stdout-and-the-exit-code-does-not-move)), it is asked of the configured scope by `check` alone ([§FS-scan.checks.unread-scan-root.scope](FS-scan.md#checksunread-scan-rootscope-asked-of-the-configured-scope-by-check-alone)), its JSON form is [§FS-scan.checks.unread-scan-root.json](FS-scan.md#checksunread-scan-rootjson-its-json-form-is-on-stdout), and it stands in place of the empty-scan caution ([§FS-scan.checks.unread-scan-root.empty-scan](FS-scan.md#checksunread-scan-rootempty-scan-it-stands-in-place-of-the-empty-scan-caution)).

- **Code:** `unread-scan-root` ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)).

#### checks.unread-scan-root.location: One warning per unread root, at its row's line

One warning per unread root, located in the `grund.toml` that configured it, at the line of the key that named the file: the `file` key of its `[[kinds]]` row, the `files` key of its version-2 row, or the `[scan] include` key. Two entries of one `include` array share that line and sort by message ([§FS-errors.4](FS-errors.md#4-determinism)). The path is spelled as every located finding in that config spells it ([§FS-cli.3.4](FS-cli.md#34---path-base--where-report-paths-are-spelled-from)).

#### checks.unread-scan-root.message: The message

The message is `<row> is not read — <reason>, so nothing in it is checked. <fix>.` The row names the file as the configuration wrote it:

| Root | `<row>` |
|---|---|
| a `[[kinds]]` `file` home | ``[[kinds]] <KIND> home `<file>` `` |
| an `[scan] include` entry | ``[scan] include entry `<entry>` `` |
| a version-2 `files` entry | ``[schema.kinds.<KIND>] file `<file>` `` |

The reason names the extension as the file name spells it, the part after the last `.`, or says there is none:

| Configuration | The file has an extension | The file has none |
|---|---|---|
| version 1 | `` `<ext>` is not in [scan] extensions`` | `it has no extension, and [scan] extensions reads a file only by its extension` |
| version 2 | ``no language in [schema.sources] languages reads `.<ext>` `` | `it has no extension, and [schema.sources] languages reads a file only by its extension` |

The fix is the one edit that makes the row say what it means. Listing the extension is offered only where there is one to list, so a file with none is never told to add it:

| Root | The file has an extension | The file has none |
|---|---|---|
| a version-1 `[[kinds]]` home | `Add "<ext>" to [scan] extensions, or give the file an extension the list holds` | `Rename the file to an extension the list holds` |
| an `[scan] include` entry | `Add "<ext>" to [scan] extensions, or drop the entry` | `Rename the file to an extension the list holds, or drop the entry` |
| a version-2 `files` entry | `Give the file an extension a listed language reads` | `Give the file an extension a listed language reads` |

A home on a row with `citable = false` may name the file only as a place in the Project map, so its fix also offers the key that says so ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)): the last `, or` of the version-1 fix becomes `,`, and the fix ends ``, or set `scan = false` on the row if it only lists a place``. A version-2 row's single fix gains the same ending. So, for one root of each shape:

```
grund.toml:10: warning: [[kinds]] TXT home `notes.txt` is not read — `txt` is not in [scan] extensions, so nothing in it is checked. Add "txt" to [scan] extensions, or give the file an extension the list holds.
grund.toml:9: warning: [scan] include entry `extra.txt` is not read — `txt` is not in [scan] extensions, so nothing in it is checked. Add "txt" to [scan] extensions, or drop the entry.
grund.toml:7: warning: [[kinds]] NOTES home `NOTES` is not read — it has no extension, and [scan] extensions reads a file only by its extension, so nothing in it is checked. Rename the file to an extension the list holds, or set `scan = false` on the row if it only lists a place.
grund.toml:8: warning: [schema.kinds.TXT] file `notes.txt` is not read — no language in [schema.sources] languages reads `.txt`, so nothing in it is checked. Give the file an extension a listed language reads.
```

#### checks.unread-scan-root.silent: What stays silent

A row marked `scan = false` is not a root and earns nothing ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)), and neither does a home or entry that does not exist ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)). A hidden-named file is the declared blind spot of [§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent): the hidden rule is asked first, so a name both hidden and unlisted earns nothing here. A value row's JSON home is read by the value pass whatever the list says ([§FS-values.2.2](FS-values.md#22-json-declarations-from-the-kind-home)), so it earns nothing either. A directory root is filtered file by file, which is what the extension list means; a directory whose files are all unlisted earns nothing here, and the empty-scan caution still speaks when the run read nothing at all ([§FS-check.2.2](FS-check.md#22-empty-scan)).

#### checks.unread-scan-root.channel: A located finding on stdout, and the exit code does not move

It is a warning, so the exit code stays `0` and `success` is withheld ([§FS-check.2.1.3](FS-check.md#213-the-success-line)), and like [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) it is a located finding on stdout rather than a CLI-level caution ([§FS-check.4.9.4](FS-check.md#494-a-located-finding-on-stdout-not-a-cli-level-caution)): a statement about the coverage of a report belongs in the report. It is not an error, because nothing the run read is wrong; what is at stake is only how much the verdict covers. It names no release and stays a warning, like [§FS-check.4.9.6](FS-check.md#496-it-names-no-release): a row that names a file the list keeps out is a configuration that says two things, and the edit that makes it say one is the repository's to choose. `--ignore unread-scan-root` and `--only` select it like any code ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)).

#### checks.unread-scan-root.scope: Asked of the configured scope, by `check` alone

Every `check` run that loads the configuration asks it of every configured file root, `--full` included: `--full` keeps the configured roots ([§FS-check.1.3.2](FS-check.md#132-the-wider-scan-reads-a-superset-each-file-once)), so it prints the same warnings and no finding from inside the file, which it does not read either. A run given an explicit path reports it only where the path covers the `grund.toml` that holds the row, like any located finding ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)); `grund check notes.txt` keeps the caution of [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path) instead. In a workspace each project is asked on its own, and the warning is located in that project's own `grund.toml`. `list`, `refs`, `show`, `cover` and `fmt` read the same files and do not say it ([§DF-unread-scan-root.2.1](../decisions/functional/DF-unread-scan-root.md#21-check-alone-says-it)).

#### checks.unread-scan-root.json: Its JSON form is on stdout

Under `--format=json` it is one warning finding on stdout with `path` and `line` set and `sites` null, like every other located finding ([§FS-errors.5](FS-errors.md#5-json-format)), carrying the message of [§FS-scan.checks.unread-scan-root.message](FS-scan.md#checksunread-scan-rootmessage-the-message) byte for byte.

#### checks.unread-scan-root.empty-scan: It stands in place of the empty-scan caution

It is a finding about the default scope, so where the unread root was the only thing the run would have read, it suppresses the generic empty-scan caution ([§FS-check.2.2.3](FS-check.md#223-exit-code-json-and-what-suppresses-it)) and the run's stderr is empty. That caution points at `include` and `grund init --docs`, the wrong fix for a file the configuration already names; this warning names the row and the edit.

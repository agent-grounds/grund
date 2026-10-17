# FS-scan: what a path argument means, which projects a run loads, and which files it reads

What a run is handed and what it reads, defined once for every command that scans a tree: what each command's `<path>` argument means, which projects a run loads and so whose kinds and declarations it reads, which files a walk reads, which path names a file it read, and which text a file has. The `[scan]` keys and their defaults are [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned)'s, and which project owns a directory is [§FS-workspace.6.2](FS-workspace.md#62-the-boundary-is-mutual)'s; this page cites both rather than restating them. Serves [§GOAL-no-dangling-refs](../goals.md#goal-no-dangling-refs-every-cited-id-resolves-to-a-declaration), whose verdicts are only as good as the files a run read, and [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file), because one point answers what a run reads.

## terms: Terms

Leans on [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, config root, workspace, member).

- **kind home** — The `folder` or `file` a `[[kinds]]` row names ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds)): the place its declarations live.
- **scan root** — Where a walk starts: an `include` entry, a kind home, or an explicit path. Which rules it outruns is [§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent).
- **resolution scope**, **report scope** — The two scopes `check` tells apart for an explicit path, defined at [§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports).
- **narrowed scan** — One scan of the enclosing project below its config root, with no workspace loaded: what `cover` runs for a path narrower than its config root ([§FS-scan.path.cover](FS-scan.md#pathcover-cover-runs-one-narrowed-scan)).
- **buffer** — The text an editor holds open for a file, saved or not, which a run reads as that file ([§FS-scan.text.buffer](FS-scan.md#textbuffer-a-files-text-is-what-a-save-would-write)).

## requirements: What every scan holds to

One requirement on what a run reads as a whole, carrying a mark of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). Four more are left out because they are stated elsewhere: *every region a run leaves unread is named and can be looked into* is [§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded); *every command answers a kind name from one reading of the configs* is [§REQ-one-answer.identity.kinds](../requirements/REQ-one-answer.md#identitykinds-every-command-reads-one-kind-verdict); *a run given a path fails what the whole run fails about the files it reports* is [§FS-check.requirements.narrow-never-lenient](FS-check.md#requirementsnarrow-never-lenient-a-narrower-run-never-passes-what-a-wider-run-fails-about-the-files-it-reports--realized), and `list`, `show` and `id` resolve as it does, so no difference between the commands' paths is left at [§REQ-one-answer.differences.scope](../requirements/REQ-one-answer.md#differencesscope-scope); and *the machine's global git excludes are the one input from outside the checkout that decides which files are read* is [§FS-config.requirements.8](FS-config.md#requirements8-a-projects-meaning-is-self-contained--realized).

### requirements.spelling: How a tree is reached changes neither what is read nor what a finding names — realized

A repository reads the same however a run arrives at it. A file reached through a link is named by the path the run met — the in-tree link, the explicit argument, the root as handed — never by its physical target, even where that target lies outside the config root ([§FS-scan.identity.link-path](FS-scan.md#identitylink-path-a-finding-names-the-in-tree-link-path), [§FS-scan.identity.root-path](FS-scan.md#identityroot-path-the-scan-root-keeps-the-path-the-run-was-handed)); a physical file reached by two spellings is read once, under the spelling a written rule picks ([§FS-scan.identity.read-once](FS-scan.md#identityread-once-one-physical-file-is-read-once)), and an editor buffer is that file whichever spelling it was opened under ([§FS-scan.identity.buffers](FS-scan.md#identitybuffers-a-buffer-is-the-file-at-its-physical-location)); and which directory links are followed is decided by the project's canonical root, which stays the fence when the repository itself was reached through a link ([§FS-scan.scope.links](FS-scan.md#scopelinks-a-symlink-in-the-tree-is-followed)). So a checkout reached through a symlinked directory, as macOS reaches `/private/var` through `/var`, reads the files and reports the findings it does at its physical path, each spelled as its reader wrote it. Serves [§GOAL-friendliness-first.1](../goals.md#1-hard-requirements), whose `path:line` is one a reader can act on, and [§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree).

## path: What a path argument means

Every command that takes a `<path>` discovers its configuration from it ([§FS-scan.path.discovery](FS-scan.md#pathdiscovery-a-path-is-where-discovery-starts)). `check`, `list`, `show` and `id` then resolve against the project the path discovers, read as `check`'s resolution scope is ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)), and the path narrows only what they report ([§DF-one-answer-rulings-run](../decisions/functional/DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held), ruling D1); each one's leaf below says what that is. `cover` and `refs` give the path the meaning their own leaf states. Rows R01 and R02, the two places the commands' paths disagreed, closed with [§DF-path-narrows-the-report-in-0-19](../decisions/functional/DF-path-narrows-the-report-in-0-19.md#df-path-narrows-the-report-in-0-19-in-0190-a-path-narrows-what-show-list-and-id-report-not-what-they-resolve-and-workspace-check--resolves-every-project-it-sees) in grund 0.19.0 ([§REQ-one-answer.differences.scope](../requirements/REQ-one-answer.md#differencesscope-scope)). Resolving against the owning project costs a path narrower than its config root one read of that project, the cost `check` already pays ([§FS-check.1.3.6.3](FS-check.md#1363-a-file-the-wider-walk-could-not-read-is-said-out-loud)) and grund 0.18.0 paid for the warnings this resolution replaced ([§GOAL-fast-feedback](../goals.md#goal-fast-feedback-grund-must-be-as-fast-as-possible)). A run with no path, or with one at or above its config root, reads what it read before.

### path.discovery: A path is where discovery starts

An explicit `<path>` argument (e.g. `grund list apps/api`, `grund refs FS-x apps/api`, `grund complete ids --path apps/api`) behaves as if the command were invoked from that path: a `<path>` inside a member is member-scoped, not workspace-aggregate, even when a workspace exists above it ([§FS-scan.projects.discovery](FS-scan.md#projectsdiscovery-the-nearest-config-decides)). `cover`, whose `<path>` bounds a scan, runs one narrowed scan below the config root instead ([§FS-scan.path.cover](FS-scan.md#pathcover-cover-runs-one-narrowed-scan)).

### path.check: `check` reads more than it reports

A run given an explicit path reads the declarations and the citations its project's ordinary run reads, and reports findings only for the files at or under the path — or, where a finding spans several sites, for a finding naming one of them ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)). Two scopes, and telling them apart is the whole rule. The **resolution scope** is the set of files the run reads so that citations resolve and citation edges count: the enclosing project's ordinary scope — `[scan] include` plus every walked kind home ([§FS-scan.scope.include](FS-scan.md#scopeinclude-include-is-scanned-from-the-config-root), [§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)) — **union the path itself**, because a path may lie outside `include` and must still be read. The **report scope** is exactly the path. Every rule runs over the resolution scope, and a diagnostic anchored at a file outside the report scope is dropped before the report is written, unless it also names a site inside it ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)). Which projects the run loads is [§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see)'s, and where it loads several the resolution scope is each one's. So for `check` a path narrows the report and not what the run resolves against; which verdicts that moves is [§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)'s to say. Decided in [§DF-path-scope-resolves-project-wide](../decisions/functional/DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path).

### path.cover: `cover` runs one narrowed scan

Of the commands [§FS-workspace.8](FS-workspace.md#8-other-commands) covers, `cover` is the one whose `<path>` bounds a **scan** rather than choosing which config answers ([§FS-cover.1](FS-cover.md#1-inputs)): a scope narrower than the config root is one narrowed scan of the enclosing project, no workspace loaded and no `project` field. `grund check <dir>` drew the line there too until grund 0.19.0 gave it the projects a run from `<dir>` sees ([§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see)); `cover` is outside the ruling that moved it ([§DF-one-answer-rulings-run](../decisions/functional/DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held), ruling D1), so it keeps the narrowed scan. Widening `grund cover src/` back to every project would answer a question the caller did not ask, and an explicit path deliberately bypasses `[scan] include`, so the narrowing is the only thing that put those files in scope at all.

### path.warning: Where the path changed an answer, the run said so

Retired in grund 0.19.0: `list`, `show` and `id` resolve against the owning project ([§FS-scan.path.list](FS-scan.md#pathlist-list-reports-the-paths-entries-and-resolves-against-the-project), [§FS-scan.path.show](FS-scan.md#pathshow-show-resolves-the-id-against-the-project-the-path-discovers), [§FS-scan.path.id](FS-scan.md#pathid-id-numbers-from-the-project-the-path-discovers)), so no answer is left for a line to announce, and none is printed. grund 0.18.0 printed one `warning:` line on stderr wherever the answer computed against the owning project differed, as [§DF-path-narrowing-warns](../decisions/functional/DF-path-narrowing-warns.md#df-path-narrowing-warns-a-path-that-narrows-what-show-list-and-id-resolve-or-what-check--resolves-an-alias-against-says-so-for-one-release) records. This address is kept so citations written before the change still resolve.

#### path.warning.shape: A text line after everything else on stderr

Retired in grund 0.19.0 with [§FS-scan.path.warning](FS-scan.md#pathwarning-where-the-path-changed-an-answer-the-run-said-so): no such line is printed. This address is kept so citations written before the change still resolve.

#### path.warning.cost: Only a narrowed path reads more

Retired in grund 0.19.0 with [§FS-scan.path.warning](FS-scan.md#pathwarning-where-the-path-changed-an-answer-the-run-said-so): what a narrowed path reads is said in [§FS-scan.path](FS-scan.md#path-what-a-path-argument-means). This address is kept so citations written before the change still resolve.

#### path.warning.release: One named release, held by a test

Retired in grund 0.19.0 with [§FS-scan.path.warning](FS-scan.md#pathwarning-where-the-path-changed-an-answer-the-run-said-so): the release the line named has shipped, and no constant or test holds it. This address is kept so citations written before the change still resolve.

### path.list: `list` reports the path's entries and resolves against the project

For `list`, `<path>` is the directory or file whose entries are reported, `.` by default. Discovery is the same as every other subcommand (walk up to a `grund.toml`, else defaults — [§FS-config.1](FS-config.md#1-file-location-and-discovery)).

The path bounds which entries print: those declared at or under it. Everything said about an entry is the owning project's answer, read as [§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports) reads it: its JSON `refs` count every citation of it in the project, `--unused` drops an entry cited anywhere in the project, by a citation that counts as use and never by a kind's own index entry alone ([§FS-list.1.3](FS-list.md#13---unused)), and an entry with another home outside the path carries the duplicate note ([§FS-list.2.6](FS-list.md#26-duplicate-declarations)). The `--size` rows ([§FS-list.3.4](FS-list.md#34---size--per-coordinate-lead-and-full-body-measurements)) and the `--summary` counts ([§FS-list.3.3](FS-list.md#33---summary)) are of the same entries, with the same answers. So `list sub --unused` and `check sub` agree about which declarations under `sub` are cited, and `list sub` over a `sub/` that declares nothing prints nothing. A run that loads a workspace prints the catalog [§FS-workspace.8.3](FS-workspace.md#83-grund-list) states.

### path.show: `show` resolves the ID against the project the path discovers

For `show`, `<path>` chooses the project the ID resolves in. Defaults to `.`. Discovery is the same as every other subcommand (walk up to a `grund.toml`, else defaults — [§FS-config.1](FS-config.md#1-file-location-and-discovery)).

The ID resolves against that project read as [§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports) reads it, so a path narrower than its config root moves no answer: an ID the project declares outside the path prints its body, exit `0`, and one it declares both inside and outside the path is ambiguous ([§FS-show.2.2.1](FS-show.md#221-ambiguous-id)). A short form ([§FS-show.1.2](FS-show.md#12-the-number-only-shorthand)) is matched over the same project, so its candidates are the project's ([§FS-show.2.2.1.1](FS-show.md#2211-an-ambiguous-shorthand-names-its-candidates)), and under `--batch` every query resolves the same way. What `show` reports is the declaration the query names, so the path bounds nothing more, with one exception: `--batch --all` generates its queries from the entries `list <path>` prints ([§FS-scan.path.list](FS-scan.md#pathlist-list-reports-the-paths-entries-and-resolves-against-the-project)), and each of them still resolves against the whole project.

### path.refs: `refs` scans the path's tree

For `refs`, `<path>` is the directory or file whose tree is scanned, default `.`, discovered as by every other subcommand ([§FS-config.1](FS-config.md#1-file-location-and-discovery)).

### path.id: `id` numbers from the project the path discovers

For `id`, `<path>` is the directory whose project is numbered; it defaults to the current directory and is resolved as by every `grund` command ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)): config discovery walks up to `grund.toml`, else takes the defaults.

The next free number is one past the highest number of the kind the project declares, read as [§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports) reads it, wherever under it that declaration lies; under a number-less format ([§FS-id.4.1](FS-id.md#41-number-less-id-formats)) a proposal the project already declares is refused ([§FS-id.5](FS-id.md#5-collision-check)) wherever it lies. So a path narrower than its config root proposes what a run at the config root proposes.

## projects: Which projects a run loads

Which projects a run loads decides whose kinds and declarations it reads. Which project owns a directory is [§FS-workspace.6.2](FS-workspace.md#62-the-boundary-is-mutual)'s, and this chapter does not restate it.

### projects.discovery: The nearest config decides

Discovery follows the same walk-up rule as `grund check` ([§FS-config.1](FS-config.md#1-file-location-and-discovery)): from the CWD (or from an explicit `<path>` argument, [§FS-scan.path.discovery](FS-scan.md#pathdiscovery-a-path-is-where-discovery-starts)), walk up to the nearest `grund.toml` in either discovery form. If the nearest config is a member's own config, the command runs member-local — qualified `<alias>/<ID>` cannot resolve, the same way `check` errors at the member scope ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)). A scope inside a member with no config of its own is member-local all the same, on [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)'s canonical defaults with the member directory as the config root; only outside every member does a nearest config at the workspace root run the command workspace-wide.

### projects.workspace-root: A run at the workspace root loads every project

`grund check` run at a workspace root checks the root project (unless `include_root = false`, [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)) and all configured members, aggregates findings, and prints the same explicit-channel `<path>:<line>: <channel>: <message>` shape as a normal check ([§FS-check.2.1](FS-check.md#21-report-format)). Paths are rendered relative to the workspace root when `[output] relative_paths = true`.

### projects.member: A member run loads the member

`grund check <member>` (or `grund check` invoked from inside a member tree) discovers the member's own config first and, for a member that declares no `[workspace]` block of its own, validates it as an independent project; one that does runs its own subtree ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)). What a qualified citation earns there is [§FS-workspace.5.1](FS-workspace.md#51-a-member-run)'s.

### projects.narrowed: A path below the config root loads what a run from it would see

`grund check <path>` with a `<path>` narrower than the config root loads the projects a run from `<path>` would load ([§FS-scan.projects.discovery](FS-scan.md#projectsdiscovery-the-nearest-config-decides), [§FS-workspace.8.8](FS-workspace.md#88-discovery-follows-grund-checks-walk-up)), resolves each of them completely, and reports only the path ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)). Inside a member that is the member ([§FS-scan.projects.member](FS-scan.md#projectsmember-a-member-run-loads-the-member)): a member directory still means a member run, which refuses a qualified citation as [§FS-workspace.5.1](FS-workspace.md#51-a-member-run) says. In a project with no workspace it is that project. Below a workspace root and outside every member it is every project the root run loads ([§FS-scan.projects.workspace-root](FS-scan.md#projectsworkspace-root-a-run-at-the-workspace-root-loads-every-project)), so a qualified citation the root run resolves resolves here too. Decided by [§DF-one-answer-rulings-run](../decisions/functional/DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held), ruling D14, and landed by [§DF-path-narrows-the-report-in-0-19](../decisions/functional/DF-path-narrows-the-report-in-0-19.md#df-path-narrows-the-report-in-0-19-in-0190-a-path-narrows-what-show-list-and-id-report-not-what-they-resolve-and-workspace-check--resolves-every-project-it-sees).

What such a run prints below a workspace root is the root run's report narrowed to the path: each finding the root run makes that is anchored at or under the path, or names a site there ([§FS-check.1.3.6.2](FS-check.md#1362-a-finding-that-spans-several-sites-is-in-scope-at-any-of-them)), kept whole — the same bytes, its path rendered against the workspace root, and an unknown alias with the outermost root's candidates ([§FS-check.3.8.2](FS-check.md#382-candidates-at-the-outermost-root)). Under `--format json` each is the object the root run writes, which carries no `project` field there either ([§FS-check.2.1](FS-check.md#21-report-format)). The run-level findings, the workspace run warnings among them, carry no path and print as the root run prints them ([§FS-check.1.3.6.1](FS-check.md#1361-a-path-scope-narrows-the-report-not-the-resolution)).

#### projects.narrowed.alias: A refused alias the workspace root resolved said so

Retired in grund 0.19.0: a run below a workspace root and outside every member loads the workspace ([§FS-scan.projects.narrowed](FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see)), so an alias the root run resolves resolves there too and no refusal is left to annotate. grund 0.18.0 appended the clause [§DF-path-narrowing-warns](../decisions/functional/DF-path-narrowing-warns.md#df-path-narrowing-warns-a-path-that-narrows-what-show-list-and-id-resolve-or-what-check--resolves-an-alias-against-says-so-for-one-release) records. A member run refuses a qualified citation with its message unchanged ([§FS-workspace.5.1](FS-workspace.md#51-a-member-run)). This address is kept so citations written before the change still resolve.

### projects.kinds: Whose kinds a run reads

A run reads the kinds of the projects its configuration declares: its own config's `[[kinds]]` and, where that config declares a `[workspace]`, each member's ([§FS-config.3.4](FS-config.md#34-kinds--recognized-kinds), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)). It reads them from the configs alone, before any file is read, so a narrowed scan ([§FS-scan.path.cover](FS-scan.md#pathcover-cover-runs-one-narrowed-scan)), which loads no member's files, still reads the kinds its declared workspace configures and judges its `[citations]` targets against them ([§FS-config.3.9.5.2](FS-config.md#3952-a-pinned-target-is-judged-against-its-projects-kinds)). A rule object there is judged in the projects the run loads, so a namespaced one stays unverifiable, as [§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify) says. Every question a command asks about a kind name is answered from that one reading, never from a set the command builds for itself.

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

A home is a **scan root**, and no scan root is pruned by `exclude`, an ignore file, or the hidden-directory rule ([AR-scanner.1](../architecture/AR-scanner.md#1-tree-walk)) — so a home the repository also excludes is read, while everything *below* it is filtered as usual. That is the honest reading of a config that says both: the `[[kinds]]` entry names the directory, and `exclude` was written about descendants. All three are rules about a descent, which is why a root outruns them. Two rules are about a file's own name, and a root does not outrun either: the hidden-file rule ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)) and the `extensions` list ([§FS-scan.scope.extensions.root](FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so)). A file root either name rule refuses is not read, and `check` names it rather than passing over it.

### scope.not-a-fence: `include` is a scan scope, not a fence

A citation in a file that neither `include` nor a kind home brings into the scan is invisible rather than merely unchecked ([§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)), so `grund check --full` scans the whole config root past this key. The flag cancels `include` and, with it, the `scan = false` prune of [§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned), and nothing else: every other rule of this chapter applies to that scan unchanged ([§FS-check.1.3.1](FS-check.md#131-the-scan-covers-the-whole-config-root)), and what it reports outside the default scope is [§FS-check.1.3](FS-check.md#13-the-full-tree-scope---full)'s to say.

### scope.hidden: A hidden file is not read, and the rule is not about descent

The scan skips a hidden directory by not descending into it; it skips a file whose own name begins with `.` by the name that file wears, before `extensions` is consulted at all ([AR-scanner.1](../architecture/AR-scanner.md#1-tree-walk)). So `docs/.notes.md` is not scanned though `md` is listed, and a citation inside it neither resolves nor dangles — it is invisible the way one outside `include` is, and `grund check --full` does not reach it either ([§FS-scan.scope.not-a-fence](FS-scan.md#scopenot-a-fence-include-is-a-scan-scope-not-a-fence)). Being a rule about a name rather than about a descent, it also reaches a scan **root**, the one exception to [§FS-scan.scope.roots](FS-scan.md#scoperoots-a-scan-root-outruns-every-rule-about-descent): a `file` home whose name is hidden, like an `include` entry naming one, is not read as a root, and `check` names it at the row that made it a root ([§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read)), so a configuration that names a dotfile to read never passes as `success`. It is a blind spot the repository can see and plan around ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)): what decides is the file's own name, so a document that must be checked is one not named as a dotfile, and a run handed such a file whose extension `extensions` *does* allow says so rather than blaming the list ([§FS-check.2.2](FS-check.md#22-empty-scan)).

### scope.extensions: An extension makes a file readable, not declarable

Listing an extension makes a file *readable*, not declarable. A prose markup format other than Markdown — AsciiDoc, reStructuredText, LaTeX — has its citations checked as soon as its extension appears in `extensions`, because the citation grammar is format-agnostic; its native heading syntax still declares nothing, since a declaration is a `#`-prefixed heading or a comment-prefixed line. Whether those formats should be declaration homes of their own is an open discussion ([§DISC-markup-format-declarations](../discussions/proposals/2026-05-25-markup-format-declarations.md#disc-markup-format-declarations-declarations-in-asciidoc-restructuredtext-latex-and-similar-markup-document-formats)), not a configured behavior.

#### scope.extensions.default: The `extensions` default follows the language table

With no `extensions` key, a run reads `md` followed by every extension grund's built-in language table names, in that table's order: each extension [§FS-inline-citation-style.1.1.2](FS-inline-citation-style.md#112-languages) assigns a doc-comment recognizer, and each language read through a comment prefix alone ([§FS-languages](FS-languages.md#fs-languages-how-a-source-files-lines-are-read), which pairs each prefix with its extensions). There is no second list to keep in step with it, so a language the table gains is read by default in the same release, and a project that wrote no `extensions` is never told `success` over a file in a language grund classifies but did not open ([§REQ-no-missed-citation.3](../requirements/REQ-no-missed-citation.md#3-proven-per-host-language)). A language the table gains is appended, which keeps every earlier default a prefix of the current one. Extensions match case-sensitively, so `r` and `R` are two entries. An explicit `extensions` still replaces the default whole. Why the default widens rather than warns is [§DF-default-extensions-follow-language-table](../decisions/functional/DF-default-extensions-follow-language-table.md#df-default-extensions-follow-language-table-the-scan-extensions-default-reads-every-language-grund-classifies).

#### scope.extensions.root: A file root whose extension is unlisted is not read, and `check` says so

A scan root that is a single file — a `[[kinds]]` `file` home, a version-2 `files` entry, or an `[scan] include` entry naming a file — is read only if its extension is one `extensions` lists, or in a version-2 file one a language of `[schema.sources] languages` reads. A file with no extension at all, such as `NOTES`, has none the list can hold, so it is refused by the same rule. The list is not widened for a root: an unlisted extension has no comment prefixes to read the file with, and a row may name a file only as a place in the Project map. What the root earns instead is a report: every `grund check` that has the root in its configured scope prints the warning of [§FS-scan.checks.unread-scan-root](FS-scan.md#checksunread-scan-root-a-configured-file-root-is-not-read), located at the row that made the file a root, so a configuration that names a file to read and a list that keeps it out never passes as `success` ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)).

Three such files earn nothing. A row marked `scan = false` is not a root ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)), and a home that does not exist scans as nothing. A value row's JSON home is read too, by the value pass rather than the walk ([§FS-values.2.2](FS-values.md#22-json-declarations-from-the-kind-home)). A directory root is filtered file by file, which is what `extensions` means, so it earns nothing either. The other commands that scan — `list`, `refs`, `show`, `cover`, `fmt` — read the same files and say nothing about the root: the fact is about the coverage of `check`'s verdict ([§DF-unread-scan-root](../decisions/functional/DF-unread-scan-root.md#df-unread-scan-root-a-configured-file-root-whose-extension-is-unlisted-is-named-not-read)).

A file root whose name is hidden is refused by [§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent) as well, and is named by the same warning, not passed over: where its extension is unlisted too, that one warning names both rules ([§DF-unread-hidden-root](../decisions/functional/DF-unread-hidden-root.md#df-unread-hidden-root-a-configured-file-root-with-a-hidden-name-is-named-not-read)).

### scope.ignore-files: `respect_gitignore` — the ignore files

`respect_gitignore` (default `true`) makes the scanner honor every form of ignore file the `ignore` crate recognizes — `.gitignore` at any depth, `.git/info/exclude`, the global `core.excludesFile`, and `.ignore` files. Set to `false` only when you genuinely need to scan ignored paths. The directory-level `exclude` list is applied **in addition** to ignore-file rules, never instead of them. A global pattern anchors at the top of its path's repository, whatever directory the run starts from ([§FS-scan.scope.ignore-files.global](FS-scan.md#scopeignore-filesglobal-a-global-excludes-pattern-anchors-at-the-top-of-its-paths-repository)). See [AR-scanner.1.1](../architecture/AR-scanner.md#11-respecting-gitignore-and-friends).

#### scope.ignore-files.global: A global excludes pattern anchors at the top of its path's repository

An ignore file means what git means by it, and the global `core.excludesFile` is no exception. git reads each of its patterns relative to the top of the repository the path belongs to. grund anchors them at the top the `ignore` crate counts, and at nothing else: the nearest directory above the path that holds `.git` or a Jujutsu `.jj`, the same top that bounds the `.gitignore` files. That is git's top, except in a jj workspace with no `.git` nested inside a git repository: git anchors there at the outer top, and grund at the workspace, where it already bounds the `.gitignore` files. So an anchored pattern, one with a `/` before its end such as `notes/gen/`, ignores `<top>/notes/gen/` and nothing at another depth, while an unanchored one such as `gen/` matches at any depth below that top. The directory grund runs from does not move the top, whether that is the repository top, a subdirectory or a directory outside the repository. A config root or scan root above the repository top does not move it either. Nor does the spelling a platform gives the paths it walks, so a run on Windows, whose walk reads canonical `\\?\` paths, ignores what a run on Linux ignores. Where a scan walks into a repository nested inside another, a path anchors at the top of the innermost repository that holds it.

Nothing else about the global file changes. A path below no such top is matched by none of its patterns, as git matches none outside a repository. Precedence stays git's: `.ignore` files, then the `.gitignore` files up to the repository top, then that repository's `info/exclude`, which a top holding only `.jj` does not have, then the global file. A `!` re-include in any of the first three still keeps a file the global file ignores. The walk and a stub target's eligibility read the one anchoring, so a file the walk does not read is not a stub target either ([§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing)). Which files a run reads does not depend on where the run was started ([§REQ-deterministic-output.2](../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation)). Why this correction may move a verdict is [§DF-global-excludes-anchor-at-the-repository-top](../decisions/functional/DF-global-excludes-anchor-at-the-repository-top.md#df-global-excludes-anchor-at-the-repository-top-a-global-excludes-pattern-anchors-at-the-repository-top-wherever-grund-runs-from).

#### scope.ignore-files.pathname: The global excludes filename is the one Git resolves

When `respect_gitignore = true`, grund loads the existing readable global excludes file at the pathname Git resolves. A `~` anywhere except a leading home abbreviation (`~/` or `~user/`) is a literal pathname character, including in Windows short names such as `RUNNER~1`. Expanding a leading abbreviation does not expand literal tildes in the resulting home pathname again. Thus `/work/a~b/global.ignore` names that file, and `~/global.ignore` under a home named `/work/h~me` names `/work/h~me/global.ignore`.

The scan, its probe of an individual path, and a stub target's eligibility use that same resolved file. This changes neither global-file discovery and configuration precedence nor pattern anchoring and ignore precedence ([§FS-scan.scope.ignore-files.global](FS-scan.md#scopeignore-filesglobal-a-global-excludes-pattern-anchors-at-the-top-of-its-paths-repository)); `respect_gitignore = false` still opts out. The verdict correction and its remedies are recorded in [§DF-global-excludes-literal-tilde](../decisions/functional/DF-global-excludes-literal-tilde.md#df-global-excludes-literal-tilde-preserve-literal-tildes-in-the-global-excludes-pathname). This point defines pathname resolution for a readable file, not a new diagnostic or exit policy for missing or unreadable global files.

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

A file's text is what saving it would write: the editor's open buffer where there is one, and the disk only where there is none. That holds for every reader a run has, the walk's scan of each file, a stub's target ([§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it)) and the body `show` slices out of it ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)) among them, so an unsaved edit moves every answer at once, exactly as saving it would ([§REQ-one-answer.text.targets](../requirements/REQ-one-answer.md#texttargets-a-stubs-target-answers-as-its-saved-text-would)).

### text.buffer-only: A buffer with no file behind it is a file

A buffer with no file on disk behind it, a new file the editor has not saved, is a file of the run. It exists for a stub, so a stub to it is never `stub link target missing` and is judged on the buffer's text ([§FS-declarations.stubs.target.1](FS-declarations.md#stubstarget1-the-target-is-read-as-a-save-would-write-it)). The walk reads it or skips it by the rules of [§FS-scan.scope](FS-scan.md#scope-which-files-a-run-reads) exactly as it would the saved file: `include` and the kind homes bring it in, and a hidden name, an unlisted extension, `exclude`, an ignore file of any form [§FS-scan.scope.ignore-files](FS-scan.md#scopeignore-files-respect_gitignore--the-ignore-files) names, a `scan = false` home and another project's root keep it out. Where the editor holds a buffer for a file the walk skips, the walk's rule wins: the buffer is the file, and the file is not read. A stub's target outside `include` is read as if it were scanned ([§FS-show.2.3.7](FS-show.md#237-a-stubs-target-is-found-by-its-id)), a buffer as much as a saved file, and only by the name and extension rules of [§FS-declarations.stubs.target.3](FS-declarations.md#stubstarget3-a-target-the-scan-does-not-read-declares-nothing).

## checks: Checks

### checks.unread-scan-root: A configured file root is not read

A scan root that is a single file — a `[[kinds]]` `file` home, a version-2 `files` entry, or an `[scan] include` entry naming a file — whose name is hidden ([§FS-scan.scope.hidden](FS-scan.md#scopehidden-a-hidden-file-is-not-read-and-the-rule-is-not-about-descent)), or whose extension the extension list leaves out ([§FS-scan.scope.extensions.root](FS-scan.md#scopeextensionsroot-a-file-root-whose-extension-is-unlisted-is-not-read-and-check-says-so)). A root refused by both rules earns one warning, not two ([§DF-unread-hidden-root](../decisions/functional/DF-unread-hidden-root.md#df-unread-hidden-root-a-configured-file-root-with-a-hidden-name-is-named-not-read)). Nothing in the file was read: no declaration in it reached a catalog and no citation in it was resolved or reported, so the configuration names a file to check and the run checks none of it. Saying so is this finding's whole job ([§REQ-no-missed-citation.1](../requirements/REQ-no-missed-citation.md#1-no-silent-skips)).

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

For a root whose name is not hidden, the reason names the extension as the file name spells it, the part after the last `.`, or says there is none:

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

For a root whose name begins with `.`, the reason says so first, and names the extension as well where the list refuses it too, so one warning names every rule that keeps the file out:

| The hidden file | Reason |
|---|---|
| has an extension the list holds | `` its name begins with `.`, and grund reads no hidden file `` |
| has one it does not, version 1 | ``its name begins with `.`, and `<ext>` is not in [scan] extensions`` |
| has one it does not, version 2 | ``its name begins with `.`, and no language in [schema.sources] languages reads `.<ext>` `` |
| has none, such as `.notes` | `` its name begins with `.`, and it has no extension `` |

Its fix is one edit, the rename, and it names everything the new name must satisfy, because renaming the file alone would leave the extension refusal standing and listing the extension alone changes nothing:

| Root | Only the name refuses it | The extension refuses it too, or it has none |
|---|---|---|
| a version-1 `[[kinds]]` home | `Rename the file to a name that is not hidden` | `Rename the file to a name that is not hidden and an extension the list holds` |
| an `[scan] include` entry | `Rename the file to a name that is not hidden, or drop the entry` | `Rename the file to a name that is not hidden and an extension the list holds, or drop the entry` |
| a version-2 `files` entry | `Rename the file to a name that is not hidden` | `Rename the file to a name that is not hidden and an extension a listed language reads` |

A home on a row with `citable = false` may name the file only as a place in the Project map, so its fix also offers the key that says so ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)): the last `, or` of the version-1 fix becomes `,`, and the fix ends ``, or set `scan = false` on the row if it only lists a place``. A version-2 row's single fix, and a hidden home's single rename, gain the same ending. So, for one root of each shape:

```
grund.toml:10: warning: [[kinds]] TXT home `notes.txt` is not read — `txt` is not in [scan] extensions, so nothing in it is checked. Add "txt" to [scan] extensions, or give the file an extension the list holds.
grund.toml:9: warning: [scan] include entry `extra.txt` is not read — `txt` is not in [scan] extensions, so nothing in it is checked. Add "txt" to [scan] extensions, or drop the entry.
grund.toml:7: warning: [[kinds]] NOTES home `NOTES` is not read — it has no extension, and [scan] extensions reads a file only by its extension, so nothing in it is checked. Rename the file to an extension the list holds, or set `scan = false` on the row if it only lists a place.
grund.toml:8: warning: [schema.kinds.TXT] file `notes.txt` is not read — no language in [schema.sources] languages reads `.txt`, so nothing in it is checked. Give the file an extension a listed language reads.
grund.toml:10: warning: [[kinds]] NOTE home `.notes.md` is not read — its name begins with `.`, and grund reads no hidden file, so nothing in it is checked. Rename the file to a name that is not hidden.
grund.toml:9: warning: [scan] include entry `.extra.md` is not read — its name begins with `.`, and grund reads no hidden file, so nothing in it is checked. Rename the file to a name that is not hidden, or drop the entry.
grund.toml:10: warning: [[kinds]] TXT home `.notes.txt` is not read — its name begins with `.`, and `txt` is not in [scan] extensions, so nothing in it is checked. Rename the file to a name that is not hidden and an extension the list holds.
grund.toml:8: warning: [schema.kinds.NOTE] file `.notes.md` is not read — its name begins with `.`, and grund reads no hidden file, so nothing in it is checked. Rename the file to a name that is not hidden.
```

#### checks.unread-scan-root.silent: What stays silent

A row marked `scan = false` is not a root and earns nothing ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)), and neither does a home or entry that does not exist ([§FS-scan.scope.homes](FS-scan.md#scopehomes-every-configured-kind-home-is-scanned)). A hidden file *inside* a directory root earns nothing: nothing in the configuration named it, and a directory is filtered file by file; a hidden home that does not exist earns nothing either. A value row's JSON home is read by the value pass whatever the list says ([§FS-values.2.2](FS-values.md#22-json-declarations-from-the-kind-home)), so it earns nothing either. A directory root is filtered file by file, which is what the extension list means; a directory whose files are all unlisted earns nothing here, and the empty-scan caution still speaks when the run read nothing at all ([§FS-check.2.2](FS-check.md#22-empty-scan)).

#### checks.unread-scan-root.channel: A located finding on stdout, and the exit code does not move

It is a warning, so the exit code stays `0` and `success` is withheld ([§FS-check.2.1.3](FS-check.md#213-the-success-line)), and like [§FS-check.4.9](FS-check.md#49-a-workspace-member-declared-optional-is-absent) it is a located finding on stdout rather than a CLI-level caution ([§FS-check.4.9.4](FS-check.md#494-a-located-finding-on-stdout-not-a-cli-level-caution)): a statement about the coverage of a report belongs in the report. It is not an error, because nothing the run read is wrong; what is at stake is only how much the verdict covers. It names no release and stays a warning, like [§FS-check.4.9.6](FS-check.md#496-it-names-no-release): a row that names a file a name rule keeps out is a configuration that says two things, and the edit that makes it say one is the repository's to choose. `--ignore unread-scan-root` and `--only` select it like any code ([§FS-check.2.1.2](FS-check.md#212-selection-filters-the-complete-report)).

#### checks.unread-scan-root.scope: Asked of the configured scope, by `check` alone

Every `check` run that loads the configuration asks it of every configured file root, `--full` included: `--full` keeps the configured roots ([§FS-check.1.3.2](FS-check.md#132-the-wider-scan-reads-a-superset-each-file-once)), so it prints the same warnings and no finding from inside the file, which it does not read either. A run given an explicit path reports it only where the path covers the `grund.toml` that holds the row, like any located finding ([§FS-scan.path.check](FS-scan.md#pathcheck-check-reads-more-than-it-reports)); `grund check notes.txt` keeps the caution of [§FS-check.2.2.2.2](FS-check.md#2222-an-explicit-path) instead, and `grund check docs/.notes.md` that of [§FS-check.2.2.2.3](FS-check.md#2223-a-hidden-file): a path typed on the command line has no row behind it. In a workspace each project is asked on its own, and the warning is located in that project's own `grund.toml`. `list`, `refs`, `show`, `cover` and `fmt` read the same files and do not say it ([§DF-unread-scan-root.2.1](../decisions/functional/DF-unread-scan-root.md#21-check-alone-says-it)).

#### checks.unread-scan-root.json: Its JSON form is on stdout

Under `--format=json` it is one warning finding on stdout with `path` and `line` set and `sites` null, like every other located finding ([§FS-errors.5](FS-errors.md#5-json-format)), carrying the message of [§FS-scan.checks.unread-scan-root.message](FS-scan.md#checksunread-scan-rootmessage-the-message) byte for byte.

#### checks.unread-scan-root.empty-scan: It stands in place of the empty-scan caution

It is a finding about the default scope, so where the unread root was the only thing the run would have read, it suppresses the generic empty-scan caution ([§FS-check.2.2.3](FS-check.md#223-exit-code-json-and-what-suppresses-it)) and the run's stderr is empty. That caution points at `include` and `grund init --docs`, the wrong fix for a file the configuration already names; this warning names the row and the edit.

# DF-path-narrows-the-report-in-0-19: in 0.19.0 a path narrows what show, list and id report, not what they resolve, and workspace check <dir> resolves every project it sees

**Date:** 2026-10-15

## status: Accepted

## context: Context

[§DF-one-answer-rulings-run](DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held) ruled that a path argument narrows what `check`, `list`, `show` and `id` report and never what they resolve (ruling D1), and that in a workspace `check <dir>` discovers the projects a run from `<dir>` would see and resolves each completely, a member directory still meaning a member run (ruling D14). Both take the warning route of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path). [§DF-path-narrowing-warns](DF-path-narrowing-warns.md#df-path-narrowing-warns-a-path-that-narrows-what-show-list-and-id-resolve-or-what-check--resolves-an-alias-against-says-so-for-one-release) was the warning release: grund 0.18.0 kept every answer and said, wherever the answer would move, that it moves in 0.19.0. That notice shipped with 0.18.0, and a notice already released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that moves the answers carries its own, here.

## decision: Decision

In 0.19.0 the answers move and the warnings go (agent-grounds/grund#621):

- `show`, `list` and `id` resolve against the project the path discovers, read as `check`'s resolution scope is, and the path bounds only what they report ([§FS-scan.path.show](../../functional-spec/FS-scan.md#pathshow-show-resolves-the-id-against-the-project-the-path-discovers), [§FS-scan.path.list](../../functional-spec/FS-scan.md#pathlist-list-reports-the-paths-entries-and-resolves-against-the-project), [§FS-scan.path.id](../../functional-spec/FS-scan.md#pathid-id-numbers-from-the-project-the-path-discovers)). `grund FS-001-a sub` prints a declaration that lies outside `sub`, `list sub` counts and selects its entries by every citation in the project, and `id FS beta sub` numbers from the whole project.
- Below a workspace root and outside every member, `check <dir>` loads every project the root run loads and prints the root run's report narrowed to `<dir>`, byte for byte and with no `project` field in JSON, as the root run's findings have none ([§FS-scan.projects.narrowed](../../functional-spec/FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see)). A qualified citation the root run resolves is no longer `unknown project alias` there. A member directory is still a member run ([§FS-workspace.5.1](../../functional-spec/FS-workspace.md#51-a-member-run)).
- No `warning:` line from [§FS-scan.path.warning](../../functional-spec/FS-scan.md#pathwarning-where-the-path-changed-an-answer-the-run-said-so) is printed, and no `unknown project alias` gains the clause [§FS-scan.projects.narrowed.alias](../../functional-spec/FS-scan.md#projectsnarrowedalias-a-refused-alias-the-workspace-root-resolved-said-so) added; both points are retired.
- Rows R01 and R02 leave [§REQ-one-answer.differences.scope](../../requirements/REQ-one-answer.md#differencesscope-scope).

A narrowed path reads the owning project once, as 0.18.0 already did to warn ([§FS-scan.path](../../functional-spec/FS-scan.md#path-what-a-path-argument-means)).

## alternatives: Alternatives refused

- **Keep the warnings for one more release beside the new answers.** A line that announces a change already made says nothing a reader can act on.
- **Make workspace `check <dir>` one scan of the enclosing project that also loads the members' aliases.** It would resolve the alias but not the rest of what the root run judges about a citation into a member, such as its target section, so `check <dir>` and the root run would still disagree about a file both report. D14 says each project is resolved completely.
- **Add a `project` field to a narrowed workspace `check`'s findings.** The root run's findings carry none, and the narrowed report is the root run's, so adding one here alone would make the two disagree in shape.

## consequences: Consequences

A script that relied on `show <path>` missing an ID declared outside the path, on `id <path>` numbering from the path alone, or on `check <dir>` failing on an alias the root resolves sees a different answer. Nothing has to be migrated in a repository: the root run already gave these answers.

## release-note: Release note

- [§DF-path-narrows-the-report-in-0-19](DF-path-narrows-the-report-in-0-19.md#df-path-narrows-the-report-in-0-19-in-0190-a-path-narrows-what-show-list-and-id-report-not-what-they-resolve-and-workspace-check--resolves-every-project-it-sees), [§FS-scan.path.show](../../functional-spec/FS-scan.md#pathshow-show-resolves-the-id-against-the-project-the-path-discovers), [§FS-scan.path.list](../../functional-spec/FS-scan.md#pathlist-list-reports-the-paths-entries-and-resolves-against-the-project), [§FS-scan.path.id](../../functional-spec/FS-scan.md#pathid-id-numbers-from-the-project-the-path-discovers), [§FS-scan.projects.narrowed](../../functional-spec/FS-scan.md#projectsnarrowed-a-path-below-the-config-root-loads-what-a-run-from-it-would-see): **a path narrows what `show`, `list` and `id` report, not what they resolve, and in a workspace `check <dir>` resolves every project a run from `<dir>` sees.** `grund <ID> <path>` finds an ID the project declares outside the path, and reports one declared both inside and outside it as ambiguous. `list <path>` counts and selects its entries by every citation in the project and notes a duplicate home outside the path. `id <KIND> <title> <path>` numbers from the whole project. Below a workspace root and outside every member, `check <dir>` prints the root run's findings within `<dir>`, so a qualified citation the root resolves is no longer `unknown project alias` and the run can exit `0`; a member directory is a member run as before. The 0.18.0 warnings that announced this are gone. **Who this breaks:** a script that relied on `show <path>` missing an ID declared outside the path, on `id <path>` numbering from the path alone, or on `check <dir>` exiting `1` for an alias the workspace root resolves. Closes [issue #621](https://github.com/agent-grounds/grund/issues/621).

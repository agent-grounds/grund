# DF-path-narrowing-warns: a path that narrows what show, list and id resolve, or what check <dir> resolves an alias against, says so for one release

**Date:** 2026-10-12

## status: Accepted

## context: Context

[§DF-one-answer-rulings-run](DF-one-answer-rulings-run.md#df-one-answer-rulings-run-a-path-narrows-the-report-a-stub-reads-what-the-walk-reads-and-a-file-is-one-file-however-it-is-spelled-or-held) rules that a path argument narrows what `check`, `list`, `show` and `id` report and never what they resolve (ruling D1), and that in a workspace `check <dir>` resolves each project a run from `<dir>` would see, a member directory still meaning a member run (ruling D14). Both rulings name the warning release of [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) as their route: a query operand is not a citation, so no existing requirement forbade the old narrowing, and the answers may move only after a release that says they will. [Issue #620](https://github.com/agent-grounds/grund/issues/620) is that release; [issue #621](https://github.com/agent-grounds/grund/issues/621) is the one that moves the answers.

Today the narrowing is silent. `grund FS-001-a sub` answers `ID not found` for an ID declared one directory up, `grund id FS "beta" sub` proposes a number the project already holds, and in a workspace whose root resolves <§>api/API-a, `grund check docs` reports `unknown project alias api` with nothing saying the root run resolves it.

## decision: Decision

Every answer stays byte for byte on stdout and keeps its exit code, and the run says, where the answer computed against the owning project would differ, that it will differ from grund 0.19.0:

- `show`, `list` and `id` add a `warning:` line on stderr naming the site outside the path that moves the answer ([§FS-scan.path.warning](../../functional-spec/FS-scan.md#pathwarning-where-the-path-changed-an-answer-the-run-said-so), [§FS-scan.path.show](../../functional-spec/FS-scan.md#pathshow-show-resolves-the-id-against-the-project-the-path-discovers), [§FS-scan.path.list](../../functional-spec/FS-scan.md#pathlist-list-reports-the-paths-entries-and-resolves-against-the-project), [§FS-scan.path.id](../../functional-spec/FS-scan.md#pathid-id-numbers-from-the-project-the-path-discovers)).
- A narrowed `check <dir>` appends one clause to `unknown project alias` where the workspace its config root declares resolves the alias; a member run gains nothing ([§FS-scan.projects.narrowed.alias](../../functional-spec/FS-scan.md#projectsnarrowedalias-a-refused-alias-the-workspace-root-resolved-said-so)).

A line fires only where the next release's answer differs. A declaration merely lying outside the path is not one: from grund 0.19.0 the path still bounds what is reported, so warning about it would announce a change that never comes.

## alternatives: Alternatives refused

- **Warn on every narrowed run.** A line that fires whether or not anything moves is one people learn to skip, which spends the window [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path) exists to give.
- **A second `warning:` finding beside the alias error.** It would add a row to the report and move `check`'s located output for every consumer; appending to the message moves only an exact-line consumer, whose migration is the stable `code` ([§FS-errors.3](../../functional-spec/FS-errors.md#3-message-text)).
- **Carry the warning in the JSON output.** `show`, `list` and `id` have no warning shape on stdout, and inventing one would change the bytes this release promises to keep.

## consequences: Consequences

A path narrower than its project's config root now reads the owning project once, the cost [§DF-path-scope-resolves-project-wide](DF-path-scope-resolves-project-wide.md#df-path-scope-resolves-project-wide-a-path-scoped-check-resolves-against-the-whole-project-and-reports-only-the-path) accepted for `check` and grund 0.19.0 pays for every command. A run with no path reads what it read before. [§REQ-one-answer.differences.scope](../../requirements/REQ-one-answer.md#differencesscope-scope) keeps listing the disagreement until #621 closes it.

## complete: The route is complete

grund 0.19.0 moved the answers on the schedule this record named, and the warnings and the clause are gone ([#621](https://github.com/agent-grounds/grund/issues/621)). The notice below is not rewritten: it shipped in 0.18.0 naming 0.19.0, and a notice already released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)). The landing release's own notice is [§DF-path-narrows-the-report-in-0-19.release-note](DF-path-narrows-the-report-in-0-19.md#release-note-release-note).

## release-note: Release note

- [§FS-scan.path.warning](../../functional-spec/FS-scan.md#pathwarning-where-the-path-changed-an-answer-the-run-said-so), [§FS-scan.path.show](../../functional-spec/FS-scan.md#pathshow-show-resolves-the-id-against-the-project-the-path-discovers), [§FS-scan.path.list](../../functional-spec/FS-scan.md#pathlist-list-reports-the-paths-entries-and-resolves-against-the-project), [§FS-scan.path.id](../../functional-spec/FS-scan.md#pathid-id-numbers-from-the-project-the-path-discovers), [§FS-scan.projects.narrowed.alias](../../functional-spec/FS-scan.md#projectsnarrowedalias-a-refused-alias-the-workspace-root-resolved-said-so), [§DF-path-narrowing-warns](DF-path-narrowing-warns.md#df-path-narrowing-warns-a-path-that-narrows-what-show-list-and-id-resolve-or-what-check--resolves-an-alias-against-says-so-for-one-release): **a path that narrows what `show`, `list` or `id` resolve now says so, and so does a narrowed `check <dir>` refusing an alias the workspace root resolves.** Every answer keeps its stdout and exit code. `show`, `list` and `id` given a path narrower than their project add a `warning:` line to stderr wherever reading the whole project would change the answer, naming the site outside the path and ending in `from grund 0.19.0 a path narrows what is reported, not what resolves`. In a workspace, `check <dir>`'s `unknown project alias <alias>` keeps its text and appends `; grund check from the workspace root resolves this alias, and from grund 0.19.0 grund check <dir> will too` where the root resolves it; a member run is unchanged. **Who this breaks:** a consumer matching the whole alias line, or asserting an empty stderr from a narrowed `show`, `list` or `id`. Closes [issue #620](https://github.com/agent-grounds/grund/issues/620).

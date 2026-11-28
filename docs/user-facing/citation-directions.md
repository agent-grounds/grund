# Citation directions

Version-2 configuration reference: [`[rules.citations.<KIND>]`](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/configuration.md#rulescitationskind) covers the keys explained here.

In version 2, `[rules.citations]` describes which kinds of documents cite which other kinds.
The seven levels use two rule classes and two enforcement surfaces, as specified
by [the citation-direction rules](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config.md#39-citations--citation-direction-rules):

| Level | Rule class | Checked per | Surface |
| --- | --- | --- | --- |
| `must` | obligation | declaration already containing a citation | `grund check` error, and the generated entrypoint |
| `warn` | obligation | declaration already containing a citation | warning, and the generated entrypoint |
| `should` | obligation | declaration already containing a citation | `--suggestions` and the generated entrypoint |
| `may` | permission | — | never checked |
| `should-not` | prohibition | citation site | `--suggestions` and the generated entrypoint |
| `warn-not` | prohibition | citation site | warning, and the generated entrypoint |
| `must-not` | prohibition | citation site | `grund check` error, and the generated entrypoint |

The grammar is: entries in one array are all required, while `|` inside one
entry means any one of its alternatives. Therefore `must = ["FS|GOAL"]`
requires a citation to either `FS` or `GOAL`, whereas `must = ["FS", "GOAL"]`
requires citations to both. Nearly every real rule is one entry with `|`.
A prohibition is the exception: each of its targets is banned on its own, so
`must-not = ["FS", "AR"]` forbids a citation to either, exactly as
`must-not = ["FS|AR"]` does, and the generated entrypoint says `must not cite FS
or AR`.
`grund schema` draws a repository's directions, each `|` entry as one
obligation.

From grund 0.19.0, a prohibited citation of a name under a number produces the ordinary `forbidden-citation` error and makes the check fail. The mixed-only warning window from 0.18.0 has ended. Recommended directions remain opt-in suggestions, and explicit `warn`/`warn-not` strengths remain warnings ([the citation-direction rules](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-rules.md#51-facts-and-identity)).

The citing side may be any configured kind, including a non-citable kind, or
the complement `code` kind. In version 2, the complement may list places of its
own; those places narrow what it covers, and files outside them are not read.
The cited side must be citable, because a citation needs an ID to point at. A
non-citable kind may cite and is labelled by its home, such as `skills/`; the
complement covers files no other configured kind claims and renders last. In a workspace a target may name another project. A bare `KIND` is judged against the local project's kinds when the config loads. `alias/KIND` is judged against the kinds of the project that `alias` names, and `*/KIND` against the kinds of every loaded project, both when the workspace is expanded. So a version-2 `[rules.citations.FS] must = ["api/API"]` at a workspace root loads once member `api` configures `API`, and it is enforced exactly as the rule `Each FS must cite at least one api/API.` is. A pinned or `*/` target the workspace cannot accept fails the load: grund prints one `error:` line naming it and exits 2. This covers an unknown or non-citable kind in that project, and an alias the workspace does not load. A run with no workspace, such as a member's own run, cannot judge a target pinned to another project at all, and its error says to check from the workspace root. That limit is on pinned targets; it does not stop a member from holding a rule of its own: a member that must cite no other project says so in its own config without naming one, as [one member that cites no other project](#one-member-that-cites-no-other-project) shows.

An obligation on a citable kind that declares no IDs has no unit: no
per-declaration unit fires. If a `must` or `should` obligation is configured,
the kind's folder is walked, and at least one non-entry file is successfully
scanned, `grund check` emits the non-fatal run-level
`empty-citation-obligation` warning ([the check specification says exactly when](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#221-citation-direction-obligation-applies-to-nothing)).
A `must` or `should` rule whose subject is that kind earns the same warning,
named by the rule rather than the table, and without the `citable = false`
question, because a rule's subject is always a citable kind
([a rule's obligation](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#221-citation-direction-obligation-applies-to-nothing)).
A directory without declarations is usually `citable = false`, not a citable
kind with an empty rule; make that setting when the directory is a place rather
than an ID namespace. This is the no-unit trap described by
[the non-citable-kinds decision](https://github.com/agent-grounds/grund/blob/main/docs/decisions/functional/DF-non-citable-kinds.md#25-obligations-get-a-per-file-unit-and-grounding-follows-the-home).

A kind's own marked index entry is still subject to citation directions. Keep
the entry when repairing a prohibition: replacing it with a plain link fails
the index rule. For a v2 kind or global negative default, add the target kind
to the citing kind's `may` list while preserving existing permissions. For
example, in version 2's `[rules.citations.REQ]`, extend `may = ["GOAL"]` to
`may = ["GOAL", "REQ"]`. An explicit prohibition instead requires changing
the matching `must-not` or `warn-not` list; an independent rule sentence
requires changing the named rule. Adding `may` does not override either.
The finding names the applicable repair; ordinary citations keep their
plain-link advice, and suggestions keep their existing wording
([the index-entry repair specification](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#312-forbidden-citation)).

The generated entrypoint states the directions a config declares and no
others ([the citation-direction rule of `grund init`](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-init.md#235-citation-directions)).
A config that declares no citation rule has no `### Citation directions`
section in its managed block, whatever kinds it declares. A grounding ladder
declares no rule, so where grounding is on, with
a version-2 `[rules.citations.grounding] must = "file"`, the
section holds the grounding sentence alone, with no legend and no closing line:
`Every source file must cite a declared ID or declare one inline.`

The config and generated section below use one version-2 config. The version-2 example
sets `[presentation] rules = "root"` so all configured guidance renders in one
root block. This is the output of `grund init` on that config:

```toml
grund_config_version = 2

[presentation]
rules = "root"

[schema.kinds.GOAL]
files = ["docs/goals.md"]

[rules.citations.FS]
should = ["GOAL|FS"]
must-not = ["AR"]

[schema.kinds.FS]
folders = ["docs/functional-spec"]

[schema.kinds.AR]
folders = ["docs/architecture"]

[rules.citations.DA]
must = ["AR", "FS"]

[schema.kinds.DA]
folders = ["docs/decisions/architectural"]

[schema.kinds.e2e]
citable = false
folders = ["tests/e2e"]

[schema.kinds.integration]
citable = false
folders = ["tests/integration"]

[schema.kinds.skill]
citable = false
folders = ["skills"]

[rules.citations.skill]
must = ["FS"]
must-not = ["AR"]

[rules.citations.code]
should = ["FS|AR"]

[rules.citations.grounding]
must = "file"
```

```markdown
### Citation directions

`must`/`never` are `grund check` errors; `should`/`avoid` are suggestions (`grund check --suggestions`). Every source file must cite a declared ID or declare one inline; every file under tests/e2e/, tests/integration/, and skills/ must cite one.

- Each **FS** declaration should cite GOAL or FS; never cite AR.
- Each **DA** declaration must cite AR and FS.
- Each file in **skills/** must cite FS; never cite AR.
- Each source file outside the Project map (**code**) that cites anything should cite FS or AR.
Anything not listed above is allowed.
```

The `skills/` bullet carries a second sentence because its rule is about what a
file that cites something cites. A skill that cites nothing slips past
`must = ["FS"]`, and turning the row's grounding on is what closes that hole. A
place closed to every citation has no such hole. In a version-1 config, close a
non-citable `tests/fixtures/` home with a `must-not` default and no list but
`must-not`, and its one rule is that its files cite nothing, which a file that
cites nothing already keeps:

```toml
[citations.fixture]         # citable = false, at tests/fixtures/
default = "must-not"
```

```markdown
- Each file in **tests/fixtures/** must not cite anything.
```

That bullet is one sentence under either config version, whether the row's
grounding is on or off. It has no `that cites anything` and names no grounding
setting, because grounding there would fail every file: an uncited one as
ungrounded, and a cited one as a forbidden citation. A `must-not` list beside
the default folds into the same clause, since every target it names is one the
clause already forbids, and the homeless `code` kind closed the same way reads
`Each source file outside the Project map (**code**) must not cite anything.`
A closed default with a `may`, `should-not` or `warn-not` exception beside it
is not closed, because that target is permitted, discouraged or only warned
about, so it keeps the bullet the levels above give it ([a row closed to every citation](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config.md#3941-a-row-closed-to-every-citation)).

**When grounding and a closed row conflict.** A scanned non-citable home
cannot satisfy grounding if it permits no citation target. For example, in
version 2:

```toml
[schema.kinds.ce]
folders = ["ce"]
citable = false

[rules.citations.ce]
default = "must-not"

[rules.citations.ce.grounding]
must = "file"
```

`grund config validate`, `grund config show` and `grund check` warn, even if
`ce/` is empty:

```text
warning: grund.toml: row `ce`: [rules.citations.ce.grounding] must = "file" asks for citations, but [rules.citations.ce] default = "must-not" permits no target; empty [rules.citations.ce.grounding] to disable grounding for this row, or permit a target in [rules.citations.ce]
```

In version 2, to keep the home citation-free, remove its grounding rung and keep
the empty `[rules.citations.ce.grounding]` table. An empty row ladder replaces
inherited grounding and disables it even under `--require-grounding`. In version 2,
to keep grounding, permit a target instead, for example `may = ["FS"]` in
`[rules.citations.ce]`.
That target must be a declared kind in the project.

The version-1 equivalent is:

```toml
[[kinds]]
kind = "ce"
folder = "ce"
citable = false
require_grounding = true

[citations.ce]
default = "must-not"
```

```text
warning: grund.toml: row `ce`: require_grounding = true on its [[kinds]] row asks for citations, but [citations.ce] default = "must-not" permits no target; set require_grounding = false on its [[kinds]] row to disable grounding for this row, or permit a target in [citations.ce]
```

In version 1, set `require_grounding = false` on that `[[kinds]]` row to keep its
files citation-free, or add `may = ["FS"]` in `[citations.ce]` to permit a target
while keeping grounding.

The warning names inherited project settings when those govern the row, and
names `--require-grounding` only when the flag alone grounds it. Version 2's
`warn` and `should` rungs also earn it without `--suggestions`; they retain
their own severities. The warning changes no file verdict or exit code, but
replaces `success` when it is the only finding. `grund config explain
closed-citation-grounding` describes it ([the warning's contract](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-check.md#checksclosed-citation-grounding-grounding-asks-for-citations-a-closed-row-permits-nowhere)).

Citable homes and the `code` complement get no warning: their source files
can satisfy grounding with inline declarations. Inside a non-citable home,
an inline declaration does not ground the file. Unscanned places and rows
that permit a target or disable grounding get no warning either.

grund's own convention climbs: goals cite why the project exists, specs cite
goals, architecture and decisions cite specs, and code and executable tests cite
the specs they realize. Citation obligations judge only declarations that
already contain a citation, so uncited source and test files pass at every
level unless a version-2 `[rules.citations.grounding]` is configured. A config holds none of
the convention until it declares it, so it is a recommended starting ruleset
rather than a default. Over the
kinds `grund init`'s starter config declares, in version 2 and at `should`, it
reads:

```toml
[rules.citations.GOAL]
should = ["GRUND|GOAL"]

[rules.citations.FS]
should = ["GOAL|FS"]

[rules.citations.REQ]
should = ["GRUND|GOAL"]

[rules.citations.AR]
should = ["FS|GOAL"]

[rules.citations.DF]
should = ["FS|GOAL"]

[rules.citations.DA]
should = ["AR|FS"]

[rules.citations.e2e]
should = ["FS"]

[rules.citations.integration]
should = ["AR"]

[rules.citations.code]
should = ["FS|AR"]
```

Start there, run `grund init` to refresh the managed block, then run
`grund check --suggestions`. In version 2, add `[rules.citations.grounding] warn = "file"`
for an intermediate warning-only step, then promote it to `must = "file"` and
each kind's rule to `must` once the repository passes. Choose a different set
only where the repository's documents genuinely depend on each other
differently.

In a version-2 config, `grund init` writes each kind's directions to an
`AGENTS.md` beside that kind's places and lists those files in the root block;
`[presentation] rules = "root"` keeps every rule in the root file. See
[scoped guidance in a version-2 config](https://github.com/agent-grounds/grund/blob/main/docs/user-facing/init.md#scoped-guidance-in-a-version-2-config).

## One member that cites no other project

Sometimes one member of a workspace must cite no other project, such as an open
edition that must never cite its closed sibling. Write that rule in the
member's own `grund.toml`. In a version-2 member, close its citation default,
then list each citing kind's own targets under `may`:

```toml
# ce/substratevm/grund.toml, after the member's kinds
[rules.citations]
default = "must-not"

[rules.citations.FS]
may = ["FS"]

[rules.citations.code]
may = ["FS"]
```

The member names no other project, yet it may cite only its own. A bare `FS`
matches the local project's `FS` only
([alias matching](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config.md#393-alias-matching)).
An explicit list wins over the default
([defaults and precedence](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config.md#394-defaults-and-precedence)),
so the member's citations of its own `FS` pass. A citation of any other project
is on no list, so it falls to the default, and a default may forbid
([`rules.citations.default`](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config-v2.md#rulescitationsdefault-a-default-may-forbid-never-oblige)).
Run from the workspace root, the example refuses two citations of the closed
member `ee`: one from the open member's `FS` file, and one from a file outside
every kind home:

```text
ce/substratevm/docs/notes.md:1: error: code must not cite ee/FS (citation direction) — re-point the citation or downgrade it to a plain Markdown link
ce/substratevm/docs/requirements.md:6: error: FS must not cite ee/FS (citation direction) — re-point the citation or downgrade it to a plain Markdown link
```

The rule has to live in the member. A workspace root is never consulted for a
member's citation sites
([the project is the widest committed scope](https://github.com/agent-grounds/grund/blob/main/docs/functional-spec/FS-config.md#principleboundary-the-project-is-the-widest-committed-scope)).
So `[rules.citations.FS] must-not = ["ee/FS"]` at a version-2 root judges only
the root's own files, and the member's citations of `ee` still pass.

The recipe has two costs.

Every citing kind must list its own targets under `may`. That includes each
non-citable kind, such as `skills/`, and the complement `code` kind. A kind left
out falls to the closed default, so its local citations fail too. With the
`code` row dropped, the example's complement file reports:

```text
ce/substratevm/docs/notes.md:1: error: code must not cite FS (citation direction) — re-point the citation or downgrade it to a plain Markdown link
```

In version 2, if an omitted non-citable home also inherits grounding, it earns
`closed-citation-grounding`: the warning names the member's config, its
inherited `[rules.citations] default = "must-not"` and the grounding setting.
Open that home's local targets under `may`, or give it an empty row grounding
table. In version 1, open targets under `[citations.<KIND>]` or set
`require_grounding = false` on the row. Closing a member to other projects
does not make its grounded homes able to cite local targets automatically.

Allowing one sibling but not another takes a pinned target, such as
`may = ["FS", "compiler/FS"]`. The workspace root accepts it. A member's own run
has no workspace to resolve the alias in, so it refuses the config and exits 2:

```text
error: grund.toml:14: [rules.citations.FS] may names `compiler/FS`: unknown project alias compiler; no workspace is in scope here, so the alias cannot be resolved — check from the workspace root; this became an error in grund 0.19.0
```

The [`workspace-isolated-member` example](https://github.com/agent-grounds/grund/blob/main/examples/workspace-isolated-member/README.md)
runs the recipe and both costs.

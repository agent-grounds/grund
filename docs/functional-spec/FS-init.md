# FS-init: grund bootstraps a new grund-conformant repo

The `init` subcommand writes the minimum set of files a project needs to start using `grund` — an agent entrypoint ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)) and a bare `<path>/grund.toml` config ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)) — so that a fresh repo, or an existing repo adopting the scheme, becomes scannable in one command. It is the minimum-effort, non-intrusive on-ramp, and every behavior below serves that: every default is the most-common case; a config the repo already has, in either form, is kept; the versioned `grund` block goes into the agent entrypoints the repo already has, and only a repo with none gets the canonical `AGENTS.md` ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)); nothing the user authored is rewritten without `--force`; and `--dry-run` previews any run before it touches the working tree. [§FS-init.3](FS-init.md#3-non-intrusive-guarantees) consolidates those guarantees. Serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers) (the managed block is the instruction layer), [§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree) (the emitted defaults are the canonical grammar), and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) (no hidden prompts; the same input produces byte-identical output).

`init` is the only `grund` subcommand that creates adoption scaffolding in the working tree. In an existing agent entrypoint it may update the managed `grund` block, and it rewrites no other user-authored content there unless the file is the canonical `AGENTS.md` and `--force` is passed ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)).

Verbose implementer fixtures — exact stderr transcripts, final tree expectations, and common existing-file cases — are in [§FS-init-fixtures](FS-init-fixtures.md#fs-init-fixtures-concrete-init-fixtures); they are examples of this spec, not a separate feature.

## 1. Inputs

```
grund init [<path>] [--name <name>] [--description <text>] [--docs] [--force] [--dry-run] [--check] [--no-vcs] [--agents-md] [--claude] [--gemini] [--pi] [--copilot] [--cursor] [--windsurf] [--zed]
```

- `<path>` — directory in which to scaffold; defaults to `.`. Applies to every form of `init`, `--docs` included, and prefixes every emitted path in [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place). Must exist: `init` does not create the target directory, because a missing target is a user error, not something to silently paper over.
- `--name <name>` — human-readable project name baked into the generated `AGENTS.md` heading when canonical `AGENTS.md` is selected, and into the `grund.toml` `project_name` key. Defaults to the basename of `<path>` resolved to an absolute path.
- `--description <text>` — one-line project description baked into the generated `grund.toml` `project_description` key ([§FS-config.3](FS-config.md#3-schema)), replacing the commented teaching line [§FS-init.2.4](FS-init.md#24-generated-grundtoml) writes by default. No default: `init` never invents a description. A `<text>` containing a line break is a CLI error, mirroring the config-side single-line rule. Like `--name`, the value only lands in a freshly written config — an existing `grund.toml` is never modified ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)).
- `--docs` — also scaffold the docs the effective config references: [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place) lists the default set, and [§FS-init.1.3](FS-init.md#13---docs-and-the-effective-config) says why each is there and how the config changes it. Off by default — most adopters already have a `docs/` of some shape and want only the entry point and config.
- `--force` — overwrite a selected canonical `AGENTS.md` and the `--docs` scaffold files where they already exist, never the config ([§FS-init.3.4](FS-init.md#34-with---force), [§FS-init.3.5](FS-init.md#35-what---force-does-not-replace)). Off by default. Existing entrypoints are appended to or updated without it ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), so it is only needed to reset a generated `AGENTS.md` or a `--docs` scaffold to its canonical bytes.
- `--dry-run` — preview the run without writing or modifying any file: each `wrote `, `appended `, or `updated ` line of the report ([§FS-init.2.2](FS-init.md#22-stdout--stderr)) is emitted as `would-write `, `would-append `, or `would-update `, while `exists ` lines and the `next:` block are unchanged. Composes with every other flag, including `--force`. Off by default.
- `--check` — the same preview, taken as a verdict: nothing is written, the report is the one `--dry-run` prints for this tree ([§FS-init.2.2](FS-init.md#22-stdout--stderr)), and the run exits `1` when any line of that report is a `would-…` ([§FS-init.4](FS-init.md#4-exit-codes)). It is what a pre-commit hook or a CI job runs; [§FS-init.1.4](FS-init.md#14-why---check-exists) says why it exists. Composes with every other flag exactly as `--dry-run` does, `--force` included; passing both is redundant, not an error, since both say *write nothing*. Off by default — bare `grund init` still writes.
- `--no-vcs` — scaffold into a target that no version-control marker covers, which is otherwise refused ([§FS-init.1.2](FS-init.md#12-refused-targets)). Off by default. It says only *this really is where I want a project*: it does not lift the unconditional refusals of [§FS-init.1.2](FS-init.md#12-refused-targets), and it is not `--force` (which decides whether files `init` owns get overwritten, [§FS-init.3](FS-init.md#3-non-intrusive-guarantees)) — the two answer different questions, and a run may need either, both, or neither.
- `--agents-md`, `--claude`, `--gemini`, `--pi`, `--copilot`, `--cursor`, `--windsurf`, `--zed` — explicitly create or update that agent's entrypoint ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)); with none of them, `init` runs in automatic mode.

Per [§FS-non-goals.10](FS-non-goals.md#10-interactive-mode), `init` is non-interactive: it never prompts. Every choice is a flag.

### 1.1 Usage examples

```
grund init                       # cwd, no docs tree
grund init path/to/repo          # auto-detect existing entrypoint, else AGENTS.md
grund init --docs                # cwd, with docs/ + tests/e2e/ + tests/integration/ scaffolds
grund init --docs path/to/repo   # explicit target, with docs/ + tests/e2e/ + tests/integration/ scaffolds
grund init --dry-run path/to/repo # preview without writing anything
grund init --check path/to/repo  # same preview as a gate: exit 1 if anything is pending
grund init --claude --gemini path/to/repo # create/update both agent entrypoints
grund init --docs --name acme path/to/repo  # full form
```

Argument order is flexible: positional `<path>` may appear before or after the flags.

### 1.2 Refused targets

`init` is the only subcommand that can write a file the user never named, and every path it writes is `<path>`-relative, which makes `<path>` load-bearing in a way no other command's is. So before it touches anything, `init` checks *where* it was pointed. Every check below runs before the first write, and a refusal is total: no file is created, appended, or updated, not even the ones that would have been unobjectionable, and `init` exits `2` ([§FS-init.4](FS-init.md#4-exit-codes)). `--dry-run` reports the same refusal rather than a preview — a preview of a run that would be refused *is* that refusal.

- **The home directory is refused, unconditionally.** A `<path>` that resolves to `$HOME` is declined and nothing is written. No flag lifts this — not `--no-vcs`, not `--force` ([§FS-init.1.2.1](FS-init.md#121-the-home-directory)).
- **A user-global instruction file is refused, unconditionally.** `init` declines when any path it would write, append to, or update — the canonical `AGENTS.md` included — is one of the file-backed user-global targets of [§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions) ([§FS-init.1.2.2](FS-init.md#122-user-global-instruction-files)).
- **A target outside version control is refused unless `--no-vcs` is passed.** `init` looks for a `.git`, `.hg`, `.jj`, or `.svn` entry in `<path>` or any ancestor and declines when it finds none, naming the flag that proceeds anyway ([§FS-init.1.2.3](FS-init.md#123-version-control)).

Which of two problems a run is told about first is [§FS-init.1.2.4](FS-init.md#124-which-refusal-is-reported-first), and what a refusal says is [§FS-init.1.2.5](FS-init.md#125-the-refusal-message).

#### 1.2.1 The home directory

`$HOME` is where the file-backed user-global agent instruction files of [§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions) live, and `init`'s repository-relative model is wrong about every one of them: `<path>/.claude/CLAUDE.md` with `<path>` at `$HOME` **is** `~/.claude/CLAUDE.md`, the machine-global file every agent session in every project loads. The collision is not incidental. The same signal — `.claude/` exists — means "this machine runs Claude, sync the small marked rendering block" to `grund integrations --write` and "this project uses Claude, scaffold the managed block" to automatic mode ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)). Both readings are correct in their own scope; in `$HOME` they name one file, and the project-shaped one would append a specific repository's instructions to every session the user ever starts. Nobody targets `$HOME` on purpose, so there is no case to keep working and no escape hatch to offer.

The home directory is whichever one the platform reports, so the rule holds wherever `init` runs. The user-global table [§FS-init.1.2.2](FS-init.md#122-user-global-instruction-files) checks is resolved separately — against `$HOME`, save that a `~/.config/…` row such as Zed's follows `$XDG_CONFIG_HOME` when it is set ([§FS-integrations.4.1.7](FS-integrations.md#417-where-a--target-resolves)) — so on a platform that does not set `$HOME` this rule is the one that still answers for `<path>` at the home directory itself.

#### 1.2.2 User-global instruction files

This is not the `$HOME` rule restated: that one fires when `<path>` **is** the home directory, and `<path>` is arbitrary. `<path>/AGENTS.md` with `<path>` at `~/.codex`, `~/.config/zed`, or `~/.pi/agent` **is** that table's file, as is `<path>/GEMINI.md` with `<path>` at `~/.gemini` and `<path>/CLAUDE.md` with `<path>` at `~/.claude` — each a target the `$HOME` rule has nothing to say about, and on a machine whose dotfiles are a repository, which is the usual state of a machine that has those directories, the version-control rule has nothing to say either. Three of those five are the canonical `AGENTS.md`, the entrypoint `init` reaches for by default and the one file it *overwrites* rather than appends to under `--force` ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)), so this rule is checked for every planned path including the canonical one.

The division is the one [§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions) already states: the user-global files carry machine-wide *policy* and are `grund integrations --write`'s to manage, the repository entrypoint carries this project's *syntax* and is `init`'s.

#### 1.2.3 Version control

Presence is tested, not type: a linked worktree and a submodule both write `.git` as a file. This is not `grund` reading history — nothing is parsed, no command is run, and [§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking) is untouched; the marker's existence is a fact about the tree in exactly the way a `grund.toml`'s is. Unlike the two rules above this one has a legitimate other side — scaffolding a directory before `git init`, or a project under no VCS at all — so it is a default, not a law, and `--no-vcs` lifts this rule only ([§FS-init.1](FS-init.md#1-inputs)).

#### 1.2.4 Which refusal is reported first

The two location rules are answered from `<path>` alone, so they are reported before anything is read. The user-global rule is not: the paths it checks are the entrypoints the run plans, and that plan depends on the effective configuration ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)), so a configuration `init` cannot parse is reported first — the same message and exit code `grund check` gives for that file ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)). Both refuse the run and write nothing; only which of the two problems the message names differs, and a target that is both wrong and unreadable is told about the file it could not read before the place it was pointed.

#### 1.2.5 The refusal message

A refused run says which rule declined it and, where one exists, the flag that proceeds:

```
error: refusing to scaffold into the home directory /home/you — init writes repository paths, and .claude/CLAUDE.md here is the machine-global agent instruction file
error: refusing to write /home/you/.claude/CLAUDE.md — that is the machine-global agent instruction file, managed by `grund integrations --write`, not a repository entrypoint
error: /tmp/nowhere is not inside a version-controlled tree — no .git, .hg, .jj, or .svn here or above; pass --no-vcs to scaffold anyway
```

These are refusals, not prompts: `init` still never asks a question ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)); it declines and names the flag that would have let the run through. That is the only shape a guard can take in a non-interactive command, and it is why the two unconditional rules have no flag at all — a rule nobody means to trip does not need one.

### 1.3 `--docs` and the effective config

The root-level `requirements.md` stub is scaffolded because the generated `FS` kind uses it as the default requirements/spec home ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)). An existing config that omits `[[kinds]]` instead keeps the compatibility FS home from [§FS-config.2](FS-config.md#2-precedence), so `--docs` scaffolds `docs/functional-spec/README.md` and points next-step guidance at `docs/functional-spec`. `roadmap.md` and `changelog.md` are scaffolded because the generated managed block's `docs/` table links to them ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)).

### 1.4 Why `--check` exists

A managed block can go stale in its *text* while its `(vN)` heading is still current. `--dry-run` has always seen and reported that drift, but as a preview rather than a verdict; `grund check` does not see it at all outside the `### Citation directions` and `### Clickable citations` sections — the two it re-renders and byte-compares ([§FS-init.2.3.5](FS-init.md#235-citation-directions), [§FS-init.2.3.6](FS-init.md#236-clickable-citations)) — because everywhere else it verifies the block's version rather than its rendered bytes ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)).

`--check` is the counterpart of `grund fmt --check` ([§FS-fmt.1](FS-fmt.md#1-inputs)), and like it a report of pending work rather than a refusal: what earns the `1` is a file `init` would have written, and the one-command fix is the same `init` run without the flag.

### 1.5 Agent-entrypoint flags

Each flag explicitly creates or updates one agent's entrypoint:

- `--agents-md` — the canonical `<path>/AGENTS.md`, even when another agent entrypoint already exists.
- `--claude` — the Claude entrypoint: every one of `<path>/CLAUDE.md` and `<path>/.claude/CLAUDE.md` the repository already has, or `<path>/CLAUDE.md` where it has neither ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)).
- `--gemini` — `<path>/GEMINI.md`.
- `--pi` — `<path>/.pi/AGENTS.md`.
- `--copilot` — `<path>/.github/copilot-instructions.md`.
- `--cursor` — the Cursor entrypoint: `<path>/.cursor/rules/grund.mdc`, and any legacy `<path>/.cursorrules` the repository already has. The legacy form is never created, and where the repository's Cursor rules already live in it, that file is updated rather than the modern one being added beside it ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)).
- `--windsurf` — `<path>/.windsurfrules`.
- `--zed` — `<path>/.rules`.

When no explicit agent-entrypoint flag is passed, `init` runs in automatic mode: update existing known agent entrypoint files if any are present; otherwise create agent-directory-triggered companions for `.claude/`, `.gemini/`, `.pi/`, `.cursor/`, or `.zed/` if those agent directories already exist; otherwise create canonical `AGENTS.md` ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)). When one or more explicit agent-entrypoint flags are passed, `init` writes exactly those requested entrypoint families and adds no automatic fallback.

## 2. Outputs

### 2.1 Files written, updated, or left in place

In the default form (no `--docs`):

- Agent entrypoints — see [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints). In automatic mode ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)), existing known entrypoints are appended or updated in place: `<path>/AGENTS.md`, `<path>/AGENTS.override.md`, `<path>/CLAUDE.md`, `<path>/.claude/CLAUDE.md`, `<path>/GEMINI.md`, `<path>/.pi/AGENTS.md`, `<path>/.github/copilot-instructions.md`, `<path>/.cursor/rules/grund.mdc`, `<path>/.cursorrules`, and `<path>/.windsurfrules`, excluding companion symlinks to `AGENTS.md`. If none exist, a missing companion is created for each agent whose agent directory already exists, one per agent ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)); if there are still no entrypoints to update or create, canonical `AGENTS.md` is written. [§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection) gives the rules of each step. Explicit flags (`--agents-md`, `--claude`, `--gemini`, `--pi`, `--copilot`, `--cursor`, `--windsurf`, `--zed`) create or update their requested entrypoints regardless of automatic detection, under the same one-per-agent rule ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)).
- `<path>/grund.toml` — see [§FS-init.2.4](FS-init.md#24-generated-grundtoml). Written only when the target carries no config in either discovery form ([§FS-config.1](FS-config.md#1-file-location-and-discovery)); a target that already has one is reported with `exists ` at the name it was found under (`exists .agents/grund.toml` for a repo on that form) and gets no second config.

With `--docs`, additionally — each a minimal starter ([§FS-init.2.1.3](FS-init.md#213-the---docs-stubs)):

- `<path>/requirements.md`
- `<path>/docs/grund.md`
- `<path>/docs/goals.md`
- `<path>/docs/roadmap.md`
- `<path>/docs/changelog.md`
- `<path>/docs/architecture/README.md`
- `<path>/docs/decisions/architectural/README.md`
- `<path>/docs/decisions/functional/README.md`
- `<path>/tests/e2e/README.md`
- `<path>/tests/integration/.gitkeep`

The exact bytes for a given `grund` version are embedded in the binary; reference copies live under `templates/` in the `grund` source tree, and two `grund init --docs` runs at the same version with the same `--name` produce byte-identical scaffolds ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). `grund check` is clean against the freshly-scaffolded tree.

#### 2.1.1 One entrypoint per agent

Some agents read more than one file: Claude reads `<path>/CLAUDE.md` and `<path>/.claude/CLAUDE.md`, and Cursor reads `<path>/.cursor/rules/grund.mdc` and the legacy `<path>/.cursorrules`. The managed block is the same bytes in each of them, so writing two hands that agent the whole block twice — the cost [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file) exists to avoid, paid for nothing — and the file the user did not ask for is one they must remember to delete again after every upgrade.

`init` therefore **creates at most one entrypoint per agent, and only for an agent that has none**. For each selected agent:

- Every entrypoint the repository already has is appended to or updated in place, however many there are. An existing file is the user's; `init` maintains what it finds and does not decide which of their files to abandon. *Has* is decided on the evidence [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place) already uses to select one, not on the path being occupied: a file at a name too generic to attribute by itself — `.rules` — is that agent's entrypoint only where the owning agent directory or a managed block from a previous run says so, and a looser reading here would let this rule and `grund check` disagree about what an entrypoint is ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)).
- An agent with no entrypoint of its own gets exactly one created: the first of its paths in the [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place) list that is free and that automatic mode or the explicit flag may create at all (so Cursor's legacy `.cursorrules` is still never the one created). For Claude that is `<path>/CLAUDE.md`, whichever signal triggered the run ([§FS-init.2.1.1.2](FS-init.md#2112-claudes-created-entrypoint)).
- A companion that is a *symlink* to `AGENTS.md` counts as the agent having one. The agent reads the canonical block through it ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)), so a second file beside it would be the same bytes twice — the case this rule exists to stop, reached by another route. The single exception is a repository committing `[reference] conversation = "link"` ([§FS-init.2.1.1.1](FS-init.md#2111-a-symlinked-companion-under-conversation--link)).

The rule governs creation, not maintenance: a repository that already carries two entrypoints for one agent keeps both and both get the current block. That state is reported rather than silently maintained — the run emits one `note:` line naming the files and the agent that reads them all ([§FS-init.2.2.1](FS-init.md#221-notes)), because the run that just wrote to each of them is the only place the duplication is visible.

##### 2.1.1.1 A symlinked companion under `conversation = "link"`

A repository committing `[reference] conversation = "link"` is the one case in which the two files differ: its canonical `AGENTS.md` carries the plain-location form that the Claude entrypoints are gated away from ([§FS-init.2.3.4.17](FS-init.md#23417-clickable-citations)). There the symlink resolves to a form Claude is not meant to read, the agent has nothing carrying its own, and an explicit request writes it the first free path the symlink has not taken — which is what [§FS-init.2.3.4.17](FS-init.md#23417-clickable-citations)'s `note:` sends the user to `grund init --claude` for. Where symlinks have taken *every* path that agent reads there is no free path, so the run writes nothing and the note names the only fix left: delete one of them first. Without that key, the surfaces are identical and the symlink is simply the agent's copy.

##### 2.1.1.2 Claude's created entrypoint

`<path>/CLAUDE.md` is the root-visible form, chosen for the reason `init` also generates the bare `grund.toml` ([§DF-config-file-location.2.3](../decisions/functional/DF-config-file-location.md#23-grund-init-writes-the-bare-grundtoml)): it is the form the rest of the ecosystem uses, and it adds no hidden directory to a repository that has none. A repository that wants the directory form instead creates `<path>/.claude/CLAUDE.md` — empty is enough — and re-runs; `init` fills in the block, because it updates every entrypoint that exists.

The tiebreak is not re-decided by the signal that triggered the run. In automatic mode a missing companion is created because the agent's directory exists, and `.claude/` is inside the very form this rule does not pick; `init` still writes `<path>/CLAUDE.md`. The agent directory is evidence that the *agent* is in use — the same fact `.gemini/` carries for `GEMINI.md` — and reading it as evidence for a *location* would make the file an agent gets depend on which of two signals reached it, for a repository whose two entrypoints are interchangeable.

#### 2.1.2 Automatic entrypoint selection

A companion symlink to `AGENTS.md` selects the canonical `AGENTS.md` target instead, including when the symlink is dangling because `AGENTS.md` has not been created yet. The agent-directory-triggered companions, each created only when its owning agent directory already exists: `.claude/` creates `CLAUDE.md`, `.gemini/` creates `GEMINI.md`, `.pi/` creates `.pi/AGENTS.md`, `.cursor/` creates `.cursor/rules/grund.mdc`, and `.zed/` creates `.rules`.

`AGENTS.override.md`, `.github/copilot-instructions.md`, `.cursorrules`, and `.windsurfrules` are automatic existing-file-only — `AGENTS.override.md` is an override channel; `.github/` is generic GitHub metadata; `.cursorrules` is Cursor's legacy single-file form (the modern `.cursor/rules/` directory is preferred when creating new); and `.windsurfrules` is a root file with no companion directory to key off, so creating one requires the explicit `--windsurf` flag. `.rules` is never detected by file existence alone, because the filename is too generic to attribute to Zed by itself: an existing `.rules` is Zed's entrypoint only where `.zed/` exists or a managed block from a previous run is already in it ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)).

#### 2.1.3 The `--docs` stubs

Each scaffolded markdown file is a minimal starter — enough structure to teach the layout, no real content:

- `grund.md` — the H1 plus a one-line note on how the project's reason for being is declared inline (`# GRUND-NNN-slug: …`), then the three H2 sections (`## 1. The problem`, `## 2. What this project does about it`, `## 3. Who it is for`), each with a one-line italic prompt to be replaced.
- `goals.md` — the H1 plus a one-line note on how goals are declared inline (`# GOAL-NNN-slug: …`).
- `requirements.md` — the H1 plus a one-line note on how `FS-` requirements/spec IDs are declared inline as H2 headings in that file.
- `roadmap.md`, `changelog.md` — the H1 plus a single `<!-- placeholder - replace with real content -->` line.
- `architecture/README.md` — the H1, the navigational note about how `AR-` IDs declare into the directory and the rule that the index lists every architecture declaration as a full link ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)), and an empty `| ID | Subject |` table to fill in.
- `decisions/architectural/README.md`, `decisions/functional/README.md` — the same shape for the two decision folders: the H1, the note on how `DA-`/`DF-` IDs declare into the directory and on citing a decision from the spec point it settles, the index rule, and an empty `| ID | Subject |` table. Every citable `folder` kind whose `index` the generated config leaves at its default gets one, which under those defaults is `AR`, `DF`, and `DA` — the two test kinds are non-citable and have no index at all ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)).
- `tests/e2e/README.md` — the H1 (`# e2e`) plus a note that every behaviour described in the effective FS home has at least one case, and that each case cites the spec point it proves. It is a layout note, not an index: `e2e` is a non-citable kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) and has no declarations to list. `tests/integration/` gets the `.gitkeep` instead of a README of its own, so the second configured test home survives a `git add` without a second near-identical note.

The `.gitkeep` file exists solely so an empty directory that gets no README of its own survives a `git add`; its content is a single line: `# placeholder — replace this directory's contents with real integration tests`.

### 2.2 Stdout / stderr

On success, stderr lists every path touched, one per line:

```
wrote AGENTS.md
wrote grund.toml
```

The prefix is `wrote ` for a newly written or overwritten file, including an agent-directory-triggered companion; `appended ` when the versioned `grund` block was added to an existing agent entrypoint; `updated ` when an existing managed block's bytes changed (an older block upgraded, or a same-version block re-rendered against a changed template or config — [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)); and `exists ` when an existing file was left unchanged — including an agent entrypoint whose managed block already matches the current rendered block byte-for-byte, in which case `init` rewrites nothing, so re-running `grund init` on an up-to-date repo touches no files. Under `--dry-run`, those three write prefixes are replaced with `would-write `, `would-append `, and `would-update ` and nothing is written to disk; `exists ` lines are identical to a non-dry run. A companion file that is a symlink to `AGENTS.md` is omitted from the output; when the canonical file is selected, that update already covers the symlink target.

After the file list come any `note:` lines ([§FS-init.2.2.1](FS-init.md#221-notes)), then the `next:` block ([§FS-init.2.2.2](FS-init.md#222-the-next-block)). `--check` ([§FS-init.1](FS-init.md#1-inputs)) prints the same conditional report as `--dry-run` and everything after it — the `note:` lines, the `next:` block, and the rule that suppresses it — so the two runs differ in one thing only: the exit code each draws from what it printed ([§FS-init.4](FS-init.md#4-exit-codes)).

Paths are relative to `<path>`. Stdout is always empty ([§FS-errors.6](FS-errors.md#6-the-grund-init-transcript) — `init`'s output is the scaffold it wrote to disk, and the transcript is progress, not output).

#### 2.2.1 Notes

The run's `note:` lines, one per line, each name something the run could not do, or a state of the tree the caller would otherwise have to notice for itself: a duplicated entrypoint ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)), a Claude entrypoint shadowed by a symlink ([§FS-init.2.3.4.17](FS-init.md#23417-clickable-citations)). A note is a report, not a finding: it changes nothing, and the exit code is the one the run would have had without it ([§FS-init.4](FS-init.md#4-exit-codes)).

Under `--dry-run` a note is written in the conditional, like every other line of that report: the run wrote nothing, so a note in the present tense describes a tree that does not exist — and an instruction that assumes the write already happened, such as deleting the symlink a preview did not replace, costs the reader the only entrypoint they have. What such an instruction waits on is the **block**, not the file: a preview's `would-append` names an entrypoint that already exists, so a condition the reader can satisfy by looking is no condition at all.

#### 2.2.2 The `next:` block

After the notes, stderr prints a short `next:` block — a blank line, then numbered first steps, then `see <entrypoint> for the full workflow.` — so the user is not left at a bare list of paths wondering what to do. `<entrypoint>` is the first agent entrypoint `init` wrote, updated, or found already current. The steps are: run `grund check` (a fresh tree is clean); allocate an ID with `ID=$(grund id FS "…")` and add it to the effective FS home (`requirements.md` with the `## <ID>: …` H2 for generated defaults, or under the configured FS folder with `# <ID>: …` when compatibility or explicit config makes FS a folder); cite it as `§<ID>` from the docs and e2e tests that depend on it.

When `--docs` was *not* passed, a re-run advice is inserted as step 1, the following steps move down, the allocation step drops its second line — the one naming the `## <ID>: …` H2 or `# <ID>: …` H1, which prints only under `--docs` — and the citation step is omitted. That advice always points at re-running with `--docs` (or creating the effective FS home, `docs/`, and `tests/` by hand). It appends the clause `until then \`grund check\` has nothing to scan` only when the effective scanner would read no file — exactly the scan decision in [§FS-config.3.5](FS-config.md#35-scan--what-gets-walked): configured extensions, ignore and exclude rules, hidden-file and symlink handling, workspace boundaries, configured include paths, walked kind homes, and `scan = false` all retain their scanner meanings. Path existence or a non-empty directory alone does not make the clause false.

The `next:` block is suppressed entirely when every reported path is `exists ` (or, under `--dry-run`, every reported path is `exists ` and no `would-…` lines were emitted) — the user already has a complete, current grund setup, so there is no next step to teach. When it is printed, it is guidance, not a finding: part of the success output, with no effect on the exit code.

### 2.3 Generated agent entrypoints

The emitted agent guidance is a canonical managed block: it teaches the session-start workflow and rules listed in [§FS-init.2.3.4](FS-init.md#234-managed-block-content-points), rendered against the target repo's effective configuration ([§FS-init.2.3.8](FS-init.md#238-substituted-content)). The block stays concise under [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file): it teaches only the rules an agent needs before work begins, leaving detail to cited specs and `grund <ID>` output.

The contract this spec makes is the *determinism and versioning*, not a literal transcript: two `grund init` runs at the same `grund` version against trees with the same `--name` and the same effective config produce byte-identical managed blocks ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)), and the block's fixed text is versioned ([§FS-init.2.3.7](FS-init.md#237-the-block-version)). Generated `init` output must pass `grund check` unmodified — the generator and the checker are never allowed to disagree about the block's own contents. The block is bounded by delimiters ([§FS-init.2.3.9](FS-init.md#239-delimiters)), written into an entrypoint by the rules of [§FS-init.2.3.10](FS-init.md#2310-writing-the-block) unless it is malformed ([§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)), and carried by the entrypoints [§FS-init.2.3.12](FS-init.md#2312-supported-agents-and-their-entrypoints) names.

#### 2.3.1 Position invariants

The managed block's **position within an existing agent entrypoint is preserved on update**. `init` does not move the block: a supported block sandwiched between user-authored sections is replaced in place with the current rendered block, with the surrounding sections — both before and after — left byte-identical. When the block is already byte-identical to the current render, nothing is rewritten at all (`exists `, [§FS-init.2.2](FS-init.md#22-stdout--stderr)) — the strongest form of "position preserved."

#### 2.3.2 Line endings

`init` does not normalize line endings. When updating or reading an existing agent entrypoint:

- The bytes outside the managed region ([§FS-init.2.3.9](FS-init.md#239-delimiters), delimited or legacy) are preserved byte-for-byte, including CRLF (`\r\n`) and lone-CR endings.
- The delimiter and H2 marker matching tolerates an optional trailing `\r` before the line end, so a CRLF-encoded file is detected correctly.
- The freshly-written block uses LF endings (the bytes embedded in the binary). On a CRLF-encoded host file the result is mixed line endings inside the managed region and CRLF outside; this is intentional. Normalizing the rest of the file would violate the "leave content alone" guarantee.

#### 2.3.3 Citation form

The managed block teaches a single citation form — `§<ID>`, bare, with an optional `.<section>` path. `[fmt.cross_refs].enabled` per [§DF-md-link-emission.2.4](../decisions/functional/DF-md-link-emission.md#24-default-on-for-generated-configs) governs whether `grund fmt --write` wraps on-disk citations in Markdown links; the managed block's guidance is the same in either mode — cite specs by ID; any Markdown wrap is generated by `grund fmt`, not hand-authored.

The generated `grund.toml` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)) sets `[scan] include = ["requirements.md", "docs", "e2e", "src"]`, naming `requirements.md` so the generated single-file FS home is scanned.

#### 2.3.4 Managed-block content points

The generated managed block is not a literal transcript in this spec ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), but its canonical template must preserve the following separately citable content points. Each point is phrased compactly in the template, because the entrypoint is read at session start and serves [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file).

##### 2.3.4.1 Entrypoint

The block identifies itself as the entrypoint for agents working in the project and instructs them to read it.

##### 2.3.4.2 Reference Scheme

The block teaches the configured reference scheme: IDs have the configured shape, citations use the configured marker and optional section path, `grund check` validates citations, `grund list` discovers IDs, `grund <ID>` reads cited declarations or sections, and `grund refs` reports what leans on a declaration.

##### 2.3.4.3 Cheap Grounding

The block teaches the cheap grounding ladder: use `grund <ID>` as the first read for a bare citation, `grund <ID> --brief` when the heading and first paragraph are enough, `grund <ID> --toc` when section navigation is needed, `grund <ID>.<section>` for section citations, `grund <ID> --full` only when a narrower read is insufficient, `grund list --kind FS,AR` for scoped discovery, and `grund refs <ID> --summary` before a full back-reference listing. Beside that ladder it teaches `grund list --size=words --top 10` as the sweep to run when a specification feels heavy, so the agent can move detail into citable child points before paying for an oversized lead.

##### 2.3.4.4 Project Map

The block describes the configured kind homes from the effective `grund.toml` as a raw-readable Markdown link list (`- [KIND](home): Title`), so the generated instructions name and link the host repo's actual artifact layout instead of hard-coding the default layout or relying on table rendering. A kind that is a place rather than an ID namespace gets the row [§FS-init.2.3.4.4.1](FS-init.md#23441-rows-for-places) gives it, or none.

The scan scope (`[scan].include` / `[scan].exclude`) is *not* surfaced here — it is configuration an agent never needs to read inline, since `grund <ID>`, `grund list`, and `grund refs` apply it transparently. Where the init target lies in a workspace, whether or not its own config declares `[workspace]`, the sibling [§FS-init.2.3.4.15](FS-init.md#23415-workspace-members) block names the workspace projects; the Project Map itself describes the *current* project's declaration homes only and is unchanged in workspace mode.

###### 2.3.4.4.1 Rows for places

A **non-citable kind** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is named by its **place** rather than its name — `- [skills/](skills): Agent review and automation skills` — because its name is a config handle an agent can never write in a citation, while the directory is something it can go and read. An unwalked kind ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)) is rendered the same way; the row is the point of configuring it. The **homeless kind** ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) — `code`, or whatever the project named it — has no place and therefore no row.

##### 2.3.4.5 Declaration Forms

The block teaches that declarations are heading lines in markdown or supported language doc-comments, and that an inline source declaration can be represented by a one-line markdown stub in the configured kind home. It also teaches, rendered according to `[id] section_heading_levels`, that numbered headings inside a declaration are citable sections; depth-matching headings (`## 1. …`, `### 1.1 …`) are required in strict mode, warned on in warn mode, and recommended in loose mode. When the effective config enables named sections, the same block additionally teaches the explicit complete-path forms `## goals: Goals`, `### goals.performance: Performance`, and `### goals.3: Ordered child`, plus the reserved `number.name` order. Disabled repositories receive no named-heading guidance. Since v10 it also teaches which headings a declaration body may carry unmarked ([§FS-init.2.3.4.5.1](FS-init.md#23451-unmarked-headings)).

###### 2.3.4.5.1 Unmarked headings

In managed block v10 the block teaches that every non-declaration ATX heading inside a Markdown declaration body must carry a numbered or enabled named section path; file titles, body-closing headings, fenced examples, non-ATX text, and source doc-comments remain exempt, while bold labels remain the non-citable alternative ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth), [§FS-check.4.14](FS-check.md#414-unmarked-markdown-heading)). Existing v9 blocks use the ordinary `grund init` in-place repair path ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)); `grund_config_version` remains 1 because no configuration meaning changes.

##### 2.3.4.6 Spec First

The rules tell agents to write or update the most-specific functional or architectural spec point before writing behavior or design code that implements it.

##### 2.3.4.7 Most-Specific Citations

The code back-reference guidance tells agents to cite the most-specific spec point the code or prose realizes: whole behavior on the function, class, or block doc-comment; narrower clauses or decisions inline where they are enforced.

##### 2.3.4.8 Refresh Before Editing

The rules tell agents to refresh the cited spec with `grund <ID>` before editing code that already carries a `§<ID>` or `§<ID>.<section>` citation.

##### 2.3.4.9 Declaration Blast Radius

The rules tell agents to run `grund refs <ID> --summary` before changing, moving, or renaming a declaration, and to use the full `grund refs <ID>` output when exact citation sites are needed.

##### 2.3.4.10 Citation Direction

The rules teach the expected citation direction: specs cite goals, architecture cites specs, code cites the specs it implements, and executable tests or cases cite the behavior they verify. When the effective `grund.toml` declares `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), this guidance is the generated, config-derived Citation directions section ([§FS-init.2.3.5](FS-init.md#235-citation-directions)) rather than the static sentence above — so the rule an agent reads is the rule `grund check` enforces. A config without `[citations]` keeps the static sentence, which links to the durable canonical explanation at `https://github.com/agent-grounds/grund/blob/main/docs/user-facing/citation-directions.md` so a generated entrypoint remains usable outside the grund source checkout. Either way the paragraph carries the grounding sentence of [§FS-init.2.3.5](FS-init.md#235-citation-directions) when any row's effective `require_grounding` is on: whether a file must ground at all is not a direction rule, so it does not wait on `[citations]` being declared.

##### 2.3.4.11 Decisions

The rules tell agents that decisions must be cited from the spec or architecture point they shaped, and that decision history is append-only: reversals are new decisions that supersede older ones rather than rewrites.

##### 2.3.4.12 Cross-Linking

The block tells agents to cross-link specs only via IDs — a single short sentence in the citations narrative, not phrased as a rule. The `[fmt.cross_refs].enabled` mode ([§FS-init.2.3.3](FS-init.md#233-citation-form)) does not change this wording.

##### 2.3.4.13 Executable Proof

The rules tell agents that behavior is proven by executable tests or cases, and that disagreements between the spec and executable proof require fixing both in the same change.

##### 2.3.4.14 Final Check

The rules tell agents to run `grund check` before committing, because dangling references are stop-the-line bugs whose diagnostics name the file and line.

##### 2.3.4.15 Workspace Members

When — and only when — the init target lies in a workspace ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), the one [§FS-init.2.3.4.15.1](FS-init.md#234151-which-workspace) finds, the block emits a `### Workspace members` section as a sibling to the Project Map ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)). The section contains exactly two things: one discoverability line ([§FS-init.2.3.4.15.2](FS-init.md#234152-the-discoverability-line)) and one bullet per resolved foreign workspace project ([§FS-init.2.3.4.15.3](FS-init.md#234153-foreign-project-rows)). Each entrypoint omits its own project from that scope ([§FS-init.2.3.4.15.4](FS-init.md#234154-self-is-omitted)), so root-side and member-side lists differ by which alias is absent and by the remaining links' relative paths; the foreign aliases retain the same ordering and the discoverability line is identical.

If workspace expansion, alias validation, alias uniqueness, or nested-workspace validation would fail under the same rules as `grund check`, `init` suppresses the Workspace members section instead of emitting partial or ambiguous guidance. `grund check` remains the command that reports the configuration error.

This serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) by making the cross-project citation scope visible at the entrypoint an agent reads first, without breaking the no-prompts-no-surprises contract in [§FS-init.3](FS-init.md#3-non-intrusive-guarantees) ([§FS-init.2.3.4.15.7](FS-init.md#234157-what-init-does-not-do)).

###### 2.3.4.15.1 Which workspace

The workspace is found by the claimed-chain climb of [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces), not by the discovery walk-up of [§FS-config.1](FS-config.md#1-file-location-and-discovery), which stops at the nearest config: whether or not the target's own `grund.toml` declares `[workspace]`, the search continues up to the **outermost** ancestor that both declares `[workspace]` and lists the directory below it, and the section reads that ancestor's block rather than the target's effective `grund.toml`. An ancestor block that does not claim this tree describes a different workspace: its aliases resolve nowhere here and its members are directories outside this project, so it is not read and the section names the tree the outermost *claiming* block resolves. The section is therefore emitted whether `init` runs at the workspace root or inside a member directory, and in a nested workspace it covers the whole tree rather than only the enclosing group: the outermost root is the scope that resolves every alias an agent may cite.

###### 2.3.4.15.2 The discoverability line

One line, verbatim except for the configured citation marker: `Cross-project citations use <marker>alias/<ID>, one alias segment per workspace level.` This is the briefest surface that teaches the workspace-citation grammar to an agent landing at the entrypoint cold. The marker is the target project's effective `[reference].marker`, the same value used by the citation example in [§FS-init.2.3.8](FS-init.md#238-substituted-content); the full grammar lives in [§FS-workspace.1](FS-workspace.md#1-citation-syntax), and the block does not duplicate it.

###### 2.3.4.15.3 Foreign project rows

One Markdown bullet per resolved **foreign** workspace project, sorted by alias. Members in `[workspace] members = [...]` are expanded the same way `grund check` expands them (single-segment trailing `*` globs, hidden directories skipped, [§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), recursing into a member that declares its own `[workspace]` so every foreign project in the tree gets a row, labelled with its whole alias path ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces)). Aliases are derived per [§FS-workspace.3](FS-workspace.md#3-aliases).

###### 2.3.4.15.4 Self is omitted

The project whose entrypoint is being rendered is omitted: self is the resolved project whose canonical root equals the canonical init target, never a project guessed from alias text. Whether the invocation writes canonical `AGENTS.md` or only a companion entrypoint does not change which canonical self project is omitted. Consequently the root omits its own row even when `include_root = true`; that setting still makes the root a foreign row in member entrypoints, while `include_root = false` omits it there too. An intermediate node in a nested workspace obeys its own block's `include_root` the same way, so a pure grouping directory contributes no row. Omitting self intentionally omits its description too; pending `--name` and `--description` values do not create a local row.

###### 2.3.4.15.5 The bullet

Each foreign bullet renders the backticked alias as the *label* of a Markdown link, so the destination path appears exactly once — `` - [`api`](apps/api/AGENTS.md) `` — the same raw-readable `- [x](y): …` list grammar the Project Map ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)) uses; a raw-bytes reader still sees the path in the link destination, and token cost stays minimal ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)). When the foreign project has a `project_description` in its own config ([§FS-config.3](FS-config.md#3-schema), [§FS-workspace.3](FS-workspace.md#3-aliases)), the bullet appends `: <description>` (colon, space) after the link, before any trailing marker; a project without one keeps the link-only bullet byte-identical, and `init` never derives a description it was not given ([§DF-workspace-member-descriptions](../decisions/functional/DF-workspace-member-descriptions.md#df-workspace-member-descriptions-member-side-project_description-for-workspace-member-lists)).

###### 2.3.4.15.6 The link target

When a foreign project's `AGENTS.md` exists on disk at the time `init` runs, the link target is `<member-root>/AGENTS.md`. When it does not, the link target is the member root directory (with a trailing `/`) and the bullet ends with the literal trailing marker `*(not yet initialized)*` — so an agent reading the block never follows a 404 link, and the missing entrypoint is surfaced as actionable information (running `grund init` inside that directory is the next step). Any foreign uninitialized project — including the workspace root when listed from a member and its `AGENTS.md` is missing — keeps the marker. Existence is checked after path canonicalization; symlinks count. Paths in link targets are emitted relative to the directory where the entrypoint being written lives, so a workspace-root `AGENTS.md` points at `apps/api/AGENTS.md` while the same foreign project emitted inside another member's `AGENTS.md` points at `../../apps/api/AGENTS.md`.

###### 2.3.4.15.7 What `init` does not do

`init` does **not** prompt for, infer, or configure workspace topology. It does not add a `[workspace]` block to a config that lacks one. It does not write or modify any file under a member directory other than the member it was invoked in — even when invoked at the workspace root, the Workspace members section is the only artifact `init` produces about sibling members; bootstrapping a member's own `AGENTS.md` remains a separate `init` invocation inside that member. Out of scope for v1: a reciprocal member→root pointer, and any collapse-to-alias-only rendering for very large workspaces.

##### 2.3.4.16 Project Namespaces

The block always emits a `### Project namespaces` section that teaches agents the workspace namespace concept before they start creating or citing declarations. It distinguishes project namespaces from documentation folders; names the current project as the local namespace; tells agents to create or use a separate namespace only for independently checked apps, packages, services, or subprojects; and gives the operational steps for that split: create the member's `grund.toml`, add it to the workspace root's `[workspace] members`, run `grund init` in the member, and set a stable `project_name`. It also teaches the cross-namespace citation form `<marker>alias/<ID>`, one alias segment per workspace level, and says full cross-namespace validation runs from the workspace root with `grund check` ([§FS-workspace.1](FS-workspace.md#1-citation-syntax), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration), [§FS-workspace.5](FS-workspace.md#5-command-scope)).

##### 2.3.4.17 Clickable Citations

The block always carries the repository-scoped sentence: `On repository web surfaces, link §<ID> to the PR branch in PR bodies, the reviewed commit in reviews, an exact commit for permalinks, and the default branch otherwise; fall back to plain when unsure.` ([§DF-neural-link-generation](../decisions/functional/DF-neural-link-generation.md#df-neural-link-generation-agents-compose-clickable-citation-links-themselves-grund-does-not-grow-a-link-command)). Every `§` in this section's sentences is the configured marker ([§FS-init.2.3.4.17.1](FS-init.md#234171-the-marker)).

When the effective `grund.toml` sets `[reference] conversation = "link"` ([§FS-config.3.1](FS-config.md#31-reference--citation-form), [§DF-repo-conversation-opinion](../decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)), the section additionally carries a config-derived local-conversation sentence in the form of the entrypoint's agent ([§FS-init.2.3.4.17.2](FS-init.md#234172-the-form-per-agent)), followed in both forms by the fixed deference clause `If a user-level grund block states a local-conversation rendering, follow that instead: that machine knows what its surface can open.` ([§FS-init.2.3.4.17.6](FS-init.md#234176-the-deference-clause)). When the key is absent, the sentence is absent, and local conversation rendering is governed solely by the user preference `grund integrations --write` installs in user-level agent instructions ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)). The full recipe, rationale, and test matrix stay in the cited decision; the repository block carries only the compact, forge-neutral convention.

###### 2.3.4.17.1 The marker

Every `§` shown in this section's sentences is the repository's configured `[reference] marker` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), rendered like the marker anywhere else in the block ([§FS-init.2.3.4.2](FS-init.md#2342-reference-scheme)) — the wording is fixed, the marker is not. A section that hardcoded `§` in a repository configured with another marker would instruct agents to write a token that repository does not treat as a citation at all: `grund check` ignores it under `strict`, so the claim it grounds is silently never verified, while the surrounding block correctly teaches the real marker.

###### 2.3.4.17.2 The form per agent

Which form is rendered depends on the entrypoint's agent, under the gate in [§DF-conversation-link-target.2.4](../decisions/functional/DF-conversation-link-target.md#24-the-form-is-gated-per-agent-and-the-fallback-is-path). The entrypoints of the agents that gate clears for the `file` target — Claude's (`CLAUDE.md`, `.claude/CLAUDE.md`) and Pi's (`.pi/AGENTS.md`) — carry `In local conversations, render §<ID> as a Markdown link whose visible text is the citation itself and whose target is file://<absolute path>#L<line> for its declaration; fall back to the bare citation when unsure.` Every other entrypoint carries `In local conversations, follow §<ID> with its declaration location as plain path:line text; fall back to the bare citation when unsure.` The committed form is always the `file` target: it is composed at write time from the repository root the agent already has, so it embeds nothing about any machine and two installs render byte-identical files ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)); a machine that names another target overrides it through its own user-level block.

###### 2.3.4.17.3 A symlinked `CLAUDE.md`

The surface is decided by the **written path**, which is what makes it reproducible — and which has one consequence worth stating: a `CLAUDE.md` that is a companion *symlink* to `AGENTS.md` selects the canonical `AGENTS.md` target ([§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection)), so the one file both agents read carries the plain form. That is the gate working as intended rather than a gap — the file Codex reads cannot also carry a form recorded as erasing citations there ([§DF-conversation-link-target.2.4](../decisions/functional/DF-conversation-link-target.md#24-the-form-is-gated-per-agent-and-the-fallback-is-path)) — and a repository that wants the linked form for Claude replaces the symlink with the real entrypoint `--claude` writes ([§FS-init.2.1.1.1](FS-init.md#2111-a-symlinked-companion-under-conversation--link): the symlink holds `CLAUDE.md`, so that entrypoint is `.claude/CLAUDE.md`).

###### 2.3.4.17.4 The shadowed-entrypoint note

`init` **says so rather than leaving the sentence quietly absent**: when the effective config commits the opinion and a Claude entrypoint resolves to the canonical target, the run emits one `note:` line naming the symlink and the fix that is still open ([§FS-init.2.3.4.17.5](FS-init.md#234175-the-fix-the-note-names)). The note is a report, not a finding ([§FS-init.2.2.1](FS-init.md#221-notes)) — but silence here is indistinguishable from the opinion not working, on the one agent the opinion was mostly written for.

###### 2.3.4.17.5 The fix the note names

Which fix is still open depends on the **tree**, not on the run: whether Claude has an entrypoint of its own is a fact about the repository, and a run that selected some other agent has not changed it, so the same tree gets the same diagnosis whatever flag reached it. Where Claude has no entrypoint of its own, the fix is `grund init --claude`, which writes one ([§FS-init.2.1.1.1](FS-init.md#2111-a-symlinked-companion-under-conversation--link)) — and where symlinks have taken every path Claude reads there is no free path, so the note names deleting one of them first. Where Claude *has* one, whether from this run or already on disk, that half is done and what remains is deleting the symlink, which otherwise keeps feeding Claude the canonical file's plain form beside the linked one. A note that advised the command that had just run would never retire.

The deletion is named as safe to do **now** only when this run left that entrypoint carrying the current block; otherwise — a preview, or an entrypoint this run did not select — the note names the same fix with the condition still on it, because a symlink deleted ahead of the block that replaces it costs the reader their only Claude entrypoint.

###### 2.3.4.17.6 The deference clause

Because the local-conversation sentence now names its scope explicitly, it no longer needs the em-dash binding the earlier *never a Markdown link* clause required to keep from reading as a revocation of the repository-web sentence. The deference clause is what keeps rendering machine-local: the committed opinion is only the fallback for machines that never stated a preference, while a machine that recorded one through `grund integrations --write` has evidence about its own surface, and that preference wins ([§DF-repo-conversation-opinion.2.3](../decisions/functional/DF-repo-conversation-opinion.md#23-precedence)).

#### 2.3.5 Citation directions

When the effective `grund.toml` declares `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), the managed block renders a `### Citation directions` section generated from those rules, replacing the static citation-direction sentence ([§FS-init.2.3.4.10](FS-init.md#23410-citation-direction)). This is the strongest enforcement point for the `should` levels, which never appear in `grund check`'s standing output ([§FS-check.2.3](FS-check.md#23-suggestions-channel-opt-in)): the agent reads the rule before writing the declaration, so most `should` obligations are met at write time. Decided in [§DF-citation-directions.2.7](../decisions/functional/DF-citation-directions.md#27-generated-agent-entrypoint-section-with-a-drift-check); the wording is [§DF-directions-render](../decisions/functional/DF-directions-render.md#df-directions-render-the-citation-directions-wording-is-chosen-once-against-a-canonical-config)'s, chosen once against a canonical config that exercises every branch.

The section is what an agent reads *instead of* `grund.toml`, so it states rules and never config grammar ([§FS-init.2.3.5.1](FS-init.md#2351-layout) to [§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence)), and `grund check` holds it to the live config ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check)).

##### 2.3.5.1 Layout

The section opens with one paragraph — the fixed legend ``must`/`never` are `grund check` errors; `should`/`avoid` are suggestions (`grund check --suggestions`).``, followed by the grounding sentence (§FS-init.2.3.5.7) where any row's effective `require_grounding` is on — then one bullet per citing kind that has any rule, in `[[kinds]]` order with the homeless kind (§FS-config.3.9.2) last, then one closing line (§FS-init.2.3.5.6). An unwalked kind (§FS-config.3.4.7) has no bullet at all: it can carry no `[citations.<kind>]` rule.

##### 2.3.5.2 The subject names its unit

**Every bullet names its unit**, because the units the levels are checked per are not interchangeable and one verb over three of them says nothing ([§FS-check.3.11](FS-check.md#311-missing-required-citation)):

| Citing kind | Subject |
|---|---|
| citable | `Each **DA** declaration` |
| non-citable, folder home | `Each file in **skills/**` |
| non-citable, single-file home | `The file **docs/runbook.md**` |
| homeless, with a `title` | `Each source file outside the Project map (**code**: Gradle build and workflows) that cites anything` |
| homeless, without one | `Each source file outside the Project map (**code**) that cites anything` |

###### 2.3.5.2.1 Places, and what the homeless kind covers

A non-citable kind is named by its place, as in the Project map ([§FS-init.2.3.4.4.1](FS-init.md#23441-rows-for-places)), so the bullet reads as the instruction it is: files in this directory cite that. The homeless kind has no place, so it keeps its name and says what it covers. Its *that cites anything* is load-bearing: the obligation constrains what a source file cites and never whether it cites at all ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), so a util that cites nothing is not a unit. A non-citable home is the opposite case — a file there with no citation is the defect — so nothing narrows that subject, and `require_grounding` is what closes the hole ([§FS-check.3.6](FS-check.md#36-ungrounded-source-file-opt-in)).

##### 2.3.5.3 The clauses

The clauses follow the subject, joined by `; ` in the order `must`, `should`, `may`, `must-not`, `should-not`, then the per-kind default. A prohibition leads its bullet with the modal — `must not cite`, `should not cite`, the wording [§FS-check.3.12](FS-check.md#312-forbidden-citation) uses in its findings — and follows a preceding clause with the short `never cite` / `avoid citing` the legend names, because the subject is a noun phrase and a bullet opening on a bare `never cite` is not a sentence.

##### 2.3.5.4 Targets

Targets render as prose, never as rule grammar. Alternatives inside one entry join with "or", taking the Oxford comma from three on (`FS, AR, or RM`); conjunctive entries join with "and", and where there is more than one entry an entry that has alternatives of its own is parenthesised — `must = ["FS|GOAL", "AR"]` renders `(FS or GOAL) and AR`, which has one reading. A pinned alias renders exactly as spelled (`api/AR`), because that is how the citation is written; `*/AR`, which is rule grammar and never a citation ([§FS-config.3.9.3](FS-config.md#393-namespace-matching)), renders as `AR in any project`.

##### 2.3.5.5 Defaults

Only `must-not` and `should-not` close anything: [§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence) makes a default of `must`, `should`, or `may` invent no obligation and forbid nothing. A per-kind `must-not` default folds into the permission it qualifies — `may cite only FS or GOAL` — when the `may` list is the whole of what the kind permits; with a `must` or `should` beside it the permitted set is wider than that list, so the bullet ends with `never cite anything else` instead. A per-kind `should-not` default ends its bullet with `avoid citing anything else`. A per-kind default that leaves its kind open says so, `may cite anything else`, only where the global default is closed and the kind is therefore a hole in it.

##### 2.3.5.6 The closing line

The closing line reports the global default alone, because a per-kind default is *listed above*: `Anything not listed above is allowed.` for an open one, including `must` and `should`; `Any citation not listed above is forbidden.` for `must-not`; `Any citation not listed above is discouraged.` for `should-not`. Either form is load-bearing — the first so an agent does not over-infer prohibitions from silence, the second so it does not miss a closed world.

##### 2.3.5.7 The grounding sentence

The grounding sentence is generated from each row's effective `require_grounding` — the row's own key, else `[reference] require_grounding`, because the row decides ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs), [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) — and the configured non-citable homes, and it renders whether or not `[citations]` is declared, because grounding is not a direction rule ([§FS-init.2.3.4.10](FS-init.md#23410-citation-direction)). It claims only what [§FS-check.3.6](FS-check.md#36-ungrounded-source-file-opt-in) enforces, so it names a place exactly when that place's row has grounding on; naming the level per place ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) is not its work.

###### 2.3.5.7.1 Citing, not declaring

The sentence distinguishes citing from declaring: `Every source file must cite a declared ID or declare one inline`, extended with `; every file under skills/ and tests/e2e/ must cite one` for the walked non-citable homes whose rows have it on, because a declaration in such a home is misplaced ([§FS-check.3.7](FS-check.md#37-misplaced-declaration-configured-kind-home)) and those files can therefore only cite. An unwalked home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-walked)) is left out: nothing in it is scanned, so the rule never reaches it.

##### 2.3.5.8 The drift check

Because the section's content derives from config rather than the template alone, the version marker ([§FS-init.2.3.7](FS-init.md#237-the-block-version)) is no longer sufficient to detect staleness: editing `[citations]` without re-running `grund init` would leave guidance that disagrees with the live rules under a current version number. `grund check` therefore re-renders the section from the live config and byte-compares it against the section in the block; a mismatch is an `agents-init` finding ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) telling the author to re-run `grund init`. Rendering determinism is what makes the comparison sound — the render *is* the hash.

##### 2.3.5.9 The block versions this section moved

The managed-block version was bumped to carry this config-derived content under [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations), and bumped again to **v8** when the rendering of [§FS-init.2.3.5.1](FS-init.md#2351-layout) to [§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence) replaced the flat one this section used to specify — which stated no unit, grouped no conjunction, rendered no grounding sentence, and leaked `*/AR` into prose ([§DF-directions-render](../decisions/functional/DF-directions-render.md#df-directions-render-the-citation-directions-wording-is-chosen-once-against-a-canonical-config)). The later v9 bump is solely the point-size sweep in [§FS-init.2.3.4.3](FS-init.md#2343-cheap-grounding); it changes no citation-direction rendering.

##### 2.3.5.10 Chapter rules

When at least one kind enables `rules = true`, the block renders a
`### Chapter rules` section immediately after `### Citation directions`, with
the existing `must`/`should` legend and one bullet per valid rule in qualified
rule-ID order. Each bullet contains the exact authored sentence followed by its
live rule citation. The section and its refusal/no-write lifecycle are
[§FS-rules.4](FS-rules.md#4-validation-lifecycle) and
[§FS-rules.9](FS-rules.md#9-managed-guidance-and-editor-parity)'s. `check`
re-renders and byte-compares it as config-derived content; a mismatch is
`agents-init`.

#### 2.3.6 Clickable citations

The managed block renders a `### Clickable citations` section carrying the content points in [§FS-init.2.3.4.17](FS-init.md#23417-clickable-citations): the fixed repository-web sentence always, plus the local-conversation sentence when `[reference] conversation = "link"` is set. The section is deterministic from the effective config and the entrypoint's own agent ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)): byte-identical in every project with the same key state, for the same entrypoint file. The per-agent split of [§FS-init.2.3.4.17.2](FS-init.md#234172-the-form-per-agent) is the one axis on which two entrypoints in the *same* repository differ, and it is a pure function of the target path, so it is as reproducible as the rest.

##### 2.3.6.1 The drift check

Because the local-conversation sentence derives from config, the section joins the citation-directions drift check ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check)): `grund check` re-renders it from the live config and byte-compares; a mismatch — including flipping the `conversation` key without re-running `grund init` — is an `agents-init` finding ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)).

##### 2.3.6.2 The block versions this section moved

Adding the fixed sentence bumped the managed-block version to **v4**; adding the config-derived local-conversation sentence bumped it to **v5**, replacing that sentence with the gated link form bumped it to **v6**, and naming the bare `grund.toml` in the namespace rule's "give that project its own config" instruction ([§FS-config.1](FS-config.md#1-file-location-and-discovery), [§DF-config-file-location.2.3](../decisions/functional/DF-config-file-location.md#23-grund-init-writes-the-bare-grundtoml)) bumped it to **v7**, all carried under [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations); an older supported block is reported outdated by `grund check` until `grund init` re-renders it.

#### 2.3.7 The block version

The canonical text for a given block version `vN` is embedded in the `grund` binary; the reference copy lives at `templates/AGENTS.md` in the `grund` source tree. The `vN` marker ([§FS-init.2.3.9](FS-init.md#239-delimiters)) is what versions it under [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations), so changing the block's fixed canonical text is itself a block-version bump, carried by that mechanism, not a silent rewrite. What `vN` versions is that fixed text, not the lines substituted in ([§FS-init.2.3.8](FS-init.md#238-substituted-content)): the inline citation style sentences among them ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)) move no version, because they are rendered, not live ([§FS-inline-citation-style.5.6](FS-inline-citation-style.md#56-no-managed-block-version)).

##### 2.3.7.1 The current version

The current schema for a repository without a rule kind is **v10**; v10 adds
the in-body Markdown heading policy in [§FS-init.2.3.4.5.1](FS-init.md#23451-unmarked-headings), and an existing v9 block is
the supported predecessor, repaired by the same one-command `grund init`
re-render ([§FS-init.2.3.10.1](FS-init.md#23101-re-rendering-an-existing-block)). A rule-enabled repository uses **v11** because §FS-init.2.3.5.10
adds byte-compared content. Removing the opt-in returns to byte-identical v10
output rather than making v11 universal. The v9 history remains the point-size
sweep added in [§FS-init.2.3.4.3](FS-init.md#2343-cheap-grounding), with v8 as its predecessor.


##### 2.3.7.2 What `check` compares

`grund check`'s agent-entrypoint validation ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) checks the marker line and the version, not a byte-diff against the canonical text — save for the config-derived `### Citation directions` and `### Clickable citations` sections, which it re-renders from the live config and byte-compares ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check), [§FS-init.2.3.6.1](FS-init.md#2361-the-drift-check)).

#### 2.3.8 Substituted content

Several things in the block are *substituted in* rather than fixed for its `vN`, so the file describes the repo it is in: the project name from `--name` (interpolated into the scaffolding H1 emitted above the block for a fresh `AGENTS.md`, [§FS-init.2.3.10](FS-init.md#2310-writing-the-block)), and the **effective ID grammar and artifact map** — taken from the `grund.toml` `init` leaves governing the target (an existing config in the target, or the defaults `init` is about to write, never an ancestor's).

##### 2.3.8.1 What the config fills in

From that config the block fills in the ID shape (`<KIND>-<NNN>-<slug>`, `<KIND>-<slug>`, …, derived from `[id].format`), one worked example ID and citation, the `[id].section_separator`, the marker and `$$`-trigger from `[reference]`, the `KIND ∈ {…}` set from `[[kinds]]`, a raw-readable link list of each kind's configured declaration home and title ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)), a sentence on whether bare ID-shaped tokens count as citations (driven by `[reference].strict`), and the inline citation style sentences defined by [§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering).

##### 2.3.8.2 The worked example is escaped

The worked example citation is written in the escaped illustration form — `<§>` around the configured marker, per [§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves) — never as a live citation: the example ID is deliberately not a real declaration in the host repo, so a live marker would make the freshly generated block fail the host repo's own `grund check` ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) as a dangling reference wherever the entrypoint falls inside the scan scope.

##### 2.3.8.3 A config that fails to load

The generator–checker symmetry of [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints) extends to reading the config: an existing `grund.toml` that fails to load is an error (`exit 2`, the message and location `grund check` gives for the same file), never a silent fall back to defaults. The block is rendered *from* that config, so substituting defaults would write agent instructions describing a repository the user does not have — an unparseable `[reference] conversation`, `marker`, or kind set would drop exactly the guidance it selects while `init` reported success — and the two commands would disagree about a file both of them read.

#### 2.3.9 Delimiters

The managed block is bounded by explicit standard `BEGIN` / `END` HTML-comment delimiters, with an H2 heading just inside the opening delimiter carrying the schema version:

```markdown
<!-- BEGIN GRUND MANAGED BLOCK -->
## Grounding with grund (vN)
…
<!-- END GRUND MANAGED BLOCK -->
```

The integer after `v` is the managed-block version. The `<!-- BEGIN GRUND MANAGED BLOCK -->` line is the block's begin marker and `<!-- END GRUND MANAGED BLOCK -->` is its end marker; both delimiter lines belong to the managed region, so everything between them — delimiters included — is `init`'s to rewrite and nothing outside them is.

##### 2.3.9.1 Why the conventional shape

The delimiters are deliberately the conventional managed-region shape rather than a grund-specific arrow syntax, so humans and agents skimming the file recognize the ownership boundary without knowing grund ([§DF-managed-block-delimiters](../decisions/functional/DF-managed-block-delimiters.md#df-managed-block-delimiters-standard-beginend-delimiters-for-the-managed-agent-instructions-block)).

##### 2.3.9.2 Legacy blocks

Blocks at v3 and earlier predate the delimiters (**legacy blocks**): there the H2 heading itself is the begin marker and the block runs until the next H1 or H2 heading, or end of file. Both `init` and `check` continue to recognize the legacy form; on its next write `init` migrates a recognized legacy block to the delimited form in place (reported `updated `), preserving the block's position and every byte outside the block.

#### 2.3.10 Writing the block

A fresh `AGENTS.md` consists of the block preceded by a one-line scaffolding H1 (`# {NAME} — agent instructions`); the H1 is *unmanaged* on the update path, so `init` without `--force` rewrites the block but leaves the title alone, while `--force` rewrites the whole canonical file, H1 included ([§FS-init.3.4](FS-init.md#34-with---force)). A freshly-created companion entrypoint consists of just the managed block, with no extra H1, so the agent-specific file remains a thin companion of the canonical workflow. If an agent entrypoint already exists and contains no managed block, `init` appends the current block after the existing content (no H1 is inserted — the host file owns its title).

##### 2.3.10.1 Re-rendering an existing block

If the file already contains any supported block version, including the current one, `init` re-renders the block from the current effective `grund.toml`, compares it to the bytes of the managed region on disk ([§FS-init.2.3.9](FS-init.md#239-delimiters), delimited or legacy), and — when they differ — replaces only those bytes, leaving all content before and after the block untouched (reported `updated `, [§FS-init.2.3.1](FS-init.md#231-position-invariants)). Same-version template and config changes therefore propagate on the next `grund init` without requiring `--force`. When the re-render is byte-identical to what is on disk, `init` writes nothing and reports the file with `exists ` ([§FS-init.2.2](FS-init.md#22-stdout--stderr)) — re-running `grund init` on an already-current repo is a no-op on every file.

##### 2.3.10.2 A newer block version

If an agent entrypoint contains a newer block version than the running binary supports, `init` exits 2 and leaves the file unchanged.

#### 2.3.11 Malformed delimiters

A file whose delimiters are present but broken is **malformed**: a `BEGIN` delimiter with no `END` after it, an `END` with no `BEGIN` before it, more than one `BEGIN`, or a delimited region that contains no `## Grounding with grund (vN)` version heading. `init` refuses to touch a malformed file — it exits 2 with a message naming the file and the specific defect, and rewrites nothing — because any splice against broken delimiters risks eating user content; `grund check` reports the same defect as an error ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). Fixing the delimiter pair by hand (or deleting the remnants of the block) and re-running `grund init` is the recovery path.

#### 2.3.12 Supported agents and their entrypoints

`AGENTS.md` is the canonical fallback entrypoint. The supported-agent set is fixed and is the superset used by every agent-facing feature: AGENTS-compatible agents (including Codex), Claude Code, Gemini, Pi, GitHub Copilot, Cursor, Windsurf, and Zed. The built-in companion set covers their common root or repository instruction files: Codex override instructions (`AGENTS.override.md`), Claude Code (`CLAUDE.md`, `.claude/CLAUDE.md`), Gemini (`GEMINI.md`), Pi (`.pi/AGENTS.md`), GitHub Copilot (`.github/copilot-instructions.md`), Cursor (`.cursor/rules/grund.mdc`, plus the legacy `.cursorrules`), Windsurf (`.windsurfrules`), and Zed (`.rules`).

##### 2.3.12.1 Which companions are created

Companion entrypoints are mostly discovery-based: `init` updates them if the repo already has them, and creates the agent-directory-triggered companions for Claude, Gemini, Pi, Cursor, or Zed — one per agent ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)) — only when the matching agent directory already shows that tool is in use. For the reasons [§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection) gives, `.github/copilot-instructions.md` is created only by `--copilot`, `.windsurfrules` only by `--windsurf`, and `.rules` only where `.zed/` exists or `--zed` is passed ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)).

##### 2.3.12.2 A companion symlinked to `AGENTS.md`

When a companion path is a symlink to `AGENTS.md`, `init` does not touch it separately. If that companion is explicitly requested, the request selects canonical `AGENTS.md` as the update target; `grund check` always treats the canonical `AGENTS.md` block as sufficient for that symlink ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)).

### 2.4 Generated `grund.toml`

`init` writes `<path>/grund.toml` — the bare, root-visible form — and **only when the target has no config at all**, in either discovery location ([§FS-init.2.4.1](FS-init.md#241-both-discovery-locations-are-probed)). An existing config is never overwritten, not even with `--force` ([§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)). The file it does write is a teaching surface ([§FS-init.2.4.3](FS-init.md#243-every-written-key-is-the-default)); [§FS-init.2.4.4](FS-init.md#244-named_sections-is-written-off) to [§FS-init.2.4.8](FS-init.md#248-values-are-illustrated-not-enabled) say how particular keys are written.

#### 2.4.1 Both discovery locations are probed

Both discovery locations of [§FS-config.1](FS-config.md#1-file-location-and-discovery) are probed, so a repository already configured under `.agents/grund.toml` is reported `exists .agents/grund.toml` and never grows the redundant pair [§FS-check.4.3](FS-check.md#43-redundant-config-pair) warns about. That probe matters more under the [§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both) tie-break than a same-name check would: writing a bare `grund.toml` beside an existing `.agents/` config would *take over* the repository's grammar, which is exactly what `init` must never do to a config it did not write. Which form a project uses stays its own choice; the bare one is what `init` recommends, by generating it ([§DF-config-file-location.2.3](../decisions/functional/DF-config-file-location.md#23-grund-init-writes-the-bare-grundtoml)).

#### 2.4.2 An existing config is never overwritten

An existing config is the repo's configuration — the one surface a project customizes ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable)) — and `init` never overwrites it, not even with `--force`: an existing config is reported as `exists` and left byte-for-byte unchanged ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)). `--force` resets the things `init` owns end to end — the canonical `AGENTS.md` and the `--docs` scaffold stubs — not the user's settings; a customized config that `init --force` clobbered would be a footgun.

#### 2.4.3 Every written key is the default

When the file is absent and `init` does write it, every key written matches the built-in default for that key — including `FS` as `file = "requirements.md"` — so the file is a teaching surface, not an override surface, and a new user can see the schema they will be editing. Top of file is `grund_config_version = 1` per [§FS-config.5](FS-config.md#5-schema-versioning); the schema written is exactly the one in [§FS-config.3](FS-config.md#3-schema), no extra keys, no missing keys.

#### 2.4.4 `named_sections` is written off

The generated `[id]` table includes the uncommented Boolean `named_sections = false`. `init` never enables the feature on a repository's behalf; the author must opt in deliberately before named headings acquire citation meaning.

#### 2.4.5 Finite value sets are named in comments

Because the generated config is also a usability surface, non-boolean keys with a finite accepted value set carry inline comments listing that set. Boolean keys are left uncommented: spelling out `true | false` is not useful guidance and should not be emitted. Free-form keys instead describe their grammar or omit the comment when a finite list would be misleading. For example, `[reference].inline_style` names `citation-with-note | citation-only`, `[id].section_heading_levels` names `strict | warn | loose`, `[output].format` names `text | json`, and `[fmt.cross_refs].anchor_format` names `github | gitlab | mkdocs | pandoc | none`. These comments are explanatory only; they are not schema keys and do not change parsing under [§FS-config.3](FS-config.md#3-schema).

#### 2.4.6 `shorthand` is written out

The generated `[reference]` table writes
`shorthand = "canonical" # canonical | accepted`. This makes the current
canonical behavior explicit while teaching the opt-in persistence policy; as
with every generated key, the written value is the built-in default
([§FS-config.3.1](FS-config.md#31-reference--citation-form)).

#### 2.4.7 `project_name` and `project_description`

A single `project_name = "<name>"` key appears at the top above the section tables. This key is metadata only — it is not consumed by any other `grund` subcommand and exists so downstream tooling (IDE status bars, CI dashboards) can read it without re-deriving the name. Directly below it the file teaches the optional `project_description` key ([§FS-config.3](FS-config.md#3-schema)): a commented `# project_description = "<one line shown next to this project in workspace member lists>"` line by default, or the real `project_description = "<text>"` key when `--description` was given ([§FS-init.1](FS-init.md#1-inputs)). The commented form is a teaching comment like the finite-value-set comments ([§FS-init.2.4.5](FS-init.md#245-finite-value-sets-are-named-in-comments)), not a schema key, so the "no extra keys, no missing keys" guarantee ([§FS-init.2.4.3](FS-init.md#243-every-written-key-is-the-default)) is unchanged.

#### 2.4.8 Values are illustrated, not enabled

The shipped template illustrates values only as a commented `CONST` row with `values = true` and `file = "values.json"`. Fresh repositories remain unopted-in, and neither the template nor the effective schema contains a `value_sources` key ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)).

## 3. Non-intrusive guarantees

What `init` won't touch, in one list; the detail lives at the section cited beside each guarantee.

- **A target that is not a project is refused before anything is written** ([§FS-init.1.2](FS-init.md#12-refused-targets)).
- **Automatic mode never adds a competing entrypoint** ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags), [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)).
- **`init` never creates a second file for an agent that already has one** ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)), save the Claude entrypoint an explicit `--claude` writes beside a symlinked companion in a `conversation = "link"` repository ([§FS-init.2.1.1.1](FS-init.md#2111-a-symlinked-companion-under-conversation--link)).
- **Ambiguous companions need an unambiguous signal** ([§FS-init.3.1](FS-init.md#31-ambiguous-companions-need-an-unambiguous-signal)).
- **User-authored content in agent entrypoints is preserved** ([§FS-init.3.2](FS-init.md#32-user-authored-content-in-agent-entrypoints-is-preserved)).
- **Line endings outside the managed block are preserved** ([§FS-init.2.3.2](FS-init.md#232-line-endings)).
- **An existing `grund.toml` is never overwritten**, in either discovery location, not even by `--force` ([§FS-init.2.4.1](FS-init.md#241-both-discovery-locations-are-probed), [§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)).
- **No prompts, ever.** Every choice is a flag ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).
- **`--dry-run` previews any run** ([§FS-init.1](FS-init.md#1-inputs), [§FS-init.2.2](FS-init.md#22-stdout--stderr)).
- **Re-running is a true no-op when the repo is already current** ([§FS-init.2.2](FS-init.md#22-stdout--stderr), [§FS-init.2.2.2](FS-init.md#222-the-next-block)).

The `--force` semantics behind the config and "refuse to clobber" guarantees are [§FS-init.3.3](FS-init.md#33-without---force) to [§FS-init.3.5](FS-init.md#35-what---force-does-not-replace).

### 3.1 Ambiguous companions need an unambiguous signal

`.github/copilot-instructions.md` is never created from `.github/` alone (it is generic GitHub metadata) and `.rules` is never created from file existence alone (its filename is too generic): the first needs `--copilot`, the second an existing `.zed/` or `--zed` ([§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection), [§FS-init.2.3.12.1](FS-init.md#23121-which-companions-are-created)).

### 3.2 User-authored content in agent entrypoints is preserved

Unless `--force` replaces the canonical `AGENTS.md` whole ([§FS-init.3.4](FS-init.md#34-with---force)), only the delimiter-bounded managed `## Grounding with grund (vN)` block is touched. Everything before and after the block is byte-for-byte preserved, including the block's position within the file ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints), [§FS-init.2.3.1](FS-init.md#231-position-invariants)). The delimiter is the ownership boundary [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) is written around.

### 3.3 Without `--force`

Without `--force`, `init` never overwrites an existing file. Existing agent entrypoints are handled by the append/update rules in [§FS-init.2.3.10](FS-init.md#2310-writing-the-block). Every other existing target file is left unchanged and reported with `exists `:

```
exists grund.toml
```

This makes repeated `grund init` runs idempotent and safe for existing repos. The `--docs` mode applies the same rule across the docs tree: existing scaffold files are reported as `exists ` and left unchanged; missing scaffold files are written.

### 3.4 With `--force`

With `--force`, a selected canonical `AGENTS.md` and the `--docs` scaffold files are overwritten in place; their previous contents are not preserved (the user has git for that, per [§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking) — `grund` does not maintain its own history). The `--docs` files are stubs `init` owns, and a repository that has filled `docs/goals.md` or `docs/grund.md` with real content loses it, which is why the flag exists and why nothing else in `init` reaches these files ([§REQ-no-data-loss.3](../requirements/REQ-no-data-loss.md#3-destructive-is-opt-in-and-never-a-side-effect)).

### 3.5 What `--force` does not replace

**`--force` never overwrites the config**: `init --force` still reports it as `exists ` and leaves it untouched ([§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)). Existing *companion* entrypoints are likewise not full-file overwritten by `--force` — the canonical `AGENTS.md` is the one entrypoint the flag replaces whole; if a companion is present and not symlinked to `AGENTS.md`, only its managed block is appended or updated so unrelated agent-specific instructions remain intact.

## 4. Exit codes

- `0` — every requested file was written, appended, updated, or already current. Under `--check` ([§FS-init.1](FS-init.md#1-inputs)) this is the tree that is current: every reported path was `exists ` and there is nothing to write.
- `1` — `--check` only: the report carried at least one `would-…` line, so the tree is not current ([§FS-init.4.1](FS-init.md#41-what-earns-the-1)).
- `2` — I/O error (target path does not exist, permission denied, disk full, etc.); a CLI-level error such as an unknown flag; a refused target ([§FS-init.1.2](FS-init.md#12-refused-targets)), in which case nothing was written; an existing config that fails to load ([§FS-init.2.3.8.3](FS-init.md#2383-a-config-that-fails-to-load)); an existing agent entrypoint contains a managed block whose schema version is newer than the running binary supports ([§FS-init.2.3.10.2](FS-init.md#23102-a-newer-block-version)); or an existing agent entrypoint has malformed delimiters ([§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)) — in both of those the file is left unchanged. `2` wins over `1` ([§FS-init.4.2](FS-init.md#42-2-wins-over-1)).

`--dry-run` alone keeps `0` whatever it reports ([§FS-init.4.3](FS-init.md#43-why---dry-run-keeps-0)). Exit-code mapping is fixed per [§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out) and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization).

### 4.1 What earns the `1`

The `would-…` lines that earn the `1` are `would-write `, `would-append `, and `would-update `. Nothing was written, and the run that fixes it is the same `init` without the flag. A `note:` never earns the `1` and neither does the `next:` block — a note is a report, not a finding ([§FS-init.2.2.1](FS-init.md#221-notes)), and the exit code is decided by the reported paths alone.

### 4.2 `2` wins over `1`

A run that could not be performed produced no report to gate on, so `--check` over a refused target exits `2` the way any other form of the run does.

### 4.3 Why `--dry-run` keeps `0`

The asymmetry between `--dry-run`'s `0` and `--check`'s `1` is deliberate. `1` is the *findings* slot of the mapping [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed) freezes, and `init` had never used it, so `--check` fills a hole in the mapping rather than moving it. Giving the `1` to `--dry-run` instead would change the verdict an existing green CI step gets on upgrade — quietly, with no finding naming the release and no one-command fix — which is the one thing [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) puts the exit-code mapping out of reach of. So there are two spellings because there are two questions: `--dry-run` asks *what would this run do*, `--check` asks *is there anything left to do*.

## 5. Agent setup instructions

`grund agent-setup-instructions` prints the AI-agent-facing guided setup instructions for adopting `grund` in an arbitrary repo. This is a read-only discovery command: stdout is Markdown, stderr is empty, exit `0`; passing any positional argument or flag is a CLI-level error (`error: agent-setup-instructions takes no arguments`, exit `2`). The command exists so a user can tell an agent only "set up grund" plus provide an installed `grund` binary, and the agent can still discover the recommended setup workflow without browsing the source repository.

### 5.1 What the instructions tell the agent

The output reads like an agent skill rather than a human tutorial. It instructs the agent to inspect the target repo first, identify the existing specs, artifact types, roadmaps, changelogs, decisions, plans, tests, and agent instruction files, recommend suitable `grund init` and `grund.toml` choices with evidence, show pros and cons for every config option, ask the user to confirm or override the recommendations, write `grund.toml`, run `grund init`, then validate with `grund config validate` and `grund check`. The config must be written before `grund init` so the generated managed block reflects the selected ID grammar, marker, strict mode, kinds, and existing artifact layout.

### 5.2 Adopting an existing docs-heavy repo

For an existing docs-heavy repo, the instructions make the adoption choice explicit before any write: show the canonical `grund` artifact types, including `GRUND` for the grounding doc, beside the detected project-specific sections, tags, or document classes, then ask whether to use canonical `grund`, canonical core plus project-specific extras, or the existing structure with `grund` citations. The recommended `grund init` form omits `--docs` unless the user selects a canonical-layout migration; existing specs are represented in `[[kinds]]` and `[scan]` settings rather than replaced by generic scaffold folders. This follows [§DF-skill-init-existing-specs](../decisions/functional/DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding).

### 5.3 One body, two surfaces

The core instructions must not fork from the distributable skill. The repository keeps the distributable skill at `skills/grund-init/SKILL.md` and the binary-embedded copy at `crates/grund-core/assets/skills/grund-init/SKILL.md`; they must be byte-identical, and `grund agent-setup-instructions` prints that Markdown source byte-for-byte. The source package therefore exposes the instructions in two ways with one body: agents that can read the repository may load `skills/grund-init/SKILL.md` as a skill; agents that only have the installed CLI may run `grund agent-setup-instructions`. A release that edits one surface without the other is invalid. The skill's `### [citations]` section contains a marked byte-identical copy of `docs/user-facing/citation-directions.md`; its marked `### Chapter rules` section likewise contains the byte-identical writing section from `docs/user-facing/rules.md`. The asset-sync integration check prevents either pair from drifting ([§FS-rules.10](FS-rules.md#10-documentation-and-executable-examples)).

## 6. Why this exists

`init` is the smallest on-ramp `grund` offers: one command, every default the most-common case, nothing the user already authored rewritten without `--force` ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)). The shape follows from three goals taken together — an agent that reads its entrypoint file at session start should arrive already taught ([§GOAL-agent-grounding.1](../goals.md#1-the-three-layers)); a conformant tree should work without configuration so the emitted defaults *are* the canonical grammar ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)); and the on-ramp should not surprise either humans or scripts, so there are no prompts and every choice is a flag ([§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)).

### 6.1 One command instead of a copy-paste

Without `init`, the on-ramp to `grund` would be a copy-paste of someone else's agent instructions, with the resulting drift between projects. `init` collapses that on-ramp to one command and freezes the selected agent entrypoint at the `grund` version that wrote it — when the canonical form evolves, `grund init` re-emits the managed block in place ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) still applies through the project's `grund.toml`, which the re-emitted block is rendered from and `init` never overwrites, [§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)).

### 6.2 The grammar before any IDs exist

`init` is also the only safe place to demonstrate the ID grammar before any IDs exist: a repo that has not yet authored its first `FS-001-…` declaration still gets a literate agent entrypoint from `init` that teaches the grammar by example.

# FS-init: grund bootstraps a new grund-conformant repo

The `init` subcommand writes the minimum set of files a project needs to start using `grund` — an agent entrypoint ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)) and a bare `<path>/grund.toml` config, a minimal version-2 file ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)), laid from a template ([§FS-init.1.6](FS-init.md#16-templates)) — so that a fresh repo, or an existing repo adopting the scheme, becomes scannable in one command. It is the minimum-effort, non-intrusive on-ramp, and every behavior below serves that: every default is the most-common case; a config the repo already has, in either form, is kept; the versioned `grund` block goes into the agent entrypoints the repo already has, and only a repo with none gets the canonical `AGENTS.md` ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)); nothing the user authored is rewritten without `--force`; and `--dry-run` previews any run before it touches the working tree. [§FS-init.3](FS-init.md#3-non-intrusive-guarantees) consolidates those guarantees. Serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers) (the managed block is the instruction layer), [§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree) (the emitted defaults are the canonical grammar), and [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible) (no hidden prompts; the same input produces byte-identical output).

`init` is the only `grund` subcommand that creates adoption scaffolding in the working tree. In an existing agent entrypoint it may update the managed `grund` block, and it rewrites no other user-authored content there unless the file is the canonical `AGENTS.md` and `--force` is passed ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)).

Verbose implementer fixtures — exact stderr transcripts, final tree expectations, and common existing-file cases — are in [§FS-init-fixtures](FS-init-fixtures.md#fs-init-fixtures-concrete-init-fixtures); they are examples of this spec, not a separate feature.

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, home, citable, body, section, lead, index),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand, citation site), [§FS-terms.terms.3](FS-terms.md#terms3-source-forms) (source
declaration, stub, doc-comment), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, scope, workspace, member, alias),
[§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, suggestion, verdict), [§FS-terms.terms.6](FS-terms.md#terms6-rules-and-directions) (direction, rule), and
[§FS-terms.terms.7](FS-terms.md#terms7-values-and-integrations) (value).

- **entrypoint** — A file an agent reads at session start that `init` writes or updates — the
  canonical `AGENTS.md`, or an agent's own.
- **companion** — An entrypoint written beside the canonical one for an agent whose directory
  already exists. A symlink to `AGENTS.md` counts as one.
- **managed block** — The region of an entrypoint `init` owns and rewrites whole. Every byte
  outside it is preserved, including line endings.
- **delimiter** — The comment line that opens a managed block and the one that closes it.
- **legacy block** — A block at v3 or earlier, written before the delimiters: its H2 heading is
  the begin delimiter and the next H1 or H2 ends it.
- **template** — What `init` lays down: a `grund.toml`, and a starter file for each home that
  config names. Two are built in ([§FS-init.1.6](FS-init.md#16-templates)).
- **starter file** — A minimal file a template places in a home to teach its layout. Only
  `--scaffold` lays one ([§FS-init.2.1.3](FS-init.md#213-the-software-development-starter-files)); bare `init` never does ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)).

## requirements: What every init run holds to

Five requirements on `init` as a whole, carrying the marks of [§FS-config.requirements](FS-config.md#requirements-what-the-config-contract-holds-to). None says what `init` may touch: writing only what it owns, and replacing a whole file only under `--force`, are [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) and [§REQ-no-data-loss.3](../requirements/REQ-no-data-loss.md#3-destructive-is-opt-in-and-never-a-side-effect), and [§FS-init.3](FS-init.md#3-non-intrusive-guarantees) lists them. None says what its bytes may name either: what it writes resolves where it lands under [§REQ-shipped-surfaces.1](../requirements/REQ-shipped-surfaces.md#1-no-shipped-or-printed-byte-names-a-declaration-of-this-repository), and a change to the block's fixed text moves the block version under [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations). [§REQ-agents-md](../requirements/REQ-agents-md.md#req-agents-md-the-agent-entrypoint-stays-managed-and-grounded) is this repository's own entrypoint, not a property of `init`.

### requirements.clean: What `init` writes, `check` accepts — realized

A tree `init` has just written passes `grund check` unmodified: the scaffold and the managed block are clean as written ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place), [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), the worked example is escaped and no bare ID-shaped token is written, so the block reads clean under `strict = false` too ([§FS-init.2.3.8.2](FS-init.md#2382-the-worked-example-is-escaped), [§FS-init.2.3.7.1](FS-init.md#2371-the-current-version)), and an `init` followed by `fmt --write` is one tree that both `fmt --check` and `init --check` accept ([§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules)). Where the two commands read one input they read it alike: an unreadable config is the error `check` gives for it ([§FS-init.2.3.8.3](FS-init.md#2383-a-config-that-fails-to-load)), an entrypoint is attributed on the evidence `check` uses ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)), and workspace members expand by `check`'s rules ([§FS-init.2.3.4.15.3](FS-init.md#234153-foreign-project-rows)). `init` is not among [§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)'s surfaces, so this is where that agreement is asked of it. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.reproducible: What `init` writes is a function of the target, its config, the flags and the version — realized

Two runs at one `grund` version over the same target with the same flags write the same bytes: the scaffold ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)), the managed block ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), and each entrypoint's clickable-citation form, which the written path decides and which embeds nothing about the machine ([§FS-init.2.3.6](FS-init.md#236-clickable-citations), [§FS-init.2.3.4.17.2](FS-init.md#234172-the-form-per-agent)). Of the target's ancestors only the workspace that claims it shapes those bytes ([§FS-init.2.3.4.15.1](FS-init.md#234151-which-workspace)); the name and the effective config come from the target alone, never from an ancestor's config ([§FS-init.2.3.8](FS-init.md#238-substituted-content)). While an optional member is absent, the member list written is the one the entrypoint already holds ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)): bytes of the target, not of the machine. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible); [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) names only the commands that read a tree, so this is the writer's half.

### requirements.faithful: The block describes the repository it lands in, and claims nothing `check` does not enforce there — realized

What the block says about the repository is rendered from its effective config, never from the defaults ([§FS-init.2.3.8.1](FS-init.md#2381-what-the-config-fills-in)): its sentences teach the configured marker ([§FS-init.2.3.4.17.1](FS-init.md#234171-the-marker)), its project map is the configured layout ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)), named-section guidance appears only where the feature is on ([§FS-init.2.3.4.5](FS-init.md#2345-declaration-forms)), and its direction and grounding rules are the ones `check` applies ([§FS-init.2.3.4.10](FS-init.md#23410-citation-direction), [§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence)). A config change that makes it untrue anywhere is caught by `check`'s byte comparison of the whole block ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check), [§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)); a workspace change that makes its workspace sections untrue is caught by the `check` at the outermost root of a full checkout ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)), and in every other run by `init --check` ([§FS-init.1.4](FS-init.md#14-why---check-exists)), save the member list in a checkout without an optional member, which only the full checkout's root `check` judges ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)). Serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers).

### requirements.total: A run that exits `2` leaves the tree as it was — realized

Every reason `init` can read before writing refuses the whole run, as [§FS-init.4.2](FS-init.md#42-2-wins-over-1) assumes when it calls such a run one that could not be performed: a refused target ([§FS-init.1.2](FS-init.md#12-refused-targets)), an unreadable config ([§FS-init.1.2.4](FS-init.md#124-which-refusal-is-reported-first)) and a CLI error write nothing, and so does an entrypoint whose block is newer than the binary or malformed ([§FS-init.2.3.10.2](FS-init.md#23102-a-newer-block-version), [§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)): every planned entrypoint's block is read before the first is written, so one planned ahead of it is left as it was ([§FS-init.1.2.6](FS-init.md#126-entrypoint-blocks-are-read-before-any-is-written)). An I/O failure while writing is outside this point. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.next-agent: An agent joins on every axis at once — directional

The supported-agent set is one list every agent-facing feature reads ([§FS-init.2.3.12](FS-init.md#2312-supported-agents-and-their-entrypoints)). An agent added to it from here on arrives, in one change, with its flag ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)); its place in automatic selection, created only on a signal that attributes it unambiguously ([§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection), [§FS-init.3.1](FS-init.md#31-ambiguous-companions-need-an-unambiguous-signal)); the one-created-entrypoint rule ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)); its clickable-citation form under the per-agent gate ([§FS-init.2.3.4.17.2](FS-init.md#234172-the-form-per-agent)); the refusal of [§FS-init.1.2.2](FS-init.md#122-user-global-instruction-files) for any user-global instruction file it reads ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)); and `check`'s verification of what `init` writes for it ([§FS-check.3.5.1](FS-check.md#351-which-entrypoints-are-verified)). This binds the next agent; it is not a claim about how the present ones were added. Serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers).

## 1. Inputs

```
grund init [<path>] [--name <name>] [--description <text>] [--template <name>] [--scaffold] [--force] [--dry-run] [--check] [--no-vcs] [--agents-md] [--claude] [--gemini] [--pi] [--copilot] [--cursor] [--windsurf] [--zed]
```

- `<path>` — directory in which to scaffold; defaults to `.`. Applies to every form of `init`, `--scaffold` included, and prefixes every emitted path in [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place). Must exist: `init` does not create the target directory, because a missing target is a user error, not something to silently paper over.
- `--name <name>` — human-readable project name baked into the generated `AGENTS.md` heading when canonical `AGENTS.md` is selected, and into the `grund.toml` `project_name` key when that config is freshly written. An explicit value wins; otherwise `init` uses `project_name` from a config at either supported location in the target, and only when the target has no configured name does it fall back to the basename of `<path>` resolved to an absolute path. An ancestor's config never supplies this default, and an existing target config is never rewritten ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)).
- `--description <text>` — one-line project description baked into the generated `grund.toml` `project_description` key ([§FS-config.3](FS-config.md#3-keys-version-1)), replacing the commented teaching line [§FS-init.2.4](FS-init.md#24-generated-grundtoml) writes by default. No default: `init` never invents a description. A `<text>` containing a line break is a CLI error, mirroring the config-side single-line rule. Like `--name`, the value only lands in a freshly written config — an existing `grund.toml` is never modified ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)).
- `--template <name>` — the template to lay: `software-development`, the default, or `none` ([§FS-init.1.6](FS-init.md#16-templates)). `--template=<name>` is the same flag. The template is where the generated config and the starter files come from; a target that already has a config keeps it ([§FS-init.1.6.5](FS-init.md#165-a-config-that-is-already-there)).
- `--scaffold` — also lay the template's starter files at the homes the effective config names: [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place) lists the default set, and [§FS-init.1.3](FS-init.md#13---scaffold-and-the-effective-config) says why each is there and how the config changes it. Off by default — most adopters already have a `docs/` of some shape and want only the entry point and config, which is what bare `init` writes ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)). Refused beside `--template=none` ([§FS-init.1.6.4](FS-init.md#164-none-has-nothing-to-scaffold)).
- `--force` — overwrite a selected canonical `AGENTS.md` and the starter files `--scaffold` lays where they already exist, never the config ([§FS-init.3.4](FS-init.md#34-with---force), [§FS-init.3.5](FS-init.md#35-what---force-does-not-replace)). Off by default. Existing entrypoints are appended to or updated without it ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), so it is only needed to reset a generated `AGENTS.md` or a starter file to its canonical bytes.
- `--dry-run` — preview the run without writing or modifying any file: each `wrote `, `appended `, or `updated ` line of the report ([§FS-init.2.2](FS-init.md#22-stdout--stderr)) is emitted as `would-write `, `would-append `, or `would-update `, while `exists ` lines and the `next:` block are unchanged. Composes with every other flag, including `--force`. Off by default.
- `--check` — the same preview, taken as a verdict: nothing is written, the report is the one `--dry-run` prints for this tree ([§FS-init.2.2](FS-init.md#22-stdout--stderr)), and the run exits `1` when any line of that report is a `would-…` ([§FS-init.4](FS-init.md#4-exit-codes)). It is what a pre-commit hook or a CI job runs; [§FS-init.1.4](FS-init.md#14-why---check-exists) says why it exists. Composes with every other flag exactly as `--dry-run` does, `--force` included; passing both is redundant, not an error, since both say *write nothing*. Off by default — bare `grund init` still writes.
- `--no-vcs` — scaffold into a target outside any version-controlled tree, which is otherwise refused ([§FS-init.1.2](FS-init.md#12-refused-targets)). Off by default. It says only *this really is where I want a project*: it does not lift the unconditional refusals of [§FS-init.1.2](FS-init.md#12-refused-targets), and it is not `--force` (which decides whether files `init` owns get overwritten, [§FS-init.3](FS-init.md#3-non-intrusive-guarantees)) — the two answer different questions, and a run may need either, both, or neither.
- `--agents-md`, `--claude`, `--gemini`, `--pi`, `--copilot`, `--cursor`, `--windsurf`, `--zed` — explicitly create or update that agent's entrypoint ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)); with none of them, `init` runs in automatic mode.

Per [§FS-non-goals.10](FS-non-goals.md#10-interactive-mode), `init` is non-interactive: it never prompts. Every choice is a flag.

### 1.1 Usage examples

```
grund init                       # cwd: the config and the agent entrypoint, no starter files
grund init path/to/repo          # auto-detect existing entrypoint, else AGENTS.md
grund init --scaffold            # cwd, plus the starter files: docs/, tests/e2e/, tests/integration/
grund init --scaffold path/to/repo # explicit target, with the same starter files
grund init --template=none       # a grund.toml that declares no kinds, for a repo that declares its own
grund init --dry-run path/to/repo # preview without writing anything
grund init --check path/to/repo  # same preview as a gate: exit 1 if anything is pending
grund init --claude --gemini path/to/repo # create/update both agent entrypoints
grund init --scaffold --name acme path/to/repo  # full form
```

Argument order is flexible: positional `<path>` may appear before or after the flags.

### 1.2 Refused targets

`init` is the only subcommand that can write a file the user never named, and every path it writes is `<path>`-relative, which makes `<path>` load-bearing in a way no other command's is. So before it touches anything, `init` checks *where* it was pointed. Every check below runs before the first write, and a refusal is total: no file is created, appended, or updated, not even the ones that would have been unobjectionable, and `init` exits `2` ([§FS-init.4](FS-init.md#4-exit-codes)). `--dry-run` reports the same refusal rather than a preview — a preview of a run that would be refused *is* that refusal.

- **The home directory is refused, unconditionally.** A `<path>` that resolves to `$HOME` is declined and nothing is written. No flag lifts this — not `--no-vcs`, not `--force` ([§FS-init.1.2.1](FS-init.md#121-the-home-directory)).
- **A user-global instruction file is refused, unconditionally.** `init` declines when any path it would write, append to, or update — the canonical `AGENTS.md` included — is one of the file-backed user-global targets of [§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions) ([§FS-init.1.2.2](FS-init.md#122-user-global-instruction-files)).
- **A target outside version control is refused unless `--no-vcs` is passed.** `init` looks for a `.git`, `.hg`, `.jj`, or `.svn` entry in `<path>` or any ancestor and declines when it finds none, naming the flag that proceeds anyway ([§FS-init.1.2.3](FS-init.md#123-version-control)).

- **An entrypoint whose managed block `init` cannot update is refused.** A planned entrypoint that already exists and carries a malformed block ([§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)) or a newer block version than the binary supports ([§FS-init.2.3.10.2](FS-init.md#23102-a-newer-block-version)) declines the whole run, every other planned entrypoint included ([§FS-init.1.2.6](FS-init.md#126-entrypoint-blocks-are-read-before-any-is-written)).

Which of two problems a run is told about first is [§FS-init.1.2.4](FS-init.md#124-which-refusal-is-reported-first), and what a refusal says is [§FS-init.1.2.5](FS-init.md#125-the-refusal-message).

#### 1.2.1 The home directory

`$HOME` is where the file-backed user-global agent instruction files of [§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions) live, and `init`'s repository-relative model is wrong about every one of them: `<path>/.claude/CLAUDE.md` with `<path>` at `$HOME` **is** `~/.claude/CLAUDE.md`, the machine-global file every agent session in every project loads. The collision is not incidental. The same signal — `.claude/` exists — means "this machine runs Claude, sync the small marked rendering block" to `grund integrations --write` and "this project uses Claude, scaffold the managed block" to automatic mode ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)). Both readings are correct in their own scope; in `$HOME` they name one file, and the project-shaped one would append a specific repository's instructions to every session the user ever starts. Nobody targets `$HOME` on purpose, so there is no case to keep working and no escape hatch to offer.

The home directory is whichever one the platform reports, so the rule holds wherever `init` runs. The user-global table [§FS-init.1.2.2](FS-init.md#122-user-global-instruction-files) checks is resolved separately — against `$HOME`, save that a `~/.config/…` row such as Zed's follows `$XDG_CONFIG_HOME` when it is set ([§FS-integrations.4.1.7](FS-integrations.md#417-where-a--target-resolves)) — so on a platform that does not set `$HOME` this rule is the one that still answers for `<path>` at the home directory itself.

#### 1.2.2 User-global instruction files

This is not the `$HOME` rule restated: that one fires when `<path>` **is** the home directory, and `<path>` is arbitrary. `<path>/AGENTS.md` with `<path>` at `~/.codex`, `~/.config/zed`, or `~/.pi/agent` **is** that table's file, as is `<path>/GEMINI.md` with `<path>` at `~/.gemini` and `<path>/CLAUDE.md` with `<path>` at `~/.claude` — each a target the `$HOME` rule has nothing to say about, and on a machine whose dotfiles are a repository, which is the usual state of a machine that has those directories, the version-control rule has nothing to say either. Three of those five are the canonical `AGENTS.md`, the entrypoint `init` reaches for by default and the one file it *overwrites* rather than appends to under `--force` ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)), so this rule is checked for every planned path including the canonical one.

The division is the one [§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions) already states: the user-global files carry machine-wide *policy* and are `grund integrations --write`'s to manage, the repository entrypoint carries this project's *syntax* and is `init`'s.

#### 1.2.3 Version control

Presence is tested, not type: a linked worktree and a submodule both write `.git` as a file. This is not `grund` reading history — nothing is parsed, no command is run, and [§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking) is untouched; the entry's existence is a fact about the tree in exactly the way a `grund.toml`'s is. Unlike the two rules above this one has a legitimate other side — scaffolding a directory before `git init`, or a project under no VCS at all — so it is a default, not a law, and `--no-vcs` lifts this rule only ([§FS-init.1](FS-init.md#1-inputs)).

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

#### 1.2.6 Entrypoint blocks are read before any is written

The location rules are not the only refusals a run can meet. Every planned entrypoint that already exists — the canonical `AGENTS.md` and each companion alike ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)) — is read and its managed block classified before the first one is written, and a block `init` would refuse to update refuses the run: a malformed delimiter pair ([§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)) or a block newer than the binary ([§FS-init.2.3.10.2](FS-init.md#23102-a-newer-block-version)). The refusal is total in the sense of [§FS-init.1.2](FS-init.md#12-refused-targets): no entrypoint is created, appended, or updated, `grund.toml` and the starter files `--scaffold` lays are not written, no `wrote`, `appended`, or `updated` line is reported, and `init` exits `2` with the message that names the file and the defect. It does not matter where the refused file sits in the order the entrypoints are written; a clean `CLAUDE.md` planned before a malformed `GEMINI.md` is left byte-for-byte as it was. `--dry-run` and `--check` report the same refusal rather than a preview.

The check runs after the user-global rule ([§FS-init.1.2.4](FS-init.md#124-which-refusal-is-reported-first)), because both read the same plan; a run with several refused blocks is told about the first in the order the entrypoints are written. Checking in the write pass alone is not enough: by the time it meets the second entrypoint it has already written the first, and a run that exits `2` would leave a half-scaffolded tree that `grund check` cannot even read for want of a `grund.toml`.

### 1.3 `--scaffold` and the effective config

The `docs/functional-spec/README.md` starter file is scaffolded because the generated config's `FS` row lists that folder as its one place, `folders = ["docs/functional-spec"]` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)), and no `requirements.md` is laid. Every home `--scaffold` lays is a place the governing config names: a row's `files` and `folders` in a version-2 config, a row's `file` or `folder` in a version-1 one. An existing config that omits `[[kinds]]` takes the same folder from the built-in `FS` row ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), so `--scaffold` lays `docs/functional-spec/README.md` there too and points next-step guidance at `docs/functional-spec`. `roadmap.md` and `changelog.md` are scaffolded because the generated managed block's `docs/` table links to them ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)).

### 1.4 Why `--check` exists

A managed block can go stale in its *text* while its `(vN)` heading is still current: a config edit, a migration, or a member joining the workspace changes what `init` would write without changing the grund that writes it. `--dry-run` has always seen and reported that drift, but as a preview rather than a verdict. `grund check` now holds the whole block too ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)), so `--check` stays the gate for the two things `check` leaves out: a file `init` would create that `check` does not require, such as an agent-directory companion ([§FS-check.3.5.1](FS-check.md#351-which-entrypoints-are-verified)), and the two workspace sections in a run that does not compare them ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)). In a checkout without an optional member it holds those sections save the member list, which `grund init` keeps there as it is and so has nothing to report for ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)).

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

### 1.6 Templates

A template is what `init` lays down: the `grund.toml` it writes when the target has no config ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)), and a starter file for each home that config names, which only `--scaffold` lays ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)). `--template <name>` picks one; `--template=<name>` is the same flag. The managed block belongs to no template: it is rendered from whichever config governs the target ([§FS-init.2.3.8](FS-init.md#238-substituted-content)). Two templates are built into the binary ([§FS-init.1.6.1](FS-init.md#161-software-development-the-default), [§FS-init.1.6.2](FS-init.md#162-none)); a template read from a path is not supported yet ([§FS-init.1.6.3](FS-init.md#163-an-unknown-name)).

#### 1.6.1 `software-development`, the default

The template `init` lays when no `--template` is given: the minimal version-2 `grund.toml` of [§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default), whose `[schema.kinds]` rows are `GRUND`, `GOAL`, `FS`, `REQ`, `AR`, `DF` and `DA`, the non-citable `e2e` and `integration` test kinds, and `RM`, each with its places and its title, and the ten starter files [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place) lists. `grund init` and `grund init --template=software-development` lay byte-identical trees.

#### 1.6.2 `none`

A version-2 `grund.toml` that declares no kinds, the envelope alone with no `[schema.kinds]` row ([§FS-init.2.4.9](FS-init.md#249-the-none-templates-config)), and no starter files. It is for a repository that declares its own kinds, the existing-structure adoption of [§FS-init.5.2](FS-init.md#52-adopting-an-existing-docs-heavy-repo). The tree it lays passes `grund check` and `grund init --check` unmodified, and the managed block it renders teaches how to add the first kind ([§FS-init.2.3.4.4.2](FS-init.md#23442-no-kinds-yet)).

#### 1.6.3 An unknown name

Any other name is refused before anything is read or written, exit `2`. The message names the built-in templates and says that a template from a path is not supported yet:

```
error: unknown template `docs` — the built-in templates are software-development and none; a template from a path is not supported yet
```

A `--template` with no value is a CLI-level error, as a `--name` with none is.

#### 1.6.4 `none` has nothing to scaffold

`--template=none --scaffold` is refused the same way, exit `2` with nothing written, because the template has no starter files to lay:

```
error: --scaffold has nothing to lay: the none template has no starter files
```

#### 1.6.5 A config that is already there

A target that already has a config keeps it, whichever template was asked for: `init` never writes a config over one ([§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)), and the managed block is rendered from the config it keeps. When `--template` was passed explicitly the run says so in a note ([§FS-init.2.2.1](FS-init.md#221-notes)), so the flag never silently does nothing:

```
note: grund.toml exists, so the none template's grund.toml is not written — init never overwrites a config
```

The note names the config at whichever discovery location it was found, and its wording is true of a preview and of a write alike. A run with no `--template` asked for no template config, so it gets no note. Under `--scaffold` the starter files are still laid, at the homes the existing config names ([§FS-init.1.3](FS-init.md#13---scaffold-and-the-effective-config)).

### 1.7 `--docs` is the deprecated spelling of `--scaffold`

`--docs` was the name `--scaffold` had before it. grund 0.18.0 kept it as a deprecated spelling under [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): it laid exactly what `--scaffold` lays and printed one warning on stderr naming the replacement and the release that removes it ([§DF-init-templates.2.2](../decisions/functional/DF-init-templates.md#22---docs-warns-until-0190-on-stderr-alone)). That release is 0.19.0, and the spelling is gone ([§DF-docs-flag-removed-in-0-19](../decisions/functional/DF-docs-flag-removed-in-0-19.md#df-docs-flag-removed-in-0-19-grund-init---docs-is-an-unknown-flag-from-0190)): `--docs` is an unknown flag, refused as every unknown flag is, before anything is written, with exit `2` and an empty stdout ([§FS-init-fixtures.4](FS-init-fixtures.md#4-target-and-flag-failures)):

```
error: unknown flag `--docs`
```

The bindings' `docs` option is not a command-line spelling and was never deprecated, so it keeps its name ([§DF-init-templates.2.3](../decisions/functional/DF-init-templates.md#23-the-bindings-keep-docs)). This address, heading text included, is kept so citations written while the spelling was deprecated still resolve.

## 2. Outputs

### 2.1 Files written, updated, or left in place

Bare `init`, with no `--scaffold`, writes these two and nothing else ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)):

- Agent entrypoints — see [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints). In automatic mode ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)), existing known entrypoints are appended or updated in place: `<path>/AGENTS.md`, `<path>/AGENTS.override.md`, `<path>/CLAUDE.md`, `<path>/.claude/CLAUDE.md`, `<path>/GEMINI.md`, `<path>/.pi/AGENTS.md`, `<path>/.github/copilot-instructions.md`, `<path>/.cursor/rules/grund.mdc`, `<path>/.cursorrules`, and `<path>/.windsurfrules`, excluding companion symlinks to `AGENTS.md`. If none exist, a missing companion is created for each agent whose agent directory already exists, one per agent ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)); if there are still no entrypoints to update or create, canonical `AGENTS.md` is written. [§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection) gives the rules of each step. Explicit flags (`--agents-md`, `--claude`, `--gemini`, `--pi`, `--copilot`, `--cursor`, `--windsurf`, `--zed`) create or update their requested entrypoints regardless of automatic detection, under the same one-per-agent rule ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)).
- Scoped entrypoints: under scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)), `<dir>/AGENTS.md` in each destination directory that already exists, with the `CLAUDE.md` and `GEMINI.md` symlinks [§FS-init.2.3.5.14.3](FS-init.md#235143-companions-in-a-scoped-directory) mirrors there, and the removal of the ones the list on disk names that no longer receive guidance ([§FS-init.2.3.5.14.2](FS-init.md#235142-orphans-are-removed)).
- `<path>/grund.toml` — see [§FS-init.2.4](FS-init.md#24-generated-grundtoml). Written only when the target carries no config in either discovery form ([§FS-config.1](FS-config.md#1-file-location-and-discovery)); a target that already has one is reported with `exists ` at the name it was found under (`exists .agents/grund.toml` for a repo on that form) and gets no second config.

With `--scaffold`, additionally the template's starter files ([§FS-init.2.1.3](FS-init.md#213-the-software-development-starter-files)). These are the `software-development` template's; the `none` template has none ([§FS-init.1.6.4](FS-init.md#164-none-has-nothing-to-scaffold)):

- `<path>/docs/functional-spec/README.md`
- `<path>/docs/requirements/README.md`, when the config declares `REQ` ([§FS-init.2.1.3](FS-init.md#213-the-software-development-starter-files))
- `<path>/docs/grund.md`
- `<path>/docs/goals.md`
- `<path>/docs/roadmap.md`
- `<path>/docs/changelog.md`
- `<path>/docs/architecture/README.md`
- `<path>/docs/decisions/architectural/README.md`
- `<path>/docs/decisions/functional/README.md`
- `<path>/tests/e2e/README.md`
- `<path>/tests/integration/.gitkeep`

The exact bytes for a given `grund` version are the `software-development` template's ([§FS-init.1.6.1](FS-init.md#161-software-development-the-default)), embedded in the binary, and two `grund init --scaffold` runs at the same version with the same `--name` produce byte-identical scaffolds ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). `grund check` is clean against the freshly-scaffolded tree.

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

#### 2.1.3 The `software-development` starter files

Each scaffolded markdown file is a minimal starter — enough structure to teach the layout, no real content:

- `grund.md` — the H1 plus a one-line note on how the project's reason for being is declared inline (`# GRUND-NNN-slug: …`), then the three H2 sections (`## 1. The problem`, `## 2. What this project does about it`, `## 3. Who it is for`), each with a one-line italic prompt to be replaced.
- `goals.md` — the H1 plus a one-line note on how goals are declared inline (`# GOAL-NNN-slug: …`).
- `functional-spec/README.md` — the index of the `FS` folder, in the shape of the decision READMEs below: the H1, the note on how `FS-` IDs declare one per file into the directory, the index rule, and an empty `| ID | Subject |` table. It is written as `README.md` in the folder the governing config's `FS` row names; a config whose `FS` row is a `file` gets that file instead, holding the H1 and a one-line note on how `FS-` IDs are declared inline as H2 headings in it.
- `requirements/README.md` — the index of the `REQ` folder ([§FS-init.2.4.10](FS-init.md#2410-req-is-declared-in-docsrequirements)), in the shape of the decision READMEs below: the H1, the note on how `REQ-` IDs declare into the directory, the index rule, and an empty `| ID | Subject |` table. It is written right after the FS home, as `README.md` in the folder the governing config's `REQ` row names, and only when that config declares `REQ` as a `folder`: a config with no `REQ` row, as every config an earlier `init` wrote, gets no `REQ` starter.
- `roadmap.md`, `changelog.md` — the H1 plus a single `<!-- placeholder - replace with real content -->` line.
- `architecture/README.md` — the H1, the navigational note about how `AR-` IDs declare into the directory and the rule that the index lists every architecture declaration as a full link ([§FS-check.3.18](FS-check.md#318-declaration-missing-from-its-kinds-index)), and an empty `| ID | Subject |` table to fill in.
- `decisions/architectural/README.md`, `decisions/functional/README.md` — the same shape for the two decision folders: the H1, the note on how `DA-`/`DF-` IDs declare into the directory and on citing a decision from the declaration it settles, the index rule, and an empty `| ID | Subject |` table. Every citable folder kind whose `index` the generated config leaves at its default gets one, which under those defaults is `FS`, `REQ`, `AR`, `DF`, and `DA` — the two test kinds are non-citable and have no index at all ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)).
- `tests/e2e/README.md` — the H1 (`# e2e`) plus a note that every behaviour described in the effective FS home has at least one case, and that each case cites the declaration it proves. It is a layout note, not an index: `e2e` is a non-citable kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) and has no declarations to list. `tests/integration/` gets the `.gitkeep` instead of a README of its own, so the second configured test home survives a `git add` without a second near-identical note.

Every scaffold-owned ID-shape example uses the effective format of the kind it
illustrates: that kind's `[[kinds]].format` when present, otherwise the
repository `[id].format` ([§FS-config.3.2](FS-config.md#32-id--id-grammar),
[§FS-config.3.4.10.1](FS-config.md#34101-format)). A kind row without a format
and an absent kind row both use the repository fallback. The examples remain
schematic: they substitute the illustrated kind and the `<NNN>` / `<slug>`
components into the configured template while preserving its literal
delimiters. This supports the numbered-with-slug, number-only, and slug-only
forms.

The rule covers the declaration example in `grund.md`, `goals.md`, and the FS
file a config names, when it names one; the declaration/bare ID, citation ID
portion (without changing its marker or section suffix), and inline-stub ID in
`architecture/README.md`; and the declaration example in each decision README,
in `functional-spec/README.md`, and in `requirements/README.md`.

The `.gitkeep` file exists solely so an empty directory that gets no README of its own survives a `git add`; its content is a single line: `# placeholder — replace this directory's contents with real integration tests`.

### 2.2 Stdout / stderr

On success, stderr lists every path touched, one per line:

```
wrote AGENTS.md
wrote grund.toml
```

The prefix is `wrote ` for a newly written or overwritten file, including an agent-directory-triggered companion; `appended ` when the versioned `grund` block was added to an existing agent entrypoint; `updated ` when an existing managed block's bytes changed (an older block upgraded, or a same-version block re-rendered against a changed template or config — [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)); and `exists ` when an existing file was left unchanged — including an agent entrypoint whose managed block already matches the current rendered block byte-for-byte, in which case `init` rewrites nothing, so re-running `grund init` on an up-to-date repo touches no files. Under `--dry-run`, those three write prefixes are replaced with `would-write `, `would-append `, and `would-update ` and nothing is written to disk; `exists ` lines are identical to a non-dry run. A companion file that is a symlink to `AGENTS.md` is omitted from the output; when the canonical file is selected, that update already covers the symlink target. Scoped files add two prefixes, `linked ` for a companion symlink created and `removed ` for a file or symlink deleted, with `would-link ` and `would-remove ` under `--dry-run`, and their lines follow the root entrypoints' ([§FS-init.2.3.5.14.4](FS-init.md#235144-the-report-names-every-scoped-file-and-link-it-touched)).

After the file list come any `note:` lines ([§FS-init.2.2.1](FS-init.md#221-notes)), then the `next:` block ([§FS-init.2.2.2](FS-init.md#222-the-next-block)). `--check` ([§FS-init.1](FS-init.md#1-inputs)) prints the same conditional report as `--dry-run` and everything after it — the `note:` lines, the `next:` block, and the rule that suppresses it — so the two runs differ in one thing only: the exit code each draws from what it printed ([§FS-init.4](FS-init.md#4-exit-codes)).

Paths are relative to `<path>`. Stdout is always empty ([§FS-errors.6](FS-errors.md#6-the-grund-init-transcript) — `init`'s output is the scaffold it wrote to disk, and the transcript is progress, not output).

#### 2.2.1 Notes

The run's `note:` lines, one per line, each name something the run could not do, or a state of the tree the caller would otherwise have to notice for itself: a duplicated entrypoint ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)), a Claude entrypoint shadowed by a symlink ([§FS-init.2.3.4.17](FS-init.md#23417-clickable-citations)), a template config the target's own config kept out ([§FS-init.1.6.5](FS-init.md#165-a-config-that-is-already-there)), a member list kept or written without an optional member that is absent ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)). A note is a report, not a finding: it changes nothing, and the exit code is the one the run would have had without it ([§FS-init.4](FS-init.md#4-exit-codes)).

Under `--dry-run` a note is written in the conditional, like every other line of that report: the run wrote nothing, so a note in the present tense describes a tree that does not exist — and an instruction that assumes the write already happened, such as deleting the symlink a preview did not replace, costs the reader the only entrypoint they have. What such an instruction waits on is the **block**, not the file: a preview's `would-append` names an entrypoint that already exists, so a condition the reader can satisfy by looking is no condition at all.

#### 2.2.2 The `next:` block

After the notes, stderr prints a short `next:` block — a blank line, then numbered first steps, then `see <entrypoint> for the full workflow.` — so the user is not left at a bare list of paths wondering what to do. `<entrypoint>` is the first agent entrypoint `init` wrote, updated, or found already current. The steps are: run `grund check`; allocate an ID with `ID=$(grund id FS "…")` and add it to the effective FS home (under the FS folder with `# <ID>: …`, as the generated config and the built-in default make it, or in the configured FS file with the `## <ID>: …` H2 when explicit config makes FS a file); cite it as `§<ID>` from the docs and e2e tests that depend on it.

The check step explains that on a fresh scaffold, before the first declaration exists, `grund check` warns `nothing recognized` under [§FS-check.4.5](FS-check.md#45-nothing-recognized) and exits `0`. This is expected and needs no repair: the next step begins grounding by adding the first declaration. Both the direct `--scaffold` guidance and the non-scaffold advice about the next scaffold teach this expectation, conditional on a **fresh** scaffold; adopting grund into an already populated repository does not predict that warning. Neither form promises a clean success verdict for a fresh scaffold. The exact check-step wording appears in [§FS-init-fixtures.1](FS-init-fixtures.md#1-default-form), [§FS-init-fixtures.1.1](FS-init-fixtures.md#11-populated-configured-form), and [§FS-init-fixtures.2](FS-init-fixtures.md#2-scaffold-form).

When `--scaffold` was *not* passed, a re-run advice is inserted as step 1, the following steps move down, the allocation step drops its second line — the one naming the `## <ID>: …` H2 or `# <ID>: …` H1, which prints only under `--scaffold` — and the citation step is omitted. Whenever it is printed, that advice points at re-running with `--scaffold` (or creating the effective FS home, `docs/`, and `tests/` by hand); a refresh of a complete setup does not print it at all ([§FS-init.2.2.2.1](FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing)). It appends the clause `until then \`grund check\` has nothing to scan` only when the effective scanner would read no file — exactly the scan decision of the governing config's version. For version 2 that is the scanned rows' places and the complement's extent, the config root unless `[schema.kinds.code]` lists places ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)), so the starter's own tree, `AGENTS.md` included, is read and a fresh run does not print the clause. For version 1 it is the scan decision in [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned): configured extensions, ignore and exclude rules, hidden-file and symlink handling, workspace boundaries, configured include paths, scanned kind homes, and `scan = false` all retain their scanner meanings. Path existence or a non-empty directory alone does not make the clause false. A config that declares no kinds gets the block of [§FS-init.2.2.2.2](FS-init.md#2222-a-project-with-no-kinds) instead.

The `next:` block is suppressed entirely when every reported path is `exists ` (or, under `--dry-run`, every reported path is `exists ` and no `would-…` lines were emitted) — the user already has a complete, current grund setup, so there is no next step to teach. It is suppressed for the same reason on a refresh of a complete setup ([§FS-init.2.2.2.1](FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing)). When it is printed, it is guidance, not a finding: part of the success output, with no effect on the exit code.

##### 2.2.2.1 A refresh of a complete setup teaches nothing

A run with no `--scaffold` is a **refresh of a complete setup** when both hold:

- every reported path is `exists ` or `updated ` — under `--dry-run` and `--check`, `exists ` or `would-update ` — so a single `wrote `, `appended `, `would-write `, or `would-append ` line makes the run something else;
- the effective FS home exists on disk: the configured FS `file`, else the configured FS `folder`, else the built-in `FS` folder `docs/functional-spec` ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), resolved against the target directory.

The lines for scoped files and their companion symlinks ([§FS-init.2.3.5.14.4](FS-init.md#235144-the-report-names-every-scoped-file-and-link-it-touched)) are not counted in the first condition: they follow the root block, so a refresh that moves a row's guidance, creating one scoped file and removing another, is still a refresh.

Such a run prints no `next:` block. It is where `grund check` sends a repository whose managed block a `grund.toml` change made stale ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)); after it the user again has the complete, current setup the suppression above describes, and the advice to scaffold an FS home that exists would send them hunting for a detection failure that did not happen. Their next step is the `grund check` that sent them here.

Everything else keeps the block exactly as [§FS-init.2.2.2](FS-init.md#222-the-next-block) renders it: a first run (`wrote ` or `appended ` of an entrypoint) even where the FS home, `docs/`, and `tests/` exist ([§FS-init-fixtures.1.1](FS-init-fixtures.md#11-populated-configured-form)), and a refresh whose effective FS home is absent. Suppressing the block moves no exit code: `--check` still exits `1` on a `would-update ` ([§FS-init.4](FS-init.md#4-exit-codes)).

##### 2.2.2.2 A project with no kinds

A config that declares no kinds ([§FS-config.3.4.4.5](FS-config.md#3445-kinds---declares-no-kinds)) has no FS home to scaffold or to allocate in, so every step above would send the user somewhere that does not exist — `--scaffold` included, which the `none` template refuses ([§FS-init.1.6.4](FS-init.md#164-none-has-nothing-to-scaffold)). The block teaches the one step that is missing instead, `<entrypoint>` as above:

```
next:
  1. declare a first kind in grund.toml, then run `grund init` again so AGENTS.md lists it
  2. run `grund check`
  3. allocate an ID:  ID=$(grund id <KIND> "…")  then add it to that kind's home
see AGENTS.md for the full workflow.
```

It is printed and suppressed by the rules of [§FS-init.2.2.2](FS-init.md#222-the-next-block), save that such a run is never a refresh of a complete setup ([§FS-init.2.2.2.1](FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing)): it has no effective FS home to find on disk, and the step it teaches is still open. The bindings read the same decision from the guidance's `declares_kinds` field ([§DF-init-templates.2.3](../decisions/functional/DF-init-templates.md#23-the-bindings-keep-docs)).

### 2.3 Generated agent entrypoints

The emitted agent guidance is a canonical managed block: it teaches the session-start workflow and rules listed in [§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections), rendered against the target repo's effective configuration ([§FS-init.2.3.8](FS-init.md#238-substituted-content)). The block stays concise under [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file): it teaches only the rules an agent needs before work begins, leaving detail to cited declarations and `grund <ID>` output.

The contract this spec makes is the *determinism and versioning*, not a literal transcript: two `grund init` runs at the same `grund` version against trees with the same `--name` and the same effective config produce byte-identical managed blocks ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)), and the block's fixed text is versioned ([§FS-init.2.3.7](FS-init.md#237-the-block-version)). Generated `init` output must pass `grund check` unmodified — the generator and the checker are never allowed to disagree about the block's own contents. The block is bounded by delimiters ([§FS-init.2.3.9](FS-init.md#239-delimiters)), written into an entrypoint by the rules of [§FS-init.2.3.10](FS-init.md#2310-writing-the-block) unless it is malformed ([§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)), and carried by the entrypoints [§FS-init.2.3.12](FS-init.md#2312-supported-agents-and-their-entrypoints) names.

#### 2.3.1 Position invariants

The managed block's **position within an existing agent entrypoint is preserved on update**. `init` does not move the block: a supported block sandwiched between user-authored sections is replaced in place with the current rendered block, with the surrounding sections — both before and after — left byte-identical. When the block is already byte-identical to the current render, nothing is rewritten at all (`exists `, [§FS-init.2.2](FS-init.md#22-stdout--stderr)) — the strongest form of "position preserved."

#### 2.3.2 Line endings

`init` does not normalize line endings. When updating or reading an existing agent entrypoint:

- The bytes outside the managed region ([§FS-init.2.3.9](FS-init.md#239-delimiters), delimited or legacy) are preserved byte-for-byte, including CRLF (`\r\n`) and lone-CR endings.
- The delimiter and H2 heading matching tolerates an optional trailing `\r` before the line end, so a CRLF-encoded file is detected correctly.
- The freshly-written block uses LF endings (the bytes embedded in the binary). On a CRLF-encoded host file the result is mixed line endings inside the managed region and CRLF outside; this is intentional. Normalizing the rest of the file would violate the "leave content alone" guarantee.

#### 2.3.3 Citation form

The managed block teaches a single citation form — `§<ID>`, bare, with an optional `.<section>` path. `[fmt.cross_refs].enabled` per [§DF-md-link-emission.2.4](../decisions/functional/DF-md-link-emission.md#24-default-on-for-generated-configs) governs whether `grund fmt --write` wraps on-disk citations in Markdown links; the managed block's guidance is the same in either mode — cite declarations by ID; any Markdown wrap is generated by `grund fmt`, not hand-authored.

The generated `grund.toml` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)) is version 2 and names no `[scan] include`: its scan is the union of its rows' places ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)), and its `FS` place is the `docs/functional-spec` folder ([§FS-init.2.4.10](FS-init.md#2410-req-is-declared-in-docsrequirements)).

#### 2.3.4 Managed-block content sections

The generated managed block is not a literal transcript in this spec ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), but its canonical template must preserve the following separately citable content sections. Each section is phrased compactly in the template, because the entrypoint is read at session start and serves [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file).

##### 2.3.4.1 Entrypoint

The block identifies itself as the entrypoint for agents working in the project and instructs them to read it.

##### 2.3.4.2 Citation Scheme

The block teaches the configured citation scheme: IDs have the configured shape, citations use the configured marker and optional section path, `grund check` validates citations, `grund list` discovers IDs, `grund <ID>` reads cited declarations or sections, and `grund refs` reports what leans on a declaration.

##### 2.3.4.3 Cheap Grounding

The block teaches the cheap grounding ladder: use `grund <ID>` as the first read for a bare citation, `grund <ID> --brief` when the heading and first paragraph are enough, `grund <ID> --toc` when section navigation is needed, `grund <ID>.<section>` for section citations, `grund <ID> --full` only when a narrower read is insufficient, `grund list --kind FS,AR` for scoped discovery, and `grund refs <ID> --summary` before a full citation listing. Beside that ladder it teaches `grund list --size=words --top 10` as the sweep to run when a specification feels heavy, so the agent can move detail into citable child sections before paying for an oversized lead.

Beside the `refs` rung, it teaches the before-and-after declaration maintenance
workflow of [§FS-init.2.3.4.9](FS-init.md#2349-declaration-blast-radius).

###### 2.3.4.3.1 Structural queries

Beside the ladder, which reads one coordinate at a time, the block teaches how to read structure across many in one bullet, with no pipe to leave the session for: `grund list --glob '<pattern>' --total` counts the coordinates a pattern selects ([§FS-list.total](FS-list.md#total---total-a-count-rung)), and `grund show --batch --glob '<pattern>' --toc --format json` reads each one's section map in the same scan ([§FS-show.1.9](FS-show.md#19---batch---all-every-coordinate-in-scope), [§FS-show.2.1.2](FS-show.md#212-section-map---toc)), from `.result.sections[]`. The bullet names `*`, `**` and `{a,b}` ([§FS-coordinate-globs.grammar](FS-coordinate-globs.md#grammar-grammar)), and links the user-facing query guide by its absolute URL, `https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md`, because the block lands in repositories that have no `docs/user-facing/` of their own ([§REQ-shipped-surfaces](../requirements/REQ-shipped-surfaces.md#req-shipped-surfaces-what-grund-ships-or-prints-resolves-where-it-lands)).

The pattern it shows is built from the project's `[id] format`: `<KIND>`, the literal the format writes after `{kind}`, then `*`. A `{kind}-{slug}` project is taught `<KIND>-*` and a `{kind}{number}` project `<KIND>*`, never a `-` its IDs do not have.

That guide, *Querying grund*, puts the query recipes in one place, each with its command, a short example, and its JSON shape: selecting units (`list --glob`, `list --selector`, `list --kind`); counting them (`list --total`); expanding a unit's subtree (`show --batch --glob` over `<ID>{,.**}`, read from `.result.sections[]`); bodies in bulk (`show --batch --brief|--full`, `--glob`, `--all`, [§FS-show.1.9](FS-show.md#19---batch---all-every-coordinate-in-scope)); who cites a point (`refs`, `--descendants`, [§FS-refs.1](FS-refs.md#1-inputs)); the whole citation graph (`cover --format json` with `enclosing_declaration` and `enclosing_section`, [§FS-cover.3.2](FS-cover.md#32---format-json)); and a review, which counts a checklist, reads its bodies, then maps changed lines to their owners with `cover FILE --lines N-M` ([§FS-cover.6](FS-cover.md#6-line-ownership)). The guide also states the grammar and its departures from the `exclude` dialect. The `grund-init` skill carries a matching section that teaches the glob recipe, with a placeholder pattern, and links the guide.

##### 2.3.4.4 Project Map

The block describes the configured kind homes from the effective `grund.toml` as a raw-readable Markdown link list (`- [KIND](home): Title`), so the generated instructions name and link the host repo's actual artifact layout instead of hard-coding the default layout or relying on table rendering. A kind that is a place rather than an ID space gets the row [§FS-init.2.3.4.4.1](FS-init.md#23441-rows-for-places) gives it, or none, and a config with no kind to list gets the paragraph of [§FS-init.2.3.4.4.2](FS-init.md#23442-no-kinds-yet) instead of an empty map. Each row's `Title` is the one [§FS-init.2.3.4.4.3](FS-init.md#23443-row-titles) gives it.

The scan scope (`[scan].include` / `[scan].exclude`) is *not* surfaced here — it is configuration an agent never needs to read inline, since `grund <ID>`, `grund list`, and `grund refs` apply it transparently. Where the init target lies in a workspace, whether or not its own config declares `[workspace]`, the sibling [§FS-init.2.3.4.15](FS-init.md#23415-workspace-members) block names the workspace projects; the Project Map itself describes the *current* project's declaration homes only and is unchanged in workspace mode.

###### 2.3.4.4.1 Rows for places

A **non-citable kind** ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is named by its **place** rather than its name — `- [skills/](skills): Agent review and automation skills` — because its name is a config key an agent can never write in a citation, while the directory is something it can go and read. An unscanned kind ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) is rendered the same way; the row is the point of configuring it. The **homeless kind** ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) — `code`, or whatever the project named it — has no place and therefore no row.

In a version-2 config a kind with several places is still one row, whose link list names every place in the row's place order, files then folders, each in written order ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)) — `- [FS](docs/spec/core), [docs/spec/plugins](docs/spec/plugins): What` — and a non-citable row is named by all of its places the same way. The complement, `code`, gets a row named by its places when it lists any, and none when its extent is the config root ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)). Untitled, that row says it is the complement ([§FS-init.2.3.4.4.3](FS-init.md#23443-row-titles)).

###### 2.3.4.4.2 No kinds yet

A config that declares no kinds ([§FS-config.3.4.4.5](FS-config.md#3445-kinds---declares-no-kinds)) has no row to render, and an empty map teaches nothing. The map is then one paragraph that teaches the first kind in the config's own spelling. For a version-1 config:

```markdown
No kinds are declared yet: `grund.toml` sets `kinds = []`. Replace it with one `[[kinds]]` table per kind of document this repository gives IDs to — `kind = "FS"`, then `folder = "docs/specs"` or `file = "requirements.md"`, and a `title` — and run `grund init` again to list them here.
```

A version-2 config with no row is taught `[schema.kinds.<KIND>]` and its `folders` or `files` the same way ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). Two other places in the block would otherwise name an empty set, and say so instead. The citation-scheme line's kind set reads ``no `KIND` is declared yet`` rather than `KIND ∈ {}` ([§FS-init.2.3.8.1](FS-init.md#2381-what-the-config-fills-in)). Where the config declares no `[citations]`, the citation-direction paragraph ([§FS-init.2.3.4.10](FS-init.md#23410-citation-direction)) reads ``No kind is declared yet, so no citation direction applies. Once kinds exist, the config's citation rules say which kinds each may cite; see <guide> for the levels and examples.``, `<guide>` being the citation-directions guide's URL, in place of the static climbing-rule sentence that names kinds the config does not have; the grounding sentence follows it as before. All three are substituted content, so they move no block version ([§FS-init.2.3.7](FS-init.md#237-the-block-version)), and `check` compares the citation-direction paragraph against the same render it always does ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check)).

###### 2.3.4.4.3 Row titles

A row's title is the one the config gives its kind — `[presentation.kinds.<KIND>] title` in a version-2 config, the `[[kinds]]` table's `title` in a version-1 one — whenever it gives one. An untitled row is titled by what it is. The complement's row ([§FS-init.2.3.4.4.1](FS-init.md#23441-rows-for-places)) reads `Every scanned file no other row claims`, because it is the one row that is not a declaration home: it declares no IDs, and holds every scanned file no other place claims ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)). Every other untitled row reads `Declaration`:

```markdown
- [FS](docs/spec): Declaration
- [build.gradle](build.gradle), [src/](src): Every scanned file no other row claims
```

The untitled complement row is the one `grund config migrate --to 2` gives a version-1 `[scan] include` that lists places outside every kind home, and that the migration's note announces ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)), since version 1 had nowhere to keep that row's title. The migration writes no title for it; the map says what the row is without one.

Both titles are wording the block renders, not defaults of the config. `grund config show` prints no title for either row, and a file `init` writes still carries no default title ([§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default)). They are substituted content, so they move no block version ([§FS-init.2.3.7](FS-init.md#237-the-block-version)); `init --check` reports a map whose untitled complement row still reads `Declaration` as stale, and `grund init` rewrites it.

The citation directions keep their own subject for the complement ([§FS-init.2.3.5.2](FS-init.md#2352-the-subject-names-its-unit)): an untitled one is `(**code**)`, with no scope, whatever the map calls its row. So no section `check` compares moves ([§FS-init.2.3.7.2](FS-init.md#2372-what-check-compares)), and the migration's guidance comparison ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)) renders the same section before and after.

##### 2.3.4.5 Declaration Forms

The block teaches that declarations are heading lines in markdown or supported language doc-comments, and that an inline source declaration can be represented by a one-line markdown stub in the configured kind home. It also teaches, rendered according to `[id] section_heading_levels`, that numbered headings inside a declaration are citable sections; depth-matching headings (`## 1. …`, `### 1.1 …`) are required in strict mode, warned on in warn mode, and recommended in loose mode. When the effective config enables named sections, the same block additionally teaches the explicit complete-path forms `## goals: Goals`, `### goals.performance: Performance`, and `### goals.3: Ordered child`, plus, since v17, that `name.number` is legal and in a Markdown declaration body so is `number.name`, a cited address with a name under a number being recorded in the section lock ([§FS-config.3.3.1](FS-config.md#331-named-components), [§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address)). Its sentence reads: "handles are letter-first lowercase names, `name.number` is legal, and in a Markdown declaration body so is `number.name`; a cited address with a name under a number is recorded in the section lock." Since v23, with v24 its rule-enabled form and v25/v26 where scoped guidance renders ([§FS-init.2.3.7.1](FS-init.md#2371-the-current-version)), it teaches both forms for a source doc-comment declaration body as well, the release that gives them a meaning there ([§FS-declarations.section.relative.where](FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190)): that sentence reads "handles are letter-first lowercase names, and `name.number` and `number.name` are both legal; a cited address with a name under a number is recorded in the section lock", and the relative-form sentence opens "In a declaration body, Markdown or doc-comment, a heading may instead write only the component it adds". An existing block of an earlier version taught both forms as Markdown-only; it is repaired by `grund init` and reports `agents-init` once until then ([§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)). An existing v16 block, or v17 where a rule kind is enabled, taught `number.name` as reserved; it is repaired by `grund init` and reports `agents-init` once until then ([§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)). Disabled repositories receive no named-heading guidance. Since v16 it also teaches the relative form beside the complete one: in a Markdown declaration body, write `.<component>` under the parent heading (`### .1: Username` under `## format: Formatting` where named sections are enabled, `### .1. Scope` under `## 3. Third` otherwise), and run `grund fmt --write` after citing one so the section lock records it ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds), [§FS-section-lock](FS-section-lock.md#fs-section-lock-a-cited-relative-section-keeps-its-text-or-check-says-where-the-text-went)); an existing v14 block, or v15 where a rule kind is enabled, is repaired by `grund init` and reports `agents-init` once until then ([§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)). Since v10 it also teaches which headings a declaration body may carry unmarked ([§FS-init.2.3.4.5.1](FS-init.md#23451-unmarked-headings)).

###### 2.3.4.5.1 Unmarked headings

In managed block v10 the block teaches that every non-declaration ATX heading inside a Markdown declaration body must carry a numbered or enabled named section path; file titles, body-closing headings, fenced examples, non-ATX text, and source doc-comments remain exempt, while bold labels remain the non-citable alternative ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth), [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)). Existing v9 blocks use the ordinary `grund init` in-place repair path ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)); `grund_config_version` remains 1 because no configuration meaning changes.

##### 2.3.4.6 Spec First

The rules tell agents to write or update the most-specific functional or architectural declaration before writing behavior or design code that implements it.

##### 2.3.4.7 Most-Specific Citations

The code citation guidance tells agents to cite the most-specific declaration the code or prose realizes: whole behavior on the function, class, or block doc-comment; narrower clauses or decisions inline where they are enforced.

##### 2.3.4.8 Refresh Before Editing

The rules tell agents to refresh the cited declaration with `grund <ID>` before editing code that already carries a `§<ID>` or `§<ID>.<section>` citation.

##### 2.3.4.9 Declaration Blast Radius

The rules tell agents to run `grund refs <ID> --summary` before changing, moving, or renaming a declaration, and to use the full `grund refs <ID>` output when exact citation sites are needed.

The generated guidance connects that discovery to the author's next actions:
edit the declaration and affected authored citation IDs, then run `grund fmt --write`
from the project's config root or workspace root to re-derive generated Markdown
link destinations and anchors when link emission applies
([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)) and maintain eligible section locks
([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)).
It directs the reader to verify with `grund check` and `grund fmt --check` and commit
the resulting lock changes. The workflow does not ask the reader to hand-edit locks
or imply that `fmt` replaces authored IDs, repairs arbitrary URLs, renames an
ID-keyed lock on a file move alone ([§FS-section-lock.file.shard](FS-section-lock.md#fileshard-one-file-per-cited-declaration)),
or approves a protected section move ([§FS-section-lock.write.never-a-move](FS-section-lock.md#writenever-a-move-fmt-never-approves-a-move)).

##### 2.3.4.10 Citation Direction

The rules teach the expected citation direction: specs cite goals, architecture cites specs, code cites the specs it implements, and executable tests or cases cite the behavior they verify. When the effective `grund.toml` declares `[citations]` ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules)), this guidance is the generated, config-derived Citation directions section ([§FS-init.2.3.5](FS-init.md#235-citation-directions)) rather than the static sentence above — so the rule an agent reads is the rule `grund check` enforces. A config without `[citations]` keeps the static sentence, which links to the durable canonical explanation at `https://github.com/agent-grounds/grund/blob/main/docs/user-facing/citation-directions.md` so a generated entrypoint remains usable outside the grund source checkout. Either way the paragraph carries the grounding sentence of [§FS-init.2.3.5](FS-init.md#235-citation-directions) when any row's effective `require_grounding` is on: whether a file must ground at all is not a direction rule, so it does not wait on `[citations]` being declared.

##### 2.3.4.11 Decisions

The rules tell agents that decisions must be cited from the functional-spec or architecture declaration they shaped, and that decision history is append-only: reversals are new decisions that supersede older ones rather than rewrites.

##### 2.3.4.12 Cross-Linking

The block tells agents to cross-link declarations only via IDs — a single short sentence in the citations narrative, not phrased as a rule. The `[fmt.cross_refs].enabled` mode ([§FS-init.2.3.3](FS-init.md#233-citation-form)) does not change this wording.

##### 2.3.4.13 Executable Proof

The rules tell agents that behavior is proven by executable tests or cases, and that disagreements between the spec and executable proof require fixing both in the same change.

##### 2.3.4.14 Final Check

The rules tell agents to run `grund check` before committing, because dangling citations are stop-the-line bugs whose findings name the file and line.

##### 2.3.4.15 Workspace Members

When — and only when — the init target lies in a workspace ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), the one [§FS-init.2.3.4.15.1](FS-init.md#234151-which-workspace) finds, the block emits a `### Workspace members` section as a sibling to the Project Map ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)). The section contains exactly two things: one discoverability line ([§FS-init.2.3.4.15.2](FS-init.md#234152-the-discoverability-line)) and one bullet per resolved foreign workspace project ([§FS-init.2.3.4.15.3](FS-init.md#234153-foreign-project-rows)). Each entrypoint omits its own project from that scope ([§FS-init.2.3.4.15.4](FS-init.md#234154-self-is-omitted)), so root-side and member-side lists differ by which alias is absent and by the remaining links' relative paths; the foreign aliases retain the same ordering and the discoverability line is identical.

If workspace expansion, alias validation, alias uniqueness, or nested-workspace validation would fail under the same rules as `grund check`, `init` suppresses the Workspace members section instead of emitting partial or ambiguous guidance. `grund check` remains the command that reports the configuration error. While an optional member is absent, an existing section is kept rather than rendered ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)).

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

Each foreign bullet renders the backticked alias as the *label* of a Markdown link, so the destination path appears exactly once — `` - [`api`](apps/api/AGENTS.md) `` — the same raw-readable `- [x](y): …` list grammar the Project Map ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)) uses; a raw-bytes reader still sees the path in the link destination, and token cost stays minimal ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)). When the foreign project has a `project_description` in its own config ([§FS-config.3](FS-config.md#3-keys-version-1), [§FS-workspace.3](FS-workspace.md#3-aliases)), the bullet appends `: <description>` (colon, space) after the link, before any trailing suffix; a project without one keeps the link-only bullet byte-identical, and `init` never derives a description it was not given ([§DF-workspace-member-descriptions](../decisions/functional/DF-workspace-member-descriptions.md#df-workspace-member-descriptions-member-side-project_description-for-workspace-member-lists)).

###### 2.3.4.15.6 The link target

When a foreign project's `AGENTS.md` exists on disk at the time `init` runs, the link target is `<member-root>/AGENTS.md`. When it does not, the link target is the member root directory (with a trailing `/`) and the bullet ends with the literal trailing suffix `*(not yet initialized)*` — so an agent reading the block never follows a 404 link, and the missing entrypoint is surfaced as actionable information (running `grund init` inside that directory is the next step). Any foreign uninitialized project — including the workspace root when listed from a member and its `AGENTS.md` is missing — keeps the suffix. Existence is checked after path canonicalization; symlinks count. Paths in link targets are emitted relative to the directory where the entrypoint being written lives, so a workspace-root `AGENTS.md` points at `apps/api/AGENTS.md` while the same foreign project emitted inside another member's `AGENTS.md` points at `../../apps/api/AGENTS.md`.

###### 2.3.4.15.7 What `init` does not do

`init` does **not** prompt for, infer, or configure workspace topology. It does not add a `[workspace]` block to a config that lacks one. It does not write or modify any file under a member directory other than the member it was invoked in — even when invoked at the workspace root, the Workspace members section is the only artifact `init` produces about sibling members; bootstrapping a member's own `AGENTS.md` remains a separate `init` invocation inside that member. Out of scope for v1: a reciprocal member→root pointer, and any collapse-to-alias-only rendering for very large workspaces.

###### 2.3.4.15.8 While an optional member is absent

In a run where an optional member is absent ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)), anywhere in the workspace, the section `init` would render is not the one a full checkout renders: it is the run whose workspace sections `check` leaves out ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout), its fourth case). So `init` does not render over a member list it cannot render whole:

- **An existing section is kept.** Where a root entrypoint's managed block already has a `### Workspace members` section, its bytes, from that heading to the next heading of the block, are kept exactly as they are, in whatever rendering wrote them, and every other section of the block is refreshed as [§FS-init.2.3.10.1](FS-init.md#23101-re-rendering-an-existing-block) says. Each entrypoint keeps its own section; nothing is copied from one entrypoint into another.
- **A new section lists what can be read.** Where there is no block, or the block has no such section, the section is rendered from the members the run can read ([§FS-init.2.3.4.15.3](FS-init.md#234153-foreign-project-rows)), so it does not list the absent one, and the full checkout's root `check` reports that block until `grund init` runs there.
- **Every mode decides the same way.** `--dry-run`, `--check` ([§FS-init.1.4](FS-init.md#14-why---check-exists)) and `--force` ([§FS-init.3.4](FS-init.md#34-with---force)) keep and render exactly what the write does, so `--check` reports `would-update` only for a difference outside a kept section, and a run whose only difference is a kept section reports `exists`.
- **The run says so, once per absent member.** One note ([§FS-init.2.2.1](FS-init.md#221-notes)) per absent member, ordered by namespace, names its entry and its namespace and then, for each root entrypoint the run reports, in report order, whether that entrypoint's section was kept or written new, the clauses joined by `; `:

  ```console
  note: optional workspace member `packages/secret` (namespace `secret`) is absent, so AGENTS.md keeps its `### Workspace members` section as it was; CLAUDE.md's new `### Workspace members` section does not list it
  ```

  Under `--dry-run` and `--check` the clauses are in the conditional: `would keep its … section as it is`, `… section would not list it`.

`check` there leaves both workspace sections out; `init` keeps only `### Workspace members`, because `### Project namespaces` reads discovery rather than members ([§FS-init.2.3.4.16](FS-init.md#23416-project-boundaries)) and renders the same in both checkouts. Nothing about the absent member is inferred ([§FS-init.2.3.4.15.7](FS-init.md#234157-what-init-does-not-do)), and an expansion error still suppresses the section as the lead says. No render changes, so no block version moves ([§FS-init.2.3.7](FS-init.md#237-the-block-version)). The member list is then refreshed only from a checkout that can see every member, which is where `check` judges it: a blind spot of `init --check`, declared by the note and bounded by that run ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)). It is what lets the blocks `init` writes in a partial checkout be the ones the full checkout's `check` accepts ([§FS-init.requirements.clean](FS-init.md#requirementsclean-what-init-writes-check-accepts--realized)). Decided in [§DF-init-keeps-the-member-list](../decisions/functional/DF-init-keeps-the-member-list.md#df-init-keeps-the-member-list-grund-init-keeps-a-blocks-member-list-while-an-optional-member-is-absent).

##### 2.3.4.16 Project Boundaries

The block emits `### Project namespaces` when — and only when — the init target lies in a workspace found by the claimed-chain discovery of [§FS-init.2.3.4.15.1](FS-init.md#234151-which-workspace) and [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces). A standalone target, including one below an ancestor workspace that does not claim it, gets neither this section nor Workspace members. Discovery and member expansion are separate facts: once a workspace is found, an expansion error or duplicate alias may suppress Workspace members under [§FS-init.2.3.4.15](FS-init.md#23415-workspace-members) but must retain Project namespaces. Nested and unclaimed-ancestor discovery semantics do not change.

For a workspace target, the section retains its existing wording and spacing byte for byte, including the configured marker in local and qualified citation examples. That frozen wording distinguishes project boundaries from documentation folders, identifies the current project as local, and gives the member-splitting recipe. It remains before Workspace members when that section is present. Every selected entrypoint uses the same discovery result; companion selection, `--name`, descriptions, `include_root`, and an empty or suppressed member list do not decide workspace presence. `init`, `--check`, `--dry-run`, and `--force` render the same condition under their existing ownership and preview rules. Setup instructions teach the complete split even for standalone repositories ([§FS-init.5.4](FS-init.md#54-splitting-off-a-member)); `init` itself does not configure topology or initialize siblings ([§FS-init.2.3.4.15.7](FS-init.md#234157-what-init-does-not-do)).

##### 2.3.4.17 Clickable Citations

The block always carries the repository-scoped sentence: `On repository web surfaces, link §<ID> to the PR branch in PR bodies, the reviewed commit in reviews, an exact commit for permalinks, and the default branch otherwise; fall back to plain when unsure.` ([§DF-neural-link-generation](../decisions/functional/DF-neural-link-generation.md#df-neural-link-generation-agents-compose-clickable-citation-links-themselves-grund-does-not-grow-a-link-command)). Every `§` in this section's sentences is the configured marker ([§FS-init.2.3.4.17.1](FS-init.md#234171-the-marker)).

When the effective `grund.toml` sets `[reference] conversation = "link"` ([§FS-config.3.1](FS-config.md#31-reference--citation-form), [§DF-repo-conversation-opinion](../decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)), the section additionally carries a config-derived local-conversation sentence in the form of the entrypoint's agent ([§FS-init.2.3.4.17.2](FS-init.md#234172-the-form-per-agent)), followed in both forms by the fixed deference clause `If a user-level grund block states a local-conversation rendering, follow that instead: that machine knows what its surface can open.` ([§FS-init.2.3.4.17.6](FS-init.md#234176-the-deference-clause)). When the key is absent, the sentence is absent, and local conversation rendering is governed solely by the user preference `grund integrations --write` installs in user-level agent instructions ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)). The full recipe, rationale, and test matrix stay in the cited decision; the repository block carries only the compact, forge-neutral convention.

###### 2.3.4.17.1 The marker

Every `§` shown in this section's sentences is the repository's configured `[reference] marker` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)), rendered like the marker anywhere else in the block ([§FS-init.2.3.4.2](FS-init.md#2342-citation-scheme)) — the wording is fixed, the marker is not. A section that hardcoded `§` in a repository configured with another marker would instruct agents to write a token that repository does not treat as a citation at all: `grund check` ignores it under `strict`, so the claim it grounds is silently never verified, while the surrounding block correctly teaches the real marker.

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

The section opens with one paragraph — the fixed legend ``must`/`never` are `grund check` errors; `should`/`avoid` are suggestions (`grund check --suggestions`).``, followed by the grounding sentence ([§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence)) where any row's effective `require_grounding` is on — then one bullet per citing kind that has any rule, in `[[kinds]]` order with the homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) last, then one closing line ([§FS-init.2.3.5.6](FS-init.md#2356-the-closing-line)). An unscanned kind ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) has no bullet at all: it can carry no `[citations.<kind>]` rule.

This layout is per destination. Under scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) the root block's section holds only the rows whose destination is the root; with none there it holds the legend and the closing line, the closing line stating the global default as before. Each scoped block lays out its own rows the same way ([§FS-init.2.3.5.12.1](FS-init.md#235121-what-a-scoped-block-holds-in-order)). Under `rules = "root"` and in every version-1 config there is one destination, the root, and the layout is the single one above ([§FS-init.2.3.5.11.5](FS-init.md#235115-rules--root-and-every-version-1-config-render-at-the-root)).

##### 2.3.5.2 The subject names its unit

**Every bullet names its unit**, because the units the levels are checked per are not interchangeable and one verb over three of them says nothing ([§FS-check.3.11](FS-check.md#311-missing-required-citation)):

| Citing kind | Subject |
|---|---|
| citable | `Each **DA** declaration` |
| non-citable, folder home | `Each file in **skills/**` |
| non-citable, folder home, grounding off | `Each file in **skills/** that cites anything` |
| non-citable, single-file home | `The file **docs/runbook.md**` |
| non-citable, single-file home, grounding off | `The file **docs/runbook.md**, if it cites anything,` |
| homeless, with a `title` | `Each source file outside the Project map (**code**: Gradle build and workflows) that cites anything` |
| homeless, without one | `Each source file outside the Project map (**code**) that cites anything` |

###### 2.3.5.2.1 Places, and what the homeless kind covers

A non-citable kind is named by its place, as in the Project map ([§FS-init.2.3.4.4.1](FS-init.md#23441-rows-for-places)), so the bullet reads as the instruction it is: files in this directory cite that. The homeless kind has no place, so it keeps its name and says what it covers. Its *that cites anything* is load-bearing: the obligation constrains what a source file cites and never whether it cites at all ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), so a util that cites nothing is not a unit. A non-citable home is the opposite case — a file there with no citation is the defect — so its subject is unnarrowed exactly when its row's grounding is on and `grund check` judges that defect ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)). Where the row grounds nothing, a file there that cites nothing is no unit either ([§FS-check.3.11](FS-check.md#311-missing-required-citation)), and the subject is narrowed as the homeless kind's is ([§FS-init.2.3.5.2.2](FS-init.md#23522-a-home-whose-row-grounds-nothing-says-so)).

###### 2.3.5.2.2 A home whose row grounds nothing says so

A non-citable home's row grounds nothing when no grounding rung of it, at any strength, reaches its files: its effective `require_grounding` is off — the row's own key or `[rules.citations.<row>.grounding]` ladder, else the global default and `--require-grounding` ([§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global), [§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). Its bullet then claims only what `grund check` enforces: the subject takes the grounding-off form of [§FS-init.2.3.5.2](FS-init.md#2352-the-subject-names-its-unit), and a second sentence says a file that cites nothing is not checked and names the one setting that makes it a finding. That setting is config grammar, which the section otherwise never renders ([§FS-init.2.3.5](FS-init.md#235-citation-directions)), because it is the one fact an agent cannot act without: the hole closes only in `grund.toml`. It is spelled in the live config's version — version 2 `` `[rules.citations.<row>.grounding] must = "file"` ``, version 1 `` `require_grounding = true` on its `[[kinds]]` row `` — so the row `e2e` at `tests/e2e/` with `must = ["FS"]` renders, under version 2:

```text
- Each file in **tests/e2e/** that cites anything must cite FS. A file there that cites nothing is not checked; `[rules.citations.e2e.grounding] must = "file"` makes it a finding.
```

A single-file home's second sentence opens `If it cites nothing, it is not checked;` and a several-file home's `One that cites nothing is not checked;`. With the row's grounding on, the bullet is the one unnarrowed sentence of [§FS-init.2.3.5.2](FS-init.md#2352-the-subject-names-its-unit) and the grounding sentence names the home ([§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence)). The homeless kind's bullet is the same either way: it is narrowed already.

##### 2.3.5.3 The clauses

The clauses follow the subject, joined by `; ` in the order `must`, `should`, `may`, `must-not`, `should-not`, then the per-kind default. A prohibition leads its bullet with the modal — `must not cite`, `should not cite`, the wording [§FS-check.3.12](FS-check.md#312-forbidden-citation) uses in its findings — and follows a preceding clause with the short `never cite` / `avoid citing` the legend names, because the subject is a noun phrase and a bullet opening on a bare `never cite` is not a sentence.

##### 2.3.5.4 Targets

Targets render as prose, never as rule grammar. Alternatives inside one entry join with "or", taking the Oxford comma from three on (`FS, AR, or RM`); conjunctive entries join with "and", and where there is more than one entry an entry that has alternatives of its own is parenthesised — `must = ["FS|GOAL", "AR"]` renders `(FS or GOAL) and AR`, which has one reading. A pinned alias renders exactly as spelled (`api/AR`), because that is how the citation is written; `*/AR`, which is rule grammar and never a citation ([§FS-config.3.9.3](FS-config.md#393-alias-matching)), renders as `AR in any project`.

##### 2.3.5.5 Defaults

Only `must-not` and `should-not` close anything: [§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence) makes a default of `must`, `should`, or `may` invent no obligation and forbid nothing. A per-kind `must-not` default folds into the permission it qualifies — `may cite only FS or GOAL` — when the `may` list is the whole of what the kind permits; with a `must` or `should` beside it the permitted set is wider than that list, so the bullet ends with `never cite anything else` instead. A per-kind `should-not` default ends its bullet with `avoid citing anything else`. A per-kind default that leaves its kind open says so, `may cite anything else`, only where the global default is closed and the kind is therefore a hole in it.

##### 2.3.5.6 The closing line

The closing line reports the global default alone, because a per-kind default is *listed above*: `Anything not listed above is allowed.` for an open one, including `must` and `should`; `Any citation not listed above is forbidden.` for `must-not`; `Any citation not listed above is discouraged.` for `should-not`. Either form is load-bearing — the first so an agent does not over-infer prohibitions from silence, the second so it does not miss a closed world.

##### 2.3.5.7 The grounding sentence

The grounding sentence is generated from each row's effective `require_grounding` — the row's own key, else `[reference] require_grounding`, because the row decides ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs), [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) — and the configured non-citable homes, and it renders whether or not `[citations]` is declared, because grounding is not a direction rule ([§FS-init.2.3.4.10](FS-init.md#23410-citation-direction)). It claims only what [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in) enforces, so it names a place exactly when that place's row has grounding on — a `[rules.citations.<row>.grounding]` ladder with a `must` rung included — and opens with the source-file clause exactly when the homeless kind's row has it on; naming the level per place ([§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) is not its work.

###### 2.3.5.7.1 Citing, not declaring

The sentence distinguishes citing from declaring: `Every source file must cite a declared ID or declare one inline`, extended with `; every file under skills/ and tests/e2e/ must cite one` for the scanned non-citable homes whose rows have it on, because a declaration in such a home is misplaced ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home)) and those files can therefore only cite. An unscanned home ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) is left out: nothing in it is scanned, so the rule never reaches it.

Under scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) each destination's sentence names only the rows that land there. The source-file clause belongs to the complement and renders at its destination. A non-citable home's clause renders at its row's destination, after the source-file clause where both land in one place and on its own as `Every file under tests/e2e/ and tests/integration/ must cite one.` where they do not, or where the homeless kind's row grounds nothing — so a grounding ladder on the one home `tests/e2e/` alone renders `Every file under tests/e2e/ must cite one.`

##### 2.3.5.8 The drift check

Because the section's content derives from config rather than the template alone, the block version ([§FS-init.2.3.7](FS-init.md#237-the-block-version)) is no longer sufficient to detect staleness: editing `[citations]` without re-running `grund init` would leave guidance that disagrees with the live rules under a current version number. `grund check` therefore re-renders the section from the live config and byte-compares it against the section in the block; a mismatch is an `agents-init` finding ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) telling the author to re-run `grund init`. Rendering determinism is what makes the comparison sound — the render *is* the hash.

The comparison is not limited to this section. Every line of the block is rendered from the template and the config, so `check` renders the whole root block as `init` writes it and compares it byte for byte, section by section: a config change that makes any line untrue — the citation scheme, the project map, the declaration forms, the inline style sentences — is reported until `grund init` runs, and the workspace sections are compared where [§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout) says. One render answers `init`, `init --check`, `check`, the editor's report and `config migrate`'s guidance gate.

Under scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) the comparison is per file. The root block is compared whole, as above, `### Declaration shapes` and `### Scoped guidance` among its sections. Each destination's `AGENTS.md` and each standalone companion beside it must carry the scoped block as rendered, byte for byte: a missing file or block is a missing block, differing bytes are a stale one, and each orphan the list on disk names ([§FS-init.2.3.5.13.1](FS-init.md#235131-the-list-on-disk-records-what-init-owns)) that still carries a managed block is a stale one too, its section named `scoped guidance`. Every one is `agents-init` ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)), and `grund init --check` reports the same files as `would-…` lines, because both read one rendering of what each file should carry.

##### 2.3.5.9 The block versions this section moved

The managed-block version was bumped to carry this config-derived content under [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations), and bumped again to **v8** when the rendering of [§FS-init.2.3.5.1](FS-init.md#2351-layout) to [§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence) replaced the flat one this section used to specify — which stated no unit, grouped no conjunction, rendered no grounding sentence, and leaked `*/AR` into prose ([§DF-directions-render](../decisions/functional/DF-directions-render.md#df-directions-render-the-citation-directions-wording-is-chosen-once-against-a-canonical-config)). The later v9 bump is solely the coordinate-size sweep in [§FS-init.2.3.4.3](FS-init.md#2343-cheap-grounding); it changes no citation-direction rendering. The v39/v40 and v41/v42 bump of [§FS-init.2.3.7.1](FS-init.md#2371-the-current-version) is this section's again: a non-citable home whose row grounds nothing narrows its bullet ([§FS-init.2.3.5.2.2](FS-init.md#23522-a-home-whose-row-grounds-nothing-says-so)), and the grounding sentence reads every row's effective grounding ([§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence)) — so a block rendered under the old wording names the versions it moved between rather than only drifting ([§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)).

##### 2.3.5.10 Chapter rules

When at least one kind enables `rules = true`, the block renders a
`### Chapter rules` section immediately after `### Citation directions`, with
the existing `must`/`should` legend and one bullet per rendered rule in
qualified rule-ID order. Each bullet contains the exact authored sentence
followed by its live rule citation. The section and its refusal/no-write
lifecycle are [§FS-rules.4](FS-rules.md#4-validation-lifecycle) and
[§FS-rules.9](FS-rules.md#9-managed-guidance-and-editor-parity)'s. `check`
re-renders and byte-compares it as config-derived content; a mismatch is
`agents-init`.

Under scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) a chapter rule renders in the block
of its subject kind's destination, and its live citation is relative to that
file. The rule that `init` followed by `fmt --write` is stable below holds per
file.

The live citation uses the configured canonical cross-reference representation
([§FS-fmt.6.2](FS-fmt.md#62-form)): with `[fmt.cross_refs] enabled = true`,
it is a Markdown link whose destination is relative to the actual entrypoint
and whose anchor follows the configured anchor profile; with cross-references
disabled, it stays bare. `init` and `check` render that same representation from
the loaded rule vocabulary. The exact authored sentence and qualified rule-ID
order are preserved in either form.

An `init` followed by `fmt --write` must leave one tree that both `fmt --check`
and `init --check` accept, without formatter suppression directives. A second
formatting pass and a repeated `init`/`fmt --write` cycle preserve the chapter-rule
section byte-for-byte, and `check --full` reports no chapter-rule `agents-init`
drift. This agreement does not weaken the byte comparison
([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)):
a changed authored rule sentence, citation destination, or cross-reference
configuration still makes a section with the old bytes stale.

A rule the run cannot verify from the scope it was given
([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)) is rendered here as authored and the block is written, so the
validation `init` performs before writing withholds the write for a genuinely
invalid rule and for nothing else. That is what keeps the bytes a member-scoped
run writes equal to the bytes a run holding the whole workspace would write for
the same file ([§FS-rules.9.1](FS-rules.md#91-one-tree-renders-one-block)). A path-scoped `check` likewise compares the
section against the render a whole-tree run compares against ([§FS-rules.9.1.1](FS-rules.md#911-a-path-scoped-check-compares-against-the-same-render)).

##### 2.3.5.11 Scoped guidance: a row's guidance lands above its places

In a version-2 config whose `[presentation] rules` is `home`, its default ([§FS-config-v2.presentation](FS-config-v2.md#presentation-the-presentation-concern)), each row's guidance is written into the managed block of the `AGENTS.md` in the row's **destination**, the deepest directory that contains every one of its places, rather than into the root entrypoint. A row's guidance is its citation-direction bullet, its clause of the grounding sentence ([§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence)), its kind's declaration shape ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)) and the chapter rules whose subject is its kind ([§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules)). An agent writing in a place then finds that kind's rules and shape in the directory it works in, before it writes ([§GOAL-agent-grounding.1](../goals.md#1-the-three-layers)), and reads that kind's rules rather than every kind's ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)). The decision, what it rejected, and which agents load such a file by themselves are [§DF-scoped-guidance](../decisions/functional/DF-scoped-guidance.md#df-scoped-guidance-a-version-2-rows-guidance-is-written-above-its-places-with-a-root-pointer-for-agents-that-do-not-load-nested-files).

###### 2.3.5.11.1 The deepest common directory

A folder place counts as itself and a file place as its parent directory, and the destination is the longest run of leading path components those directories share. `docs/spec/core` and `docs/spec/plugins` land in `docs/spec`; `docs/spec` alone lands in `docs/spec`; `docs/goals.md` lands in `docs`; `docs/spec/core` with `docs/goals.md` lands in `docs`. A file at the config root, or places that share no leading component, make the destination the root. The **root destination** is the root entrypoint's own block: a row whose destination is the root renders there exactly as [§FS-init.2.3.5.1](FS-init.md#2351-layout) lays it out.

###### 2.3.5.11.2 A citable kind with no place, and the complement

A citable kind with no place, declared only in doc-comments ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)), lands at the root. The complement `code` lands at the root when its extent is the config root, and at the deepest common directory of its places when it lists some ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)). A `scan = false` row renders no bullet ([§FS-init.2.3.5.1](FS-init.md#2351-layout)) and no grounding clause ([§FS-init.2.3.5.7.1](FS-init.md#23571-citing-not-declaring)), so it has no destination unless its kind has fields or chapter rules.

###### 2.3.5.11.3 Only a row that renders something has a destination

A row with no citation-direction bullet, no grounding clause, no field and no chapter rule has no destination and writes nothing anywhere. A version-2 config with no citation rule, no grounding, no field and no chapter rule therefore writes no scoped file, and its root block is the one [§FS-init.2.3.5.1](FS-init.md#2351-layout) lays out.

###### 2.3.5.11.4 A destination directory that does not exist renders at the root

When the destination directory does not exist, the row renders in the root block as though its destination were the root, and `init` creates no directory ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)). Whether it exists is read at the run, so creating the directory later moves the live render: `grund check` reports the root block stale and the scoped block missing ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check)), and one `grund init` moves the guidance.

###### 2.3.5.11.5 `rules = "root"`, and every version-1 config, render at the root

With `[presentation] rules = "root"`, and in every version-1 config, every row's destination is the root: no scoped file is written and no `### Scoped guidance` section renders. Such a config's root block is byte for byte the block it rendered before scoped guidance existed, at the version [§FS-init.2.3.7.1](FS-init.md#2371-the-current-version) selects for it, save that a version-2 config with fields also renders their shapes there ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)).

##### 2.3.5.12 The scoped block

###### 2.3.5.12.1 What a scoped block holds, in order

A scoped block is bounded by the delimiters and the version heading of [§FS-init.2.3.9](FS-init.md#239-delimiters), at the same `block vN` as the root block of the same run. A blank line follows the heading, then one line naming the scope, `` These rules cover the files under `<dir>/`; the rest of this project's grund guidance is in the root `AGENTS.md`. ``, then each of these sections that has content, in this order and separated by a blank line:

1. `### Citation directions`: the legend followed by the destination's grounding sentence ([§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence)) as one paragraph, a blank line, the bullets of the rows that land there in the order of [§FS-init.2.3.5.1](FS-init.md#2351-layout), and the closing line for the global default ([§FS-init.2.3.5.6](FS-init.md#2356-the-closing-line)). It renders when a bullet or a grounding clause lands there.
2. `### Declaration shapes` ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)), when a kind that lands there has fields.
3. `### Chapter rules`: the legend and the bullets whose subject kind lands there ([§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules)).
4. `### Scoped guidance` ([§FS-init.2.3.5.13](FS-init.md#23513-the--scoped-guidance-pointer)), in the complement's destination when that is not the root, so a source declaration's guidance names where its kind's shape lives.

###### 2.3.5.12.2 Declaration shapes

A kind's declaration shape is rendered from the record the checks read ([§FS-fields.record](FS-fields.md#record-the-record)), in the words of `grund schema`'s field lines ([§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node)), so the block, the view and the checks cannot describe two shapes. The section is `### Declaration shapes`, a blank line, the paragraph `` `grund check` holds each declaration of these kinds to these fields, as `grund schema` lists them: ``, a blank line, then one bullet `- **<KIND>**` per kind that lands at the destination and declares fields, in row order, each followed by one nested bullet per line [§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node) prints under that kind's node, in its order, with a single space where the view pads columns: `  - field terms must · first`, `  - closed must`. In the root block, under `rules = "root"` or for a kind whose destination is the root, the section renders after `### Citation directions` and before `### Chapter rules`.

###### 2.3.5.12.3 A scoped file `init` creates is the block alone

A scoped `AGENTS.md` that `init` creates holds the managed block and nothing else: no generated title, unlike the root file ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)), and the end delimiter's line ending is its last byte. That is what makes "nothing but the block is left" a byte comparison rather than a guess ([§FS-init.2.3.5.14.2](FS-init.md#235142-orphans-are-removed)).

##### 2.3.5.13 The `### Scoped guidance` pointer

When at least one row lands outside the root, the root block renders a `### Scoped guidance` section after the config-derived sections that precede it (`### Citation directions`, `### Declaration shapes`, `### Chapter rules`) and before `### Clickable citations`:

```markdown
### Scoped guidance

Some places' citation directions and declaration shapes are in the `AGENTS.md` of the directory above them. If your agent has not already loaded that file, read it before writing there:

- `docs/spec/AGENTS.md`: FS
- `tests/AGENTS.md`: tests/e2e/, tests/integration/
```

There is one line per destination, ordered by the first row that lands there. Each names its rows as a bullet's subject names them ([§FS-init.2.3.5.2](FS-init.md#2352-the-subject-names-its-unit)): a citable kind by its name, a non-citable row by its places with a trailing `/` for a folder, the complement as `code`, joined by `, ` in row order. The pointer is the fallback the decision approves for an agent that does not load a nested file by itself ([§DF-scoped-guidance.loading](../decisions/functional/DF-scoped-guidance.md#loading-which-agents-load-a-scoped-file-by-themselves)). A config that renders no scoped file renders no such section.

###### 2.3.5.13.1 The list on disk records what `init` owns

The `### Scoped guidance` list of the root block **on disk** is the record of the scoped files `init` wrote, so neither `init` nor `check` walks the tree for managed blocks. `init` reads it before it rewrites the root block, and `check` reads it beside the live render: a path the list on disk names that is not a destination of the live render is an **orphan**. Only a line `` - `<path>`: … `` inside that section of a delimited root block names a path; a path that is absolute, contains `..`, or does not end in `/AGENTS.md` is ignored. A scoped block the list does not name is not `init`'s and is never touched.

##### 2.3.5.14 Writing scoped files

###### 2.3.5.14.1 Create or update

For each destination other than the root, `init` writes `<dir>/AGENTS.md` as it writes an entrypoint ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)): a missing file is created holding the block alone ([§FS-init.2.3.5.12.3](FS-init.md#235123-a-scoped-file-init-creates-is-the-block-alone)), an existing file without a block has it appended after its content, and an existing block is re-rendered in place, with every byte around it preserved ([§FS-init.3.2](FS-init.md#32-user-authored-content-in-agent-entrypoints-is-preserved)) and a byte-identical block left untouched and reported `exists `. The read-before-write rules hold for these files too: a newer block version ([§FS-init.2.3.10.2](FS-init.md#23102-a-newer-block-version)) or malformed delimiters ([§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)) in a scoped file exit `2` and write nothing anywhere ([§FS-init.1.2.6](FS-init.md#126-entrypoint-blocks-are-read-before-any-is-written)), and so does a scoped `AGENTS.md` that is a symlink, which `init` never writes through. `--force` never replaces a scoped file whole ([§FS-init.3.5](FS-init.md#35-what---force-does-not-replace)).

###### 2.3.5.14.2 Orphans are removed

For each orphan ([§FS-init.2.3.5.13.1](FS-init.md#235131-the-list-on-disk-records-what-init-owns)) whose file carries a managed block, `init` removes the block. It deletes the file only when the managed block was the whole file, byte for byte. Otherwise it cuts the region from the begin delimiter through the end delimiter's line ending and leaves every other byte as it was, handwritten text before and after the block included. An orphan whose file is gone, or carries no block, is left alone and not reported.

###### 2.3.5.14.3 Companions in a scoped directory

Where the root `CLAUDE.md` or `GEMINI.md` is a symlink to `AGENTS.md` ([§FS-init.2.3.12.2](FS-init.md#23122-a-companion-symlinked-to-agentsmd)), each destination gets the same name as a symlink to its own `AGENTS.md` when that name is free there; an existing symlink to `AGENTS.md` is left as it is, and an existing standalone file has the block appended or re-rendered as a companion's is ([§FS-init.3.5](FS-init.md#35-what---force-does-not-replace)). Those are the only companions in a live destination that `init` writes and `check` compares: a symlink there to anything else, and a standalone `CLAUDE.md` or `GEMINI.md` whose root name is not such a symlink, are left as they are and expected nothing of. In an orphan's directory, the `AGENTS.md` and each standalone `CLAUDE.md` or `GEMINI.md` have their block removed under [§FS-init.2.3.5.14.2](FS-init.md#235142-orphans-are-removed), each on its own account, whichever root name is mirrored and whether or not the `AGENTS.md` is still there or still carries a block; those are the files there that `init` cuts and `check` compares. When the `AGENTS.md` is deleted, the symlinks to it in its directory are deleted with it. No other symlink there is touched, an `AGENTS.md` that is itself a symlink included: `init` never writes through one, so `check` expects nothing of it. The root-only entrypoints are never mirrored: `AGENTS.override.md`, `.claude/CLAUDE.md`, `.pi/AGENTS.md`, `.github/copilot-instructions.md`, `.cursor/rules/grund.mdc`, `.cursorrules`, `.windsurfrules` and `.rules`.

###### 2.3.5.14.4 The report names every scoped file and link it touched

`init` deletes files now, so its report ([§FS-init.2.2](FS-init.md#22-stdout--stderr)) names each scoped file and companion symlink it creates, updates or removes. After the root entrypoints' lines and before the config's, it prints one line per scoped file in pointer order, each followed by its companions, then one line per orphan in the order the list on disk names them, each followed by its companions:

- `wrote <dir>/AGENTS.md` for a created file, `appended ` and `updated ` as for any entrypoint, `updated ` also for a kept file whose orphaned block was cut, and `exists ` for a current one;
- `linked <dir>/CLAUDE.md -> AGENTS.md` for a created companion symlink;
- `removed <path>` for a deleted scoped file or companion symlink.

An existing companion symlink that the run leaves as it is prints nothing, as at the root. Under `--dry-run` and `--check` the new prefixes read `would-link ` and `would-remove `, and both earn `--check`'s `1` ([§FS-init.4.1](FS-init.md#41-what-earns-the-1)).

##### 2.3.5.15 A generated scoped entrypoint is not a unit of its place

A scoped `AGENTS.md` that is a destination of the live render, or that the list on disk names, and a companion beside it, are not units of the place they sit in: they draw no grounding, citation-direction, index or misplaced-declaration finding as a file of that row ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in), [§FS-check.3.11](FS-check.md#311-missing-required-citation)). Their citations are still resolved, so a chapter rule's live citation in a scoped block is still checked. A row that lists the file in its `files` claims it, and it is then that row's unit like any other file, while `init` still writes its block ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)).

#### 2.3.6 Clickable citations

The managed block renders a `### Clickable citations` section carrying the content sections in [§FS-init.2.3.4.17](FS-init.md#23417-clickable-citations): the fixed repository-web sentence always, plus the local-conversation sentence when `[reference] conversation = "link"` is set. The section is deterministic from the effective config and the entrypoint's own agent ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)): byte-identical in every project with the same key state, for the same entrypoint file. The per-agent split of [§FS-init.2.3.4.17.2](FS-init.md#234172-the-form-per-agent) is the one axis on which two entrypoints in the *same* repository differ, and it is a pure function of the target path, so it is as reproducible as the rest.

##### 2.3.6.1 The drift check

Because the local-conversation sentence derives from config, the section joins the citation-directions drift check ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check)): `grund check` re-renders it from the live config and byte-compares; a mismatch — including flipping the `conversation` key without re-running `grund init` — is an `agents-init` finding ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)).

##### 2.3.6.2 The block versions this section moved

Adding the fixed sentence bumped the managed-block version to **v4**; adding the config-derived local-conversation sentence bumped it to **v5**, replacing that sentence with the gated link form bumped it to **v6**, and naming the bare `grund.toml` in the namespace rule's "give that project its own config" instruction ([§FS-config.1](FS-config.md#1-file-location-and-discovery), [§DF-config-file-location.2.3](../decisions/functional/DF-config-file-location.md#23-grund-init-writes-the-bare-grundtoml)) bumped it to **v7**, all carried under [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations); an older supported block is reported outdated by `grund check` until `grund init` re-renders it.

#### 2.3.7 The block version

The canonical text for a given block version `vN` is embedded in the `grund` binary; its one copy in the source tree is `crates/grund-core/assets/templates/AGENTS.md`, which the binary embeds. The `vN` in the block's version heading ([§FS-init.2.3.9](FS-init.md#239-delimiters)) is what versions it under [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations), so a change to the block is itself a block-version bump, carried by that mechanism, not a silent rewrite. What `vN` versions is every byte the block renders: its fixed text and the lines substituted in ([§FS-init.2.3.8](FS-init.md#238-substituted-content)), the inline citation style sentences among them ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)). The rule is about grund, not about a repository: a change **in grund** to what the block renders **for a given config and workspace** moves the version, because `check` compares every line ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)) and a mismatch under an unchanged version would name no change. A repository editing its config, migrating it, or adding a workspace member changes what the block renders without any new version: `check` reports that block stale and `grund init` refreshes it. Changes to the render that land between two releases share one bump, as the whole-block comparison shares the v39–v42 pairs below. `vN` is the revision of the block's text and nothing else: it is not a grund release, the two numbers move independently, and a grund release is read from `grund -V` alone. The heading says so by labelling it `block vN` ([§FS-init.2.3.9.3](FS-init.md#2393-the-headings-two-spellings)).

##### 2.3.7.1 The current version

Narrowing the bullet of a non-citable home whose row grounds nothing ([§FS-init.2.3.5.2.2](FS-init.md#23522-a-home-whose-row-grounds-nothing-says-so)) and naming each row's effective grounding in the grounding sentence ([§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence)) change byte-compared block sections, so both current pairs advance: v35/v36 becomes **v39/v40** and v37/v38 becomes **v41/v42**. The maximum supported version becomes v42; each config keeps its pair selection, four higher. An earlier block is outdated and one `grund init` re-renders it, preserving unmanaged content; the release note of [§DF-directions-render.release-note](../decisions/functional/DF-directions-render.md#release-note-release-note) names these pairs and that remedy ([§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)). v39–v42 are also the first blocks `check` compares whole ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)); that change shares these pairs rather than adding one, since no release wrote them before it, and the release note of [§DF-check-compares-the-whole-block.release-note](../decisions/functional/DF-check-compares-the-whole-block.md#release-note-release-note) names them too. The paragraphs below record the preceding bumps.

Teaching the declaration-maintenance workflow ([§FS-init.2.3.4.9](FS-init.md#2349-declaration-blast-radius))
changes the fixed canonical text and advances both current pairs: v31/v32 becomes
**v35/v36** and v33/v34 becomes **v37/v38**. The maximum supported version becomes
v38; each config keeps its pair selection, four higher. An earlier block is
outdated and one `grund init` re-renders it, preserving unmanaged content. The
decision's release note names these pairs and the remedy
([§DF-fmt-maintenance-guidance.release-note](../decisions/functional/DF-fmt-maintenance-guidance.md#release-note-release-note),
[§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)).
The rest of this section records the preceding bumps.

Labelling the version heading `block vN` ([§FS-init.2.3.9.3](FS-init.md#2393-the-headings-two-spellings)) changes the block's fixed
text, so both current pairs advance again: v27/v28 becomes **v31/v32** and v29/v30 becomes
**v33/v34**, the maximum supported version becomes v34, and each config keeps the pair it
selected, four higher. Every earlier block is outdated and one `grund init` re-renders it;
the release note of [§DF-managed-block-revision-label](../decisions/functional/DF-managed-block-revision-label.md#df-managed-block-revision-label-the-managed-block-heading-labels-its-number-as-the-blocks-revision) names the pairs, that remedy, and what the new spelling breaks
([§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)). The rest of this section describes the pairs as they were
before that bump.

Teaching coordinate globs in place of the `jq` pipe ([§FS-init.2.3.4.3.1](FS-init.md#23431-structural-queries)) advances both
current pairs again: v23/v24 becomes **v27/v28** and v25/v26 becomes **v29/v30**, so the
maximum supported version becomes v30 and each config keeps the pair it selected, four
higher. Every earlier block is outdated and one `grund init` re-renders it; the release note
of [§DF-coordinate-globs](../decisions/functional/DF-coordinate-globs.md#df-coordinate-globs-coordinates-are-selected-by-a-glob-of-their-own-and-a-brace-before-a-separator-opens-a-pattern) names the pairs and that remedy
([§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)). The rest of this section describes the pairs as they
were before that bump.

Teaching the doc-comment section forms ([§FS-init.2.3.4.5](FS-init.md#2345-declaration-forms)) in 0.19.0
advances both current pairs at once: v19/v20 becomes **v23/v24** and v21/v22 becomes
**v25/v26**, so the maximum supported version becomes v26 and each config keeps the
pair it selected, four higher. Every block of an earlier version is outdated as below
and one `grund init` re-renders it; the release note of
[§DF-one-parse-lands-in-0-19](../decisions/functional/DF-one-parse-lands-in-0-19.md#df-one-parse-lands-in-0-19-every-command-answers-from-one-parse-in-0190-and-the-managed-block-teaches-doc-comment-sections) names the pairs and that remedy
([§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)). The rest of this section describes the pairs
as they were before that bump.

Scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)) adds a second current pair, **v21/v22**,
which the config selects: a config that renders a scoped file or a declaration
shape ([§FS-init.2.3.5.12.2](FS-init.md#235122-declaration-shapes)) renders v21, or v22 with a rule kind. Every other
config keeps the v19/v20 pair below byte for byte: every version-1 config, a
version-2 config with `rules = "root"` and no field, and one that renders no
scoped file and no shape. The implementation-time baseline is `main` at
`aa4484657d` (2026-10-14), whose pair is v19/v20, and the maximum supported
version becomes v22. A block older than the version the live config selects is
outdated as below. A v21 or v22 block where the live config selects v19 or v20
is stale instead, its `### Scoped guidance` or `### Declaration shapes` section
differing from the render, and one `grund init` re-renders it either way. One
universal bump was rejected because it changes version-1 bytes when version-1
content does not change ([§DF-scoped-guidance](../decisions/functional/DF-scoped-guidance.md#df-scoped-guidance-a-version-2-rows-guidance-is-written-above-its-places-with-a-root-pointer-for-agents-that-do-not-load-nested-files)).

The workspace-only Project namespaces change ([§FS-init.2.3.4.16](FS-init.md#23416-project-boundaries)) advances the
current base and rule-enabled versions as one pair, keeping rule-enabled at base
plus one. The implementation-time `main` baseline is **v17/v18** at
`23010de6dfbfd37864a6c9cef753041594219920` (2026-10-14); the current pair is
**v19/v20**: v19 without a rule kind, v20 with one.
The maximum supported version, checker selector, template heading and rule-heading
promotion must agree on that pair. All earlier blocks, including workspace blocks,
are reported outdated by `grund check` and `grund init --check`; one `grund init`
re-renders them, preserving unmanaged content. The decision's release note must
name the actual before/after pair and this remedy ([§DF-project-namespaces-in-workspaces.release-note](../decisions/functional/DF-project-namespaces-in-workspaces.md#release-note-release-note),
[§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations)).

The preceding schema for a repository without a rule kind is **v17**; v17 teaches
that a name may follow a number in a Markdown declaration body and that a cited
address with one is locked ([§FS-init.2.3.4.5](FS-init.md#2345-declaration-forms)). A rule-enabled repository uses
**v18** because [§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules) adds byte-compared content. Removing the opt-in
returns to byte-identical v17 output rather than making v18 universal —
rule-enabled stays base plus one. v16, with v17 its rule-enabled form, added the
relative section heading and its section lock to the declaration forms of
[§FS-init.2.3.4.5](FS-init.md#2345-declaration-forms). v14, with v15 its rule-enabled form, added the structural query of
[§FS-init.2.3.4.3.1](FS-init.md#23431-structural-queries) to the cheap-read ladder. v12, with
v13 its rule-enabled form, stopped writing a bare ID-shaped illustration into
the block's own prose — the escaped example teaches the shape once and the two declaration lines use
the `<ID>` placeholder — so the scaffold lands clean under `[reference] strict = false`
too ([§FS-check.1.1.9](FS-check.md#119-an-id-in-an-escape-position)), and every earlier block is repaired by the same
one-command `grund init` re-render ([§FS-init.2.3.10.1](FS-init.md#23101-re-rendering-an-existing-block)). The v11 history is that same chapter-rules
section over a v10 base, v10 the in-body Markdown heading policy of
[§FS-init.2.3.4.5.1](FS-init.md#23451-unmarked-headings), and v9 the coordinate-size sweep added in
[§FS-init.2.3.4.3](FS-init.md#2343-cheap-grounding).


##### 2.3.7.2 What `check` compares

`grund check`'s agent-entrypoint validation ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) checks the delimiter lines and the version, then compares the whole block against `init`'s render, section by section ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check), [§FS-init.2.3.6.1](FS-init.md#2361-the-drift-check)), the two workspace sections only where [§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout) says. `grund config migrate` is narrower on purpose: its guidance gate refuses only a change to the block version or to one of the five generated sections, and passes the rest ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)).

#### 2.3.8 Substituted content

Several things in the generated entrypoint are *substituted in* rather than fixed for its `vN`, so the file describes the repo it is in. The generated project name is the explicit `--name`, otherwise the `project_name` from a config at either supported location in the target, otherwise the resolved target basename. That name is interpolated into the scaffolding H1 emitted above the block whenever canonical `AGENTS.md` is created or forcibly regenerated ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)); an ordinary update still preserves the unmanaged H1. The block's **effective ID grammar and artifact map** come from the `grund.toml` `init` leaves governing the target (an existing config in the target, or the defaults `init` is about to write). Neither the generated name nor the effective configuration is inherited from an ancestor when the target has no config.

##### 2.3.8.1 What the config fills in

From that config the block fills in the ID shape (`<KIND>-<NNN>-<slug>`, `<KIND>-<slug>`, …, derived from `[id].format`), one worked example citation, the `[id].section_separator`, the marker and `$$`-trigger from `[reference]`, the `KIND ∈ {…}` set from `[[kinds]]` (``no `KIND` is declared yet`` where there is none, [§FS-init.2.3.4.4.2](FS-init.md#23442-no-kinds-yet)), a raw-readable link list of each kind's configured declaration home and title ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)), a config-version-aware sentence on whether bare ID-shaped tokens count as citations, and the inline citation style sentences defined by [§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering).

For version 1, the bare-token explanation retains its existing wording for both
`[reference] strict = true` and `strict = false`, including the latter's advice
to remove the compatibility override or set strict back to `true`. For version
2, including a config produced by migration, the explanation says that bare
ID-shaped tokens are ignored, only citations prefixed with the configured
marker are recognized, and there is no opt-out
([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). It neither
claims that `[reference] strict` is set nor advises changing it: that setting
does not exist in version 2
([§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed)).

##### 2.3.8.2 The worked example is escaped

The worked example citation is written in the escaped illustration form — `<§>` around the configured marker, per [§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves) — never as a live citation: the example ID is deliberately not a real declaration in the host repo, so a live marker would make the freshly generated block fail the host repo's own `grund check` ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)) as a dangling citation wherever the entrypoint falls inside the scan scope.

##### 2.3.8.3 A config that fails to load

The generator–checker symmetry of [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints) extends to reading the config: an existing `grund.toml` that fails to load is an error (`exit 2`, the message and location `grund check` gives for the same file), never a silent fall back to defaults. The block is rendered *from* that config, so substituting defaults would write agent instructions describing a repository the user does not have — an unparseable `[reference] conversation`, `marker`, or kind set would drop exactly the guidance it selects while `init` reported success — and the two commands would disagree about a file both of them read.

#### 2.3.9 Delimiters

The managed block is bounded by explicit standard `BEGIN` / `END` HTML-comment delimiters, with an H2 heading just inside the opening delimiter carrying the block version, labelled as one:

```markdown
<!-- BEGIN GRUND MANAGED BLOCK -->
## Grounding with grund (block vN)
…
<!-- END GRUND MANAGED BLOCK -->
```

The integer after `block v` is the managed-block version ([§FS-init.2.3.7](FS-init.md#237-the-block-version)): the revision of the block's text, never a grund release. This heading is the block's **version heading**. The `<!-- BEGIN GRUND MANAGED BLOCK -->` line is the block's begin delimiter and `<!-- END GRUND MANAGED BLOCK -->` is its end delimiter; both delimiter lines belong to the managed region, so everything between them — delimiters included — is `init`'s to rewrite and nothing outside them is.

##### 2.3.9.1 Why the conventional shape

The delimiters are deliberately the conventional managed-region shape rather than a grund-specific arrow syntax, so humans and agents skimming the file recognize the ownership boundary without knowing grund ([§DF-managed-block-delimiters](../decisions/functional/DF-managed-block-delimiters.md#df-managed-block-delimiters-standard-beginend-delimiters-for-the-managed-agent-instructions-block)).

##### 2.3.9.2 Legacy blocks

Blocks at v3 and earlier predate the delimiters (**legacy blocks**): there the H2 heading itself is the begin delimiter and the block runs until the next H1 or H2 heading, or end of file. Both `init` and `check` continue to recognize the legacy form; on its next write `init` migrates a recognized legacy block to the delimited form in place (reported `updated `), preserving the block's position and every byte outside the block. Only the unlabelled spelling opens a legacy block: no block without delimiters was ever written with `block vN`, so a `## Grounding with grund (block vN)` heading outside the delimiters opens nothing, and the file has no block ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)).

##### 2.3.9.3 The heading's two spellings

`init` writes the version heading as `## Grounding with grund (block vN)` in every block it renders: the root block, a standalone companion's and a scoped file's ([§FS-init.2.3.5.12.1](FS-init.md#235121-what-a-scoped-block-holds-in-order)). Blocks up to v30 spell it `## Grounding with grund (vN)`. Between the delimiters both spellings are the version heading and carry the same version, so `init` and `check` read a block in either spelling wherever this specification names the version heading: its version is compared, reported outdated, or refused as newer the same way ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block), [§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). The spelling is part of the fixed text a version renders, so `init`'s re-render writes the labelled one in place of the other. The delimiters do not change: a reader that finds the block by them keeps finding it.

#### 2.3.10 Writing the block

A fresh `AGENTS.md` consists of the block preceded by a one-line scaffolding H1 (`# {NAME} — agent instructions`); the H1 is *unmanaged* on the update path, so `init` without `--force` rewrites the block but leaves the title alone, while `--force` rewrites the whole canonical file, H1 included ([§FS-init.3.4](FS-init.md#34-with---force)). A freshly-created companion entrypoint consists of just the managed block, with no extra H1, so the agent-specific file remains a thin companion of the canonical workflow. If an agent entrypoint already exists and contains no managed block, `init` appends the current block after the existing content (no H1 is inserted — the host file owns its title).

##### 2.3.10.1 Re-rendering an existing block

If the file already contains any supported block version, including the current one, in either spelling of its version heading ([§FS-init.2.3.9.3](FS-init.md#2393-the-headings-two-spellings)), `init` re-renders the block from the current effective `grund.toml`, compares it to the bytes of the managed region on disk ([§FS-init.2.3.9](FS-init.md#239-delimiters), delimited or legacy), and — when they differ — replaces only those bytes, leaving all content before and after the block untouched (reported `updated `, [§FS-init.2.3.1](FS-init.md#231-position-invariants)). Same-version template and config changes therefore propagate on the next `grund init` without requiring `--force`. When the re-render is byte-identical to what is on disk, `init` writes nothing and reports the file with `exists ` ([§FS-init.2.2](FS-init.md#22-stdout--stderr)) — re-running `grund init` on an already-current repo is a no-op on every file. A block whose heading is the unlabelled spelling is therefore `updated ` once, to the labelled one, and `exists ` after that. While an optional member is absent, the re-render keeps the block's `### Workspace members` section as it is on disk ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)).

##### 2.3.10.2 A newer block version

If an agent entrypoint contains a newer block version than the running binary supports, in either spelling (`(block v99)` as `(v99)`), `init` exits 2 and writes nothing: neither that file nor any other entrypoint, `grund.toml`, nor the starter files `--scaffold` lays are touched, because the version is read before the first write ([§FS-init.1.2.6](FS-init.md#126-entrypoint-blocks-are-read-before-any-is-written)).

#### 2.3.11 Malformed delimiters

A file whose delimiters are present but broken is **malformed**: a `BEGIN` delimiter with no `END` after it, an `END` with no `BEGIN` before it, more than one `BEGIN`, or a delimited region that contains no version heading in either spelling ([§FS-init.2.3.9.3](FS-init.md#2393-the-headings-two-spellings)). `init` refuses to touch a malformed file — it exits 2 with a message naming the file and the specific defect, and rewrites nothing, in that file or any other the run planned ([§FS-init.1.2.6](FS-init.md#126-entrypoint-blocks-are-read-before-any-is-written)) — because any splice against broken delimiters risks eating user content; `grund check` reports the same defect as an error ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). Fixing the delimiter pair by hand (or deleting the remnants of the block) and re-running `grund init` is the recovery path.

#### 2.3.12 Supported agents and their entrypoints

`AGENTS.md` is the canonical fallback entrypoint. The supported-agent set is fixed and is the superset used by every agent-facing feature: AGENTS-compatible agents (including Codex), Claude Code, Gemini, Pi, GitHub Copilot, Cursor, Windsurf, and Zed. The built-in companion set covers their common root or repository instruction files: Codex override instructions (`AGENTS.override.md`), Claude Code (`CLAUDE.md`, `.claude/CLAUDE.md`), Gemini (`GEMINI.md`), Pi (`.pi/AGENTS.md`), GitHub Copilot (`.github/copilot-instructions.md`), Cursor (`.cursor/rules/grund.mdc`, plus the legacy `.cursorrules`), Windsurf (`.windsurfrules`), and Zed (`.rules`). Under scoped guidance a destination directory carries `AGENTS.md` and mirrors only the `CLAUDE.md` and `GEMINI.md` symlinks ([§FS-init.2.3.5.14.3](FS-init.md#235143-companions-in-a-scoped-directory)); which agents load such a nested file by themselves, and what the others are told, is [§DF-scoped-guidance.loading](../decisions/functional/DF-scoped-guidance.md#loading-which-agents-load-a-scoped-file-by-themselves).

##### 2.3.12.1 Which companions are created

Companion entrypoints are mostly discovery-based: `init` updates them if the repo already has them, and creates the agent-directory-triggered companions for Claude, Gemini, Pi, Cursor, or Zed — one per agent ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)) — only when the matching agent directory already shows that tool is in use. For the reasons [§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection) gives, `.github/copilot-instructions.md` is created only by `--copilot`, `.windsurfrules` only by `--windsurf`, and `.rules` only where `.zed/` exists or `--zed` is passed ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)).

##### 2.3.12.2 A companion symlinked to `AGENTS.md`

When a companion path is a symlink to `AGENTS.md`, `init` does not touch it separately. If that companion is explicitly requested, the request selects canonical `AGENTS.md` as the update target; `grund check` always treats the canonical `AGENTS.md` block as sufficient for that symlink ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)).

### 2.4 Generated `grund.toml`

`init` writes `<path>/grund.toml` — the bare, root-visible form — and **only when the target has no config at all**, in either discovery location ([§FS-init.2.4.1](FS-init.md#241-both-discovery-locations-are-probed)). An existing config is never overwritten, not even with `--force` ([§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)). The file it does write is the template's config ([§FS-init.1.6](FS-init.md#16-templates)): a version-2 file ([§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults)) that holds the project's own choices and nothing `grund` would assume anyway ([§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default)), while `grund config show` prints every effective setting ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)). [§FS-init.2.4.4](FS-init.md#244-no-section-switch-is-written) to [§FS-init.2.4.8](FS-init.md#248-values-are-not-enabled) and [§FS-init.2.4.10](FS-init.md#2410-req-is-declared-in-docsrequirements) say how particular settings are written or left out, and [§FS-init.2.4.9](FS-init.md#249-the-none-templates-config) what the `none` template writes instead. The starter is written fresh rather than migrated from the version-1 file earlier releases wrote, which `grund config migrate` would refuse as it stands ([§DF-v2-onboarding](../decisions/functional/DF-v2-onboarding.md#df-v2-onboarding-grund-init-writes-a-minimal-version-2-file-and-grund-config-show-does-the-teaching)).

#### 2.4.1 Both discovery locations are probed

Both discovery locations of [§FS-config.1](FS-config.md#1-file-location-and-discovery) are probed, so a repository already configured under `.agents/grund.toml` is reported `exists .agents/grund.toml` and never grows the redundant pair [§FS-check.4.3](FS-check.md#43-redundant-config-pair) warns about. That probe matters more under the [§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both) tie-break than a same-name check would: writing a bare `grund.toml` beside an existing `.agents/` config would *take over* the repository's grammar, which is exactly what `init` must never do to a config it did not write. Which form a project uses stays its own choice; the bare one is what `init` recommends, by generating it ([§DF-config-file-location.2.3](../decisions/functional/DF-config-file-location.md#23-grund-init-writes-the-bare-grundtoml)).

#### 2.4.2 An existing config is never overwritten

An existing config is the repo's configuration — the one surface a project customizes ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable)) — and `init` never overwrites it, not even with `--force`: an existing config is reported as `exists` and left byte-for-byte unchanged ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)). `--force` resets the things `init` owns end to end — the canonical `AGENTS.md` and the starter files `--scaffold` lays — not the user's settings; a customized config that `init --force` clobbered would be a footgun.

#### 2.4.3 Every written key differs from its default

When the file is absent and `init` does write it, the file is version 2 ([§FS-config.5](FS-config.md#5-schema-versioning)), and every key it writes either differs from its version-2 default ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)) or has none. v2 has no built-in kinds and no default titles, so a row, its places and its title are written because without them the file declares nothing ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)). The file is therefore minimal: taking any written key or table out of it changes the effective configuration. It is an override surface, not a teaching surface; `grund config show` is the teaching surface, and prints every setting the file leaves at its default, in the file's own version ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)).

What is not written follows from that. No `[schema.sources]` and no `[schema.kinds.code]` is written, so a new project scans v2's default extent: the rows' places and the config root, less what `.gitignore` excludes ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)). No `[schema] id_format` is written, so a new project takes v2's slug-only format `{kind}-{slug}`: `grund id FS "Login"` allocates `FS-login`, and the starter files follow that format ([§FS-init.2.1.3](FS-init.md#213-the-software-development-starter-files)). No `[rules]` table is written, so no citation direction is enforced until the project adopts one.

The `software-development` file is exactly these bytes, with `{NAME}` the project name of [§FS-init.2.4.7](FS-init.md#247-project_name-and-presentation-description):

```toml
# Only what differs from grund's defaults; `grund config show` prints every setting.
grund_config_version = 2
project_name = "{NAME}"

[schema.kinds.GRUND]
files = ["docs/grund.md"]
[schema.kinds.GOAL]
files = ["docs/goals.md"]
[schema.kinds.FS]
folders = ["docs/functional-spec"]
[schema.kinds.REQ]
folders = ["docs/requirements"]
[schema.kinds.AR]
folders = ["docs/architecture"]
[schema.kinds.DF]
folders = ["docs/decisions/functional"]
[schema.kinds.DA]
folders = ["docs/decisions/architectural"]
[schema.kinds.e2e]
citable = false
folders = ["tests/e2e"]
[schema.kinds.integration]
citable = false
folders = ["tests/integration"]
[schema.kinds.RM]
files = ["docs/roadmap.md"]

[presentation.kinds.GRUND]
title = "Why: project motivation"
[presentation.kinds.GOAL]
title = "Where: project direction and outcomes"
[presentation.kinds.FS]
title = "What: behavior, requirements, and constraints"
[presentation.kinds.REQ]
title = "Hard requirements: what the project must never break"
[presentation.kinds.AR]
title = "How: high-level implementation, structure, and design"
[presentation.kinds.DF]
title = "Product behavior decisions and tradeoffs"
[presentation.kinds.DA]
title = "Architecture decisions and tradeoffs"
[presentation.kinds.e2e]
title = "User scenarios: black-box proof of the spec"
[presentation.kinds.integration]
title = "Integration tests: proof that the parts fit as designed"
[presentation.kinds.RM]
title = "Planned milestones and sequencing"
```

The `[schema.kinds]` rows come first and the `[presentation.kinds]` titles follow in the same row order, which is the order the managed block's Project map lists them in ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)). A non-citable row writes `citable = false` above its places.

#### 2.4.4 No section switch is written

v2 has no `named_sections` switch: named sections are always enabled ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). The file writes none, and the managed block rendered from it teaches named section paths. `init` still never enables the feature on a repository's behalf where a switch exists: a version-1 config keeps `named_sections` as it wrote it ([§FS-config.3.2](FS-config.md#32-id--id-grammar)).

#### 2.4.5 One comment is kept

The file carries one comment, its first line, which says where the rest of the settings are:

```toml
# Only what differs from grund's defaults; `grund config show` prints every setting.
```

No key carries a value-set comment and no commented-out key is written: a minimal file has no default-valued key to annotate. The commented citation ruleset and the commented `CONST` row of earlier starters are dropped; the managed block links the citation-directions guide. The `none` template keeps one more comment, which says how to declare a first kind ([§FS-init.2.4.9](FS-init.md#249-the-none-templates-config)). Comments are not schema keys and do not change parsing.

#### 2.4.6 `shorthand` is left at its default

No `[schema] shorthand` is written: the canonical policy is the v2 default, and `grund config show` prints it ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). The opt-in `accepted` policy is a choice a project writes for itself ([§FS-config.3.1](FS-config.md#31-reference--citation-form)).

#### 2.4.7 `project_name` and `[presentation] description`

`project_name = "<name>"` appears right below `grund_config_version = 2`, above every table. `init` consumes it as the generated project-name default when no explicit `--name` is supplied ([§FS-init.2.3.8](FS-init.md#238-substituted-content)); no other non-workspace subcommand consumes it. It also remains metadata that downstream tooling (IDE status bars, CI dashboards) can read without re-deriving the name. When `--description` was given ([§FS-init.1](FS-init.md#1-inputs)), the file also writes the description at its v2 home, a `[presentation]` table holding `description = "<text>"`, the v2 spelling of v1's `project_description` ([§FS-config-v2.presentation](FS-config-v2.md#presentation-the-presentation-concern)). The table stands right above the first `[presentation.kinds]` table, or at the end of a file that has none. Without `--description` neither the key nor a commented placeholder is written.

#### 2.4.8 Values are not enabled

The starter declares no value kind and illustrates none. Fresh repositories remain unopted-in, and the effective schema contains no `value_sources` key ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)).

#### 2.4.9 The `none` template's config

The `none` template ([§FS-init.1.6.2](FS-init.md#162-none)) writes the envelope alone: the comment of [§FS-init.2.4.5](FS-init.md#245-one-comment-is-kept), `grund_config_version = 2`, `project_name`, and one comment saying how to declare a first kind. With no `[schema.kinds]` row the file declares no kinds and still loads ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). It is exactly these bytes:

```toml
# Only what differs from grund's defaults; `grund config show` prints every setting.
grund_config_version = 2
project_name = "{NAME}"

# No kinds yet. Add a [schema.kinds.<KIND>] table per kind of document this
# repository gives IDs to, then run `grund init` again so AGENTS.md lists them.
```

`--description` appends the `[presentation]` table of [§FS-init.2.4.7](FS-init.md#247-project_name-and-presentation-description) after the last comment.

#### 2.4.10 `REQ` is declared in `docs/requirements/`

The `software-development` template declares `REQ`, the requirements a project must never break, apart from the behaviour `FS` specifies, as grund keeps its own. Its row follows the `FS` row, and its title follows the `FS` title:

```toml
[schema.kinds.REQ]
folders = ["docs/requirements"]
```

```toml
[presentation.kinds.REQ]
title = "Hard requirements: what the project must never break"
```

`FS` is `folders = ["docs/functional-spec"]`, the built-in `FS` home ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)), so the template and the default give one answer ([§DF-fs-home-moves-from-0-19.2.1](../decisions/functional/DF-fs-home-moves-from-0-19.md#21-the-built-in-fs-home-moves-back-to-docsfunctional-spec)). The managed block renders the kinds the written config declares ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), so the project map gains `- [REQ](docs/requirements): Hard requirements: what the project must never break` right after the `FS` row, and the `KIND ∈ {…}` vocabulary lists `REQ` right after `FS`. A version-1 config without a `REQ` row, as every config an earlier `init` wrote, declares no `REQ` ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)).

## 3. Non-intrusive guarantees

What `init` won't touch, in one list; the detail lives at the section cited beside each guarantee.

- **A target that is not a project is refused before anything is written** ([§FS-init.1.2](FS-init.md#12-refused-targets)).
- **Bare `init` creates nothing in a home**: it writes the config and the entrypoints, and nothing under any kind home, `docs/` or `tests/` ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)).
- **Automatic mode never adds a competing entrypoint** ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags), [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)).
- **`init` never creates a second file for an agent that already has one** ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)), save the Claude entrypoint an explicit `--claude` writes beside a symlinked companion in a `conversation = "link"` repository ([§FS-init.2.1.1.1](FS-init.md#2111-a-symlinked-companion-under-conversation--link)).
- **Ambiguous companions need an unambiguous signal** ([§FS-init.3.1](FS-init.md#31-ambiguous-companions-need-an-unambiguous-signal)).
- **User-authored content in agent entrypoints is preserved** ([§FS-init.3.2](FS-init.md#32-user-authored-content-in-agent-entrypoints-is-preserved)), in scoped entrypoints too; a scoped file is deleted only when the block was the whole file ([§FS-init.2.3.5.14.2](FS-init.md#235142-orphans-are-removed)).
- **Line endings outside the managed block are preserved** ([§FS-init.2.3.2](FS-init.md#232-line-endings)).
- **An existing `grund.toml` is never overwritten**, in either discovery location, not even by `--force` ([§FS-init.2.4.1](FS-init.md#241-both-discovery-locations-are-probed), [§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)).
- **No prompts, ever.** Every choice is a flag ([§FS-non-goals.10](FS-non-goals.md#10-interactive-mode)).
- **`--dry-run` previews any run** ([§FS-init.1](FS-init.md#1-inputs), [§FS-init.2.2](FS-init.md#22-stdout--stderr)).
- **Re-running is a true no-op when the repo is already current** ([§FS-init.2.2](FS-init.md#22-stdout--stderr), [§FS-init.2.2.2](FS-init.md#222-the-next-block)).

The `--force` semantics behind the config and "refuse to clobber" guarantees are [§FS-init.3.3](FS-init.md#33-without---force) to [§FS-init.3.5](FS-init.md#35-what---force-does-not-replace).

### 3.1 Ambiguous companions need an unambiguous signal

`.github/copilot-instructions.md` is never created from `.github/` alone (it is generic GitHub metadata) and `.rules` is never created from file existence alone (its filename is too generic): the first needs `--copilot`, the second an existing `.zed/` or `--zed` ([§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection), [§FS-init.2.3.12.1](FS-init.md#23121-which-companions-are-created)).

### 3.2 User-authored content in agent entrypoints is preserved

Unless `--force` replaces the canonical `AGENTS.md` whole ([§FS-init.3.4](FS-init.md#34-with---force)), only the delimiter-bounded managed block, headed by its version heading ([§FS-init.2.3.9](FS-init.md#239-delimiters)), is touched. Everything before and after the block is byte-for-byte preserved, including the block's position within the file ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints), [§FS-init.2.3.1](FS-init.md#231-position-invariants)). The delimiter is the ownership boundary [§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own) is written around.

The same holds for every scoped entrypoint and its standalone companions ([§FS-init.2.3.5.14](FS-init.md#23514-writing-scoped-files)). Removing an orphaned block is the one deletion `init` makes, and it deletes the file only when the managed block was the whole file, byte for byte; any other byte in the file keeps it, with the block cut out and nothing else changed ([§FS-init.2.3.5.14.2](FS-init.md#235142-orphans-are-removed)). A companion symlink `init` mirrored goes with the file it points at.

### 3.3 Without `--force`

Without `--force`, `init` never overwrites an existing file. Existing agent entrypoints are handled by the append/update rules in [§FS-init.2.3.10](FS-init.md#2310-writing-the-block). Every other existing target file is left unchanged and reported with `exists `:

```
exists grund.toml
```

This makes repeated `grund init` runs idempotent and safe for existing repos. `--scaffold` applies the same rule to the starter files: an existing one is reported as `exists ` and left unchanged; a missing one is written.

### 3.4 With `--force`

With `--force`, a selected canonical `AGENTS.md` and the starter files `--scaffold` lays are overwritten in place; their previous contents are not preserved, save the `### Workspace members` section a canonical `AGENTS.md` keeps while an optional member is absent, which is read from the file before it is overwritten ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)) (the user has git for that, per [§FS-non-goals.6](FS-non-goals.md#6-decision-database-audit-log-history-tracking) — `grund` does not maintain its own history). The starter files are files `init` owns, and a repository that has filled `docs/goals.md` or `docs/grund.md` with real content loses it, which is why the flag exists and why nothing else in `init` reaches these files ([§REQ-no-data-loss.3](../requirements/REQ-no-data-loss.md#3-destructive-is-opt-in-and-never-a-side-effect)).

### 3.5 What `--force` does not replace

**`--force` never overwrites the config**: `init --force` still reports it as `exists ` and leaves it untouched ([§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)). Existing *companion* entrypoints are likewise not full-file overwritten by `--force` — the canonical `AGENTS.md` is the one entrypoint the flag replaces whole; if a companion is present and not symlinked to `AGENTS.md`, only its managed block is appended or updated so unrelated agent-specific instructions remain intact.

### 3.6 Bare `init` writes the config and the entrypoints, and nothing in a home

`grund init` without `--scaffold`, whichever its template, lays only what a repository needs to be checkable and to tell its agents about grund:

- It writes the template's `grund.toml`, and only when the target has no config in either discovery location. It never overwrites one, not even with `--force` ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)).
- It writes or updates the managed `grund` block in the agent entrypoints the repository already has; only a repository with none gets the canonical `AGENTS.md` ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags), [§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent), [§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection)).
- It creates nothing under any kind home, and nothing under `docs/` or `tests/`: no starter file, no directory, no `.gitkeep`. The one exception is scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)): the managed scoped `AGENTS.md`, and the companion symlinks mirrored beside it, written only into a destination directory that already exists ([§FS-init.2.3.5.11.4](FS-init.md#235114-a-destination-directory-that-does-not-exist-renders-at-the-root)). That is what makes it safe on a repository that already has specs, and what the `grund-init` skill relies on when it leaves the scaffold off an existing repository ([§DF-skill-init-existing-specs](../decisions/functional/DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding)).
- It prints the `next:` block, whose first step points at `--scaffold`, or at creating the homes by hand, while the effective FS home does not exist ([§FS-init.2.2.2](FS-init.md#222-the-next-block)).

## 4. Exit codes

- `0` — every requested file was written, appended, updated, or already current. Under `--check` ([§FS-init.1](FS-init.md#1-inputs)) this is the tree that is current: every reported path was `exists ` and there is nothing to write.
- `1` — `--check` only: the report carried at least one `would-…` line, so the tree is not current ([§FS-init.4.1](FS-init.md#41-what-earns-the-1)).
- `2` — I/O error (target path does not exist, permission denied, disk full, etc.); a CLI-level error such as an unknown flag; a refused target ([§FS-init.1.2](FS-init.md#12-refused-targets)), in which case nothing was written; an existing config that fails to load ([§FS-init.2.3.8.3](FS-init.md#2383-a-config-that-fails-to-load)); a refused template, an unknown name or `none` beside `--scaffold` ([§FS-init.1.6.3](FS-init.md#163-an-unknown-name), [§FS-init.1.6.4](FS-init.md#164-none-has-nothing-to-scaffold)), in which case nothing was written; an existing agent entrypoint contains a managed block whose schema version is newer than the running binary supports ([§FS-init.2.3.10.2](FS-init.md#23102-a-newer-block-version)); or an existing agent entrypoint has malformed delimiters ([§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)) — in both of those the file is left unchanged. `2` wins over `1` ([§FS-init.4.2](FS-init.md#42-2-wins-over-1)).

`--dry-run` alone keeps `0` whatever it reports ([§FS-init.4.3](FS-init.md#43-why---dry-run-keeps-0)). Exit-code mapping is fixed per [§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out) and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization).

### 4.1 What earns the `1`

The `would-…` lines that earn the `1` are `would-write `, `would-append `, and `would-update `. Nothing was written, and the run that fixes it is the same `init` without the flag. A `note:` never earns the `1` and neither does the `next:` block — a note is a report, not a finding ([§FS-init.2.2.1](FS-init.md#221-notes)), and the exit code is decided by the reported paths alone.

### 4.2 `2` wins over `1`

A run that could not be performed produced no report to gate on, so `--check` over a refused target exits `2` the way any other form of the run does.

### 4.3 Why `--dry-run` keeps `0`

The asymmetry between `--dry-run`'s `0` and `--check`'s `1` is deliberate. `1` is the *findings* slot of the mapping [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed) freezes, and `init` had never used it, so `--check` fills a hole in the mapping rather than moving it. Giving the `1` to `--dry-run` instead would change the verdict an existing green CI step gets on upgrade — quietly, with no finding naming the release and no one-command fix — which is the one thing [§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered) puts the exit-code mapping out of reach of. So there are two spellings because there are two questions: `--dry-run` asks *what would this run do*, `--check` asks *is there anything left to do*.

## 5. The guide

`grund guide` prints the guide to adopting and using `grund` in an arbitrary repo, written for an agent to follow. This is a read-only discovery command: stdout is Markdown, stderr is empty, exit `0`, and it reads no tree, no config and no network. With no argument it prints the front page ([§FS-init.5.6](FS-init.md#56-the-front-page)); `grund guide <section>` prints that one section, and `--full` prints every section ([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)). The command accepts at most one section name and the one flag `--full`, and nothing else. Every other invocation is a CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)): the message on stderr, empty stdout, exit `2`.

- An unknown section name prints ``error: unknown guide section `<name>` `` and then, on a second line, every section in `--full` order: `known sections: setup, config, citations, languages, members, scoped-guidance, schema, queries, editor, integrations, rules`.
- A second section name prints `error: guide takes at most one section`.
- A section name together with `--full` prints `error: a section and --full cannot be used together`.
- Any other flag is the usual ``error: unknown flag `<flag>` ``.

The command exists so a user can tell an agent only "set up grund" plus provide an installed `grund` binary, and the agent can still discover the recommended setup workflow without browsing the source repository. Its name says what it prints for any reader; `agent-setup-instructions`, the name it had before, was removed in grund 0.20.0 ([§FS-init.5.7](FS-init.md#57-agent-setup-instructions-is-the-deprecated-name-of-guide), [§DF-guide-command](../decisions/functional/DF-guide-command.md#df-guide-command-grund-agent-setup-instructions-becomes-grund-guide-whose-bare-run-prints-a-front-page-and-whose-sections-are-named-positionally)).

### 5.1 What the instructions tell the agent

The core, the section `setup`, reads like an agent skill rather than a human tutorial. After the front matter, the title and a one-paragraph introduction, its first three `##` sections are, in this order:

1. **Fresh repository.** A repository is fresh when it has no `grund.toml`, at its root or under `.agents/`, and no specs, decision records or tests to adopt. The section has the agent say it is taking the defaults and, once the user agrees, run three commands in this order: `grund init --scaffold --name <name> --agents-md`, with an agent flag for each agent in use ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)); `grund init --check`; and `grund check`. It says what success looks like: each of the three exits `0`, and on a fresh scaffold `grund check` warns that nothing is recognized ([§FS-check.4.5](FS-check.md#45-nothing-recognized)) until the first declaration exists, which is expected and nothing to fix. A fresh repository takes every default after that one confirmation. The per-option interview is for adoption, and any repository that is not fresh is an adoption. The front page prints this section on its own ([§FS-init.5.6](FS-init.md#56-the-front-page)), so it names the adoption workflow by its invocation, `grund guide setup`, never by its place in the core.
2. **Contents.** It names the core's sections, then gives one line per reference section, in the order of [§FS-init.5.5](FS-init.md#55-the-core-and-its-sections). Each line names how that section is printed, `grund guide <name>`, and says that where the skill is installed as a directory the same text is `references/<name>.md`, then says in one phrase what the section holds. It uses no relative Markdown link, which in printed output would point at nothing.
3. **Workflow.** The adoption workflow, in its existing order. It instructs the agent to inspect the target repo first, identify the existing specs, artifact types, roadmaps, changelogs, decisions, plans, tests, and agent instruction files, recommend suitable `grund init` and `grund.toml` choices with evidence, show pros and cons for every config option, ask the user to confirm or override the recommendations, write `grund.toml`, run `grund init`, then validate with `grund config validate` and `grund check`. The config questions live in the `config` and `citations` sections, so the step that asks them first has an adopting agent fetch both, with `grund guide config` and `grund guide citations`. The config must be written before `grund init` so the generated managed block reflects the selected ID grammar, marker, strict mode, kinds, and existing artifact layout.

The skill tells the agent to ask `grund config explain` for a key or a check code it does not know, so the answer is in the version the project's file is read as and comes from the grund it runs ([§FS-config-explain](FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on)); where the skill itself names a version-specific spelling, it names its version with it ([§REQ-discoverable-offline.1](../requirements/REQ-discoverable-offline.md#1-guidance-names-its-version)). The config questions and the member-splitting recipe of [§FS-init.5.4](FS-init.md#54-splitting-off-a-member) are printed on request. Validation guidance points to the recipe by its `grund guide members` invocation, and the recipe is available whether the target is standalone or already in a workspace. Every other cross-reference from one file of the skill to another names the target's `grund guide <name>` invocation.

### 5.2 Adopting an existing docs-heavy repo

For an existing docs-heavy repo, the instructions make the adoption choice explicit before any write: show the canonical `grund` artifact types, including `GRUND` for the grounding doc, beside the detected project-specific sections, tags, or document classes, then ask whether to use canonical `grund`, canonical core plus project-specific extras, or the existing structure with `grund` citations. The recommended `grund init` form omits `--scaffold` unless the user selects a canonical-layout migration; existing specs are represented in `[[kinds]]` and `[scan]` settings rather than replaced by generic scaffold folders. For the existing-structure model the starting config is `grund init --template=none`'s, which declares no kinds for the agent to fill with the repository's own ([§FS-init.1.6.2](FS-init.md#162-none)) before `grund init` runs again to render them. This follows [§DF-skill-init-existing-specs](../decisions/functional/DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding).

### 5.3 One directory, two surfaces

The core instructions must not fork from the distributable skill. The repository keeps the distributable skill as the directory `skills/grund-init/`, holding `SKILL.md` and one `references/<name>.md` per section of [§FS-init.5.5](FS-init.md#55-the-core-and-its-sections), and the binary-embedded copy as `crates/grund-core/assets/skills/grund-init/`. The two directories are byte-identical file for file, with no extra file on either side. `grund guide <section>` prints one of those files per call, byte for byte: `SKILL.md` for `setup` and `references/<name>.md` for any other `<name>`. With `--full` it prints all of them. The front page is the one output that is no single file, and it holds no guidance of its own: it is cut from `SKILL.md` and from the summaries of its Contents lines ([§FS-init.5.6](FS-init.md#56-the-front-page)). The source package therefore exposes the instructions in two ways with one directory: agents that can read the repository may load `skills/grund-init/` as a skill, whose references load on demand; agents that only have the installed CLI may run `grund guide`. A release that edits one surface without the other is invalid.

The skill's `references/citations.md` contains a marked byte-identical copy of `docs/user-facing/citation-directions.md`, and its `references/rules.md` the marked `### Chapter rules` writing section of `docs/user-facing/rules.md`. Neither copy appears in any other file of the skill. The asset-sync integration check prevents either pair from drifting ([§FS-rules.10](FS-rules.md#10-documentation-and-executable-examples)).

### 5.4 Splitting off a member

Both byte-identical surfaces ([§FS-init.5.3](FS-init.md#53-one-directory-two-surfaces)) teach when and how to split off a member, in the `members` section ([§FS-init.5.5](FS-init.md#55-the-core-and-its-sections)). An independently checked app, package, service, or subproject warrants its own member; an ordinary module or component that still belongs to the current project does not. A documentation folder alone is not a project boundary. New prose uses member and alias ([§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure)); the frozen workspace template wording is retained ([§FS-init.2.3.4.16](FS-init.md#23416-project-boundaries)).

The recipe gives the member its own `grund.toml` and a stable `project_name`, registers it in the workspace root's `[workspace] members` (creating that root configuration when splitting a standalone repository), and runs `grund init` in the member. Local IDs remain local; qualified citations use `<marker>alias/<ID>` with one alias segment per nesting level. Run `grund check` from the workspace root for full validation ([§FS-workspace.1](FS-workspace.md#1-citation-syntax), [§FS-workspace.2](FS-workspace.md#2-workspace-configuration), [§FS-workspace.5](FS-workspace.md#5-command-scope)). Setup advice explains this topology; it does not claim that `grund init` creates it automatically ([§FS-init.2.3.4.15.7](FS-init.md#234157-what-init-does-not-do)).

### 5.5 The core and its sections

The core, `SKILL.md`, is the section `setup`. It holds the opening of [§FS-init.5.1](FS-init.md#51-what-the-instructions-tell-the-agent), *Fresh repository*, *Contents* and *Workflow*, and the adoption guidance an agent needs before it asks for another section. It is at most **16,000 bytes**. A core that would outgrow the budget moves text into a section; the budget does not rise to fit it.

The sections are a closed set of eleven: `setup`, then ten reference sections. Each is printed by `grund guide <name>` and named, in this order, by the front page, the `known sections:` line and `--full`, and the Contents list names the ten reference sections in the same order:

| section | what it holds |
|---|---|
| `setup` | the core: adopting grund in a repository that is not fresh, its workflow, questions and validation |
| `config` | how to recommend each setting, then the config questions, except `[citations]` |
| `citations` | the `[citations]` question and the marked citation-directions copy ([§FS-init.5.3](FS-init.md#53-one-directory-two-surfaces)) |
| `languages` | the language and repository-shape examples, with version-2 sources and the move from version 1 |
| `members` | the member-splitting recipe ([§FS-init.5.4](FS-init.md#54-splitting-off-a-member)) |
| `scoped-guidance` | scoped guidance |
| `schema` | seeing the schema with `grund schema` ([§FS-schema.14](FS-schema.md#14-where-an-agent-learns-it)) |
| `queries` | querying the structure |
| `editor` | editor setup |
| `integrations` | the clickable-citation integrations, except the chapter rules |
| `rules` | the marked chapter-rules copy ([§FS-init.5.3](FS-init.md#53-one-directory-two-surfaces)) |

Every part of the guide is in exactly one file, so nothing is lost by the split, only deferred: each section is one `grund guide <name>` away from the installed binary, offline.

`--full` prints every section in that order, the core first, each file byte for byte, with one newline between consecutive files, so one blank line separates them. It is the whole guide in one response, but not the bytes of the single file that preceded the split: the same text in a new order ([§DF-setup-guide-core-by-default](../decisions/functional/DF-setup-guide-core-by-default.md#df-setup-guide-core-by-default-grund-agent-setup-instructions-prints-a-core-that-opens-with-the-fresh-path-and-prints-each-reference-section-on-request)).

Adding a section, or renaming one, is a deliberate change of this point, as adding an integrations client is a change of [§FS-integrations](FS-integrations.md#fs-integrations-grund-prints-and-installs-its-rendering-layer-integrations). Serves [§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file): the cheap read is the default and every escalation is named.

### 5.6 The front page

Bare `grund guide` prints the front page: what a fresh repository runs, and what every section holds and costs to read, in one short response. It is, in order:

1. The title line `# grund guide` and a blank line.
2. The core's *Fresh repository* section ([§FS-init.5.1](FS-init.md#51-what-the-instructions-tell-the-agent)) byte for byte, from its `## Fresh repository` heading up to the core's next `##` heading.
3. A `## Sections` heading and one sentence saying that `grund guide <section>` prints one section and `grund guide --full` prints every section, with the size of the whole guide.
4. A `text` fence holding one line per section, in `--full` order, with the section's name, its size and its summary in aligned columns. A size is the byte count the section prints, in kilobytes of 1,000 bytes rounded to the nearest whole and never below `1`, written `<n> KB`. A reference section's summary is the phrase its Contents line ends with, without the closing period. The summary of `setup` is the phrase the table above gives it.

Every byte of guidance on the front page is the core's or a Contents line's, and each size is measured from the text its section prints, so the front page cannot disagree with the guide. It serves the two readers the guide has: a fresh repository runs the commands it prints and needs no second call, and an adoption reads `grund guide setup` next ([§DF-guide-command](../decisions/functional/DF-guide-command.md#df-guide-command-grund-agent-setup-instructions-becomes-grund-guide-whose-bare-run-prints-a-front-page-and-whose-sections-are-named-positionally)).

### 5.7 `agent-setup-instructions` is the deprecated name of `guide`

`agent-setup-instructions` was the command's name through grund 0.18.0. grund 0.19.0 kept it as a deprecated name under [§REQ-backwards-compatibility.2](../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): it printed the core, the text `grund guide setup` prints, after one warning on stderr naming the replacement and the release that removes the name ([§DF-guide-command](../decisions/functional/DF-guide-command.md#df-guide-command-grund-agent-setup-instructions-becomes-grund-guide-whose-bare-run-prints-a-front-page-and-whose-sections-are-named-positionally)). That release is 0.20.0, and the name is gone ([§DF-agent-setup-instructions-removed-in-0-20](../decisions/functional/DF-agent-setup-instructions-removed-in-0-20.md#df-agent-setup-instructions-removed-in-0-20-grund-agent-setup-instructions-is-no-command-from-0200)): it is no subcommand, hidden or listed, and nothing answers to it by name. It is refused as every first word that is neither a subcommand nor an ID is ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)). `grund agent-setup-instructions` is the ID query of [§FS-cli.1](FS-cli.md#1-the-default-subcommand), and `--full` is that query's flag, so the bare name and the name with `--full` are both its failure of [§FS-cli.1.2](FS-cli.md#12-a-first-word-that-is-not-an-id): the invalid ID and the format hint, and no `grund --help` hint, because the word holds a `-`, with an empty stdout and exit `1`; any other argument is read as that query reads it. Where the word is no valid ID, as under the default `{kind}-{slug}` format, the first line reads:

```
invalid ID `agent-setup-instructions`
```

`grund help agent-setup-instructions` and `grund agent-setup-instructions -h` are the help request with an unknown first word of [§FS-cli.1.3](FS-cli.md#13-a-help-request-with-an-unknown-first-word): `error: unknown command: agent-setup-instructions` and the `known commands:` line, which names `guide`, with exit `2`. Neither names `grund guide` as the replacement; the release note and this point do.

The bindings keep `agent_setup_instructions()` and `agentSetupInstructions()`, which return the `--full` text: they are not a command-line spelling, so nothing of theirs is renamed or removed ([§DF-setup-guide-core-by-default.2.3](../decisions/functional/DF-setup-guide-core-by-default.md#23-the-bindings-return-the-whole-guide)). This address, heading text included, is kept so citations written while the name was deprecated still resolve.

## 6. Why this exists

`init` is the smallest on-ramp `grund` offers: one command, every default the most-common case, nothing the user already authored rewritten without `--force` ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)). The shape follows from three goals taken together — an agent that reads its entrypoint file at session start should arrive already taught ([§GOAL-agent-grounding.1](../goals.md#1-the-three-layers)); a conformant tree should work without configuration so the emitted defaults *are* the canonical grammar ([§GOAL-zero-config](../goals.md#goal-zero-config-works-on-any-conformant-tree)); and the on-ramp should not surprise either humans or scripts, so there are no prompts and every choice is a flag ([§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)).

### 6.1 One command instead of a copy-paste

Without `init`, the on-ramp to `grund` would be a copy-paste of someone else's agent instructions, with the resulting drift between projects. `init` collapses that on-ramp to one command and freezes the selected agent entrypoint at the `grund` version that wrote it — when the canonical form evolves, `grund init` re-emits the managed block in place ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable) still applies through the project's `grund.toml`, which the re-emitted block is rendered from and `init` never overwrites, [§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)).

### 6.2 The grammar before any IDs exist

`init` is also the only safe place to demonstrate the ID grammar before any IDs exist: a repo that has not yet authored its first `FS-001-…` declaration still gets a literate agent entrypoint from `init` that teaches the grammar by example.

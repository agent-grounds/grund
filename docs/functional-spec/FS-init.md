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

A tree `init` has just written passes `grund check` unmodified: the scaffold and the managed block are clean as written ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place), [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), the worked example is escaped and no bare ID-shaped token is written, so the block reads clean under `strict = false` too ([§FS-init.2.3.8.2](FS-init.md#2382-the-worked-example-is-escaped), [§FS-init.2.3.7.1](FS-init.md#2371-the-current-version)), and an `init` followed by `fmt --write` is one tree that both `fmt --check` and `init --check` accept ([§FS-init.2.3.5.10](FS-init.md#23510-chapter-rules)). Where the two commands read one input they read it alike: an unreadable config is the error `check` gives for it ([§FS-init.2.3.8.3](FS-init.md#2383-a-config-that-fails-to-load)), an entrypoint is attributed on the evidence `check` uses ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)), and workspace members expand by `check`'s rules ([§FS-init.2.3.4.15.3](FS-init.md#234153-foreign-project-rows)). In a fresh workspace, one `grund init` in each project, in any order, leaves the outermost root's `grund check` clean: no block describes a sibling as it was before that sibling's own `init` ([§FS-init.2.3.4.15.6](FS-init.md#234156-the-link-target), [§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)). `init` is not among [§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)'s surfaces, so this is where that agreement is asked of it. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.reproducible: What `init` writes is a function of the target, its config, the flags and the version — realized

Two runs at one `grund` version over the same target with the same flags write the same bytes: the scaffold ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)), the managed block ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)), and each entrypoint's clickable-citation form, which the written path decides and which embeds nothing about the machine ([§FS-init.2.3.6](FS-init.md#236-clickable-citations), [§FS-init.2.3.4.17.2](FS-init.md#234172-the-form-per-agent)). Of the target's ancestors only the workspace that claims it shapes those bytes ([§FS-init.2.3.4.15.1](FS-init.md#234151-which-workspace)); the name and the effective config come from the target alone, never from an ancestor's config ([§FS-init.2.3.8](FS-init.md#238-substituted-content)). While an optional member is absent, the member list written is the one the entrypoint already holds ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)): bytes of the target, not of the machine. Otherwise the member list is a function of the workspace's configs alone, not of which of its projects have been initialized ([§FS-init.2.3.4.15.6](FS-init.md#234156-the-link-target)). Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible); [§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes) names only the commands that read a tree, so this is the writer's half.

### requirements.faithful: The block describes the repository it lands in, and claims nothing `check` does not enforce there — realized

What the block says about the repository is rendered from its effective config, never from the defaults ([§FS-init.2.3.8.1](FS-init.md#2381-what-the-config-fills-in)): its sentences teach the configured marker ([§FS-init.2.3.4.17.1](FS-init.md#234171-the-marker)), its project map is the configured layout ([§FS-init.2.3.4.4](FS-init.md#2344-project-map)), named-section guidance appears only where the feature is on ([§FS-init.2.3.4.5](FS-init.md#2345-declaration-forms)), and its direction and grounding rules are the ones `grund.toml` makes `check` apply ([§FS-init.2.3.4.10](FS-init.md#23410-citation-direction), [§FS-init.2.3.5.7](FS-init.md#2357-the-grounding-sentence)). A config change that makes it untrue anywhere is caught by `check`'s byte comparison of the whole block ([§FS-init.2.3.5.8](FS-init.md#2358-the-drift-check), [§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)); a workspace change that makes its workspace sections untrue is caught by the `check` at the outermost root of a full checkout ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)), and in every other run by `init --check` ([§FS-init.1.4](FS-init.md#14-why---check-exists)), save the member list in a checkout without an optional member, which only the full checkout's root `check` judges ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)). Serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers).

### requirements.total: A run that exits `2` leaves the tree as it was — realized

Every reason `init` can read before writing refuses the whole run, as [§FS-init.4.2](FS-init.md#42-2-wins-over-1) assumes when it calls such a run one that could not be performed: a refused target ([§FS-init.1.2](FS-init.md#12-refused-targets)), an unreadable config ([§FS-init.1.2.4](FS-init.md#124-which-refusal-is-reported-first)) and a CLI error write nothing, and so does an entrypoint whose block is newer than the binary or malformed ([§FS-init.2.3.10.2](FS-init.md#23102-a-newer-block-version), [§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)): every planned entrypoint's block is read before the first is written, so one planned ahead of it is left as it was ([§FS-init.1.2.6](FS-init.md#126-entrypoint-blocks-are-read-before-any-is-written)). An I/O failure while writing is outside this point. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.next-agent: An agent joins on every axis at once — directional

The supported-agent set is one list every agent-facing feature reads ([§FS-init.2.3.12](FS-init.md#2312-supported-agents-and-their-entrypoints)). An agent added to it from here on arrives, in one change, with its flag ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)); its place in automatic selection, created only on a signal that attributes it unambiguously ([§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection), [§FS-init.3.1](FS-init.md#31-ambiguous-companions-need-an-unambiguous-signal)); the one-created-entrypoint rule ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)); its clickable-citation form under the per-agent gate ([§FS-init.2.3.4.17.2](FS-init.md#234172-the-form-per-agent)); the refusal of [§FS-init.1.2.2](FS-init.md#122-user-global-instruction-files) for any user-global instruction file it reads ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)); and `check`'s verification of what `init` writes for it ([§FS-check.3.5.1](FS-check.md#351-which-entrypoints-are-verified)). This binds the next agent; it is not a claim about how the present ones were added. Serves [§GOAL-agent-grounding.1](../goals.md#1-the-three-layers).

## 1. Inputs

```
grund init [<path>] [--name <name>] [--description <text>] [--template <name>] [--scaffold] [--force] [--dry-run] [--check] [--no-vcs] [--agents-md] [--claude] [--gemini] [--pi] [--copilot] [--cursor] [--windsurf] [--zed]
```

- `<path>` — directory in which to scaffold; defaults to `.`. Applies to every form of `init`, `--scaffold` included, and prefixes every emitted path in [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place). Must exist: `init` does not create the target directory, because a missing target is a user error, not something to silently paper over. A target whose lookup is refused for lack of permission, on the target or on a directory above it, is not a missing one, because that lookup says nothing about it ([§FS-scan.scope.homes.lookup.path](FS-scan.md#scopehomeslookuppath-a-path-the-command-is-given-is-missing-only-when-its-lookup-says-so)): `init` refuses it at the path as given with the reason `permission denied`, exits `2`, and writes nothing. A lookup that fails for any reason but absence or a refusal, such as a link that loops, is refused the same way with its own reason ([§FS-scan.scope.links.unresolvable.named](FS-scan.md#scopelinksunresolvablenamed-a-looping-link-named-as-the-path-is-reported-at-the-link-not-as-missing)).
- `--name <name>` — human-readable project name baked into the generated `AGENTS.md` heading when canonical `AGENTS.md` is selected, and into the `grund.toml` `project_name` key when that config is freshly written. An explicit value wins; otherwise `init` uses `project_name` from a config at either supported location in the target, and only when the target has no configured name does it fall back to the basename of `<path>` resolved to an absolute path. An ancestor's config never supplies this default, and an existing target config is never rewritten ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)).
- `--description <text>` — one-line project description baked into the generated `grund.toml` `project_description` key ([§FS-config.3](FS-config.md#3-keys-version-1)), replacing the commented teaching line [§FS-init.2.4](FS-init.md#24-generated-grundtoml) writes by default. No default: `init` never invents a description. A `<text>` containing a line break is a CLI error, mirroring the config-side single-line rule. Like `--name`, the value only lands in a freshly written config — an existing `grund.toml` is never modified ([§FS-init.2.4](FS-init.md#24-generated-grundtoml)).
- `--template <name>` — the template to lay: `software-development`, the default, or `none` ([§FS-init.1.6](FS-init.md#16-templates)). `--template=<name>` is the same flag. The template is where the generated config and the starter files come from; a target that already has a config keeps it ([§FS-init.1.6.5](FS-init.md#165-a-config-that-is-already-there)).
- `--scaffold` — also lay the template's starter files at the homes the effective config names: [§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place) lists the default set, and [§FS-init.1.3](FS-init.md#13---scaffold-and-the-effective-config) says why each is there and how the config changes it. Off by default — most adopters already have a `docs/` of some shape, and bare `init` lays nothing in a home ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)). Refused beside `--template=none` ([§FS-init.1.6.4](FS-init.md#164-none-has-nothing-to-scaffold)).
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

Bare `init`, with no `--scaffold`, writes only these ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)):

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

Paths are relative to `<path>`. The path transcript stays on stderr
([§FS-errors.6](FS-errors.md#6-the-grund-init-transcript)); located rule findings
are printed on stdout ([§FS-init.4.1](FS-init.md#41-what-earns-the-1)). With no rule finding, stdout is empty.

#### 2.2.1 Notes

The run's `note:` lines, one per line, each name something the run could not do, or a state of the tree the caller would otherwise have to notice for itself: a duplicated entrypoint ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)), a Claude entrypoint shadowed by a symlink ([§FS-init.2.3.4.17](FS-init.md#23417-clickable-citations)), a template config the target's own config kept out ([§FS-init.1.6.5](FS-init.md#165-a-config-that-is-already-there)), a member list kept or written without an optional member that is absent ([§FS-init.2.3.4.15.8](FS-init.md#234158-while-an-optional-member-is-absent)). A note is a report, not a finding: it changes nothing, and the exit code is the one the run would have had without it ([§FS-init.4](FS-init.md#4-exit-codes)).

Under `--dry-run` a note is written in the conditional, like every other line of that report: the run wrote nothing, so a note in the present tense describes a tree that does not exist — and an instruction that assumes the write already happened, such as deleting the symlink a preview did not replace, costs the reader the only entrypoint they have. What such an instruction waits on is the **block**, not the file: a preview's `would-append` names an entrypoint that already exists, so a condition the reader can satisfy by looking is no condition at all.

#### 2.2.2 The `next:` block

After the notes, stderr prints a short `next:` block — a blank line, then numbered first steps, then `see <entrypoint> for the full workflow.` — so the user is not left at a bare list of paths wondering what to do. `<entrypoint>` is the first agent entrypoint `init` wrote, updated, or found already current. The steps are: run `grund check`; allocate an ID in the kind [§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names) selects, `ID=$(grund id FS "…")` where that is FS, and add it to the selected home (under a folder with `# <ID>: …`, as the generated config makes FS's, or in a file with the `## <ID>: …` H2 when explicit config makes the home a file); cite it as `§<ID>` from the docs and e2e tests that depend on it. A project whose selection is not FS with a home gets the configured-kind form of [§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names) instead of the FS form below.

The check step explains that on a fresh scaffold, before the first declaration exists, `grund check` warns `nothing recognized` under [§FS-check.4.5](FS-check.md#45-nothing-recognized) and exits `0`. This is expected and needs no repair: the next step begins grounding by adding the first declaration. Both the direct `--scaffold` guidance and the non-scaffold advice about the next scaffold teach this expectation, conditional on a **fresh** scaffold; adopting grund into an already populated repository does not predict that warning. Neither form promises a clean success verdict for a fresh scaffold, and the configured-kind form without `--scaffold`, which offers none, prints the plain step instead ([§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names)). The exact check-step wording appears in [§FS-init-fixtures.1](FS-init-fixtures.md#1-default-form), [§FS-init-fixtures.1.1](FS-init-fixtures.md#11-populated-configured-form), and [§FS-init-fixtures.2](FS-init-fixtures.md#2-scaffold-form).

When `--scaffold` was *not* passed, a re-run advice is inserted as step 1, the following steps move down, the allocation step drops its second line — the one naming the `## <ID>: …` H2 or `# <ID>: …` H1, which prints only under `--scaffold` — and the citation step is omitted. Whenever it is printed, that advice points at re-running with `--scaffold` (or creating the FS home, `docs/`, and `tests/` by hand); a refresh of a complete setup does not print it at all ([§FS-init.2.2.2.1](FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing)). It appends the clause `until then \`grund check\` has nothing to scan` only when the effective scanner would read no file — exactly the scan decision of the governing config's version. For version 2 that is the scanned rows' places and the complement's extent, the config root unless `[schema.kinds.code]` lists places ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)), so the starter's own tree, `AGENTS.md` included, is read and a fresh run does not print the clause. For version 1 it is the scan decision in [§FS-config.3.5](FS-config.md#35-scan--what-gets-scanned): configured extensions, ignore and exclude rules, hidden-file and symlink handling, workspace boundaries, configured include paths, scanned kind homes, and `scan = false` all retain their scanner meanings. Path existence or a non-empty directory alone does not make the clause false. A config that declares no kinds gets the block of [§FS-init.2.2.2.2](FS-init.md#2222-a-project-with-no-kinds) instead.

Without `--scaffold`, the `next:` block is suppressed on a refresh of a complete setup ([§FS-init.2.2.2.1](FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing)) and nowhere else. A run whose every reported path is `exists ` (or, under `--dry-run` and `--check`, `exists ` with no `would-…` line) is not a complete setup by that alone: it is one case of that refresh, suppressed exactly when that test holds, so it keeps the block while the selected home is missing and whenever the config declares no citable kind ([§FS-init.2.2.2.2](FS-init.md#2222-a-project-with-no-kinds)) — a re-run that changed nothing has finished no step the previous run taught. A `--scaffold` run whose every reported path is `exists ` prints no block: every file the scaffold lays is already there. When it is printed, it is guidance, not a finding: part of the success output, with no effect on the exit code.

##### 2.2.2.1 A refresh of a complete setup teaches nothing

A run with no `--scaffold` is a **refresh of a complete setup** when both hold:

- every reported path is `exists ` or `updated ` — under `--dry-run` and `--check`, `exists ` or `would-update ` — a run whose every line is `exists ` included, so a single `wrote `, `appended `, `would-write `, or `would-append ` line makes the run something else;
- the home [§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names) selects exists on disk, resolved against the target directory, or the selected row has no home, so there is no place left to create. The built-in `FS` folder `docs/functional-spec` ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)) is not tested unless the effective FS row is the selection and names it: a project that does not declare that folder is not missing it.

The lines for scoped files and their companion symlinks ([§FS-init.2.3.5.14.4](FS-init.md#235144-the-report-names-every-scoped-file-and-link-it-touched)) are not counted in the first condition: they follow the root block, so a refresh that moves a row's guidance, creating one scoped file and removing another, is still a refresh.

Such a run prints no `next:` block. It is where `grund check` sends a repository whose managed block a `grund.toml` change made stale ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)); after it the user has a complete, current setup, and the advice to scaffold an FS home that exists would send them hunting for a detection failure that did not happen. Their next step is the `grund check` that sent them here.

Everything else keeps the block exactly as [§FS-init.2.2.2](FS-init.md#222-the-next-block) renders it: a first run (`wrote ` or `appended ` of an entrypoint) even where the FS home, `docs/`, and `tests/` exist ([§FS-init-fixtures.1.1](FS-init-fixtures.md#11-populated-configured-form)), and a refresh whose selected home is absent, a re-run that reports only `exists ` lines included: the step to create that home is as open as it was on the run that first taught it. Suppressing the block moves no exit code: `--check` still exits `1` on a `would-update ` ([§FS-init.4](FS-init.md#4-exit-codes)).

##### 2.2.2.2 A project with no kinds

A config that declares no kinds ([§FS-config.3.4.4.5](FS-config.md#3445-kinds---declares-no-kinds)) has no FS home to scaffold or to allocate in, so every step above would send the user somewhere that does not exist — `--scaffold` included, which the `none` template refuses ([§FS-init.1.6.4](FS-init.md#164-none-has-nothing-to-scaffold)). The block teaches the one step that is missing instead, `<entrypoint>` as above:

```
next:
  1. declare a first kind in grund.toml, then run `grund init` again so AGENTS.md lists it
  2. run `grund check`
  3. allocate an ID:  ID=$(grund id <KIND> "…")  then add it to that kind's home
see AGENTS.md for the full workflow.
```

It is printed and suppressed by the rules of [§FS-init.2.2.2](FS-init.md#222-the-next-block), save that such a run, a re-run that reports only `exists ` lines included, is never a refresh of a complete setup ([§FS-init.2.2.2.1](FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing)): it has no effective FS home to find on disk, and the step it teaches is still open. The bindings read the same decision from the guidance's `declares_kinds` field ([§DF-init-templates.2.3](../decisions/functional/DF-init-templates.md#23-the-bindings-keep-docs)).

##### 2.2.2.3 The kind the guidance names

The allocation step and the refresh test of [§FS-init.2.2.2.1](FS-init.md#2221-a-refresh-of-a-complete-setup-teaches-nothing) name one kind of the effective config, and at most one place of it: the `FS` row when it is citable and has a home; otherwise the first citable row, in configuration order, that has a home; otherwise the first citable row. The selected row's place is its first `file`, else its first `folder`, each in written order ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)), and the selection never moves to another place, or another row, because that one exists. A non-citable row never qualifies, the complement `code` included ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)), and a config with no citable row selects nothing and gets the block of [§FS-init.2.2.2.2](FS-init.md#2222-a-project-with-no-kinds). The managed block's discovery rung leads its `--kind` example with the same selection's kind ([§FS-init.2.3.4.3](FS-init.md#2343-cheap-grounding)). The built-in `FS` folder `docs/functional-spec` ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)) stays the home `--scaffold` lays the FS starter in where no FS row has one ([§FS-init.2.1](FS-init.md#21-files-written-updated-or-left-in-place)); the guidance names it only where the effective FS row does ([§DF-init-configured-kind-guidance](../decisions/functional/DF-init-configured-kind-guidance.md#df-init-configured-kind-guidance-grund-inits-guidance-names-a-kind-and-a-home-the-project-declares)).

A selection that is FS with a home gets the FS form of [§FS-init.2.2.2](FS-init.md#222-the-next-block), unchanged. Every other selection, another kind or a row with no home, FS's included, gets the **configured-kind form**, whose exact blocks are [§FS-init-fixtures.8](FS-init-fixtures.md#8-a-project-without-fs): its allocation step names the selected kind in `grund id <KIND>` and its place, or, for a row with no home, says to declare the ID in a source doc-comment and names no directory. Without `--scaffold` it never offers `--scaffold`, which lays grund's whole software-development starter, the FS home included ([§FS-init.2.1.3](FS-init.md#213-the-software-development-starter-files)), rather than the selected home alone: that tree holds the selected home only where the starter carries the selected kind at that place, as it does a `REQ` row's folder, so offering it would lay a tree the project did not ask for, and for a kind or a folder the starter does not carry it would leave the selected home missing still. Its first step asks the user to create the selected home, only while that home is missing, and carries the `until then` clause by the rule of [§FS-init.2.2.2](FS-init.md#222-the-next-block); its check step is the plain `run \`grund check\``, because the form neither laid nor offered a scaffold for "on a fresh scaffold" to refer to. With `--scaffold` the run writes what it writes for any project (§FS-init.2.1), and the block is the FS scaffold form with the allocation step naming the selected kind and place; its check step keeps the fresh-scaffold wording, for the scaffold the run just laid.

The guidance carries the selection as data. `InitNext.allocation` is an `Option<InitAllocation>`, `None` exactly when the config has no citable row. `InitAllocation { kind, home, home_exists }` names the selected kind, its place as an `Option<InitAllocationHome>` (`File { path, heading_name, heading_marker }` or `Folder { path }`, the shapes `InitFsHome` gives a place), and whether that place exists on disk, `false` for a row with no home. The bindings carry it as `allocation`, with `home_exists` and `homeExists` ([§FS-distribution.3.2.2.2](FS-distribution.md#3222-result-fields)). `InitNext.fs_home` and `InitFsHome` keep their meaning, the home `--scaffold` lays the FS starter in, built-in folder included. The selection moves no exit code and adds no finding.

### 2.3 Generated agent entrypoints

The guidance `init` writes into an entrypoint is a managed block ([§FS-init.terms](FS-init.md#terms-terms)): the session-start workflow and rules an agent needs before work begins, rendered against the target's effective configuration, leaving detail to the declarations it cites and to `grund <ID>` ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)). This section is not a transcript of the block, whose bytes are the template the binary embeds for the block's version, rendered against that config ([§FS-init.2.3.7](FS-init.md#237-the-block-version), [§FS-init.2.3.8](FS-init.md#238-substituted-content)). It fixes what each part says and when it renders, and that the render is deterministic: two `grund init` runs at the same `grund` version against trees with the same `--name` and the same effective config write byte-identical blocks ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)), and generated output passes `grund check` unmodified, so the generator and the checker never disagree about the block's own contents ([§FS-init.requirements.clean](FS-init.md#requirementsclean-what-init-writes-check-accepts--realized)).

#### 2.3.4 Managed-block content sections

This is what the block says from its opening paragraph through `### Rules`, phrased compactly because the entrypoint is read at the start of every session ([§GOAL-token-economy](../goals.md#goal-token-economy-give-an-agent-the-right-amount-of-spec-not-the-whole-file)).

**Citation scheme.** The block identifies itself as the entrypoint for agents working in the project and instructs them to read it. It teaches the configured citation scheme: IDs have the configured shape, citations use the configured marker and optional section path, `grund check` validates citations, `grund list` discovers IDs, `grund <ID>` reads cited declarations or sections, and `grund refs` reports what leans on a declaration. It teaches one citation form, `§<ID>`, bare, with an optional `.<section>` path, in the same words whichever `[fmt.cross_refs].enabled` mode applies ([§DF-md-link-emission.2.4](../decisions/functional/DF-md-link-emission.md#24-default-on-for-generated-configs)): any Markdown link around a citation on disk is generated by `grund fmt --write`, not hand-authored. One short sentence of the citations narrative, not phrased as a rule, tells agents to cross-link declarations only via IDs.

**Grounding from a citation.** The block teaches the cheap grounding ladder: `grund <ID>` as the first read for a bare citation, `grund <ID> --brief` when the heading and first paragraph are enough, `grund <ID> --toc` when section navigation is needed, `grund <ID>.<section>` for section citations, and `grund <ID> --full` only when a narrower read is insufficient. The first rung says what the default read prints, whole declaration and section alike ([§FS-show.2.2](FS-show.md#22-section), [§DF-block-names-section-heading](../decisions/functional/DF-block-names-section-heading.md#df-block-names-section-heading-the-blocks-first-rung-says-a-section-read-opens-with-its-heading-line)); it reads exactly `` - `grund <ID>` — the lead, cut at the first child section: heading-less for a whole declaration, while `grund <ID>.<section>` opens with that section's heading line. The cheap first read for a bare `§<ID>` citation. ``, with the configured marker in place of `§`.

The `refs` rung tells agents to run `grund refs <ID> --summary` before changing, moving, or renaming a declaration, and the full `grund refs <ID>` when exact citation sites are needed. Beside it the block teaches what follows ([§DF-fmt-maintenance-guidance](../decisions/functional/DF-fmt-maintenance-guidance.md#df-fmt-maintenance-guidance-teach-the-existing-post-rename-formatter-maintenance)): edit the declaration and the affected authored citation IDs, run `grund fmt --write` from the config/project root or workspace root, which re-derives links ([§FS-fmt.6.3](FS-fmt.md#63-idempotency-and-re-derive)) and maintains locks ([§FS-section-lock.write.rules](FS-section-lock.md#writerules-add-refresh-drop)), verify with `grund check` and `grund fmt --check`, and commit the lock changes; the guide it links, *Maintaining declarations*, says which root that is ([§FS-section-lock.checks.remedy](FS-section-lock.md#checksremedy-the-run-that-writes-the-lock)). It never asks the reader to hand-edit a lock, or implies that `fmt` replaces authored IDs, repairs arbitrary URLs, renames an ID-keyed lock on a file move alone ([§FS-section-lock.file.shard](FS-section-lock.md#fileshard-one-file-per-cited-declaration)), or approves a protected section move ([§FS-section-lock.write.never-a-move](FS-section-lock.md#writenever-a-move-fmt-never-approves-a-move)).

The discovery rung scopes `grund list` by a `--kind` example naming the project's own citable kinds from the set `KIND ∈ {…}` lists: first the kind init's `next:` block names ([§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names)), then the first other citable kind with a `folder` home, in configuration order, if there is one, joined by `,`. So the default kinds ([§FS-config.3.4.4](FS-config.md#344-the-default-kinds)) read `` - `grund list` / `grund list --kind FS,AR` — discover IDs if you get lost ``, a project whose only kind is `REQ` reads `grund list --kind REQ`, and a config with no citable kind gets `` - `grund list` — discover IDs if you get lost `` with no `--kind` half: every recipe the rung teaches runs in the project it was rendered for. Directly after it, one bullet reads exactly `` - `grund list --unused` — the declarations nothing cites yet, in one call ``, the set `check` warns on ([§FS-list.1.3](FS-list.md#13---unused)); it carries no recipe ([§DF-block-teaches-unused](../decisions/functional/DF-block-teaches-unused.md#df-block-teaches-unused-the-managed-block-teaches-grund-list---unused)). Beside the ladder, a bullet teaches `grund list --size=words --top 10` as the sweep for heavy leads.

One bullet teaches reading structure across many coordinates, with no pipe: `grund list --glob '<pattern>' --total` counts the coordinates a pattern selects ([§FS-list.total](FS-list.md#total---total-a-count-rung)), and `grund show --batch --glob '<pattern>' --toc --format json` reads each one's section map in the same scan ([§FS-show.1.9](FS-show.md#19---batch---all-every-coordinate-in-scope), [§FS-show.2.1.2](FS-show.md#212-section-map---toc)), from `.result.sections[]`. It names `*`, `**` and `{a,b}` ([§FS-coordinate-globs.grammar](FS-coordinate-globs.md#grammar-grammar)) and links the user-facing guide *Querying grund* by its absolute URL, `https://github.com/agent-grounds/grund/blob/main/docs/user-facing/querying.md`, because the block lands in repositories with no `docs/user-facing/` of their own ([§REQ-shipped-surfaces](../requirements/REQ-shipped-surfaces.md#req-shipped-surfaces-what-grund-ships-or-prints-resolves-where-it-lands)). Its pattern is `<KIND>`, the literal the project's `[id] format` writes after `{kind}`, then `*`: `<KIND>-*` for `{kind}-{slug}` and `<KIND>*` for `{kind}{number}`, never a `-` the IDs do not have.

That guide puts the query recipes in one place, each with its command, a short example and its JSON shape: selecting units (`list --glob`, `list --selector`, `list --kind`), counting them (`list --total`), expanding a unit's subtree (`show --batch --glob` over `<ID>{,.**}`), bodies in bulk (`show --batch --brief|--full`, `--glob`, `--all`), who cites a point (`refs`, `--descendants`, [§FS-refs.1](FS-refs.md#1-inputs)), the whole citation graph (`cover --format json` with `enclosing_declaration` and `enclosing_section`, [§FS-cover.3.2](FS-cover.md#32---format-json)), a review, which counts a checklist, reads its bodies, then maps changed lines to their owners (`cover FILE --lines N-M`, [§FS-cover.6](FS-cover.md#6-line-ownership)), and the declarations nothing cites yet, with `list --unused` and as one object per declaration with `--format json`, linking the checking guide's *When only an index cites a declaration*. It also states the glob grammar and its departures from the `exclude` dialect. The `grund-init` skill carries a matching section that teaches the glob recipe with a placeholder pattern and links the guide. Both name `grund list --unused` as the one call for what nothing cites yet, which leaves out the cases of a configured `E2E` kind ([§FS-list.1.3.1](FS-list.md#131-a-configured-e2e-kind-is-left-out-unless-selected)), and neither teaches `--unused --kind E2E`, a form the default kinds refuse ([§FS-list.1.3.2](FS-list.md#132-the-default-kinds-have-no-e2e-kind)).

**Project map.** The map lists the kind homes of the effective `grund.toml` as a Markdown link list, `- [KIND](home): Title`, never a table. It lists the current project's homes only, in a workspace too, and never the scan scope.

A non-citable kind ([§FS-config.3.4.1](FS-config.md#341-citable--kinds-that-declare-no-ids)) is named by its place, `- [skills/](skills): Agent review and automation skills`. An unscanned kind ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) is rendered the same way. The homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) has no place and so no row. In a version-2 config a kind with several places is still one row, linking every place in the row's place order, files then folders, each in written order ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)): `- [FS](docs/spec/core), [docs/spec/plugins](docs/spec/plugins): Declaration`. A non-citable row is named by all its places the same way, and the complement, `code`, gets a row named by its places when it lists any and none when its extent is the config root ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)).

A row's title is the one the config gives its kind — `[presentation.kinds.<KIND>] title` in version 2, the `[[kinds]]` table's `title` in version 1 — whenever it gives one. Untitled, the complement's row, the one row that declares no IDs, reads `Every scanned file no other row claims`, and every other row reads `Declaration`:

```markdown
- [FS](docs/spec): Declaration
- [src/app/](src/app), [src/lib/](src/lib): Every scanned file no other row claims
```

Both titles are the block's wording, not config defaults: `grund config show` prints no title for either row, and a file `init` writes carries no default title ([§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default)).

A config that declares no kinds ([§FS-config.3.4.4.5](FS-config.md#3445-kinds---declares-no-kinds)) gets, in place of the map, one paragraph teaching the first kind in the config's own spelling; for a version-1 config:

```markdown
No kinds are declared yet: `grund.toml` sets `kinds = []`. Replace it with one `[[kinds]]` table per kind of document this repository gives IDs to — `kind = "FS"`, then `folder = "docs/specs"` or `file = "requirements.md"`, and a `title` — and run `grund init` again to list them here.
```

A version-2 config with no row is taught `[schema.kinds.<KIND>]` and its `folders` or `files` the same way ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)), and the citation-scheme line's kind set reads ``no `KIND` is declared yet`` rather than `KIND ∈ {}`.

**Workspace sections.** When — and only when — the init target lies in a workspace ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), the block carries `### Project namespaces` and, after it, `### Workspace members`, beside the Project map ([§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible)). The workspace is found by the claimed-chain climb of [§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces), not by the discovery walk-up of [§FS-config.1](FS-config.md#1-file-location-and-discovery), which stops at the nearest config: whether or not the target's own `grund.toml` declares `[workspace]`, it is the **outermost** ancestor that both declares `[workspace]` and lists the directory below it, and the sections read that ancestor's block rather than the target's config. So they render whether `init` runs at the workspace root or inside a member, and in a nested workspace they cover the whole tree. An ancestor block that does not claim the tree is not read, so a standalone target, one below such an ancestor included, gets neither section. Every selected entrypoint uses the same discovery result: companion selection, `--name`, descriptions, `include_root`, and an empty or suppressed member list do not decide workspace presence, and `init`, `--check`, `--dry-run` and `--force` render the same condition.

`### Project namespaces` is fixed text, only the configured marker substituted in its local and qualified citation examples: it distinguishes project boundaries from documentation folders, identifies the current project as local, and gives the member-splitting recipe, which the setup instructions teach in full even for a standalone repository ([§FS-init.5.4](FS-init.md#54-splitting-off-a-member)).

`### Workspace members` holds exactly two things: the line `Cross-project citations use <marker>alias/<ID>, one alias segment per workspace level.`, verbatim except for the target's configured marker, standing for the grammar of [§FS-workspace.1](FS-workspace.md#1-citation-syntax), and one bullet per resolved **foreign** workspace project, sorted by alias. Members are expanded as `grund check` expands them ([§FS-workspace.2](FS-workspace.md#2-workspace-configuration)), recursing into a member that declares its own `[workspace]`, so every foreign project in the tree gets a row labelled with its whole alias path ([§FS-workspace.6.1](FS-workspace.md#61-nested-workspaces), [§FS-workspace.3](FS-workspace.md#3-aliases)). The project whose entrypoint is rendered is omitted: self is the resolved project whose canonical root equals the canonical init target, never one guessed from alias text, whether the run writes canonical `AGENTS.md` or only a companion. So the root omits its own row even when `include_root = true`, which still makes it a foreign row in member entrypoints, while `include_root = false` omits it there too; an intermediate node obeys its own block's `include_root` the same way, so a pure grouping directory contributes no row. Pending `--name` and `--description` values create no local row.

A bullet labels a Markdown link with the backticked alias, `` - [`api`](apps/api/) ``, and appends `: <description>` when the foreign project's own config has a `project_description` ([§FS-config.3](FS-config.md#3-keys-version-1), [§FS-workspace.3](FS-workspace.md#3-aliases)); without one it is the link alone, and `init` never derives a description it was not given ([§DF-workspace-member-descriptions](../decisions/functional/DF-workspace-member-descriptions.md#df-workspace-member-descriptions-member-side-project_description-for-workspace-member-lists)). The link target is the project's root directory with a trailing `/`, never a file in it, relative to the directory of the entrypoint being written: `apps/api/` from a workspace-root `AGENTS.md`, `../../apps/api/` from another member's at that depth, and `../../` for the root listed from `apps/api/AGENTS.md`. Nothing on disk about a foreign project — whether its `AGENTS.md` or a companion exists — changes its bullet, and no bullet says whether it has been initialized: the section is a function of the workspace's configuration ([§FS-init.requirements.reproducible](FS-init.md#requirementsreproducible-what-init-writes-is-a-function-of-the-target-its-config-the-flags-and-the-version--realized), [§DF-member-link-names-the-directory](../decisions/functional/DF-member-link-names-the-directory.md#df-member-link-names-the-directory-a-workspace-member-link-names-the-members-directory)).

If workspace expansion, alias validation, alias uniqueness, or nested-workspace validation would fail under the rules of `grund check`, `init` leaves `### Workspace members` out, and `check` reports the configuration error; `### Project namespaces` still renders. `init` does not prompt for, infer, or configure workspace topology ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)): it adds no `[workspace]` block, and writes no file under a member directory other than the member it was invoked in, so each member's own `AGENTS.md` is a separate `init` there.

While an optional member is absent anywhere in the workspace ([§FS-workspace.2.2](FS-workspace.md#22-a-member-that-may-be-legitimately-absent)), the run is one whose workspace sections `check` leaves out ([§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout)), and `init` keeps the member list it cannot render whole ([§DF-init-keeps-the-member-list](../decisions/functional/DF-init-keeps-the-member-list.md#df-init-keeps-the-member-list-grund-init-keeps-a-blocks-member-list-while-an-optional-member-is-absent)). A root entrypoint whose block already has a `### Workspace members` section keeps that section's bytes, from its heading to the block's next heading, as they are, while the rest of the block is refreshed; each entrypoint keeps its own. Where the block or the section is missing, the section lists the members the run can read, and the full checkout's root `check` reports that block until `grund init` runs there. A workspace that fails expansion still leaves the section out, as above. `--dry-run`, `--check` ([§FS-init.1.4](FS-init.md#14-why---check-exists)) and `--force` ([§FS-init.3.4](FS-init.md#34-with---force)) keep and render exactly what the write does, so a run whose only difference is inside a kept section reports `exists`, and `--check` reports `would-update` only for a difference outside one. The run prints one note ([§FS-init.2.2.1](FS-init.md#221-notes)) per absent member, ordered by namespace, naming its entry and namespace and then, for each root entrypoint it reports, in report order, whether that entrypoint's section was kept or written new, the clauses joined by `; `. Under a write, the note for a kept section and the note for one written new read:

```console
note: optional workspace member `packages/secret` (namespace `secret`) is absent, so AGENTS.md keeps its `### Workspace members` section as it was
note: optional workspace member `packages/secret` (namespace `secret`) is absent, so AGENTS.md's new `### Workspace members` section does not list it
```

Under `--dry-run` and `--check` the clauses are in the conditional: `would keep its … section as it is` and `… section would not list it`.

Only `### Workspace members` is kept: `### Project namespaces` renders the same in both checkouts. The member list is refreshed only from a checkout that sees every member, where `check` judges it: a blind spot of `init --check`, declared by the note and bounded by that run ([§REQ-no-missed-citation.2](../requirements/REQ-no-missed-citation.md#2-every-blind-spot-is-declared-and-bounded)).

**Declarations and citations.** The block teaches that declarations are heading lines in Markdown or in supported language doc-comments, and that an inline source declaration can be represented by a one-line Markdown stub in the configured kind home. Rendered according to `[id] section_heading_levels`, it teaches that numbered headings inside a declaration are citable sections, and that depth-matching headings (`## 1. …`, `### 1.1 …`) are required in strict mode, warned on in warn mode, and recommended in loose mode. Where named sections are enabled it also teaches the complete-path forms `## goals: Goals`, `### goals.performance: Performance` and `### goals.3: Ordered child`, saying "handles are letter-first lowercase names, and `name.number` and `number.name` are both legal; a cited address with a name under a number is recorded in the section lock" ([§FS-config.3.3.1](FS-config.md#331-named-components), [§FS-section-lock.holds.entry](FS-section-lock.md#holdsentry-one-line-per-locked-address)); a repository without them receives no named-heading guidance. It teaches the relative form for a Markdown and a doc-comment declaration body alike ([§FS-declarations.section.relative.where](FS-declarations.md#sectionrelativewhere-markdown-declaration-bodies-and-doc-comments-from-0190)), in a sentence opening "In a declaration body, Markdown or doc-comment, a heading may instead write only the component it adds": `### .1: Username` under `## format: Formatting` where named sections are enabled, `### .1. Scope` under `## 3. Third` otherwise, and `grund fmt --write` after citing one so the section lock records it ([§FS-declarations.section.relative](FS-declarations.md#sectionrelative-a-heading-may-write-only-the-component-it-adds), [§FS-section-lock](FS-section-lock.md#fs-section-lock-a-cited-relative-or-mixed-section-keeps-its-text-or-check-says-where-the-text-went)). And it teaches that every non-declaration ATX heading inside a Markdown declaration body must carry a numbered or enabled named section path, while file titles, body-closing headings, fenced examples, non-ATX text and source doc-comments are exempt and bold labels are the non-citable alternative ([§FS-config.3.3](FS-config.md#33-section-paths--arbitrary-nesting-depth), [§FS-declarations.checks.unmarked-heading](FS-declarations.md#checksunmarked-heading-unmarked-markdown-heading)).

**Rules.** The rules tell agents to write or update the most-specific functional or architectural declaration before the behavior or design code that implements it; to cite the most-specific declaration the code or prose realizes, a whole behavior on its function, class or block doc-comment and a narrower clause or decision inline where it is enforced; to refresh the cited declaration with `grund <ID>` before editing code that already carries a `§<ID>` or `§<ID>.<section>` citation; that decisions must be cited from the functional-spec or architecture declaration they shaped, and that decision history is append-only, a reversal being a new decision that supersedes the older one; that behavior is proven by executable tests or cases, and a disagreement between the spec and that proof is fixed on both sides in the same change; and to run `grund check` before committing, because a dangling citation is a stop-the-line bug whose finding names the file and line. The marker rule ends by naming a fenced code block as one way to mention an ID without citing it. Where `[schema.sources] fenced` lists tags ([§FS-config-v2.schema.sources.fenced](FS-config-v2.md#schemasourcesfenced-fenced-names-the-fences-whose-citations-are-read)), it names only the fences those tags leave unread, as ``or use a fenced code block not tagged `mermaid`.``, the tags in written order and joined as a prohibition's targets are ([§FS-init.2.3.5](FS-init.md#235-citation-directions)), ``not tagged `mermaid`, `dot`, or `plantuml`.``; where it lists none, the bullet ends ``or use a fenced code block.`` The inline citation style sentences are [§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)'s.

#### 2.3.5 Citation directions

The block's `### Citation directions` section states the citation-direction rules the effective config declares and no others — `[citations]` in version 1, `[rules.citations]` in version 2, where a grounding ladder alone declares none ([§FS-config.3.9](FS-config.md#39-citations--citation-direction-rules), [§FS-config-v2.rules.citations](FS-config-v2.md#rulescitations-rulescitations)) — so the rule an agent reads is the rule `grund check` enforces ([§DF-citation-directions.2.7](../decisions/functional/DF-citation-directions.md#27-generated-agent-entrypoint-section-with-a-drift-check)). It states rules, never config grammar, in the wording [§DF-directions-render](../decisions/functional/DF-directions-render.md#df-directions-render-the-citation-directions-wording-is-chosen-once-against-a-canonical-config) chose.

A config that declares no rule renders no `### Citation directions` section, whatever kinds it declares and with `kinds = []` alike ([§DF-directions-only-from-rules](../decisions/functional/DF-directions-only-from-rules.md#df-directions-only-from-rules-the-citation-directions-section-renders-only-the-rules-a-config-declares)). Grounding is the one exception: where some row's effective `require_grounding` lands at a destination, the root or a scoped block, that destination's section is the heading, a blank line, and its grounding sentence alone, with no legend and no closing line. The climbing convention — specs cite goals, architecture cites specs, code and tests cite the specs they realize — is taught by `grund guide citations` instead ([§FS-init.5](FS-init.md#5-the-guide)).

Otherwise the section opens with one paragraph — the fixed legend ``must`/`never` are `grund check` errors; `should`/`avoid` are suggestions (`grund check --suggestions`).``, followed by the grounding sentence where any row's effective `require_grounding` is on — then one bullet per citing kind that has any rule, in `[[kinds]]` order with the homeless kind ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind)) last, then one closing line. An unscanned kind ([§FS-config.3.4.7](FS-config.md#347-scan--a-place-that-is-listed-not-scanned)) has no bullet: it can carry no rule. Under scoped guidance the layout is per destination, and the root block's section holds only the rows whose destination is the root, or, with none there, the legend and the closing line.

**Every bullet names its unit** ([§FS-check.3.11](FS-check.md#311-missing-required-citation), [§DF-directions-render.2.2](../decisions/functional/DF-directions-render.md#22-the-rules-the-render-follows)):

| Citing kind | Subject |
|---|---|
| citable | `Each **DA** declaration` |
| non-citable, folder home | `Each file in **skills/**` |
| non-citable, folder home, grounding off | `Each file in **skills/** that cites anything` |
| non-citable, single-file home | `The file **docs/runbook.md**` |
| non-citable, single-file home, grounding off | `The file **docs/runbook.md**, if it cites anything,` |
| homeless, with a `title` | `Each source file outside the Project map (**code**: Gradle build and workflows) that cites anything` |
| homeless, without one | `Each source file outside the Project map (**code**) that cites anything` |

A non-citable kind is named by its place, as in the Project map. The homeless kind keeps its name and says what it covers, `(**code**)` when untitled whatever the map calls its row ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)), and its *that cites anything* is load-bearing: the obligation constrains what a source file cites, never whether it cites ([§FS-config.3.9.2](FS-config.md#392-the-homeless-kind), [§DF-directions-render.2.3](../decisions/functional/DF-directions-render.md#23-two-lines-the-ticket-settled-and-why-they-differ)). A non-citable home's subject is unnarrowed exactly when its row's grounding is on and `check` judges a file there with no citation ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in)).

A non-citable home's row grounds nothing when no grounding rung of it, at any strength, reaches its files: its effective `require_grounding` is off — the row's own key or `[rules.citations.<row>.grounding]` ladder, else the global default ([§FS-config.3.4.8.3](FS-config.md#3483-precedence-is-row-over-global), [§FS-config-v2.rules.grounding](FS-config-v2.md#rulesgrounding-grounding-is-a-ladder-replaced-whole)). Its bullet then claims only what `grund.toml` makes `check` enforce ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)): the subject takes the grounding-off form, and a second sentence says a file that cites nothing is not checked and names the one setting that makes it a finding, the one piece of config grammar the section renders ([§DF-directions-render.6.1](../decisions/functional/DF-directions-render.md#61-decision)). The setting is spelled in the live config's version — version 2 `` `[rules.citations.<row>.grounding] must = "file"` ``, version 1 `` `require_grounding = true` on its `[[kinds]]` row `` — so the row `e2e` at `tests/e2e/` with `must = ["FS"]` renders, under version 2:

```text
- Each file in **tests/e2e/** that cites anything must cite FS. A file there that cites nothing is not checked; `[rules.citations.e2e.grounding] must = "file"` makes it a finding.
```

A single-file home's second sentence opens `If it cites nothing, it is not checked;` and a several-file home's `One that cites nothing is not checked;`. The homeless kind's bullet is the same either way: it is narrowed already.

The clauses follow the subject, joined by `; ` in the order `must`, `should`, `may`, `must-not`, `should-not`, then the per-kind default. A prohibition that leads its bullet takes the modal — `must not cite`, `should not cite`, the wording of [§FS-check.3.12](FS-check.md#312-forbidden-citation)'s findings — and one after a preceding clause the short `never cite` / `avoid citing` the legend names ([§DF-directions-render.2.6](../decisions/functional/DF-directions-render.md#26-a-prohibition-that-leads-its-bullet-takes-the-modal)).

Targets render as prose, never as rule grammar. Alternatives inside one entry join with "or", with the Oxford comma from three on (`FS, AR, or RM`). An obligation's entries join with "and", and where there is more than one entry, an entry with alternatives of its own is parenthesised: `must = ["FS|GOAL", "AR"]` renders `(FS or GOAL) and AR`. A prohibition bans each target on its own, at every citation site to it ([§FS-config.3.9.1.1](FS-config.md#3911-obligations-and-prohibitions), [§FS-check.3.12](FS-check.md#312-forbidden-citation)), so its targets join as one list of alternatives, every entry's alternatives in it and none parenthesised — `must not cite FS or AR`, `never cite FS, AR, or RM` ([§DF-prohibition-targets-join-with-or](../decisions/functional/DF-prohibition-targets-join-with-or.md#df-prohibition-targets-join-with-or-a-prohibitions-targets-join-with-or)). A pinned alias renders exactly as spelled (`api/AR`); `*/AR`, rule grammar and never a citation ([§FS-config.3.9.3](FS-config.md#393-alias-matching)), renders as `AR in any project`.

Only `must-not` and `should-not` close anything: a default of `must`, `should` or `may` invents no obligation and forbids nothing ([§FS-config.3.9.4](FS-config.md#394-defaults-and-precedence)). A per-kind `must-not` default folds into the permission it qualifies — `may cite only FS or GOAL` — when the `may` list is the whole of what the kind permits; with a `must` or `should` beside it the permitted set is wider, so the bullet ends `never cite anything else` instead. A per-kind `should-not` default ends its bullet with `avoid citing anything else`. A per-kind default that leaves its kind open says `may cite anything else` only where the global default is closed.

The closing line reports only the global default: `Anything not listed above is allowed.` for an open one, `must` and `should` included; `Any citation not listed above is forbidden.` for `must-not`; `Any citation not listed above is discouraged.` for `should-not`.

The grounding sentence is generated from each row's effective `require_grounding` — the row's own key, else `[reference] require_grounding` ([§FS-check.3.6.1](FS-check.md#361-which-files-a-row-governs), [§FS-config.3.4.8](FS-config.md#348-require_grounding-and-grounding_level--grounding-per-place-and-per-level)) — and the configured non-citable homes. It claims only what [§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in) enforces: it names a place exactly when that place's row has grounding on, in version 2 a `[rules.citations.<row>.grounding]` ladder with a `must` rung, and opens with the source-file clause exactly when the homeless kind's row has it on; it names no level. It distinguishes citing from declaring: `Every source file must cite a declared ID or declare one inline`, extended with `; every file under skills/ and tests/e2e/ must cite one` for the scanned non-citable homes whose rows have it on ([§FS-declarations.checks.misplaced-declaration](FS-declarations.md#checksmisplaced-declaration-misplaced-declaration-configured-kind-home), [§DF-directions-render.2.3](../decisions/functional/DF-directions-render.md#23-two-lines-the-ticket-settled-and-why-they-differ)). An unscanned home is left out. Under scoped guidance each destination's sentence names only the rows that land there: the source-file clause renders at the complement's destination, and a non-citable home's clause at its row's, after the source-file clause where both land in one place, and on its own, as `Every file under tests/e2e/ and tests/integration/ must cite one.`, where they do not or where the homeless kind's row grounds nothing.

**Declaration shapes.** A kind that declares fields has its shape rendered under `### Declaration shapes` from the record the checks read ([§FS-fields.record](FS-fields.md#record-the-record)), in the words of `grund schema`'s shape lines ([§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node)). The section is the heading, a blank line, the paragraph `` `grund check` holds each declaration of these kinds to these fields, as `grund schema` lists them. Each heading is shown under a declaration written `# <ID>: …`; a field always sits one level below its declaration's heading: ``, a blank line, then one bullet `- **<KIND>**` per kind that lands at the destination and declares fields, in row order, each followed by one nested bullet per line [§FS-schema.4.1](FS-schema.md#41-fields-are-lines-inside-their-kinds-node) prints under that kind's node, in its order. A nested bullet is the line's text without the `field <name>`, `key <name>` or `closed` the view prints before it, as in ``  - `## status: <one of Draft, Accepted>` (must, first): the title is the value``, and adds no escaping of its own ([§FS-schema.4.1.2](FS-schema.md#412-the-code-span-and-the-placeholders)). A `front-matter` field's bullet is never a heading ([§FS-schema.4.1.3](FS-schema.md#413-front-matter-is-no-heading)), and its keys and its own `closed` nest one level under it ([§FS-fields.front-matter.record](FS-fields.md#front-matterrecord-the-record-and-what-renders-it)). In the root block the section renders after `### Citation directions` and before `### Chapter rules`.

**Chapter rules.** When at least one kind enables `rules = true`, the block carries the `### Chapter rules` section of [§FS-rules.9](FS-rules.md#9-managed-guidance-and-editor-parity), whose refusal and no-write lifecycle is [§FS-rules.4](FS-rules.md#4-validation-lifecycle)'s. Each bullet's live rule citation takes the configured canonical cross-reference representation ([§FS-fmt.6.2](FS-fmt.md#62-form)): with `[fmt.cross_refs] enabled = true`, a Markdown link whose destination is relative to the actual entrypoint and whose anchor follows the configured anchor profile; with cross-references disabled, bare. `init` and `check` render it from the loaded rule vocabulary, keeping the exact authored sentence and the qualified rule-ID order in either form. So `init` then `fmt --write` leaves a tree that both `fmt --check` and `init --check` accept, with no formatter suppression directive, and repeating either keeps the section byte for byte.

**Scoped guidance.** In a version-2 config whose `[presentation] rules` is `home`, its default ([§FS-config-v2.presentation](FS-config-v2.md#presentation-the-presentation-concern)), each row's guidance — its citation-direction bullet, its clause of the grounding sentence, its kind's declaration shape, and the chapter rules whose subject is its kind — is written into the managed block of the `AGENTS.md` in the row's **destination**, the deepest directory that contains every one of its places, rather than into the root entrypoint ([§DF-scoped-guidance](../decisions/functional/DF-scoped-guidance.md#df-scoped-guidance-a-version-2-rows-guidance-is-written-above-its-places-with-a-root-pointer-for-agents-that-do-not-load-nested-files)).

A folder place counts as itself and a file place as its parent directory, and the destination is the longest run of leading path components they share: `docs/spec/core` and `docs/spec/plugins` land in `docs/spec`, and `docs/spec/core` with `docs/goals.md` in `docs`. A file at the config root, or places that share no leading component, make the destination the root, which is the root entrypoint's own block. A citable kind with no place, declared only in doc-comments ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)), lands at the root; the complement `code` lands at the root when its extent is the config root, and at the deepest common directory of its places when it lists some ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)). Only a row that renders a bullet, a grounding clause, a field or a chapter rule has a destination, so a `scan = false` row has one only through its kind's fields or chapter rules, and a config with none of the four writes no scoped file. A destination counts when its directory exists or the run's `--scaffold` lays it ([§FS-init.2.1.3](FS-init.md#213-the-software-development-starter-files)); otherwise the row renders in the root block as though its destination were the root, and `init` creates no directory ([§FS-init.3.6](FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)). Existence is read at each run: creating the directory later makes `check` report the root block stale and the scoped block missing until one `grund init` moves the guidance. With `[presentation] rules = "root"`, and in every version-1 config, every destination is the root: no scoped file is written, no `### Scoped guidance` section renders, and a version-2 config's field shapes render in the root block.

A scoped block is bounded by the delimiters and version heading of [§FS-init.2.3.9](FS-init.md#239-delimiters), at the same `block vN` as the run's root block. A blank line follows the heading, then one line naming the scope, `` These rules cover the files under `<dir>/`; the rest of this project's grund guidance is in the root `AGENTS.md`. ``, then each of these sections that has content, in this order and separated by a blank line:

1. `### Citation directions`: the legend and the destination's grounding sentence as one paragraph, a blank line, the bullets of the rows that land there in layout order, and the closing line for the global default. It renders when a bullet or a grounding clause lands there, and where the config declares no rule, as such a config's does.
2. `### Declaration shapes`, when a kind that lands there has fields.
3. `### Chapter rules`: the legend and the bullets whose subject kind lands there, each citation relative to that file.
4. `### Scoped guidance`, in the complement's destination when that is not the root, so a source declaration's guidance names where its kind's shape lives.

A scoped `AGENTS.md` that `init` creates holds the managed block and nothing else — no generated title, unlike the root file ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)) — and the end delimiter's line ending is its last byte, so "nothing but the block is left" is a byte comparison.

When, and only when, at least one row lands outside the root, the root block renders a `### Scoped guidance` section after `### Citation directions`, `### Declaration shapes` and `### Chapter rules` and before `### Clickable citations`:

```markdown
### Scoped guidance

Some places' citation directions and declaration shapes are in the `AGENTS.md` of the directory above them. If your agent has not already loaded that file, read it before writing there:

- `docs/spec/AGENTS.md`: FS
- `tests/AGENTS.md`: tests/e2e/, tests/integration/
```

There is one line per destination, ordered by the first row that lands there, each naming its rows as a bullet's subject names them: a citable kind by its name, a non-citable row by its places with a trailing `/` for a folder, the complement as `code`, joined by `, ` in row order. It is the fallback for an agent that does not load a nested file by itself ([§DF-scoped-guidance.loading](../decisions/functional/DF-scoped-guidance.md#loading-which-agents-load-a-scoped-file-by-themselves)). That list, in the root block **on disk**, is the record of the scoped files `init` wrote, so neither `init` nor `check` walks the tree for managed blocks: `init` reads it before it rewrites the root block, `check` reads it beside the live render, and a path it names that is not a destination of the live render is an **orphan**. Only a line `` - `<path>`: … `` inside that section of a delimited root block names a path, and a path that is absolute, contains `..`, or does not end in `/AGENTS.md` is ignored. A scoped block the list does not name is not `init`'s and is never touched.

`init` writes each destination's `<dir>/AGENTS.md` as it writes an entrypoint ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)): created holding the block alone, appended to, or re-rendered in place with every byte around the block preserved ([§FS-init.3.2](FS-init.md#32-user-authored-content-in-agent-entrypoints-is-preserved)), and a byte-identical block is left untouched and reported `exists `. The read-before-write rules hold for these files too ([§FS-init.1.2.6](FS-init.md#126-entrypoint-blocks-are-read-before-any-is-written)): a newer block version or malformed delimiters in a scoped file, or a scoped `AGENTS.md` that is a symlink, which `init` never writes through, exits `2` and writes nothing anywhere. `--force` never replaces a scoped file whole ([§FS-init.3.5](FS-init.md#35-what---force-does-not-replace)). For each orphan whose file carries a managed block, `init` removes the block: it deletes the file only when the block was the whole file, byte for byte, and otherwise cuts the region from the begin delimiter through the end delimiter's line ending, leaving every other byte, handwritten text before and after the block included. An orphan whose file is gone, or carries no block, is left alone and not reported.

Where the root `CLAUDE.md` or `GEMINI.md` is a symlink to `AGENTS.md`, each destination gets the same name as a symlink to its own `AGENTS.md` when that name is free there; an existing symlink to `AGENTS.md` is left as it is, and an existing standalone file has the block appended or re-rendered as a companion's is ([§FS-init.3.5](FS-init.md#35-what---force-does-not-replace)). Those are the only companions in a live destination that `init` writes and `check` compares: a symlink there to anything else, and a standalone `CLAUDE.md` or `GEMINI.md` whose root name is not such a symlink, are left as they are and expected nothing of. In an orphan's directory, the `AGENTS.md` and each standalone `CLAUDE.md` or `GEMINI.md` have their block removed as above, each on its own account, whichever root name is mirrored and whether or not the `AGENTS.md` is still there or still carries a block; those are the files there that `init` cuts and `check` compares. Deleting the `AGENTS.md` deletes the symlinks to it in its directory. No other symlink there is touched or compared, an `AGENTS.md` that is itself a symlink included, and no other entrypoint of [§FS-init.2.3.12](FS-init.md#2312-supported-agents-and-their-entrypoints) is mirrored.

`init`'s report ([§FS-init.2.2](FS-init.md#22-stdout--stderr)) names each scoped file and companion symlink it creates, updates or removes. After the root entrypoints' lines and before the config's, it prints one line per scoped file in pointer order, each followed by its companions, then one per orphan in the order the list on disk names them, each followed by its companions: `wrote <dir>/AGENTS.md` for a created file; `appended ` and `updated ` as for any entrypoint, `updated ` also for a kept file whose orphaned block was cut; `exists ` for a current one; `linked <dir>/CLAUDE.md -> AGENTS.md` for a created companion symlink; and `removed <path>` for a deleted scoped file or companion symlink. A companion symlink the run leaves as it is prints nothing, as at the root. Under `--dry-run` and `--check` the new prefixes read `would-link ` and `would-remove `, and both earn `--check`'s `1` ([§FS-init.4.1](FS-init.md#41-what-earns-the-1)).

A scoped `AGENTS.md` that is a destination of the live render or that the list on disk names, and a companion beside it, are not units of the place they sit in: they draw no grounding, citation-direction, index or misplaced-declaration finding as a file of that row ([§FS-check.3.6](FS-check.md#36-ungrounded-unit-opt-in), [§FS-check.3.11](FS-check.md#311-missing-required-citation)), though their citations are still resolved, a chapter rule's live citation among them. A row that lists the file in its `files` claims it, and it is then that row's unit like any other file, while `init` still writes its block ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)).

**The drift check.** Under scoped guidance `check`'s comparison ([§FS-init.2.3.7](FS-init.md#237-the-block-version)) is per file: the root block is compared whole, `### Declaration shapes` and `### Scoped guidance` among its sections, and each destination's `AGENTS.md` and each standalone companion beside it must carry its scoped block as rendered — a missing file or block is a missing block, differing bytes a stale one — while each orphan the list on disk names that still carries a managed block is stale too, its section named `scoped guidance`. Every one is `agents-init` ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)), and `grund init --check` reports the same files as `would-…` lines.

#### 2.3.6 Clickable citations

The block's `### Clickable citations` section always carries the repository-web sentence ([§DF-neural-link-generation](../decisions/functional/DF-neural-link-generation.md#df-neural-link-generation-agents-compose-clickable-citation-links-themselves-grund-does-not-grow-a-link-command)). When the effective `grund.toml` sets `[reference] conversation = "link"` ([§FS-config.3.1](FS-config.md#31-reference--citation-form), [§DF-repo-conversation-opinion](../decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)), it also carries a local-conversation sentence in the form of the entrypoint's agent, followed in both forms by the fixed deference clause. A Claude entrypoint under that key reads:

```markdown
On repository web surfaces, link `§<ID>` to the PR branch in PR bodies, the reviewed commit in reviews, an exact commit for permalinks, and the default branch otherwise; fall back to plain when unsure. In local conversations, render `§<ID>` as a Markdown link whose visible text is the citation itself and whose target is `file://<absolute path>#L<line>` for its declaration; fall back to the bare citation when unsure. If a user-level grund block states a local-conversation rendering, follow that instead: that machine knows what its surface can open.
```

Every `§` in the section is the configured `[reference] marker` ([§FS-config.3.1](FS-config.md#31-reference--citation-form)): the wording is fixed and the marker is not, because a hardcoded `§` in a repository with another marker would teach a token `grund check` ignores under `strict`, so the claim it grounds would never be verified.

The form follows the gate of [§DF-conversation-link-target.2.4](../decisions/functional/DF-conversation-link-target.md#24-the-form-is-gated-per-agent-and-the-fallback-is-path). Claude's entrypoints (`CLAUDE.md`, `.claude/CLAUDE.md`) and Pi's (`.pi/AGENTS.md`) carry the `file://` link form above; every other entrypoint carries ``In local conversations, follow `§<ID>` with its declaration location as plain `path:line` text; fall back to the bare citation when unsure.`` The committed form is always the `file` target, composed at write time from the repository root the agent already has, so it embeds nothing about a machine and two installs render byte-identical files ([§FS-non-goals.13](FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree)). The section is a function of the effective config and the entrypoint's written path, so it is byte-identical in every project with the same key state, for the same entrypoint file, and the agent's form is the one axis on which two entrypoints of one repository differ. When the key is absent, so is the local-conversation sentence, and local rendering is governed solely by the user preference `grund integrations --write` installs in user-level agent instructions ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)). The deference clause keeps rendering machine-local: the committed opinion is the fallback for a machine that never stated a preference, and one that recorded a preference wins ([§DF-repo-conversation-opinion.2.3](../decisions/functional/DF-repo-conversation-opinion.md#23-precedence)).

Because the written path decides the form, a `CLAUDE.md` that is a companion symlink to `AGENTS.md` selects the canonical `AGENTS.md` target ([§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection)), so the one file both agents read carries the plain form, as the gate requires; a repository that wants the linked form for Claude replaces the symlink with the real entrypoint `--claude` writes ([§FS-init.2.1.1.1](FS-init.md#2111-a-symlinked-companion-under-conversation--link)). When the effective config commits the opinion and a Claude entrypoint resolves to the canonical target, the run emits one `note:` line ([§FS-init.2.2.1](FS-init.md#221-notes)) naming the symlink and the fix still open. The fix depends on the tree, not on the run, so the same tree gets the same diagnosis whatever flag reached it. Where Claude has no entrypoint of its own, the fix is `grund init --claude`, and where symlinks have taken every path Claude reads, the note names deleting one of them first. Where Claude has one, from this run or already on disk, the fix is deleting the symlink, named as safe to do now only when this run left that entrypoint carrying the current block, and otherwise — a preview, or an entrypoint this run did not select — named with the condition still on it, because a symlink deleted ahead of the block that replaces it costs the reader their only Claude entrypoint.

#### 2.3.7 The block version

The block's text for a block version `vN` is embedded in the `grund` binary; its one copy in the source tree is `crates/grund-core/assets/templates/AGENTS.md`, which the binary embeds. The `vN` in the version heading ([§FS-init.2.3.9](FS-init.md#239-delimiters)) versions every byte the block renders, its fixed text and the lines substituted in ([§FS-init.2.3.8](FS-init.md#238-substituted-content)), the inline citation style sentences among them ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)), under [§REQ-backwards-compatibility.3](../requirements/REQ-backwards-compatibility.md#3-loud-mechanical-migrations): a change to the block is a version bump carried by that mechanism, never a silent rewrite. A change **in grund** to what the block renders **for a given config and workspace** moves the version, and changes that land between two releases share one bump ([§DF-check-compares-the-whole-block.2.5](../decisions/functional/DF-check-compares-the-whole-block.md#25-every-change-to-the-render-moves-the-version)). A repository that edits its config, migrates it, or adds a workspace member changes what the block renders under the same version: `check` reports the block stale and `grund init` refreshes it. `vN` is the revision of the block's text and nothing else: it is not a grund release, the two numbers move independently, and a grund release is read from `grund -V` alone.

The current versions are two pairs, and the config selects among them: **v47** for a config that renders no scoped file and no declaration shape, **v48** for such a config once a kind enables rules ([§FS-init.2.3.5](FS-init.md#235-citation-directions)), and **v49** and **v50** the same two for a config that renders a scoped file or a declaration shape ([§DF-scoped-guidance.2.4](../decisions/functional/DF-scoped-guidance.md#24-the-block-version)). The maximum supported version is **v50**.

`grund check`'s agent-entrypoint validation ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)) checks the delimiter lines and the version, then renders the whole block as `init` writes it and compares it byte for byte, section by section ([§FS-check.3.5.4](FS-check.md#354-no-part-of-the-block-is-exempt-from-the-comparison)), the two workspace sections only where [§FS-check.3.5.5](FS-check.md#355-the-workspace-sections-are-judged-at-the-outermost-root-of-a-full-checkout) says. One render answers `init`, `init --check`, `check`, the editor's report and `config migrate`'s guidance gate ([§FS-config-migrate.5.3](FS-config-migrate.md#53-guidance)).

A block is unsupported only when this grund cannot handle it; one `grund init` can rewrite is stale. So `check` judges a block's version against the maximum this binary supports, not the version the config selects:

- **Below the selected version**, the block is `outdated` ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)), and one `grund init` re-renders it, preserving unmanaged content.
- **Above the maximum**, it is **unsupported**. `check`'s message names that maximum, the number `grund init` names when it refuses such a block ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)), never the selected version ([§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages)): the remedy it implies is a newer grund, and the two commands name one limit for one block ([§FS-init.requirements.clean](FS-init.md#requirementsclean-what-init-writes-check-accepts--realized)).
- **At or below the maximum and above the selected version**, it is compared section by section like a block at the selected version, and every section that differs is reported stale ([§FS-check.3.5.3](FS-check.md#353-code-and-message)). Where no compared section tells it apart, the one stale finding names the section the selection turns on: `scoped guidance` across the scoped and unscoped pairs — a v49 or v50 block where v47 or v48 is selected; `chapter rules` across a pair's rule-kind member, which renders `### Chapter rules` — a v48 block where v47 is selected, or a v50 one where v49 is; and where both differ — a v50 block where v47 is selected — `scoped guidance`, as one finding.

Each is an `agents-init` error; which template reports it, and the number that template names, are [§FS-errors.3.6](FS-errors.md#36-the-agents-init-messages)'s.

#### 2.3.8 Substituted content

Several things in the generated entrypoint are substituted in rather than fixed for its `vN`, so the file describes the repository it is in. The generated project name is the explicit `--name`, otherwise the `project_name` from a config at either supported location in the target, otherwise the resolved target's basename; it fills the scaffolding H1 written above the block whenever canonical `AGENTS.md` is created or forcibly regenerated ([§FS-init.2.3.10](FS-init.md#2310-writing-the-block)), while an ordinary update preserves the unmanaged H1. The rest comes from the `grund.toml` `init` leaves governing the target: an existing config in the target, or the defaults `init` is about to write. Neither the name nor the configuration is inherited from an ancestor when the target has no config.

From that config the block fills in the ID shape (`<KIND>-<NNN>-<slug>`, `<KIND>-<slug>`, …, derived from `[id].format`), one worked example citation, the `[id].section_separator`, the marker and `$$`-trigger from `[reference]` with the example ID the trigger sentence names, the `KIND ∈ {…}` set from the kinds (``no `KIND` is declared yet`` where there is none), the discovery rung's `--kind` example and the Project map ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)), a config-version-aware sentence on whether bare ID-shaped tokens count as citations, the tags `[schema.sources] fenced` lists in the marker bullet's last clause, and the inline citation style sentences ([§FS-inline-citation-style.5](FS-inline-citation-style.md#5-agent-facing-rendering)).

For version 1 it says that bare ID-shaped tokens are ignored under `[reference] strict = true` and also recognized as citations under `strict = false`, the latter advising to remove that compatibility override or set strict back to `true`. For version 2, a config produced by migration included, it says that bare ID-shaped tokens are ignored, only citations prefixed with the configured marker are recognized, and there is no opt-out ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)); it neither claims that `[reference] strict` is set nor advises changing it ([§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed)).

The worked example citation is written escaped, `<§>` around the configured marker ([§FS-check.2.3.1](FS-check.md#231-escaped-citation-resolves)), never as a live citation: its ID is deliberately no real declaration in the host repository, so a live marker would make the fresh block fail that repository's own `grund check` as a dangling citation wherever the entrypoint is in scan scope.

An existing `grund.toml` that fails to load is an error, exit `2` with the message and location `grund check` gives for the same file, never a silent fall back to defaults: a block rendered from defaults would describe a repository the user does not have, dropping exactly the guidance an unparseable key selects while `init` reported success.

The trigger sentence teaches what a grund-aware editor does ([§FS-lsp.1.4](FS-lsp.md#14-live-trigger-transform)): the configured trigger becomes the configured marker once a token with an ID's shape follows it, so the sentence names that following ID, never says the trigger alone becomes the marker, and says the token must have its kind's shape, never that the ID must be recognized or resolve ([§DF-trigger-sentence-names-the-id](../decisions/functional/DF-trigger-sentence-names-the-id.md#df-trigger-sentence-names-the-id-the-blocks-trigger-sentence-names-the-id-that-follows-the-trigger)). It illustrates with the trigger written straight before an example ID in one inline code span, which neither `grund fmt`'s trigger pass nor the editor rewrites ([§FS-citations.zones](FS-citations.md#zones-where-citation-text-is-never-rewritten)). The example ID is the effective `[id].format` of the kind init's guidance names ([§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names)), filled with that kind and the worked example's number and slug; a config with no citable kind fills the format with `FS`. Under the default trigger and the one kind `FS` with format `{kind}-{slug}`, the example is `$$FS-user-login`. In the compatibility mode of a version-1 config with `[reference] strict = false` — the whole mode, read from the switch the scanner reads, not per format — the sentence carries no example ID, neither a concrete one nor a `<KIND>` shape.

#### 2.3.9 Delimiters

The managed block is bounded by standard `BEGIN` / `END` HTML-comment delimiters ([§DF-managed-block-delimiters](../decisions/functional/DF-managed-block-delimiters.md#df-managed-block-delimiters-standard-beginend-delimiters-for-the-managed-agent-instructions-block)). Just inside the begin delimiter, an H2 heading, the block's **version heading**, carries the block version ([§FS-init.2.3.7](FS-init.md#237-the-block-version)), labelled as one:

```markdown
<!-- BEGIN GRUND MANAGED BLOCK -->
## Grounding with grund (block vN)
…
<!-- END GRUND MANAGED BLOCK -->
```

Both delimiter lines belong to the managed region: everything between them, delimiters included, is `init`'s to rewrite, and nothing outside them is. `init` writes the version heading as `## Grounding with grund (block vN)` in every block it renders — the root block, a standalone companion's and a scoped file's — labelling the number as the revision of the block's text, never a grund release ([§DF-managed-block-revision-label](../decisions/functional/DF-managed-block-revision-label.md#df-managed-block-revision-label-the-managed-block-heading-labels-its-number-as-the-blocks-revision)). Between the delimiters, the unlabelled `## Grounding with grund (vN)` is the version heading too, with the same version, so `init` and `check` read a block in either spelling wherever this specification names the version heading: its version is compared, reported outdated, or refused as newer the same way, and `init`'s re-render writes the labelled spelling. The delimiters never change.

`init` and `check` recognize a legacy block ([§FS-init.terms](FS-init.md#terms-terms)), which also ends at the end of the file, and on its next write `init` migrates it to the delimited form in place, reported `updated `, preserving its position and every byte outside it. Only the unlabelled spelling opens a legacy block, so a `## Grounding with grund (block vN)` heading outside the delimiters opens nothing, and the file has no block.

#### 2.3.10 Writing the block

A fresh `AGENTS.md` is the block preceded by a one-line scaffolding H1, `# {NAME} — agent instructions`. The H1 is unmanaged on the update path: `init` without `--force` rewrites the block but leaves the title alone, while `--force` rewrites the whole canonical file, H1 included ([§FS-init.3.4](FS-init.md#34-with---force)). A freshly created companion entrypoint is the managed block alone, with no H1. An existing entrypoint with no managed block has the current block appended after its content, with no H1: the host file owns its title.

An entrypoint with any supported block version, the current one included, in either spelling of its version heading ([§FS-init.2.3.9](FS-init.md#239-delimiters)), has the block re-rendered from the current effective `grund.toml` and compared with the bytes of the managed region on disk, delimited or legacy. When they differ, `init` replaces only those bytes, in place, leaving the block's position and all content before and after it untouched ([§FS-init.3.2](FS-init.md#32-user-authored-content-in-agent-entrypoints-is-preserved)), and reports `updated `. When the re-render is byte-identical, it writes nothing and reports `exists ` ([§FS-init.2.2](FS-init.md#22-stdout--stderr)), so re-running `grund init` on an already-current repository is a no-op on every file. A block in the unlabelled spelling is `updated ` once, to the labelled one, and `exists ` after. While an optional member is absent, the re-render keeps the block's `### Workspace members` section as it is on disk ([§FS-init.2.3.4](FS-init.md#234-managed-block-content-sections)).

`init` does not normalize line endings. The bytes outside the managed region are preserved, CRLF (`\r\n`) and lone-CR endings included; matching a delimiter or the version heading tolerates an optional trailing `\r` before the line end, so a CRLF-encoded file is read correctly; and the block is written with the LF endings the binary embeds. On a CRLF-encoded host file the result is mixed line endings inside the managed region and CRLF outside, on purpose: normalizing the rest of the file would break the guarantee that content outside the block is left alone.

If an agent entrypoint contains a newer block version than the running binary supports, in either spelling (`(block v99)` as `(v99)`), `init` exits `2` and writes nothing — neither that file nor any other entrypoint, `grund.toml`, nor the starter files `--scaffold` lays — because the version is read before the first write ([§FS-init.1.2.6](FS-init.md#126-entrypoint-blocks-are-read-before-any-is-written)).

#### 2.3.11 Malformed delimiters

A file whose delimiters are present but broken is **malformed**: a `BEGIN` delimiter with no `END` after it, an `END` with no `BEGIN` before it, more than one `BEGIN`, or a delimited region that contains no version heading in either spelling ([§FS-init.2.3.9](FS-init.md#239-delimiters)). `init` refuses to touch a malformed file ([§DF-managed-block-delimiters](../decisions/functional/DF-managed-block-delimiters.md#df-managed-block-delimiters-standard-beginend-delimiters-for-the-managed-agent-instructions-block)): it exits `2` with a message naming the file and the specific defect, and rewrites nothing, in that file or any other the run planned ([§FS-init.1.2.6](FS-init.md#126-entrypoint-blocks-are-read-before-any-is-written)); `grund check` reports the same defect as an error ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). Fixing the delimiter pair by hand, or deleting the remnants of the block, and re-running `grund init` is the recovery path.

#### 2.3.12 Supported agents and their entrypoints

`AGENTS.md` is the canonical fallback entrypoint. The supported-agent set is fixed and is the superset every agent-facing feature uses: AGENTS-compatible agents (including Codex), Claude Code, Gemini, Pi, GitHub Copilot, Cursor, Windsurf, and Zed. The built-in companion set covers their common root or repository instruction files: Codex override instructions (`AGENTS.override.md`), Claude Code (`CLAUDE.md`, `.claude/CLAUDE.md`), Gemini (`GEMINI.md`), Pi (`.pi/AGENTS.md`), GitHub Copilot (`.github/copilot-instructions.md`), Cursor (`.cursor/rules/grund.mdc`, plus the legacy `.cursorrules`), Windsurf (`.windsurfrules`), and Zed (`.rules`). `init` updates the companions a repository already has, and creates one per agent ([§FS-init.2.1.1](FS-init.md#211-one-entrypoint-per-agent)) only where [§FS-init.2.1.2](FS-init.md#212-automatic-entrypoint-selection) and the agent flags ([§FS-init.1.5](FS-init.md#15-agent-entrypoint-flags)) say. A companion that is a symlink to `AGENTS.md` is not touched separately: an explicit request for it selects canonical `AGENTS.md` as the update target, and `grund check` always takes the canonical block as sufficient for the symlink ([§FS-check.3.5](FS-check.md#35-invalid-agent-entrypoint-init-block)). Under scoped guidance a destination directory carries `AGENTS.md` and mirrors only the `CLAUDE.md` and `GEMINI.md` symlinks ([§FS-init.2.3.5](FS-init.md#235-citation-directions)); which agents load such a nested file by themselves, and what the others are told, is [§DF-scoped-guidance.loading](../decisions/functional/DF-scoped-guidance.md#loading-which-agents-load-a-scoped-file-by-themselves).

### 2.4 Generated `grund.toml`

`init` writes `<path>/grund.toml` — the bare, root-visible form — and **only when the target has no config at all**, in either discovery location ([§FS-init.2.4.1](FS-init.md#241-both-discovery-locations-are-probed)). An existing config is never overwritten, not even with `--force` ([§FS-init.2.4.2](FS-init.md#242-an-existing-config-is-never-overwritten)). The file it does write is the template's config ([§FS-init.1.6](FS-init.md#16-templates)): a version-2 file ([§FS-config-v2](FS-config-v2.md#fs-config-v2-grund-reads-a-version-2-config-by-concern-with-one-strength-vocabulary-and-fixed-defaults)) that holds the project's own choices and nothing `grund` would assume anyway ([§FS-init.2.4.3](FS-init.md#243-every-written-key-differs-from-its-default)), while `grund config show` prints what applies and `grund config explain` every setting ([§FS-config.4.2.3.1](FS-config.md#4231-what-sends-a-reader-to-show-names-explain)). [§FS-init.2.4.4](FS-init.md#244-no-section-switch-is-written) to [§FS-init.2.4.8](FS-init.md#248-values-are-not-enabled) and [§FS-init.2.4.10](FS-init.md#2410-req-is-declared-in-docsrequirements) say how particular settings are written or left out, and [§FS-init.2.4.9](FS-init.md#249-the-none-templates-config) what the `none` template writes instead. The starter is written fresh rather than migrated from the version-1 file earlier releases wrote, which `grund config migrate` would refuse as it stands ([§DF-v2-onboarding](../decisions/functional/DF-v2-onboarding.md#df-v2-onboarding-grund-init-writes-a-minimal-version-2-file-and-grund-config-show-does-the-teaching)).

#### 2.4.1 Both discovery locations are probed

Both discovery locations of [§FS-config.1](FS-config.md#1-file-location-and-discovery) are probed, so a repository already configured under `.agents/grund.toml` is reported `exists .agents/grund.toml` and never grows the redundant pair [§FS-check.4.3](FS-check.md#43-redundant-config-pair) warns about. That probe matters more under the [§FS-config.1.1](FS-config.md#11-when-one-directory-carries-both) tie-break than a same-name check would: writing a bare `grund.toml` beside an existing `.agents/` config would *take over* the repository's grammar, which is exactly what `init` must never do to a config it did not write. Which form a project uses stays its own choice; the bare one is what `init` recommends, by generating it ([§DF-config-file-location.2.3](../decisions/functional/DF-config-file-location.md#23-grund-init-writes-the-bare-grundtoml)).

#### 2.4.2 An existing config is never overwritten

An existing config is the repo's configuration — the one surface a project customizes ([§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable)) — and `init` never overwrites it, not even with `--force`: an existing config is reported as `exists` and left byte-for-byte unchanged ([§FS-init.3](FS-init.md#3-non-intrusive-guarantees)). `--force` resets the things `init` owns end to end — the canonical `AGENTS.md` and the starter files `--scaffold` lays — not the user's settings; a customized config that `init --force` clobbered would be a footgun.

#### 2.4.3 Every written key differs from its default

When the file is absent and `init` does write it, the file is version 2 ([§FS-config.5](FS-config.md#5-schema-versioning)), and every key it writes either differs from its version-2 default ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)) or has none. v2 has no built-in kinds and no default titles, so a row, its places and its title are written because without them the file declares nothing ([§FS-config-v2.schema.places](FS-config-v2.md#schemaplaces-rows-and-their-places)). The file is therefore minimal: taking any written key or table out of it changes the effective configuration. It is an override surface, not a teaching surface; `grund config show` is the teaching surface, and prints the settings in effect that the file leaves at their defaults, in the file's own version ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)), while `grund config explain` answers for an opt-in table that is off ([§FS-config.4.2.3.1](FS-config.md#4231-what-sends-a-reader-to-show-names-explain)).

What is not written follows from that. No `[schema.sources]` and no `[schema.kinds.code]` is written, so a new project scans v2's default extent: the rows' places and the config root, less what `.gitignore` excludes ([§FS-config-v2.schema.places.complement](FS-config-v2.md#schemaplacescomplement-code-is-the-complement-by-name)). No `[schema] id_format` is written, so a new project takes v2's slug-only format `{kind}-{slug}`: `grund id FS "Login"` allocates `FS-login`, and the starter files follow that format ([§FS-init.2.1.3](FS-init.md#213-the-software-development-starter-files)). No `[rules]` table is written, so no citation direction is enforced until the project adopts one.

The `software-development` file is exactly these bytes, with `{NAME}` the project name of [§FS-init.2.4.7](FS-init.md#247-project_name-and-presentation-description):

```toml
# Only non-defaults; `grund config show` prints what applies, `grund config explain` every setting.
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

The file carries one comment, its first line, which says where the rest of the settings are, in the sentence of [§FS-config.4.2.3.1](FS-config.md#4231-what-sends-a-reader-to-show-names-explain):

```toml
# Only non-defaults; `grund config show` prints what applies, `grund config explain` every setting.
```

No key carries a value-set comment and no commented-out key is written: a minimal file has no default-valued key to annotate. The commented citation ruleset and the commented `CONST` row of earlier starters are dropped; `grund guide citations` prints the citation-directions guide ([§FS-init.5](FS-init.md#5-the-guide)). The `none` template keeps one more comment, which says how to declare a first kind ([§FS-init.2.4.9](FS-init.md#249-the-none-templates-config)). Comments are not schema keys and do not change parsing.

#### 2.4.6 `shorthand` is left at its default

No `[schema] shorthand` is written: the canonical policy is the v2 default, and `grund config show` prints it ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). The opt-in `accepted` policy is a choice a project writes for itself ([§FS-config.3.1](FS-config.md#31-reference--citation-form)).

#### 2.4.7 `project_name` and `[presentation] description`

`project_name = "<name>"` appears right below `grund_config_version = 2`, above every table. `init` consumes it as the generated project-name default when no explicit `--name` is supplied ([§FS-init.2.3.8](FS-init.md#238-substituted-content)); no other non-workspace subcommand consumes it. It also remains metadata that downstream tooling (IDE status bars, CI dashboards) can read without re-deriving the name. When `--description` was given ([§FS-init.1](FS-init.md#1-inputs)), the file also writes the description at its v2 home, a `[presentation]` table holding `description = "<text>"`, the v2 spelling of v1's `project_description` ([§FS-config-v2.presentation](FS-config-v2.md#presentation-the-presentation-concern)). The table stands right above the first `[presentation.kinds]` table, or at the end of a file that has none. Without `--description` neither the key nor a commented placeholder is written.

#### 2.4.8 Values are not enabled

The starter declares no value kind and illustrates none. Fresh repositories remain unopted-in, and the effective schema contains no `value_sources` key ([§FS-config.3.4.9](FS-config.md#349-values--first-class-value-declarations)).

#### 2.4.9 The `none` template's config

The `none` template ([§FS-init.1.6.2](FS-init.md#162-none)) writes the envelope alone: the comment of [§FS-init.2.4.5](FS-init.md#245-one-comment-is-kept), `grund_config_version = 2`, `project_name`, and one comment saying how to declare a first kind. With no `[schema.kinds]` row the file declares no kinds and still loads ([§FS-config-v2.defaults.1](FS-config-v2.md#defaults1-what-v2-fixes)). It is exactly these bytes:

```toml
# Only non-defaults; `grund config show` prints what applies, `grund config explain` every setting.
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
- It creates nothing under any kind home, and nothing under `docs/` or `tests/`: no starter file, no directory, no `.gitkeep`. The one exception is scoped guidance ([§FS-init.2.3.5.11](FS-init.md#23511-scoped-guidance-a-rows-guidance-lands-above-its-places)): the managed scoped `AGENTS.md`, and the companion symlinks mirrored beside it, written only into a destination directory that already exists ([§FS-init.2.3.5.11.4](FS-init.md#235114-a-destination-renders-at-the-root-until-its-directory-exists-or-the-scaffold-lays-it)). That is what makes it safe on a repository that already has specs, and what the `grund-init` skill relies on when it leaves the scaffold off an existing repository ([§DF-skill-init-existing-specs](../decisions/functional/DF-skill-init-existing-specs.md#df-skill-init-existing-specs-grund-init-adopts-existing-specs-before-scaffolding)).
- It prints the `next:` block ([§FS-init.2.2.2](FS-init.md#222-the-next-block)), whose guidance names the kind and home [§FS-init.2.2.2.3](FS-init.md#2223-the-kind-the-guidance-names) selects: its first step points at `--scaffold`, or at creating the homes by hand, only where that selection is FS with a home, and otherwise at creating the selected home by hand while it is missing.

## 4. Exit codes

- `0` — no rule finding was reported and every requested file was written, appended, updated, or already current; a `--dry-run` preview may still report pending paths ([§FS-init.4.3](FS-init.md#43-why---dry-run-keeps-0)). Under `--check` ([§FS-init.1](FS-init.md#1-inputs)), there is no rule finding and nothing pending.
- `1` — a rule finding was reported ([§FS-rules.4](FS-rules.md#4-validation-lifecycle), [§FS-rules.4.1.2](FS-rules.md#412-what-init-writes-anyway)), independently of the path events; otherwise, `--check` reported at least one pending path ([§FS-init.4.1](FS-init.md#41-what-earns-the-1)).
- `2` — I/O error (target path does not exist, permission denied, disk full, etc.); a CLI-level error such as an unknown flag; a refused target ([§FS-init.1.2](FS-init.md#12-refused-targets)), in which case nothing was written; an existing config that fails to load ([§FS-init.2.3.8.3](FS-init.md#2383-a-config-that-fails-to-load)); a refused template, an unknown name or `none` beside `--scaffold` ([§FS-init.1.6.3](FS-init.md#163-an-unknown-name), [§FS-init.1.6.4](FS-init.md#164-none-has-nothing-to-scaffold)), in which case nothing was written; an existing agent entrypoint contains a managed block whose schema version is newer than the running binary supports ([§FS-init.2.3.10.2](FS-init.md#23102-a-newer-block-version)); or an existing agent entrypoint has malformed delimiters ([§FS-init.2.3.11](FS-init.md#2311-malformed-delimiters)) — in both of those the file is left unchanged. `2` wins over `1` ([§FS-init.4.2](FS-init.md#42-2-wins-over-1)).

Rule findings take precedence over the pending-path check, including under
`--dry-run` ([§FS-init.4.3](FS-init.md#43-why---dry-run-keeps-0)). Exit-code mapping is fixed per [§GOAL-friendliness-first.2](../goals.md#2-what-this-rules-out) and [§FS-non-goals.9](FS-non-goals.md#9-severity-exit-code-or-report-ordering-customization).

### 4.1 What earns the `1`

Rule findings earn `1` in ordinary, `--check` and `--dry-run` runs. Their
validation and write consequences are defined by [§FS-rules.4](FS-rules.md#4-validation-lifecycle) and
[§FS-rules.4.1.2](FS-rules.md#412-what-init-writes-anyway): an invalid rule withholds the write; an unverifiable rule is
reported and rendered as authored. The latter still earns `1` when the block
was written, updated or already current, including a current `--check` run.
`grund check` from the workspace root can judge the alias-pinned rule with the
full vocabulary ([§FS-rules.4.1](FS-rules.md#41-a-rule-this-scope-cannot-verify)).

Independently, under `--check`, the pending-path events `would-write `,
`would-append ` and `would-update ` earn `1` (as do pending scoped-file events
defined by [§FS-init.2.3.5.14.4](FS-init.md#235144-the-report-names-every-scoped-file-and-link-it-touched)). Nothing was written, and the run that refreshes
those paths is the same `init` without the flag. A `note:` never earns `1` and
neither does the `next:` block — a note is a report, not a finding
([§FS-init.2.2.1](FS-init.md#221-notes)).

Help and user guidance must explain both causes and the existing reports that
distinguish them: located rule errors are on stdout; path events are on stderr
([§FS-init.2.2](FS-init.md#22-stdout--stderr)). A rule finding and pending events can coexist. A rule finding
alone says nothing about pending paths; `exists` or `updated` alone says
nothing about other paths or rule findings. In particular, a report containing
only current path events can accompany a rule finding and exit `1`, so the exit
alone cannot serve as a member's staleness gate. No new report format is required.

### 4.2 `2` wins over `1`

A run that could not be performed produced no report to gate on, so `--check` over a refused target exits `2` the way any other form of the run does.

### 4.3 Why `--dry-run` keeps `0`

With no rule finding or operational failure, `--dry-run` keeps `0` even when
paths are pending, while `--check` earns `1` for those paths. This asymmetry is
deliberate: `--dry-run` asks *what would this run do*, `--check` asks *is there
anything left to do*. Making pending paths alone fail a preview would change
an existing green CI step's verdict on upgrade ([§REQ-backwards-compatibility.1](../requirements/REQ-backwards-compatibility.md#1-what-is-covered)).
Both forms still use the findings slot of [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed) for rule errors
([§FS-init.4.1](FS-init.md#41-what-earns-the-1)); `--dry-run` is not an unconditional-success escape from the
rule verdict preserved by [§DF-unverifiable-rule-scope.2.2](../decisions/functional/DF-unverifiable-rule-scope.md#22-unverifiable-is-a-verdict-the-parser-reaches-not-a-judgement).

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

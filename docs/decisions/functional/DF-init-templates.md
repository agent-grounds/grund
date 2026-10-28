# DF-init-templates: `grund init` lays a named template, and `--scaffold` replaces `--docs`

**Date:** 2026-10-12

## status: Accepted

## 1. Context

`grund init` answers two separate questions, and one flag named after a directory stood in for the second. The first is *which template*: what config and which starter files to lay. The second is *how much of it*: only what makes a repository checkable and tells its agents about grund, or the starter files as well. `--docs` answered the second, but half of what it laid is not under `docs/`, and bare `init` was defined only as "without `--docs`" ([§FS-init.2.1](../../functional-spec/FS-init.md#21-files-written-updated-or-left-in-place)). A repository that declares its own kinds had no template at all: it got the nine defaults and had to delete them, because grund could not express a config with no kinds — `kinds = []` was refused as an unknown key, and omitting `[[kinds]]` keeps the built-in set ([§FS-config.3.4.4](../../functional-spec/FS-config.md#344-the-default-kinds)). Tracked in agent-grounds/grund#653, whose model this record implements.

## 2. Decision

`init` takes `--template <name>` and `--scaffold` ([§FS-init.1.6](../../functional-spec/FS-init.md#16-templates)). Two templates are built in, `software-development` (the default) and `none`, and bare `init` is a point of its own ([§FS-init.3.6](../../functional-spec/FS-init.md#36-bare-init-writes-the-config-and-the-entrypoints-and-nothing-in-a-home)).

### 2.1 Two flags, not `--scaffold[=<path>]`

The two questions are independent. A template carries a config as well as starter files, and `--template <name>` without `--scaffold` is the existing-repository case: take the template's kinds and grammar, lay nothing over the docs that are there. `--template=none` is that case taken all the way. An optional-value `--scaffold` could say neither, and it would clash with `init`'s positional `PATH`, since `grund init --scaffold foo` must still scaffold into `foo`. `--template` always takes a value, so both `--template none` and `--template=none` work, and template paths can be added later without a break.

### 2.2 `--docs` warns until 0.19.0, on stderr alone

`--docs` stayed for one release as a deprecated alias under [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path), and was removed in `0.19.0` ([§FS-init.1.7](../../functional-spec/FS-init.md#17---docs-is-the-deprecated-spelling-of---scaffold)). `--scaffold` shipped in `0.18.0`, so `0.19.0` was the release after the one that introduced it, and a release guard that refuses a pending removal shipped at its own deadline ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)) could never have met it on the release that introduced it. It was the release the other open removals named, so a repository migrated them together. The window closed as planned: from `0.19.0`, `--docs` is an unknown flag, and [§DF-docs-flag-removed-in-0-19](DF-docs-flag-removed-in-0-19.md#df-docs-flag-removed-in-0-19-grund-init---docs-is-an-unknown-flag-from-0190) carries the notice of the release that removed it.

The warning went to stderr and nowhere else. `init` has no `--format json`: its report is the transcript on stderr ([§FS-errors.6](../../functional-spec/FS-errors.md#6-the-grund-init-transcript)), and that is where the warning landed, before the report. The bindings have no command-line spelling to deprecate ([§DF-init-templates.2.3](DF-init-templates.md#23-the-bindings-keep-docs)), so no JSON result carried it.

### 2.3 The bindings keep `docs`

`InitOpts.docs`, the guidance's `InitNext.docs`, the Python and Node `docs` option, and the `next.docs` field of the init JSON result keep their name. They are the published surface of `grund-core` and of both bindings ([§FS-distribution.3.1](../../functional-spec/FS-distribution.md#31-rust-grund-core-crate)), and a rename there is an API break that would need a deprecation path of its own across three languages, to rename a field whose meaning has not moved: it still says the run laid the starter files. The CLI flag was renamed because its name taught the wrong thing to a new user at the first command they read; the field is read by code that already knows what it means.

Two additive changes reach that surface. `InitOpts` gains `template`, the explicit `--template` value or none, so `grund-core` decides the template vocabulary and its refusals in one place. The guidance gains `declares_kinds`, which the `next:` block of a project with no kinds is drawn from ([§FS-init.2.2.2.2](../../functional-spec/FS-init.md#2222-a-project-with-no-kinds)); the bindings carry it as `declares_kinds` and `declaresKinds`. The bindings take no `template` option yet: they lay the default template as before, and the option is one more additive field when it is wanted.

### 2.4 `kinds = []` is the empty kind set

A version-1 config writes `kinds = []` to declare no kinds ([§FS-config.3.4.4.5](../../functional-spec/FS-config.md#3445-kinds---declares-no-kinds)). It is the natural TOML for an empty array of the tables `[[kinds]]` writes, it is distinct from omitting the key, which keeps its compatibility meaning, and it needs no new schema version: no file that loaded before means anything different now, because the key was refused. It matches version 2, where a file with no `[schema.kinds]` row has no kinds ([§FS-config-v2.defaults.1](../../functional-spec/FS-config-v2.md#defaults1-what-v2-fixes)), so a later migration maps it to no rows at all.

What follows from no kinds is chosen so that a fresh `grund init --template=none` tree passes `grund check` and `grund init --check`, and so that every surface that would have named an empty set teaches the first kind instead:

- **Scan.** `[scan] include` alone, because no home adds to it; the template writes the default list.
- **`check`.** No marked token is a citation, since an ID opens with a configured kind, so nothing is reported dangling; the run-level cautions say the project declares no kinds rather than listing `{}` ([§FS-citations.recognized.no-kinds](../../functional-spec/FS-citations.md#recognizedno-kinds-with-no-kind-no-token-is-a-citation), [§FS-check.4.5.2](../../functional-spec/FS-check.md#452-the-message)). Reporting every marked token as dangling was rejected: with no kind there is no grammar to say where such a token ends, and a project that has configured some kinds already treats a token of an unconfigured one as text.
- **`grund id`.** The `known kinds:` line reads `none — declare one in grund.toml first` ([§FS-list.1.1.1](../../functional-spec/FS-list.md#111-a-scope-with-no-citable-kind)), in the one renderer every refusal shares.
- **Managed block.** The Project map is a paragraph teaching the first `[[kinds]]` table, the kind set and the citation-direction paragraph say no kind is declared yet ([§FS-init.2.3.4.4.2](../../functional-spec/FS-init.md#23442-no-kinds-yet)), and none of it moves the block version, because it is substituted content. The citation-direction paragraph has since gone, with the section every config without direction rules leaves out ([§DF-directions-only-from-rules](DF-directions-only-from-rules.md#df-directions-only-from-rules-the-citation-directions-section-renders-only-the-rules-a-config-declares)).

### 2.5 One directory per template

The embedded templates move into one directory each: `crates/grund-core/assets/templates/software-development/` holds the config and the ten starter files, `none/` holds its config, and the managed block's `AGENTS.md`, which belongs to no template, stays at the top. The reference copy under `templates/` has the same shape, and the asset-sync test compares the two trees recursively. A template from a path is expected to be a directory shaped like these, which is the question the follow-up ticket for `--template <path>` settles.

## 3. Consequences

- `--help`, the `next:` block, the empty-scan warning, the README, the specs and the shipped `grund-init` skill say `--scaffold`; `--docs` survives in the deprecation note and in archived changelogs.
- The `grund-init` skill's adoption table names `--template=none` for the existing-structure model ([§DF-skill-init-existing-specs.3](DF-skill-init-existing-specs.md#3-artifact-model-prompt)).
- A Rust caller that builds `InitOpts` or `InitNext` with an exhaustive struct literal must add the new field; one that starts from `InitOpts::default()` is unaffected.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| `--scaffold[=<path>]` | Cannot say "this template's config, no starter files", and clashes with the positional `PATH`. |
| Rename the bindings' `docs` as well | An API break across Rust, Python and Node for a field whose meaning did not change. |
| Omit `[[kinds]]` to mean no kinds | It already means the built-in kinds with the older `FS` home, and existing configs rely on it. |
| A `none` template that writes no `grund.toml` | The repository would get the built-in kinds the moment it ran `grund check`, which is the opposite of what it asked for. |

## release-note: Release note

- [§FS-init.1.6](../../functional-spec/FS-init.md#16-templates), [§FS-init.1.7](../../functional-spec/FS-init.md#17---docs-is-the-deprecated-spelling-of---scaffold), [§FS-config.3.4.4.5](../../functional-spec/FS-config.md#3445-kinds---declares-no-kinds), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **`grund init --scaffold` replaces `grund init --docs`, and `--template <software-development|none>` picks what `init` lays.** `--docs` still works and lays the same files, after a stderr warning that it is removed in `0.19.0`. `--template=none` writes a `grund.toml` with `kinds = []`, the new spelling of a config that declares no kinds, which earlier releases refused as an unknown key. **Who this affects:** a script or CI step that runs `grund init --docs` sees one new stderr line and nothing else until `0.19.0`; spell it `--scaffold`. A Rust caller that builds `InitOpts` or `InitNext` with an exhaustive struct literal adds `template` or `declares_kinds`. Closes [issue #653](https://github.com/agent-grounds/grund/issues/653). (PR #663)

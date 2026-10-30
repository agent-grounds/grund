# DF-imports: an import is a named input read live from a sibling tree, never a remote

**Date:** 2026-10-29

## status: Accepted

## 1. Context

A build may already place several repositories side by side at the revisions it chose:
`checkout/graal/substratevm/` holds the `native-image` project and `checkout/enterprise/`
cites `<§>native-image/FS-security-providers` from its own source. Nothing above the two is
configured, and neither repository owns the directory that holds them. Today the consumer
cannot reach the dependency from its own config: a `..` member is refused, because a member
stays inside its root ([§FS-workspace.2.3](../../functional-spec/FS-workspace.md#23-what-a-member-entry-may-name)), a citation with no workspace is an unknown alias,
and a remote ([§DF-remote-projects](DF-remote-projects.md#df-remote-projects-a-remote-project-is-a-workspace-member-whose-bytes-were-fetched-mounted-as-its-own-root)) would pin and commit a second copy of a tree
the build already placed, under a second revision mechanism (agent-grounds/grund#1080).

The bright line is [§FS-non-goals.13](../../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree): two installs given the same input never
disagree. It never meant that a verdict depends on nothing outside the checkout:
[§REQ-deterministic-output.2](../../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation) already names inputs that reach past it, and
[§FS-config.requirements.8](../../functional-spec/FS-config.md#requirements8-a-projects-meaning-is-self-contained--realized) lists them.

## 2. Decision

### 2.1 An import is one more named input

A config may declare `[workspace.imports.<alias>]` with a `path` and the `project` it
expects there ([§FS-imports.declaration.table](../../functional-spec/FS-imports.md#declarationtable-the-import-table)). The tree at that path is read live,
read-only and offline, as a mounted root under the alias
([§FS-imports.mount](../../functional-spec/FS-imports.md#mount-an-import-is-a-mounted-root-read-live-and-read-only)). It is named as input where inputs are named, and only for a config
that declares one: the tree of [§REQ-deterministic-output.2](../../requirements/REQ-deterministic-output.md#2-input-is-the-tree-the-config-and-the-invocation) includes it, and so does
the list of [§FS-config.requirements.8](../../functional-spec/FS-config.md#requirements8-a-projects-meaning-is-self-contained--realized). Two installs given the same composition still
agree, so [§FS-non-goals.13](../../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree) holds word for word and gets no exception.

### 2.2 An import is not a remote

An import claims no pin. It can neither replace nor shadow a declared remote: an import alias
that names a remote is an alias collision ([§FS-imports.declaration.alias](../../functional-spec/FS-imports.md#declarationalias-the-alias-is-the-table-key)). Compared with a
pinned remote, an import does not carry:

- reproduction from the consumer commit alone: the consumer commit no longer decides the
  verdict by itself, and `check` says so once per import
  ([§FS-imports.checks.import-live](../../functional-spec/FS-imports.md#checksimport-live-an-import-is-read-live-unpinned));
- a lock;
- an integrity proof;
- a freshness check.

### 2.3 What an absent import gives

A missing import is an error, never a skipped namespace and never a fallback to another tree
([§FS-imports.checks.import-missing](../../functional-spec/FS-imports.md#checksimport-missing-an-import-names-no-directory)). A config with imports is verifiable offline once
the named trees are present; without one the run fails at the import and never reports a
pass.

## 3. Consequences

- The contract landed ahead of its release: this decision, [§FS-imports](../../functional-spec/FS-imports.md#fs-imports-a-project-cites-a-sibling-project-its-build-places-beside-it-read-live-from-that-tree), and one
  sentence each in the points that name a run's inputs. The command line now reads
  `[workspace.imports]`; the editor's half ([§FS-imports.editor](../../functional-spec/FS-imports.md#editor-the-editor-reads-the-import-graph-the-command-line-reads)) is still ahead of it.
- Imports reuse the mounted-root resolver [§FS-remote-projects.mount.rule](../../functional-spec/FS-remote-projects.md#mountrule-a-mounted-root-resolves-natively-and-is-shown-under-its-alias) specifies, so
  the layer that reads a projection later is the layer an import reads with now; reading
  live is a property of the import edge, not of the layer.
- `members` keeps its containment, `mounted_members` and `[workspace.remotes]` keep their
  refusals, and a config with no imports reads nothing new
  ([§FS-imports.declaration.none](../../functional-spec/FS-imports.md#declarationnone-a-config-with-no-imports-reads-nothing-new)).
- An adopter never sees `success`, CI must place every imported tree or the run exits `1`,
  and a link `fmt` writes into an import leaves the repository.

## 4. Alternatives considered

| Approach | Why rejected |
|---|---|
| A shared parent workspace above both repositories | Its config is owned by neither repository, and a run from the consumer alone does not obtain it. |
| A `..` entry in `members` | A member's names change with whoever is outermost ([§DF-remote-projects](DF-remote-projects.md#df-remote-projects-a-remote-project-is-a-workspace-member-whose-bytes-were-fetched-mounted-as-its-own-root)), and a member stays inside its root. |
| A committed local `source` on a remote | Refused by [§DF-remote-projects.2.3](DF-remote-projects.md#23-what-stays-refused): colleagues and CI could not reproduce the pin. |
| An exception to [§FS-non-goals.13](../../functional-spec/FS-non-goals.md#13-anything-that-would-let-two-grund-installs-disagree) for configs with imports | Ruled out: it says more than the feature needs, and naming the input keeps the line whole. |

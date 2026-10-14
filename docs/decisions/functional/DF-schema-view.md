# DF-schema-view: the schema view is its own command, prints no JSON yet, shows a workspace's root, and draws an any-of entry through one junction

**Status:** Accepted
**Date:** 2026-10-14

## 1. Context

A repository's schema and rules are spread over a `grund.toml` that `grund config
show` prints back as about sixty lines of TOML, rules sorted alphabetically and far
from the kinds they govern, an any-of entry indistinguishable from any other
string. agent-grounds/grund#463 asked for a quick view of the same facts that needs
no browser, and [§FS-schema](../../functional-spec/FS-schema.md#fs-schema-grund-schema-shows-what-a-repository-is-held-to-read-from-its-configuration-alone) specifies it. Four choices shaped it, each against a
real alternative; the author approved them as proposed on the issue.

## 2. Decision

### 2.1 A standalone command, not printers on `config show`

`grund schema` is its own subcommand ([§FS-schema.1](../../functional-spec/FS-schema.md#1-the-command)). `config show` answers what a
TOML evaluates to, and [§FS-config.4.2](../../functional-spec/FS-config.md#42-grund-config-show-path) keeps its bytes; hanging a picture on it
would give one command two questions to answer. The #347 visualization prototypes
proposed `config show --format mermaid`; that proposal is superseded.

### 2.2 No `--format json` yet

A JSON shape is a covered, stable contract ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)), and
the record it would freeze is still moving: multi-place rows (#457) and the agent
guidance that will read the same record (#460) are both open. A caller that needs
machine-readable facts has `config show`, whose TOML loads back to the same
values. `--format json` is refused like any unknown value ([§FS-schema.13](../../functional-spec/FS-schema.md#13-what-is-absent)).

### 2.3 A workspace root shows the root, and names its members

At a workspace root the view is the root project, with one line naming the members
and how to see each ([§FS-schema.2.1](../../functional-spec/FS-schema.md#21-a-workspace-root-shows-the-root-and-names-its-members)). Rendering every member in turn has no single
meaning for a Mermaid or DOT consumer, which expects one graph, and would load every
member's configuration for a view meant to be quick.

### 2.4 An any-of entry is one junction

In the graph formats an any-of entry is drawn through one junction node: one line
from the subject, one arrow to each alternative ([§FS-schema.5.1](../../functional-spec/FS-schema.md#51-an-any-of-entry-is-one-obligation)). A set of labelled
edges, one per alternative, draws two arrows that read as two requirements, which is
exactly the misreading the view exists to prevent.

## 3. Consequences

- The subcommand surface grows by one verb, `schema`, which takes the bare word from
  the default ID query by the precedence already written ([§FS-cli.requirements.as-written](../../functional-spec/FS-cli.md#requirementsas-written-an-invocation-runs-as-written-or-is-refused--realized-directional-for-the-next-subcommand)).
- The view and `grund check` read one record, so the view cannot describe a schema
  the checks do not enforce ([§REQ-one-answer](../../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)).
- A later `--format json`, a per-member workspace view, or a different drawing of
  any-of is a new decision, not a quiet change.

## 4. Alternatives considered

- **Printers on `config show`.** Rejected in [§DF-schema-view.2.1](DF-schema-view.md#21-a-standalone-command-not-printers-on-config-show).
- **`--format json` now.** Deferred in [§DF-schema-view.2.2](DF-schema-view.md#22-no---format-json-yet) until #457 and #460 settle
  the record.
- **Every member of a workspace in turn.** Rejected in [§DF-schema-view.2.3](DF-schema-view.md#23-a-workspace-root-shows-the-root-and-names-its-members).
- **Labelled edge sets for any-of.** Rejected in [§DF-schema-view.2.4](DF-schema-view.md#24-an-any-of-entry-is-one-junction).
- **Fields as Mermaid subgraphs.** Rejected: the #347 prototypes showed subgraphs
  hiding edges behind neighbouring boxes, so fields are lines inside their kind's
  node ([§FS-schema.4.1](../../functional-spec/FS-schema.md#41-fields-are-lines-inside-their-kinds-node)).

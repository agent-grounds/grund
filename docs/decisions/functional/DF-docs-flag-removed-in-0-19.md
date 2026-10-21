# DF-docs-flag-removed-in-0-19: `grund init --docs` is an unknown flag from 0.19.0

**Date:** 2026-10-21

## status: Accepted

## 1. Context

[§DF-init-templates.2.2](DF-init-templates.md#22---docs-warns-until-0190-on-stderr-alone) renamed `grund init --docs` to `--scaffold` in two releases: 0.18.0 kept
`--docs` as a deprecated spelling that laid the same files after one stderr warning naming
0.19.0, and 0.19.0 removes it. That record's notice shipped with 0.18.0, and a notice already
released is not published again ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that removes the
spelling needs a notice of its own, and this record carries it.

## 2. Decision

In 0.19.0 (agent-grounds/grund#881):

- `--docs` is an unknown flag: `grund init --docs` prints ``error: unknown flag `--docs` ``,
  writes nothing and exits `2`, as every unknown flag does ([§FS-init.1.7](../../functional-spec/FS-init.md#17---docs-is-the-deprecated-spelling-of---scaffold),
  [§FS-init-fixtures.4](../../functional-spec/FS-init-fixtures.md#4-target-and-flag-failures));
- `grund init --help` names `--scaffold` alone ([§FS-cli.2.3](../../functional-spec/FS-cli.md#23-a-subcommands-help-page));
- the bindings keep their `docs` option, `InitOpts.docs` and `next.docs`, which were never a
  command-line spelling and were never deprecated ([§DF-init-templates.2.3](DF-init-templates.md#23-the-bindings-keep-docs)).

## 3. Rejected alternatives

- **Keep `--docs` for another release.** 0.18.0 printed 0.19.0 as the release that removes it,
  and the release guard refuses a version that reaches a promise it does not keep
  ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). Moving the date would make the warning 0.18.0 printed false.
- **Refuse `--docs` with a message naming `--scaffold`.** It would be a second refusal for one
  flag, kept forever; the unknown-flag refusal is the one every removed spelling gets, and the
  release note and [§FS-init.1.7](../../functional-spec/FS-init.md#17---docs-is-the-deprecated-spelling-of---scaffold) name the replacement.

## release-note: Release note

- [§DF-docs-flag-removed-in-0-19](DF-docs-flag-removed-in-0-19.md#df-docs-flag-removed-in-0-19-grund-init---docs-is-an-unknown-flag-from-0190), [§FS-init.1.7](../../functional-spec/FS-init.md#17---docs-is-the-deprecated-spelling-of---scaffold), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **`grund init --docs` is removed; spell it `--scaffold`.** What grund 0.18.0 announced lands: `--docs` is now an unknown flag, so `grund init --docs` prints ``error: unknown flag `--docs` ``, writes nothing and exits `2`, and `grund init --help` no longer mentions it. `--scaffold` lays the same starter files `--docs` did. **Who this affects:** a script or CI step that still runs `grund init --docs`, which saw the deprecation warning in 0.18.0: change it to `grund init --scaffold`. The Rust, Python and Node `docs` option keeps its name and is unaffected. Closes [issue #881](https://github.com/agent-grounds/grund/issues/881).

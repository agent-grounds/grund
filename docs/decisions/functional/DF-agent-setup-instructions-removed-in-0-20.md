# DF-agent-setup-instructions-removed-in-0-20: `grund agent-setup-instructions` is no command from 0.20.0

**Date:** 2026-10-22

## status: Accepted

## 1. Context

[§DF-guide-command](DF-guide-command.md#df-guide-command-grund-agent-setup-instructions-becomes-grund-guide-whose-bare-run-prints-a-front-page-and-whose-sections-are-named-positionally) renamed `grund agent-setup-instructions` to `grund guide` in two releases:
0.19.0 kept the old name as a deprecated one that printed the core, the text `grund guide setup`
prints, after one stderr warning naming 0.20.0, and 0.20.0 removes it. That record's notice
shipped with 0.19.0, and a notice already released is not published again
([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)), so the release that removes the name needs a notice of its own, and
this record carries it.

## 2. Decision

In 0.20.0 (agent-grounds/grund#930):

- `agent-setup-instructions` is no subcommand, hidden or listed: `complete` is the only hidden
  one left ([§FS-cli.1](../../functional-spec/FS-cli.md#1-the-default-subcommand), [§FS-init.5.7](../../functional-spec/FS-init.md#57-agent-setup-instructions-is-the-deprecated-name-of-guide));
- the name gets the refusal every first word that is neither a subcommand nor an ID gets
  ([§FS-cli.4](../../functional-spec/FS-cli.md#4-errors-with-no-source-location)). `grund agent-setup-instructions`, alone or with `--full`, is the failed ID query of
  [§FS-cli.1.2](../../functional-spec/FS-cli.md#12-a-first-word-that-is-not-an-id): ``invalid ID `agent-setup-instructions` `` and the format hint, an empty stdout and
  exit `1`. `grund help agent-setup-instructions` is
  ``error: unknown command: agent-setup-instructions``, exit `2` ([§FS-cli.1.3](../../functional-spec/FS-cli.md#13-a-help-request-with-an-unknown-first-word));
- the bindings keep `agent_setup_instructions()` and `agentSetupInstructions()`, which were never a
  command-line spelling and were never deprecated ([§DF-setup-guide-core-by-default.2.3](DF-setup-guide-core-by-default.md#23-the-bindings-return-the-whole-guide),
  [§DF-guide-command.3](DF-guide-command.md#3-rejected-alternatives)).

## 3. Rejected alternatives

- **Keep the name for another release.** 0.19.0 printed 0.20.0 as the release that removes it, and
  the release guard refuses a version that reaches a promise it does not keep
  ([§FS-distribution.4.2](../../functional-spec/FS-distribution.md#42-a-release-may-not-contradict-the-releases-the-trees-own-messages-name)). Moving the date would make the warning 0.19.0 printed false, which is
  why [§DF-docs-flag-removed-in-0-19.3](DF-docs-flag-removed-in-0-19.md#3-rejected-alternatives) kept `--docs` to its date.
- **Refuse the name with a message naming `grund guide`, exit `2`.** Bare `grund` lands that way
  ([§DF-bare-grund-lands-on-an-error](DF-bare-grund-lands-on-an-error.md#df-bare-grund-lands-on-an-error-bare-grund-lands-on-a-cli-level-error-not-the-top-level-help-page)), because its generic reading, the top-level help page, would
  have been a success, the silent no-op the window was bought to prevent. Here the generic reading
  already fails: a script still spelled the old name goes red either way. A dedicated refusal
  would be a second refusal for one removed word, kept forever, and the generic one is what every
  removed spelling gets ([§DF-docs-flag-removed-in-0-19.3](DF-docs-flag-removed-in-0-19.md#3-rejected-alternatives)). The release note and [§FS-init.5.7](../../functional-spec/FS-init.md#57-agent-setup-instructions-is-the-deprecated-name-of-guide) name
  the replacement.

## release-note: Release note

- [§DF-agent-setup-instructions-removed-in-0-20](DF-agent-setup-instructions-removed-in-0-20.md#df-agent-setup-instructions-removed-in-0-20-grund-agent-setup-instructions-is-no-command-from-0200), [§FS-init.5.7](../../functional-spec/FS-init.md#57-agent-setup-instructions-is-the-deprecated-name-of-guide), [§REQ-backwards-compatibility.2](../../requirements/REQ-backwards-compatibility.md#2-the-deprecation-path): **`grund agent-setup-instructions` is removed; run `grund guide setup`, which prints the same text.** What grund 0.19.0 announced lands: the name is no longer a command, so `grund agent-setup-instructions` is read as an ID query and fails with ``invalid ID `agent-setup-instructions` ``, prints nothing on stdout and exits `1`, and `grund help agent-setup-instructions` is ``error: unknown command: agent-setup-instructions`` with exit `2`. **Who this affects:** a script, CI step or agent instruction that still runs `grund agent-setup-instructions`, which saw the deprecation warning in 0.19.0: change it to `grund guide setup`, or to `grund guide --full` for the whole guide. The Python and Node `agent_setup_instructions()` and `agentSetupInstructions()` keep their names and are unaffected. Closes [issue #930](https://github.com/agent-grounds/grund/issues/930).

# DF-guide-command: grund agent-setup-instructions becomes grund guide, whose bare run prints a front page and whose sections are named positionally

**Date:** 2026-10-21

## status: Accepted

## 1. Context

[§DF-setup-guide-core-by-default](DF-setup-guide-core-by-default.md#df-setup-guide-core-by-default-grund-agent-setup-instructions-prints-a-core-that-opens-with-the-fresh-path-and-prints-each-reference-section-on-request) cut the setup guide into a core of at most 16,000 bytes and ten reference sections. The split worked, but nothing on the way in said it existed ([grund#893](https://github.com/agent-grounds/grund/issues/893)):

- `grund --help` gave the command one row that named a flag, `--section NAME for more`, but not the sections or where they were listed. At 24 characters, its name was also the only one too long for the page's command column.
- `grund agent-setup-instructions -h` listed the section names alone, with nothing on what each holds or what it costs to read: 0.7 KB for `schema`, 14 KB for `rules`. The one-line descriptions existed only inside the 15 KB core.
- The bare command printed that core, and most of it is an agent's interview script for adopting grund in a repository that is not fresh. The three commands a fresh repository runs were its first 0.8 KB.
- The name said the guide was for agents and for setup alone, though `rules`, `queries`, `citations` and `editor` are reference anyone reads, and "agent instructions" is already what grund calls the `AGENTS.md` that `init` writes ([§FS-init.2.3](../../functional-spec/FS-init.md#23-generated-agent-entrypoints)).

## 2. Decision

The command is `grund guide`, an ordinary row first in the *Set up* group of the top-level page ([§FS-cli.2.2](../../functional-spec/FS-cli.md#22-the-top-level-help-page)), and its menu is one level down:

- Bare `grund guide` prints the front page: the core's own *Fresh repository* section, then every section with its size and a one-line summary ([§FS-init.5.6](../../functional-spec/FS-init.md#56-the-front-page)).
- `grund guide <section>` prints one section. The core is the section `setup`, so the eleven sections are named one way, and `--full` prints them all in the order it did ([§FS-init.5.5](../../functional-spec/FS-init.md#55-the-core-and-its-sections)).
- An unknown name is refused with the `known sections:` line, which now opens with `setup` ([§FS-init.5](../../functional-spec/FS-init.md#5-the-guide)).
- `agent-setup-instructions` stayed through 0.19.0 as a deprecated name and was removed in 0.20.0 ([§FS-init.5.7](../../functional-spec/FS-init.md#57-agent-setup-instructions-is-the-deprecated-name-of-guide)). The window closed as planned, and [§DF-agent-setup-instructions-removed-in-0-20](DF-agent-setup-instructions-removed-in-0-20.md#df-agent-setup-instructions-removed-in-0-20-grund-agent-setup-instructions-is-no-command-from-0200) carries the notice of the release that removed it.

### 2.1 The section is a positional, which the previous record rejected

[§DF-setup-guide-core-by-default.3](DF-setup-guide-core-by-default.md#3-rejected-alternatives) rejected `grund agent-setup-instructions rules` because `show` already spells "one part of a document" as `--section`. Under `show`, though, `--section` narrows a coordinate the positional has already named. Under `guide` the section is the only thing a call names, as the client is for `grund integrations wezterm` and the shell for `grund completions bash`, so it takes the positional those commands give it. `--section` shipped in no release, so it is replaced rather than deprecated.

### 2.2 The core moves behind a name, which the previous record rejected

[§DF-setup-guide-core-by-default.3](DF-setup-guide-core-by-default.md#3-rejected-alternatives) also rejected moving the adoption workflow out of the default, because the adopting agent would then need two calls to start. It still does, but the cost is now one front page of about 3 KB, once per repository, and what it buys is a default that names everything the guide holds, sizes included. The fresh repository, the reader the old default made read 15 KB for 0.8 KB of commands, needs one call, since the front page carries those commands. The core keeps its 16,000-byte budget as the section `setup`.

### 2.3 Why `guide`

It fits the top-level page's 14-character command column, and it names what the command prints for any reader. `agent-instructions` was the other candidate: it is too long for the column, and it names the `AGENTS.md` block that `init` writes. `setup` beside `init` reads as a command that changes files.

## 3. Rejected alternatives

- **List the sections on the top-level page.** `grund --help` is read on every visit and fits 24 lines ([§FS-cli.2.2](../../functional-spec/FS-cli.md#22-the-top-level-help-page)), while the guide is read once per repository. Three section names, `config`, `schema` and `integrations`, are also command names, and the same page would show each twice.
- **Name the sections on the top-level row.** The names alone are what `-h` already printed, and they said nothing of what each section holds.
- **Rename the bindings too.** `agent_setup_instructions()` and `agentSetupInstructions()` return the whole guide as data, and their names are no command-line spelling, so a rename would be an API break across two languages for no reader's benefit, as it was for `docs` ([§DF-init-templates.2.3](DF-init-templates.md#23-the-bindings-keep-docs)).

## 4. Compatibility

No verdict moves: no tree passes or fails differently, and the exit codes keep their meaning, `0` printed and `2` a CLI-level error ([§REQ-backwards-compatibility.1](../../requirements/REQ-backwards-compatibility.md#1-what-is-covered)).

- **`grund agent-setup-instructions` warns.** In 0.19.0 it printed the core, as [§DF-setup-guide-core-by-default](DF-setup-guide-core-by-default.md#df-setup-guide-core-by-default-grund-agent-setup-instructions-prints-a-core-that-opens-with-the-fresh-path-and-prints-each-reference-section-on-request) made it do, after one stderr warning naming `grund guide setup` and grund 0.20.0, the release that removes the name. It took no arguments, as in 0.18.0. From 0.20.0 the name answers nothing ([§DF-agent-setup-instructions-removed-in-0-20](DF-agent-setup-instructions-removed-in-0-20.md#df-agent-setup-instructions-removed-in-0-20-grund-agent-setup-instructions-is-no-command-from-0200)).
- **The section list opens with `setup`.** A caller that parsed the `known sections:` line sees eleven names, the first of them new.

This record carries the release note for both records: the guide's split and its new name ship together in 0.19.0, and the note of [§DF-setup-guide-core-by-default](DF-setup-guide-core-by-default.md#df-setup-guide-core-by-default-grund-agent-setup-instructions-prints-a-core-that-opens-with-the-fresh-path-and-prints-each-reference-section-on-request) would have announced a flag that never shipped.

## release-note: Release note

- [§DF-guide-command](DF-guide-command.md#df-guide-command-grund-agent-setup-instructions-becomes-grund-guide-whose-bare-run-prints-a-front-page-and-whose-sections-are-named-positionally), [§FS-init.5](../../functional-spec/FS-init.md#5-the-guide): **`grund agent-setup-instructions` is now `grund guide`, and the guide is in sections.** Bare `grund guide` prints the three commands a fresh repository runs, then every section with its size and what it holds. `grund guide setup` prints the adoption workflow, at most 16,000 bytes, and `grund guide <section>` any of ten reference sections (`config`, `citations`, `languages`, `members`, `scoped-guidance`, `schema`, `queries`, `editor`, `integrations`, `rules`). `grund guide --full` prints the whole guide in one response, in a new order. **Who this affects:** `grund agent-setup-instructions` keeps working until grund 0.20.0, printing what `grund guide setup` prints with a warning on stderr, so a caller that read the whole guide from it passes `grund guide --full` instead. The Python and Node `agent_setup_instructions()` and `agentSetupInstructions()` keep their names and still return the whole guide. No verdict moves.

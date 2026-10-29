# DF-config-explain: the installed grund explains its own settings and codes under `grund config`

**Date:** 2026-10-20

## status: Accepted

## 1. Context

agent-grounds/grund#832 asked that the installed binary say what every setting and every check code means. Before it, `grund check --help` listed the codes by name and nothing more, and the place that said what turns a code on was the specification's catalog on GitHub, which spelled most switches the version-1 way that a version-2 file refuses. An agent asking about `oversized-lead` in a version-2 project was sent to `[reference] lead_size_warning` and refused at the next load. An Agora ruled on how the answer is built and proved, the author answered the open questions on the issue, and approved the plan on 2026-10-20 (comment 6289751915). The requirement is [§REQ-discoverable-offline](../../requirements/REQ-discoverable-offline.md#req-discoverable-offline-the-installed-grund-answers-for-every-setting-and-every-check-code) and the command is [§FS-config-explain](../../functional-spec/FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on).

## 2. Decision

### 2.1 A rung of `grund config`, not more `config show`

`config show` prints what this project sets, as TOML that loads back to the same values, and its bytes are kept ([§FS-config.4.2](../../functional-spec/FS-config.md#42-grund-config-show-path)). A meaning beside every key would break that round trip and print the whole reference to answer one question, which is what [§GOAL-token-economy.1](../../goals.md#1-what-this-requires) asks a rung not to do. `grund schema` describes the project's declarations, not grund's own vocabulary. A top-level command was rejected by the author: the answer is about configuration and codes, so it sits under `grund config` beside `show`, `validate` and `migrate`.

### 2.2 It narrows "`config show` is the teaching surface"

[§DF-v2-onboarding.2.2](DF-v2-onboarding.md#22-config-show-is-the-teaching-surface) made `show` the place a user learns the schema, because the minimal file `grund init` writes no longer lists every setting. That stays true of the configuration in effect: `show` still prints the settings in effect at their values. What a key means and what values it accepts is `explain`'s, and so is a setting `show` leaves out because it is not enabled ([§FS-config.4.2.3](../../functional-spec/FS-config.md#423-named_sections-and-values-print-only-when-enabled)), which is why the text that sends a reader to `show` names `explain` beside it ([§FS-config.4.2.3.1](../../functional-spec/FS-config.md#4231-what-sends-a-reader-to-show-names-explain)).

### 2.3 The catalog and the schema are rendered, never retyped

A code's meaning, severity and switch are written once, in its row of [§FS-errors.5.5](../../functional-spec/FS-errors.md#55-the-check-code-catalog), and a key's values and meaning once, in the published JSON schema. Both are rendered into files the crate embeds and compared by a test, so the binary says what the specification says, and no meaning is typed in Rust. The catalog is also the one place that says what turns a code on in each version: the line `check` prints when a selected check was off reads the same rendered rows ([§FS-check.1.4.1](../../functional-spec/FS-check.md#141-a-selected-check-the-run-leaves-off-says-so)), so the two cannot disagree. Typing the catalog in Rust was rejected because a second copy drifts, and reading the published schema at run time because it is not in the package and carries specification IDs ([§REQ-shipped-surfaces.1](../../requirements/REQ-shipped-surfaces.md#1-no-shipped-or-printed-byte-names-a-declaration-of-this-repository)).

### 2.4 It enumerates from what the readers admit

The schema words a key, but the list of keys comes from the readers' own gates, so an answer never names a key the reader would refuse and no key the reader accepts goes without one ([§FS-config-explain.3](../../functional-spec/FS-config-explain.md#3-a-keys-answer)). A test asks for every key and every code in each version from outside any checkout, over a project whose fetcher would leave a mark if it ran.

### 2.5 It reads only the version

A file the reader refuses is exactly when an agent needs the answer, so `explain` reads the version and nothing else, and answers in it ([§FS-config-explain.2](../../functional-spec/FS-config-explain.md#2-which-version-answers)). With no file it answers in both versions, version 2 first, because `grund init` writes version 2 and `check` reads version 1's defaults until a file exists.

### 2.6 Help and the guide label their versions instead

Help pages and the setup guide never load a config, so they cannot speak in the project's version. They name a version-specific spelling only with its version, or in words that hold in both ([§REQ-discoverable-offline.1](../../requirements/REQ-discoverable-offline.md#1-guidance-names-its-version)), and point at `explain` for the rest.

## 3. Consequences

Editing a catalog row or a schema description changes what the binary prints, so both are shipped text from here on and hold no specification ID. `config show`, `grund schema`, every refusal and every finding keep their bytes, and so does the line a selected check that was off prints. No verdict moves, so no release note is owed ([§FS-distribution.4.6.4](../../functional-spec/FS-distribution.md#464-compatibility-notices-come-from-the-decisions)).

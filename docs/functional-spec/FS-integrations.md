# FS-integrations: grund prints and installs its rendering-layer integrations

A plain `§<ID>` citation is made clickable where the reader is by a rendering layer, not by
grund ([§DF-neural-link-generation](../decisions/functional/DF-neural-link-generation.md#df-neural-link-generation-agents-compose-clickable-citation-links-themselves-grund-does-not-grow-a-link-command)). `grund integrations` prints and installs that layer: the
binary carries every client's artifact, prints it on demand, and writes it into the client's
configuration only when asked, so `cargo install grund` is enough to get clickable
citations in a terminal or editor. Why a one-time setup step is a subcommand is
[§DF-integrations-command](../decisions/functional/DF-integrations-command.md#df-integrations-command-integrations-earns-a-cli-slot-as-one-time-setup-where-a-per-citation-link-command-did-not). Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

## terms: Terms

Leans on [§FS-terms.terms.1](FS-terms.md#terms1-declarations-and-coordinates) (declaration, ID, kind, body, section, lead, catalog),
[§FS-terms.terms.2](FS-terms.md#terms2-citations) (marker, citation, shorthand), [§FS-terms.terms.4](FS-terms.md#terms4-scanning-and-project-structure) (scan, config root,
workspace, member, alias), and [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding).

- **rendering layer** — Whatever makes a bare citation clickable where the reader is — a
  terminal, an editor, or a forge — and where resolution belongs.
- **client** — One supported application this command prints an artifact for, or writes one
  into.
- **resolver script** — The embedded `grund-open` shell script that takes a citation, resolves
  it to a `path:line` site, and opens it.

## requirements: What every installed integration holds to

Two promises on the integrations as a whole are stated where they live: detection and every
printed form write nothing ([§REQ-no-data-loss.1](../requirements/REQ-no-data-loss.md#1-the-query-surface-never-writes)), and `--write` touches only the marked
things it owns ([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)).

### requirements.through-grund: Every client answers a citation with what `grund <ID>` answers

No client resolves a citation on its own: the terminal resolver asks
`grund <ID>[.<section>] --format json`, so a qualified or shorthand citation resolves through
`grund`'s own rules, never a client's regex ([§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2)); the editor extension asks
the `--brief` form ([§FS-integrations.3.2](FS-integrations.md#32-editor-clients-vscode-codium)); a peek is the same resolver in `--peek` mode, and a
keyboard pick goes through the call a click makes ([§FS-integrations.3.3](FS-integrations.md#33-reading-a-declaration-without-leaving)). So a click, a peek
and a hover cannot disagree with `grund <ID>` or with each other; only a location, which is
not a citation, opens without asking ([§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2)). Every client holds to this, and
it binds the next client, gesture and citation form to resolve by calling `grund` too. Serves
[§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree) and [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

### requirements.loud: An integration that cannot work says so

A citation that silently does nothing looks like an integration never installed, so every
silent failure named here is designed out or reported. Designed out: the
spawn's `PATH`, WezTerm's mouse capture and Flatpak boundary ([§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2)),
VSCodium's extensions root ([§FS-integrations.3.2](FS-integrations.md#32-editor-clients-vscode-codium)) and `$XDG_CONFIG_HOME`
([§FS-integrations.4.1.7](FS-integrations.md#417-where-a--target-resolves)). Reported: a click with no pane directory ([§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2)), a
config that never calls the installed helper ([§FS-integrations.4.1](FS-integrations.md#41-marked-blocks-in-dotfiles)), an agent skipped or gated
to another form ([§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files), [§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)), a manual client's state as not
knowable ([§FS-integrations.3.4](FS-integrations.md#34-manual-client-iterm2)), and the prerequisites `--write` cannot perform, in the guide
detection names ([§FS-integrations.2](FS-integrations.md#2-detection)). Every client holds to this, and it binds the next one to
prevent or report each silent failure it could cost. Serves [§GOAL-friendliness-first](../goals.md#goal-friendliness-first-as-user--and-agent-friendly-as-possible).

### requirements.any-repository: What is installed once serves every repository the user opens

The artifacts are user-global, so nothing in them is one repository's. The matchers and the
resolver accept every configured marker, and the resolver and the extension climb to the first
root that resolves the citation, past a workspace member's own config ([§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2),
[§FS-integrations.3.2](FS-integrations.md#32-editor-clients-vscode-codium)); the preference is the machine's ([§FS-integrations.4.3.1](FS-integrations.md#431-the-conversation-key)); and the
instruction blocks scope themselves to grund repositories and name no marker, carrying the
policy while each repository's entrypoint carries its syntax and the repository-web rule
([§FS-integrations.4.3.11](FS-integrations.md#4311-the-plain-block-text), [§FS-integrations.4.3.12](FS-integrations.md#4312-the-link-block-texts), [§FS-init.2.3.6](FS-init.md#236-clickable-citations)). Serves
[§GOAL-configurable](../goals.md#goal-configurable-every-default-is-overridable), whose per-repository marker no installed artifact may undo, and
[§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

### requirements.never-worse: No instruction grund writes leaves a citation harder to read than a bare one

The linked form falls back to the bare citation when unsure ([§FS-integrations.4.3.12](FS-integrations.md#4312-the-link-block-texts)), the
default `file` target degrades to a readable location where its scheme does not dispatch
([§FS-integrations.4.3.2](FS-integrations.md#432-the-conversation_target-key)), a pair of agent and target no click test has verified is written as
`path` ([§FS-integrations.4.3.3](FS-integrations.md#433-the-per-agent-gate)), and a per-agent override moves the request, never the
gate's verdict ([§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)). Every agent and target holds to this, and it binds a
new one to enter the gate at `path` until a click test verifies the pair, because a citation
erased where nothing says so is a silent failure. Serves [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work).

## 1. User-facing command

```
grund integrations [<client>] [--write] [--conversation plain|link] [--conversation-target <target>] [--agent <name>] [--format text|json]
```

- With no `<client>` and no write flags, grund detects the caller's environment and prints
  the integrations that apply, each with its one-line install ([§FS-integrations.2](FS-integrations.md#2-detection)), or, under
  `--format json`, the same detection as a plan ([§FS-integrations.5](FS-integrations.md#5-json-format)).
- With a `<client>`, one of `codium`, `iterm2`, `kitty`, `tmux`, `vscode` or `wezterm`, grund
  prints that client's integration ([§FS-integrations.3](FS-integrations.md#3-per-client-artifacts)).
- With a `<client>` and `--write`, grund installs the integration into the client's
  configuration instead of printing it, and records the user's local-conversation preference
  in the global agent instructions ([§FS-integrations.4](FS-integrations.md#4-managed-writes---write)); the manual client installs what it
  can and prints the rest ([§FS-integrations.3.4](FS-integrations.md#34-manual-client-iterm2)).

### 1.1 `--conversation`

`--conversation plain|link` sets the user's local-conversation preference
([§FS-integrations.4.3.1](FS-integrations.md#431-the-conversation-key)) and is accepted only with `--write`. Without a client, the write
updates only the preference and the global agent instructions and installs no client. With no
conversation flag, the first client write records `plain`, and a stored preference survives
later writes unless a flag changes it.

### 1.2 `--conversation-target` and `--agent`

`--conversation-target <target>` sets how the `link` form addresses a declaration, one of
`file` (default), `path`, `web`, `vscode`, `vscodium` or `cursor` ([§FS-integrations.4.3.2](FS-integrations.md#432-the-conversation_target-key)),
and `--agent <name>` scopes it to one agent instead of the machine ([§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)).
Both are accepted only with `--write`, `--agent` only beside `--conversation-target`, and
either conversation flag is enough for the clientless form of [§FS-integrations.1.1](FS-integrations.md#11---conversation). A target
given while the effective preference is `plain` is recorded and inert, so a machine that
later switches to `link` keeps the target it chose. An unknown target or agent is a
CLI-level error listing the accepted set, exit `2`, and so is bare
`grund integrations --write`, with neither a client nor a conversation flag
([§FS-integrations.6.2](FS-integrations.md#62-exit-codes)).

### 1.3 Which side owns what

`integrations(IntegrationsOpts)` is the one engine operation behind every form of the
command, and it answers with data, rendering none of it ([§FS-distribution.3.1.3](FS-distribution.md#313-integrations-is-the-one-integrations-entry-point)). The CLI's
text and JSON plan and the Python and Node `integrations` operations
([§FS-distribution.3.3.6](FS-distribution.md#336-explicit-mutation-opt-ins)) all render that answer, so nothing a `--write` changes is planned,
spliced or written outside it.

`IntegrationsOpts` is built from `Default` and carries `client: Option<IntegrationClient>`,
`write: bool`, `conversation: Option<ConversationRendering>`,
`conversation_target: Option<ConversationTarget>` and `agent: Option<IntegrationAgent>`, the
command's arguments in their closed vocabularies. The operation refuses a combination
[§FS-integrations.6.2](FS-integrations.md#62-exit-codes) refuses before it reads anything, reads the preference file once
([§FS-integrations.4.3.5](FS-integrations.md#435-nothing-in-the-file-is-an-error)), and plans every guidance target before it writes the first
([§FS-integrations.4.3.14](FS-integrations.md#4314-delimiters-and-failures)).

`IntegrationsOutput` holds facts, never a line of output:

- `detected`, the clients the environment shows ([§FS-integrations.2](FS-integrations.md#2-detection));
- `clients`, one `IntegrationClientStatus` per client: its client, its `InstallKind`, whether
  it is detected, whether its artifacts are installed and current, and its config target, but
  no install command, which the CLI builds from the client's `name` and the JSON plan prints
  as `install`;
- `artifact`, an `IntegrationArtifact` for a named client: its snippet, and the resolver,
  `package.json` and `extension.js` as content with the resolver's target
  ([§FS-integrations.3](FS-integrations.md#3-per-client-artifacts));
- `events`, one `IntegrationEvent` per target a `--write` touched, in order: its path, an
  `IntegrationOutcome` (`Wrote`, `Appended`, `Updated`, `Unchanged` or `Skipped`, whose `name`
  is the verb of [§FS-integrations.4.1.6](FS-integrations.md#416-outcome-verbs), `exists` for `Unchanged`), and an optional
  `IntegrationNote`: the WezTerm config's missing apply call ([§FS-integrations.4.1](FS-integrations.md#41-marked-blocks-in-dotfiles)), an
  agent block's written form, requested target and override ([§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)), or the
  agent home whose absence skipped it ([§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files));
- `manual_steps`, one `IntegrationManualStep` per client the user must set up by hand, naming
  the client and its snippet ([§FS-integrations.3.4](FS-integrations.md#34-manual-client-iterm2));
- `warnings`, the user configuration's cautions as `Finding`s with path and line
  ([§FS-integrations.4.3.5](FS-integrations.md#435-nothing-in-the-file-is-an-error)).

`IntegrationsError` implements `std::error::Error` and carries the events written before it
and one `IntegrationsFailure`: the option combination it refused, or the path and the
filesystem's message. So the CLI prints what a run that failed partway wrote, then the
failure as its `error:` line, and exits `2`.

### 1.4 Unknown client

An unknown client is a CLI-level error ([§FS-cli.4](FS-cli.md#4-errors-with-no-source-location)): these two lines on stderr, empty stdout,
exit `2`. The client set is closed.

```text
error: unknown integration client `<client>`
known clients: codium, iterm2, kitty, tmux, vscode, wezterm
```

## 2. Detection

With no `<client>`, grund decides which integrations apply from these environment variables
alone, and never writes or scans ([§REQ-no-data-loss.1](../requirements/REQ-no-data-loss.md#1-the-query-surface-never-writes)):

- `WEZTERM_EXECUTABLE` → `wezterm`
- `KITTY_WINDOW_ID` → `kitty`
- `TERM_PROGRAM` — value `WezTerm` → `wezterm`, `iTerm.app` → `iterm2`, `tmux` → `tmux`, `vscode` → `vscode`
- `VSCODE_PID` or any `VSCODE_*` variable → `vscode`; additionally → `codium` when any of those values names VSCodium's application directory
- `TMUX` (non-empty) → `tmux`

The clients that match are printed deduplicated in the client order
`codium, iterm2, kitty, tmux, vscode, wezterm`, each with its one-line
install; when none matches, grund prints every client with its install and a note that none
was detected. Either list closes with how to preview a client before installing it, the URL
of the setup guide, which writes down the prerequisites `--write` cannot perform and each
client's manual step, and two lines pointing at `grund-lsp`:

```text
Editors get go-to-definition and hover from grund-lsp (IntelliJ: `grund-lsp integrations lsp4ij`).
LSP guide: https://github.com/agent-grounds/grund/blob/main/docs/user-facing/lsp.md
```

Go-to-definition, hover, references, completion and diagnostics in an editor's buffer come
from `grund-lsp` ([§FS-lsp.2.3](FS-lsp.md#23-editor-configuration-one-time-per-editor), [§FS-lsp.2.4](FS-lsp.md#24-installed-editor-integrations)), which alone serves an editor with no
clickable-citation client, such as the IntelliJ family, Helix or Neovim. So the body of
`grund integrations -h` says that go-to-definition, hover and completion in an editor come
from `grund-lsp`, not from these integrations, names `grund-lsp integrations lsp4ij`, and
lists the LSP guide after the setup guide ([§FS-cli.2.3](FS-cli.md#23-a-subcommands-help-page)); the `integrations` row of
`grund --help` names `grund-lsp` ([§FS-cli.2.2](FS-cli.md#22-the-top-level-help-page)). The JSON plan carries no pointer
([§FS-integrations.5](FS-integrations.md#5-json-format)), and `grund integrations` never detects, installs or runs `grund-lsp`.
Detection exits `0` in every case.

## 3. Per-client artifacts

Every artifact is embedded in the binary: nothing is fetched, and nothing is published to a
marketplace ([§FS-non-goals.12.2](FS-non-goals.md#122-first-party-per-editor-plugins)).

### 3.1 Terminal clients: `wezterm`, `kitty`, `tmux`, `iterm2`

Each prints on stdout, exit `0`, a config snippet that binds `§<ID>` citations to the
resolver, as a click or hints handler or, in tmux, which cannot make text clickable, a prefix
key over the copy buffer; and the `grund-open` resolver, a POSIX shell script that also opens
a location.

The resolver opens a site with `GRUND_OPEN_CMD` when it is set, else `EDITOR`, else a
platform default, never an editor named in repository text ([§DF-neural-link-generation](../decisions/functional/DF-neural-link-generation.md#df-neural-link-generation-agents-compose-clickable-citation-links-themselves-grund-does-not-grow-a-link-command)).
`GRUND_OPEN_CMD` is an argv prefix, not shell source: one `path:line` argument is appended and
no repository path is evaluated as a command, and the text a handler passes in, a tmux copy
buffer or a kitty hint, arrives as one quoted argument.

A citation is matched as one to three characters that are neither word nor space, then an
optional `<alias>/`, then a kind that begins with an uppercase letter and may carry digits
(`E2E`), `-` and a slug, which may begin with a digit, then the whole `.<section>` suffix,
each component digits or a letter-first lowercase handle ([§FS-config.3.3.1](FS-config.md#331-named-components)); a sentence-ending
`.` is left out. The resolver drops any leading run of non-alphanumeric characters. Neither
hardcodes `§`, because `[reference] marker` is per repository ([§FS-config.3.1](FS-config.md#31-reference--citation-form)) and an artifact
is written once for all of them. The resolver then:

1. starts in the clicked pane's directory, which every handler `cd`s into except kitty's,
   whose hints kitten starts there itself;
2. tries every ancestor holding a config under either discovery name ([§FS-config.1](FS-config.md#1-file-location-and-discovery)), nearest
   first, and keeps the first root where `grund <ID>[.<section>] --format json` resolves the
   citation. A failed resolution continues the climb, so a qualified `<alias>/<ID>` clicked
   inside a workspace member resolves from the workspace root ([§FS-workspace.8.1](FS-workspace.md#81-grund-aliasid)), and a
   number-only shorthand opens because `grund` resolves it ([§FS-citations.shorthand.cli](FS-citations.md#shorthandcli-the-shorthand-as-a-cli-id-argument),
   [§DF-number-only-citation-shorthand.4](../decisions/functional/DF-number-only-citation-shorthand.md#4-consequences));
3. joins the reported `path`, relative to the config root ([§FS-config.3.6](FS-config.md#36-output--report-format)), onto that root
   and opens the absolute path at the cited section's own line, or, for an E2E case answered
   with the manifest shape ([§FS-show.2.4](FS-show.md#24-e2e-cases)), opens the case directory with no line.
   A body that itself holds `"path":` or `"line":` does not move the site.

With no config-bearing ancestor the resolver prints
`grund-open: no grund.toml (root or .agents/) at or above <dir>` and exits `1`; when every
ancestor fails to resolve the citation, the error is the ID's.

A token whose final path segment has a dot-extension and ends in `:<line>`, optionally
`:<col>`, is a **location**, the `path:line` of the `path` conversation target
([§FS-integrations.4.3.2](FS-integrations.md#432-the-conversation_target-key)); a section suffix is dotted, never coloned. Every terminal matcher but
iTerm2's citation-only rule ([§FS-integrations.3.4](FS-integrations.md#34-manual-client-iterm2)) hands a location to `grund-open`, which
opens it without consulting `grund`: it climbs ancestors, nearest first, until the relative
path exists (an absolute path is tested directly) and opens that line, dropping any column,
in any repository.

Two false positives cost one explanatory error each, never a wrong file, and the real citation
on the same line still matches: an ID-shaped file name, its extension read as a section, since
no portable matcher can look around a match; and `host.com:8080`, which has a location's
shape. WezTerm and kitty try the location shape first, so there a `path:line` token opens
whole and the file-name false positive arises only without a line suffix.

Every GUI terminal spawns the resolver through `sh` with `~/.local/bin` and `~/.cargo/bin`
prepended to `PATH`, since a terminal started from a desktop launcher inherits neither
directory. Beyond that:

- **kitty** binds its gestures under its own hints prefix, `ctrl+shift+p`
  ([§FS-integrations.3.3](FS-integrations.md#33-reading-a-declaration-without-leaving)), and never maps `ctrl+shift+p` alone, which would discard every
  sequence kitty maps under it.
- **WezTerm** registers the click and peek gestures twice, once with
  `mouse_reporting = true`, because it ignores user mouse bindings while a program such as an
  agent TUI has captured the mouse. Under Flatpak (`FLATPAK_ID`) the click's spawn goes to the
  host through `flatpak-spawn --host`, while the peek, a pane command WezTerm already routes
  there, is not wrapped again. The pane's directory comes first from WezTerm's own report of
  it, asked under both spellings builds have so that neither one's absence aborts the handler;
  that report answers only when the shell emits `OSC 7`, which most shells must be made to do.
  Failing it, the directory comes from the foreground process's `cwd`, except under Flatpak,
  where that is the sandbox helper's, so there `OSC 7` is the only source. The artifact and
  the setup guide say so, and a click with no directory raises a toast and a WezTerm log line.

### 3.2 Editor clients: `vscode`, `codium`

`grund integrations vscode` prints the terminal-citations extension's source and its
`--write` install ([§FS-integrations.4.2](FS-integrations.md#42-the-vs-code-extension)). The extension is a `TerminalLinkProvider` that turns
citations in the integrated terminal into links:

- It matches and strips the marker as the terminal clients do, keeps the `.<section>` suffix
  ([§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2)), and resolves with one
  `grund <ID>[.<section>] --brief --format json`, whose `path` and `line` a click needs and
  whose body is the hover ([§FS-integrations.3.3](FS-integrations.md#33-reading-a-declaration-without-leaving)). It caches only successful resolutions,
  so a citation that failed resolves once its declaration exists.
- It finds the root by the resolver's climb from the workspace folder
  ([§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2)), runs `grund` there, and joins the reported path onto it.
- An E2E case, answered with the manifest shape ([§FS-show.2.4](FS-show.md#24-e2e-cases)), reveals its case directory in
  the Explorer, or in the OS file manager when the directory lies outside the opened folder.

`codium` installs the same extension into VSCodium's own root, `~/.vscode-oss/extensions`,
as a client of its own, because an extension written under `~/.vscode` for a VSCodium user is
never loaded while the install reports success. Both editors export the same `VSCODE_*`
variables, so detection marks `codium` beside `vscode`, not instead of it, when one of their
values names VSCodium's application directory ([§FS-integrations.2](FS-integrations.md#2-detection)).

### 3.3 Reading a declaration without leaving

Every client also reads a citation in place. In VS Code a hover shows the declaration's
`--brief` slice and its resolved `path:line` with no click, from the resolution the extension
caches ([§FS-integrations.3.2](FS-integrations.md#32-editor-clients-vscode-codium)). A terminal has no event for motion with no button held, so the
terminal clients offer **peek**: a second gesture runs `grund-open --peek <citation>`, which
prints the resolved `path:line` and the declaration's lead, or for a location the file from
the cited line, through a pager, in a surface that closes when the pager exits:

| Client | Peek | Surface |
| --- | --- | --- |
| kitty | `ctrl+shift+p` `i`, then the hint label | overlay window |
| tmux | `prefix` + `G` | popup (requires tmux 3.2+; inert on older tmux, where `prefix` + `g` still opens) |
| wezterm | `ctrl+shift+`-click, or `ctrl+shift+i` and the label | split pane |

WezTerm and kitty also pick from the keyboard, `g` to go and `i` to inspect: `ctrl+shift+g` in
WezTerm and `ctrl+shift+p` `g` in kitty go, and the `i` gestures of the table inspect. Either
gesture labels every citation on the screen with the shape of [§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2), and the
picked label goes to the call a click makes; WezTerm's picker leaves the clipboard untouched.

### 3.4 Manual client: `iterm2`

iTerm2 keeps its configuration in a binary property list, with no comment syntax for a
managed block's delimiters ([§FS-integrations.4.1](FS-integrations.md#41-marked-blocks-in-dotfiles)), and grund does not rewrite it under the
user, so `iterm2` is the one **manual** client. `grund integrations iterm2` prints the Smart
Selection rule to add: the citation regex of [§FS-integrations.3.1](FS-integrations.md#31-terminal-clients-wezterm-kitty-tmux-iterm2) at precision `Very High`,
so it does not lose to iTerm2's built-in filesystem-path rule; a first action `Run Command…`
of `grund-open \0`, which cmd-click fires; and a second, `grund-open --peek \0`, which the
right-click menu offers as peek ([§FS-integrations.3.3](FS-integrations.md#33-reading-a-declaration-without-leaving)). `\0` is the whole match, marker
included.

`grund integrations iterm2 --write` installs what grund can, the `grund-open` resolver and
the user preference and global agent instructions ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)), then prints the
steps it cannot apply and reports `manual <client> (<where>)` on stderr, exit `0`. A
manual client always reports `installed: false`, since grund never reads that property list,
and `"install_kind":"manual"`, so a caller can tell *not knowable* from *not installed*
([§FS-integrations.5](FS-integrations.md#5-json-format)).

iTerm2's Semantic History already makes a plain `path:line` cmd-clickable, so
`--conversation link` with the `path` target ([§FS-integrations.4.3.2](FS-integrations.md#432-the-conversation_target-key)) makes citations
clickable there with no rule installed.

### 3.5 Clients grund cannot support, and why

A terminal can be a client only if its configuration binds arbitrary matched text to an
arbitrary command; these cannot, so the closed client set ([§FS-integrations.1.4](FS-integrations.md#14-unknown-client)) leaves them
out:

- **Ghostty** marks its regex-to-action `link` option `TODO: This can't currently be set!`,
  and its built-in URL matcher opens no `path:line`, so `--conversation link` does not help
  either.
- **Apple Terminal.app** has no regex-to-action mechanism of any kind.
- **Windows Terminal** detects URLs only, with no configurable matcher.

## 4. Managed writes (`--write`)

`--write` never prints an artifact: with a client it installs that integration, and without
one it updates only the user guidance ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)). It reports on stderr
([§FS-integrations.4.1.6](FS-integrations.md#416-outcome-verbs)) and exits `0`. It touches only what grund owns
([§REQ-no-data-loss.2](../requirements/REQ-no-data-loss.md#2-writers-touch-only-what-they-own)): a marked block in a client configuration or an agent's instruction
file, the resolver, its own extension directory, and the keys it records in the preference
file. Outside these it writes only the starter scaffold of a WezTerm config it creates
([§FS-integrations.4.1](FS-integrations.md#41-marked-blocks-in-dotfiles)). Every write is idempotent, as `init`'s managed block is ([§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints)): a re-run of a
current integration changes nothing and reports `exists`, an upgrade rewrites only what grund
owns, and removal is deleting it.

### 4.1 Marked blocks in dotfiles

For `kitty`, `tmux`, and `wezterm`, `--write` splices a managed block into the client's
configuration file and installs the `grund-open` resolver:

```
# >>> grund integrations (v1) >>>
<snippet>
# <<< grund integrations (v1) <<<
```

- The delimiters use the host file's own comment token, `#` in `kitty.conf` and `.tmux.conf`
  and `--` in `wezterm.lua`, and a block is looked up under that token alone.
- One existing block at any supported version has its complete delimiter-line span replaced,
  indented delimiter lines included, and every byte outside it kept; a file with no block gets
  the block appended after a blank line. A missing matching delimiter, several blocks, or a
  block newer than the binary understands is an error, exit `2`.
- A WezTerm config that `--write` creates also gets an unmanaged starter scaffold below the
  block, which the user owns from then on, because WezTerm applies hyperlink rules only from
  the config object the file returns; an existing config never gets one. While the config
  outside the block does not call `grund_apply_hyperlink_rule(config)`, every write prints one
  `note:` line on stderr naming the call and the file, with the exit code unchanged, since the
  block installs the same either way.
- The resolver is written to its fixed path, `~/.local/bin/grund-open`, only when absent or
  when its embedded version differs, and failing to make it executable is an I/O error, not a
  successful install.

#### 4.1.6 Outcome verbs

Each target a write touches is one stderr line, `<verb> <path>`, in the order the write
touched them: the client's own targets, then the preference file, then each agent's
instruction file in the order of the table of [§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files). The verb is `wrote` for
the resolver script or a first VS Code extension ([§FS-integrations.4.2](FS-integrations.md#42-the-vs-code-extension)), `appended` for an
added block, `updated` for a replaced block or extension, `exists` for a current target, and
`skipped` for a user-guidance target whose agent is not in use ([§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files)).

#### 4.1.7 Where a `~` target resolves

Install targets are printed `~`-rooted, so the artifacts and the JSON descriptor are the
same bytes on every machine ([§FS-integrations.6](FS-integrations.md#6-determinism-and-exit-codes)), and only a write resolves them. A
`~/.config/…` target resolves against `$XDG_CONFIG_HOME` when it is set and non-empty, else
`~/.config`, as kitty, WezTerm, Zed and grund's own user configuration do. Every other target,
`~/.local/bin/grund-open`, `~/.vscode/extensions`, `~/.tmux.conf` and the agent instruction
files of [§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files) other than Zed's, resolves against `$HOME` alone.

### 4.2 The VS Code extension

`vscode --write` writes the unpacked extension directory (its `package.json`, the provider
script, and a stamp file recording the embedded version) into the editor extensions
directory. A present, byte-current same-version extension is left untouched (`exists`); a
different, missing, or damaged grund-owned file is repaired. grund writes only files it owns
inside its own extension directory; it never edits the user's `settings.json`.

### 4.3 User preference and global agent instructions

Every client `--write`, and every clientless `--write` with a conversation flag, records the
effective local-conversation preference in the user preference file ([§FS-integrations.4.3.1](FS-integrations.md#431-the-conversation-key))
and writes it into the global instruction file of every agent in use
([§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files)).

#### 4.3.1 The `conversation` key

**`[reference] conversation`**, in the user preference file `~/.config/grund/config.toml`
([§FS-integrations.4.1.7](FS-integrations.md#417-where-a--target-resolves)), is the repository config's key at the user's scope, where it takes
`plain` or `link` ([§FS-config.3.1.3](FS-config.md#313-conversation--one-name-two-scopes)); the preference is the machine's, not a repository's
([§DF-integrations-command.3](../decisions/functional/DF-integrations-command.md#3-consequences)). `plain` says a rendering layer of [§FS-integrations.3](FS-integrations.md#3-per-client-artifacts) makes a
bare citation clickable; `link` says the surface cannot, as a stock terminal, `ssh` or a TUI
with no matcher cannot, so the location travels with the citation. Editing the file by hand
is equivalent to `--conversation`, and `--write` is what carries it into the instruction
blocks. A repository's committed `link` opinion ([§FS-config.3.1](FS-config.md#31-reference--citation-form),
[§DF-repo-conversation-opinion](../decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)) is the fallback for a machine that recorded no preference; a
preference recorded here wins over it ([§DF-repo-conversation-opinion.2.3](../decisions/functional/DF-repo-conversation-opinion.md#23-precedence)).

#### 4.3.2 The `conversation_target` key

**`[reference] conversation_target`** selects how a linked citation addresses its
declaration, and has no repository spelling ([§DF-conversation-link-target.2.3](../decisions/functional/DF-conversation-link-target.md#23-the-target-is-user-scoped-but-the-default-is-committable)). Each value
names one fixed template, filled from the declaration's absolute path and line:

| Value | Rendered target |
| --- | --- |
| `file` (default) | `file://<abs>#L<line>` |
| `path` | *(no URI)* — the location as plain `path:line` text beside the citation |
| `web` | the forge blob URL for the current ref, per the repository-web rule ([§FS-init.2.3.6](FS-init.md#236-clickable-citations)) |
| `vscode`, `vscodium`, `cursor` | `<scheme>://file<abs>:<line>` |

`file` is the default, presuming only a handler for `file:` and degrading to a readable
location where none dispatches ([§DF-conversation-link-target.2.2](../decisions/functional/DF-conversation-link-target.md#22-the-scheme-is-chosen-per-machine-and-file-is-the-default)). An
unknown value is a CLI error on the flag ([§FS-integrations.6.2](FS-integrations.md#62-exit-codes)), and, written in the file, a
warning that leaves no target from the file, so `file` applies ([§FS-integrations.4.3.5](FS-integrations.md#435-nothing-in-the-file-is-an-error)).

#### 4.3.3 The per-agent gate

The linked form is gated per agent ([§DF-conversation-link-target.2.4](../decisions/functional/DF-conversation-link-target.md#24-the-form-is-gated-per-agent-and-the-fallback-is-path)): a target is written
into an agent's block only where the pair is verified, and every other block falls back to
`path`. Claude receives every target; Pi receives `file` and `web`; Codex receives `web`;
Gemini, Copilot, and Zed receive `path`. The gate is reported rather than silent: each
instruction target's line names the form it received and, when that is not the form asked
for, why ([§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides)).

#### 4.3.5 Nothing in the file is an error

grund reads `reference.conversation`, `reference.conversation_target` and the per-agent
overrides of [§FS-integrations.4.4](FS-integrations.md#44-per-agent-overrides) as TOML defines them, so whitespace inside a section
header, a dotted key path and either quote style are one setting. It reads and reports the
file once per invocation, before any client artifact is installed, and nothing in it is an
error: each line grund ignores is one `warning:` naming the line and what is ignored, and the
run continues on the effective preference. A line is ignored when it holds:

- a key grund does not read, reported as `unused key <path>; grund reads only reference.conversation, reference.conversation_target, and reference.agents.<agent>.conversation_target from this file`;
- an override under an agent grund does not know, reported as `unknown agent <name> in reference.agents; known agents: codex, claude, gemini, copilot, zed, pi`;
- a value grund cannot interpret, quoted wrongly or outside `plain | link` for `conversation`
  and the target set for `conversation_target`, which leaves grund with no such preference
  from the file, so the run uses the flag if given and otherwise the `plain` and `file`
  defaults. The two keys fail independently;
- a repeated key after its first occurrence, which is the one a read uses and a write
  rewrites.

The only failure the file can cause is being unreachable ([§FS-integrations.4.3.14](FS-integrations.md#4314-delimiters-and-failures)). It warns
where the repository config refuses ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)) because it is machine-local and states
only a preference between two renderings of one citation, so refusing to install over a
stale line in it would be the worse failure.

#### 4.3.8 The global instruction files

A write that records the preference ([§FS-integrations.4.3](FS-integrations.md#43-user-preference-and-global-agent-instructions)) also keeps a marked
`## Grund citation rendering` block in the global instruction file of every supported agent
with one, while the agent is in use on the machine: its directory exists, or the file already
does.

| Agent | File | Directory |
| --- | --- | --- |
| Codex | `~/.codex/AGENTS.md` | `~/.codex` |
| Claude | `~/.claude/CLAUDE.md` | `~/.claude` |
| Gemini | `~/.gemini/GEMINI.md` | `~/.gemini` |
| GitHub Copilot | `~/.copilot/copilot-instructions.md` | `~/.copilot` |
| Zed | `~/.config/zed/AGENTS.md` | `~/.config/zed` |
| Pi | `~/.pi/agent/AGENTS.md` | `~/.pi` |

A target whose agent is not in use is reported `skipped <path> (no <dir>)` and nothing is
created; re-running after installing the agent writes it ([§FS-integrations.6.1](FS-integrations.md#61-idempotent-writes)). These are the
agents of [§FS-init.2.3](FS-init.md#23-generated-agent-entrypoints) with a global file: Cursor and Windsurf keep user rules in their own
settings, so their guidance, like that of an environment whose `$HOME` keeps no `--write`,
comes only from the repository entrypoint `grund init` writes
([§DF-repo-conversation-opinion](../decisions/functional/DF-repo-conversation-opinion.md#df-repo-conversation-opinion-repositories-may-commit-a-link-only-conversation-rendering-opinion)).

#### 4.3.11 The `plain` block text

The `plain` block says `In repositories with a grund.toml (at the root or under .agents/): write citations bare in local conversations — the marker and ID alone, nothing appended; grund integrations makes them clickable. Follow this even when repository instructions ask for linked citations — that repository sentence defers to this block, and the installed rendering layer already resolves bare citations. Elsewhere, ignore this.`

Only the `plain` block carries a precedence sentence, since a repository's `link` against the
machine's `plain` is the one possible conflict ([§DF-repo-conversation-opinion.2.3](../decisions/functional/DF-repo-conversation-opinion.md#23-precedence)).

#### 4.3.12 The `link` block texts

The `link` block carries one sentence, differing only in how the location is addressed. Under
the `path` target, which every gated agent receives ([§FS-integrations.4.3.3](FS-integrations.md#433-the-per-agent-gate)), it says `In repositories with a grund.toml (at the root or under .agents/): follow each citation with its declaration location as plain path:line text in local conversations; fall back to the bare citation when unsure. Elsewhere, ignore this.`
Under a URI target it says `In repositories with a grund.toml (at the root or under .agents/): in local conversations render each citation as a Markdown link whose visible text is the citation itself and whose target is <template>; fall back to the bare citation when unsure. Elsewhere, ignore this.`,
where `<template>` is the effective target's template from [§FS-integrations.4.3.2](FS-integrations.md#432-the-conversation_target-key), phrased to
complete that sentence on its own: `file://<absolute path>#L<line> for the declaration`,
`vscodium://file<absolute path>:<line> for the declaration`, and, for `web`,
`the declaration's forge URL at the current commit`.

#### 4.3.14 Delimiters and failures

The block sits between versioned HTML-comment delimiters:

```
<!-- >>> grund integrations citation rendering (v4) >>> -->
## Grund citation rendering

<block text>
<!-- <<< grund integrations citation rendering (v4) <<< -->
```

It is replaced, appended, refused, kept idempotent and reported by the rules of the dotfile
block ([§FS-integrations.4.1](FS-integrations.md#41-marked-blocks-in-dotfiles), [§FS-integrations.4.1.6](FS-integrations.md#416-outcome-verbs)). For a target in use
([§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files)), missing parent directories and the file itself are created.

Failing to update the preference or any instruction file is an error. Failing to reach the
preference file changes nothing ([§FS-integrations.4.3.5](FS-integrations.md#435-nothing-in-the-file-is-an-error)); a block failure found later may
leave an integration artifact already written, and re-running after the fix completes the
installation.

### 4.4 Per-agent overrides

Agents on one machine do not render alike ([§FS-integrations.4.3.3](FS-integrations.md#433-the-per-agent-gate)), so
`[reference.agents.<agent>]`, where `<agent>` is one of `codex`, `claude`, `gemini`, `copilot`,
`zed` or `pi`, the agents of [§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files), is a partial of the settings above,
merged over them:

```toml
[reference]
conversation        = "link"
conversation_target = "vscodium"   # the machine-wide base

[reference.agents.codex]
conversation_target = "web"        # this agent only
```

A key present under an agent replaces the machine-wide value for that agent, and an absent
key inherits it; there is no per-agent default and no way to unset the base, so deleting the
entry restores inheritance. `conversation_target` is the only key the partial accepts
([§DF-conversation-link-target.2.5](../decisions/functional/DF-conversation-link-target.md#25-a-per-agent-override-is-a-preference-not-evidence)). `--agent <name>` takes the same names and scopes
`--conversation-target`, and nothing else, to that agent: it writes one such entry and leaves
both machine-wide keys as they were ([§FS-integrations.1.2](FS-integrations.md#12---conversation-target-and---agent)).

An override is a preference and the gate of [§FS-integrations.4.3.3](FS-integrations.md#433-the-per-agent-gate) is evidence, so the gate
runs last: `[reference.agents.codex] conversation_target = "vscodium"` resolves to
`vscodium` and is then written as `path`. Each instruction target's report line shows both
layers:

```
$ grund integrations kitty --write --conversation link --conversation-target web --agent codex
appended <sandbox>/xdg/kitty/kitty.conf
wrote <sandbox>/home/.local/bin/grund-open
appended <sandbox>/xdg/grund/config.toml
appended <sandbox>/home/.codex/AGENTS.md (link → web; agent override)
appended <sandbox>/home/.claude/CLAUDE.md (link → file)
appended <sandbox>/home/.gemini/GEMINI.md (link → path; file unverified here)
appended <sandbox>/home/.copilot/copilot-instructions.md (link → path; file unverified here)
appended <sandbox>/xdg/zed/AGENTS.md (link → path; file unverified here)
appended <sandbox>/home/.pi/agent/AGENTS.md (link → file)
```

## 5. JSON format

`grund integrations --format json` with no client prints the detection plan as one JSON
object on stdout, and any CLI-level `error:` on stderr ([§FS-errors.1](FS-errors.md#1-streams)). The plan is
`{"detected":[…],"clients":[…]}`: `detected` lists the detected clients in client order, and
`clients` holds one object per client, in client order, with these fields in this order:
`client`, its name; `detected`; `installed`, whether its grund-owned artifacts are installed
and current; `install_kind`, one of `block`, `extension` or `manual`
([§FS-integrations.3.4](FS-integrations.md#34-manual-client-iterm2)); and `install`, the exact `--write` command an agent would run.
Installation checks read only the fixed integration targets and never search the filesystem.
It is the surface `grund guide` points an agent at ([§FS-init.5](FS-init.md#5-the-guide)): detect, show the diff, and
write only on the user's confirmation.

### 5.1 The single-client object

`grund integrations <client> --format json` prints one object describing that client's
artifact, its target paths and its `--write` command, without the artifact's bytes:

```json
{"client":"wezterm","kind":"terminal","install":"grund integrations wezterm --write","install_kind":"block","config_target":"~/.config/wezterm/wezterm.lua","resolver_target":"~/.local/bin/grund-open"}
```

## 6. Determinism and exit codes

With an explicit `<client>`, output depends only on the binary version and the flags, never
on the environment, a tree or the clock. Detection with no client depends also on the
variables of [§FS-integrations.2](FS-integrations.md#2-detection) and, for JSON, the fixed installation targets.

### 6.1 Idempotent writes

`--write` is idempotent ([§FS-integrations.4](FS-integrations.md#4-managed-writes---write)), the preference file included: a recorded value
equal to the effective one is left byte for byte, with any comment beside it, rather than
rewritten in grund's spelling. A manual client ([§FS-integrations.3.4](FS-integrations.md#34-manual-client-iterm2)) has no target to
compare, so its `--write` reinstalls the resolver only when stale and prints the same steps.

### 6.2 Exit codes

Exit codes follow the fixed mapping of [§FS-cli.5](FS-cli.md#5-exit-code-mapping-is-fixed):

- `0`: printed, written, or already current.
- `2`: an unknown client; bare `--write` with neither a client nor a conversation flag;
  `--conversation` or `--conversation-target` without `--write`; `--agent` without `--write`
  or without `--conversation-target`; `--write` with `--format json`; an unknown agent name,
  target or flag; a managed block [§FS-integrations.4.1](FS-integrations.md#41-marked-blocks-in-dotfiles) refuses; or an I/O failure
  ([§FS-integrations.4.1](FS-integrations.md#41-marked-blocks-in-dotfiles), [§FS-integrations.4.3.5](FS-integrations.md#435-nothing-in-the-file-is-an-error), [§FS-integrations.4.3.14](FS-integrations.md#4314-delimiters-and-failures)).

There is no `1`: `integrations` has no findings. Nothing in the user preference file and no
`skipped` target moves the exit code ([§FS-integrations.4.3.5](FS-integrations.md#435-nothing-in-the-file-is-an-error), [§FS-integrations.4.3.8](FS-integrations.md#438-the-global-instruction-files)), while
an invalid `--conversation` or `--conversation-target` argument on the command line is an
error.

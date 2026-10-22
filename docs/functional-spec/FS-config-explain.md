# FS-config-explain: grund config explain answers, from the installed binary and in the version the project's config is read as, what any key accepts and means and what any check code means and what turns it on

`grund config explain` is the rung of `grund config` that says what a setting or a check code *is*, where `config show` says what this project sets ([§FS-config.4](FS-config.md#4-validation-and-inspection)). It answers for every table and key the configuration reader admits and every code `check` can report ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), in the configuration version the project's file is read as, from what the binary itself carries: no network, no other checkout, and nothing the repository configures is run. An agent that meets a finding code or a key it does not know asks the binary it is running rather than a page that may describe another version or another release, which is [§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work)'s loop closed for the tool's own vocabulary. It answers in rungs, one target at a time, and prints everything only when asked ([§GOAL-token-economy.1](../goals.md#1-what-this-requires)). The requirement it realizes is [§REQ-discoverable-offline](../requirements/REQ-discoverable-offline.md#req-discoverable-offline-the-installed-grund-answers-for-every-setting-and-every-check-code); why it is a new subcommand rather than more `config show` is [§DF-config-explain](../decisions/functional/DF-config-explain.md#df-config-explain-the-installed-grund-explains-its-own-settings-and-codes-under-grund-config).

## terms: Terms

Leans on [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion) and [§FS-config-v2.terms](FS-config-v2.md#terms-terms) (reader). Defines:

- **Target**: what one call asks about: a check code, a table, a key, or a member of an inline table, spelled as [§FS-config-explain.1.1](FS-config-explain.md#11-targets) says.
- **Answer**: what `explain` prints on stdout for a target, in one configuration version or in both.
- **Rung**: how much one call prints: the index, one table, one key or code, every code, or everything ([§FS-config-explain.1.2](FS-config-explain.md#12-rungs)).
- **Gate**: the list a reader admits a table's keys from, or the catalog's list of codes. An answer enumerates from the gates, so it names exactly what the reader accepts.

## 1. The command

```text
grund config explain [TARGET] [PATH] [--path PATH] [--codes | --all]
```

The first positional is the target and the second the path. `--path PATH` names the path as a flag, as [§FS-show.1.4](FS-show.md#14-path-and---path) does, and leaves the first positional the target. `--codes` and `--all` take no target and do not combine. A second target, a target beside `--codes` or `--all`, or both flags at once is a usage error, exit `2`, answered before anything is loaded ([§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read--realized-directional-for-the-next-flag)). `--format` is refused as it is for every `config` command, with `` error: unknown flag `--format` ``, exit `2` ([§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)).

### 1.1 Targets

A target is read in this order:

- A target that starts with `[` is a table, or a key or an inline member of one. `'[schema.leads.words]'` and `'[[kinds]]'` are tables. `'[schema.leads.words] warn'`, `'[[kinds]] fetch'` and the dotted `'[schema.leads.words].warn'` are keys. `'[reference] lead_size_warning.max'` is a member of an inline table.
- A table keyed by a name is given with a name, `'[schema.kinds.FS]'`, or with its placeholder, `'[schema.kinds.<NAME>]'`, and is answered as its placeholder. Any name stands for the placeholder, so the answer is the same whatever kind the project declares.
- Any other target is a top-level key, such as `grund_config_version`, where the version admits one by that name, and otherwise a check code. A dotted path such as `schema.sources.exclude` is neither, so it is not answered; where it names a table or a key the version admits, the failure names the bracketed spelling that would answer ([§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know)).

The quotes are the shell's: a target holding a space or a bracket is one argument. `grund config --help` shows one example of each kind of target, a table, a key of one and a check code, so the brackets are taught where the command is.

### 1.2 Rungs

- **No target** prints the index: one line per table the version admits, with what the table is for, a line for the top-level keys, and the number of check codes.
- **A table** prints what the table is for and one line per key it admits, each with its values and its meaning ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer)). A table that holds only tables names the tables under it.
- **A key, a member or a code** prints that one answer ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer), [§FS-config-explain.4](FS-config-explain.md#4-a-codes-answer)).
- **`--codes`** prints every code the version can report, one line each with its meaning, in catalog order.
- **`--all`** prints every table with its keys and then every code, as their own rungs print them.

Every rung but `--all` ends in one `Next:` line that names the call for the rung below it, where there is one. Every list an answer prints is in a fixed order, so the same binary and the same file give the same bytes ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)).

### 1.3 Streams and exits

- The answer goes to stdout, exit `0`.
- A target the version does not know is a query that found nothing: [§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know), exit `1`.
- A usage error, or a config the command cannot read a version from ([§FS-config-explain.2](FS-config-explain.md#2-which-version-answers)), is exit `2` with one `error:` line on stderr and nothing on stdout.

## 2. Which version answers

The config is discovered as `config show` discovers it, by walking up from the path, `.` when none is given ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)). A path inside a workspace member answers for that member's config, and no member is expanded. The answer is in the version the file is read as ([§FS-config.5](FS-config.md#5-schema-versioning)): an explicit `2` is version 2, and an omitted version or an explicit `1` is version 1. Each answer names that version once, before what it says.

`explain` reads the file's version and nothing else of it. A file whose tables or keys the reader refuses is still answered in its own version, because that is when the question is asked: this is the one other command, beside `config validate` and the hidden completion helper, that a refused file does not stop ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior)). A file whose version cannot be read, because it does not parse as TOML or because its `grund_config_version` names a version this grund does not read, gets the same refusal every command gives it, exit `2`.

With no config file, an answer gives both versions, version 2 first, each under its own name, after one line that says which version is which:

```text
No grund.toml here. grund init writes version 2; until a file exists, check reads version 1's defaults.
```

A target only one version knows is answered in that one, and for the other the answer says what [§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know) would, still exit `0`. A target neither version knows is exit `1`.

The answer never prints what the project sets: that is `config show`'s ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)). `config show` and `grund schema` keep their bytes.

## 3. A key's answer

A key's answer gives the values the key accepts and what it does. The values are put into words by the rule below, from the schema the binary carries; what it does is the key's description in that schema, as the schema writes it, with no specification ID in it. An inline member is answered the same way as a key. A key whose name is any name, such as a kind in `[rules.resolution]`, is answered once under its placeholder, with what the name stands for.

Every key the reader admits has an answer, and every answer is for a key the reader admits: the keys come from the reader's gates, not from the schema, and the schema only words them. A version-1 key and a version-2 key of the same name are two keys, each answered in its own version.

### 3.1 Values in words

One rule puts a key's type into words, and the rendered configuration page uses the same words, its Markdown aside ([§FS-config.4](FS-config.md#4-validation-and-inspection)):

- a closed set of values is each value as TOML, joined by ` | `, such as `"strict" | "warn" | "loose"`;
- an integer with both bounds is `integer, 1–6`, and with a lower bound alone `integer, at least 1`;
- a list is `array of` followed by its items' closed set, or by the plural of their type, such as `array of strings`;
- an inline table is `inline table { max, unit }`, naming its members;
- a value of several forms is each form's words, joined by ` or `;
- anything else is its type: `string`, `boolean`, `integer` or `table`.

## 4. A code's answer

A code's answer gives what the code means, its severity, and what turns it on, in each version that answers, read from the code's row of the catalog by [§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read):

- **What it means** is the row's *Meaning*, as written.
- **Its severity** says which of these the cell is: fixed, with the severity; a suggestion, shown with `--suggestions`; spent, with what became of it; or chosen by a setting, with the setting and, for a strength, what each strength gives (`must` an error, `warn` a warning, `should` a suggestion, `may` not checked).
- **What turns it on** is `always` for `—`; for `not in version N`, that a file read as version N never turns it on, and, in version 1, that `grund config migrate --to 2` gets to a version that can; and otherwise each clause, saying whether a setting, a flag, either or a condition of the tree meets it, with its remark.
- **A ramp**, where the row's *Ramp* names one, says the release and the severity it moves to.

Every setting a version's clause names is one that version's reader admits, so a version-2 answer never names a version-1 spelling, and the reverse. Where a clause names a table, the answer's `Next:` line asks for it. A row marked `planned` has no answer: its code is unknown ([§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know)). The severity the answer states is the one `check` reports by, because both read the one row ([§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree)).

## 5. A target the version does not know

A target the version does not know prints nothing on stdout and one bare query failure on stderr, exit `1` ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)). It names the target and the version, and the next step:

- A version-1 table asked under version 2 names the version-2 tables its settings are written in, from the same list the version-2 reader's refusal of that table names ([§FS-config-v2.reader.1.v1-tables](FS-config-v2.md#reader1v1-tables-a-v1-table-names-the-v2-tables-its-settings-are-written-in)).
- A version-2 table asked under version 1 says it is a version-2 table and names `grund config migrate --to 2`.
- A key its table does not admit names the keys the table admits, as the reader's refusal of that key does ([§FS-config-v2.reader.1.keys](FS-config-v2.md#reader1keys-an-unknown-key-names-the-keys-its-table-admits)).
- A target that does not start with `[`, is no top-level key and no code, but read as a dotted TOML key names a table or a key the version admits, names the bracketed spelling that answers: `` version 2 has no top-level key or check code `schema.sources`; for the table, ask grund config explain '[schema.sources]' ``, and for a key `` …; for the key, ask grund config explain '[schema.sources] exclude' ``. The dotted path is read against the version's own tables: the longest leading run of its segments that spells an admitted table is the table, so under version 1 `fmt.cross_refs` is the table `[fmt.cross_refs]` and not a key of `[fmt]`, and the rest, joined by `.`, is a key or an inline member of it, so `reference.lead_size_warning.max` is `'[reference] lead_size_warning.max'`. A name stands for a placeholder as [§FS-config-explain.1.1](FS-config-explain.md#11-targets) says, and the spelling keeps the name as written: `schema.kinds.FS` names `'[schema.kinds.FS]'`. Where the rest names no key of that table, or no leading run spells a table, the path names nothing and takes the line below; a bare name that is no path from the top, such as `sources` under version 2, is one of those.
- Anything else, a misspelt or a planned code included, names no table, key or code under any spelling and says so, and how to list what there is: `grund config explain` for the tables, `grund config explain --codes` for the codes.

A refusal is not an answer: the reader's refusals keep their own bytes, and the failure above borrows their lists, never their text ([§FS-config-v2.reader.1](FS-config-v2.md#reader1-tables-and-keys-are-closed)).

## 6. It is a read

`explain` answers from what the binary carries: the readers' gates, the code catalog and the schema it embeds. It opens no network connection and reads no checkout but the config file's version. Nothing the repository configures runs: no fetcher, no scan, no workspace expansion, and no member's config is loaded ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)). An answer for a project whose kind configures a fetcher is the same answer, and the fetcher does not run.

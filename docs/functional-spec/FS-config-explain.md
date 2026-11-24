# FS-config-explain: grund config explain answers, from the installed binary and in the version the project's config is read as, what any key accepts and means and what any check code means and what turns it on

`grund config explain` says what a setting or a check code *is*, where `config show` says what this project sets ([§FS-config.4](FS-config.md#4-validation-and-inspection)). It answers for every table and key the configuration reader admits and every code `check` can report ([§FS-errors.5.5](FS-errors.md#55-the-check-code-catalog)), in the version the project's file is read as, from the installed binary alone. An agent that meets a key or a code it does not know asks the binary it is running rather than a page that may describe another release ([§GOAL-agent-grounding](../goals.md#goal-agent-grounding-agents-stay-cited-as-they-work)), one target at a time ([§GOAL-token-economy.1](../goals.md#1-what-this-requires)). It realizes [§REQ-discoverable-offline](../requirements/REQ-discoverable-offline.md#req-discoverable-offline-the-installed-grund-answers-for-every-setting-and-every-check-code), as [§DF-config-explain](../decisions/functional/DF-config-explain.md#df-config-explain-the-installed-grund-explains-its-own-settings-and-codes-under-grund-config) decides.

## terms: Terms

Leans on [§FS-terms.terms.5](FS-terms.md#terms5-findings) (finding, severity, suggestion) and [§FS-config-v2.terms](FS-config-v2.md#terms-terms) (reader). Defines:

- **Target**: what one call asks about: a check code, a table, a key, or a member of an inline table, spelled as [§FS-config-explain.1.1](FS-config-explain.md#11-targets) says.
- **Answer**: what `explain` prints on stdout for a target, in one configuration version or in both.
- **Rung**: how much one call prints: the index, one table, one key or code, every code, or everything ([§FS-config-explain.1.2](FS-config-explain.md#12-rungs)).
- **Gate**: the list a reader admits a table's keys from, or the catalog's list of codes.

## 1. The command

```text
grund config explain [TARGET] [PATH] [--path PATH] [--codes | --all]
```

### 1.1 Targets

The first positional is the target and the second the path, which `--path` may give instead, as in [§FS-show.1.4](FS-show.md#14-path-and---path). A target is read in this order:

- One that starts with `[` is a table, `'[schema.leads.words]'` or `'[[kinds]]'`; a key of one, `'[schema.leads.words] warn'` or `'[schema.leads.words].warn'`; or a member of an inline table, `'[reference] lead_size_warning.max'`. A table keyed by a name, `'[schema.kinds.FS]'`, is answered as its placeholder, `'[schema.kinds.<NAME>]'`.
- Any other is a top-level key the version admits, then a check code, and otherwise a dotted path, read against the version's tables ([§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know)). A path with exactly one reading is answered as its bracketed spelling is, exit `0`: `schema.sources` as `'[schema.sources]'`, and the one segment `schema`, which spells a table from the top, as `'[schema]'`.
- A path with more than one reading, and a bare name, one segment that is no path from the top, such as `exclude`, even where only one table holds it, are not answered ([§DF-config-explain-dotted-answered.2](../decisions/functional/DF-config-explain-dotted-answered.md#2-decision)); nor is a path into a table only the other version admits. Each fails as [§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know) says.

The quotes are the shell's: a target holding a space or a bracket is one argument. `grund config --help` shows one example each of a table, a key of one and a check code.

### 1.2 Rungs

- **No target** prints the index: one line per table the version admits, with what it is for, a line for the top-level keys, and the number of check codes.
- **A table** prints what it is for and each key it admits with its values and meaning ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer)); a table that holds only tables names the tables under it. Under version 2, `[schema.kinds]` also prints the key rows of `[schema.kinds.<NAME>]`, the ones that table's own answer prints and in its order, labelled as that child's keys and not its own, even with no kind declared. No other table expands one under it, and no table below `[schema.kinds.<NAME>]` is expanded.
- **A key, a member or a code** prints that one answer ([§FS-config-explain.3](FS-config-explain.md#3-a-keys-answer), [§FS-config-explain.4](FS-config-explain.md#4-a-codes-answer)).
- **`--codes`** prints every code the version can report with its meaning, one line each, in catalog order.
- **`--all`** prints every table with its keys and then every code, each as its own rung prints it but with no `Next:` line, and the key rows of `[schema.kinds.<NAME>]` once, under that table.

Every rung but `--all` ends in one `Next:` line naming the call for the rung below it, where there is one. Every list an answer prints is in a fixed order ([§REQ-deterministic-output](../requirements/REQ-deterministic-output.md#req-deterministic-output-same-input-same-bytes)).

### 1.3 Streams and exits

- The answer goes to stdout, exit `0`.
- A target the version does not know is exit `1` ([§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know)).
- A usage error, or a config the command cannot read a version from ([§FS-config-explain.2](FS-config-explain.md#2-which-version-answers)), is exit `2` with one `error:` line on stderr and nothing on stdout. A second target, a target beside `--codes` or `--all`, both flags at once, and `--format` are usage errors, answered before anything is loaded ([§FS-cli.requirements.before-load](FS-cli.md#requirementsbefore-load-a-usage-mistake-is-answered-before-the-tree-is-read)); `--format` gets `` error: unknown flag `--format` ``, as from every `config` command ([§FS-errors.5](FS-errors.md#5-json-format), [§FS-cli.3](FS-cli.md#3-cross-subcommand-flags)).

## 2. Which version answers

The config is discovered as `config show` discovers it, stopping at a workspace member's boundary ([§FS-config.4.2](FS-config.md#42-grund-config-show-path)), so a path inside a member answers for that member's config. The answer is in the version the file is read as ([§FS-config.5](FS-config.md#5-schema-versioning)) and names it once, before what it says; it prints nothing else the project sets, which is `config show`'s to print. `explain` reads that file's version and nothing else of it, so it answers a file every other command refuses for its tables, its keys or its TOML syntax ([§FS-config.4.3](FS-config.md#43-invalid-config-behavior), [§DF-config-explain.2.5](../decisions/functional/DF-config-explain.md#25-it-reads-only-the-version)); a file that starts with a byte-order mark, or whose version this grund does not read, gets the refusal every command gives, exit `2`.

With no config file, as in a workspace member with neither config form, an answer gives both versions, version 2 first, each under its own name, after one line:

```text
No grund.toml here. grund init writes version 2; until a file exists, check reads version 1's defaults.
```

A target only one version knows is answered in that one, and for the other the answer says what [§FS-config-explain.5](FS-config-explain.md#5-a-target-the-version-does-not-know) would, and the call exits `0`; a target neither knows is exit `1`.

## 3. A key's answer

A key's answer gives the values it accepts, in words ([§FS-config-explain.3.1](FS-config-explain.md#31-values-in-words)), and what it does: its description in the schema the binary carries, as the schema writes it, with no specification ID in it. An inline member is answered as a key is, and a key whose name is any name, such as a kind in `[rules.resolution]`, is answered once, under its placeholder, with what the name stands for.

The keys come from the readers' gates, which the schema only words, so every key a reader admits has an answer and every answer is for a key it admits ([§DF-config-explain.2.4](../decisions/functional/DF-config-explain.md#24-it-enumerates-from-what-the-readers-admit)). A version-1 key and a version-2 key of the same name are two keys, each answered in its own version.

### 3.1 Values in words

One rule puts a key's type into words, and the rendered configuration page uses the same words, its Markdown aside ([§FS-config.4](FS-config.md#4-validation-and-inspection)):

- a closed set of values is each value as TOML, joined by ` | `, such as `"strict" | "warn" | "loose"`;
- an integer with both bounds is `integer, 1–6`, and with a lower bound alone `integer, at least 1`;
- a list is `array of` followed by its items' closed set, or by the plural of their type, such as `array of strings`;
- an inline table is `inline table { max, unit }`, naming its members;
- a value of several forms is each form's words, joined by ` or `;
- anything else is its type: `string`, `boolean`, `integer` or `table`.

## 4. A code's answer

A code's answer puts the code's row of the catalog into words, in each version that answers, as [§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read) reads it:

- its *Meaning*, as written;
- its severity, saying which of four the cell is: fixed, with the severity; a suggestion, shown with `--suggestions`; spent, with what became of it; or chosen by a setting, with the setting and, for a strength, what each strength gives;
- what turns it on: `always` for `—`; for `not in version N`, that the version never turns it on and, in version 1, `grund config migrate --to 2`; otherwise each clause, saying whether a setting, a flag, either or the tree meets it, with its remark;
- a ramp's release and the severity it moves to.

Because `check` reads the same row, the severity is the one `check` reports by, and the fix, filled from a finding, is the remedy that finding's message carries ([§REQ-one-answer](../requirements/REQ-one-answer.md#req-one-answer-every-command-gives-the-same-answer-about-one-tree), [§DF-config-explain.2.3](../decisions/functional/DF-config-explain.md#23-the-catalog-and-the-schema-are-rendered-never-retyped)).

The fix is the version's side of *Fix*, slots unfilled and backticks kept, a reference printed as the template it stands for: `Fix: <template>`, one `Fix, <label>: <template>` line per case in the cell's order, or `Fix: none — <why>`, which is `Fix: none — not in version N` for a side that is `not in version N`. No line is wrapped or given a closing period. Where the version's side of *Note* is not `—`, it is one `Note:` line after the fix.

Every setting a clause names is one that version's reader admits, so a version-2 answer never names a version-1 spelling, and the reverse; where a clause names a table, the `Next:` line asks for it.

## 5. A target the version does not know

A target the version does not know prints one bare query failure, exit `1` ([§FS-errors.2.3](FS-errors.md#23-bare-query-failure)), naming the target, the version and the next step. A version-1 table asked under version 2 names the version-2 tables its settings are written in, from the list the version-2 reader's refusal of that table names ([§FS-config-v2.reader.1.v1-tables](FS-config-v2.md#reader1v1-tables-a-v1-table-names-the-v2-tables-its-settings-are-written-in)); a version-2 table asked under version 1 says it is one and names `grund config migrate --to 2`; and a key its table does not admit names the keys the table admits, the list the reader's refusal of that key names ([§FS-config-v2.reader.1.keys](FS-config-v2.md#reader1keys-an-unknown-key-names-the-keys-its-table-admits)).

A target that does not start with `[` and is no top-level key or code is read as a path, by the first of these that names something; a path with an empty segment or a segment holding a bracket, such as `schema.kinds.`, gets the last:

- **The version's own tables.** The longest leading run of segments that spells an admitted table is the table, and the rest is a key or an inline member of it: under version 1 `fmt.cross_refs` is the table `[fmt.cross_refs]`, not a key of `[fmt]`. Where the rest names no key or member of that table, or no run spells a table, the reading names nothing. Where the longest-run reading fills a placeholder of a table's header with a segment, the path also reads with the name left out, the placeholder written in front of that segment and the path read again. A segment that spells a table by that table's own name, such as `grounding` in `rules.citations.grounding`, fills no placeholder, and a reading that names nothing is not counted, so that path and `schema.kinds.FS` read once and are answered ([§FS-config-explain.1.1](FS-config-explain.md#11-targets)). Several readings are all named, those with a name left out first ([§DF-config-explain-dotted-answered.4](../decisions/functional/DF-config-explain-dotted-answered.md#4-alternatives-considered)): the first on the line, as `for the key` or `for the table`, and each other on a `hint:` line of its own, where the reading that takes a segment as a name says what it would name, a kind for `<NAME>` and `<KIND>`, a language for `<name>` and a field for `<field>`:

  ```text
  version 2 has no top-level key or check code `schema.kinds.id_format`; for the key, ask grund config explain '[schema.kinds.<NAME>] id_format'
  hint: for a kind named id_format, ask grund config explain '[schema.kinds.id_format]'
  ```

- **The other version's tables.** A path whose longest leading run spells a table only the other version admits gets the failure that table's bracketed spelling gets, whatever the rest: under version 2 `reference.strict` gets what `'[reference]'` gets. A table both versions admit, such as `[workspace]`, is the version's own.
- **A bare name**, one segment that is no path from the top in either version, names each table that holds a key of exactly that name and each whose header ends in it, in the order the index lists them ([§FS-config-explain.1.2](FS-config-explain.md#12-rungs)); no placeholder matches a name. The failure joins the spellings with `, ` and ` or ` before the last, after `for a key of a table` where all are keys, `for a table` where all are tables and `for a table or a key of one` where they mix:

  ```text
  version 2 has no top-level key or check code `exclude`; for a key of a table, ask grund config explain '[schema.sources] exclude' or '[presentation.fmt] exclude'
  ```

- **Nothing.** The failure says the target, a misspelt or a `planned` code included ([§FS-errors.5.5.1](FS-errors.md#551-how-a-row-is-read)), names no table, key or code, and that `grund config explain` lists the tables and `grund config explain --codes` the codes.

## 6. It is a read

`explain` answers from what the binary carries. It opens no network connection, reads of the tree only the selected config's version and the enclosing workspace's membership, and runs nothing the repository configures: no fetcher, no scan, no workspace expansion, and no other member's config is loaded ([§REQ-runs-offline.1](../requirements/REQ-runs-offline.md#1-read-and-verification-paths-execute-nothing)).

# Asking the binary what a key or a check code means

`grund config explain` says what a table, a key or a check code *is*, from the
grund you are running and in the version your `grund.toml` is read as. It needs
no network and no other checkout, and it runs nothing the repository configures.
Use it when a finding names a code you do not know, or a guide names a key and
you are not sure your config's version has it. `grund config show` is the other
half: it says what this project *sets*. The contract is
[§FS-config-explain](../functional-spec/FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on); the runnable
[`examples/config-explain/`](../../examples/config-explain/) prints each answer
below.

```bash
grund config explain                                # the tables, and how many codes
grund config explain '[schema.leads.words]'         # a table and its keys
grund config explain '[schema.leads.words] warn'    # one key
grund config explain oversized-lead                 # a check code
grund config explain --codes                        # every code, one line each
grund config explain --all                          # everything, at once
grund config explain oversized-lead docs/           # the config docs/ is under
```

Quote a target that holds a space or a bracket, so the shell passes it as one
argument. The first positional is the target and the second the path; `--path`
names the path as a flag instead.

## It answers in rungs

With no target, the answer is the index: the top-level keys, one line per table
with what it is for, and the number of check codes. Every answer but `--all`
ends in a `Next:` line that names the call one rung down, so you can follow it
rather than read everything:

```text
grund.toml is read as version 2.

Top-level keys: grund_config_version, project_name
[schema]  What exists: the citation marker, the ID grammar, the sources, the note and lead measures, and the kinds.
[schema.sources]  The sources a version-2 project is read from.
...
[workspace]  The sub-projects a workspace root checks, each in its own namespace.
65 check codes: grund config explain --codes

Next: grund config explain '[schema]'
```

A table prints each key it admits with its values and its meaning, and a key
prints that one line in full. The values are in the same words the
[configuration reference](configuration.md) uses:

```text
grund.toml is read as version 2.

[schema.leads.words] warn
Values: integer
Words past which a declaration's lead is a warning.
```

A table keyed by names, such as `[schema.kinds.FS]`, is answered as its
placeholder `[schema.kinds.<NAME>]`, so the answer is the same whatever kinds
the project declares.

## A check code

A code's answer says what it means, how severe it is, and what turns it on, read
from the same catalog `grund check` reports by
([§FS-errors.5.5](../functional-spec/FS-errors.md#55-the-check-code-catalog)):

```text
grund.toml is read as version 2.

oversized-lead
A declaration's lead is longer than the configured word limit.

Severity: by strength: [schema.leads.words]. A strength gives: must an error, warn a warning, should a suggestion, may not checked.
On: when [schema.leads.words] warn = <N> is set (a setting)
    a must or should limit turns it on too, at that strength

Next: grund config explain '[schema.leads.words]'
```

The `On:` line says whether a setting, a flag, either, or a condition of the
tree meets each clause, and every setting it names is spelled in your file's
version. A code that version never reports says so, and in version 1 it names
`grund config migrate --to 2`, the way to a version that can.

## Which version answers

The config is found as `config show` finds it, by walking up from the path, and
the answer is in the version that file is read as. `explain` reads that version
and nothing else of the file, so a `grund.toml` that `grund check` refuses is
still answered: that is usually when you are asking. Only a file whose version
cannot be read is refused, exactly as every command refuses it.

With no `grund.toml` at all, the answer gives both versions, version 2 first,
after one line saying which is which:

```text
No grund.toml here. grund init writes version 2; until a file exists, check reads version 1's defaults.
```

## A target your version does not have

A target the version does not know prints nothing on stdout and one line on
stderr, exit `1`, naming the next step. A version-1 table asked in a version-2
project says so, and a `hint:` line names where version 2 writes its settings:

```text
version 2 has no table `[scan]`: it is a version 1 table
hint: version 2 writes its settings in [schema.sources], [schema.sources.extensions], [schema.sources.definitions.*], [schema.kinds.code]
```

A version-2 table asked in a version-1 project names
`grund config migrate --to 2`; a key the table does not have lists the keys it
does. A target written the way TOML writes a nested key, such as
`schema.sources` or `schema.sources.exclude`, is not answered: the brackets are
part of a table's target. Where the dotted path names a table or a key the
version has, the line gives the spelling that answers:

```text
version 2 has no top-level key or check code `schema.sources.exclude`; for the key, ask grund config explain '[schema.sources] exclude'
```

Anything else says how to list what there is: `grund config explain`
for the tables, `grund config explain --codes` for the codes. A usage mistake,
or a file whose version cannot be read, is exit `2` with one `error:` line.

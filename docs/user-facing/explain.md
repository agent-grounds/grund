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

To learn how to declare the first kind, ask `grund config explain schema.kinds`
or `grund config explain '[schema.kinds]'`. Alongside the child table name, the
answer prints `Keys of [schema.kinds.<NAME>]:` and all of that child's key rows,
with their values and meanings, even when no kind is declared
([§FS-config-explain.1.2](../functional-spec/FS-config-explain.md#12-rungs)). These
keys belong in a table such as `[schema.kinds.ADR]`. Other parent tables name
their children without expanding their keys; the index stays one line per
table, and `--all` prints the kind keys once under their owning child.

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
Fix: move detail into citable child sections, or promote a child section to its own ID after running grund refs {coordinate} --summary

Next: grund config explain '[schema.leads.words]'
```

The `On:` line says whether a setting, a flag, either, or a condition of the
tree meets each clause, and every setting it names is spelled in your file's
version. A code that the current configuration never reports says so.

The `Fix:` line is what clears the finding, worded once in the code's catalog
row and carried by every message of that code with its `{slots}` filled in
([§FS-errors.5.5.1](../functional-spec/FS-errors.md#551-how-a-row-is-read),
[§FS-config-explain.4](../functional-spec/FS-config-explain.md#4-a-codes-answer)).
A code with more than one way out prints one `Fix, <case>:` line per way, and a
code nothing you edit can clear prints `Fix: none —` and why. A `Note:` line,
when there is one, says what the check compares or what not to do instead:
`grund config explain section-address-changed` names the three fixes for a moved
section and says that a lock is never edited by hand, because only
`grund fmt --write` writes one and it never approves a move.

## Which version answers

The config is found as `config show` finds it, by walking up from the path, and
the answer is in the version that file is read as. `explain` reads that version
and nothing else of the file, so a `grund.toml` that `grund check` refuses is
still answered: that is usually when you are asking. Only a file whose version
cannot be read is refused, exactly as every command refuses it.

The upward search keeps the path you supplied through directory links
([§FS-config.1.5](../functional-spec/FS-config.md#15-the-ancestor-search-keeps-the-path-through-directory-links)).
If `e2e/cases` links outside your project, `grund check e2e/cases/b`,
`grund config explain --path e2e/cases/b`, and
`grund config migrate --to 2 e2e/cases/b` still search the ancestors of
`e2e/cases/b` in your project. They do not jump to a config above the link's
target. Absolute paths through the same link work the same way. The selected
config still applies its usual validation: version 2 refuses a kind home linked
outside the project. Fix the named home or file as the finding directs.


The same answer applies inside a workspace member with neither config form,
including its descendants and an explicit `--path` into it: discovery stops at
that member's boundary ([§FS-config-explain.2](../functional-spec/FS-config-explain.md#2-which-version-answers)).
There, `grund config show` reports the member's canonical version-1 defaults
([§FS-config.4.2](../functional-spec/FS-config.md#42-grund-config-show-path)),
and `grund schema` names the member with `defaults, no grund.toml · v1`
([§FS-schema.2](../functional-spec/FS-schema.md#2-selection)).
Run `grund init` inside that member of a version-controlled tree to write its
own version-2 `grund.toml`
([§FS-init.2.4](../functional-spec/FS-init.md#24-generated-grundtoml)).

## A target your version does not have

An unsupported table prints nothing on stdout and one line on stderr, exit `1`,
naming the next step and the current table that holds those settings. For
source settings, the hint points to `[schema.sources]` and
`[schema.kinds.code]`.

A key the table does not have lists the keys it does. A target written the way TOML
writes a nested key, such as `schema.sources` or `schema.sources.exclude`, is
read against the current tables. Where it reads one way only, it is answered
exactly as its bracketed spelling is. Where a segment could be a kind's name
or a key of the kind's table, the path reads two ways, so it is not answered.

A bare name, such as `exclude`, is not answered either, but the failure names
every table that holds a key of that name, or whose header ends in it:

```text
version 2 has no top-level key or check code `exclude`; for a key of a table, ask grund config explain '[schema.sources] exclude' or '[presentation.fmt] exclude'
```

A dotted path such as `schema.sources.exclude` is answered as its table
spelling would be. `[workspace]` is read from the selected configuration.

Anything else says how to list what there is: `grund config explain`
for the tables, `grund config explain --codes` for the codes. A usage mistake,
or a file whose version cannot be read, is exit `2` with one `error:` line.

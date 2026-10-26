# Config explain: ask the binary what a key or a check code means

This runnable walkthrough asks `grund config explain` about a version-2 project ([§FS-config-explain](../../docs/functional-spec/FS-config-explain.md#fs-config-explain-grund-config-explain-answers-from-the-installed-binary-and-in-the-version-the-projects-config-is-read-as-what-any-key-accepts-and-means-and-what-any-check-code-means-and-what-turns-it-on)). It is for anyone who meets a check code or a config key they do not know, an agent above all: the answer comes from the grund that is running, in the version the project's `grund.toml` is read as, with no network and no other checkout. Open `explain-1-index/repo/grund.toml` first. It is version 2, declares one kind, and sets one lead limit, `[schema.leads.words] warn = 120`.

The example pins five runs, one per step, each against the same config ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). Every run reads the config and writes nothing.

## Step 1: the index

With no target, the answer is every table the version admits, with what each is for, and the number of check codes ([§FS-config-explain.1.2](../../docs/functional-spec/FS-config-explain.md#12-rungs)):

```bash
grund config explain --path examples/config-explain/explain-1-index/repo
echo $?    # 0
```

```text
grund.toml is read as version 2.

Top-level keys: grund_config_version, project_name
[schema]  What exists: the citation marker, the ID grammar, the sources, the note and lead measures, and the kinds.
…
[workspace]  The sub-projects a workspace root checks, each in its own namespace.
66 check codes: grund config explain --codes

Next: grund config explain '[schema]' examples/config-explain/explain-1-index/repo
```

The first line names the version once, because every answer depends on it. The last line is the call one rung down; each answer but `--all` ends in one, so you follow the answer rather than read everything. `explain-1-index/expected.stdout` holds the whole index.

## Step 2: a key

```bash
grund config explain '[schema.leads.words] warn' examples/config-explain/explain-2-key/repo
echo $?    # 0
```

```text
grund.toml is read as version 2.

[schema.leads.words] warn
Values: integer
Words past which a declaration's lead is a warning.
```

The quotes are the shell's: a target holding a space or a bracket is one argument. The answer does not say `120`. `explain` reads the file's version and nothing else of it, so it says what the key *is*; `grund config show` says what this project sets it to.

## Step 3: a check code

```bash
grund config explain oversized-lead examples/config-explain/explain-3-code/repo
echo $?    # 0
```

```text
grund.toml is read as version 2.

oversized-lead
A declaration's lead is longer than the configured word limit.

Severity: by strength: [schema.leads.words]. A strength gives: must an error, warn a warning, should a suggestion, may not checked.
On: when [schema.leads.words] warn = <N> is set (a setting)
    a must or should limit turns it on too, at that strength
Fix: move detail into citable child sections, or promote a child section to its own ID after running grund refs {coordinate} --summary

Next: grund config explain '[schema.leads.words]' examples/config-explain/explain-3-code/repo
```

The answer is read from the same catalog row `grund check` reports by, so the severity it states is the one a finding carries ([§FS-config-explain.4](../../docs/functional-spec/FS-config-explain.md#4-a-codes-answer)). This project sets `warn = 120`, so `oversized-lead` is a warning here. The setting is spelled in version 2, the file's version; in a version-1 project the same code names `[reference] lead_size_warning` instead.

## Step 4: a table from the other version

Guides written for version 1 name `[scan]`. Ask for it here:

```bash
grund config explain '[scan]' examples/config-explain/explain-4-version-1-table/repo
echo $?    # 1
```

Stdout is empty. Stderr says that `[scan]` is a version 1 table, and its `hint:` line says where version 2 writes what it held ([§FS-config-explain.5](../../docs/functional-spec/FS-config-explain.md#5-a-target-the-version-does-not-know)):

```text
version 2 has no table `[scan]`: it is a version 1 table
hint: version 2 writes its settings in [schema.sources], [schema.sources.extensions], [schema.sources.definitions.*], [schema.kinds.code]
```

Exit `1` is a query that found nothing, as with `grund show` for an unknown ID; exit `2` is kept for a usage mistake or a file whose version cannot be read. A misspelt target says how to list what there is: `grund config explain` for the tables, `grund config explain --codes` for the codes.

## Step 5: what clears a finding

A check's message ends in what clears it, and the code's answer prints those
words once, with the slots the message fills left as `{names}`, followed by a
note on what the check compares
([§FS-errors.5.5.1](../../docs/functional-spec/FS-errors.md#551-how-a-row-is-read)):

```bash
grund config explain section-address-changed examples/config-explain/explain-5-fix/repo
echo $?    # 0
```

```text
grund.toml is read as version 2.

section-address-changed
A cited section's text has moved to another address, and the cited address now holds something else.

Severity: error, fixed.
On: when a citation names a relative heading's address (a condition of the tree)
Fix: move it back, cite {new address} instead, or leave a pointer to it at {cited address}; if {new address} is a copy, edit the copy or cite it from {cited address}
Note: check compares the text at a cited address with the fingerprint .grund/sections/<ID>.lock recorded for it, and finds that fingerprint at another address of the declaration. Only grund fmt --write writes a lock (grund config explain unrecorded-section-citation says where to run it). It never approves a move: the one thing it records at the old address is a pointer to the new one. Do not edit a lock by hand.
```

The `Fix:` line is the same three ways out every `section-address-changed`
error names: undo the move, follow it, or acknowledge it with a pointer. The
`Note:` line is the rule that keeps you from a fourth way that only silences
the error: the lock is written by `grund fmt --write` and never by hand.
[`relative-sections-lock/`](../relative-sections-lock/) is a tree to try it on.

## What to take from it

Ask the binary, not a page: a guide may describe another version or another release, and `explain` answers for exactly the grund and the config in front of you. The trade-off is that it says what a setting means, never what this project sets, so `grund config show` stays the other half of the question. The [user guide](../../docs/user-facing/explain.md) has every rung and flag.

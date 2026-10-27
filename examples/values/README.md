# Values: one declaration, checked uses

This runnable mini-repository demonstrates [§FS-values](../../docs/functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value) on a physics
simulation, whose integration time step is the kind of number that gets tuned and
then goes stale everywhere it was quoted. It shows a `CONST` kind with Markdown,
JSON and YAML declarations, a section marked independently with `<!-- grund:value -->`, a
chapter whose named children are roots by schema, exact numeric equality, prose and
Python-comment bindings, direct application reads of the JSON source, the deliberate
unbackticked non-binding, and one caught mismatch.

Run it from the repository root:

```bash
grund check examples/values/repo
echo $?    # 1: the stale time step is caught
```

The solver document binds the time step once, as the quantity it is: `1.0e-3 s`
against the whole `CONST-time-step`, with no component coordinate. The literal
is the declaration's components joined by one ASCII space, so `grund` splits it
there and compares each part with its own component — `1.0e-3` with `0.001` as an
exact decimal, `s` with `s` as a string — and neither half can drift
unchecked ([§FS-values.3.1.2](../../docs/functional-spec/FS-values.md#312-a-binding-aimed-at-the-root)). The Python comment still binds the amount alone, as
`1e-3` against `CONST-time-step.1`. That space is also why a value with a
space *inside* a component, such as the JSON damping's `per step`, cannot be bound
at its root: the split could no longer tell its components apart, so that form is
refused and each component is bound by its coordinate instead.

The stability analysis deliberately still assumes `0.002 s`, the step before it was
tuned, producing the golden `value-mismatch` in [`expected.stdout`](expected.stdout):

```text
docs/solver.md:7: error: value mismatch for CONST-time-step.1: bound `0.002`, declared `0.001` at values/time-step.md:2 — whole binding: value mismatch for CONST-time-step (component 1): bound `0.002 s`, declared `0.001 s` at values/time-step.md:2; this wording changes in grund 0.22.0; change the literal to the declared value, or the declaration to the bound one
```

The part after ` — whole binding: ` names what the line wrote: the root it cites,
the literal whole, and component 1 as the one that differs
([§FS-values.5.2.2](../../docs/functional-spec/FS-values.md#522-a-root-aimed-mismatch-names-the-first-unequal-component)). The citation is right, so keep it and correct the literal:
change it to `0.001 s` and the repository passes; its config is on version 1, so
the run prints that caution where `success` would stand
([§FS-check.checks.config-version-1](../../docs/functional-spec/FS-check.md#checksconfig-version-1-a-config-file-read-as-version-1)). The part before it is the message
earlier releases printed, kept whole until grund 0.22.0 drops it
([§FS-errors.3.9](../../docs/functional-spec/FS-errors.md#39-the-root-aimed-value-mismatch-wording-migration)).

The solver's limits live in [`values/limits.yaml`](repo/values/limits.yaml), a
YAML value source beside the JSON one, read the same way and checked the same
way ([§FS-values.2.2.3](../../docs/functional-spec/FS-values.md#223-how-a-yaml-source-is-read)). YAML is there for its comments: each value says in
words where it came from, and the integrator's version is quoted so it stays
the string `1.10` rather than becoming a number. A `§` citation in one of those
comments would be read by nothing, so the provenance is cited from the solver
document instead. Change the bound substep cap `8` to `9` and the run reports
the mismatch at `values/limits.yaml:7`.

`DOC-solver.1` is the second authority form: an ordinary section inside the solver
document, outside any `values = true` kind, owns the strict component
`DOC-solver.1.1` because its heading carries the exact marker
([§FS-values.2.4](../../docs/functional-spec/FS-values.md#24-embedded-section-value-roots)). A root of one component gives the same verdict either way,
so the Courant limit `0.50` agrees at `DOC-solver.1.1` and at `DOC-solver.1` alike.

`DOC-solver.values.gravity` is the third: no marker anywhere, because the
`DOC` row sets `value_chapter = "values"` and every named child of that chapter
is a root in every declaration of the kind ([§FS-config.3.4.13](../../docs/functional-spec/FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values), [§FS-values.2.5](../../docs/functional-spec/FS-values.md#25-chapter-declared-value-roots)). The
coordinate is the name rather than a number, so `gravity` survives anything
inserted or reordered around it. `9.81 m/s²` binds the whole root, amount and unit
together. The chapter needs `[id] named_sections = true`, which this
repository's `grund.toml` sets for that reason alone.

The nearby unbackticked `0.001 (§CONST-time-step.1)` is an ordinary citation,
not inferred value syntax. This keeps intent explicit and avoids guessing which
number in a sentence belongs to a citation.

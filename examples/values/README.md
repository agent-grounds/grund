# Values: one declaration, checked uses

This runnable mini-repository demonstrates [§FS-values](../../docs/functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value) on a physics
simulation, whose integration time step is the kind of number that gets tuned and
then goes stale everywhere it was quoted. It shows a `CONST` kind with Markdown and
JSON declarations, a section marked independently with `<!-- grund:value -->`, a
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
tuned, producing the golden `value-mismatch` below. Change it to `0.001 s` and the
repository prints `success`.

`DOC-solver.1` is the second authority form: an ordinary section inside the solver
document, outside any `values = true` kind, owns the strict component
`DOC-solver.1.1` because its heading carries the exact marker
([§FS-values.2.4](../../docs/functional-spec/FS-values.md#24-embedded-section-value-roots)). A root of one component binds the same either way,
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

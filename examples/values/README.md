# Values: one declaration, checked uses

This runnable mini-repository demonstrates [§FS-values](../../docs/functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value): a `CONST` kind with Markdown and JSON declarations, a section marked independently with `<!-- grund:value -->`, a chapter whose named children are roots by schema, exact numeric equality, prose and Python-comment bindings, direct application reads of the JSON source, the deliberate unbackticked non-binding, and one caught mismatch.

Run it from the repository root:

```bash
grund check examples/values/repo
echo $?    # 1: the intentional discount mismatch is caught
```

The offer binds the field price once, as the quantity it is: `1200.0 USD`
against the whole `CONST-field-price`, with no component coordinate. The literal
is the declaration's components joined by one ASCII space, so `grund` splits it
there and compares each part with its own component — `1200.0` with `1200` as an
exact decimal, `USD` with `USD` as a string — and neither half can drift
unchecked ([§FS-values.3.1.2](../../docs/functional-spec/FS-values.md#312-a-binding-aimed-at-the-root)). The Python comment still binds the amount alone, as
`1.2e3` against `CONST-field-price.1`. That space is also why a value with a
space *inside* a component, such as a street address, cannot be bound at its
root: the split could no longer tell its components apart, so that form is
refused and each component is bound by its coordinate instead.

The discount binding deliberately writes `0.30` against JSON `0.25`, producing
the golden `value-mismatch` below. Change it to `0.25` and the repository prints
`success`.

`DOC-offer.1` is the second authority form: an ordinary section inside the offer
document, outside any `values = true` kind, owns the strict component
`DOC-offer.1.1` because its heading carries the exact marker
([§FS-values.2.4](../../docs/functional-spec/FS-values.md#24-embedded-section-value-roots)). A root of one component binds the same either way,
so `45.0` agrees at `DOC-offer.1.1` and at `DOC-offer.1` alike.

`DOC-offer.values.aux-voltage` is the third: no marker anywhere, because the
`DOC` row sets `value_chapter = "values"` and every named child of that chapter
is a root in every declaration of the kind ([§FS-config.3.4.13](../../docs/functional-spec/FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values), [§FS-values.2.5](../../docs/functional-spec/FS-values.md#25-chapter-declared-value-roots)). The
coordinate is the name rather than a number, so `aux-voltage` survives anything
inserted or reordered around it. `24 V` binds the whole root, amount and unit
together. The chapter needs `[id] named_sections = true`, which this
repository's `grund.toml` sets for that reason alone.

The nearby unbackticked `1200 (§CONST-field-price.1)` is an ordinary citation,
not inferred value syntax. This keeps intent explicit and avoids guessing which
number in a sentence belongs to a citation.

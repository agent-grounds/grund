# Values: one declaration, checked uses

This runnable mini-repository demonstrates [§FS-values](../../docs/functional-spec/FS-values.md#fs-values-opted-in-kinds-bind-authored-components-to-one-declared-value): a `CONST` kind with Markdown and JSON declarations, a section marked independently with `<!-- grund:value -->`, a chapter whose named children are roots by schema, exact numeric equality, prose and Python-comment bindings, direct application reads of the JSON source, the deliberate unbackticked non-binding, and one caught mismatch.

Run it from the repository root:

```bash
grund check examples/values/repo
echo $?    # 1: the intentional discount mismatch is caught
```

The field-price bindings use `1200.0` and `1.2e3` for a declaration written as
`1200`; exact decimal comparison accepts both. The discount binding deliberately
writes `0.30` against JSON `0.25`, producing the golden `value-mismatch` below.
Change it to `0.25` and the repository prints `success`.

`DOC-offer.1` is the second authority form: an ordinary section inside the offer
document, outside any `values = true` kind, owns the strict component
`DOC-offer.1.1` because its heading carries the exact marker
([§FS-values.2.4](../../docs/functional-spec/FS-values.md#24-embedded-section-value-roots)).

`DOC-offer.values.aux-voltage` is the third: no marker anywhere, because the
`DOC` row sets `value_chapter = "values"` and every named child of that chapter
is a root in every declaration of the kind ([§FS-config.3.4.13](../../docs/functional-spec/FS-config.md#3413-value_chapter--the-chapter-whose-named-children-are-values), [§FS-values.2.5](../../docs/functional-spec/FS-values.md#25-chapter-declared-value-roots)). The
coordinate is the name rather than a number, so `aux-voltage` survives anything
inserted or reordered around it. The chapter needs `[id] named_sections = true`,
which this repository's `grund.toml` sets for that reason alone.

The nearby unbackticked `1200 (§CONST-field-price.1)` is an ordinary citation,
not inferred value syntax. This keeps intent explicit and avoids guessing which
number in a sentence belongs to a citation.

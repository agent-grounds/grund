# Field-value rules: a rule over the declarations whose field holds one value

A requirement kind records how each requirement is verified in a scalar field,
`verification`, and only a requirement verified by test needs a test case. A
rule subject ending in a whose clause states that once:
`Each SWR whose verification is Test must be cited by at least one TC.`
([§FS-rules.2.2](../../docs/functional-spec/FS-rules.md#22-a-whose-clause-selects-on-a-scalar-fields-value)). This example is for anyone writing a coverage
rule that depends on what a declaration says about itself, rather than splitting
the kind by value or leaving the rule to review. Its use-case is stating a
convention as a checked rule ([§FS-examples.2](../../docs/functional-spec/FS-examples.md#2-canonical-use-cases)).

The fixture is the same in every run, a version-2 `grund.toml` with four kinds:

- `SWR`, whose scalar field `verification` admits `Test`, `Analysis`,
  `Inspection` and `Demonstration`. [`SWR-brake-latency`](1-every-swr/repo/req/SWR-brake-latency.md)
  is verified by `Test`, and [`SWR-fault-log`](1-every-swr/repo/req/SWR-fault-log.md) by `Inspection`.
- `TC`, the test cases, of which none is written yet.
- `AN`, an analysis that cites both requirements.
- `RULE`, a rule kind declaring [`RULE-tested`](1-every-swr/repo/rules/RULE-tested.md),
  whose title is the filtered sentence.

The example pins five runs ([§FS-examples.5.5](../../docs/functional-spec/FS-examples.md#55-an-example-may-pin-several-runs)). Each runs inside its
`repo/`, and none writes anything.

## Step 1: the rule over every SWR

Without the clause, the rule asks a test case of every requirement, including
the one verified by inspection:

```bash
cd examples/rule-field-value/1-every-swr/repo
grund check --rule 'Each SWR must be cited by at least one TC.' --only-rule
echo $?    # 1
```

```text
req/SWR-brake-latency.md:1: error: SWR-brake-latency is cited by TC 0 times; --rule requires at least one; add or remove the citations of it until the count fits
req/SWR-fault-log.md:1: error: SWR-fault-log is cited by TC 0 times; --rule requires at least one; add or remove the citations of it until the count fits
```

## Step 2: narrow it with a whose clause

```bash
cd examples/rule-field-value/2-whose/repo
grund check --rule 'Each SWR whose verification is Test must be cited by at least one TC.' --only-rule
echo $?    # 1
```

```text
req/SWR-brake-latency.md:1: error: SWR-brake-latency is cited by TC 0 times; --rule, RULE-tested requires at least one; add or remove the citations of it until the count fits
```

`SWR-fault-log` says `Inspection`, so it is outside the rule and nothing
reports it. The clause compares the field's value byte for byte with one of its
`one_of` entries, so `is test` would not match `Test`. The trial sentence is
the one `RULE-tested` already declares, so the two collapse into one finding
that names both authorities ([§FS-rules.6](../../docs/functional-spec/FS-rules.md#6-semantic-deduplication)).

## Step 3: the same sentence as a declared rule

```bash
cd examples/rule-field-value/3-declared/repo
grund check --only uncited-unit
echo $?    # 1
```

```text
req/SWR-brake-latency.md:1: error: SWR-brake-latency is cited by TC 0 times; RULE-tested requires at least one; add or remove the citations of it until the count fits
```

A filtered subject is never kind-wide, so the rule never folds into a
`[rules.citations]` direction: both report where both apply.

## Step 4: the subject as a query

A subject is also a selector ([§FS-rules.8](../../docs/functional-spec/FS-rules.md#8-command-surfaces)), so `list` shows which
declarations the rule reaches before you write it:

```bash
cd examples/rule-field-value/4-selector/repo
grund list --selector 'Each SWR whose verification is Test'
echo $?    # 0
```

```text
SWR-brake-latency  req/SWR-brake-latency.md:1  The brake command reaches the actuator within 10 ms
```

## Step 5: a field the kind does not declare

```bash
cd examples/rule-field-value/5-refused/repo
grund check --rule 'Each SWR whose verfication is Test must be cited by at least one TC.'
echo $?    # 2
```

```text
error: unknown kind "SWR whose verfication is Test" — "verfication" is not a field of kind SWR; its scalar fields: verification; this wording changes in grund 0.22.0
```

The reason after ` — ` names the misspelled field and the scalar fields the kind
declares, and a value outside `one_of` is answered the same way, with the
entries the field admits
([§FS-rule-refusals.sentences.whose](../../docs/functional-spec/FS-rule-refusals.md#sentenceswhose-a-whose-clause-is-refused-for-the-part-that-failed)). Until grund 0.22.0 it keeps the
`unknown kind` reason that grund printed before whose clauses existed as its prefix, so a
consumer matching that prefix keeps working through the window ([§FS-errors.3.10](../../docs/functional-spec/FS-errors.md#310-the-whose-clause-wording-migration)).

## Trade-offs

- A whose clause reads only a scalar field: a version-2 field with `one_of`. A
  prose field or a version-1 config is refused with a reason that says so.
- A declaration whose field is absent, or holds a value outside `one_of`, is
  outside the rule. The field's own `missing-field` or `invalid-field-value`
  finding is what reports it, so nothing goes unreported.
- The clause narrows only the subject. An object kind with a clause is refused.

See the [chapter-rules guide](../../docs/user-facing/rules.md) for every refusal and the
[fields guide](../../docs/user-facing/fields.md) for scalar fields.

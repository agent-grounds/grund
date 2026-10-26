# Fields: the shape of a kind, checked

This runnable mini-repository demonstrates [§FS-fields](../../docs/functional-spec/FS-fields.md#fs-fields-a-v2-kind-states-the-shape-of-its-declarations-and-check-holds-every-declaration-to-it):
a version-2 `grund.toml` that says what an FS, an AR and a DF look like, and
`grund check` holding every declaration to it.

Run it from the repository root:

```bash
grund check examples/fields/repo
echo $?    # 1: four shape errors, one shape warning
```

The config states three shapes:

- **FS** must open with a `terms` chapter titled `Terms`. `FS-signup` does;
  `FS-login` writes its Terms after `1. Behaviour`, and is told it is section 2
  of 2 when its kind places it first.
- **AR** must open with `placement`, and `terms` comes *directly* after it, as a
  `warn`. `AR-store` is clean. `AR-session` has no placement: that is one
  `missing-field` error, and its `terms` is not also reported out of place,
  because an absent field is skipped and the next one takes its place
  ([§FS-fields.order.absent](../../docs/functional-spec/FS-fields.md#orderabsent-an-absent-member-is-skipped)). `AR-audit` puts `1. Design` between the two,
  which is a warning, because `terms` is `presence = "warn"`.
- **DF** is `closed`, and its one field is a scalar: the `status` chapter's
  display name must be one of `Proposed`, `Accepted` or `Superseded`.
  `DF-cache` says `Accepted`; `DF-hashing` says `Maybe`, and writes a
  `surprise` chapter its kind does not declare.

```text
docs/ar/AR-session.md:1: error: AR-session has no placement field; kind AR declares it; add its placement section
docs/df/DF-hashing.md:3: error: DF-hashing.status is "Maybe"; kind DF admits Proposed, Accepted, Superseded; retitle the heading to one of them
docs/df/DF-hashing.md:7: error: DF-hashing.surprise is not a field of kind DF, which is closed; its fields: status; rename it to one of them, number it, or add it to the kind's fields
docs/fs/FS-login.md:9: error: FS-login.terms is section 2 of 2; kind FS places it first; move the section there
docs/ar/AR-audit.md:13: warning: AR-audit.terms is section 3 of 3; kind AR places it directly after placement; move the section there
```

A field is found by its section name, never by its title, so
`DF-hashing.status` is the same coordinate whatever the status says. Fix the
four errors and the run exits `0` with the standing warning; change `terms` on
AR to `presence = "must"` and that warning becomes an error.

The [fields guide](../../docs/user-facing/fields.md) has every key, the order
rules, the strength table and what the loader refuses.

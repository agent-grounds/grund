# Staged red cases

Each directory here is an e2e case shaped like one under `tests/e2e/cases/` ([§FS-examples.5](../../../docs/functional-spec/FS-examples.md#5-e2e-reuse-without-duplication)),
written ahead of the change that makes it pass. They pin imports ([§FS-imports](../../../docs/functional-spec/FS-imports.md#fs-imports-a-project-cites-a-sibling-project-its-build-places-beside-it-read-live-from-that-tree)), which is
specified ahead of its release: run today, every case fails at load with today's refusal of an
unknown config section, which is the point.

They wait here rather than in `tests/e2e/cases/` because that corpus is the gate: every case
in it must match its goldens on every run, so a case written for behaviour that has not shipped
would fail every commit. Nothing discovers this directory. The ignored runners in
`crates/grund-cli/tests/imports.rs` name every case here, and a test there that runs today
fails if a case is added or removed without a runner naming it, so none is dropped silently.

Run them with:

```sh
cargo test -p grund --test imports -- --ignored
```

The change that ships imports moves each case into `tests/e2e/cases/`, removes the `ignore`
from its runner, and deletes this directory once it is empty.

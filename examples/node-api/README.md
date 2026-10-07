# Node package consumer

This example awaits the shared engine from a locally installed `grund-cli`
package ([§FS-distribution.3.2.4](../../docs/functional-spec/FS-distribution.md#324-acceptance-evidence)).
Build, pack and install the API rehearsal following the
[Node guide](../../docs/user-facing/node-api.md), then copy `example.mjs` into
that consumer and run:

```sh
node example.mjs /absolute/path/to/checkout/tests/e2e/cases/json-report/repo
```

The fixture contains a dangling citation; the awaited check resolves to its
report and show resolves to the known declaration. `expected.stdout` records
those two consumer facts. The binding package test runs this script from a
fresh tarball installation. It is a Node consumer example, while the scheme
examples exercise the CLI directly. No registry installation is claimed.

# Read grounding data in Python

This workflow embeds the engine through the locally installed Python package
([§FS-distribution.3.3.7](../../docs/functional-spec/FS-distribution.md#337-local-source-and-typing-handoff)).
From the repository root:

```sh
python -m pip install crates/grund-py
python examples/python-api/read_fixture.py
```

The [expected output](expected.stdout) captures `dangling 3` and
`warnings: config-version-1, unused`. The script reads the existing json-report
fixture, iterates the errors of its immutable report and shows the declaration's brief body;
the cross-project-citation-without-workspace fixture demonstrates the pathless
caution preceding a located warning ([§FS-errors.4.1](../../docs/functional-spec/FS-errors.md#41-ordering)).
It changes no file. Its coverage lives in the shared binding acceptance corpus,
which also checks native parity and the local-source build contract.
See the [Python API guide](../../docs/user-facing/python-api.md) for defaults,
structured exceptions, threading and explicit mutation options. PyPI publication
remains pending.

# Read grounding data in Python

This workflow embeds the engine through the locally installed Python package
([§FS-distribution.3.3.7](../../docs/functional-spec/FS-distribution.md#337-local-source-and-typing-handoff)).
From the repository root:

```sh
python -m pip install .
python examples/python-api/read_fixture.py
```

Expected output is `dangling 3`. The script reads the existing json-report
fixture, iterates its immutable report and shows the declaration's brief body.
It changes no file. Its coverage lives in the shared binding acceptance corpus,
which also checks native parity and the local-source build contract.
See the [Python API guide](../../docs/user-facing/python-api.md) for defaults,
structured exceptions, threading and explicit mutation options. PyPI publication
remains pending.

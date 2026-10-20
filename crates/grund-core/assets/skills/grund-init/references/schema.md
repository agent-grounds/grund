## Seeing the Schema

Before writing a declaration or a citation, run `grund schema` to see what the repository is held to: its kinds and places, each kind's fields, which units must be grounded, and who must cite whom and how strongly. It reads the configuration alone, so it answers on a tree whose citations are still broken:

```text
api · grund.toml · v1

kinds and places
  FS    kind        docs/
  code  complement  every scanned file no other row claims

citations
  FS    should  FS (itself)
  code  —
```

`--format mermaid` and `--format dot` print the same facts as graph source. `grund config show` is for the effective TOML, every value filled in.

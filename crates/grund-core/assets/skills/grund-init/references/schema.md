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

A kind's fields follow it, one `field` line each. Where a field sets a `title`, its line ends with ``titled `Title` ``: that is the exact text the field's heading must carry, `## <handle>: Title`, and `grund check` refuses a heading that differs by one byte.

`--format mermaid` and `--format dot` print the same facts as graph source. For the TOML, `grund config show` prints what applies, `grund config explain` every setting.

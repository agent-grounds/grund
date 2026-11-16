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

A kind's fields follow it, one `field` line each, and each line is the heading to write, then its presence and placement in `grund check`'s own words:

```text
  FS    kind                  docs/fs/
          field reason        `## reason: Reason for Existence` (must, first)
          field terminology   `## terminology: Terminology` (must, directly after reason)
          field requirements  `## requirements: Requirements` (must, last)
```

The title inside the code span is the exact text the heading must carry, and `grund check` refuses a heading that differs by one byte. `<title>` is a title the field leaves open, and `<one of …>` a scalar whose title is its value. Each heading is shown under a declaration written `# <ID>: …`; a field always sits one level below its declaration's heading, so under `## SWR-001: …` the same field is `### terminology: Terminology`.

`--format mermaid` and `--format dot` print the same facts as graph source. For the TOML, `grund config show` prints what applies, `grund config explain` every setting.
